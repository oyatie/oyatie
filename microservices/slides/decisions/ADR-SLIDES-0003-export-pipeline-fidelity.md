---
id: ADR-SLIDES-0003
title: Export pipeline fidelity — PPTX round-trippable subset + PDF/A + deterministic MP4
microservice: slides
status: Accepted
date: 2026-05-17
owner: axis-workspace + ops-security + council-architecture
deciders: council-architecture, axis-workspace, ops-security, ops-accessibility
supersedes: []
superseded_by: []
related: [ADR-0105, ADR-0135, ADR-0131, ADR-0133]
related_specs: []
related_artifacts:
  - microservices/slides/PRD.md (AC-02, AC-15)
  - microservices/slides/PHASE-01-SLIDES-FOUNDATION.md (IP-011)
  - microservices/slides/runbooks/export-pipeline-failure-pptx.md
  - microservices/slides/competitor-parity-matrix.md
purpose: Choose the import/export library substrate (PPTX, ODP, PDF, Keynote, MP4) with fidelity guarantees and security isolation for parsing untrusted files.
doc_status: published
---

# ADR-SLIDES-0003: Export pipeline fidelity — PPTX round-trippable subset, PDF/A via WeasyPrint or Chromium-headless, deterministic MP4 via ffmpeg

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The slides µservice must import and export across multiple presentation formats:

- **PPTX** (Microsoft PowerPoint — OOXML PresentationML per ECMA-376): primary competitor parity format. Tenants frequently migrate decks from / share decks back to PowerPoint Web users.
- **ODP** (OpenDocument Presentation 1.3 per ISO/IEC 26300): native format for LibreOffice Impress + ONLYOFFICE; required for EU government tenants that mandate ODF.
- **PDF** (ISO 32000-1 + PDF/A-1b + PDF/A-2u): archival format; PAdES signatures for legally-signed PDFs (per eIDAS where applicable).
- **Keynote** (.key, Apple closed binary plist + XML): import-only best-effort; Apple does not publish a stable format spec.
- **MP4** (ISO/IEC 14496-14): video export of present-mode for sharing on social/email; deterministic mode required for reproducibility.
- **PNG-per-slide**: simple per-slide raster export.

Per PRD AC-02 + AC-15, **PPTX round-trip subset fidelity ≥ 95%**: import a PPTX → render visually → export PPTX → reimport → byte-equal over the round-trippable OOXML PresentationML subset (excluding format-specific binary blobs we do not preserve like proprietary SmartArt extensions).

Per `threat-model.md` T-T-03 and T-E-04, all import workers run in **gVisor sandbox** with ClamAV + OPSWAT pre-parse scanning. Imported PPTX files are untrusted by definition (potentially maliciously-crafted OOXML).

Per `dashboards/export-and-import-pipeline.json` and `cost-budget.md`, export throughput targets:
- PDF 50-slide p95 ≤ 3s
- PPTX 50-slide p95 ≤ 5s
- MP4 p95 ≤ slide_count × 1s + 5s overhead
- PNG p95 ≤ slide_count × 100ms

PRD Open Questions 2 + 3:
- Q2: PPTX library — pure-Rust pptx-rs (preview) vs Pandoc bridge vs calamine-derived?
- Q3: PDF renderer — WeasyPrint vs Chromium-headless vs typst-pdf?

## Decision

Adopt the following pipeline:

### PPTX import + export

- **Import**: Pandoc 3.x bridge with custom slides reader that maps OOXML PresentationML → slides canonical JSON. Pandoc invoked in gVisor sandbox. ClamAV + OPSWAT dual-scan pre-parse.
- **Export**: bespoke OOXML serializer in Rust (`oya-slides-import-export-adapter-pptx-serializer`) targeting the round-trippable OOXML PresentationML subset. **Not** Pandoc on the export side — Pandoc lossy in PresentationML round-trip per upstream Pandoc maintainer notes.
- **Round-trip invariant**: 100 golden PPTX corpus under `tests/golden/pptx/`; import → export → reimport produces byte-equal output for ≥ 95% of OOXML PresentationML round-trippable subset.
- **Unsupported features**: SmartArt, complex 3-D shapes, embedded VBA macros (always stripped on import per security policy), proprietary chart types beyond OOXML chartSpace. Communicated to tenant via `EmitDiagnostic` warnings on import.

### ODP import + export

- **Import + Export**: Pandoc 3.x bridge. ODP-side schema simpler than OOXML; Pandoc round-trip reasonable for the ODP subset oyatie supports.

