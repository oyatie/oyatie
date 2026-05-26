---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P18-IP-001
title: Changeset state kernel + event log + monotonicity lane
status: scaffolded
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_adr: ../../../../../../docs/decisions/ADR-0110-changeset-state-machine.md
purpose: Land the foundational changeset state machine (12-value closed enum, monotonic transitions, event-sourced log) that every other M01-P18 IP depends on.
---

# M01-P18-IP-001 — Changeset state kernel + event log + monotonicity lane

## Scope

Implement ADR-0110 wave-A:

- New crate `oya-vcs-changeset-state-kernel` — closed-enum
  + monotonicity validator (port-in-kernel, pure-domain).
- New crate `oya-vcs-changeset-state-app` — runner that
  appends signed events to `registry/vcs/changeset-event-log.json`.
- New CI lane `oya-governance-changeset-state-monotonicity`
  — asserts every changeset's event log is monotonic.
- New CI lane `oya-governance-changeset-state-enum-closed`
  — asserts every emitted `to_state` is in the closed 12-value enum.

## Dependencies

None. Foundation IP. Must land before IP-002/003/004/005.

## Acceptance

- 12-value closed enum exposed at
  `oya_vcs_changeset_state_kernel::ChangesetState`.
- `validate_monotonic_event_log(events: &[ChangesetEvent])` returns
  `Result<MonotonicityReport, MonotonicityError>` per the kernel
  trait shape.
- `registry/vcs/changeset-event-log.json` initialized empty.
- First synthetic event (state `opened`) emitted via the app
  binary; smoke-test verifies the row is signed (Ed25519 per
  ADR-0058) and includes `dedup_key` + `cost_budget_remaining`.
- Both new fitness lanes wired via `oya gate validate
  changeset-state-monotonicity` and `... -enum-closed`; both
  green on the new empty log.

## Symbols to grit-claim

- `crates/oya-vcs-changeset-state-kernel/src/lib.rs::*`
- `crates/oya-vcs-changeset-state-app/src/main.rs::main`
- `registry/vcs/changeset-event-log.json::*` (initial empty)
- `crates/oya-dev-cli/src/{changeset_state_monotonicity_gate,changeset_state_enum_closed_gate}.rs::*`
- `crates/oya-dev-cli/src/commands/gate/mod.rs::run` (two new
  validate arms)

## Exit evidence

- `/evidence/agentic-vcs-pipeline/ip-001-changeset-state-kernel.json`
- `/evidence/agentic-vcs-pipeline/ip-001-monotonicity-lane-first-green.json`
