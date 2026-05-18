---
id: ADR-SOC-0006
status: Accepted
date: 2026-05-17
microservice: social
deciders: council-architecture, ops-security, axis-social, ops-sre-reliability, cloud-secrets
owner: axis-social
supersedes: []
superseded_by: []
related:
  - ADR-0105
  - ADR-0117
  - ADR-0126
  - ADR-0131
related_artifacts:
  - microservices/social/PRD.md (§"Performance" + §"Bounded Contexts" post-composition + §"Functional Requirements" FR-19 alt-text)
  - microservices/social/threat-model.md (T-T-02, T-E-04, T-E-05, T-I-02)
  - microservices/social/policy/data-residency.md
  - microservices/social/cost-budget.md (§Media transcode + CDN line items)
  - microservices/social/runbooks/federation-bridge-degraded.md (Personal-tier never federates media)
purpose: Define the media transcode and storage stack for the social µservice — ImageMagick 7.1 LTS for images; ffmpeg 7.x LTS for HLS video; S3 with per-tenant prefix + KMS SSE; CDN (Cloudflare R2 / OCI Object Storage); per-pack data-residency.
---

# ADR-SOC-0006: Media transcode + storage — ImageMagick 7.1 LTS for image variants; ffmpeg 7.x LTS for HLS video; S3 per-tenant prefix + KMS SSE; CDN tier; per-pack data-residency

## Status

Accepted — 2026-05-17.

## Context

The social µservice's `post-composition` BC supports media-rich posts: images (avatar, header, post media), short videos (≤ 200MB) with HLS streaming variants. PRD §"Performance" sets transcode targets:

- Image (≤ 10MB) p95 ≤ 2s; p99 ≤ 4s.
- Video (≤ 200MB) HLS segmented p95 ≤ 90s; p99 ≤ 180s.

PRD §"Functional Requirements" FR-19 requires alt-text for accessibility (WCAG 2.2 Level AA compliance); the `capabilities/T1-assist.yaml` auto-alt-text capability uses OCR via ImageMagick.

Threat-model rows T-E-05 (ImageMagick / ffmpeg CVE allows RCE in transcode worker), T-T-02 (media blob tampering in S3), T-E-04 (media scanner bypass), and T-I-02 (PHI leak in media for pack-us-healthcare) drive the security posture.

Industry decisions to draw from:

- **ImageMagick** is the most widely-used open-source image transcoder; 7.x is the current LTS branch (7.1 stable; security-supported through 2027+). Notable historical CVEs include ImageTragick (CVE-2016-3714); current LTS includes mitigation policies.
- **GraphicsMagick** (an ImageMagick fork) is leaner but smaller ecosystem. Picture-rendering quality and codec coverage favour ImageMagick.
- **libvips** is a memory-efficient alternative; growing adoption (Shopify, Cloudflare). Faster for simple resize; less feature complete.
- **ffmpeg 7.x** is the de-facto open-source video transcoder; HLS (RFC 8216) is the canonical streaming format for social-tier video (Twitter / X / Instagram / TikTok all use HLS).
- **WebP / AVIF** are modern image codecs offering 25-50% size reduction vs JPEG/PNG; supported by all modern browsers; encoders shipped in ImageMagick 7.1.

S3 storage options:

- **OCI Object Storage** is the default oyatie cloud (per cloud-iac substrate).
- **Cloudflare R2** offers S3-compatible API with zero egress fees; oyatie uses R2 + Cloudflare Workers as CDN tier per `cost-budget.md`.
- Other clouds (AWS S3, GCS) are out-of-scope per ADR-0117 single-cloud-substrate.

CDN tier:

- Cloudflare R2 + Workers (most cost-effective for public media; zero egress).
- OCI Object Storage + CDN (alternative; lower complexity, higher egress cost).

Per-pack data-residency (per `policy/data-residency.md`): media stays in the pack's region. Cross-pack media URL sharing forbidden by default; federation peer fetches via oyatie-signed CDN URL within source pack's CDN POP (no blob copy).

Sandbox requirement (per T-E-05): media transcode workers run in gVisor / Kata Container sandbox; non-root; read-only root FS; no network egress except to S3 quarantine + production buckets.

The decision needs to (a) pick image + video transcoders, (b) pick the storage + CDN substrate, (c) define security posture (sandbox, scan path, integrity check), (d) bound per-pack data-residency for media, (e) define alt-text + accessibility path, (f) coordinate with `content-moderation` BC's media-scan path (CVE-prone codecs + malware-scan-first pattern).

## Decision

oyatie social adopts a **single canonical media stack** for P01:

1. **Image transcoder: ImageMagick 7.1 LTS** in `oya-social-post-composition-adapter-imagemagick`.
   - Variants: thumbnail (128×128 WebP), small (480px WebP/AVIF), medium (1024px WebP/AVIF), large (2048px WebP/AVIF).
   - Output formats: WebP primary (90% size reduction vs PNG; browser-wide support); AVIF secondary (where higher compression beneficial; Chrome / Firefox / Safari support).
   - Original retained for archival (Professional-tier WORM compliance per HIPAA / SEC 17a-4).
   - OCR for alt-text auto-draft (capability T1-assist).
