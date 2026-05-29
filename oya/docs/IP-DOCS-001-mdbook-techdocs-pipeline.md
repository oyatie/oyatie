# IP-DOCS-001 — mdbook + Backstage TechDocs pipeline

> ADR anchor: ADR-0203, ADR-0170.
> Owner: `oya-docs`.
> Estimate: 4 days.

## Goal

Build the Tier-1 (mdbook) and Tier-2 (Backstage TechDocs)
rendering pipelines so internal engineers can browse the same
Markdown source through two surfaces.

## Why this IP

Per ADR-0203 three tiers serve three audiences. Tiers 1 + 2
share the Markdown source-of-truth; only Tier 3 (SvelteKit)
needs a separate content bundle. This IP wires the shared
pipeline.

## Tasks

### 1. mdbook source root

- `docs/standards/`, `docs/operators/`, `docs/decisions/` are
  the canonical mdbook source roots.
- `book.toml` declares the chapters + theme.

### 2. mdbook build

- CI job `docs.mdbook.render` runs `mdbook build` on every
  PR; failure blocks merge.
- Render published to internal docs bucket on `dev`
  promotion.

### 3. Backstage TechDocs

- Per-µservice `mkdocs.yml` declares the Markdown sources for
  that service's catalog entry.
- `techdocs-cli generate` runs as part of the cluster Helm
  release for the Backstage µservice.

### 4. Tests

- `mdbook build` returns 0 on a sample chapter set.
- Sample µservice TechDocs renders end-to-end.

## Acceptance criteria

- Every standards / decisions doc renders in mdbook.
- Every µservice docs surface renders in TechDocs.

## References

- ADR-0203, ADR-0170.
- mdbook upstream.
- Backstage TechDocs upstream.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for mdBook/TechDocs; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For mdBook/TechDocs, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/PHASE-01-DOCS-FOUNDATION.md`.
Domain entities or operational surfaces: documentation publication pipeline.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind mdBook/TechDocs to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `mdBook/TechDocs` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `mdBook/TechDocs` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `mdBook/TechDocs` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
