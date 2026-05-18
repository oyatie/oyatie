---
id: ADR-SHEETS-0001
title: CRDT library selection for sheets collaborative editing — Loro 1.x aligned with workflow-studio ADR-WS-0001
microservice: sheets
status: Accepted
date: 2026-05-17
owner: axis-sheets + council-architecture
deciders: council-architecture, axis-sheets, council-design-system, ops-security
supersedes: []
superseded_by: []
related: [ADR-0065, ADR-0105, ADR-0126, ADR-0131]
related_external_adrs: [microservices/workflow-studio/decisions/ADR-WS-0001, microservices/docs/decisions/ADR-DOCS-0001]
related_specs: [/specs/products/sheets.json]
related_artifacts:
  - microservices/sheets/PRD.md (FR-06, AC-06)
  - microservices/sheets/PHASE-01-SHEETS-FOUNDATION.md (IP-005)
  - microservices/sheets/dashboards/collab-and-fanout.json
  - microservices/sheets/IP-005-collab-crdt-loro-aligned-ws-0001.md
  - microservices/workflow-studio/decisions/ADR-WS-0001-crdt-library-selection.md
purpose: Resolve PRD Open Question 1 — choose the CRDT library backing the sheets collab-crdt bounded context, with the never-silent-loss invariant (AC-06) as the load-bearing constraint AND with cross-µservice alignment to workflow-studio's CRDT choice as a load-bearing architectural concern.
doc_status: published
---

# ADR-SHEETS-0001: CRDT library selection — Loro 1.x (aligned with workflow-studio ADR-WS-0001 + docs ADR-DOCS-0001)

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The sheets µservice's `collab-crdt` bounded context (PRD §"Bounded Contexts") requires a conflict-free replicated data type (CRDT) substrate for real-time multi-user editing of cell-grid workbooks. Per PRD §"Tenant Value" Outcome 4 and AC-06, the load-bearing invariant is **"never silent loss"**: two business users editing the same workbook concurrently MUST have their non-conflicting cell-edits merged, and conflicting edits MUST surface an explicit conflict UI — last-writer-wins is forbidden.

Workbooks are conceptually a tree of (workbook → sheets → cells → optional comments + chart + pivot config attachments), with the cell graph as the dominant data shape. Cell-level edits, range-level edits (drag-fill), and named-range / chart / pivot edits all need CRDT-merge semantics.

Performance envelope (PRD §"Performance"):
- Collab cursor sync p99 ≤ 150ms (aligned with workflow-studio collab budget).
- Up to 10 concurrent collaborators per workbook (graceful degradation beyond).
- 100,000 active editor sessions per region (XL tier).
- WASM bundle must stay within sheet-open cold ≤ 400ms CDN-cached budget.

Substrate constraints:
- Rust kernel (ADR-0105 layer enum); CRDT merge engine lives in `oya-sheets-collab-crdt-domain` (pure, deterministic).
- Browser-WASM target (per ADR-0065); CRDT library MUST compile to `wasm32-unknown-unknown`.
- Ephemeral CRDT state in Redis (PRD §"Horizontal Scalability" mixed-state strategy); reconstructable from Postgres on cold-start.
- WebSocket gateway long-lived worker (`-worker` crate) fans out CRDT ops via consistent-hash on `workbook_id`.

Operational constraints:
- The CRDT library is supply-chain critical: a vulnerability in merge logic can corrupt every tenant's authored workbooks.
- License must be permissive (MIT / Apache-2.0).

**Cross-µservice alignment constraint (the determining factor):**

Sheets is one of three sibling µservices that ship real-time collaborative editing surfaces at M03+:
- `workflow-studio` — visual workflow canvas (per ADR-WS-0001 settled on Loro 1.x).
- `docs` — rich-text document editor (per ADR-DOCS-0001 settled on Loro 1.x).
- `sheets` — cell-grid workbook editor (this ADR).

