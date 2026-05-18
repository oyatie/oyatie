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

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-005-media-segment-bc
depends_on_changesets: [CS-RECORDINGS-IP-003-recording-ingest-bc, CS-RECORDINGS-IP-004-recording-bc]
parallel_safe_with_changesets: [CS-RECORDINGS-IP-006-transcript-bc]
enables: [CS-RECORDINGS-IP-011-playback-share-link-watermark-bcs]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | HLS m3u8 segment output conforms to RFC 8216 | `cargo test -p oya-recordings-media-segment-adapter-ffmpeg -- hls_rfc8216_conformance` |
| AC-02 | CMAF fragments emit per ISO/IEC 23000-19 | `cargo test -p oya-recordings-media-segment-adapter-ffmpeg -- cmaf_iso_conformance` |
| AC-03 | Multi-bitrate ladder (360p/540p/720p/1080p) emits with correct GOP alignment | `cargo test -p oya-recordings-media-segment-adapter-ffmpeg -- bitrate_ladder` |
| AC-04 | ffmpeg runs in gVisor sandbox — no host fs / egress access | `cargo test --test ffmpeg_sandbox_isolation` |
| AC-05 | `oya gate validate layer-correctness --microservice recordings` exits 0 | ADR-0131 |

## Build Sequence

1. Kernel: `MediaSegmenter`, `BitrateLadder`, `CdnEmitter` ports.
2. Domain: `Segment`, `Ladder`, `Gop`, `LoudnessTarget`.
3. Adapter: `-adapter-ffmpeg` (ffmpeg 7.x in gVisor).
4. Adapter: `-adapter-s3` for segment write.
5. Adapter: `-adapter-cdn-cloudfront-stub-or-self` for purge + warm.
6. `cargo test -p oya-recordings-media-segment-adapter-ffmpeg -- hls_rfc8216_conformance`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings FR | FR-03 (playback), FR-06 (export) |
| PRD-recordings NFR | NFR perf — HLS time-to-first-byte |
| ADR | ADR-RECORDINGS-0004 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| ffmpeg CVE exploited via crafted input | gVisor + seccomp + read-only rootfs |
| GOP-misalignment causes adaptive-bitrate stutter | `bitrate_ladder` test verifies keyframe alignment |
| Loudness exceeds EBU R128 ceiling | `audio-loudness` crate normalises to -23 LUFS ±0.5 |

## References

- RFC 8216 (HTTP Live Streaming).
- ISO/IEC 23000-19 (Common Media Application Format).
- ITU-R BS.1770-4 (Loudness measurement) / EBU R 128 (Loudness normalisation).
- ffmpeg documentation (`ffmpeg.org/documentation.html`).
- gVisor runtime documentation (`gvisor.dev/docs`).
- ADR-RECORDINGS-0004.

## Next IP

[`IP-006-transcript-bc.md`](IP-006-transcript-bc.md)