### PDF export

- **Default path — WeasyPrint** (Python; CSS-paged-media renderer) for PDF/A-1b emission. WeasyPrint is widely used (Mozilla Foundation legal tooling cited reference), well-maintained, CSS-paged-media compliant, supports PDF/A profile.
- **Fallback — Chromium-headless** (via `chromium-headless-shell` containerized in gVisor) for decks with CSS that WeasyPrint cannot render (e.g., complex flexbox + CSS animation snapshots). Chromium-headless emits PDF; post-processed via `qpdf` for PDF/A-1b conformance.
- **PAdES signing**: for legally-signed PDFs, post-process via OpenBao-Transit + KMS-held signing keys per `compliance.md` eIDAS section.

### Keynote import

- Best-effort via Pandoc bridge with a custom Keynote reader; supported features limited to: slide structure, text-boxes, images, basic shapes, basic animations. Documented limitations to tenant.

### MP4 export

- ffmpeg 7.x deterministic mode: `-shortest -fflags +genpts -copyts -map_metadata -1` flag set; per-slide PNG frames assembled to MP4 with fixed framerate (30fps) + audio track from any audio-embed.
- gVisor sandbox.
- Output sha256 logged for determinism verification (`tests/integration/mp4_deterministic.rs`).

### PNG-per-slide export

- WeasyPrint or Chromium-headless render to PNG per slide; deterministic file naming `slide-{ordinal:04d}.png`.

## Alternatives Considered

### PPTX

#### A — Pure-Rust `pptx-rs` (preview-grade)

- **Pros**: Pure Rust; same toolchain as core. No Python/Pandoc dependency.
- **Cons**: Preview-grade as of evaluation; OOXML coverage incomplete; missing chart + table + complex shape support.
- **Rejected reason**: not production-ready for AC-02 fidelity target.

#### B — `calamine`-derived approach

- **Pros**: calamine is mature for OOXML.
- **Cons**: calamine is xlsx-focused (sheets); does NOT cover PresentationML PPTX. Building PPTX support on calamine equates to writing pptx-rs.
- **Rejected reason**: misaligned to PPTX.

#### C — `LibreOffice` headless conversion bridge

- **Pros**: LibreOffice has comprehensive PPTX support; would handle every feature.
- **Cons**: LibreOffice container is large (~500 MB); slow startup (~3-5s per invocation). Pandoc-bridge throughput is ~10x faster.
- **Rejected reason**: throughput.

### PDF

#### D — `typst-pdf` (Rust-native)

- **Pros**: Pure Rust; modern document model.
- **Cons**: typst is its own document format; output PDF is good but mapping CSS-styled slides to typst's model adds engineering. PDF/A-1b conformance evaluated subsequent-to-M03-completion.
- **Rejected reason**: M03 timeline; revisit subsequent-to-M03-completion.

#### E — `pdfium` direct rendering

- **Pros**: Chromium's PDF rendering engine.
- **Cons**: pdfium is for rendering existing PDFs, not creating them.
- **Rejected reason**: wrong direction.

#### F — `wkhtmltopdf`

- **Pros**: HTML-to-PDF; widely used.
- **Cons**: wkhtmltopdf is deprecated upstream (2023); uses outdated WebKit; no CSP support; security concerns.
- **Rejected reason**: deprecated.

### MP4

#### G — `gstreamer` pipeline

- **Pros**: gstreamer is modular + comprehensive.
- **Cons**: Larger dependency surface; less reproducible than ffmpeg.
- **Rejected reason**: ffmpeg simpler + more reproducible.

#### H — Pure-Rust video encoder (`av1-rs` or similar)

- **Pros**: Pure Rust.
- **Cons**: Maturity insufficient for M03; ffmpeg's broad codec coverage and deterministic-mode discipline more proven.
- **Rejected reason**: maturity.

### Round-trip strategy

#### I — Strict round-trip (100% byte-equal) over full PPTX feature set

- **Pros**: Strongest fidelity guarantee.
- **Cons**: Many PPTX features (SmartArt, embedded VBA macros, proprietary 3-D shapes) are not openly specified or are security risks (VBA macros stripped on import). 100% round-trip impossible.
- **Rejected reason**: impossible.

#### J — Best-effort no-invariant

- **Pros**: Simpler.
- **Cons**: No defendable fidelity claim; PowerPoint Web competitor benchmark beats us by clear margin; tenants distrust round-trips.
- **Rejected reason**: insufficient competitor parity.