2. **Video transcoder: ffmpeg 7.x LTS** in `oya-social-post-composition-adapter-ffmpeg`.
   - Output: HLS (RFC 8216) with three quality tiers (hls_low 360p 800kbps, hls_med 720p 2500kbps, hls_high 1080p 5000kbps).
   - Segment duration: 6s (standard).
   - Audio codec: AAC; video codec: H.264 (broadest device support) + AV1 (where CPU + device support allows for better compression).
   - Captions: WebVTT (RFC 9420) for accessibility (WCAG 2.2 Level AA).
3. **Storage: S3-compatible (OCI Object Storage primary) + Cloudflare R2 CDN tier.**
   - Hot tier (≤ 30d): OCI Object Storage with per-tenant prefix isolation.
   - Cold tier (> 30d): OCI Object Storage Archive class; per-pack residency.
   - CDN: Cloudflare R2 + Workers for public media delivery (cost-optimised per `cost-budget.md`); R2 buckets per pack.
   - SSE-KMS with tenant-DEK envelope encryption (per Bominal ADR-0111).
   - Object Lock (WORM) on Professional-tier for HIPAA §164.530(j) + SEC 17a-4(f) retention.
4. **Sandbox: gVisor (or Kata Container) for transcode workers** per T-E-05 mitigation.
   - `runtimeClassName: gvisor` declared in deployment manifest.
   - Non-root user (UID 65534); read-only root filesystem.
   - No network egress except S3 quarantine + production buckets.
   - Weekly CVE scan via Trivy + Grype; LTS pin tracking.
5. **Scan-first lifecycle** per content-moderation BC integration:
   - Upload → quarantine bucket (read-only by transcode workers; write-only by upload endpoint).
   - Scanner (OPSWAT / ClamAV) verdict required before transcode.
   - Clean blob → copy to production bucket; quarantine blob deleted.
   - Infected blob → blob retained in quarantine for forensics; sender notified; tenant security-admin notified.
6. **Integrity check** per T-T-02 mitigation:
   - Every media blob carries `digest_sha256` recorded in Postgres.
   - Fetch path verifies digest; mismatch triggers quarantine + Sev-2 alert.
7. **Signed-URL TTL ≤ 15min** for media access:
   - Per-fetch Cedar re-evaluation.
   - Public-visibility posts use Cedar-checked CDN URL (no signing).
   - Private-visibility posts require signed URL per fetch.
8. **Per-pack data-residency:**
   - Media inherits parent post's `context_kind` and pack.
   - Cross-pack media URL sharing forbidden by default.
   - Federation peer fetches via oyatie-signed CDN URL within source pack's CDN POP (no blob crosses pack boundary).
9. **Pack-us-healthcare specifics** per T-I-02 mitigation:
   - PHI redactor scans OCR output on media before publication.
   - Auto-preview disabled by default; tenant-admin enables per-channel only.
   - HIPAA §164.514 safe-harbor de-identification.

## Alternatives Considered

### A. libvips for images (instead of ImageMagick)

- Pros: ~5x faster for simple resize; lower memory; growing industry adoption.
- Cons: smaller ecosystem; less codec coverage; no OCR (would need separate Tesseract integration); team familiarity favours ImageMagick.
- Rejected (for P01); kept as future option if image-transcode latency becomes bottleneck.

### B. GraphicsMagick for images

- Pros: ImageMagick fork; arguably leaner.
- Cons: significantly smaller ecosystem; fewer security updates; less feature complete; team familiarity favours ImageMagick.
- Rejected.

### C. GStreamer for video (instead of ffmpeg)

- Pros: GObject-based; pluggable pipeline architecture.
- Cons: smaller adoption for social-video transcode; ffmpeg is the industry default; GStreamer pipeline complexity exceeds the use case.
- Rejected.

### D. MP4 progressive download (no HLS)

- Pros: simpler.
- Cons: poor mobile streaming UX (no adaptive bitrate); industry default is HLS for social; competitive disadvantage.
- Rejected.

### E. DASH (Dynamic Adaptive Streaming over HTTP) instead of HLS

- Pros: MPEG standard.
- Cons: HLS has wider client support (Safari + iOS native); HLS is the de-facto social-video standard; can revisit DASH alongside HLS later.
- Rejected (for P01).

### F. Native (non-sandboxed) transcode workers

- Pros: lower runtime overhead; simpler deployment.
- Cons: violates T-E-05 mitigation; ImageMagick / ffmpeg CVE history shows RCE risk is significant; non-negotiable.
- Rejected.

### G. AWS S3 instead of OCI Object Storage

- Pros: broader ecosystem; mature.
- Cons: violates single-cloud-substrate (ADR-0117); cross-cloud orchestration adds operational complexity.
- Rejected.

### H. Cloud-native managed transcode service (e.g., AWS MediaConvert, Cloudflare Stream)

- Pros: zero operational burden; managed service.
- Cons: per-pack data-residency complexity (managed services may not have per-pack POP); cost; vendor lock-in; opacity for EU AI Act + GDPR Art. 28 processor relationship.
- Rejected.

