---
id: ADR-SHEETS-0004
title: Recalc-engine architecture — dependency-graph + topological + parallel-task-graph
microservice: sheets
status: Accepted
date: 2026-05-17
owner: axis-sheets + council-architecture
deciders: council-architecture, axis-sheets, ops-sre-reliability
related: [ADR-0056, ADR-0105, ADR-0135, ADR-0131, ADR-SHEETS-0002, ADR-SHEETS-0003]
related_artifacts:
  - microservices/sheets/PRD.md (FR-04, AC-07, AC-08)
  - microservices/sheets/IP-004-recalc-engine-dep-graph-parallel.md
  - microservices/sheets/runbooks/recalc-storm-throttle.md
purpose: Resolve PRD Open Question 4 — choose the recalc-engine architecture that achieves 100k-cell ≤ 1s + 1M-cell ≤ 10s p95 budgets.
doc_status: published
---

# ADR-SHEETS-0004: Recalc-engine architecture — dependency-graph + topological + parallel-task-graph (rayon-backed)

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Sheets's recalc engine is the load-bearing performance surface: every cell edit triggers a recalc plan over the cell dependency-graph, propagating new values to downstream cells. Per PRD §"Performance":
- Recalc 100k-cell sheet p95 ≤ 1s.
- Recalc 1M-cell workbook p95 ≤ 10s.
- Cell-edit-render p99 ≤ 50ms (which depends on the incremental recalc cost for the dirty subset).

The recalc-engine architecture choice determines whether these budgets are achievable.

Conceptual model:
- Cells form a directed acyclic graph (DAG) where edge `(A, B)` means cell `B`'s formula references cell `A`.
- An edit to cell `A` marks `A` dirty; all cells reachable from `A` in the dep-graph also become dirty.
- A recalc plan walks the dirty subset in topological order, recomputing each cell once.
- Cells at the same topological level have no inter-dependencies; they can be evaluated in parallel.

Constraints:
- Recalc determinism: same dep-graph + same edits → same final state (load-bearing for collab CRDT correctness + audit-chain).
- Cycle detection: tenant formulas can accidentally form cycles (`A=B+1; B=A+1`); recalc-engine must refuse cycles, not infinite-loop.
- Slow-formula budget: a tenant-authored expensive formula (huge VLOOKUP, deep array operation) must not block the entire recalc plan indefinitely.
- Hot/cold tier transparency: per ADR-SHEETS-0003, dep-graph spans hot Postgres tier + cold Arrow/Parquet tier; recalc-engine must traverse seamlessly.

## Decision

Adopt **dependency-graph + topological sort + parallel-task-graph** recalc architecture:

### Dependency-graph

- Built incrementally per workbook session at open time + on every cell-edit.
- Stored in Redis hot for active sessions; reconstructable from Postgres `cells_hot.formula` field.
- Cycle detection at insert: a new edge that would create a cycle is rejected; affected cells return `#CIRCULAR` error per PRD formula-error taxonomy.

### Topological sort

- Kahn's algorithm produces levels where each level contains cells with no inter-dependencies.
- Levels are processed sequentially; cells within a level are parallel-safe.

### Parallel-task-graph executor

- Rayon thread pool sized to pod CPU count (typically 4-8 threads).
- Each level dispatches its cells to rayon's work-stealing queue.
- Cell evaluation calls into formula-engine SDK (pure; deterministic per ADR-SHEETS-0002).
- Per-cell **30s slow-formula budget**: if a single cell evaluation exceeds 30s wall-clock, it is killed and `#SLOW!` returned; the rest of the recalc plan proceeds.

### Hot/cold tier handling

- Dep-graph traversal queries cell formulas through the `HybridStore` facade per ADR-SHEETS-0003.
- Cold-tier blocks are loaded lazily; hot ranges materialised opportunistically.
- For 1M-cell workbooks, the dep-graph is partitioned by Arrow block; intra-block recalcs are vectorised; inter-block dependencies coordinate at block boundaries.

### Checkpointing for large recalc

- Recalc plans for 1M-cell workbooks emit checkpoints to Postgres every 5s.
- DR failover (per `multi-region.md`) resumes from last checkpoint on the DR-pair recalc-worker.

### Progress streaming

- WebSocket stream `RecalcFrame` events to subscribed clients during long recalcs.
- Browser-side: progress bar; tenant sees "Recalculating 750k cells... (75%)".

