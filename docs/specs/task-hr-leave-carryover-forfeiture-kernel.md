# Spec: hr-leave-carryover-forfeiture-kernel

## Objective

Add a pure deterministic period-boundary kernel function
`evaluate_leave_carryover_forfeiture` to `oya-hr-employment-domain`.

The existing `evaluate_leave_balance_accrual` hard-errors with
`CarryOverCapExceeded` whenever the closing balance exceeds the cap.
This slice adds a **distinct** function that instead **splits** the balance:
- `carried_over_units` = balance clamped to `[statutory_min_floor, cap]`
- `forfeited_units` = excess above cap (or 0 when balance ≤ cap)

This pattern is required for jurisdictions (e.g. Korea LSA Article 60) where
unused leave above a statutory cap is forfeited at period-end rather than
rejected as an error.

## Crate boundary

`oya-hr-employment-domain` only. Single flat crate per ADR-0509.
No new workspace member. No new file outside the crate.

## Mod layout (flat-clean-arch)

All additions live in `src/lib.rs` — the single module in this crate.
No sub-modules introduced for single-use logic (rule from ADR-0509).

## Contracts

- **No I/O** — pure deterministic function, no async, no side-effects.
- **No new dependencies** — only `data-boundary-kernel` (already declared).
- Field-level classification: all unit fields carry `DataClass::Financial` via
  `Classified<f64>`, consistent with `LeaveBalanceLedgerProjection`.
- `idempotency_key`: deterministic string `"{tenant_id}:{employee_id}:{period_boundary_date}:{rulepack_ref}"`.
- `schema_version`: `LEAVE_CARRYOVER_FORFEITURE_SCHEMA_VERSION` (= 1), classified `PUBLIC`.

## New public surface

```rust
pub struct LeaveCarryoverForfeitureInput {
    pub tenant_id: String,               // INTERNAL_ONLY
    pub legal_entity_id: String,         // INTERNAL_ONLY
    pub employee_id: String,             // INTERNAL_ONLY
    pub period_boundary_date: String,    // INTERNAL_ONLY  (ISO-8601 YYYY-MM-DD)
    pub closing_balance_units: f64,      // FINANCIAL
    pub statutory_min_floor_units: f64,  // FINANCIAL
    pub carry_over_cap_units: f64,       // FINANCIAL
    pub rulepack_ref: String,            // INTERNAL_ONLY
    pub rulepack_effective_date: String, // INTERNAL_ONLY
    pub evidence_ref: String,            // INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: u64, // INTERNAL_ONLY
}

pub struct LeaveCarryoverForfeitureProjection {
    pub tenant_id: Classified<TenantId>,                    // INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,         // INTERNAL_ONLY
    pub employee_id: Classified<EmployeeId>,                // INTERNAL_ONLY
    pub period_boundary_date: Classified<String>,           // INTERNAL_ONLY
    pub closing_balance_units: Classified<f64>,             // FINANCIAL
    pub statutory_min_floor_units: Classified<f64>,         // FINANCIAL
    pub carry_over_cap_units: Classified<f64>,              // FINANCIAL
    pub carried_over_units: Classified<f64>,                // FINANCIAL
    pub forfeited_units: Classified<f64>,                   // FINANCIAL
    pub rulepack_ref: Classified<RulepackRef>,              // INTERNAL_ONLY
    pub rulepack_effective_date: Classified<RulepackEffectiveDate>, // INTERNAL_ONLY
    pub evidence_ref: Classified<AuditEvidenceRef>,         // INTERNAL_ONLY
    pub idempotency_key: Classified<String>,                // INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: Classified<u64>,        // INTERNAL_ONLY
    pub schema_version: Classified<u32>,                    // PUBLIC
}

pub fn evaluate_leave_carryover_forfeiture(
    input: LeaveCarryoverForfeitureInput,
) -> Result<LeaveCarryoverForfeitureProjection, HrDomainError>
```

## New error variant

`HrDomainError::CarryOverCapBelowFloor` — returned when
`carry_over_cap_units < statutory_min_floor_units` (invalid policy configuration).

## New schema-version constant

`const LEAVE_CARRYOVER_FORFEITURE_SCHEMA_VERSION: u32 = 1;`

## Kernel logic

```
validate identifiers (tenant, legal entity, employee)
validate period_boundary_date (ISO-8601 YYYY-MM-DD)
validate rulepack_ref (rulepack/ prefix)
validate rulepack_effective_date (ISO-8601)
validate evidence_ref (audit/ prefix)
reject evaluated_at_epoch_seconds == 0

for each f64 in [closing_balance_units, statutory_min_floor_units, carry_over_cap_units]:
    reject if !is_finite() || < 0.0  -> InvalidAccrualUnits

if carry_over_cap_units < statutory_min_floor_units:
    -> CarryOverCapBelowFloor

carried_over = closing_balance_units.clamp(statutory_min_floor_units, carry_over_cap_units)
forfeited    = (closing_balance_units - carry_over_cap_units).max(0.0)
```

Note: when `closing_balance_units < statutory_min_floor_units`, the statutory
minimum is granted; forfeited remains 0 (no forfeiture when below floor).

## Testing strategy

Hermetic unit tests only, no I/O. File: `tests/leave_carryover_forfeiture.rs`.

| Test | Scenario |
|---|---|
| `balance_at_or_below_cap_zero_forfeiture` | balance=6, cap=10 → forfeited=0, carried=6 |
| `balance_above_cap_splits_correctly` | balance=12, cap=10, floor=5 → forfeited=2, carried=10 |
| `balance_below_floor_floor_granted` | balance=2, floor=5, cap=10 → carried=5, forfeited=0 |
| `cap_below_floor_returns_error` | cap=3, floor=5 → CarryOverCapBelowFloor |
| `negative_inputs_rejected` | closing=-1 → InvalidAccrualUnits |
| `nan_inputs_rejected` | closing=NaN → InvalidAccrualUnits |
| `financial_class_on_all_unit_fields` | all unit Classified fields carry FINANCIAL |
| `idempotency_key_format` | key = tenant:emp:date:rulepack |
| `schema_version_is_1` | schema_version.value == 1, data_class PUBLIC |

## Observability / SLO

Pure domain kernel — no SLO file required at this layer.
The calling service layer owns the SLO surface per ADR-0130.

## Cloud-native readiness

- No I/O → trivially containerisable and schedulable.
- Deterministic → safe for speculative execution in merge-queue workflows.
- No external dependencies → passes hyperscaler-lens filter.
