---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-007-dependency-graph-and-cycle-prevention
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-test, dependency-graph-cycle-prevention]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: dependency-graph BC + cycle prevention at write time

## Intent

Ship the `dependency-graph` BC end-to-end. Entities: `DependencyEdge`
(kinds: blocks / blocked_by / relates_to), `CycleDecision`,
`CriticalPath`. Cycle prevention enforced at the domain layer
(bounded BFS) AND at the database constraint level (Postgres trigger
function refuses cycle-inducing INSERTs). Per ADR-TASKS-0002, cycle-
creating writes return `DependencyCycle::Refused` 409 with the cycle
path included.

PRD AC-02 sets correctness budget at 100% — no error budget; cycle-
inducing write must always refuse. Critical-path computation (longest-
path through DAG with task durations) ships behind the same BC for
Gantt rendering (PRD §FR-03 gantt view).

## ChangeSet boundary

7 dependency-graph crates (kernel/domain/usecase/api/adapter/rest/app)
+ Postgres trigger function migration + property tests.

## Crate Naming

`oya-tasks-dependency-graph-*` per ADR-0056 v4.1; adapter rolls into
`task-store-adapter-postgres` so the same trigger function covers both
edge inserts + cycle detection (per PRD §"Bounded Contexts" note).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-dependency-graph-{kernel,domain,usecase,api,adapter,rest,app}/src/lib.rs` | created/replaced | 7-crate stack |
| `microservices/tasks/src/oya-tasks-task-store-adapter-postgres/migrations/20260518_cycle_prevention.sql` | created | trigger function |
| `microservices/tasks/src/oya-tasks-dependency-graph-domain/tests/cycle_refusal.rs` | created | property test suite |
| `microservices/tasks/catalog/oya-tasks-dependency-graph-*.yaml` | created | catalog entries |

## Acceptance Gates

```bash
cargo test -p oya-tasks-dependency-graph-domain
cargo test -p oya-tasks-dependency-graph-usecase
cargo run -p oya-dev-cli -- gate validate dependency-graph-cycle-prevention --microservice tasks
```

## Test Plan

- A→B→A direct cycle refused (1-hop).
- A→B→C→A indirect cycle refused (3-hop).
- A→B→C→D→A 4-hop cycle refused; BFS bounded at PRD-stated depth of
  10 — beyond 10 hops, refuse on the side of safety with the partial
  cycle.
- relates_to edges are NOT considered cyclic (per ADR-TASKS-0002 only
  `blocks` + `blocked_by` count toward dependency DAG).
- Critical-path computation matches reference oracle for random DAGs
  up to 10k nodes.

## Halt Conditions

- Any cycle-creating write reaches commit — refuse to ship; this is
  load-bearing.

## Next IP

[`IP-008-recurring-task-engine.md`](IP-008-recurring-task-engine.md)

## References

- ADR-TASKS-0002 (DAG enforcement at write time).
- PRD AC-02 (correctness 100%).