## Alternatives Considered

### Alternative A — Single-threaded recalc

- **Pros**
  - Trivially deterministic.
  - Simplest implementation.
- **Cons**
  - Cannot meet 1M-cell ≤ 10s p95 budget on 4-8-core hardware.
  - Wastes available CPU.
- **Rejected reason**: budget violation at 99th-percentile workbook size.

### Alternative B — Event-loop with cooperative cell evaluation

A single-threaded event-loop yielding control between cells (tokio-style).

- **Pros**
  - Avoids thread synchronisation cost.
  - Determinism trivial (single execution thread).
- **Cons**
  - No parallelism; cannot leverage multi-core hardware.
  - Same throughput as Alternative A.
- **Rejected reason**: same as A.

### Alternative C — Dataflow / actor-model recalc

Each cell is an actor; edits propagate as messages.

- **Pros**
  - Naturally parallel.
  - No global lock.
- **Cons**
  - Message-passing overhead per cell is high vs in-memory dep-graph traversal.
  - Determinism harder to guarantee under message-reordering.
  - Cycle detection at scale is hard (no global view).
- **Rejected reason**: per-cell message overhead unacceptable at 1M-cell scale; determinism concerns.

### Alternative D — Pre-computed recalc plan compiled at workbook open

Compile the dep-graph + topological plan once at workbook open; on edits, apply incremental updates to a pre-computed plan.

- **Pros**
  - Edit-path cost is minimal.
- **Cons**
  - Workbook open cost is high (must traverse + compile entire dep-graph upfront).
  - Misses cold sheet-open cold p95 ≤ 400ms target.
  - Compile-cost not amortised for shortlived editor sessions.
- **Rejected reason**: open-path cost vs benefit unfavourable. The chosen design builds dep-graph incrementally; the steady-state cost is comparable but the open-path is fast.

## Consequences

### Architectural

- `oya-sheets-recalc-engine-domain` is pure; `oya-sheets-recalc-engine-worker` hosts the rayon-backed executor.
- Recalc plans expose a deterministic `RecalcPlan { levels: Vec<Vec<CellRef>> }` shape.
- Slow-formula `#SLOW!` error becomes a new value in the FormulaError enum.

### Downstream impact

1. **IP-004** authors the recalc engine.
2. **IP-006 (large-sheet-storage)** — dep-graph traversal queries through `HybridStore` per ADR-SHEETS-0003.
3. **IP-005 (collab-crdt)** — CRDT merges apply to dep-graph; merged edits are processed by recalc-engine.
4. **dashboards/recalc-engine-health.json** — exposes recalc queue depth, dep-graph depth, slow-formula kills, parallel utilisation.
5. **`runbooks/recalc-storm-throttle.md`** — handles cluster-wide recalc surge.

### CI lanes

- `oya-governance-sheets-recalc-determinism` — new BLOCKER lane on dev.

### SLOs

- `sheets.recalc_100k_cells_seconds` — 95% under 1s.
- `sheets.recalc_1m_cells_seconds` — 95% under 10s.

### Risk register

- **Risk**: Rayon work-stealing interleaving produces non-deterministic output if formula-engine has hidden non-determinism. **Mitigation**: formula-engine is pure per ADR-SHEETS-0002; corpus pass-rate enforced.
- **Risk**: Tenant-authored cycle bypasses cycle detection. **Mitigation**: cycle detection at every edge insert; property test corpus.
- **Risk**: Slow-formula budget triggers on a legitimate (just slow) formula; tenant frustrated. **Mitigation**: 30s budget is generous for most formulas; tenant can refactor; documented in tenant-facing docs.

## References

- PRD `microservices/sheets/PRD.md` §FR-04, AC-07, AC-08.
- `microservices/sheets/IP-004-recalc-engine-dep-graph-parallel.md`.
- `microservices/sheets/runbooks/recalc-storm-throttle.md`.
- rayon — `docs.rs/rayon`.
- Kahn's topological sort algorithm.
- "Spreadsheets and Calculation" — Joel Spolsky on Excel dep-graph.
- ADR-SHEETS-0002 — formula-engine conformance.
- ADR-SHEETS-0003 — large-sheet storage substrate.
- ADR-0056, ADR-0105, ADR-0135, ADR-0131.
