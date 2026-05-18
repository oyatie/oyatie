---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-003-task-store-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-test, layer-correctness, port-location, task-state-machine-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: task-store kernel + domain — entities, ports, invariants

## Intent

Author the `task-store` BC kernel + domain layers. Kernel defines the
canonical entity types (`Task`, `TaskComment`, `TaskHistoryEntry`,
`LegalHoldRef`, `RetentionPolicyRef`) and port traits (`TaskRepository`,
`TaskHistoryStore`, `RetentionPolicyResolver`, `LegalHoldStore`).
`TaskContext` is a closed enum (`Personal | Professional`) enforced at
the type level — Cedar refusal at the API edge backs structural
isolation per ADR-TASKS-0001.

Domain layer carries pure invariant math: status-transition validity
(driven by the per-project `StatusWorkflow`); priority ordering;
custom-field type coercion (refuse silent string→number coerce per
ADR-TASKS-0001); legal-hold coverage propagation. Zero I/O; zero
business orchestration. `cargo test -p oya-tasks-task-store-domain`
covers AC-04 (`InvalidTransition::Refused`) + AC-07 (context isolation
at the domain layer).

## ChangeSet boundary

2 crates (kernel + domain) with full `lib.rs` + property tests +
`#[data_class(...)]` annotations on every kernel field per Bominal
ADR-0028. Adapters land in IP-004; usecase in IP-005.

## Crate Naming

`oya-tasks-task-store-kernel` + `oya-tasks-task-store-domain` per
ADR-0056 v4.1 BNF.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-task-store-kernel/src/lib.rs` | replaced | entity types + port traits |
| `microservices/tasks/src/oya-tasks-task-store-domain/src/lib.rs` | replaced | invariant math |
| `microservices/tasks/src/oya-tasks-task-store-domain/tests/*.rs` | created | property tests for state-machine + context isolation |
| `microservices/tasks/catalog/oya-tasks-task-store-kernel.yaml` | created | catalog entry |
| `microservices/tasks/catalog/oya-tasks-task-store-domain.yaml` | created | catalog entry |

## Acceptance Gates

```bash
cargo test -p oya-tasks-task-store-kernel
cargo test -p oya-tasks-task-store-domain
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice tasks
cargo run -p oya-dev-cli -- gate validate port-location --microservice tasks
cargo run -p oya-dev-cli -- gate validate task-state-machine-correctness --microservice tasks
```

## Test Plan

- State-machine: every `(from_state, to_state)` outside allowed
  transitions returns `InvalidTransition::Refused`.
- Context isolation: `Personal` task cannot leak into `Professional`
  query — enforced at the type level via separate enum branches and
  Cedar policy.
- Custom-field type coercion: `"42"` written to a `Number` field is
  refused (ADR-TASKS-0001 strict typing).
- Legal-hold propagation: hold open on task → propagates to comments +
  history + dependency edges + time-tracking entries (AC-06).

## Halt Conditions

- Any `Task` field missing `#[data_class(...)]` — refuse.
- State-machine property test reveals reachable invalid transition —
  fix domain; do not relax the test.

## Next IP

[`IP-004-task-store-adapter-postgres.md`](IP-004-task-store-adapter-postgres.md)

## References

- ADR-0028 Bominal (data-class annotation); ADR-0105 (13-layer); ADR-0106 (usecase).
- ADR-TASKS-0001 (data model + custom fields); PRD AC-04 + AC-06 + AC-07.