These three µservices share:
- The same Layer-A Redis substrate per pack (cell cluster).
- The same WebSocket gateway operational pattern (lease-coordinated; consistent-hash on per-doc id).
- The same threat model (T-T-01 CRDT op forgery; T-I-04 cross-tenant collab leak).
- The same on-call rotation (axis-collab-experts span all three).
- The same browser-WASM bundle budget concerns (Loro WASM ~250 KB gzip; second instance free under shared-chunk caching).

Choosing a different CRDT library for sheets than workflow-studio + docs would:
- Multiply the supply-chain attack surface (two Rust CRDT dependencies to monitor + patch).
- Multiply the operational expertise required on-call (engineers must learn two CRDT-merge algebras).
- Multiply the property-test corpus (each library has its own correctness-test surface).
- Double the WASM bundle weight in tenants that use both Sheets + Docs (~500 KB gzip vs ~250 KB).
- Lose the ability to share collab-crdt-kernel port traits across the three µservices.

The alignment constraint is **load-bearing**: it is a primary input to the decision rather than a tiebreaker.

## Decision

Adopt **Loro 1.x** (`crates.io/crates/loro`) as the sheets CRDT library, **aligned identically with workflow-studio ADR-WS-0001 and docs ADR-DOCS-0001**, with the following constraints:

1. **Identical version pin** to workflow-studio + docs. Loro version bumps in any of the three µservices require a coordinated upgrade across all three; one µservice's upgrade is gated on the same upgrade landing in the other two.
2. Loro types are wrapped in sheets's own `oya-sheets-collab-crdt-kernel` port traits (`CrdtMergeEngine`, `CrdtState`, `MergeOp`, `Conflict`); the library is an implementation detail of the `-adapter-loro` crate.
3. The Loro Map CRDT backs the workbook → sheets → cells nested structure; Loro Lists back row + column ordering; Loro Text backs cell formula source where applicable (for collaborative formula editing).
4. Conflict surfacing uses Loro's built-in version-vector + frontier API; the `Conflict` entity in the kernel wraps Loro `Frontiers` into a UI-renderable shape.
5. CRDT-to-canonical-cell-graph projection deterministically orders Loro nodes by their stable IDs at the cell-grid-domain boundary.
6. Loro snapshot encoding is used for Redis persistence (`snapshot()` + `import_snapshot()`).
7. Loro version pinning + Ed25519-signed advisory feed monitoring; major-version upgrades require a fresh round-trip-corpus drill against the 100-workbook golden corpus before merge.
8. The sheets-crdt-no-silent-loss CI lane runs Loro's example test suite + sheets's own AC-06 property test (10 concurrent editors, randomized cell-edit interleaving, assertion that every accepted op is reachable from final state OR surfaced as conflict — never silently dropped).
9. **Cross-µservice port-trait sharing**: where compatible, the `CrdtMergeEngine` port trait shape is identical across workflow-studio, docs, and sheets, so an engineer working in any of the three µservices sees the same kernel-level abstraction.

## Alternatives Considered

### Alternative A — Yjs (via yrs, the Rust port)

Yjs is the most-deployed CRDT library in production (Notion, Figma, Linear all use Yjs-class systems); yrs is the Rust port.

- **Pros**
  - Largest production install base; battle-tested.
  - Mature ecosystem; reference implementations for spreadsheet-class workloads exist (e.g., y-quill, y-prosemirror).
  - Targets `wasm32-unknown-unknown`.
- **Cons**
  - **Cross-µservice misalignment** — workflow-studio + docs already settled on Loro per ADR-WS-0001 + ADR-DOCS-0001; choosing Yjs for sheets would introduce a second CRDT library across the three µservices, multiplying every concern enumerated in the §"Cross-µservice alignment constraint" above.
  - yrs lags Yjs feature parity by ~2 minor versions historically.
  - For nested-tree workloads (workbook → sheets → cells), Yjs's nested-map performance is documented to degrade at deep nesting; Loro's tree CRDT is purpose-built for this.
- **Rejected reason**: cross-µservice misalignment is the dominant cost. The technical-merit comparison vs Loro is close enough that the alignment constraint decides.

