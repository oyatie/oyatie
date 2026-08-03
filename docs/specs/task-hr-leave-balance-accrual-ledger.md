# Spec: HR Leave Balance Accrual Ledger

status: preview-not-deployed
vertical: b2b
crate: oya-hr-employment-domain
x-oyatie-contract-status: preview-not-deployed
x-oyatie-non-claims:
  - no deployed HTTP runtime
  - no storage adapter
  - no payroll calculation runtime
  - no statutory filing transport
  - no workflow dispatch execution

## Objective

Extend `oya-hr-employment-domain` (flat single-crate per ADR-0509) with a pure-domain
leave-balance accrual/deduction evaluator. The evaluator mirrors the existing
`plan_leave_payroll_impact` pattern: it accepts a validated input envelope, applies
rulepack-bound business rules, and emits a `Classified` projection with deterministic
idempotency key and `schema_version`.

No network I/O, no storage, no workflow dispatch. All computation is pure domain logic.

## Vertical and ownership

| Property         | Value                          |
|------------------|--------------------------------|
| vertical         | b2b                            |
| crate            | oya-hr-employment-domain       |
| src root         | crates/oya-hr-employment-domain/src/lib.rs |
| test file        | crates/oya-hr-employment-domain/tests/leave_balance.rs |
| lane             | feat/task-hr-leave-balance-accrual-ledger-2026-05-28 |

## Mod layout (flat clean-arch per ADR-0509)

All types and the evaluator function live in `src/lib.rs` (the crate has a single-file
domain layer; no sub-mods are introduced). This mirrors every prior addition in this crate
(`plan_leave_payroll_impact`, `evaluate_labor_compliance`, `evaluate_sensitive_hr_read`).

## New types

### LeaveBalanceAccrualInput

Flat input envelope. All string fields are validated by existing crate helpers before use.

| Field                    | Type    | data_class     | Description |
|--------------------------|---------|----------------|-------------|
| tenant_id                | String  | INTERNAL_ONLY  | Validated with `ten_` prefix |
| legal_entity_id          | String  | INTERNAL_ONLY  | Validated with `le_` prefix |
| employee_id              | String  | INTERNAL_ONLY  | Validated with `emp_` prefix |
| payroll_period           | String  | FINANCIAL      | YYYY-MM; validated by `validate_payroll_period` |
| prior_accrued_units      | f64     | FINANCIAL      | Existing balance in leave units; must be >= 0 |
| accrual_units            | f64     | FINANCIAL      | Units accruing this period; must be >= 0 |
| deduction_units          | f64     | FINANCIAL      | Units consumed by approved leave; must be >= 0 |
| carry_over_cap_units     | f64     | FINANCIAL      | Maximum balance that carries over; must be >= 0 |
| rulepack_ref             | String  | INTERNAL_ONLY  | Validated with `rulepack/` prefix |
| rulepack_effective_date  | String  | INTERNAL_ONLY  | ISO date YYYY-MM-DD |
| accrual_evidence_ref     | String  | INTERNAL_ONLY  | Validated with `audit/` prefix |
| deduction_evidence_ref   | String  | INTERNAL_ONLY  | Validated with `audit/` prefix |
| decided_at_epoch_seconds | u64     | INTERNAL_ONLY  | Must be > 0 |

### LeaveBalanceLedgerProjection

Output projection; all fields wrapped in `Classified<T>`.

