---
id: ADR-SHORTS-0001
status: Accepted
date: 2026-05-17
microservice: shorts
deciders: council-architecture, ops-security, axis-shorts, cloud-k8s, cloud-secrets, ops-sre-reliability, ops-finops
owner: axis-shorts
supersedes: []
superseded_by: []
related:
  - ADR-0117
  - ADR-0135
  - ADR-0131
  - ADR-SHORTS-0004
  - ADR-SHORTS-0006
related_artifacts:
  - microservices/shorts/PRD.md (§Performance NFR)
  - microservices/shorts/threat-model.md (T-D-02, T-E-05)
  - microservices/shorts/iac/helm/shorts/values.yaml
  - microservices/shorts/runbooks/transcode-queue-backup.md
  - microservices/shorts/slos/transcode-throughput.openslo.yaml
purpose: Establish the video-transcode pipeline (codec/format ladder, sandboxing, CDN) for the shorts µservice.
---

# ADR-SHORTS-0001: Video transcode pipeline — ffmpeg 7.x LTS multi-bitrate HLS+DASH ABR ladder in gVisor sandbox; Cloudflare R2 + Workers CDN

## Status

Accepted — 2026-05-17.

## Context

shorts ingests short videos (≤ 60s, ≤ 500MB) and must publish them for playback within p95 ≤ 30s for the 60s-source-video case. The transcode pipeline produces a multi-bitrate ABR ladder (HLS + DASH) so that diverse client devices and network conditions can stream effectively.

Industry leaders ship multi-bitrate ladders (TikTok / Reels / YouTube Shorts), but the choice of codecs, encoders, sandboxing, and CDN substrate is technically load-bearing and security-load-bearing. ffmpeg + Chromaprint historically carry RCE-class CVEs at non-trivial rate; sandboxed worker isolation is essential per threat-model T-E-05.

The decision needs to cover (a) codec ladder (H.264 / H.265 / AV1 / AAC / Opus), (b) container format (CMAF vs separate HLS/DASH stack), (c) transcode tool (ffmpeg vs alternative), (d) sandboxing approach (gVisor / Kata / runc-only), (e) CDN substrate (Cloudflare R2 + Workers vs AWS CloudFront vs OCI CDN vs in-house), (f) per-pack residency rules.

PRD §Performance NFR target: 60s video p95 ≤ 30s transcode (5 bitrate rungs). PRD §Tenant Outcome 1 specifies hyperscaler-grade latency competitive with TikTok / Reels / Shorts. PRD §Multi-pack residency specifies CDN POP-presence per pack.

EU AVMSD 2018/1808 Art. 28b(2): video-sharing-platform minor-protection obligations; technical safeguard layer.

OCI primary cloud per ADR-0117; CDN substrate may be a complementary vendor.

## Decision

oyatie shorts adopts the **following pipeline**:

1. **Transcode tool: ffmpeg 7.x LTS** — open-source, industry-standard, broadly tested, codec coverage across H.264/H.265/AV1/AAC/Opus.
2. **Sandbox: gVisor primary; Kata Container fallback** — ffmpeg worker containers run under gVisor runtimeclass (kernel-syscall interception); for high-throughput workers needing direct kernel performance, Kata Container is the fallback.
3. **Ladder: 5 rungs** —
   - 360p (500 kbps H.264 baseline + AAC 96 kbps)
   - 480p (1.2 Mbps H.264 main + AAC 96 kbps)
   - 720p (2.5 Mbps H.264 high + AAC 128 kbps)
   - 1080p (4 Mbps H.265 main + AAC 128 kbps + Opus 64 kbps fallback)
   - 1440p (6 Mbps AV1 + Opus 96 kbps) — optional; auto-disabled on low-bandwidth packs