## Consequences

### Positive

- Single canonical media stack across the social µservice; team expertise concentrated.
- Sandboxed transcode workers mitigate CVE risk (T-E-05).
- Quarantine-first lifecycle ensures malware never reaches production (T-E-04 + content-moderation BC).
- Per-tenant prefix + KMS SSE preserves per-tenant data isolation in S3 (T-T-02 + GDPR Art. 32).
- HLS + WebP/AVIF + AAC give competitive video + image quality + cost-efficient delivery.
- Per-pack data-residency preserved; federation peer fetches via signed CDN URL without crossing pack boundary.
- CDN tier (Cloudflare R2 + Workers) provides ~30% egress-cost reduction vs OCI-only path per `cost-budget.md`.
- WCAG 2.2 Level AA accessibility via alt-text + WebVTT captions.
- HIPAA-Covered Entity tenants in pack-us-healthcare get PHI-redactor + auto-preview-OFF + safe-harbor de-identification.

### Negative

- ImageMagick + ffmpeg LTS upgrade cadence must be tracked weekly (CVE risk).
- Sandbox overhead (gVisor) costs ~10-20% transcode throughput; capacity model accounts for it.
- HLS storage cost is ~30% higher than progressive MP4 due to segment overhead + multi-bitrate; cost-budget reflects.
- CDN tier (Cloudflare R2 + Workers) adds a second cloud provider relationship; ops-finops + cloud-secrets must maintain dual-vendor config.
- libvips alternative (Alt. A) lost performance opportunity; revisit if image-transcode latency becomes user-visible.

### Operational

- Cargo workspace: `oya-social-post-composition-adapter-imagemagick` + `-adapter-ffmpeg` + `-adapter-s3` per ADR-0105 Amendment 3 naming.
- Helm: `media-transcode-worker` deployment with `runtimeClassName: gvisor`; non-root + read-only root FS.
- Weekly CVE scan via Trivy + Grype on transcode worker images.
- LTS pin tracking: ImageMagick 7.1 + ffmpeg 7.x + libwebp + libaom (AVIF encoder) in `Cargo.toml` + Helm values.
- Quarantine bucket + production bucket lifecycle managed via S3 bucket policies + IAM (per-bucket service-account scopes).
- Cloudflare R2 + Workers config managed via Terraform.
- Per-pack S3 + CDN bucket naming: `oya-social-media-<pack>` + `oya-social-cdn-<pack>` + `oya-social-quarantine-<pack>`.
- Integrity check worker: periodic digest verification on Postgres-stored digests vs S3-fetched blobs.
- Runbook: `runbooks/media-transcode-degraded.md` (Slice B; same shape as messenger attachment-restore).

### Regulatory

- **GDPR Art. 32**: technical measures preserved (KMS SSE + per-tenant prefix + sandboxed workers).
- **GDPR Art. 25**: privacy-by-design (PHI redactor + signed-URL TTL + Cedar re-evaluation per fetch).
- **GDPR Arts. 44-50**: cross-border transfer (federation peer fetches without crossing pack boundary).
- **KR PIPA Art. 29**: technical safeguards preserved.
- **HIPAA 45 CFR §164.502 + §164.514 + §164.530(j)**: PHI redactor + auto-preview-OFF + 6y WORM retention.
- **SEC Rule 17a-4(f)**: WORM retention via S3 Object Lock on Professional-tier.
- **WCAG 2.2 Level AA**: alt-text + WebVTT captions.
- **EU DSA Art. 14**: per-tenant ToS discloses CDN sub-processor (Cloudflare) per Art. 28 sub-processor registry.

## References

- ADR-0105 + Amendment 3 (backend-qualified adapter naming).
- ADR-0117 (single-cloud-substrate; primary OCI).
- ADR-0126 (parallel; dual-context).
- ADR-0131 (per-microservice flat layout).
- ImageMagick 7.1 LTS docs `imagemagick.org/script/release-notes.php`.
- ffmpeg 7.x docs `ffmpeg.org/documentation.html`.
- HLS RFC 8216.
- WebP `developers.google.com/speed/webp`.
- AVIF `aomedia.org/av1/specification/`.
- WebVTT RFC 9420.
- OCI Object Storage docs `docs.oracle.com/iaas/Content/Object`.
- Cloudflare R2 + Workers docs.
- gVisor `gvisor.dev`.
- Kata Container `katacontainers.io`.
- Trivy + Grype CVE scanners.
- WCAG 2.2 W3C Recommendation.
- HIPAA 45 CFR §164.502, §164.514, §164.530(j).
- SEC Rule 17a-4(f).
- GDPR Arts. 25, 32, 44-50.
- KR PIPA Art. 29.
- ImageTragick CVE-2016-3714 (historical context).
- `microservices/social/PRD.md` §"Performance" + §"Bounded Contexts".
- `microservices/social/threat-model.md` T-T-02, T-E-04, T-E-05, T-I-02.
- `microservices/social/cost-budget.md` §Media transcode + CDN.
- `microservices/social/policy/data-residency.md`.
