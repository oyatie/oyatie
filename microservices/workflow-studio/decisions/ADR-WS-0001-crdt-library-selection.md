---
id: ADR-WS-0001
title: CRDT library selection for workflow-studio collaborative editing
microservice: workflow-studio
status: Accepted
date: 2026-05-17
owner: axis-workflow + council-architecture
deciders: council-architecture, axis-workflow, council-design-system, ops-security
supersedes: []
superseded_by: []
related: [ADR-0065, ADR-0105, ADR-0131]
related_specs: [/specs/microservices/workflow-studio.json]
related_artifacts:
  - microservices/workflow-studio/PRD.md (Open Question 1, AC-06, AC-02)
  - microservices/workflow-studio/PHASE-01-VISUAL-AUTHORING-SUBSTRATE.md (IP-005)
  - microservices/workflow-studio/dashboards/collab-health.json
  - microservices/workflow-studio/IP-005-collab-crdt-kernel-domain-adapter.md
purpose: Resolve PRD Open Question 1 — choose the CRDT library backing the workflow-studio collab-crdt bounded context, with the round-trip byte-equality (AC-02) and never-silent-loss (AC-06) invariants as load-bearing constraints.
doc_status: published
---

# ADR-WS-0001: CRDT library selection — Loro

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The workflow-studio µservice's `collab-crdt` bounded context (PRD §"Bounded Contexts") requires a conflict-free replicated data type (CRDT) substrate for real-time multi-user editing of the workflow definition canvas. Per PRD §"Tenant Value" Outcome 4 and AC-06, the load-bearing invariant is **"never silent loss"**: two business users editing the same workflow definition concurrently MUST have their non-conflicting edits merged, and conflicting edits MUST surface an explicit conflict UI — last-writer-wins is forbidden.

PRD §"Open Questions" Q1 (gates IP-005): "CRDT library: yrs (yjs Rust port) vs loro vs bespoke?" with a stated bias toward Loro for tree-aware CRDT (matches DAG structure).

Workflow definitions are tree-shaped (nodes, edges, parameters, nested groups). The DSL canonical form (workflow_spec.v1.json, settled by ADR-WS-0002) is the authoring source of truth; the visual canvas derives from it. AC-02 demands that **load(emit(canvas)) is byte-equal to the original spec** over a 100-spec reference corpus. The CRDT layer sits between the visual canvas and the dsl-emitter/dsl-loader, so the CRDT's serialized state MUST be projectable to the canonical JSON without lossy conversion or key-ordering nondeterminism.

Performance envelope (PRD §"Performance"):
- Collab CRDT merge p50 ≤ 30ms, p99 ≤ 100ms, p999 ≤ 250ms.
- Up to 10 concurrent collaborators per definition (graceful degradation beyond).
- 100,000 active editor sessions per region (XL tier).
- WASM bundle must stay within Editor TTI ≤ 2s p99 cold-load CDN-cached budget.

Substrate constraints:
- Rust kernel (ADR-0105 layer enum); CRDT merge engine lives in `oya-workflow-studio-collab-crdt-domain` (pure, deterministic).
- Browser-WASM target (per ADR-0065 Leptos WASM frontend, settled by ADR-WS-0003); CRDT library MUST compile to `wasm32-unknown-unknown`.
- Ephemeral CRDT state in Valkey (PRD §"Horizontal Scalability" mixed-state strategy); reconstructable from Postgres on cold-start.
- WebSocket gateway long-lived worker (`-worker` crate) fans out CRDT ops via consistent-hash on `definition_id` to keep collaborators on the same pod.

Operational constraints:
- The CRDT library is supply-chain critical: a vulnerability in merge logic can corrupt every tenant's authored workflows.
- License must be permissive (MIT / Apache-2.0); copyleft (GPL/AGPL) is incompatible with the closed-source SDK distribution model per `sdk-plan.md`.

## Decision

Adopt **Loro 1.x** (`crates.io/crates/loro`) as the workflow-studio CRDT library, with the following constraints:

1. Loro types are wrapped in workflow-studio's own `oya-workflow-studio-collab-crdt-kernel` port traits (`CrdtMergeEngine`, `CrdtState`, `MergeOp`); the library is an implementation detail of the `-domain` crate. Public APIs across BCs MUST NOT leak Loro types — this preserves the option to swap the library without breaking the canvas, dsl-emitter, or SDK contracts.
2. The Loro tree CRDT (`LoroTree`) backs the node/edge graph; Loro maps (`LoroMap`) back node parameter dictionaries; Loro lists (`LoroList`) back edge ordering. Conflict surfacing uses Loro's built-in version-vector + frontier API; the `Conflict` entity in the kernel wraps Loro `Frontiers` into a UI-renderable shape.
3. CRDT-to-spec projection (`emit`) deterministically orders Loro nodes by their stable `TreeID`s, with map keys lex-sorted at the dsl-emitter boundary. This makes Loro state projection deterministic and is the seam that lets AC-02 (round-trip byte-equality) hold even though Loro's internal op log is not byte-canonical.
4. Loro snapshot encoding is used for Valkey persistence (`snapshot()` + `import_snapshot()`); JSON projection is used only for the canonical spec emission.
5. Loro version pinning + Ed25519-signed advisory feed monitoring; major-version upgrades require a fresh round-trip-corpus drill against the 100-spec reference corpus before merge.
6. The collab-crdt CI lane (`oya-governance-collab-no-silent-loss`) runs Loro's example test set + workflow-studio's own AC-06 property test (10 concurrent editors, randomized op interleaving, assertion that every accepted op is reachable from final state OR surfaced as conflict — never silently dropped).

