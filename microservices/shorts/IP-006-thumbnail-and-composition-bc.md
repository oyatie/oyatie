---
doc_class: ImplementationPlan
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-006-thumbnail-and-composition-bc
status: pending
owner: axis-shorts
depends_on: [IP-004]
---

# IP-006: thumbnail-generation + video-composition BC end-to-end

## Intent

- `thumbnail-generation` BC: poster JPEG + animated GIF + WebP preview per video; emoji + reaction overlays. ffmpeg 7.x in gVisor.
- `video-composition` BC: server-side clip + cut + sticker + caption overlay finalisation when client preview is partial; preview-only path retained for client-side composers.

## ChangeSet boundary

8 + 8 = 16 crates: `oya-shorts-thumbnail-generation-{...}` + `oya-shorts-video-composition-{...}`.

## Concrete File Targets

Key entities: `ThumbnailJob`, `PosterFrame`, `AnimatedPreview`, `Clip`, `Cut`, `StickerOverlay`, `CaptionOverlay`, `Composition`.

Ports: `ThumbnailGenerator`, `CompositionEngine`, `StickerCatalog`.

## Acceptance Gates

```bash
cargo build -p oya-shorts-thumbnail-generation-worker
cargo build -p oya-shorts-video-composition-worker
cargo nextest run -p oya-shorts-thumbnail-generation-{kernel,domain,usecase,adapter-ffmpeg}
cargo nextest run -p oya-shorts-video-composition-{kernel,domain,usecase,adapter-ffmpeg}
```

E2E: poster + animated GIF + WebP generated within 2s p95; composition with sticker + caption overlay produces playable output.

## Test Plan

- Thumbnail unit tests for frame-sampling.
- Composition unit tests for clip-and-cut math.
- Integration ffmpeg 7.x sandboxed.

## Halt Conditions

- ffmpeg overlay CVE.

## Next IP

[`IP-007-audio-track-library-and-attribution-bc.md`](IP-007-audio-track-library-and-attribution-bc.md)

## References

- PRD FR-04, FR-02.
- ADR-SHORTS-0001.
