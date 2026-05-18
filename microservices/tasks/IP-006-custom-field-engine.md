---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-006-custom-field-engine
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-test, type-coercion-strictness]
---

# IP-006: custom-field engine — typed-schema-per-project + flexible JSON

## Intent

Implement the hybrid custom-field model per ADR-TASKS-0001: typed
schema declared at the project level (per the eight supported kinds —
text, number, date, dropdown, multi_select, person, url, checkbox);
stored as JSONB column per task with sidecar `custom_field_definitions`
table. Validation refuses silent type coercion at write time (no
implicit string→number; no truncation of multi-select to scalar; no
date-string variance).

Eight `CustomFieldKind` validators authored in domain layer with
property tests covering boundary cases (empty multi-select; large
number; ISO-8601 date variance; URL scheme allowlist; person assignee
membership check).

## ChangeSet boundary

Code lives across `project-list-domain`, `project-list-usecase`,
`task-store-domain`, `task-store-usecase` — this IP adds the cross-
cutting custom-field test suite + the strict-coercion gate.

## Crate Naming

Crates already exist (IP-005 + IP-003); this IP adds property tests +
the `type-coercion-strictness` lane.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-project-list-domain/src/custom_field.rs` | created | typed validators |
| `microservices/tasks/src/oya-tasks-task-store-domain/src/custom_field_coercion.rs` | created | refusal logic |
| `microservices/tasks/src/oya-tasks-project-list-domain/tests/custom_field_property.rs` | created | proptest suite |
| `microservices/tasks/tests/integration/custom-field-strictness.rs` | created | E2E refusal path |

## Acceptance Gates

```bash
cargo test -p oya-tasks-project-list-domain custom_field
cargo test -p oya-tasks-task-store-domain custom_field
cargo run -p oya-dev-cli -- gate validate type-coercion-strictness --microservice tasks
```

## Test Plan

- Writing `"42"` (string) to a `Number` field → 422
  `CustomFieldCoercion::Refused`.
- Writing `42.0` to an integer-kind `Number` → accepts (Rust f64 with
  fractional 0).
- Multi-select with empty option array → refused unless field is
  optional.
- Person field referencing non-existent tenant member → refused.

## Halt Conditions

- Any test reveals silent coerce — refuse; fix at domain.

## Next IP

[`IP-007-dependency-graph-and-cycle-prevention.md`](IP-007-dependency-graph-and-cycle-prevention.md)

## References

- ADR-TASKS-0001 (typed schema per project).
- PRD §FR-16 + §"Hyrum" #2 + #7.
