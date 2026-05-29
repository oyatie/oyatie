---
id: ADR-MEET-0002
status: Accepted
date: 2026-05-17
microservice: meet
deciders: council-architecture, axis-meet, axis-foundry-runtime, ops-sre-reliability, ops-security
owner: axis-meet
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-MEET-0001
  - ADR-MEET-0003
  - ADR-MEET-0006
related_artifacts:
  - microservices/meet/PRD.md
  - microservices/meet/IP-008-recording-pipeline.md
  - microservices/meet/IP-009-transcription-pipeline.md
  - microservices/meet/threat-model.md (T-E-02 ffmpeg CVE escape)
  - microservices/meet/iac/helm/meet/templates/deployment.yaml
purpose: Choose the recording + transcription pipeline substrate and isolation posture; balance per-tenant tenant-DEK-encrypted-recordings + sovereign-residency + GPU-cost + ffmpeg-CVE-blast-radius.
---

# ADR-MEET-0002: Recording + transcription pipeline — ffmpeg under gVisor; Whisper-large default; faster-whisper batch acceleration; opt-in cloud-API alternatives; on-prem option for sovereign tenants

## Status

Accepted — 2026-05-17.

## Context

The meet µservice needs to produce cloud recordings (audio + video + screen-share composites) and post-meeting transcripts + live captions. Two concerns dominate:

1. **Recording integrity + sandbox**: ffmpeg is the de-facto multimedia mux/transcode binary; it parses an enormous variety of container + codec formats; its CVE history includes dozens of severe RCE class issues (CVE-2020-22043, CVE-2021-38114, CVE-2023-49502 among others). A multimedia parser ingesting attacker-influenced media (a participant could craft a malicious screen-share) is a high-CVE-blast-radius surface. Mainstream meeting platforms (Zoom, Google Meet, Teams) sandbox or isolate this surface.

2. **Transcription quality vs cost vs sovereignty**: Whisper (OpenAI) is the OSS state-of-the-art ASR model; Whisper-large is multilingual-60+-languages and best-in-class WER; faster-whisper (CTranslate2) accelerates batch by 4-5×; Whisper.cpp runs on CPU but slowly. Cloud-API alternatives (Google Speech-to-Text, AWS Transcribe, Azure Speech) offer higher accuracy in some languages but cross-border-data-transfer issues for pack-eu/pack-kr/pack-us-healthcare. Per `competitor-parity-matrix.md` Zoom uses proprietary stack + AWS Transcribe fallback; Google Meet uses Google internal; Teams uses Microsoft Azure Speech; Webex uses Webex Assistant.

Three sub-questions:
- (a) which ASR for live captions (real-time, ≤ 500ms p99)?
- (b) which ASR for batch transcripts (post-meeting, ≤ 60s for 60min meeting)?
- (c) how to isolate the ffmpeg recording-mux process from media-parser CVE blast radius?

For E2E mode (per ADR-MEET-0003), recording + transcription are structurally disabled by Cedar deny — neither pipeline ever sees plaintext. This ADR governs the non-E2E default path.

## Decision

meet µservice adopts a **three-layer recording + transcription pipeline**:

1. **Recording mux: ffmpeg 7.x under gVisor sandbox**
   - `oya-meet-recording-adapter-ffmpeg` spawns ffmpeg subprocesses under Kubernetes `runtimeClassName: gvisor`.
   - gVisor provides a user-space kernel that contains syscall-level escape attempts; the blast radius of an ffmpeg CVE is the gVisor sandbox, not the host kernel.
   - Pod additionally configured: `runAsNonRoot: true`, `readOnlyRootFilesystem: true` (except scratch volume), `capabilities: {drop: ["ALL"]}`, `seccompProfile: {type: RuntimeDefault}`.
   - ffmpeg image SHA-pinned; CVE-monitoring on the ffmpeg upstream; emergency-rotate runbook on CVE disclosure.

2. **Live caption ASR: Whisper-medium streaming via faster-whisper (CTranslate2) on GPU**
   - `oya-meet-transcription-adapter-whisper` streaming path uses Whisper-medium (~ 769M params; ≤ 500ms p99 on A10 GPU with VAD-segmented streaming).
   - GPU node selector `nvidia.com/gpu=true`; per-pack reserved GPU pool.
   - Per-language: 60+ languages supported natively.
   - Live caption frames pushed to clients via WebSocket; ≤ 500ms p99 from audio chunk to caption frame.

