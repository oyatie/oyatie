---
id: ADR-RECORDINGS-0001
status: Accepted
date: 2026-05-17
microservice: recordings
deciders: axis-recordings, axis-foundry-runtime, council-architecture, council-privacy
owner: axis-recordings
supersedes: []
superseded_by: []
related: [ADR-0131, ADR-0132, ADR-0133, ADR-RECORDINGS-0003, ADR-RECORDINGS-0006]
related_artifacts:
  - microservices/recordings/PRD.md (Open Question 1 — Whisper-large vs medium; FR-04 transcript)
  - microservices/recordings/PHASE-01-RECORDINGS-FOUNDATION.md
  - microservices/recordings/capabilities/T1-assist.yaml
  - microservices/recordings/runbooks/transcript-pipeline-degraded-whisper.md
  - microservices/recordings/slos/transcript-search-latency.openslo.yaml
purpose: |
  Close PRD-recordings Open Question 1 (Whisper-large vs Whisper-medium per
  pack) and Open Question 4 (diarization engine). Fix the canonical
  transcription + speaker-diarization pipeline for every recording.
---

# ADR-RECORDINGS-0001: Transcription + diarization pipeline — Whisper-large default; pyannote 3.x diarization; Whisper-medium fallback under load

## Status

Accepted — 2026-05-17.

## Context

PRD-recordings FR-04 mandates speaker-diarised transcripts with timestamps
+ per-segment confidence as a first-class capability. The PRD lists Whisper
+ pyannote in Layer-A substrate but does not fix specific models, versions,
or fallback policy. Recordings is also the centralised store for meet +
messenger huddles + live-broadcast + manual uploads — accuracy and latency
budgets must hold across all source kinds and pack overlays.

Open question candidates:

- **Whisper-large-v3** (1.55 B params, 99 languages, ~5.5 % WER English avg,
  ~7-9 % WER on KR/JP/ES/FR/DE) — best accuracy; 10 GB GPU memory; ~6 min
  wall-time per 60 min audio on A10/L4.
- **Whisper-medium** (769 M params, ~7 % WER English avg, ~10-13 % WER on
  KR/JP) — 2× faster; 5 GB GPU memory; useful as queue-pressure fallback.
- **Whisper-small** (244 M params, ~10 % WER English avg) — too inaccurate
  for hero-product positioning; rejected.
- **AssemblyAI** / **Deepgram** / **Speechmatics** / **Rev AI** — managed
  APIs with strong accuracy; license + data-residency posture forbids them
  for pack-eu / pack-kr / pack-us-healthcare unless DPA / BAA signed; cost
  is 2-5× Whisper-large at scale.
- **Speech-to-Text from major cloud (Azure, GCP, AWS)** — locks the data
  plane to a single cloud; tenant-tier-cost concerns at oyatie's hyperscaler-
  comparator scale; data-residency posture varies per cloud.

Diarization candidates:

- **pyannote.audio 3.x** (open-weights speaker-diarization toolkit;
  state-of-the-art DER ~12 % on multi-speaker CALLHOME + VoxConverse
  benchmarks; Apache-2 license).
- **NeMo Speaker Diarization** (NVIDIA; competitive accuracy; CUDA-locked;
  Apache-2).
- **Speaker-id from major cloud** — same data-residency concern.
- **In-house diarization (build)** — > 18-month roadmap; rejected for
  hero-product launch.

## Decision

oyatie recordings ships a **two-component foundry-runtime-hosted pipeline**:

1. **Transcription default: Whisper-large-v3** (open-weights, pinned model
   version). Adapter:
   `oya-recordings-transcript-adapter-whisper` invokes via foundry-runtime
   `cap:foundry-runtime:whisper-large:v3:v1`.
2. **Diarization: pyannote.audio 3.1.x** (open-weights, pinned model).
   Adapter: `oya-recordings-transcript-adapter-pyannote` invokes via
   foundry-runtime `cap:foundry-runtime:pyannote:v3.1:v1`.
3. **Fallback: Whisper-medium** under sustained queue pressure (queue depth
   > 60 min). Activated automatically per
   `runbooks/transcript-pipeline-degraded-whisper.md`. Per-recording
   `model_version` field records which model produced the transcript;
   backfill workflow re-transcribes with Whisper-large when capacity allows.
4. **Speaker-cluster threshold tuning**: pyannote-default `min_cluster_size=10`
   speech frames; per-recording configurable per tenant override; KR/JP pack
   defaults to `min_cluster_size=15` to reduce over-segmentation common with
   tonal languages.
5. **Both engines run in foundry-runtime gVisor sandbox**: no host process
   privilege; sandbox restart cadence 4h.
6. **Open-weights only, no fine-tuning on tenant data**: model bytes are
   shipped via foundry-runtime model registry; fine-tuning is not enabled
   to avoid GDPR Art. 22 + EU AI Act Annex III concerns.
7. **Per-pack consent gate**: transcription refused if `consent_banner_confirmed`
   is false (per KR 통신비밀보호법, TIA Act, ePrivacy Art. 5(3)).
8. **EU AI Act Art. 50 transparency**: transcript output labelled
   `ai-generated; model=whisper-large-v3; diarization=pyannote-3.1`.

