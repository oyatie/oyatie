---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-011-playback-share-link-watermark-bcs
status: pending
owner: axis-recordings
acceptance_lanes: [port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-011-playback-share-link-watermark-bcs
depends_on_changesets: [CS-RECORDINGS-IP-004-recording-bc, CS-RECORDINGS-IP-005-media-segment-bc]
parallel_safe_with_changesets: [CS-RECORDINGS-IP-010-retention-legal-hold-bcs]
enables: [CS-RECORDINGS-IP-012-export-ediscovery-bcs]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | HLS playback with chapter-skip + caption-toggle + 2x-speed | `cargo nextest run -p oya-recordings-playback-rest -- playback_features` |
| AC-02 | Share-link Ed25519-signed; password Argon2id; view-cap atomic; expiry strict | `cargo nextest run -p oya-recordings-share-link-* -- share_link_invariants` |
| AC-03 | Per-viewer dynamic + steganographic watermark survives downscale to 480p | `cargo nextest run -p oya-recordings-watermarking-adapter-ffmpeg -- watermark_survives_downscale` |
| AC-04 | Captions accessible per WCAG 2.2 AA (caption-toggle + per-segment timing) | `cargo nextest run -p oya-recordings-accessibility-captions-domain -- wcag_2_2_aa` |
| AC-05 | DRM-stub interface present for future Widevine/FairPlay integration | `cargo nextest run -p oya-recordings-drm-stub-domain -- interface_present` |

## Build Sequence

1. Playback kernel + adapter (CDN/Redis) + REST handler.
2. Share-link kernel + adapter-postgres + adapter-redis + REST.
3. Watermarking kernel + adapter-ffmpeg + worker.
4. DRM-stub kernel + adapter (interface only).
5. Accessibility-captions kernel + adapter-postgres + REST.
6. `cargo nextest run -p oya-recordings-{playback,share-link,watermarking,drm-stub,accessibility-captions}-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings FR | FR-03 (playback), FR-07 (share-link), FR-16 (watermarking) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Watermark removed by lossy recompression | Steganographic + visible variants; survives 480p downscale test |
| Share-link enumeration | Per-IP rate-limit + audit-chain on invalid attempts |
| Caption timing drift on adaptive-bitrate switch | Captions emit against segment timestamps not wall clock |

## References

- RFC 8216 (HLS).
- WCAG 2.2 AA (W3C — `www.w3.org/TR/WCAG22`).
- RFC 8032 (Ed25519); RFC 9106 (Argon2).
- Cox et al. "Digital Watermarking and Steganography", 2nd ed. (Morgan Kaufmann 2008).
- Widevine / FairPlay DRM reference (Google / Apple developer docs).

## Next IP

[`IP-012-export-ediscovery-bcs.md`](IP-012-export-ediscovery-bcs.md)
