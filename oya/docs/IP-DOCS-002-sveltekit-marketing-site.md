# IP-DOCS-002 — SvelteKit marketing + public-docs site

> ADR anchor: ADR-0203, ADR-0185.
> Owner: `oya-docs`.
> Estimate: 8 days.

## Goal

Build the Tier-3 (public-facing) docs surface on SvelteKit.
Source: the Markdown content bundle emitted from the same
files mdbook + TechDocs consume.

## Why this IP

External customers + prospects + search engines consume a
different shape of docs than internal engineers. SvelteKit is
the Phase-1 client stack per ADR-0185 — using it here keeps
the substrate aligned.

## Tasks

### 1. Content bundle generator

- A generator script reads `docs/` + selected
  `microservices/<ms>/` Markdown.
- Emits a per-page JSON bundle the SvelteKit site loads.
- Generator runs at CI time; the bundle ships as a release
  artifact.

### 2. SvelteKit app

- Standard SvelteKit project at
  `microservices/docs/sveltekit/`.
- Routes for marketing pages, onboarding tutorials, API
  reference (Redoc / Stoplight), and search.

### 3. Search

- Build-time search index (Lunr or similar) generated from
  the content bundle.
- Per-tier search index (don't conflate internal Tier-1 with
  external Tier-3).

### 4. Tests

- Build-time: `pnpm build` succeeds.
- Lighthouse score ≥ 95 on the home page.

## Acceptance criteria

- Public-facing site renders with the substrate's marketing
  copy + API reference.
- Same Markdown source backs all three tiers (no drift).

## References

- ADR-0203, ADR-0185.
- SvelteKit upstream.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for SvelteKit docs site; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For SvelteKit docs site, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/README.md`.
Domain entities or operational surfaces: tenant-facing docs web surface.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind SvelteKit docs site to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `SvelteKit docs site` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `SvelteKit docs site` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `SvelteKit docs site` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