3. **Batch transcript ASR: Whisper-large via faster-whisper (CTranslate2) on GPU**
   - `oya-meet-transcription-adapter-whisper` batch path uses Whisper-large (~ 1.5B params; real-time-x5 batch throughput on L4 GPU).
   - Post-meeting batch: 60min meeting transcribed in ≤ 60s.
   - Transcript JSON written to S3 (tenant-DEK envelope) + Meilisearch index for search.
   - Per-language: 60+ native; opt-in translation via foundry-runtime post-transcription.

4. **Per-tenant GPU pool segregation**
   - Whisper worker batches are single-tenant by construction; metric `oya_meet_transcription_cross_tenant_batch_total` (target = 0) enforces.
   - This prevents cross-tenant inference-side-channel leaks (a real concern for shared GPU inference).

5. **Recording lifecycle**
   - LiveKit Egress → ffmpeg subprocess (gVisor) → S3 multipart upload with SSE-KMS tenant-DEK envelope encryption + Object Lock (WORM) per SEC Rule 17a-4(f).
   - Postgres recording manifest (room_id, instance_id, recording_id, content_hash, retention_bound).
   - `RecordingFinalized` event emitted to audit-chain with Ed25519 seal.
   - Per-pack retention floors applied (HIPAA 6y; SEC 17a-4 3-7y; MiFID II 5-7y; KR PIPA 1-5y).

6. **Opt-in cloud-API alternatives**
   - For tenants who explicitly opt out of sovereign-residency for transcription (e.g., a pack-us tenant willing to use AWS Transcribe for better latency on certain technical-jargon dialects): a `transcription_provider: aws-transcribe | google-stt | azure-speech | whisper-onprem` configuration is permitted with tenant attestation.
   - Cloud-API providers REFUSED by default for: pack-us-healthcare (BAA conflict), pack-eu (GDPR cross-border), pack-kr (PIPA Art. 28), pack-us-financial (SEC sovereignty), pack-ksa/pack-ae (sovereignty).

7. **On-prem Whisper option for sovereign tenants**
   - For pack-ksa / pack-ae / pack-eu tenants who require strict data-locality even within the cloud-managed pack: a `whisper-onprem` mode runs Whisper on tenant-controlled GPU (BYO Kubernetes nodes). Documented in `iac/` overlays.

## Alternatives Considered

### A. ffmpeg in standard Linux container (no sandbox)
- Pros: simple; lower overhead.
- Cons: ffmpeg CVE blast radius reaches host kernel; participant-crafted media is attacker-influenced input; recent ffmpeg CVE history shows this is not theoretical.
- Rejected: blast radius unacceptable for a media-parser surface.

### B. ffmpeg in Kata Containers (VM-based sandbox)
- Pros: stronger isolation than gVisor (full VM).
- Cons: 5-10× resource overhead per pod; slower start (multi-second); operator burden in OKE today.
- Rejected: cost + start-latency unacceptable for per-meeting recording-worker scale; gVisor is the better cost-isolation tradeoff for this workload.

### C. ffmpeg in Firecracker microVM
- Pros: very strong isolation; fast start.
- Cons: not natively supported in OKE; operator burden of Firecracker integration.
- Rejected: ops cost; gVisor is the supported pattern.

### D. Whisper-large for BOTH live captions and batch
- Pros: best WER; uniform pipeline.
- Cons: Whisper-large streaming on GPU is real-time-x0.5 at best (Whisper-large was designed for batch, not streaming); cannot hit ≤ 500ms p99 live-caption budget.
- Rejected: violates performance NFR.

### E. Whisper-tiny for live captions
- Pros: fastest; cheapest GPU.
- Cons: -7 BLEU vs Whisper-medium on multilingual; not enterprise-tier quality.
- Rejected: quality bar.

### F. Cloud-API ASR (AWS Transcribe / Google STT / Azure Speech) as default
- Pros: zero GPU operator burden; possibly higher accuracy on certain dialects.
- Cons: cross-border data flow (defeats tenant residency); BAA gaps for pack-us-healthcare; per-minute pricing higher than self-hosted Whisper at oyatie scale; vendor coupling.
- Rejected as default; permitted as opt-in for tenants who explicitly accept the trade-off.

### G. Self-host on CPU only (Whisper.cpp)
- Pros: no GPU operator burden; lower hardware cost.
- Cons: Whisper.cpp CPU is 10-50× slower than GPU; cannot meet live-caption p99 budget even with Whisper-medium; batch is acceptable for very small tenants.
- Rejected as primary; permitted for batch-only single-cell minimal deployments.

### H. ffmpeg replaced by Rust-native mux library (e.g., re_mp4, symphonia)
- Pros: no ffmpeg CVE blast radius; Rust safety.
- Cons: feature gap — Rust-native libraries do not yet match ffmpeg's container/codec coverage for screen-share + simulcast composite + recording-format diversity; would re-introduce CVE risk in our own code.
- Rejected for now; revisit when Rust ecosystem matures.