#### K — Strict round-trip over a defined subset (the choice)

- **Pros**: Defendable claim; tested via golden corpus; tenants can verify.
- **Cons**: Tenant must accept subset list (clearly published in `docs/standards/slides-pptx-subset.md`).
- **Accepted**: chosen approach.

## Consequences

### Architectural

- The `import-export` BC crates: `oya-slides-import-export-{kernel, domain, usecase, api, adapter, adapter-pandoc, adapter-weasyprint, adapter-chromium-headless, adapter-ffmpeg, worker, sdk}`.
- All format adapters are backend-qualified per ADR-0105 Amendment 3; library-specific code stays in the qualified adapter.
- gVisor sandbox + ClamAV + OPSWAT pre-parse for every import worker.
- Deterministic-mode flags pinned per format; output sha256 logged.

### Downstream impact on other µservices and IPs

1. **IP-011 (import-export full)** — authors the pipeline.
2. **observability µservice** — slides-specific export/import SLIs (latency p95 by format + error rate by format + scan verdict distribution).
3. **cloud-iac µservice** — gVisor sandbox + worker pool Helm chart; ffmpeg + WeasyPrint + Chromium-headless + Pandoc container images SLSA-L3-attested.
4. **audit-chain µservice** — every export + import emits Ed25519-sealed `ExportJobCompleted` + `ImportJobCompleted` audit row.

### SLOs gaining new dimensions

- `slides.export_pdf_latency_p95` — target ≤ 3s.
- `slides.export_pptx_latency_p95` — target ≤ 5s.
- `slides.export_mp4_latency_p95` — target ≤ slide_count × 1s + 5s.
- `slides.export_error_rate_by_format` — < 1% over 10m.
- `slides.import_scan_verdict_distribution` — clean:suspicious:malware tracked.
- `slides.pptx_roundtrip_subset_pass_rate` — ≥ 0.95 (CI lane).

### CI lanes added

- `oya-governance-slides-pptx-roundtrip-subset` — 100 golden corpus drill; BLOCKER on dev + staging.
- `oya-governance-pdf-a-conformance` — verify PDF/A-1b conformance on export-generated PDFs.
- `oya-governance-mp4-determinism` — sha256-equal across re-runs.

### Supply-chain + security

- Pandoc 3.x, WeasyPrint, Chromium-headless, ffmpeg 7.x — version-pinned LTS; advisory feeds subscribed.
- ClamAV + OPSWAT signature feeds; dual-scanner required.
- gVisor advisory feed monitoring.
- SLSA-L3 build provenance for every export worker image.

### Risk register

- **Risk**: PPTX round-trip subset coverage misalignment with tenant expectations. **Mitigation**: subset list published in `docs/standards/slides-pptx-subset.md`; `EmitDiagnostic` warnings surface unsupported features.
- **Risk**: WeasyPrint OOM on complex deck. **Mitigation**: Chromium-headless fallback (per `failure-modes.md` FM-09).
- **Risk**: ffmpeg deterministic-mode regression on upgrade. **Mitigation**: per-version sha256 stability test.
- **Risk**: gVisor sandbox escape (CVE). **Mitigation**: gVisor advisory feed; immediate upgrade SLA.
- **Risk**: Pandoc maintainer-attrition or large breaking change. **Mitigation**: kernel-port-wrapping the bridge; ability to swap to LibreOffice headless if needed (slower but functional fallback).

## References

- PRD `microservices/slides/PRD.md` AC-02, AC-15.
- ECMA-376 — OOXML PresentationML.
- ISO 32000-1 — PDF 1.7.
- ISO 19005-1 — PDF/A-1.
- ISO 19005-2 — PDF/A-2.
- ISO/IEC 26300 — ODF Presentation 1.3.
- ISO/IEC 14496-14 — MP4 file format.
- eIDAS Regulation (EU) 910/2014 — PAdES signatures.
- ffmpeg 7.x deterministic-mode docs — `ffmpeg.org/ffmpeg.html`.
- WeasyPrint — `weasyprint.org`.
- Chromium-headless — `chromium.org/chromium-headless`.
- Pandoc 3.x — `pandoc.org`.
- ADR-0105 (backend-qualified adapters).
- ADR-0133 (industry-best-practice conformance).
- threat-model.md T-T-03, T-E-04, T-T-04.
- failure-modes.md FM-08, FM-09, FM-10.