4. **Container format: CMAF (ISO/IEC 23000-19)** — single segment format for both HLS and DASH; reduces storage 50% vs separate stacks.
5. **Manifest emission: HLS (RFC 8216) + MPEG-DASH (ISO/IEC 23009-1)** — both formats from CMAF segments; serves all device classes.
6. **CDN: Cloudflare R2 + Workers (primary tier)** — zero-egress pricing model; global POP network; Workers for edge logic + signed-URL evaluation; pairs with OCI Object Storage as origin.
7. **Worker pool autoscale: KEDA queue-depth-based** — min 16, max 1000 workers at XS tier; scales with capacity tier.
8. **Per-pack residency**: transcode workers + S3 buckets + CDN POPs all within pack region; CDN cross-POP cache propagation only for in-pack content.
9. **Signed-URL TTL ≤ 15min** for video fetches; signed-manifest also Ed25519-signed at write time (threat-model T-T-02).
10. **Backfill path for codec upgrade** (e.g., adopting AV1 at scale): per `backfill-replay.md` BF-01.

## Alternatives Considered

### A. AWS Elemental MediaConvert direct (managed transcode service)

- Pros: managed; ABR ladder built-in; lower operational overhead; battle-tested.
- Cons: vendor lock-in; per-job pricing significantly higher than self-hosted ffmpeg at scale (~$0.015/min vs ~$0.005/min ffmpeg amortised); ADR-0117 primary cloud is OCI not AWS.
- Rejected: vendor lock-in + pricing + cross-cloud complexity.

### B. Bento4 + Shaka Packager (alternative open-source toolchain)

- Pros: well-engineered packagers; CMAF + HLS + DASH support; smaller surface than ffmpeg.
- Cons: doesn't do encoding (only packaging); still need ffmpeg or similar for encode; adds dependency without removing one; less codec coverage out-of-box.
- Rejected: doesn't replace ffmpeg; adds complexity without simplification.

### C. GStreamer pipeline

- Pros: composable; Rust bindings available; lower CVE history than ffmpeg historically.
- Cons: codec coverage narrower for AV1; community smaller; documentation thinner for video-on-demand encode at scale.
- Rejected (for primary): retained as future evaluation if ffmpeg CVE cadence becomes prohibitive.

### D. In-house Rust-native encoder

- Pros: minimal supply-chain surface; type-safety throughout.
- Cons: building production-grade H.264/H.265/AV1 encoder is years of effort; bug-bug compatibility risk; codec edge-cases.
- Rejected: build-vs-buy obviously buy; no engineering value in reinventing this wheel.

### E. AWS CloudFront for CDN (instead of Cloudflare R2 + Workers)

- Pros: tight integration with AWS S3 origin; established platform.
- Cons: pricing model (egress dominant); ADR-0117 primary cloud is OCI; cross-cloud complexity; less aggressive global POP network than Cloudflare.
- Rejected: pricing + ADR-0117 alignment.

### F. OCI CDN exclusively (no Cloudflare)

- Pros: single-cloud alignment with ADR-0117; lower cross-vendor complexity.
- Cons: smaller POP footprint than Cloudflare globally; pricing higher per-egress than Cloudflare R2's zero-egress model; less mature edge-Worker capability for signed-URL evaluation.
- Rejected (for primary): OCI CDN retained as in-pack origin fallback when Cloudflare R2 unavailable.

### G. runc-only sandbox (no gVisor / Kata)

- Pros: lowest overhead; standard k8s sandbox.
- Cons: shared-kernel attack surface; ffmpeg CVE history makes this unacceptable for transcode workers; threat-model T-E-05 explicitly mitigates via sandbox.
- Rejected: security floor.

## Consequences

### Positive

- ffmpeg 7.x LTS is industry-standard; broadest codec coverage; lowest cost per transcode at scale.
- gVisor sandbox contains ffmpeg CVE blast-radius; threat-model T-E-05 mitigated by design.
- CMAF unifies HLS + DASH; 50% storage savings vs separate format stacks.
- Cloudflare R2 + Workers: zero-egress pricing; massive cost-savings for video delivery at scale (~$0 egress vs ~$1500/mo OCI egress per pack at XS tier).
- 5-rung ladder covers 360p–1440p; supports all common device classes.
- AV1 at 1440p tier achieves ~30% storage savings vs H.265; future-proofs as AV1 hardware decode adoption grows.
- KEDA queue-depth autoscale handles bursty upload patterns (celebrity events).
- Per-pack residency built-in.
- Backfill path defined for codec upgrades.

