---
id: ADR-0065
status: Superseded
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0065: Documentation as Leptos web pages with machine-readable JSON/YAML/TOML co-emission

> **Status:** Accepted
> **Owner:** `council-architecture` + `axis-foundry`
> **Date:** 2026-05-13
> **Related:** ADR-0056, ADR-0058, ADR-0059, ADR-0061, ADR-0063, ADR-0064

---

## Context

Per user instruction 2026-05-13: "I want to maintain all our documentation (currently in markdown) in Leptos. I want to build auto generation of the documentation into it. And have machine readable optimized JSON, YAML or TOML. This is a shift in direction. We now maintain all our docs as web pages."

Today every artifact in the doc set (per ADR-0063) is hand-authored markdown: ADRs, PRDs, microservice records, BC registrations, phase-specs, impl-plans, milestone READMEs, pack manifests, evidence bundles. Two consumer surfaces use them differently:

- **Humans** (founder, council, GTM) want browsable, searchable, cross-linked web pages — not raw markdown.
- **Agents** (executors, fitness lanes, autopilot, ralplan, codex review) want machine-readable structured data — not markdown re-parsed each time.

The single-source-of-truth markdown serves neither audience optimally. The shift: markdown remains the **authoring** format; Leptos pages + structured manifests are the **published** format.

---

## Decision

### 1. Triple-output documentation pipeline

Every doc artifact ships in three forms:

| Form | Audience | Path |
|---|---|---|
| **Markdown** (source) | Authors (humans + agents writing docs) | `docs/**/*.md` (current state preserved) |
| **Leptos web pages** (rendered) | Human readers; search; cross-linking; live diff vs commit | served by `oya-docs-portal-rest` µservice |
| **Machine-readable** (JSON / YAML / TOML) | Agents, fitness lanes, autopilot, ralplan, codex | emitted to `docs/.generated/` + served via `oya-docs-portal-rest /api/v1/...` |

Markdown is the source of truth. Pages and structured outputs are deterministic derivations regenerated on every commit; they MUST NOT be hand-edited (CI lane enforces).

### 2. New µservice: `docs` (catalog entry)

Add `docs` to the flat µservice catalog (per ADR-0058) with BCs:

- `portal` — Leptos web app serving rendered pages (B2B + Personal contexts; same dual-context model as `connector` per ADR-0208 inheritance)
- `generator` — parser + transformer: markdown + frontmatter → AST → Leptos components + JSON/YAML/TOML emitter
- `search` — full-text + semantic search over the doc corpus (consumes `search` substrate + `vector` substrate)
- `cross-ref` — bidirectional link graph (ADR ↔ PRD ↔ phase-spec ↔ impl-plan ↔ µservice record); supports "what cites this ADR", "what depends on this phase-spec"
- `manifest` — single-pane structured manifest of every doc artifact (`docs/.generated/manifest.json`) for agents to query without re-parsing
- `live-diff` — shows commit-by-commit diff of doc state; integrates with `audit-chain` substrate for signed change history

Naming per BNF v4.1: `oya-docs-portal-*`, `oya-docs-generator-*`, `oya-docs-search-*`, `oya-docs-cross-ref-*`, `oya-docs-manifest-*`, `oya-docs-live-diff-*`.

### 3. Frontmatter schema (structured source)

Every markdown doc carries YAML frontmatter that the generator parses into typed records. The frontmatter is the structured source-of-truth; the markdown body is the human-readable elaboration.

Required fields per doc class:

```yaml
# ADR (existing pattern, formalized)
doc_class: ADR
adr_id: ADR-0065                  # NNNN
status: Proposed | Accepted | Rejected | Superseded | Retired
date: 2026-05-13
owner: <team-id>
supersedes: [ADR-####, ...]
superseded_by: [ADR-0709]
related: [ADR-####, ...]

# PRD
doc_class: PRD
microservice: <kebab>
status: Draft | Accepted | Shipped
date: 2026-05-13
audience: internal | b2b | b2c
owner_team: <team-id>
canonical_base_only: true | false
adrs_cited: [ADR-####, ...]
performance_targets:
  - { dimension: "API p99 read", target: "≤50ms" }
horizontal_scalability:
  state_strategy: stateless | postgres-citus | object-storage | persistent-volume | mixed
  active_active_compatibility: stateless-compatible | single-writer-compatible
competitors: ["Stripe", "Workday", ...]

# Microservice record
doc_class: Microservice
microservice: <kebab>
cluster: workforce | healthcare | fintech | industrial | connector | hospitality | substrate | cloud | foundry | application | adapter
status: planned | shipped | retired
introducing_phase: M02-P01
bounded_contexts: [<bc-name>, ...]
layer_crates_planned: [kernel, domain, application, adapter, rest, worker, app, sdk]
kr_pack_scope: true | false
kr_pack_material_scope: true | false

# Bounded-context registration
doc_class: BoundedContext
microservice: <kebab>
bounded_context: <kebab>
status: Proposed | Active | Retired
ownership_pillar: org | person | system
crate_paths: [crates/oya-<ms>-<bc>-kernel, ...]
port_traits: [<TraitName>, ...]
events: [<EventName>, ...]

# Phase-spec (already has frontmatter; ADR-0063 §4 enforced)
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02b-substrate
phase: P20-ci-lanes-operational
status: Proposed | InProgress | Complete
acceptance_lanes: [<lane-id>, ...]
depends_on:
  - { milestone: M02, phase: P01-foundry-engine-consolidation, reason: "..." }
entry_gate: |
  ...
exit_gate: |
  ...

# Impl-plan
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P20-ci-lanes-operational
impl_plan_id: IP-001-ci-lanes-statelessness-shardability
status: pending | in-progress | complete
blocked_by: [<impl_plan_id>, ...]
acceptance_lanes: [<lane-id>, ...]

# Milestone README
doc_class: MilestoneReadme
template_id: TPL-MILE-README
milestone_id: M02b-substrate
status: Proposed | Active | Complete
entry_gate: ...
exit_gate: ...
bominal_adrs_inherited: [ADR-####, ...]
oyatie_adrs_cited: [ADR-####, ...]

# Pack manifest (already YAML; canonical)
# Pack overview, evidence bundle, etc.: similar pattern
```

CI lane `oya-check-documentation` (existing LEAN-A5) verifies frontmatter schema conformance per doc_class. Lane extended (M02-P20 IP-005 scope) to:

1. Validate frontmatter against a JSON Schema per doc_class.
2. Emit `docs/.generated/manifest.json` indexing every doc by (doc_class, id, path, status, related-ids).
3. Emit per-doc `docs/.generated/<path>.json` containing parsed frontmatter + parsed body AST.
4. Run the cross-ref graph builder.

### 4. Generator pipeline

`oya-docs-generator-*` is a pure Rust binary (no async; deterministic; sandboxed per workspace) wired into:

- **Build-time**: invoked by `cargo xtask docs-build` (alias `oya doc build`). Emits Leptos page components under `crates/oya-docs-portal-app/src/pages/`. Emits machine-readable bundles under `docs/.generated/`.
- **CI**: `lean-a5-documentation` lane invokes generator in `--check` mode; PR fails if generated output diverges from source (forcing the generator to be deterministic + regen on every change).
- **Watch mode** (developer): `oya doc serve` runs generator + Leptos dev server with hot reload on `docs/**/*.md`.

Generator stages:

1. **Parse**: pulldown-cmark for markdown body; serde_yaml for frontmatter.
2. **Validate**: frontmatter schema per doc_class; required-field check.
3. **Transform**: resolve cross-refs (`[[ADR-0064]]` → typed link); compute back-references (which docs cite this); compute breadcrumb path; extract code blocks with syntax highlights.
4. **Emit Leptos**: one component per doc; uses `oya-docs-portal-kernel` types so the portal app composes them.
5. **Emit JSON/YAML/TOML**:
   - `docs/.generated/manifest.json` — top-level index, every doc
   - `docs/.generated/<path>.json` — full per-doc structured record (frontmatter + body AST + cross-refs + back-refs)
   - `docs/.generated/manifest.yaml` — same as `.json` in YAML for tooling preferring YAML
   - `docs/.generated/manifest.toml` — Cargo-tooling-friendly TOML for inline workspace lookups

`docs/.generated/` is **gitignored** in source workflow but **committed** in CI output workflow (`m02-exit-checklist.md` + CI artifact upload). Sources of truth never live under `.generated/`.

