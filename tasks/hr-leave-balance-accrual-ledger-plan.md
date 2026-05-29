# Task Plan: hr-leave-balance-accrual-ledger

vertical: b2b
crate: oya-hr-employment-domain
branch: feat/task-hr-leave-balance-accrual-ledger-2026-05-28
base: origin/dev

## Objective

Extend the HR employment domain crate with a pure-domain leave-balance accrual/deduction
evaluator. Given an employee's prior accrued balance plus a rulepack-governed accrual policy
and an approved LeavePayrollImpactPlan, compute the resulting leave-balance ledger projection
(carry-over cap, negative-balance guard, rulepack-effective-date binding) and emit a Classified
projection with evidence ref, idempotency key, and schema_version.

## Subtasks

### [hr-1] Data types and error variants

**What:** Add `LeaveBalanceAccrualInput`, `LeaveBalanceLedgerProjection`, and new
`HrDomainError` variants (`InvalidAccrualUnits`, `NegativeLeaveBalance`,
`CarryOverCapExceeded`) to `src/lib.rs`.

**Acceptance:**
- `cargo check -p oya-hr-employment-domain --all-targets` passes
- New types and variants compile and are `pub`-exported
- No edits to root `Cargo.toml` or any other crate

---

### [hr-2] Evaluator implementation

**What:** Implement `evaluate_leave_balance_accrual(input: LeaveBalanceAccrualInput) ->
Result<LeaveBalanceLedgerProjection, HrDomainError>` in `src/lib.rs`.

Logic:
1. Validate all ids/evidence-refs/dates using existing crate helpers.
2. Validate `prior_accrued_units >= 0.0` and `accrual_units >= 0.0` and
   `deduction_units >= 0.0`; return `InvalidAccrualUnits` on violation.
3. Compute `gross = prior_accrued_units + accrual_units`.
4. Apply deduction: `after_deduction = gross - deduction_units`.
5. Negative-balance guard: if `after_deduction < 0.0` return `NegativeLeaveBalance`.
6. Apply carry-over cap: `carried_over = after_deduction.min(carry_over_cap_units)`,
   `forfeited = after_deduction - carried_over`.
7. If `after_deduction > carry_over_cap_units` return `CarryOverCapExceeded`
   (projection is NOT emitted — the cap is a hard guard, not silent truncation).
8. Derive deterministic `idempotency_key =
   "{tenant_id}:{employee_id}:{payroll_period}:{rulepack_ref}"`.
9. Set `schema_version = 1`.

**Acceptance:**
- Function compiles
- Valid input returns projection where `resulting_balance_units = prior + accrued - deducted`
  clamped to carry-over cap
- Negative result yields `HrDomainError::NegativeLeaveBalance`
- Over-cap yields `HrDomainError::CarryOverCapExceeded`

---

### [hr-3] Integration tests

**What:** Add `tests/leave_balance.rs` following the `tests/leave.rs` idiom.

Tests required:
- `test_happy_path_balance_projection` — valid input, assert balance math + carry_over +
  forfeited + idempotency_key + schema_version + DataClass on financial fields
- `test_exact_carry_over_cap_accepted` — `after_deduction == carry_over_cap_units` is OK
- `test_carry_over_cap_exceeded_returns_error` — over cap returns `CarryOverCapExceeded`
- `test_negative_balance_returns_error` — deduction > gross returns `NegativeLeaveBalance`
- `test_invalid_accrual_units_negative` — negative accrual_units returns `InvalidAccrualUnits`
- `test_invalid_evidence_ref_rejected` — bad evidence ref returns `InvalidAuditEvidenceRef`
- `test_invalid_rulepack_ref_rejected` — bad rulepack_ref returns `InvalidRulepackRef`

**Acceptance:**
- `cargo nextest run -p oya-hr-employment-domain` is green including new leave_balance tests
- Existing tests still pass

## Boundaries

- Only `crates/oya-hr-employment-domain/src/lib.rs` and
  `crates/oya-hr-employment-domain/tests/leave_balance.rs` are modified/created.
- Root `Cargo.toml` is never touched.
- No other crate is touched.
- No new workspace members are added.