### Alternative B — Automerge 2.0

Academic-provenance CRDT; Apache-2 licensed.

- **Pros**
  - Strong JSON-document fit; deterministic `save()`.
  - Apache-2.0.
- **Cons**
  - **Cross-µservice misalignment** — same concern as A.
  - WASM bundle size larger than Loro by ~2-3x.
  - History-preservation-first design retains every op in the document forever unless explicitly compacted; multiplies storage for long-lived workbook drafts (problem at scale; pack-us-healthcare workbooks may exist for years).
  - Performance for high-frequency cell-edit interleaving is documented O(n log n) in op count.
- **Rejected reason**: bundle size + cross-µservice misalignment + history-preservation cost vs ephemeral-Redis state model.

### Alternative C — Bespoke Rust CRDT tailored to spreadsheet cell-graph

A sheets-authored CRDT implementing exactly the operations the cell-grid needs.

- **Pros**
  - Zero external dependency; full control.
  - Tailored to cell-graph shape; could fold formula-engine recalc-graph awareness into merge semantics.
- **Cons**
  - **Cross-µservice misalignment maximal** — three custom CRDT implementations vs one shared library.
  - CRDT correctness proofs are research-grade work; bespoke implementations have historically taken multiple years to reach production stability.
  - No external review surface.
- **Rejected reason**: build-vs-buy ratio unfavourable; CRDT correctness is well-trodden academic territory; differentiation lives in cell-grid UX + formula-engine correctness + per-range ACL, not in CRDT-implementation novelty.

### Alternative D — Hand-rolled Operational Transform (OT)

Pre-CRDT industry standard (Google Docs, Etherpad historically).

- **Pros**
  - Mature literature.
  - Server-mediated; simpler invariants when one server is authoritative.
- **Cons**
  - Single authoritative server per document required; fundamentally incompatible with multi-cell, multi-region scale-out.
  - OT transform functions notoriously fragile to add new operation types; the cell-edit op set is open-ended (cell value + formula + format + named-range bindings + chart + pivot all need merge semantics).
  - "Never silent loss" requires explicit conflict resolution on every transform; in practice OT systems fall back to last-writer-wins under load.
  - **Cross-µservice misalignment** — workflow-studio + docs use Loro; introducing OT in sheets is the largest divergence.
- **Rejected reason**: incompatible with horizontal scale-out + AC-06 invariant + cross-µservice alignment. Modern collaborative editors have collectively moved off OT for these reasons.

## Consequences

### Architectural

- The collab-crdt `-domain` crate depends on Loro 1.x (pinned with `^1.0` and a vendor advisory subscription).
- `oya-sheets-collab-crdt-kernel` declares `CrdtMergeEngine`, `CrdtState`, `MergeOp`, `Conflict` port traits with no Loro types in the signature; the Loro adapter lives behind these ports.
- The dsl-emitter consumes the CRDT projection through a single `project_to_canonical_cell_graph(state) -> CanonicalCellGraph` boundary.
- WASM bundle gains ~0 KB additional (Loro already loaded by workflow-studio + docs in tenants that use those products; sheets-only tenants pay the ~250 KB gzip Loro cost once).

### Cross-µservice operational

- **Coordinated Loro upgrade contract**: a Loro version bump in any of {workflow-studio, docs, sheets} requires:
  1. PR landing the bump in one µservice with full property-test + corpus pass.
  2. Companion PRs in the other two µservices landed within the same calendar week.
  3. Joint operational drill (10-user collab across all three products in a synthetic tenant) before promotion to staging.
  4. ADR-SHEETS-0001 + ADR-WS-0001 + ADR-DOCS-0001 all updated with the new pinned version (the three ADRs supersession-link to each other in this respect).
- **Shared on-call expertise**: the axis-collab on-call rotation covers all three µservices; runbooks are cross-linked.
- **Shared port-trait surface**: `CrdtMergeEngine` is shape-identical across all three; an engineer reading the kernel in one product can apply intuition to the others.

