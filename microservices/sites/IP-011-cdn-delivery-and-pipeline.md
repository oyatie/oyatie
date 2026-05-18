---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-011-cdn-delivery-and-pipeline
status: pending
execution_unit: ChangeSet
owner: axis-sites
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-layer-correctness]
---

# IP-011: cdn-delivery BC + S3 + Cloudflare-stub + libvips + Pandoc adapters

## Intent

Author the `cdn-delivery` BC per ADR-SITES-0003 (CDN substrate + cache strategy) and ADR-SITES-0007 (image pipeline). Implements `PublishedArtifact`, `CdnCacheKey`, `InvalidationRequest`. Signed Ed25519 CDN purge. Cache-key includes version-hash. libvips streaming pipeline emits WebP/AVIF/JPEG-XL responsive variants. Pandoc Markdown-to-HTML for portable-text. AC-02 + AC-14 covered.

## ChangeSet boundary

10 crates: `oya-sites-cdn-delivery-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-cloudflare-cdn-stub,adapter-libvips,adapter-pandoc,rest,worker,app}`.

## Acceptance Gates

```bash
cargo nextest run -p oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub -- signed_purge_ed25519
cargo nextest run -p oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub -- invalidate_p95_lt_2s
cargo nextest run -p oya-sites-cdn-delivery-adapter-libvips -- optimize_streaming
cargo nextest run -p oya-sites-cdn-delivery-adapter-libvips -- svg_sanitize_strip_script
cargo nextest run -p oya-sites-cdn-delivery-adapter-pandoc -- markdown_to_html_roundtrip
cargo nextest run -p oya-sites-cdn-delivery-usecase -- publish_100_pages_p95_lt_5s
```

## Test Plan

- Unit: signed-purge Ed25519 verification.
- Unit: cache-key version-hash inclusion (no version-blind reuse per Hyrum #5).
- Integration: libvips streaming pipeline emits WebP/AVIF/JPEG-XL.
- Integration: libvips OOM kill on > 50MP source.
- Integration: SVG sanitiser strips `<script>`.
- Integration: Pandoc Markdown round-trip via portable-text.
- Integration: 100-page publish p95 ≤ 5s.

## References

- ADR-SITES-0003 (CDN substrate + cache strategy).
- ADR-SITES-0007 (image pipeline).
- libvips — `libvips.github.io/libvips`.
- Pandoc — `pandoc.org`.
- WebP (RFC 9711); AVIF (AV1 Image File Format) — AOM; JPEG-XL (ISO/IEC 18181).
- HTTP `Cache-Control` (RFC 9111); `stale-while-revalidate` (RFC 5861).
