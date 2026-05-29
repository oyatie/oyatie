#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_hr_employment_domain::{
    HrDomainError, LeaveBalanceAccrualInput, evaluate_leave_balance_accrual,
};

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
