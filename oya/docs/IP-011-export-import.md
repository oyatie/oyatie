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
buck2 build //:quality-lane-registry-authority-check # lane=export-sandbox-conformance --microservice docs
buck2 build //:quality-lane-registry-authority-check # lane=ooxml-import-fidelity --microservice docs
```

## References

- ADR-DOCS-0003 (export pipeline architecture).
- ADR-DOCS-0006 (DOCX import fidelity matrix).
- veraPDF — `verapdf.org` (PDF/A validator).
- gVisor — `gvisor.dev`.
- Pandoc — `pandoc.org`.
- WeasyPrint — `weasyprint.org`.
- Chromium release notes — `chromiumdash.appspot.com/`.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for export/import; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For export/import, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/catalog/oya-docs-export-import-kernel.yaml`.
Domain entities or operational surfaces: ExportJob, ImportJob, FormatProfile.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind export/import to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `export/import` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `export/import` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `export/import` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
