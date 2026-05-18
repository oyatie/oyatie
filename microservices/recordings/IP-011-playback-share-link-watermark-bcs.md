---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-011-playback-share-link-watermark-bcs
status: pending
owner: axis-recordings
acceptance_lanes: [port-location]
---

# IP-011: Playback BC + Share-link BC + Watermarking BC + DRM-stub BC

## Intent

Land HLS playback with chapter-skip + caption-toggle + speaker-filter +
2x-speed + per-viewer dynamic + steganographic watermark + signed-URL
share-link with password + view-count cap + expiry + DRM-stub.

## Concrete crates

- `oya-recordings-playback-{kernel,domain,usecase,api,adapter-cdn-cloudfront-stub-or-self,adapter-redis,rest,sdk,app}`
- `oya-recordings-share-link-{kernel,domain,usecase,api,adapter-postgres,adapter-redis,rest,worker,sdk,app}`
- `oya-recordings-watermarking-{kernel,domain,usecase,adapter-ffmpeg,worker,app}`
- `oya-recordings-drm-stub-{kernel,domain,usecase,adapter,app}`
- `oya-recordings-accessibility-captions-{kernel,domain,usecase,api,adapter-postgres,rest,sdk,app}`

## Acceptance Gates

```bash
cargo nextest run -p oya-recordings-playback-kernel
cargo nextest run -p oya-recordings-share-link-kernel
```

## Next IP

[`IP-012-export-ediscovery-bcs.md`](IP-012-export-ediscovery-bcs.md)
