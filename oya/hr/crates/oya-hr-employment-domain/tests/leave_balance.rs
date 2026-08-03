#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::DataClass;
use oya_hr_employment_domain::{
    HrDomainError, LeaveBalanceAccrualInput, evaluate_leave_balance_accrual,
};

// ---------------------------------------------------------------------------
// [RED] Additional acceptance-criteria tests (hr-3 full coverage)
// These tests were written before the implementation per TDD discipline.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Happy-path
// ---------------------------------------------------------------------------

#[test]
fn test_happy_path_balance_projection() {
    // prior=5.0 + accrual=3.0 - deduction=2.0 = 6.0; cap=10.0 (not exceeded)
    let proj = evaluate_leave_balance_accrual(valid_input()).expect("happy-path projection");

    assert_eq!(proj.tenant_id.value.value, "ten_acme");
    assert_eq!(proj.legal_entity_id.value.value, "le_kr_001");
    assert_eq!(proj.employee_id.value.value, "emp_001");
    assert_eq!(proj.payroll_period.value, "2026-06");
    assert_eq!(proj.prior_accrued_units.value, 5.0);
    assert_eq!(proj.accrual_units.value, 3.0);
    assert_eq!(proj.deduction_units.value, 2.0);
    assert_eq!(proj.resulting_balance_units.value, 6.0);
    assert_eq!(proj.carried_over_units.value, 6.0);
    assert_eq!(proj.forfeited_units.value, 0.0);
    assert_eq!(proj.carry_over_cap_units.value, 10.0);
    assert_eq!(proj.rulepack_ref.value.value, "rulepack/kr-labor-2026");
    assert_eq!(proj.rulepack_effective_date.value.value, "2026-01-01");
    assert_eq!(
        proj.idempotency_key.value,
        "ten_acme:emp_001:2026-06:rulepack/kr-labor-2026"
    );
    assert_eq!(proj.schema_version.value, 1);

    // Financial DataClass assertions
    assert_eq!(
        proj.payroll_period
            .data_class
            .compatibility_data_class(),
        DataClass::Financial
    );
    assert_eq!(
        proj.resulting_balance_units
            .data_class
            .compatibility_data_class(),
        DataClass::Financial
    );
    assert_eq!(
        proj.carried_over_units
            .data_class
            .compatibility_data_class(),
        DataClass::Financial
    );
    assert_eq!(
        proj.forfeited_units
            .data_class
            .compatibility_data_class(),
        DataClass::Financial
    );
}

// ---------------------------------------------------------------------------
// Carry-over cap boundary
// ---------------------------------------------------------------------------

#[test]
fn test_exact_carry_over_cap_accepted() {
    // after_deduction == carry_over_cap_units → Ok
    let proj = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        prior_accrued_units: 5.0,
        accrual_units: 3.0,
        deduction_units: 2.0,
        carry_over_cap_units: 6.0, // exactly equals after_deduction
        ..valid_input()
    })
    .expect("exact cap boundary must be accepted");

    assert_eq!(proj.resulting_balance_units.value, 6.0);
    assert_eq!(proj.carried_over_units.value, 6.0);
    assert_eq!(proj.forfeited_units.value, 0.0);
}

#[test]
fn test_carry_over_cap_exceeded_returns_error() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        prior_accrued_units: 5.0,
        accrual_units: 3.0,
        deduction_units: 2.0,
        carry_over_cap_units: 5.9, // after_deduction=6.0 > cap=5.9
        ..valid_input()
    })
    .expect_err("over-cap must return CarryOverCapExceeded");

    assert_eq!(err, HrDomainError::CarryOverCapExceeded);
}

// ---------------------------------------------------------------------------
// Negative balance guard
// ---------------------------------------------------------------------------

#[test]
fn test_negative_balance_returns_error() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        prior_accrued_units: 1.0,
        accrual_units: 0.0,
        deduction_units: 5.0, // deduction > gross → negative
        ..valid_input()
    })
    .expect_err("negative result must return NegativeLeaveBalance");

    assert_eq!(err, HrDomainError::NegativeLeaveBalance);
}

// ---------------------------------------------------------------------------
// Invalid accrual units
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_accrual_units_negative() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        accrual_units: -1.0,
        ..valid_input()
    })
    .expect_err("negative accrual_units must return InvalidAccrualUnits");

    assert_eq!(err, HrDomainError::InvalidAccrualUnits);
}

