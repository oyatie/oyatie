---
id: ADR-RECORDINGS-0004
status: Accepted
date: 2026-05-17
microservice: recordings
deciders: axis-recordings, ops-sre-reliability, ops-security, council-privacy
owner: axis-recordings
supersedes: []
superseded_by: []
related: [ADR-0117, ADR-0131, ADR-0133, ADR-SHORTS-0004]
related_artifacts:
  - microservices/recordings/PRD.md (FR-03 playback; FR-07 share-link; FR-16 watermark)
  - microservices/recordings/runbooks/playback-cdn-cache-cascade.md
  - microservices/recordings/runbooks/watermark-key-rotation.md
  - microservices/recordings/runbooks/transcode-pipeline-failure.md
  - microservices/recordings/slos/playback-start-latency.openslo.yaml
  - microservices/recordings/multi-region.md
purpose: |
  Fix the playback + CDN architecture: HLS multi-bitrate ladder + DRM tier
  stub + per-viewer dynamic + steganographic watermark + signed-URL share-
  link + CDN cache strategy. Aligns with RFC 8216 (HLS), ISO/IEC 23000-19
  (CMAF), MPEG-DASH-IF, and EBU R128 (audio loudness).
---

# ADR-RECORDINGS-0004: Playback + CDN strategy — HLS multi-bitrate + CMAF + per-viewer watermark + DRM stub

## Status

Accepted — 2026-05-17.

## Context

PRD-recordings FR-03 + FR-07 + FR-16 mandate:
- low-latency playback start (warm p99 ≤ 400ms / cold ≤ 1s)
- chapter-skip + caption-toggle + speaker-filter + 2x-speed
- signed-URL share-link with password + view-count cap + expiry
- per-viewer dynamic watermark for sensitive recordings

Playback substrate choices:

- **HLS RFC 8216** (Apple-led standard; ubiquitous browser + iOS support).
- **DASH ISO/IEC 23009-1** (MPEG standard; broader codec support).
- **CMAF ISO/IEC 23000-19** (common-media-format; enables HLS + DASH from
  same segments).

CDN choices:

- **CloudFront** (managed; broad edge coverage; HLS-native; CMAF-friendly;
  per-pack-region edge selection; signed-URL native).
- **Bunny + Fastly + nginx-vod self-host** (residency-strict for pack-kr
  + pack-ksa where CloudFront posture is constrained).