| Field                    | Type          | data_class     | Description |
|--------------------------|---------------|----------------|-------------|
| tenant_id                | TenantId      | INTERNAL_ONLY  | |
| legal_entity_id          | LegalEntityId | INTERNAL_ONLY  | |
| employee_id              | EmployeeId    | INTERNAL_ONLY  | |
| payroll_period           | String        | FINANCIAL      | Carries `DataClass::Financial` |
| prior_accrued_units      | f64           | FINANCIAL      | Input echo |
| accrual_units            | f64           | FINANCIAL      | Input echo |
| deduction_units          | f64           | FINANCIAL      | Input echo |
| resulting_balance_units  | f64           | FINANCIAL      | prior + accrual - deduction; clamped |
| carried_over_units       | f64           | FINANCIAL      | = resulting_balance_units (== cap or less) |
| forfeited_units          | f64           | FINANCIAL      | = 0.0 (cap is a hard guard, not truncation) |
| carry_over_cap_units     | f64           | FINANCIAL      | Input echo |
| rulepack_ref             | RulepackRef   | INTERNAL_ONLY  | |
| rulepack_effective_date  | RulepackEffectiveDate | INTERNAL_ONLY | |
| accrual_evidence_ref     | AuditEvidenceRef | INTERNAL_ONLY | |
| deduction_evidence_ref   | AuditEvidenceRef | INTERNAL_ONLY | |
| idempotency_key          | String        | INTERNAL_ONLY  | Deterministic; see derivation below |
| decided_at_epoch_seconds | u64           | INTERNAL_ONLY  | |
| schema_version           | u32           | PUBLIC         | = 1 |

### New HrDomainError variants

| Variant              | Trigger condition |
|----------------------|-------------------|
| InvalidAccrualUnits  | prior_accrued_units, accrual_units, deduction_units, or carry_over_cap_units is negative (< 0.0) or NaN |
| NegativeLeaveBalance | (prior + accrual - deduction) < 0.0 |
| CarryOverCapExceeded | (prior + accrual - deduction) > carry_over_cap_units |

## Evaluator function signature

```rust
pub fn evaluate_leave_balance_accrual(
    input: LeaveBalanceAccrualInput,
) -> Result<LeaveBalanceLedgerProjection, HrDomainError>
```

### Business logic (ordered)

1. Validate tenant_id, legal_entity_id, employee_id (identifier helpers).
2. Validate payroll_period (`validate_payroll_period`).
3. Validate rulepack_ref (`validate_ref` with `rulepack/` prefix).
4. Validate rulepack_effective_date (`validate_iso_date`).
5. Validate accrual_evidence_ref and deduction_evidence_ref (`validate_evidence_ref`).
6. Validate decided_at_epoch_seconds > 0.
7. Validate all unit fields are finite and >= 0.0; return `InvalidAccrualUnits` otherwise.
8. Compute `gross = prior_accrued_units + accrual_units`.
9. Compute `after_deduction = gross - deduction_units`.
10. If `after_deduction < 0.0` return `NegativeLeaveBalance`.
11. If `after_deduction > carry_over_cap_units` return `CarryOverCapExceeded`.
12. `resulting_balance_units = after_deduction` (which is <= cap).
13. `carried_over_units = resulting_balance_units`.
14. `forfeited_units = 0.0` (no silent truncation; cap is a hard guard).
15. Derive `idempotency_key = format!("{tenant_id}:{employee_id}:{payroll_period}:{rulepack_ref}")`.
16. `schema_version = LEAVE_BALANCE_LEDGER_SCHEMA_VERSION` (= 1).

## Idempotency key derivation

```
{tenant_id}:{employee_id}:{payroll_period}:{rulepack_ref}
```

Example: `ten_acme:emp_001:2026-06:rulepack/kr-labor-2026`

Deterministic across retries for the same (tenant, employee, period, rulepack) tuple.
Collisions across distinct inputs for the same key are acceptable at the domain layer
(upstream callers own deduplication storage).

## OpenAPI 3.2.0 contract fragment

The following path fragment extends `microservices/hr/contracts/openapi-v1.yaml` in a future
adapter PR. It is informational for this domain-only spec.