### Negative

- ffmpeg CVE cadence requires weekly Trivy + Grype scans + LTS-pin maintenance (capacity-model ops cost).
- gVisor overhead ~15-20% vs runc; offset by sandbox safety; acceptable for the workload.
- Cloudflare vendor dependency for primary CDN; OCI CDN as in-pack fallback.
- AV1 encode CPU-intensive at 1440p; budget revision if encoder cost exceeds expected.
- KEDA autoscale latency ~30-60s; bursty patterns may queue briefly.

### Operational

- Worker pool runs under gVisor runtimeclass: `runtimeClassName: gvisor` in pod spec.
- ffmpeg LTS pin recorded in `iac/helm/shorts/values.yaml` (currently 7.1).
- Per-worker CVE scan via Trivy + Grype weekly; SBOM signed via sigstore per ADR-0140 (retired per ADR-0145).
- CDN failover: in-pack S3 origin as primary fallback if Cloudflare R2 outage; per `runbooks/cdn-cache-invalidation-cascade.md`.
- Backfill operations (BF-01): off-peak windowed; per-tenant rate-limited.
- KEDA queue-depth metric exported per pack: `oya_shorts_transcode_queue_depth{pack=X}`.

### Regulatory

- **GDPR Art. 32 (security of processing)**: gVisor sandbox + signed manifests + KMS-encrypted S3 satisfy "appropriate technical measures".
- **EU AVMSD Art. 28b(2) (minor protection)**: per `policy/tenant-scope.cedar` + `age-gate` BC, mature/adult content age-classified at upload; technical floor below moderation classifier.
- **EU DSA Art. 17 (Statement of Reasons)**: per-takedown manifest invalidation traced via `runbooks/cdn-cache-invalidation-cascade.md`.
- **DMCA §512(c)**: blob removal on takedown propagates via CDN cache invalidation; signed-URL TTL ≤ 15 min ensures stale URL expiry.
- **Pack residency (GDPR Arts. 44-50; KR PIPA Art. 28; LGPD Arts. 33-46)**: per-pack CDN POPs + per-pack S3 buckets enforce; LEAN-lane `oya-check-pack-residency` validates.

## References

- ADR-0117 single-cloud-substrate (OCI primary).
- Parallel ADR-0135 dual-context.
- ADR-0131 per-microservice flat layout.
- ADR-SHORTS-0004 (DRM substrate; HLS/DASH manifest is encrypt-substrate).
- ADR-SHORTS-0006 (age-classification gating; mature/adult never anonymous-readable).
- ffmpeg 7.x release notes; LTS-tracking `ffmpeg.org`.
- gVisor docs `gvisor.dev`; Kata Container docs `katacontainers.io`.
- HLS RFC 8216 + 8217; MPEG-DASH ISO/IEC 23009-1; CMAF ISO/IEC 23000-19.
- H.264/AVC ISO/IEC 14496-10; H.265/HEVC ISO/IEC 23008-2; AV1 AOMedia.
- AAC ISO/IEC 14496-3; Opus RFC 6716.
- Cloudflare R2 docs `developers.cloudflare.com/r2`; Cloudflare Workers docs.
- OCI Object Storage docs.
- KEDA docs `keda.sh`.
- Trivy + Grype scanners.
- sigstore / SLSA L3 supply-chain framework.
- `microservices/shorts/PRD.md` §Performance NFR.
- `microservices/shorts/threat-model.md` T-D-02, T-E-05, T-T-02.
- `microservices/shorts/runbooks/transcode-queue-backup.md`.
- `microservices/shorts/runbooks/cdn-cache-invalidation-cascade.md`.
- `microservices/shorts/slos/transcode-throughput.openslo.yaml`.
- `microservices/shorts/slos/video-start-latency.openslo.yaml`.