- **Akamai** (mature OTT-grade; cost-prohibitive at oyatie's scale).
- **Cloudflare Stream** (cheap + good; less HLS-CMAF flexibility for
  enterprise watermarking).

DRM choices (future, but the recordings µservice surface must accommodate):

- **Widevine** (Google; ubiquitous Android + Chrome).
- **Fairplay** (Apple; iOS + Safari).
- **PlayReady** (Microsoft; enterprise).

Watermarking choices:

- **Visible only** (logo + viewer + timestamp at ffmpeg overlay).
- **Steganographic only** (DCT-coefficient HMAC embedding; survives screen-
  capture re-encode).
- **Both** (defence-in-depth — visible deters, steganographic enables
  post-hoc attribution).

## Decision

oyatie recordings ships an **HLS-multi-bitrate-primary + CMAF-segmented +
CloudFront-primary-with-self-host-fallback CDN + visible+steganographic
watermark + DRM stub**:

1. **Playback container: HLS RFC 8216 + CMAF ISO/IEC 23000-19**. Segments
   are CMAF (.m4s); manifests are HLS (.m3u8). DASH manifest emitted in
   parallel from the same CMAF segments (DASH-HLS-IF alignment) for clients
   that prefer DASH.
2. **Multi-bitrate ladder** (per ADR-RECORDINGS-0001's ffmpeg adapter):
   - 240p @ 400 kbps (mobile-bandwidth-constrained)
   - 480p @ 800 kbps (mobile default)
   - 720p @ 1.5 Mbps (web default)
   - 1080p @ 3 Mbps (high-bandwidth)
   - 4K stub @ 8 Mbps (tenant-tier ≥ enterprise; per ADR-SHORTS-0004 future
     surface)
3. **Audio**: AAC LC 128 kbps + Opus 96 kbps for browser-native; EBU R128
   loudness-normalised to -23 LUFS (broadcast-grade).
4. **Captions**: WebVTT (.vtt) primary; TTML / EBU-TT-D (.ttml) for
   broadcast-class tenants; descriptive-audio metadata per SMPTE-TT for
   accessibility.
5. **CDN backend choice**:
   - Primary: CloudFront (pack-eu, pack-us, pack-us-healthcare, pack-us-
     financial, pack-jp, pack-sg, pack-au, pack-in, pack-br, pack-ae).
   - Self-host: Bunny + Fastly + nginx-vod (pack-kr, pack-ksa where strict
     residency forbids CloudFront edges).
   - Backend-qualified adapter:
     `oya-recordings-media-segment-adapter-cdn-cloudfront-stub-or-self`
     selects per pack at Helm value time.
6. **Cache strategy**:
   - Cache key: `<pack>/<tenant_id>/<recording_id>/<bitrate>/<segment_index>`
     + signed-URL ID (per-viewer watermark variants don't share cache keys).
   - TTL: 24h for static segments; 1h for manifests.
   - Purge: invalidation on redaction-overlay change.
   - Per-pack origin shield via Lambda@Edge (or nginx-vod equivalent).
7. **Signed-URL share-link**:
   - HMAC-SHA256 per-tenant secret from `${openbao:secret/recordings/<tenant>/share-link-hmac}`.
   - 24h default TTL (max 7d).
   - View-count cap enforced via Valkey counter (per-share-link).
   - Optional password (bcrypt hash; client supplies password → server
     verifies → emits playback-session token).
   - Per ADR-0117 residency — share-links are pack-local; cross-pack
     refused.
8. **Per-viewer watermark**:
   - **Visible**: tenant logo + viewer email + recording timestamp; overlaid
     by ffmpeg `drawtext + drawbox` at playback transcode (per-viewer
     transcode for sensitive recordings).
   - **Steganographic**: HMAC-SHA256(tenant-watermark-seed, viewer_ref ||
     recording_id || session_id) bits embedded in DCT coefficients of
     I-frames; survives moderate-bitrate screen-capture re-encode.
   - Key rotation per `runbooks/watermark-key-rotation.md`.
9. **DRM stub**: `oya-recordings-drm-stub-*` exposes Widevine / Fairplay /
   PlayReady stub interfaces per ADR-SHORTS-0004 future-proofing. No DRM
   service at M02; the adapter rejects DRM requests with `NotYetImplemented`
   error. M04 successor-IP ADR activates per-vendor DRM service integration.
10. **HLS manifest byte stability** (Hyrum's-Law preservation per
    migration-from-connect.md): manifest emits CMAF segments in
    monotonic order; segment-byte-range deterministic within ±64 bytes.

## Alternatives Considered

### A. DASH-primary

- Pros: ISO/IEC standardised; broader codec support; less Apple lock-in.
- Cons: iOS Safari needs HLS; if DASH-primary we'd still ship HLS, doubling
  manifest serving.
- Rejected as primary; emitted in parallel from CMAF segments.

### B. Akamai CDN

- Pros: OTT-grade; mature.
- Cons: cost is 2-3× CloudFront at oyatie's hyperscaler-comparator scale;
  edge coverage advantages are marginal for the recordings use case.
- Rejected; CloudFront primary; self-host for residency-strict packs.

### C. Visible-only watermark

- Pros: simpler.
- Cons: post-hoc attribution after a screen-capture leak is much harder.
- Rejected; defence-in-depth (visible + steganographic) wins.

### D. Single 1080p bitrate (no ladder)

- Pros: simpler transcode; less storage.
- Cons: mobile-bandwidth-constrained tenants suffer; CDN cost higher on
  egress because every viewer downloads 1080p.
- Rejected; multi-bitrate ladder is industry standard for OTT-grade.

### E. Build a DRM service at M02

- Pros: enables true content-protection at launch.
- Cons: 6-month roadmap minimum; over-narrow at hero-product launch where
  watermarking covers 90 % of the use case.
- Rejected; defer DRM activation to M04 ADR.

## Consequences

### Positive

- HLS + CMAF combination gives broad client compatibility with single
  transcode.
- Per-viewer watermark + steganographic seal solves leak-attribution.
- CDN backend-qualified adapter allows per-pack residency without code
  branching.
- Multi-bitrate ladder reduces CDN egress on mobile traffic.
- EBU R128 loudness normalisation gives broadcast-class playback consistency.

### Negative

- Per-viewer watermark requires per-viewer transcode for sensitive
  recordings (mitigated by ffmpeg-on-the-fly per playback session; cached
  per signed-URL TTL).
- ffmpeg 7.x in gVisor sandbox is a quarterly-CVE-review surface.
- CMAF segment alignment with HLS + DASH manifests adds encode complexity.

### Operational

- Cargo workspace adds `oya-recordings-playback-*` (8 crates) +
  `oya-recordings-share-link-*` (9 crates) + `oya-recordings-media-segment-*`
  (8 crates) + `oya-recordings-watermarking-*` (7 crates) +
  `oya-recordings-video-encode-ladder-*` (6 crates) +
  `oya-recordings-audio-loudness-*` (6 crates) +
  `oya-recordings-drm-stub-*` (5 crates).
- IaC: per-pack CloudFront vs self-host overlay (per
  `iac/kustomize/overlays/pack-*`).
- CI: HLS + DASH manifest conformance lane against the RFC 8216 + ISO/IEC
  23009-1 fixtures; CMAF compliance lane against ISO/IEC 23000-19 fixtures.

### Regulatory

- **HLS RFC 8216 + DASH ISO/IEC 23009-1 + CMAF ISO/IEC 23000-19** —
  industry-standard codecs reduce regulatory surface vs. proprietary.
- **EBU R128**: broadcast-grade audio loudness.
- **SMPTE-TT + EBU-TT-D**: accessibility captions per WCAG 2.2.
- **DRM stub** is future-proofed; M04 ADR engages per-vendor.
- **Watermarking**: tenant-policy-controlled with balancing-test record
  per `legal/balancing-test-watermark.md` per `dpia.md` R-07.

## References

- RFC 8216 — HLS.
- ISO/IEC 23009-1 — DASH.
- ISO/IEC 23000-19 — CMAF.
- ISO/IEC 14496-12 — MP4.
- RFC 6716 — Opus.
- AAC LC (ISO/IEC 14496-3).
- W3C WebVTT, W3C TTML.
- EBU-TT-D, SMPTE-TT.
- EBU R128 — audio loudness.
- MPEG-DASH-IF.
- Widevine, Fairplay, PlayReady documentation.
- CloudFront documentation (`docs.aws.amazon.com/AmazonCloudFront`).
- Bunny / Fastly / nginx-vod docs.
- HMAC-SHA256 — FIPS 198-1.
- ADR-0117, ADR-0131, ADR-0133, ADR-SHORTS-0004.
- microservices/recordings/PRD.md FR-03 + FR-07 + FR-16.
- microservices/recordings/runbooks/playback-cdn-cache-cascade.md.
- microservices/recordings/runbooks/watermark-key-rotation.md.
- microservices/recordings/slos/playback-start-latency.openslo.yaml.
- microservices/recordings/multi-region.md.
