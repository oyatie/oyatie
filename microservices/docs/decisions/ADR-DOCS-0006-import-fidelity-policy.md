---
id: ADR-DOCS-0006
title: DOCX import fidelity policy — best-effort with named edge-case test matrix; strict-round-trip rejected
microservice: docs
status: Accepted
date: 2026-05-17
owner: axis-docs + council-product
deciders: council-architecture, axis-docs, council-product, ops-security
supersedes: []
superseded_by: []
related: [ADR-0131, ADR-0133, ADR-DOCS-0002, ADR-DOCS-0003]
related_artifacts:
  - microservices/docs/PRD.md (FR-10, AC-03)
  - microservices/docs/IP-011-export-import.md
  - microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md
purpose: |
  Settle the DOCX import fidelity tier: best-effort (surface unsupported
  features) vs strict-round-trip (refuse import if any feature would be lost).
  Closes PRD AC-03 + FR-10.
doc_status: published
---

# ADR-DOCS-0006: DOCX import fidelity — best-effort with named edge-case test matrix

## Status

Accepted — 2026-05-17.

## Context

The docs µservice imports DOCX (per PRD FR-10) and exports DOCX (per FR-09). The canonical document model is block-based (per ADR-DOCS-0002); DOCX/OOXML is a document-tree model. The two models are not isomorphic — OOXML expresses features that have no block-based equivalent (legacy field codes, OLE objects, content controls, structured-document tags, custom XML data parts, comments-with-replies, math-OMML mixed inline, vector graphics in DrawingML, etc.).

Per PRD AC-03: OOXML round-trip must preserve ≥ 95% of features on the Microsoft test corpus. ECMA-376 (OOXML) is a multi-thousand-page spec; full fidelity is not achievable by any open-source conversion path. Pandoc 3.6.0 (per ADR-DOCS-0003) covers most common features but has known gaps.

Three fidelity-policy tiers exist:

1. **Strict round-trip**: refuse import if any feature would be lost. Pro: zero data loss. Con: refuses ~30% of real-world DOCX files (per industry conversion-tool experience).

2. **Best-effort with explicit warnings**: import everything that converts cleanly; surface unsupported features as a fidelity warning + per-feature line item. Tenant sees what was lost. Pro: high acceptance; honest disclosure. Con: tenant must review fidelity report.

3. **Silent lossy import**: import what we can; silently drop the rest. Pro: smoothest UX. Con: violates `feedback_no_silent_regression`; legal-doc tenants particularly damaged.

Per `feedback_no_silent_regression.md`, silent loss is forbidden. The choice is between strict and best-effort.

## Decision

Adopt **best-effort import fidelity with a named edge-case test matrix**:

### Fidelity floor

- Per AC-03, ≥ 95% OOXML round-trip fidelity on the Microsoft corpus (Pandoc native + oyatie additions).
- Per-import job emits a fidelity ratio in `[0.0, 1.0]` + an array of unsupported features detected.
- Below the 95% floor on a per-import basis: the import succeeds but a Sev-3 alert fires on aggregate trend (per `failure-modes.md` FM-11).

### Named edge-case test matrix

A maintained corpus of OOXML edge cases with explicit per-feature handling:

| Feature | Handling | Fidelity tier |
|---|---|---|
| Bold, italic, strikethrough, underline | Preserved | 100% |
| Headings (1-3) | Preserved | 100% |
| Headings (4-9) | Mapped to heading_3 + bold/size attribute | best-effort |
| Ordered + unordered lists | Preserved | 100% |
| Tables (basic) | Preserved | 100% |
| Tables (nested) | Preserved up to 3 levels | best-effort |
| Tables (with merged cells across rows + columns) | Preserved | 95% |
| Images (embedded) | Preserved; re-encoded to strip EXIF | 100% |
| Images with text wrap | Preserved as block image; text-wrap dropped | best-effort |
| Math equations (OMML) | Converted to KaTeX where possible; surfaced as fidelity warning otherwise | best-effort |
| Footnotes + endnotes | Preserved as callout blocks | best-effort |
| Comments (basic) | Imported as docs comments | 100% |
| Comments with replies | Preserved as thread | 100% |
| Track-changes (revisions) | Imported as suggestions | 100% |
| Hyperlinks | Preserved | 100% |
| Cross-references | Preserved when target stable; converted to literal text otherwise | best-effort |
| Field codes (legacy) | Converted to literal text; surfaced as fidelity warning | best-effort |
| Content controls / structured-document tags | Surfaced as fidelity warning; literal text imported | unsupported |
| Custom XML data parts | Surfaced as fidelity warning; dropped | unsupported |
| OLE objects (embedded spreadsheets / charts) | Surfaced as fidelity warning; placeholder image inserted | unsupported (M03+1) |
| Vector graphics (DrawingML) | Rasterised to PNG; surfaced as fidelity warning | best-effort |
| VBA macros | REFUSED at parser; security-relevant; per ADR-DOCS-0003 sandbox + Hyrum #2 | refused |
| ActiveX controls | REFUSED at parser; security-relevant | refused |
| Linked external files | REFUSED at parser (security-relevant; would require network egress in sandbox which is forbidden) | refused |
| OOXML namespace extensions (vendor-specific) | Surfaced as fidelity warning; treated as literal text | best-effort |

