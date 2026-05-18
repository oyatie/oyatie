---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-009-state-workflow-engine-cross-link
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-test, task-state-machine-correctness, oya-governance-cross-product-isolation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: status-workflow engine + workflow-engine cross-link

## Intent

Wire the per-project `StatusWorkflow` (configurable Todo→InProgress→
Review→Done) into the task-store transition usecase. Each transition
checks: (a) `(from_state, to_state)` is present in the workflow
definition; (b) caller's role is in `allowed_roles`; (c) optimistic-
concurrency version matches. Refuse otherwise with
`InvalidTransition::Refused` 422 (AC-04).

Cross-link to workflow-engine per ADR-TASKS-0005: every transition
emits `TaskStateTransitioned` on `tasks.task.state.v1`. The workflow-
engine consumes via the canonical bus; tasks µservice does NOT call
workflow-engine directly (cross-product rule). The reverse path —
workflow-engine creates / mutates tasks — uses the public gRPC service
`oya.tasks.v1.TaskStore` (the workflow-engine is a client like any
other tenant).

## ChangeSet boundary

`task-store-usecase` + `task-store-worker` (event emission); new
integration test against an in-memory NATS broker; cross-product-
isolation lane verifies no direct workflow-engine crate import.

## Crate Naming

n/a — modifies existing task-store crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-task-store-usecase/src/transition.rs` | created | state-machine orchestrator |
| `microservices/tasks/src/oya-tasks-task-store-worker/src/event_emitter.rs` | created | NATS publish |
| `microservices/tasks/tests/integration/state-workflow-emit.rs` | created | E2E |

## Acceptance Gates

```bash
cargo test -p oya-tasks-task-store-usecase transition
cargo test -p oya-tasks-task-store-worker event_emitter
cargo run -p oya-dev-cli -- gate validate task-state-machine-correctness --microservice tasks
cargo run -p oya-dev-cli -- gate validate cross-product-isolation --microservice tasks
```

## Test Plan

- Valid transition → 200 + event emitted with `from_state`,
  `to_state`, `transitioned_by_user_id_hashed`.
- Invalid transition → 422 `InvalidTransition::Refused`; NO event
  emitted.
- workflow-engine driving a transition: `via_workflow_id` populated on
  the emitted event.
- Cross-product isolation: no `oya-workflow-engine-*` crate imported.

## Halt Conditions

- Any transition emits an event without matching DB write — refuse
  (transaction outbox required).
- Direct workflow-engine crate dependency detected — refuse.

## Next IP

[`IP-010-view-engine-and-board-realtime.md`](IP-010-view-engine-and-board-realtime.md)

## References

- ADR-TASKS-0005 (cross-µservice automation via Workflow events).
- ADR-0140 (retired per ADR-0145) (Cedar role gating).
- PRD AC-04.