## Alternatives Considered

### Alternative A — Yjs (via yrs, the Rust port)

Yjs is the most-deployed CRDT library in production (Notion, Figma, Linear all use Yjs-class systems); yrs is the Y Crate Authors' Rust port (`crates.io/crates/yrs`).

- **Pros**
  - Largest production install base; battle-tested merge semantics over ~7 years.
  - Mature ecosystem: yjs-codemirror, yjs-prosemirror, y-websocket reference server.
  - Rich client tooling in TypeScript (relevant if oyatie ever ships a JS SDK).
  - yrs explicitly targets `wasm32-unknown-unknown`.
- **Cons**
  - Yjs's tree representation is built on top of Yjs Maps; tree-rebalance semantics are emulated, not native. Workflow DAGs hit this corner case heavily (subgraph drag-reparent is a primary canvas gesture).
  - yrs lags Yjs feature parity by ~2 minor versions historically; new Yjs features arrive in yrs months later.
  - Yjs's awareness protocol (cursor/presence) and ops protocol are coupled; separating them for oyatie's WebSocket gateway adds friction.
  - Snapshot format is documented but is a binary encoding; debugging requires Yjs tooling, not stdlib JSON.
- **Rejected reason**: tree-rebalance is a primary workflow-canvas gesture and Yjs's emulated tree semantics introduce a class of subtle reparenting bugs that the AC-06 "never silent loss" invariant will catch only after they've shipped. Loro's native tree CRDT eliminates that class of failure structurally. Production-install-base advantage does not outweigh structural-fitness disadvantage when AC-06 is Sev-1.

### Alternative B — Automerge 2.0 (`crates.io/crates/automerge`)

Automerge 2.0 is the rewrite-in-Rust of the historic JavaScript-first Automerge; widely cited in CRDT literature (Kleppmann et al.).

- **Pros**
  - Academic provenance; Kleppmann's papers underpin the design.
  - Strong JSON-document fit: Automerge documents are JSON-shaped natively.
  - Push for byte-canonical encoded changes (`save()` is deterministic for a given history).
  - Apache-2.0 licensed.
- **Cons**
  - No native tree CRDT; trees are emulated via maps + list-of-children, same limitation as Yjs.
  - WASM bundle size larger than Loro by ~2-3x at the time of evaluation (Loro ~250 KB gzip vs Automerge ~600 KB gzip per upstream benchmarks 2025-Q4).
  - History-preservation-first design retains every op in the document forever unless explicitly compacted; this multiplies storage for long-lived workflow drafts.
  - Performance for high-frequency op interleaving (drag-while-collaborator-drags) is documented as O(n log n) in op count; Loro's RGA-tree variant is closer to O(log n).
- **Rejected reason**: bundle size hits the TTI ≤ 2s p99 budget (PRD §"Performance") and there is no native tree CRDT for the workflow DAG. History-preservation default conflicts with the ephemeral-Valkey state model.

### Alternative C — Bespoke Rust CRDT

A workflow-studio-authored CRDT implementing exactly the operations the canvas needs (node insert/delete/move, edge connect/disconnect, parameter map update).

- **Pros**
  - Zero external dependency; full control over invariants.
  - Tailored to workflow_spec.v1.json projection (could make AC-02 byte-equality structural rather than a projection-layer concern).
  - No third-party version-pinning + upgrade tax.
