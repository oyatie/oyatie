# Spec: b2b-payroll-group-gl-posting-dispatch

vertical: b2b
crate: payroll-run-domain (single-crate; no new workspace member)
branch: feat/task-b2b-payroll-group-gl-posting-dispatch-2026-05-28
adr-layout: ADR-0131 flat per-microservice layout; ADR-0509 single-crate-per-service

## Objective

Add a group-level GL posting dispatch slice to the existing pure payroll-run domain
crate.  Given a set of per-entity `PayrollJournalInput` entries bundled under a single
`GroupGlPostingInput`, the builder produces a `GroupGlPostingBatch` containing one
`PayrollJournalDraft` per closed legal entity, aggregated group-level totals, and a
group idempotency key.

The builder is a strict gate: it refuses to emit unless (a) the entity set is
non-empty, (b) no legal entity appears more than once, and (c) every per-entity
journal balances (sum debits == sum credits).  Balancing is delegated entirely to the
existing `build_payroll_journal` — no duplicated logic.

## Vertical and Crate Boundaries

- Vertical: b2b
- Owning crate: `crates/payroll-run-domain` (`src/lib.rs`)
- No new crate; no root `Cargo.toml` change.
- No REST/gRPC adapter; domain-only additions.

## Data Model

### Input type: `GroupGlPostingInput`

```
pub struct GroupGlPostingInput {
    pub rollup_id: String,               // data_class: INTERNAL_ONLY  prefix: pgrp_
    pub tenant_id: String,               // data_class: INTERNAL_ONLY  prefix: ten_
    pub entries: Vec<PayrollJournalInput>, // data_class: INTERNAL_ONLY  one per legal entity
    pub group_idempotency_key: String,   // data_class: INTERNAL_ONLY
}
```

Each `PayrollJournalInput` entry carries its own `legal_entity_id`, `run_id`,
`journal_id`, etc. — all validated by the delegate `build_payroll_journal`.

### Output type: `GroupGlPostingBatch`

```
pub struct GroupGlPostingBatch {
    pub rollup_id: Classified<GroupPayrollRollupId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,             // data_class: INTERNAL_ONLY
    pub drafts: Classified<Vec<PayrollJournalDraft>>, // data_class: INTERNAL_ONLY
    pub total_debit_minor: Classified<i64>,          // data_class: INTERNAL_ONLY
    pub total_credit_minor: Classified<i64>,         // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,         // data_class: INTERNAL_ONLY
}
```

### New error variants

```
PayrollDomainError::GroupPostingEntitiesRequired
PayrollDomainError::DuplicateLegalEntityInGroup
```

Added to the existing `PayrollDomainError` enum in `src/lib.rs`.

## Builder Contract: `build_group_gl_posting`

```
pub fn build_group_gl_posting(
    input: GroupGlPostingInput,
) -> Result<GroupGlPostingBatch, PayrollDomainError>
```

Validation order:

1. `validate_identifier(&input.rollup_id, "pgrp_", InvalidGroupRollupId)`
2. `validate_identifier(&input.tenant_id, "ten_", InvalidTenantId)`
3. `validate_idempotency_key(&input.group_idempotency_key)`
4. `input.entries.is_empty()` → `GroupPostingEntitiesRequired`
5. Duplicate `legal_entity_id` scan across entries → `DuplicateLegalEntityInGroup`
6. Per-entry: `build_payroll_journal(entry)?` — propagates all errors verbatim
7. Accumulate `total_debit_minor` / `total_credit_minor` from draft classified values
8. Construct and return `GroupGlPostingBatch`

The function never panics.  It does not perform tax calculation, disbursement,
storage I/O, or workflow side-effects.

## Mod Layout (flat clean-arch within src/lib.rs)

This crate is a pure-domain single-file library.  All types and functions live in
`src/lib.rs`.  New additions are appended after the existing `build_payroll_journal`
function following the established section comment style.  No new modules, no new
files under `src/`.

## Testing Strategy

File: `tests/group_gl_posting.rs`

Pattern matches existing `tests/accounting_bridge.rs` and `tests/group.rs`:

- `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` header
- `mod support; use support::digest;` for shared fixtures
- One `#[test]` function per scenario; no parameterized macros

Test matrix:

| Test name | Scenario | Expected outcome |
|---|---|---|
| `test_two_balanced_entities_produces_correct_batch` | Two entities, both balanced | Batch with 2 drafts; group totals equal sum; debit == credit; idempotency key preserved |
| `test_unbalanced_entity_propagates_error` | One balanced + one unbalanced | `Err(UnbalancedJournal)` |
| `test_empty_entries_returns_required_error` | Empty entries vec | `Err(GroupPostingEntitiesRequired)` |
| `test_duplicate_legal_entity_returns_error` | Two entries same `legal_entity_id` | `Err(DuplicateLegalEntityInGroup)` |
| `test_invalid_identifier_returns_error` | Malformed `run_id` in one entry | `Err(InvalidRunId)` |

Pre-existing tests (`accounting_bridge`, `group`, `kr_close`, `filing`) must remain
green.

## Contracts (domain-only; no HTTP/gRPC surface)

This slice is pure domain logic.  There is no OpenAPI 3.2.0 or proto3 surface for
this task.  The `GroupGlPostingBatch` output is the contract boundary consumed by
a future posting-dispatch use-case adapter (not in scope here).

## OpenSLO

No new SLO file required for a pure-domain extension; the owning microservice's
existing SLO (if any) governs.  No cloud-native promotion gate is triggered by this
domain-only addition.

## Boundaries and Non-Goals

- No new crate, no new workspace member.
- No REST/gRPC adapter in this task.
- No disbursement, tax calculation, or filing logic.
- No external dependencies added.
- No changes to `Cargo.toml` (workspace or crate-level).
- Adjacent builder functions (`trial_close`, `close_group_rollup`, etc.) are not
  modified.
