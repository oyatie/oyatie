#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::DataClass;
use payroll_run_app::prepare_hr_leave_impact_intake;
use payroll_run_domain::{HrLeaveImpactIntakeInput, HrLeaveImpactKind};

#[test]
fn hr_leave_impact_intake_envelope_is_metadata_only() {
    let outcome = prepare_hr_leave_impact_intake(valid_input()).expect("HR leave impact intake");

    assert_eq!(
        outcome.intake_envelope.topic.value,
        "integration.payroll.hr.leave-impact-intake"
    );
    assert_eq!(
        outcome.intake_envelope.run_id.value.value,
        "prun_kr_2026_06"
    );
    assert_eq!(outcome.intake_envelope.payroll_period.value, "2026-06");
    assert_eq!(
        outcome.intake_envelope.leave_request_id.value.value,
        "leave_001"
    );
    assert_eq!(
        outcome.intake_envelope.impact_kind.value,
        HrLeaveImpactKind::UnpaidLeaveDeduction
    );
    assert_eq!(
        outcome
            .intake_envelope
            .payroll_impact_evidence_ref
            .value
            .value,
        "audit/hr/leave/leave_001/payroll-impact"
    );
    assert_eq!(
        outcome.intake_envelope.payload_data_class.value,
        DataClass::Financial
    );
    assert_eq!(outcome.intake_envelope.schema_version.value, 1);
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
        impact_kind: HrLeaveImpactKind::UnpaidLeaveDeduction,
        source_topic: "integration.hr.payroll.leave-impact".to_owned(),
        source_hr_idempotency_key: "ten_acme:leave_001:Approved:2026-06".to_owned(),
        decision_evidence_ref: "audit/hr/leave/leave_001/decision".to_owned(),
        routing_evidence_ref: "audit/hr/leave/leave_001/escalation".to_owned(),
        payroll_impact_evidence_ref: "audit/hr/leave/leave_001/payroll-impact".to_owned(),
        payroll_intake_evidence_ref: "audit/payroll/hr-leave/leave_001/intake".to_owned(),
        rulepack_ref: "rulepack/kr-payroll-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        received_at_epoch_seconds: 1_779_535_200,
    }
}
