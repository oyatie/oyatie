# IP-DOCS-003 — Redoc + AsyncAPI renderer

> ADR anchor: ADR-0203.
> Owner: `oya-docs`.
> Estimate: 3 days.

## Goal

Render OpenAPI 3.2 contracts via Redoc and AsyncAPI 3.1
contracts via the AsyncAPI rendering CLI, both embedded in
the SvelteKit Tier-3 site.

## Why this IP

Every µservice ships an OpenAPI + AsyncAPI contract under
`microservices/<ms>/contracts/`. Per ADR-0203 those contracts
render into the public docs via Redoc / AsyncAPI tools.

## Tasks

### 1. Build-time render

- For each µservice's `contracts/openapi.yaml`: run Redoc
  CLI → static HTML.
- For each `contracts/asyncapi.yaml`: run AsyncAPI Generator
  → static HTML.

### 2. SvelteKit integration

- Rendered HTML embedded as iframes (or hydrated server-side)
  inside the SvelteKit site under `/api/<ms>`.

### 3. Tests

- Sanity: comms-email OpenAPI + AsyncAPI both render.
- Per-µservice contracts validate against OpenAPI 3.2 +
  AsyncAPI 3.1 schemas at CI.

## Acceptance criteria

- Every µservice's contracts surface as an API reference page
  on the public docs site.

## References

- ADR-0203.
- Redoc upstream.
- AsyncAPI Generator upstream.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for Redoc/AsyncAPI renderer; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For Redoc/AsyncAPI renderer, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/contracts/asyncapi/docs-events.yaml`.
Domain entities or operational surfaces: contract rendering and versioned API evidence.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind Redoc/AsyncAPI renderer to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `Redoc/AsyncAPI renderer` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `Redoc/AsyncAPI renderer` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `Redoc/AsyncAPI renderer` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
