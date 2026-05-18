---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-004-video-transcode-bc
status: pending
execution_unit: ChangeSet
owner: axis-shorts + ops-sre-reliability + cloud-k8s
acceptance_lanes: [cargo-build, cargo-nextest, helm-lint, oya-governance-sandbox-isolation]
depends_on: [IP-003]
---

# IP-004: video-transcode BC end-to-end

## Intent

Implement `video-transcode` BC: multi-bitrate HLS + DASH ladder; H.264 +
H.265 + AV1 + AAC + Opus + CMAF segment writer; ffmpeg 7.x LTS sandboxed
worker pool (gVisor / Kata); KEDA queue-depth autoscale.

Per ADR-SHORTS-0001:
- Ladder: 360p (500 kbps H.264) / 480p (1.2 Mbps H.264) / 720p (2.5 Mbps H.264) / 1080p (4 Mbps H.265) / 1440p (6 Mbps AV1).
- CMAF segments per ISO/IEC 23000-19.
- HLS manifests per RFC 8216; DASH manifests per ISO/IEC 23009-1.
- Per-bitrate-rung audio: AAC 96kbps + Opus 64kbps fallback.

## ChangeSet boundary

9 crates: `oya-shorts-video-transcode-{kernel,domain,usecase,api,adapter,adapter-ffmpeg,adapter-s3,worker,sdk}`.

## Concrete File Targets

| Crate | Key types |
|---|---|
| `oya-shorts-video-transcode-kernel` | `TranscodeJob`, `BitrateRung`, `Manifest`, `Segment`, `Codec`; port traits: `VideoTranscoder`, `ManifestWriter` |
| `oya-shorts-video-transcode-domain` | rung ladder selection; codec selection; CMAF segment math |
| `oya-shorts-video-transcode-usecase` | orchestrate dequeue→transcode→manifest-write→S3-publish→`TranscodeComplete` emission |
| `oya-shorts-video-transcode-adapter-ffmpeg` | impl `VideoTranscoder` via ffmpeg 7.x LTS in gVisor sandbox |
| `oya-shorts-video-transcode-adapter-s3` | impl `ManifestWriter`; HLS .m3u8 + DASH .mpd + CMAF segments |
| `oya-shorts-video-transcode-worker` | binary; KEDA-queue-driven worker loop |

## Acceptance Gates

```bash
cargo build -p oya-shorts-video-transcode-worker
cargo nextest run -p oya-shorts-video-transcode-{kernel,domain,usecase,adapter-ffmpeg,adapter-s3,worker}
cargo run -p oya-dev-cli -- gate validate sandbox-isolation --microservice shorts --bc video-transcode
```

E2E (kind cluster): submit 60s 1080p H.264 source → 5-rung HLS/DASH ladder published within 30s p95.

## Test Plan

- adapter-ffmpeg: integration vs real ffmpeg 7.x in gVisor sandbox; codec roundtrip; CMAF compliance verified via Bento4 (test-only).
- worker: end-to-end smoke; queue-depth autoscale; gVisor breakout attempt (must fail).
- adapter-s3: signed manifest roundtrip; CDN priming via Cloudflare Workers.

## Halt Conditions

- ffmpeg CVE — pin sunset; re-bench.
- Sandbox breakout — Sev-1; engage ops-security + cloud-k8s.
- AV1 encode latency exceeds budget — fall back to H.265 for 1440p tier.

## Next IP

[`IP-005-video-storage-and-cdn-bc.md`](IP-005-video-storage-and-cdn-bc.md)

## References

- PRD FR-03.
- ADR-SHORTS-0001 (transcode pipeline).
- `threat-model.md` T-D-02, T-D-07, T-E-05.
- HLS RFC 8216; MPEG-DASH ISO/IEC 23009-1; CMAF ISO/IEC 23000-19; H.264/AVC; H.265/HEVC; AV1 AOMedia; AAC; Opus RFC 6716.
- gVisor / Kata Container docs.