### Surface to user

Every import emits an `ImportJob` record with:
- `fidelity_ratio: f32` — overall ratio in [0, 1].
- `unsupported_features: Vec<String>` — list of detected feature codes.
- `import_warnings: Vec<ImportWarning>` — per-feature line items with location.

The editor renders an "Import fidelity report" UI surface that the user can inspect; tenant can choose to accept or roll back the import.

### CI lane

`oya-governance-ooxml-import-fidelity` runs the Microsoft OOXML test corpus + oyatie's additional corpus on every PR; fails if aggregate fidelity drops below 95%.

### Versioning

Pandoc version (per ADR-DOCS-0003) is pinned. Pandoc upgrades require fresh corpus drill; if upgrade improves fidelity → ship; if upgrade regresses → block. Migration Hyrum #2 documents the byte-stability surface for cross-version DOCX exports.

## Alternatives Considered

### A. Strict round-trip (refuse import if any feature would be lost)

- Pros: zero data loss; legal-doc tenants confident.
- Cons: refuses ~30% of real-world DOCX files. Industry experience (Pandoc + LibreOffice + Collabora) confirms this rejection rate. Tenants migrating from Microsoft Word would face ~30% of their docs failing to import; unworkable for migration tier.
- Rejected: incompatible with migration utility.

### B. Silent lossy (drop unsupported features without warning)

- Pros: smoothest UX.
- Cons: violates `feedback_no_silent_regression`; legal-doc + clinical-notes tenants damaged when content silently lost; tenant trust breach.
- Rejected: forbidden by no-silent-regression principle.

### C. Tenant-configurable strict/best-effort flag (per-tenant)

- Pros: tenants choose their own fidelity floor.
- Cons: adds complexity to onboarding; most tenants don't know which they want; admin overhead.
- Rejected: best-effort with explicit reporting subsumes this; tenants who need strict can review the fidelity report and decide to roll back.

### D. Fidelity tier per-feature with tenant override (subset of best-effort)

- Pros: even finer control.
- Cons: matrix exposure complexity; UX overload; per-tenant policy proliferation.
- Rejected: best-effort with named matrix is sufficient.

## Consequences

### Architectural

- `oya-docs-export-import-adapter-pandoc` carries the OOXML feature-mapping table.
- `oya-docs-export-import-domain` enforces fidelity-warning emission.
- ImportJob entity carries `fidelity_ratio` + `unsupported_features` + `import_warnings`.
- Editor UI renders the fidelity report.
- CI lane `oya-governance-ooxml-import-fidelity` enforces the 95% floor on the test corpus.

### Downstream impact

1. **PRD AC-03** — directly specified (≥ 95% floor on Microsoft corpus).
2. **PRD FR-10** — import flow specified (with fidelity report).
3. **`failure-modes.md` FM-11** — covered (fidelity below threshold).
4. **`runbooks/export-pipeline-failure-pandoc-rollback.md` Section D** — per ADR-DOCS-0006 fidelity drift mitigation.
5. **migration-from-connect.md Hyrum #2** — DOCX byte-stability across Pandoc 2.x → 3.x.
6. **competitor-parity-matrix.md** — OOXML round-trip fidelity ≥ 95% claim is verified.

### SLOs gaining new dimensions

- `docs.ooxml_import_fidelity_ratio` — avg across imports; alert if < 0.95.
- `docs.ooxml_import_lost_feature_count` — per-feature counter.
- `docs.ooxml_import_failure_rate` — operational failure rate (distinct from fidelity).

### Risk register

- **Risk**: Pandoc upgrade regresses on a feature in the named matrix. **Mitigation**: pre-upgrade corpus drill; rollback runbook.
- **Risk**: Tenant relies on a feature in the "unsupported" tier; data effectively lost. **Mitigation**: fidelity warning surfaced in editor + Workflow event for tenant automation.
- **Risk**: New OOXML feature shipped by Microsoft that's not in our matrix. **Mitigation**: quarterly Microsoft OOXML release review; matrix updated; corpus re-baseline.

## References

- PRD `microservices/docs/PRD.md` FR-10, AC-03.
- ADR-DOCS-0002 (block-type system; canonical form).
- ADR-DOCS-0003 (export pipeline architecture; Pandoc backend).
- ECMA-376 — OOXML reference (multi-part).
- Pandoc OOXML writer + reader notes — `pandoc.org/MANUAL.html#docx`.
- Microsoft OOXML test corpus — `learn.microsoft.com/openspecs/office_standards/ms-docx/`.
- Industry experience: ONLYOFFICE + Collabora OOXML conversion reports.
- `feedback_no_silent_regression.md` — no-silent-regression principle.
- `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md` §D.
- `microservices/docs/failure-modes.md` FM-11.