### 5. Leptos web app shape (`oya-docs-portal-*`)

| Crate | BNF layer | Responsibility |
|---|---|---|
| `oya-docs-portal-kernel` | kernel | Doc record types (Adr, Prd, Microservice, BoundedContext, PhaseSpec, ImplPlan, MilestoneReadme, PackManifest, EvidenceBundle), `DocStore` port trait, `SearchStore` port trait, `CrossRefStore` port trait |
| `oya-docs-portal-domain` | domain | Cross-ref graph algorithms (transitive supersession, citation graph closure); navigation breadcrumbs; doc-status state machine |
| `oya-docs-portal-application` | application | Use-cases: `BrowseDocByPath`, `SearchDocs`, `ResolveCrossRef`, `RenderManifestForAgent`, `DiffDocAtCommits` |
| `oya-docs-portal-adapter` | adapter | `JsonManifestStore` (reads `docs/.generated/`), `PgroongaSearchStore`, `PgvectorSearchStore`, `GitHistoryStore` (for live-diff) |
| `oya-docs-portal-rest` | rest | OpenAPI: `/api/v1/docs/{path}`, `/api/v1/search`, `/api/v1/cross-ref/{id}`, `/api/v1/manifest`, `/api/v1/diff/{commit-a}/{commit-b}` |
| `oya-docs-portal-leptos` | rest (Leptos SSR + SPA islands) | Web pages; renders pages from `oya-docs-portal-application`; Leptos SSR pre-auth + SPA islands post-auth (per Bominal ADR-0209 client-stack inheritance) |
| `oya-docs-portal-app` | app | Composition root binary |
| `oya-docs-generator` (binary) | cli | The build-time generator |

**Dual-context** (per Bominal ADR-0208 inheritance):

- B2B context: org docs visible to tenant org members (per Cedar policy); proprietary content (per-tenant ADRs / customized regulatory bindings) gated.
- Personal context: oyatie's open ADRs / PRDs / masterplan are public; gated content lives behind tenant SSO.

### 6. Machine-readable surface contract

The agent / fitness-lane / autopilot consumer contract (per ADR-0063 + Foundry primitives):

```typescript
// docs/.generated/manifest.json schema
{
  "generated_at": "2026-05-13T08:00:00Z",
  "generated_commit": "<sha>",
  "schema_version": 1,
  "docs": [
    {
      "doc_class": "ADR" | "PRD" | "Microservice" | ...,
      "id": "ADR-0064" | "microservice-payroll" | ...,
      "path": "docs/decisions/ADR-0064-...md",
      "frontmatter": { /* typed per doc_class */ },
      "title": "...",
      "status": "Accepted",
      "outgoing_refs": [{ "to_id": "ADR-0058", "ref_kind": "related" }],
      "incoming_refs": [{ "from_id": "MASTERPLAN", "ref_kind": "cited-in" }],
      "body_summary": "first 280 chars of body",
      "headings": [...],
      "code_blocks_languages": ["rust", "sql", "yaml"]
    },
    ...
  ],
  "indexes": {
    "by_doc_class": { "ADR": [...], "PRD": [...], ... },
    "by_microservice": { "payroll": [...], "hr": [...], ... },
    "by_milestone": { "M02b-substrate": [...], "M07-first-tenant": [...], ... },
    "by_pack": { "kr": [...] }
  }
}
```

Per-doc detail in `docs/.generated/<path>.json` includes the full body AST so agents reconstruct without re-parsing markdown.

### 7. Migration: existing markdown → generated portal

- **Phase A (immediate; this commit)**: ADR + masterplan update; new `docs` µservice in catalog; goal artifact reflects shift.
- **Phase B (M02-P20 IP-005 extension)**: `oya-check-documentation` extended to emit `docs/.generated/manifest.json` + per-doc records. Frontmatter schema enforcement (per doc_class) starts as report-only.
- **Phase C (M02-P21)**: `oya-docs-generator` Rust binary authored; emits Leptos page components + machine-readable bundles. Generator deterministic + CI-checked.
- **Phase D (M02-P19 extension)**: `oya-docs-portal-*` crates authored (kernel + domain + application + adapter + leptos + rest + app). Application B2B shell exposes Docs Portal as the first product (alongside Workflow Studio).
- **Phase E (M03-P06 or new dedicated phase)**: Docs Portal live in Stage 0 OCI ARM64 cell; tenant SSO scoping; search + vector indexes seeded; cross-ref graph live.