### Downstream impact on other µservices and IPs

1. **IP-005 (collab-crdt kernel/domain/adapter)** — adopts Loro as the concrete merge engine.
2. **IP-006 (large-sheet-storage)** — Arrow/Parquet cold-tier blocks are immutable snapshots; CRDT state operates only on hot Postgres tier; per ADR-SHEETS-0003 boundary.
3. **IP-013 (cell-grid app)** — composition wires Loro adapter into editor session.
4. **workflow-studio µservice** — ADR-WS-0001 supersession links to this ADR; coordinated upgrade contract applies.
5. **docs µservice** — ADR-DOCS-0001 supersession links to this ADR.
6. **cell µservice** — unaffected at cell-storage substrate; cell µservice receives canonical cell-graph projections only.
7. **observability µservice** — `collab-and-fanout.json` dashboard gains a new dimension `crdt_library=loro` for upgrade-rollout observation across all three µservices.

### SLOs gaining new dimensions

- `sheets.collab_cursor_sync_p99_ms` — tagged with `crdt_library=loro`.
- `sheets.collab_silent_loss_count` — Sev-1 alert if non-zero in any 24h window.
- `sheets.crdt_wasm_bundle_size_bytes` — release-gated; alarms if Loro upgrade bumps it past budget set in IP-014.

### Supply-chain + security

- Loro added to `cargo deny` allowlist with explicit version pin.
- Loro author/maintainer set monitored via GitHub Security Advisories + RustSec advisory database — same monitoring set as workflow-studio + docs.
- Major-version Loro upgrade is gated on:
  (a) 100-workbook golden corpus drill green.
  (b) AC-06 property test suite green.
  (c) WASM bundle size delta ≤ +50 KB gzip.
  (d) **Coordinated upgrade across {workflow-studio, docs, sheets}** within the same calendar week.
- Loro upstream maintainers are notified out-of-band of any CRDT-correctness issue oyatie surfaces.

### Risk register

- **Risk**: Loro pre-1.0 vintage of any subsequent breaking-change release. **Mitigation**: pin to `^1.0` only; major-version upgrades require fresh round-trip drill + property-test corpus + cross-µservice coordination.
- **Risk**: Loro maintainer-attrition. **Mitigation**: kernel port-trait wrapper makes library swap a contained refactor; three ADRs (this + ADR-WS-0001 + ADR-DOCS-0001) would be superseded jointly rather than allowed to rot.
- **Risk**: Bundle-size creep over Loro minor versions. **Mitigation**: `crdt_wasm_bundle_size_bytes` SLO + release gate.
- **Risk**: Spreadsheet cell-edit workload exposes Loro performance corner-case (e.g., large drag-fill ops). **Mitigation**: AC-06 property test corpus includes drag-fill scenarios; benchmark suite tracks Loro performance per cell-op pattern; regression triggers Loro upstream issue + temporary fallback to single-writer mode.

## References

- PRD `microservices/sheets/PRD.md` §"Tenant Value" Outcome 4, FR-06, AC-06.
- `microservices/sheets/PHASE-01-SHEETS-FOUNDATION.md` IP-005.
- `microservices/sheets/dashboards/collab-and-fanout.json`.
- ADR-WS-0001 — workflow-studio CRDT library selection (Loro 1.x).
- ADR-DOCS-0001 — docs CRDT library selection (Loro 1.x).
- Loro — `loro.dev`, `github.com/loro-dev/loro`.
- Yjs / yrs — `github.com/yjs/y-crdt`.
- Automerge 2.0 — `automerge.org`, `github.com/automerge/automerge`.
- Shapiro, M. et al. (2011), "Conflict-free Replicated Data Types," INRIA RR-7687.
- Preguiça, N. (2018), "Conflict-free Replicated Data Types: An Overview," arXiv:1806.10254.
- ADR-0065 — Leptos WASM substrate.
- ADR-0105 — 13-layer enum.
- ADR-0126 — Sheets net-new µservice.
- ADR-0131 — Per-microservice flat layout.
