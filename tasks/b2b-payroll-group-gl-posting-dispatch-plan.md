# Task Plan: b2b-payroll-group-gl-posting-dispatch

vertical: b2b
crate: oya-payroll-run-domain
branch: feat/task-b2b-payroll-group-gl-posting-dispatch-2026-05-28

## Objective

Extend the existing pure-domain crate with a group-level GL posting dispatch slice.
Given a `GroupPayrollRollup` (one closed legal entity per slot), produce a
`GroupGlPostingBatch` that holds one `PayrollJournalDraft` per entity, an aggregated
group-level debit/credit total, and a group idempotency key.  The builder refuses to
emit unless every entity's per-entity journal balances and no entity appears twice.

## Subtasks

### [st1] Add group GL posting model to `crates/oya-payroll-run-domain/src/lib.rs`

Add the following new public types alongside the existing journal types:

- `GroupGlPostingInput` — flat input carrying `rollup_id: String`,
  `tenant_id: String`, `entries: Vec<PayrollJournalInput>`, and
  `group_idempotency_key: String`.
- `GroupGlPostingBatch` — classified output carrying `rollup_id`,
  `tenant_id`, `drafts: Vec<PayrollJournalDraft>`, `total_debit_minor`,
  `total_credit_minor`, and `idempotency_key`.
- Two new `PayrollDomainError` variants: `GroupPostingEntitiesRequired` and
  `DuplicateLegalEntityInGroup`.

All classified fields must carry `data_class` annotations consistent with the
existing `PayrollJournalDraft` pattern.

Acceptance: `cargo check -p oya-payroll-run-domain --all-targets` is clean; new
types compile; data-class annotations match existing conventions; no duplication of
balancing logic.

### [st2] Implement `build_group_gl_posting` pure builder fn

Add a public `build_group_gl_posting(input: GroupGlPostingInput) -> Result<GroupGlPostingBatch, PayrollDomainError>` function in `src/lib.rs`:

1. Validate `rollup_id` via `validate_identifier` (prefix `pgrp_`).
2. Validate `tenant_id` via `validate_identifier` (prefix `ten_`).
3. Validate `group_idempotency_key` via `validate_idempotency_key`.
4. Reject empty `entries` with `GroupPostingEntitiesRequired`.
5. Detect duplicate `legal_entity_id` values across entries; return
   `DuplicateLegalEntityInGroup` on first collision.
6. Delegate each entry to `build_payroll_journal`; propagate any error
   (`UnbalancedJournal`, `JournalLinesRequired`, `InvalidRunId`, …) directly.
7. Accumulate `total_debit_minor` and `total_credit_minor` from each draft's
   classified values; assert group debit == group credit before constructing the
   batch (this is structurally guaranteed by per-entity balance, but the
   accumulation must be verified).
8. Return `GroupGlPostingBatch` with all classified fields.

Never panics. Returns `Result<_, PayrollDomainError>`.

Acceptance: Builder returns a batch only when all per-entity journals balance; any
unbalanced entity propagates `UnbalancedJournal`; empty entity set returns
`GroupPostingEntitiesRequired`; duplicate legal entity returns
`DuplicateLegalEntityInGroup`; group totals equal the sum of per-entity debit/credit
and group debit == group credit.

### [st3] Add `tests/group_gl_posting.rs`

Following the `tests/<topic>.rs` convention (same header pattern as
`tests/accounting_bridge.rs` and `tests/group.rs`):

- `test_two_balanced_entities_produces_correct_batch`: two entities, distinct
  `legal_entity_id` values, both balanced journals; assert batch has 2 drafts,
  `total_debit_minor == sum of per-draft debits`, `total_debit_minor ==
  total_credit_minor`, idempotency key preserved.
- `test_unbalanced_entity_propagates_error`: one balanced + one unbalanced entry;
  assert `Err(PayrollDomainError::UnbalancedJournal)`.
- `test_empty_entries_returns_required_error`: empty entries vec; assert
  `Err(PayrollDomainError::GroupPostingEntitiesRequired)`.
- `test_duplicate_legal_entity_returns_error`: two entries with identical
  `legal_entity_id`; assert `Err(PayrollDomainError::DuplicateLegalEntityInGroup)`.
- `test_invalid_identifier_returns_error`: entry with a malformed `run_id`; assert
  matching `PayrollDomainError` (e.g. `InvalidRunId`).

Acceptance: `cargo nextest run -p oya-payroll-run-domain` passes including all new
group GL posting tests; pre-existing `accounting_bridge`, `group`, `kr_close`, and
`filing` tests remain green.

## Constraints

- No new workspace member; no root `Cargo.toml` edit.
- No REST/gRPC adapter; domain-only.
- Reuse `build_payroll_journal` for per-entity balancing — no duplicated balance logic.
- All `Classified<T>` wrappers must carry `data_class` comments matching existing pattern.
- `validate_idempotency_key` enforces the existing charset (`ascii-alnum | : | _ | -`).