## Consequences

### Positive

- ffmpeg CVE blast radius contained to gVisor sandbox; host kernel + other workloads protected.
- Live caption p99 ≤ 500ms achieved via Whisper-medium + faster-whisper + GPU pool.
- Batch transcript p95 ≤ 60s for 60min meeting via Whisper-large + faster-whisper batch.
- Per-tenant GPU pool segregation prevents inference-side-channel leakage.
- Recording WORM + tenant-DEK envelope satisfies SEC 17a-4(f), MiFID II, HIPAA, KR PIPA retention + integrity requirements.
- Sovereign-residency preserved by default (Whisper on oyatie GPU pool; not cloud-API).

### Negative

- gVisor overhead: ~ 10-20 % CPU overhead vs runc on ffmpeg workload; mitigated by HPA + acceptable trade-off for sandbox.
- GPU pool cost: significant ongoing OPEX (per `cost-budget.md`); mitigated by faster-whisper batching + Whisper-medium for live; per-pack reserved GPU capacity tuned to peak.
- faster-whisper depends on CTranslate2; pinned LTS; CVE-monitoring needed.
- Operator burden of dual model versions (Whisper-medium streaming + Whisper-large batch); mitigated by single `-adapter-whisper` crate with two code paths.
- On-prem Whisper option adds BYO-GPU operator complexity; mitigated by IaC overlays documented in `iac/`.

### Operational

- IaC `iac/helm/meet/templates/recording-worker-deployment.yaml` declares `runtimeClassName: gvisor`; cloud-k8s confirms gVisor RuntimeClass installed in target clusters.
- IaC `iac/helm/meet/templates/transcription-worker-deployment.yaml` declares `nodeSelector: nvidia.com/gpu=true`; cloud-k8s provisions GPU nodes per pack.
- Whisper model weights stored as ReadOnly PV (pre-baked); model loading on cold start ≤ 30s.
- Dashboards: meet recording-pipeline + ai-features-quality dashboards include G.107 MOS + WER + GPU-pool-depth panels.
- Runbook `runbooks/recording-storage-degraded.md` covers S3 outage + local-disk-buffer fallback (≤ 1h capacity).
- Runbook `runbooks/transcription-classifier-rollback.md` covers Whisper model rollback procedure.

### Regulatory

- **SEC Rule 17a-4(f) WORM**: S3 Object Lock + content_hash sealed; satisfies tamper-evident retention.
- **FINRA Rule 4511**: supervisory review path via four-eyes recording disclosure.
- **HIPAA §164.312(c)(1)** integrity: content-hash + audit-chain Ed25519.
- **HIPAA §164.502(b)** minimum-necessary: transcript redactor per `policy/redaction-phi.md` for pack-us-healthcare.
- **MiFID II RTS 6** tamper-evident: audit-chain Ed25519.
- **KR PIPA Art. 29** technical safeguards: every mitigation listed contributes; KR PIPA Art. 21 retention floor enforced.
- **GDPR Art. 25 + Art. 32**: privacy-by-design via tenant-DEK envelope + per-pack residency; appropriate technical measures via gVisor + sandbox + audit-chain.
- **EU AI Act Art. 13 + Art. 50** transparency: transcription labelled AI-generated per ADR-MEET-0006.

## References

- ffmpeg upstream — `ffmpeg.org`
- ffmpeg CVE history — `cve.mitre.org` (search ffmpeg)
- gVisor — `gvisor.dev` (paper: "The True Cost of Containing: A gVisor Case Study", Bosamiya et al. 2020)
- Kata Containers — `katacontainers.io`
- Firecracker — `firecracker-microvm.github.io`
- OpenAI Whisper paper — Radford et al., "Robust Speech Recognition via Large-Scale Weak Supervision" `arxiv.org/abs/2212.04356`
- faster-whisper — `github.com/SYSTRAN/faster-whisper`
- Whisper.cpp — `github.com/ggerganov/whisper.cpp`
- CTranslate2 — `github.com/OpenNMT/CTranslate2`
- LibriSpeech / CommonVoice WER benchmarks
- AWS Transcribe / Google STT / Azure Speech docs (alternatives)
- LiveKit Egress — `docs.livekit.io/realtime/egress/`
- SEC Rule 17a-4(f) — `sec.gov/files/rules/final/2022/34-96034.pdf`
- FINRA Rule 4511 — `finra.org/rules-guidance/rulebooks/finra-rules/4511`
- HIPAA 45 CFR §164.312
- MiFID II RTS 6
- ADR-0131; ADR-0132; ADR-MEET-0001; ADR-MEET-0003; ADR-MEET-0006