#[test]
fn test_invalid_prior_accrued_units_nan() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        prior_accrued_units: f64::NAN,
        ..valid_input()
    })
    .expect_err("NaN prior_accrued_units must return InvalidAccrualUnits");

    assert_eq!(err, HrDomainError::InvalidAccrualUnits);
}

// ---------------------------------------------------------------------------
// Evidence ref validation
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_evidence_ref_rejected() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        accrual_evidence_ref: "audit/".to_owned(), // empty suffix
        ..valid_input()
    })
    .expect_err("empty evidence ref suffix must be rejected");

    assert_eq!(err, HrDomainError::InvalidAuditEvidenceRef);
}

// ---------------------------------------------------------------------------
// Rulepack ref validation
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_rulepack_ref_rejected() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        rulepack_ref: "policy/kr-labor".to_owned(), // wrong prefix
        ..valid_input()
    })
    .expect_err("wrong rulepack prefix must be rejected");

    assert_eq!(err, HrDomainError::InvalidRulepackRef);
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn valid_input() -> LeaveBalanceAccrualInput {
    LeaveBalanceAccrualInput {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        employee_id: "emp_001".to_owned(),
        payroll_period: "2026-06".to_owned(),
        prior_accrued_units: 5.0,
        accrual_units: 3.0,
        deduction_units: 2.0,
        carry_over_cap_units: 10.0,
        rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        accrual_evidence_ref: "audit/hr/leave-balance/emp_001/accrual".to_owned(),
        deduction_evidence_ref: "audit/hr/leave-balance/emp_001/deduction".to_owned(),
        decided_at_epoch_seconds: 1_779_532_800,
    }
}

// ---------------------------------------------------------------------------
// [RED] id / timestamp validation for evaluate_leave_balance_accrual
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_tenant_id_rejected() {
    // missing "ten_" prefix
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        tenant_id: "acme".to_owned(),
        ..valid_input()
    })
    .expect_err("tenant_id without ten_ prefix must be rejected");

    assert_eq!(err, HrDomainError::InvalidTenantId);
}

#[test]
fn test_invalid_legal_entity_id_rejected() {
    // missing "le_" prefix
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        legal_entity_id: "kr_001".to_owned(),
        ..valid_input()
    })
    .expect_err("legal_entity_id without le_ prefix must be rejected");

    assert_eq!(err, HrDomainError::InvalidLegalEntityId);
}

#[test]
fn test_invalid_employee_id_rejected() {
    // missing "emp_" prefix
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        employee_id: "001".to_owned(),
        ..valid_input()
    })
    .expect_err("employee_id without emp_ prefix must be rejected");

    assert_eq!(err, HrDomainError::InvalidEmployeeId);
}

#[test]
fn test_zero_decided_at_returns_invalid_decision_timestamp() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        decided_at_epoch_seconds: 0,
        ..valid_input()
    })
    .expect_err("zero decided_at_epoch_seconds must return InvalidDecisionTimestamp");

    assert_eq!(err, HrDomainError::InvalidDecisionTimestamp);
}

// ---------------------------------------------------------------------------
// [RED] payroll period and rulepack date validation
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_payroll_period_returns_error() {
    // month 99 is not a valid calendar month
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        payroll_period: "2026-99".to_owned(),
        ..valid_input()
    })
    .expect_err("payroll_period with invalid month must return InvalidPayrollPeriod");

    assert_eq!(err, HrDomainError::InvalidPayrollPeriod);
}

#[test]
fn test_invalid_rulepack_effective_date_returns_error() {
    // not ISO-8601 YYYY-MM-DD format
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        rulepack_effective_date: "01-01-2026".to_owned(),
        ..valid_input()
    })
    .expect_err("rulepack_effective_date not in YYYY-MM-DD format must be rejected");

    assert_eq!(err, HrDomainError::InvalidRulepackEffectiveDate);
}

// ---------------------------------------------------------------------------
// [RED] deduction evidence ref validation
// ---------------------------------------------------------------------------

#[test]
fn test_deduction_evidence_ref_empty_suffix_rejected() {
    // deduction_evidence_ref with only the prefix and no suffix
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        deduction_evidence_ref: "audit/".to_owned(),
        ..valid_input()
    })
    .expect_err("deduction_evidence_ref with empty suffix must be rejected");

    assert_eq!(err, HrDomainError::InvalidAuditEvidenceRef);
}

