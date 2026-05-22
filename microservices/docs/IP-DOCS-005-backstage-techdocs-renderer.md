# IP-DOCS-005 — Backstage TechDocs renderer adapter

> ADR anchor: ADR-0203, ADR-0170.
> Owner: `oya-docs`.
> Estimate: 3 days.

## Goal

Wrap Backstage TechDocs behind an adapter so a Phase-2
in-house `oya-developer-portal` (ADR-0203 trigger-conditional)
can swap in without rewriting service catalog integration.

## Why this IP

ADR-0203 §"In-house roadmap" names `oya-developer-portal` as
a Phase-2 conditional build. To preserve the option, the
substrate keeps TechDocs behind an adapter rather than baking
TechDocs assumptions into business logic.

## Tasks

### 1. Adapter trait

- `DocsCatalogAdapter` trait declares the operations
  Backstage TechDocs supplies (catalog query, doc render,
  search).

### 2. TechDocs implementation

- `BackstageTechDocsAdapter` implements the trait against
  the Backstage HTTP API.

### 3. Tests

- Integration test against a local Backstage instance.

## Acceptance criteria

- Every consumer of docs catalog data goes through the trait.
- Swapping the adapter (Phase 2) requires no consumer
  changes.

## References

- ADR-0203, ADR-0170.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for Backstage TechDocs renderer; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For Backstage TechDocs renderer, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/catalog/`.
Domain entities or operational surfaces: service catalog TechDocs integration.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind Backstage TechDocs renderer to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `Backstage TechDocs renderer` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `Backstage TechDocs renderer` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `Backstage TechDocs renderer` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
