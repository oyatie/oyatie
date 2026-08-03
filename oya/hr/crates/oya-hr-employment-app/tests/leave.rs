#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::DataClass;
use oya_hr_employment_app::plan_leave_payroll_impact_envelope;
use oya_hr_employment_domain::{
    LeaveDecision, LeavePayrollImpactInput, LeaveRoutingMode, PayrollImpactKind,
};

#[test]
fn leave_payroll_impact_envelope_is_metadata_only() {
    let outcome = plan_leave_payroll_impact_envelope(valid_input()).expect("leave payroll outcome");

    assert_eq!(
        outcome.payroll_impact_envelope.topic.value,
        "integration.hr.payroll.leave-impact"
    );
    assert_eq!(
        outcome.payroll_impact_envelope.leave_request_id.value.value,
        "leave_001"
    );
    assert_eq!(
        outcome.payroll_impact_envelope.decision.value,
        LeaveDecision::Approved
    );
    assert_eq!(
        outcome.payroll_impact_envelope.routing_mode.value,
        LeaveRoutingMode::EscalatedHr
    );
    assert_eq!(
        outcome.payroll_impact_envelope.payroll_period.value,
        "2026-06"
    );
    assert_eq!(
        outcome.payroll_impact_envelope.payroll_impact_kind.value,
        PayrollImpactKind::UnpaidLeaveDeduction
    );
    assert_eq!(
        outcome
            .payroll_impact_envelope
            .payroll_impact_evidence_ref
            .value
            .value,
        "audit/hr/leave/leave_001/payroll-impact"
    );
    assert_eq!(
        outcome.payroll_impact_envelope.payload_data_class.value,
        DataClass::Financial
    );
    assert_eq!(outcome.payroll_impact_envelope.schema_version.value, 1);
}

fn valid_input() -> LeavePayrollImpactInput {
    LeavePayrollImpactInput {
        leave_request_id: "leave_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        employee_id: "emp_001".to_owned(),
        approver_id: "emp_hr_001".to_owned(),
        decision: LeaveDecision::Approved,
        routing_mode: LeaveRoutingMode::EscalatedHr,
        start_date: "2026-06-01".to_owned(),
        end_date: "2026-06-03".to_owned(),
        payroll_period: "2026-06".to_owned(),
        payroll_impact_kind: PayrollImpactKind::UnpaidLeaveDeduction,
        workflow_ref: "workflow/hr-leave/kr".to_owned(),
        rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        decision_evidence_ref: "audit/hr/leave/leave_001/decision".to_owned(),
        routing_evidence_ref: "audit/hr/leave/leave_001/escalation".to_owned(),
        payroll_impact_evidence_ref: "audit/hr/leave/leave_001/payroll-impact".to_owned(),
        decided_at_epoch_seconds: 1_779_532_800,
    }
}