#[test]
fn test_deduction_evidence_ref_credential_like_rejected() {
    // evidence refs that look like credentials must be rejected
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        deduction_evidence_ref: "audit/hr/leave-balance/bearer-token".to_owned(),
        ..valid_input()
    })
    .expect_err("deduction_evidence_ref containing 'bearer' must be rejected");

    assert_eq!(err, HrDomainError::InvalidAuditEvidenceRef);
}

// ---------------------------------------------------------------------------
// [RED] accrual unit domain invariants
// ---------------------------------------------------------------------------

#[test]
fn test_infinite_carry_over_cap_returns_invalid_accrual_units() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        carry_over_cap_units: f64::INFINITY,
        ..valid_input()
    })
    .expect_err("infinite carry_over_cap_units must return InvalidAccrualUnits");

    assert_eq!(err, HrDomainError::InvalidAccrualUnits);
}

#[test]
fn test_negative_carry_over_cap_returns_invalid_accrual_units() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        carry_over_cap_units: -0.1,
        ..valid_input()
    })
    .expect_err("negative carry_over_cap_units must return InvalidAccrualUnits");

    assert_eq!(err, HrDomainError::InvalidAccrualUnits);
}

#[test]
fn test_infinite_accrual_units_returns_invalid_accrual_units() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        accrual_units: f64::INFINITY,
        ..valid_input()
    })
    .expect_err("infinite accrual_units must return InvalidAccrualUnits");

    assert_eq!(err, HrDomainError::InvalidAccrualUnits);
}

#[test]
fn test_negative_deduction_units_returns_invalid_accrual_units() {
    let err = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        deduction_units: -1.0,
        ..valid_input()
    })
    .expect_err("negative deduction_units must return InvalidAccrualUnits");

    assert_eq!(err, HrDomainError::InvalidAccrualUnits);
}

// ---------------------------------------------------------------------------
// [RED] happy-path: DataClass on all financial output fields
// ---------------------------------------------------------------------------

#[test]
fn test_all_financial_output_fields_carry_financial_data_class() {
    let proj = evaluate_leave_balance_accrual(valid_input()).expect("projection");

    // Every unit field must be classified FINANCIAL per the data-boundary contract
    for (name, dc) in [
        ("prior_accrued_units", proj.prior_accrued_units.data_class.compatibility_data_class()),
        ("accrual_units", proj.accrual_units.data_class.compatibility_data_class()),
        ("deduction_units", proj.deduction_units.data_class.compatibility_data_class()),
        ("resulting_balance_units", proj.resulting_balance_units.data_class.compatibility_data_class()),
        ("carried_over_units", proj.carried_over_units.data_class.compatibility_data_class()),
        ("forfeited_units", proj.forfeited_units.data_class.compatibility_data_class()),
        ("carry_over_cap_units", proj.carry_over_cap_units.data_class.compatibility_data_class()),
    ] {
        assert_eq!(dc, DataClass::Financial, "field {name} must be FINANCIAL");
    }
}

// ---------------------------------------------------------------------------
// [RED] happy-path: decided_at_epoch_seconds and rulepack_effective_date roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_decided_at_and_rulepack_date_round_trip_in_projection() {
    let proj = evaluate_leave_balance_accrual(valid_input()).expect("projection");

    assert_eq!(proj.decided_at_epoch_seconds.value, 1_779_532_800);
    assert_eq!(proj.rulepack_effective_date.value.value, "2026-01-01");
    assert_eq!(
        proj.accrual_evidence_ref.value.value,
        "audit/hr/leave-balance/emp_001/accrual"
    );
    assert_eq!(
        proj.deduction_evidence_ref.value.value,
        "audit/hr/leave-balance/emp_001/deduction"
    );
}

// ---------------------------------------------------------------------------
// [RED] happy-path: zero-balance edge (no prior, no accrual, no deduction)
// ---------------------------------------------------------------------------

#[test]
fn test_zero_balance_with_zero_accrual_and_deduction_is_valid() {
    // KR LSA: an employee with no prior balance and no accrual in the period
    // must still produce a valid projection (zero carry-over, zero forfeited)
    let proj = evaluate_leave_balance_accrual(LeaveBalanceAccrualInput {
        prior_accrued_units: 0.0,
        accrual_units: 0.0,
        deduction_units: 0.0,
        carry_over_cap_units: 0.0,
        ..valid_input()
    })
    .expect("zero-balance projection must succeed");

    assert_eq!(proj.resulting_balance_units.value, 0.0);
    assert_eq!(proj.carried_over_units.value, 0.0);
    assert_eq!(proj.forfeited_units.value, 0.0);
}
