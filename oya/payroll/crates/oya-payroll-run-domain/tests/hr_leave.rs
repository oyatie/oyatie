#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::DataClass;
use oya_payroll_run_domain::{
    HrLeaveImpactIntakeInput, HrLeaveImpactKind, PayrollDomainError, ingest_hr_leave_impact,
};

#[test]
fn test_hr_leave_impact_intake_requires_source_evidence() {
    let intake = ingest_hr_leave_impact(valid_input()).expect("HR leave impact intake");

    assert_eq!(intake.run_id.value.value, "prun_kr_2026_06");
    assert_eq!(intake.tenant_id.value.value, "ten_acme");
    assert_eq!(intake.legal_entity_id.value.value, "le_kr_001");
    assert_eq!(intake.payroll_period.value, "2026-06");
    assert_eq!(intake.payee_id.value.value, "payee_001");
    assert_eq!(intake.employee_id.value.value, "emp_001");
    assert_eq!(intake.leave_request_id.value.value, "leave_001");
    assert_eq!(intake.impact_kind.value, HrLeaveImpactKind::PaidLeave);
    assert_eq!(
        intake.source_topic.value,
        "integration.hr.payroll.leave-impact"
    );
    assert_eq!(
        intake.routing_evidence_ref.value.value,
        "audit/hr/leave/leave_001/delegation"
    );
    assert_eq!(
        intake.payroll_impact_evidence_ref.value.value,
        "audit/hr/leave/leave_001/payroll-impact"
    );
    assert_eq!(
        intake
            .payroll_impact_evidence_ref
            .data_class
            .compatibility_data_class(),
        DataClass::Financial
    );
    assert_eq!(
        intake.idempotency_key.value,
        "prun_kr_2026_06:payee_001:leave_001:2026-06:PaidLeave:ten_acme:leave_001:Approved:2026-06"
    );
    assert_eq!(intake.schema_version.value, 1);
}

#[test]
fn hr_leave_impact_rejects_wrong_source_topic() {
    let error = ingest_hr_leave_impact(HrLeaveImpactIntakeInput {
        source_topic: "integration.hr.payroll.raw-leave".to_owned(),
        ..valid_input()
    })
    .expect_err("source topic must be the HR leave impact topic");

    assert_eq!(error, PayrollDomainError::InvalidHrLeaveImpactTopic);
}

#[test]
fn hr_leave_impact_rejects_unsafe_payroll_evidence() {
    let error = ingest_hr_leave_impact(HrLeaveImpactIntakeInput {
        payroll_impact_evidence_ref: "audit/hr/leave/bearer-token".to_owned(),
        ..valid_input()
    })
    .expect_err("payroll impact evidence cannot look like credentials");

    assert_eq!(error, PayrollDomainError::InvalidEvidenceRef);
}

#[test]
fn hr_leave_impact_rejects_invalid_period_and_idempotency() {
    let period_error = ingest_hr_leave_impact(HrLeaveImpactIntakeInput {
        payroll_period: "2026-99".to_owned(),
        ..valid_input()
    })
    .expect_err("payroll period must be YYYY-MM with real month");
    assert_eq!(period_error, PayrollDomainError::InvalidPeriod);

    let idempotency_error = ingest_hr_leave_impact(HrLeaveImpactIntakeInput {
        source_hr_idempotency_key: "ten_acme leave_001".to_owned(),
        ..valid_input()
    })
    .expect_err("source HR idempotency key must be safe metadata");
    assert_eq!(idempotency_error, PayrollDomainError::InvalidIdempotencyKey);
}

fn valid_input() -> HrLeaveImpactIntakeInput {
    HrLeaveImpactIntakeInput {
        run_id: "prun_kr_2026_06".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payroll_period: "2026-06".to_owned(),
        payee_id: "payee_001".to_owned(),
        employee_id: "emp_001".to_owned(),
        leave_request_id: "leave_001".to_owned(),
        impact_kind: HrLeaveImpactKind::PaidLeave,
        source_topic: "integration.hr.payroll.leave-impact".to_owned(),
        source_hr_idempotency_key: "ten_acme:leave_001:Approved:2026-06".to_owned(),
        decision_evidence_ref: "audit/hr/leave/leave_001/decision".to_owned(),
        routing_evidence_ref: "audit/hr/leave/leave_001/delegation".to_owned(),
        payroll_impact_evidence_ref: "audit/hr/leave/leave_001/payroll-impact".to_owned(),
        payroll_intake_evidence_ref: "audit/payroll/hr-leave/leave_001/intake".to_owned(),
        rulepack_ref: "rulepack/kr-payroll-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        received_at_epoch_seconds: 1_779_535_200,
    }
}
