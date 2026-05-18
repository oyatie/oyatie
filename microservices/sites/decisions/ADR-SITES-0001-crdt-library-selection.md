---
id: ADR-SITES-0001
status: Accepted
date: 2026-05-17
microservice: sites
deciders: axis-sites, council-architecture, axis-docs, axis-sheets, axis-slides, axis-workflow-studio
owner: axis-sites + council-architecture
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-WS-0001
  - ADR-DOCS-0001
related_artifacts:
  - microservices/sites/PRD.md §"Bounded Contexts" → block BC
  - microservices/sites/IP-004-block-bc-and-loro.md
  - microservices/sites/policy/editor-isolation.md Invariant 5
purpose: |
  Choose the CRDT library powering the `block` bounded-context's
  concurrent-edit substrate. Decision must align with sibling
  collab-bearing µservices (`docs`, `sheets`, `slides`,
  `workflow-studio`) for cross-µservice consistency and shared
  operational substrate.
---

# ADR-SITES-0001: CRDT library — Loro 1.x; Yjs rejected; Automerge rejected; in-house OT rejected

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The sites µservice's `block` bounded-context needs concurrent-edit
support so multiple editors can co-author a page in real time. The
block-store data model is portable-text per ADR-SITES-0005; the CRDT
library must support tree-shaped op-logs over portable-text node
structure.

Per ADR-0132 (no-suite forward-policy) AND the cross-µservice
collab-substrate consistency principle (`feedback_workflow_is_shared.md`
+ `feedback_flat_product_catalog.md`), the CRDT library choice MUST be
consistent across `sites` + `docs` + `sheets` + `slides` +
`workflow-studio` — agents and tenants must not encounter divergent
collab semantics across the suite.

The four production-grade CRDT libraries are:
1. **Loro** (Rust + WASM; MIT). Active upstream as of 2026-05;
   tree + text + map + list CRDTs; Rust-native (fits our µservice
   substrate); WASM browser binding for the editor SDK; v1.x stable.
2. **Yjs** (JS; MIT). The most widely-deployed CRDT engine on the
   web — powers Notion, Linear, Atlassian Confluence Live Editor,
   and many others. JS-native; tree + text via Y.XmlFragment;
   Rust binding (`yrs`) exists but lags Yjs JS feature set.
3. **Automerge** (Rust + JS; MIT). Mature; document-oriented; less
   tree-flexible than Loro/Yjs; v2.x stable.
4. **In-house Operational-Transform engine** (legacy `oya-connect-sites-*`
   used this; eventually-consistent with merge-conflict prompts).

Decision criteria per `feedback_quality_performance_scalability_bar.md`:
- Rust-native (zero JS runtime in µservice plane).
- Cross-µservice consistent (same engine, same version pin across the
  five collab µservices).
- Browser binding for editor SDK.
- Tree CRDT (block-list reordering + portable-text node mutation).
- Active upstream (CVEs patched; v1.x or later).

## Decision

The sites µservice adopts **Loro 1.x as the CRDT engine** for the
`block` bounded-context, aligned 1:1 with the sibling decisions:
- `microservices/docs/decisions/ADR-DOCS-0001` — Loro 1.x.
- `microservices/sheets/decisions/ADR-SHEETS-0001` — Loro 1.x.
- `microservices/slides/decisions/ADR-SLIDES-0001` — Loro 1.x.
- `microservices/workflow-studio/decisions/ADR-WS-0001` — Loro 1.x.

Concrete bindings:
- Crate: `oya-sites-block-adapter-loro` (backend-qualified per
  ADR-0105 Amendment 3).
- Browser binding: `@oyatie/sites-editor-sdk` depends on
  `loro-crdt` npm.
- Pin: `loro = "1.x"` — exact patch version pinned via workspace
  lockfile; cross-µservice version bumps coordinated via
  ChangeSet across all five collab µservices.
- Per-tenant CRDT log namespace; cross-tenant op refused at session-
  token validation (Invariant 5 of `policy/editor-isolation.md`).
- Op-log signed Ed25519 per Bominal ADR-0028; replay-attack resistant.

## Alternatives Considered

### A. Yjs (with `yrs` Rust binding)

- **Pros**:
  - Largest deployment base; Notion + Linear + Atlassian = scale-proven.
  - Mature browser binding; rich ecosystem.
  - Tree + text + array + map CRDTs.
- **Cons**:
  - JS-first design; the Rust binding `yrs` lags the JS feature set —
    we'd end up rendering features in JS first and back-porting.
  - License (MIT) is fine, but the upstream is Hyrum's-Law-bound to
    Notion's data model; surprise changes when Notion's needs shift.
  - Cross-µservice consistency requires all of docs/sheets/slides/
    workflow-studio to also pick Yjs — historically Loro chosen across
    those µservices because their teams found Rust-native cheaper.
- **Rejected** because the cross-µservice alignment with Loro wins
  over Yjs's ecosystem maturity.

