---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-005-project-and-board-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-test, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: project-list BC + board groundwork

## Intent

Ship the `project-list` BC end-to-end: `kernel` + `domain` + `usecase` +
`api` + `adapter` + `adapter-postgres` + `rest` + `app`. Entities:
`Project`, `CustomFieldSchema` (referenced separately in IP-006),
`StatusWorkflow`, `ProjectMember`, `Sprint`, `Milestone`. Project
creation provisions tenant-DEK-scoped default custom-field schema +
default status-workflow (Todo→InProgress→Review→Done) + default board
view. Sprint / milestone records ride on project-list (PRD §FR-13 +
FR-14).

Board groundwork: the `view-engine` BC kernel + domain layers land here
too (board rendering invariants, deterministic rank-key generation per
ADR-TASKS-0004); the realtime / CRDT integration ships in IP-010.

## ChangeSet boundary

7 project-list crates + 2 view-engine crates (kernel + domain). Cedar
policy entries authored per project-membership role (owner / admin /
editor / viewer).

## Crate Naming

`oya-tasks-project-list-{kernel,domain,usecase,api,adapter,adapter-
postgres,rest,app}` + `oya-tasks-view-engine-{kernel,domain}` per ADR-
0056 v4.1.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-project-list-*/src/lib.rs` | replaced | 7-crate BC stack |
| `microservices/tasks/src/oya-tasks-view-engine-{kernel,domain}/src/lib.rs` | replaced | board invariant math + rank-key |
| `microservices/tasks/catalog/oya-tasks-project-list-*.yaml` | created | catalog entries |
| `microservices/tasks/catalog/oya-tasks-view-engine-{kernel,domain}.yaml` | created | catalog entries |
| `microservices/tasks/policy/project-membership-role.cedar` | created | per-role authorisation |

## Acceptance Gates

```bash
cargo test -p oya-tasks-project-list-domain
cargo test -p oya-tasks-project-list-usecase
cargo test -p oya-tasks-view-engine-domain
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice tasks
```

## Test Plan

- Default-template project provisions custom-field schema + status-
  workflow + default board view in one transaction.
- Project-member role transitions tested against Cedar policy.
- Sprint/milestone domain math (start ≤ end; capacity ≥ 0).
- Rank-key determinism: identical reorder ops always produce identical
  final order (ADR-TASKS-0004 AC).

## Halt Conditions

- Status-workflow not persisted at project create — block.
- Rank-key non-determinism — refuse; fix at domain.

## Next IP

[`IP-006-custom-field-engine.md`](IP-006-custom-field-engine.md)

## References

- ADR-0140 (retired per ADR-0145) (Cedar pack overlay); ADR-TASKS-0001 (schema-per-project);
  ADR-TASKS-0004 (rank-key + CRDT).
- PRD AC-04 + PRD §FR-13 + FR-14.
