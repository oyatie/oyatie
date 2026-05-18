---
id: ADR-TASKS-0002
status: Accepted
date: 2026-05-17
microservice: tasks
deciders: axis-tasks, council-architecture, ops-data-platform
owner: axis-tasks + council-architecture
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-TASKS-0001
related_artifacts:
  - microservices/tasks/PRD.md (AC-02 correctness 100%; FR-04 dependency graph)
  - microservices/tasks/IP-007-dependency-graph-and-cycle-prevention.md
  - microservices/tasks/contracts/openapi/tasks.yaml (DependencyCycleRefusal)
  - microservices/tasks/contracts/proto/tasks.proto (DependencyGraph service)
purpose: |
  Close the load-bearing AC-02 invariant: dependency-graph cycle prevention
  is enforced at write time, with no error budget. The decision drives the
  IP-007 implementation + the trigger function in IP-004.
---

# ADR-TASKS-0002: Dependency graph + cycle prevention at write time — DAG enforcement at adapter + domain layer; circular-dependency policy refuses cycle-creating writes

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD-tasks §FR-04 mandates `blocks` / `blocked_by` / `relates_to`
dependency edges between tasks. PRD AC-02 sets the correctness budget
at **100% — no error budget**: a cycle-creating write must ALWAYS be
refused. Cycle detection is a textbook problem (bounded BFS / DFS
yields O(V+E) detection on the affected subgraph) but where it runs
matters.

Three placement strategies compete:

1. **Lazy detection** (Jira-style). Allow any edge; detect cycles
   asynchronously during render or scheduled scan. Pros: write-path
   fast. Cons: a cycle existing in the database for any duration
   violates AC-02 by definition; downstream consumers (critical-path
   solver, Gantt rendering, workflow-engine triggers) observe garbage.
2. **Write-path domain check** (BFS at usecase). Insert refused if
   adding the edge would close a cycle. Pros: invariant always holds.
   Cons: race condition between two concurrent edge-adds — A→B and
   B→A inserted on different connections both pass the read-side BFS
   and commit, producing a cycle.
3. **Write-path database trigger** (Postgres `BEFORE INSERT` trigger
   function with serialisable subgraph BFS). Defence-in-depth: domain
   AND database both refuse. Pros: race-free even under MVCC because
   the trigger function takes a row-level lock on the start + end
   nodes' edge-set before deciding. Cons: trigger function is harder
   to optimise; deep dependency graphs cost more BFS at insert time.

Per PRD §"Performance" the dependency-cycle-detection-on-add budget is
p99 ≤ 50ms; per PRD §"Horizontal Scalability" the bounded-BFS depth is
capped at 10 hops (beyond which refuse on the side of safety with the
partial cycle).

Edge kinds: `blocks`, `blocked_by`, `relates_to`. Only `blocks` +
`blocked_by` form the directed DAG (they're inverses). `relates_to` is
informational (undirected; doesn't enter cycle math).

## Decision

The tasks µservice ships **defence-in-depth cycle prevention**:

1. **Domain layer** (`oya-tasks-dependency-graph-domain`) runs a
   bounded BFS at edge-add usecase time. Depth cap = 10. If the new
   edge closes a cycle, the usecase returns
   `DependencyCycle::Refused(cycle_path: Vec<TaskId>)` 409.
2. **Database trigger** (`oya-tasks-task-store-adapter-postgres`
   migration) runs a row-locking serialisable BFS in
   `pg_trigger_depth() = 0`. If the trigger detects a cycle, it
   raises an exception that the usecase translates back to the same
   `DependencyCycle::Refused`. The trigger function takes a row-
   level lock on `(from_task_id, to_task_id)`-keyed advisory locks
   in fixed order to guarantee deterministic ordering across concurrent
   inserts (no deadlock).
3. **Bulk-edit pipeline** (IP-012) pre-validates the entire batch
   against the post-batch graph before applying any row; failing
   pre-validation refuses the entire batch (all-or-nothing per
   ADR-TASKS-0005 + AC-05).
4. **`relates_to` edges** do NOT enter cycle math; they're plain
   informational links.

## Alternatives Considered

### Alternative 1 — Lazy detection (Jira-style)

- Pros:
  - Write path is trivial (no BFS cost).
- Cons:
  - AC-02 = 100% correctness is incompatible with any window where a
    cycle exists in the database.
  - Downstream consumers (critical-path solver, Gantt renderer,
    workflow-engine triggers) observe corrupt state.
- Rejected because: incompatible with the AC-02 hard invariant.

### Alternative 2 — Domain-only check (no trigger)

- Pros:
  - Single point of refusal; simpler.
- Cons:
  - Race between two concurrent edge-adds on different connections —
    A→B and B→A both pass the read-side BFS at the same instant; both
    commit; a cycle is born.
  - Cannot prevent direct SQL bypass (admin tooling, ops scripts,
    importer edge cases). Defence-in-depth required.
- Rejected because: race condition violates the 100% correctness
  invariant.

### Alternative 3 — Trigger-only check (no domain pre-check)

- Pros:
  - Authoritative refusal at the storage layer.
- Cons:
  - Round-trip latency cost: a usecase needs to catch a Postgres
    exception to translate to a structured `DependencyCycle::Refused`;
    the diagnostic path is brittle.
  - Domain layer can't run cycle-aware planning (e.g., critical-path
    pre-compute) without re-implementing BFS — duplication is OK in
    defence-in-depth.
- Rejected because: while close, the diagnostic-quality gap is
  significant — the domain layer's structured refusal includes the
  cycle path; the trigger's exception does not.

## Consequences

### Consequence 1 — IP-004 migration carries the cycle-prevention trigger

The trigger function lands in the same SQL migration as the
`task_dependencies` table itself. The migration is non-revertable
without a same-shape rewrite — if a future ADR relaxes cycle
prevention, the trigger is removed in a new ADR + migration, not
backed out.

### Consequence 2 — Cycle refusal storm becomes an observability signal

A spike in `oya_tasks_dependency_cycle_refused_total` indicates
either a client retry storm (e.g., a bug in a 3rd-party automation
loop) or an attack. The IaC PrometheusRule
(`TasksDependencyCycleRefusalRateAnomaly`) flags this above 5 per
5min.

### Consequence 3 — Bulk-edit cycle pre-validation cost added to bulk path

Per IP-012, bulk-edits pre-validate the entire batch against the
post-batch graph. This adds O((B+E) × log V) cost per bulk; the
p95 ≤ 300ms budget (AC-05) accommodates this for the 100-task
target. Larger batches up to 10k tasks run as a job (BulkEditJob
state machine).

## References

- ADR-TASKS-0001 (typed schema); ADR-0105 (13-layer); ADR-0131 (flat).
- PRD-tasks AC-02 (100% correctness); §FR-04; §"Performance"
  dependency-cycle-detection-on-add.
- Tarjan, "Depth-first search and linear graph algorithms" (1972) —
  cycle detection foundation.
- Postgres advisory locks — `www.postgresql.org/docs/16/explicit-locking.html`.
- Jira issue-link cycles — `community.atlassian.com` (lazy-detection critique).