### B. Automerge 2.x

- **Pros**:
  - Mature Rust crate; document-oriented; great for offline-first.
  - MIT; active upstream.
  - Type system bindings (Automerge schema).
- **Cons**:
  - Document-oriented vs tree-oriented; block-list reordering +
    portable-text node mutation is awkward to model.
  - Slower convergence than Loro on tree-shaped workloads (per
    Loro benchmark suite published 2025-12).
  - Same cross-µservice alignment issue as Yjs.
- **Rejected** because document-orientation does not match the block-
  tree shape, and the Loro alignment wins.

### C. In-house Operational-Transform engine (legacy `oya-connect-sites-*` path)

- **Pros**:
  - Already implemented in the legacy stack.
  - Tailored to oyatie's block model.
- **Cons**:
  - Operational Transform is eventually-consistent, not strongly-
    consistent — merge-conflict prompts surface to users.
  - Maintenance cost is entirely on axis-sites; no upstream share.
  - Hyrum's-Law-bound to legacy semantics; can't easily evolve.
  - Cross-µservice consistency impossible (docs/sheets/slides/
    workflow-studio do not use OT).
- **Rejected** as the migration target; preserved in adapter shim
  during the Strangler window per `migration-from-connect.md`
  Hyrum #6 (block serialisation format).

### D. tiptap-collab / Hocuspocus (Yjs-based)

- **Pros**:
  - Managed CRDT relay; less ops effort.
- **Cons**:
  - JS-only; vendor lock-in to tiptap GmbH.
  - License posture conflicts with oyatie sovereignty model.
- **Rejected**.

## Consequences

### Positive

- **Cross-µservice alignment.** Same engine, same version pin, same
  ops + same security model across all five collab µservices.
- **Rust-native µservice plane.** No JS runtime in the µservice; only
  in the browser SDK.
- **Tree-shaped CRDT.** Block-list reordering + portable-text node
  mutation is natural; deterministic convergence per Loro 1.x
  guarantees.
- **Per-tenant CRDT log namespace** at the relay layer; cross-tenant
  op refused — Invariant 5 of `policy/editor-isolation.md`
  structurally enforced.

### Negative

- **Loro is less battle-tested than Yjs.** Smaller deployment base; we
  bear more upstream-issue risk. Mitigation: pin version + run the
  Loro test corpus in CI + contribute upstream regression tests.
- **Pre-1.0 API churn (historical).** Loro reached 1.0 in 2025; the
  1.x line is stable. Mitigation: pin patch version; coordinate
  bumps via cross-µservice ChangeSet.
- **Browser SDK requires WASM binding.** The browser editor SDK
  depends on `loro-crdt` npm (WASM). Mitigation: SDK is a normal
  npm dependency; oyatie web-clients already ship WASM for
  ontology + workflow-studio.

### Operational

- **New CI lane `oya-governance-crdt-tenant-scope`** (BLOCKER from
  M03): validates per-tenant CRDT log namespace; refuses code paths
  that allow cross-tenant op-log replay.
- **Coordinated version bumps.** All five collab µservices share the
  Loro pin; bumps go through a single coordinator ChangeSet that
  retests cross-µservice convergence.
- **Per-µservice relay pods** in the shared `shared-crdt` namespace;
  per-tenant session tokens validate at relay; refused tokens emit
  audit-chain.

### Regulatory

- **GDPR Art. 17 erasure**: Loro CRDT log entries can be erased per
  page; compaction removes erased entries past retention horizon.
- **EU AI Act**: out of scope (CRDT is not AI).
- **KR PIPA Art. 23-2 (sensitive)**: per-tenant log isolation prevents
  sensitive-content leak across tenants.

## Verification

- [ ] **Loro 1.x deterministic convergence** —
  `cargo nextest run -p oya-sites-block-adapter-loro -- crdt_converge`.
- [ ] **Per-tenant log isolation** —
  `cargo nextest run -p oya-sites-block-adapter-loro -- crdt_tenant_scope`.
- [ ] **Cross-µservice version pin alignment** —
  `cargo run -p oya-dev-cli -- gate validate crdt-version-alignment`
  (compares Loro pin across sites/docs/sheets/slides/workflow-studio).

## References

- Loro CRDT — `loro.dev`; v1.x release notes.
- Yjs — `yjs.dev` (rejected reference).
- Automerge — `automerge.org` (rejected reference).
- ADR-0056 (BNF v4.1); ADR-0105 Amendment 3 (backend-qualified
  adapters); ADR-0131; ADR-0132; ADR-0133.
- ADR-WS-0001 (workflow-studio Loro); ADR-DOCS-0001 (docs Loro).
- Bominal ADR-0028 (audit-chain Ed25519).
- `microservices/sites/PRD.md` AC-10.
- `microservices/sites/policy/editor-isolation.md` Invariant 5.
- Conflict-free Replicated Data Types — Shapiro et al. 2011.