## Alternatives Considered

### A. Whisper-medium as default (cheaper)

- Pros: 2× faster; 40 % cheaper GPU cost; lighter foundry-runtime load.
- Cons: 1.5-2 percentage-points worse WER on average; 3-4 points worse on
  non-English; hero-product positioning suffers; auto-redact PII downstream
  is less reliable on lower-accuracy transcript.
- Rejected as default; accepted as fallback for queue-pressure case where
  the alternative is no transcript.

### B. AssemblyAI (managed API) as default

- Pros: best-in-class accuracy on English; turnkey ops.
- Cons: per-pack residency posture forbids unless DPA on file per tenant;
  cost is 3-5× Whisper-large at scale; binds the data plane to a third party
  that is hard to swap out subsequent-to-launch.
- Rejected for default; remains an option as `oya-recordings-transcript-
  adapter-assemblyai` if a regulated tenant requires managed-API attestation.

### C. Deepgram (managed API) as default

- Pros: low-latency real-time; mature.
- Cons: same residency + cost concerns as A.
- Rejected; same reasoning.

### D. NeMo Speaker Diarization

- Pros: NVIDIA-supported; competitive accuracy.
- Cons: CUDA-locked; harder to run in foundry-runtime gVisor sandbox; smaller
  community than pyannote.
- Rejected for default; remains optional adapter.

### E. Whisper-small + heuristic diarization

- Pros: cheapest.
- Cons: hero-product positioning destroyed by inaccuracy; auto-redact PII
  recall craters; rejected on quality grounds.

### F. Build in-house transcription model

- Pros: full control.
- Cons: 18+ month roadmap; competes with hero-product launch; rejected.

## Consequences

### Positive

- Best-in-class open-weights accuracy at hero-product cost-efficiency.
- foundry-runtime gVisor sandbox isolates the inference path; no host-
  process compromise risk.
- Open-weights posture aligns with `feedback_no_silent_regression` — model
  bytes are reviewable and reproducible.
- Apache-2 / MIT-class licenses on both models avoid license posture
  concerns at pack-level legal review.
- Fallback path keeps the pipeline live under queue pressure.

### Negative

- Two engines to operate (Whisper + pyannote); two model upgrades per
  quarter.
- Whisper-large GPU cost is the largest single line-item in the recordings
  cost-budget; cost-management depends on aggressive cache + Whisper-medium
  fallback under burst.
- Per-pack consent-gate adds friction at ingest; mitigated by producer-side
  banner.

### Operational

- Cargo workspace adds `oya-recordings-transcript-adapter-whisper` +
  `oya-recordings-transcript-adapter-pyannote`.
- IaC: foundry-runtime model-registry pins (whisper-large-v3,
  pyannote-3.1.x); GPU pool sizing per `capacity-model.md`.
- CI: contract tests run against both engines on a golden multilingual set.
- Per ADR-0130 SLO-gated promotion: `transcript-search-latency.openslo.yaml`
  + future `transcription-throughput.openslo.yaml` (Phase-02) gate releases.

### Regulatory

- **EU AI Act Art. 50** transparency — every transcript output is labelled
  `ai-generated; model=whisper-large-v3`.
- **EU AI Act Annex III §4(a) (employment)** — if transcript is used in
  employment context, falls under high-risk; handled by `ADR-RECORDINGS-0006`.
- **GDPR Art. 9(1)** (biometric) — diarization with speaker-name binding
  requires explicit consent; default emits cluster labels only.
- **HIPAA 45 CFR §164.502** — PHI-aware Whisper post-processing for pack-us-
  healthcare; mandatory PII redaction at transcription time per
  `ADR-RECORDINGS-0003`.
- **KR 통신비밀보호법** — producer-side `consent_banner_confirmed` required.
- **License posture**: both engines Apache-2 / MIT-class.

## References

- OpenAI Whisper paper: Radford et al., 2022 — "Robust Speech Recognition
  via Large-Scale Weak Supervision" (https://arxiv.org/abs/2212.04356).
- pyannote.audio: Bredin et al., 2020 — "pyannote.audio: neural building
  blocks for speaker diarization" (https://arxiv.org/abs/1911.01255) +
  pyannote-3.0 model card on Hugging Face.
- LibriSpeech, VoxConverse, CALLHOME benchmarks.
- AssemblyAI documentation (`assemblyai.com/docs`), Deepgram documentation
  (`developers.deepgram.com`), Speechmatics documentation, Rev AI docs.
- NVIDIA NeMo Speaker Diarization (`docs.nvidia.com/nemo`).
- ADR-0131 — per-µservice flat layout.
- ADR-0132 — no-suite forward-policy.
- ADR-0133 — industry best-practice.
- ADR-RECORDINGS-0003 — redaction policy.
- ADR-RECORDINGS-0006 — AI feature bounds.
- microservices/recordings/PRD.md FR-04, Open Question 1.
- microservices/recordings/runbooks/transcript-pipeline-degraded-whisper.md.
- foundry-runtime model-registry pinning surface.
- EU AI Act Arts. 13/27/50/Annex III.
- HIPAA 45 CFR §164.502.
- KR 통신비밀보호법.
- GDPR Art. 9(1).
