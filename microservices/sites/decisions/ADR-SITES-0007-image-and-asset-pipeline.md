---
id: ADR-SITES-0007
status: Accepted
date: 2026-05-17
microservice: sites
deciders: axis-sites, council-architecture, ops-sre-reliability, ops-security
owner: axis-sites
supersedes: []
superseded_by: []
related:
  - ADR-0105
  - ADR-0131
  - ADR-0133
  - ADR-SITES-0003
related_artifacts:
  - microservices/sites/PRD.md §FR-16, Performance image-optimize p95 ≤ 1s
  - microservices/sites/IP-011-cdn-delivery-and-pipeline.md
  - microservices/sites/runbooks/asset-optimization-degraded.md
  - microservices/sites/threat-model.md SVG XSS entry
purpose: |
  Choose the image + asset pipeline library and emission formats.
  Image optimization is on the hot publish path; correctness +
  performance matter equally; SVG security matters.
---

# ADR-SITES-0007: Image + asset pipeline — libvips 8.16 streaming; WebP + AVIF + JPEG-XL emission; ImageMagick rejected; Sharp rejected; W3C SRI for assets

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Sites publishes images at scale: per the capacity model, 500
image-optimize jobs/min baseline (5k/min peak). Per PRD-sites
§"Performance", image-optimize p95 ≤ 1s per source image. Per
PRD-sites §FR-16, every uploaded image must produce WebP + AVIF +
JPEG-XL responsive variants.

Three library candidates:
1. **libvips** (C; LGPL-2.1). Streaming image processing; very low
   memory footprint; horizontal-scalable. Used by Sharp (Node.js
   binding), CloudFlare, Wikimedia, Shopify Image CDN.
2. **ImageMagick / GraphicsMagick** (C; Apache-2.0). Mature; rich
   feature set; whole-image-in-memory (high memory cost).
3. **Sharp (Node.js)** (MIT). Node.js binding to libvips; we'd
   pull in Node runtime.

EXIF + SVG handling matters:
- EXIF strip is required (per ADR-SITES-0007 + GDPR Art. 25
  data-minimisation; per HIPAA on patient-portal sites; per
  ePrivacy).
- SVG sanitisation is required (SVG can embed `<script>` for XSS;
  per `threat-model.md`).
- libvips supports both natively + integrates with `svgsanitizer`
  for SVG.

Output formats:
- **WebP** (RFC 9711): 30-50% smaller than JPEG at same quality;
  wide browser support (~96%).
- **AVIF** (AOM AV1 Image File Format): 40-60% smaller than WebP;
  growing browser support (~88%).
- **JPEG-XL** (ISO/IEC 18181): 50%+ smaller; lossy + lossless;
  emerging browser support (~75% as of 2026-05; Chromium re-enabling
  after experimental period).
- **Original JPEG/PNG**: fallback `srcset` for old browsers.

Per W3C Subresource Integrity Recommendation, published assets must
emit SRI hashes (`integrity="sha384-..."`) for cache-integrity
verification at browser load.

## Decision

The sites µservice ships **libvips 8.16 LTS as the image pipeline**
with **WebP + AVIF + JPEG-XL responsive variants** emitted at
publish time, plus W3C SRI hashes for all assets.

Concrete bindings:
- **Library**: libvips 8.16 (LGPL-2.1) — bound via the Rust `libvips`
  crate.
- **Adapter**: `oya-sites-cdn-delivery-adapter-libvips` (backend-
  qualified per ADR-0105 Amendment 3).
- **Source size limit**: 100 MB max file size; 50MP max resolution
  (per `threat-model.md` DoS mitigation).
- **Output formats per source**:
  - JPEG / PNG source → WebP + AVIF + JPEG-XL + original (kept as
    legacy `srcset` for old browsers).
  - GIF source → WebP (animated) + AVIF (animated) + original.
  - SVG source → SVG (sanitised; `<script>` stripped via
    svgsanitizer) — vector format; no raster variants.
  - WebP / AVIF / HEIC source → re-encode to canonical WebP + AVIF
    + JPEG-XL for consistency.
- **Responsive widths**: 320, 640, 960, 1280, 1920, 2560 (configurable
  per theme).
- **EXIF strip**: always at upload.
- **SRI hashes**: SHA-384 per asset emitted into the `<img srcset>`
  / `<link>` / `<script>` tags at SSG render time.
- **Streaming**: libvips pipelined streaming with per-job memory cap
  of 512 MB (per `iac/helm/values.yaml`).
- **Per-job timeout**: 30s; abort if exceeded.

## Alternatives Considered

### A. ImageMagick / GraphicsMagick

- **Pros**:
  - Mature; most familiar to many engineers.
  - Rich filter set.
- **Cons**:
  - Whole-image-in-memory; OOM on > 50MP source.
  - Subprocess-spawn pattern (one process per job); high per-job
    overhead.
  - CVE history more active.
- **Rejected** because of memory + performance characteristics at
  our scale.

### B. Sharp (Node.js binding to libvips)

- **Pros**:
  - Most popular Node.js image library.
  - Same underlying libvips engine.
