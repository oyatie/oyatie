---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-012-bulk-edit-pipeline
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-test, bulk-edit-atomicity, dependency-graph-cycle-prevention]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: bulk-edit pipeline — atomic 1..10k task patch with cycle-aware refusal

## Intent

Implement the bulk-edit pipeline at the `task-store-usecase` +
`task-store-worker` boundary. PRD §FR-07 + AC-05 require atomic
all-or-nothing semantics: a 100-task bulk edit completes ≤ 300ms p95;
either every task in the batch is updated or none is. Larger batches
(up to 10k tasks per the OpenAPI surface) run as a job with a
`BulkEditJob` state machine (pending → applying → completed |
partial_failure | refused).

Cycle-aware refusal: a bulk patch that would induce a dependency cycle
(e.g., reassigning a task into a different project that creates a
cross-project edge cycle) is refused with row-level errors per
ADR-TASKS-0002. Idempotency: `idempotency_key` (uuid) deduplicates
retries via per-tenant Redis SET with 24h TTL.

≥ 10k-task operations are refused at the API edge (validation per
OpenAPI `maxItems: 10000`) and require explicit second-confirmation
per PRD §Security.

## ChangeSet boundary

`task-store-usecase` + `task-store-worker` + `task-store-rest`. Bulk
job state persisted in Postgres + Redis idempotency cache.

## Crate Naming

n/a — modifies existing task-store crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-task-store-usecase/src/bulk.rs` | created | atomic-batch orchestrator |
| `microservices/tasks/src/oya-tasks-task-store-worker/src/bulk_job.rs` | created | state machine |
| `microservices/tasks/src/oya-tasks-task-store-rest/src/routes/bulk.rs` | created | HTTP handler |
| `microservices/tasks/tests/integration/bulk_atomicity.rs` | created | AC-05 verification |

## Acceptance Gates

```bash
cargo test -p oya-tasks-task-store-usecase bulk
cargo bench -p oya-tasks-task-store-usecase bulk
cargo run -p oya-dev-cli -- gate validate bulk-edit-atomicity --microservice tasks
```

## Test Plan

- 100-task batch p95 ≤ 300ms (AC-05).
- Single-task failure inside the batch → entire batch rolled back; no
  partial mutation visible to readers (snapshot isolation).
- Cycle-inducing dependency patch refused at row level; non-cycle rows
  in the same batch still refused (all-or-nothing for cross-edge
  bulks; configurable for non-cycle patches).
- Idempotency: same `idempotency_key` replays the original job result
  within 24h.

## Halt Conditions

- Partial mutation visible — refuse; fix isolation.
- Cycle-inducing bulk slips through — P0 refusal.

## Next IP

[`IP-013-rest-and-websocket-api-surface.md`](IP-013-rest-and-websocket-api-surface.md)

## References

- ADR-TASKS-0002 (dependency cycle); ADR-TASKS-0005 (event-emit
  coupling); PRD AC-05.
