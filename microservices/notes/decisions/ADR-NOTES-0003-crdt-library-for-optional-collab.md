---
id: ADR-NOTES-0003
status: Accepted
date: 2026-05-17
microservice: notes
deciders: axis-notes, council-architecture, axis-workflow-studio
owner: axis-notes
supersedes: []
superseded_by: []
related:
  - ADR-WS-0001  # workflow-studio Loro CRDT
  - ADR-0131
  - ADR-0132
  - ADR-NOTES-0001
related_artifacts:
  - microservices/notes/PRD.md (FR-17)
  - microservices/notes/catalog/oya-notes-collab-edit-kernel.yaml
  - microservices/notes/catalog/oya-notes-collab-edit-adapter-loro.yaml
purpose: Select the CRDT library + version for opt-in real-time collaboration on Professional-tier notes; align with workflow-studio/docs/sheets/slides/sites sibling decisions to a single canonical CRDT substrate.
---

# ADR-NOTES-0003: Loro 1.x (Rust) is the canonical CRDT for opt-in real-time collaboration on Professional-tier notes; E2E-tier collab refused

## Status

Accepted — 2026-05-17.

## Context

The PRD §FR-17 calls for opt-in real-time collaboration on Professional-tier notes (concurrent edit by 2-10 collaborators, ≤ 250ms convergence latency).

The CRDT-library landscape at 2026-05 includes:

- **Yjs** (TypeScript; reference implementation since 2016) — large ecosystem; well-tested at scale (Notion, Linear, Tldraw, many SaaS); minor Rust binding (`y-crdt`).
- **Automerge 2.x** (Rust + TypeScript) — academic provenance; rich JSON-native model; mature; slower than Yjs at large doc sizes.
- **Loro 1.x** (Rust + Wasm + Swift + Kotlin bindings) — modern; explicit ordering + rich-text support; up to 10× smaller op-log than Yjs in benchmarks; *adopted by workflow-studio ADR-WS-0001 (sibling) as the canonical CRDT for collaborative engine*; benchmark-leading on note-shaped workloads (medium docs with rich-text + many concurrent ops).
- **diamond-types** (Rust) — fast list CRDT; less mature; minimal binding ecosystem.

The workflow-studio µservice has already settled on Loro 1.x per ADR-WS-0001. Per the master-plan canonical-base posture (ADR-0064), every µservice that adds a CRDT substrate should align to the same library to avoid bifurcating substrate; the sibling-µservice `docs` is in flight on the same alignment, and `sheets`/`slides`/`sites` are downstream of the same decision.

Privacy: Loro requires server-side reconciliation in practice (the broker holds the op-log + dispatches to peers). This makes Loro unsuitable for E2E-tier (Personal) collab — the server would need plaintext op content. Per ADR-NOTES-0001, Personal-tier MUST NOT have server-side plaintext access. Therefore Loro collab is **Professional-tier-only** at the type-system level.

## Decision

oyatie notes adopts **Loro 1.x as the canonical CRDT** for opt-in real-time collaboration on Professional-tier notes:

1. **Loro 1.x via Rust crate `loro 1.x` (LTS pin)** at the kernel + adapter layer (`oya-notes-collab-edit-kernel` + `oya-notes-collab-edit-adapter-loro`).
2. **Loro is opt-in per Professional-tier note** at note-creation (or via "enable collab" affordance). Default-off; user-explicit-opt-in.
3. **E2E-tier (Personal) collab is structurally refused.** `oya-notes-collab-edit-usecase::start_session()` accepts only `ProfessionalNoteRef`; `PersonalNoteRef` cannot be passed. The `CollabSessionStore` port's signature enforces this. Cedar `collab-edit-scope.cedar` also forbids `Action::start_collab_session` on Personal resources.
4. **Server-side reconciliation**. `oya-notes-collab-edit-worker` hosts the per-session Loro doc state in memory; persists op-log to Postgres `loro_op` table for replay; emits per-op `LoroOpAppended` events to peers via WebSocket.
5. **Op-log compaction** at 1h idle: compact to current snapshot + truncate op-log; emit `LoroSnapshotPersisted`.
6. **Cross-region collab** out of scope at minimum-shippable-tier; future ADR if needed.
7. **Convergence latency target**: p99 ≤ 250ms intra-region; p99 ≤ 500ms cross-AZ.
8. **Loro version pin: 1.x LTS** (specific minor pinned in `Cargo.lock` + tracked by `oya gate validate version-pinning-conformance`).
9. **Sibling alignment**: docs / sheets / slides / sites / workflow-studio all converge to Loro 1.x. Future migration (if any) is coordinated cross-µservice via a substrate-level ADR.