- **Cons**
  - CRDT correctness proofs are research-grade work (Shapiro et al., 2011; Preguiça, 2018); bespoke implementations have historically taken multiple years to reach production stability (cf. Figma's published rewrite timeline).
  - No external review surface; AC-06 violations only caught by oyatie's own property tests.
  - Owns the WASM build target end-to-end; loses Loro's WASM-first engineering.
  - Recruiting and onboarding cost: every contributor must learn the bespoke algebra.
- **Rejected reason**: build-vs-buy ratio is unfavourable. CRDT correctness is well-trodden academic territory; oyatie does not differentiate by reinventing it. Differentiation lives in the canvas UX, the round-trip-byte-equality contract, and the jurisdiction-overlay renderer — none of which are CRDT-implementation concerns.

### Alternative D — Hand-rolled Operational Transform (OT)

The pre-CRDT industry standard (Google Docs, Etherpad).

- **Pros**
  - Mature literature; widely understood.
  - Server-mediated; simpler invariants when one server is authoritative.
- **Cons**
  - Requires a single authoritative server per document for correctness; this is fundamentally incompatible with the multi-cell, multi-region scale-out story in PRD §"Horizontal Scalability".
  - OT transform functions are notoriously fragile to add new operation types (every new op needs O(n²) transform functions); the canvas op set is open-ended.
  - "Never silent loss" requires explicit conflict resolution on every transform; in practice OT systems fall back to last-writer-wins under load.
- **Rejected reason**: incompatible with horizontal scale-out + the AC-06 invariant. Modern collaborative editors have collectively moved off OT for these reasons.

## Consequences

### Architectural

- The collab-crdt `-domain` crate depends on Loro 1.x (pinned with `^1.0` and a vendor advisory subscription).
- `oya-workflow-studio-collab-crdt-kernel` declares `CrdtMergeEngine`, `CrdtState`, `MergeOp`, `Conflict` port traits with no Loro types in the signature; the Loro adapter lives behind these ports.
- The dsl-emitter consumes the CRDT projection through a single `project_to_canonical(state) -> CanonicalSpec` boundary; this is the seam that makes AC-02 byte-equality (per ADR-WS-0002) hold despite Loro's non-canonical internal encoding.
- WASM bundle gains ~250 KB gzip from Loro; the TTI ≤ 2s p99 budget is preserved on initial measurement but must be re-verified as the canvas adapter grows (gates IP-012).

### Downstream impact on other µservices and IPs

1. **IP-005 (collab-crdt kernel/domain/adapter)** — adopts Loro as the concrete merge engine; property test set for AC-06 must run Loro op streams plus a randomized-interleaving fuzzer.
2. **IP-006 (collab-crdt worker + SDK)** — WebSocket gateway encodes/decodes Loro ops; tenant SDKs (Rust + future TypeScript) MUST NOT expose Loro types directly — they expose `CrdtOp` envelopes only.
3. **IP-003/IP-004 (dsl-emitter/dsl-loader)** — projection layer between Loro state and canonical JSON is co-authored with IP-005; deterministic node-ordering by stable `TreeID` is the contract.
4. **workflow-engine µservice** — unaffected at the engine side; engine consumes only the canonical spec, never CRDT state.
5. **tenancy µservice** — collab event metadata (`CollabMerged`, `CollabConflictSurfaced`) is unaffected by library choice.
6. **observability µservice** — `collab-health.json` dashboard's `collab_merge_p99_ms`, `collab_conflict_rate`, `collab_silent_loss_count` SLIs gain a new dimension `crdt_library=loro` for upgrade-rollout observation.

### SLOs gaining new dimensions

- `workflow-studio.collab_merge_latency_p99` — tagged with `crdt_library=loro`.
- `workflow-studio.collab_no_silent_loss_count` — Sev-1 alert if non-zero in any 24h window.
- `workflow-studio.crdt_wasm_bundle_size_bytes` — release-gated; alarms if Loro upgrade bumps it past the budget set in IP-013.

### Supply-chain + security

- Loro added to `cargo deny` allowlist with explicit version pin.
- Loro author/maintainer set monitored via GitHub Security Advisories + RustSec advisory database.
- Major-version Loro upgrade is gated on: (a) 100-spec round-trip-corpus drill green, (b) AC-06 property test set green, (c) WASM bundle size delta ≤ +50 KB gzip.
- Loro upstream maintainers are notified out-of-band of any CRDT-correctness issue oyatie surfaces; the issue + fix is contributed back per ADR-0133 axis-4 industry-best-practice conformance.

### Risk register

- **Risk**: Loro pre-1.0 vintage of any subsequent breaking-change release. **Mitigation**: pin to `^1.0` only; major-version upgrades require fresh round-trip drill + property-test corpus.
- **Risk**: Loro maintainer-attrition. **Mitigation**: kernel port-trait wrapper makes library swap a contained refactor; ADR-WS-0001 will be superseded rather than allowed to rot.
- **Risk**: Bundle-size creep over Loro minor versions. **Mitigation**: `crdt_wasm_bundle_size_bytes` SLO + release gate.

## References

- PRD `microservices/workflow-studio/PRD.md` §"Open Questions" Q1, §"Tenant Value" Outcome 4, AC-06, AC-02.
- `microservices/workflow-studio/PHASE-01-VISUAL-AUTHORING-SUBSTRATE.md` IP-005, IP-006.
- `microservices/workflow-studio/dashboards/collab-health.json`.
- ADR-WS-0002 — DSL canonical form (round-trip byte-equality).
- ADR-WS-0003 — Leptos WASM substrate (browser target).
- Loro — `loro.dev`, `github.com/loro-dev/loro`.
- Yjs / yrs — `github.com/yjs/y-crdt`.
- Automerge 2.0 — `automerge.org`, `github.com/automerge/automerge`.
- Shapiro, M. et al. (2011), "Conflict-free Replicated Data Types," INRIA RR-7687.
- Preguiça, N. (2018), "Conflict-free Replicated Data Types: An Overview," arXiv:1806.10254.
