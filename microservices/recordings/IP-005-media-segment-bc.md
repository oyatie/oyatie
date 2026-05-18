---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-005-media-segment-bc
status: pending
owner: axis-recordings + ops-sre-reliability
acceptance_lanes: [port-location, layer-correctness]
---

# IP-005: Media-segment BC — HLS multi-bitrate + CMAF segmentation (ffmpeg gVisor)

## Intent

Land the ffmpeg-7.x-in-gVisor transcode pipeline that emits HLS + CMAF
multi-bitrate ladder (per ADR-RECORDINGS-0004). Adapters: -adapter-s3 (segment
write) + -adapter-cdn-cloudfront-stub-or-self (purge + warm) + -adapter-ffmpeg
(transcode).

## Concrete crates

`oya-recordings-media-segment-{kernel,domain,usecase,adapter-s3,adapter-cdn-cloudfront-stub-or-self,adapter-ffmpeg,worker,app}` +
shared `oya-recordings-video-encode-ladder-*` + `oya-recordings-audio-loudness-*`.

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice recordings
# HLS conformance fixture
cargo test -p oya-recordings-media-segment-adapter-ffmpeg -- hls_rfc8216_conformance
```

## Next IP

[`IP-006-transcript-bc.md`](IP-006-transcript-bc.md)