Existing markdown remains the source-of-truth throughout the migration; the portal is a derived view. No conversion-cost lock-in.

### 8. CI enforcement

- `oya-check-documentation` (LEAN-A5; existing): extended to validate frontmatter schema per doc_class; emit `docs/.generated/manifest.json`; verify generator output is deterministic (re-run produces identical bytes).
- New gate: `lean-a6-docs-generated-consistency` — CI verifies `docs/.generated/` (re-emitted) matches the version committed in the PR; PRs that touch markdown but don't regenerate fail.

---

## Consequences

**Positive:**

- Humans get browsable, searchable, cross-linked docs.
- Agents get a deterministic machine-readable surface (`docs/.generated/manifest.json`) — no markdown re-parsing per query.
- Cross-refs become first-class (back-references for every cited ADR/PRD/phase-spec).
- Tenant-scoped doc visibility becomes possible (per-tenant ADRs gated by Cedar; org docs vs personal context).
- Workflow Studio + Docs Portal share the same Leptos SSR/SPA stack — single client-tier pattern.
- Auto-generation eliminates drift between hand-maintained tables (e.g., "stale 14 lanes" wording the critic flagged in earlier rounds).

**Negative:**

- Generator development cost (~3 person-weeks per impl-plan estimate); shifts M02-P20/P21/P19 scope.
- `docs/.generated/` files in CI workflow (committed only post-CI) — adds repo size; mitigated by .gitignore in source workflow.
- Frontmatter schema strictness may friction casual ADR authoring; mitigated by `oya doc lint` developer tool with autofix suggestions.

**Neutral:**

- Inherits Bominal ADR-0209 Leptos client-stack policy.
- Inherits ADR-0063 doc-coverage contract; extends it with schema + generated-output validation.

---

## Alternatives considered

| Alternative | Why rejected |
|---|---|
| **Hugo / Docusaurus / mdbook + JSON sidecar** | External tool chains; not part of oyatie Rust stack; can't integrate with `oya-foundry-*` fitness lanes natively; Leptos pattern reused from Workflow Studio + makes a single client-tier story |
| **Markdown-only (status quo)** | Humans get the worst-of-both; agents must re-parse markdown per query; cross-refs are hand-maintained |
| **Pure-Leptos hand-authored (no markdown source)** | Friction for ADR/PRD authoring; loses git-blame trail on prose; bypasses the existing 60+ ADR / 75+ µservice doc base |
| **JSON-only source (no markdown)** | Loses human readability; harder to git-diff; loses "view raw" path for transparency |
| **Per-µservice docs site** | Fragmentation; cross-µservice references collapse; doesn't compose into the single-pane masterplan view |

---

## Compliance

CI lanes:

- `lean-a5-documentation` (extended; existing): frontmatter schema + generator output validation
- `lean-a6-docs-generated-consistency` (new; M02-P20 scope): committed `docs/.generated/` matches regenerated output

Owner team: `axis-foundry` (generator + lanes) + `council-architecture` (frontmatter schema design) + `gtm-customer-success-kr` (KR pack overlay doc workflow when KR pack ships).

First green window: M02-P22 (doc-coverage `--blocker` includes generator validation).

---

## References

- ADR-0056 (BNF v4.1)
- ADR-0058 (flat catalog; `docs` enters the catalog)
- ADR-0061 (Application B2B shell; Docs Portal as first product alongside Workflow Studio)
- ADR-0063 (documentation set coverage; extended to schema + generated output)
- ADR-0064 (canonical base + localization packs; pack overlay docs participate in the portal)
- Bominal ADR-0209 (Leptos client-stack policy; inherited)
- `docs/MASTERPLAN.md` §2.1 catalog (adding `docs` µservice)
- `docs/architecture/product-graph.md` + `product-graph.html` (the M01-M12+ topology — first artifact authored under the new doc-portal pattern; `product-graph.html` already proves the standalone-HTML pattern works)
