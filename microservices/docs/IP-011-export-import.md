---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-011-export-import
status: pending
execution_unit: ChangeSet
owner: axis-docs + ops-security
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-export-sandbox-conformance, oya-governance-ooxml-import-fidelity, oya-governance-pdfa-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: export-import BC (11 crates) — Pandoc + WeasyPrint + Chromium inside gVisor sandbox

## Intent

Implement export-import pipeline per ADR-DOCS-0003: Pandoc 3.x for DOCX / Markdown / HTML / EPUB / LaTeX; WeasyPrint 62.x default for PDF; Chromium-headless opt-in for high-fidelity PDF. All workers run in gVisor sandbox per ADR-DOCS-0003 + per `runbooks/export-pipeline-failure-pandoc-rollback.md`. OOXML import fidelity matrix per ADR-DOCS-0006. PDF/A-1b + PDF/A-2u archival output per AC-10.

## ChangeSet boundary

11 crates: kernel + domain + usecase + api + adapter + adapter-pandoc + adapter-weasyprint + adapter-chromium + rest + worker + app.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-export-import-{kernel,domain,usecase,api,adapter,adapter-pandoc,adapter-weasyprint,adapter-chromium,rest,worker,app}/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-export-import-domain/src/{format_profile,fidelity_matrix,pdfa_validator}.rs` | create |
| `microservices/docs/src/crates/oya-docs-export-import-adapter-pandoc/src/{lib,docx_writer,markdown_writer,html_writer,epub_writer,latex_writer,docx_reader,html_reader}.rs` | create |
| `microservices/docs/src/crates/oya-docs-export-import-adapter-weasyprint/src/{lib,pdf_renderer}.rs` | create |
| `microservices/docs/src/crates/oya-docs-export-import-adapter-chromium/src/{lib,pdf_renderer,headless_runtime}.rs` | create |
| `microservices/docs/src/crates/oya-docs-export-import-worker/src/{lib,gvisor_pool,job_dispatcher,sandbox_lifecycle}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-export-import-adapter-pandoc -- ooxml_roundtrip  # AC-03
cargo nextest run -p oya-docs-export-import-adapter-pandoc -- gvisor_escape_blocked  # AC-09
cargo nextest run -p oya-docs-export-import-adapter-weasyprint -- pdfa_validation  # AC-10
cargo nextest run -p oya-docs-export-import-worker -- per_job_tmpfs_isolation
cargo run -p oya-dev-cli -- gate validate export-sandbox-conformance --microservice docs
cargo run -p oya-dev-cli -- gate validate ooxml-import-fidelity --microservice docs
```

## References

- ADR-DOCS-0003 (export pipeline architecture).
- ADR-DOCS-0006 (DOCX import fidelity matrix).
- veraPDF — `verapdf.org` (PDF/A validator).
- gVisor — `gvisor.dev`.
- Pandoc — `pandoc.org`.
- WeasyPrint — `weasyprint.org`.
- Chromium release notes — `chromiumdash.appspot.com/`.