- **Cons**:
  - Pulls in Node.js runtime; our µservice plane is Rust-only.
  - Sharp's API surface adds nothing on top of direct libvips
    bindings.
- **Rejected** in favor of direct Rust libvips bindings.

### C. Cloudflare Images / imgix / Cloudinary

- **Pros**:
  - Managed service; no ops.
  - Edge-side optimization.
- **Cons**:
  - Vendor lock-in; per ADR-SITES-0003 substrate-portability matters.
  - Per-request cost adds up at our scale.
  - Per-pack residency: managed services don't guarantee per-pack
    edge selection.
- **Rejected** in favor of self-managed pipeline.

### D. WebAssembly-based image lib (Photon-RS, image-rs)

- **Pros**:
  - Pure Rust; portable.
- **Cons**:
  - Performance benchmarks show ~3-5× slower than libvips on the
    same operations.
  - AVIF + JPEG-XL encoder maturity lags.
- **Rejected** in favor of libvips.

### E. libvips streaming with WebP + AVIF + JPEG-XL output + W3C SRI  ← **CHOSEN**

- **Pros**:
  - Streaming = low memory; can handle 50MP source within bound.
  - All three modern formats emitted = covers ~99% of browser support.
  - SRI hashes = browser-side cache integrity verified.
  - Substrate-portable (LGPL-2.1; no vendor lock-in).
  - Industry-standard at scale (Cloudflare, Shopify, Wikimedia).
- **Cons**:
  - Three output formats per source = 3× publish time vs single-
    format. Mitigated by parallelism.
  - JPEG-XL encoder maturity still evolving (libjxl). Mitigated by
    feature-flag.
- **Accepted**.

## Consequences

### Positive

- **Image-optimize p95 ≤ 1s achievable** via streaming + bounded
  per-job memory.
- **WebP + AVIF + JPEG-XL** = bandwidth reduction 40-60% vs JPEG.
- **EXIF strip + SVG sanitisation** = privacy + security.
- **W3C SRI** = cache integrity browser-verified.
- **Substrate-portable** (LGPL-2.1; not vendor-locked).

### Negative

- **3× publish-time cost** for image variant emission. Mitigated by
  parallel libvips workers.
- **JPEG-XL encoder maturity** still evolving; feature-flag exposes
  on/off.
- **libvips LGPL-2.1** requires source-availability for libvips
  itself if we ship modified binaries; we don't modify libvips so
  the obligation is satisfied by linking to the unmodified library.

### Operational

- **libvips worker pods** with 512 MB memory limit each.
- **Per-job timeout** 30s; OOM-kill triggers runbook
  `asset-optimization-degraded.md`.
- **Image upload rate-limit** at REST layer (per-tenant
  100 uploads/min default).
- **PrometheusRule alerts** on optimize queue > 5 min.

### Regulatory

- **GDPR Art. 25**: EXIF strip = data-minimisation.
- **HIPAA**: pack-us-healthcare patient-portal images get EXIF-strip
  + SVG sanitise; no metadata leak.
- **W3C SRI Recommendation**: per spec.
- **WCAG 2.2 SC 1.1.1**: alt-text required at publish (T0-suggest
  capability assists; tenant must accept).

## Verification

- [ ] **Streaming pipeline** —
  `cargo nextest run -p oya-sites-cdn-delivery-adapter-libvips -- optimize_streaming`.
- [ ] **Output format coverage** —
  `cargo nextest run -p oya-sites-cdn-delivery-adapter-libvips -- emit_webp_avif_jpegxl`.
- [ ] **SVG script strip** —
  `cargo nextest run -p oya-sites-cdn-delivery-adapter-libvips -- svg_sanitize_strip_script`.
- [ ] **EXIF strip** —
  `cargo nextest run -p oya-sites-cdn-delivery-adapter-libvips -- exif_strip`.
- [ ] **SRI hash emission** —
  `cargo nextest run -p oya-sites-cdn-delivery-domain -- sri_hash_in_srcset`.
- [ ] **Image-optimize p95 ≤ 1s** —
  `cargo bench -p oya-sites-cdn-delivery-adapter-libvips -- optimize`.

## References

- libvips — `libvips.github.io/libvips`.
- libjxl (JPEG-XL reference encoder) — `github.com/libjxl/libjxl`.
- WebP — RFC 9711.
- AVIF — AOM AV1 Image File Format spec.
- JPEG-XL — ISO/IEC 18181.
- W3C Subresource Integrity Recommendation — `w3.org/TR/SRI/`.
- WCAG 2.2 SC 1.1.1 (non-text content).
- ADR-0105 Amendment 3 (backend-qualified adapters).
- ADR-SITES-0003 (CDN substrate).
- `microservices/sites/PRD.md` §FR-16, §"Performance".
- `microservices/sites/IP-011-cdn-delivery-and-pipeline.md`.
- `microservices/sites/runbooks/asset-optimization-degraded.md`.
- `microservices/sites/threat-model.md` SVG XSS entry.
- ImageMagick CVE history — `imagemagick.org/script/security-policy.php`.