## Alternatives Considered

### A. Yjs (TypeScript primary, y-crdt for Rust)
- Pros: largest ecosystem; battle-tested at scale; many open-source plugin libraries.
- Cons: y-crdt Rust binding less mature than Loro 1.x native Rust; ecosystem inertia (oyatie is Rust-primary); workflow-studio + docs sibling decisions already on Loro — divergence cost; benchmark performance on rich-text + medium-doc op-shapes Loro-favoured.
- Rejected for alignment + Rust-native preference.

### B. Automerge 2.x
- Pros: rich JSON-native model; long-term academic backing; good Rust + TS bindings.
- Cons: slower than Loro on rich-text benchmarks (Loro paper §6.4 shows 8-12× advantage on rich-text-heavy workloads); larger op-log; less optimised for the note-shape.
- Rejected for performance.

### C. Loro 1.x (this ADR's choice)
- Pros: sibling-aligned (workflow-studio + docs + sheets + slides + sites); Rust-native; benchmark-leading on note workloads; explicit ordering + rich-text support; smaller op-log; active development.
- Accepted.

### D. diamond-types
- Pros: fast list-CRDT performance.
- Cons: less mature; minimal binding ecosystem (Swift / Kotlin / TS missing or thin); not suitable for non-list CRDT shapes; doesn't align with sibling decisions.
- Rejected.

### E. No CRDT (server-arbitrated last-writer-wins + optimistic UI)
- Pros: simpler; no library dependency.
- Cons: kills the FR-17 use case; produces data-loss when two clients edit concurrently; competitors (Notion, Google Docs, Apple Notes shared) all use CRDT or OT.
- Rejected.

### F. Operational Transformation (OT) custom implementation
- Pros: simpler protocol per op-type.
- Cons: implementation complexity ~ Yjs/Loro for general docs; oyatie would maintain its own protocol; sibling-bifurcation; rejected by every modern CRDT comparison (OT requires central authority and is harder to extend).
- Rejected.

## Consequences

### Positive

- Sibling alignment: workflow-studio + docs + sheets + slides + sites + notes all on Loro 1.x → shared expertise + shared substrate ops + shared LTS pin.
- Performance budget achievable: Loro paper §6.4 + benchmarks meet p99 ≤ 250ms on intra-region ops.
- Privacy preserved: E2E-tier (Personal) collab is type-system-refused — no risk of Loro-broker plaintext leak on Personal notes.
- Op-log compaction limits storage growth.

### Negative

- Loro 1.x is newer than Yjs / Automerge — bus-factor risk. Mitigated by Rust-native (oyatie can fork if needed); active upstream development.
- Cross-region collab scheduled-for-distinct-tracked-work. Customers wanting it must accept intra-region grouping at minimum-shippable-tier.
- Loro-broker is stateful (in-memory doc state) — careful HPA + session-affinity required. Documented in `iac/helm/notes/templates/`.

### Operational

- Crate `oya-notes-collab-edit-{kernel,domain,usecase,api,adapter,adapter-loro,worker,sdk,app}` enumerated.
- Postgres `loro_op` table with TimescaleDB-style time-partitioning by `(tenant_id, note_id, year-month)`.
- Broker HPA: min 3, max 30; session-affinity via `(tenant_id, note_id)` hash.
- SLO `notes-collab-convergence-latency.openslo.yaml` (under `slos/`).
- Reference-implementation conformance test (`tests/e2e/loro-collab-convergence.rs` per PRD AC-15).

### Cross-µservice

- The shared Loro substrate is a strong invariant; any divergence requires a coordinated ADR.
- Documentation in `microservices/notes/sdk-plan.md` notes which SDK languages bind Loro.

## References

- Loro CRDT paper + benchmarks (`loro.dev`).
- Yjs / Automerge / diamond-types comparison: `crdt-benchmarks` (open source).
- ADR-WS-0001 (sibling workflow-studio Loro adoption).
- ADR-0064 (canonical base + localization).
- ADR-NOTES-0001 (E2E posture; refusal of Personal-tier collab).
- `microservices/notes/PRD.md` FR-17.
- `microservices/notes/capacity-model.md`.
