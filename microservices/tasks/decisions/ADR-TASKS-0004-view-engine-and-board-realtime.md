---
id: ADR-TASKS-0004
status: Accepted
date: 2026-05-17
microservice: tasks
deciders: axis-tasks, council-architecture, axis-workflow-studio
owner: axis-tasks + council-architecture
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-NOTES-0001
related_artifacts:
  - microservices/tasks/PRD.md (FR-03 multi-view; §"Performance" board DnD; FR-08 search)
  - microservices/tasks/IP-010-view-engine-and-board-realtime.md
  - microservices/tasks/contracts/openapi/tasks.yaml (BoardView + reorder)
purpose: |
  Pick the realtime substrate for board drag-and-drop + collaborative
  description editing, aligned with workflow-studio's CRDT choice so
  both products share a single CRDT engine.
---

# ADR-TASKS-0004: View engine + board realtime — Loro CRDT 1.x for description; deterministic LexoRank for board moves; per-tenant Redis 7.2 LTS view-state

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD-tasks §FR-03 mandates six view kinds (list / board / gantt /
calendar / timeline / table) over the same project store. PRD §
"Performance" requires:

- Board render with DnD p95 ≤ 50ms perceived (client DnD; server
  commit async).
- Task-list render (200 tasks) p95 ≤ 200ms.

Two distinct realtime problems live inside the view-engine BC:

1. **Board reorder concurrency**. Two users dragging the same task to
   different columns in the same instant. Naïve approaches (last-
   write-wins on integer rank) cause flicker, divergent client state,
   and lost moves.
2. **Collaborative description editing**. Two users editing the same
   task's description text simultaneously (e.g., adding bullets, fixing
   typos). This is a textbook CRDT use case.

Candidates per problem:

**Board reorder** —

1. **Naïve integer ranks (rank = 1, 2, 3, ...).** Pros: simple. Cons:
   inserting between rank=N and rank=N+1 needs to renumber half the
   column; concurrent inserts collide; reorders cost O(N) re-writes.
2. **Fractional indexing** (a la Figma; key = midpoint of neighbours).
   Pros: O(1) insert. Cons: keys grow unbounded under adversarial
   reordering.
3. **LexoRank-style lexicographic rank keys**. Pros: O(1) insert
   amortised; bounded key length under realistic workloads;
   deterministic merge under concurrent insert (when both clients
   pick the same midpoint string, the tie is broken by `task_id` lex
   order). Cons: needs periodic compaction.
4. **CRDT for ordering** (RGA / Yjs Array). Pros: provably converges.
   Cons: overkill for the board ordering use case; merge cost > LexoRank.

**Collaborative description editing** —

1. **Operational Transformation (OT)** (Google Docs original).
   Pros: extensively studied. Cons: notoriously hard to implement
   correctly; centralised transformation server required.
2. **Yjs CRDT** (de facto OSS CRDT). Pros: mature; fast; widely
   adopted. Cons: JS-first; Rust bindings exist but less polished;
   licensing acceptable.
3. **Loro CRDT 1.x** (Rust-native; aligned with workflow-studio
   ADR-WS-0001). Pros: Rust-native; binary stable as of 1.x; same
   library as workflow-studio's editor → one CRDT engine across two
   products. Cons: less mature than Yjs in the broader ecosystem.
4. **Automerge 2.x**. Pros: production-grade Rust CRDT. Cons: heavier
   wire format; binary not stable across versions; workflow-studio
   has not adopted.

## Decision

The tasks µservice ships:

- **Deterministic LexoRank-style rank keys** for board reorder.
  Insert is O(1) amortised; concurrent inserts deterministically merge
  (lex order of `(rank_key, task_id)`). Periodic compaction at 100k
  reorder ops per board. Per IP-010 test plan, identical reorder ops
  in different arrival order converge to identical final order.
- **Loro CRDT 1.x for collaborative task-description editing**.
  Aligned 1:1 with workflow-studio ADR-WS-0001. One CRDT engine across
  the two products avoids duplicate ops-burden and capability gaps.
  CRDT scope is strictly limited to the `description` field — title,
  status, priority, assignees, etc. are non-CRDT (last-write-wins on
  optimistic version).
- **Per-tenant Redis 7.2 LTS** for `ViewStateStore` (presence cursor,
  view filter state, ephemeral DnD lock). Cluster-mode-safe; per-
  tenant key prefix; eviction `allkeys-lru`.
- **WebSocket gateway** on `view-engine-rest` port 8443. Per-connection
  backpressure ring buffer (1k events); slow consumers drop with
  `Channel::Backpressure` and reconnect with `last_event_id` replay.

## Alternatives Considered

### Alternative 1 — CRDT for board reorder

- Pros:
  - Provably converges under arbitrary concurrent ops.
- Cons:
  - Memory + wire cost of an RGA / Yjs Array per board column.
  - Diagnostic difficulty: when ordering looks wrong to a human, the
    CRDT graph is harder to inspect than a deterministic rank-key.
  - LexoRank is already provably-convergent under the
    (rank_key, task_id) tiebreaker.
- Rejected because: LexoRank is sufficient + cheaper.

### Alternative 2 — Yjs over Loro

- Pros:
  - Larger ecosystem; more battle-tested.
- Cons:
  - workflow-studio already picked Loro (ADR-WS-0001). Forking the
    CRDT choice across products doubles ops-burden + capability gaps.
- Rejected because: workflow-studio's pick is load-bearing; second-mover
  alignment costs less than independent decision.

### Alternative 3 — Last-write-wins on integer rank (naïve)

- Pros:
  - Simplest possible storage.
- Cons:
  - Concurrent moves flicker; reorders cost O(N) writes when inserting
    between two adjacent ranks.
  - Fails the AC-implicit "no lost moves" requirement.
- Rejected because: PRD §"Performance" board DnD budget is incompatible
  with O(N) re-writes; concurrent-move flicker is a UX regression
  competitors don't have.

## Consequences

### Consequence 1 — Loro 1.x pinned at workflow-studio's version

The `Cargo.lock` for tasks pins Loro at the exact patch version that
workflow-studio runs. Upgrades are coordinated via the
`docs/standards/version-pinning.md` lane. Drift triggers the
`oya-governance-version-pinning-conformance` gate failure.

### Consequence 2 — Redis 7.2 is a load-bearing dependency

Loss of Redis flips view-engine into a degraded mode: live presence +
view-filter caching disabled; clients fall back to direct REST polling
at a degraded p95. Per `failure-modes.md`, Redis loss is recoverable
≤ 1 min via the substrate Redis cluster failover.

### Consequence 3 — LexoRank periodic compaction is a worker

`oya-tasks-view-engine-worker` runs a periodic compaction job that
rewrites rank keys to canonical form when any column exceeds 100k
reorder ops. This is non-blocking; new reorders queue on the worker
during the rewrite window (typically < 60s per column).

## References

- ADR-WS-0001 workflow-studio CRDT (Loro).
- ADR-TASKS-0001 (data model); ADR-TASKS-0002 (dependency graph).
- LexoRank — `medium.com/whisperarts/lexorank-a-ranking-algorithm`.
- Loro CRDT 1.x — `loro.dev`.
- Yjs — `docs.yjs.dev`.
- Shapiro et al., "A comprehensive study of Convergent and Commutative
  Replicated Data Types" (INRIA RR-7506).
- Figma fractional-indexing — `figma.com/blog/realtime-editing-of-ordered-sequences/`.
- PRD-tasks §FR-03; §"Performance" board DnD.