```yaml
/hr/v1/leave-balance-ledger-projections:
  post:
    operationId: evaluateLeaveBalanceAccrual
    summary: Project leave-balance accrual ledger for a payroll period.
    x-oyatie-contract-status: preview-not-deployed
    x-oyatie-app-function: oya_hr_employment_domain::evaluate_leave_balance_accrual
    requestBody:
      required: true
      content:
        application/json:
          schema:
            $ref: "#/components/schemas/LeaveBalanceAccrualRequest"
    responses:
      "200":
        description: Leave balance ledger projection.
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/LeaveBalanceLedgerProjectionResponse"
      "400":
        $ref: "#/components/responses/ValidationError"
      "422":
        description: Business rule violation (negative balance or cap exceeded).
        content:
          application/json:
            schema:
              $ref: "#/components/responses/DomainError"
```

## Proto3 contract fragment

The following message definitions are informational for future gRPC adapter work.

```proto
syntax = "proto3";
package oyatie.hr.v1;

message LeaveBalanceAccrualRequest {
  string tenant_id                = 1;
  string legal_entity_id          = 2;
  string employee_id              = 3;
  string payroll_period           = 4;
  double prior_accrued_units      = 5;
  double accrual_units            = 6;
  double deduction_units          = 7;
  double carry_over_cap_units     = 8;
  string rulepack_ref             = 9;
  string rulepack_effective_date  = 10;
  string accrual_evidence_ref     = 11;
  string deduction_evidence_ref   = 12;
  uint64 decided_at_epoch_seconds = 13;
}

message LeaveBalanceLedgerProjectionResponse {
  string tenant_id                = 1;
  string legal_entity_id          = 2;
  string employee_id              = 3;
  string payroll_period           = 4;
  double prior_accrued_units      = 5;
  double accrual_units            = 6;
  double deduction_units          = 7;
  double resulting_balance_units  = 8;
  double carried_over_units       = 9;
  double forfeited_units          = 10;
  double carry_over_cap_units     = 11;
  string rulepack_ref             = 12;
  string rulepack_effective_date  = 13;
  string idempotency_key          = 14;
  uint64 decided_at_epoch_seconds = 15;
  uint32 schema_version           = 16;
}
```

## Testing strategy

File: `crates/oya-hr-employment-domain/tests/leave_balance.rs`

Follows the `tests/leave.rs` idiom:
- `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` header
- `use oya_hr_employment_domain::{...}` imports
- `use data_boundary_kernel::DataClass;` for financial DataClass assertions
- Private `valid_input()` helper returning a known-good `LeaveBalanceAccrualInput`

Test matrix:

| Test name | Scenario | Key assertion |
|-----------|----------|---------------|
| test_happy_path_balance_projection | prior=5.0 accrual=3.0 deduction=2.0 cap=10.0 | resulting=6.0, carried_over=6.0, forfeited=0.0, idempotency_key format, schema_version=1, payroll_period DataClass::Financial |
| test_exact_carry_over_cap_accepted | after_deduction == carry_over_cap | Ok result, resulting=cap |
| test_carry_over_cap_exceeded_returns_error | after_deduction > cap | CarryOverCapExceeded |
| test_negative_balance_returns_error | deduction > gross | NegativeLeaveBalance |
| test_invalid_accrual_units_negative | accrual_units = -1.0 | InvalidAccrualUnits |
| test_invalid_evidence_ref_rejected | accrual_evidence_ref = "audit/" (empty suffix) | InvalidAuditEvidenceRef |
| test_invalid_rulepack_ref_rejected | rulepack_ref = "policy/kr-labor" | InvalidRulepackRef |

## Boundaries

- No new crate, no new workspace member, no root Cargo.toml edit.
- No storage, no HTTP runtime, no workflow dispatch.
- No changes to any other crate.
- Financial fields carry `DataClass::Financial` (via `Classified::new(v, DataClass::Financial)`).
- Internal fields carry `PrivacyDataClass::internal_only()` (via `internal()` helper).
- Public fields (schema_version) carry `DataClass::Public` (via `public()` helper).

## OpenSLO

SLO authoring is not in scope for this pure-domain task (no runtime adapter).
The existing `microservices/hr/` SLO path governs the future REST/gRPC adapter PR.
