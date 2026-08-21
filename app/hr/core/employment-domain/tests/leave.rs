#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use hr_employment_domain::{
    HrDomainError, LeaveDecision, LeavePayrollImpactInput, LeaveRoutingMode, PayrollImpactKind,
    plan_leave_payroll_impact,
};
use data_boundary_kernel::DataClass;

#[test]
fn test_leave_approval_emits_payroll_impact() {
    let plan = plan_leave_payroll_impact(valid_input()).expect("leave payroll impact plan");

    assert_eq!(plan.tenant_id.value.value, "ten_acme");
    assert_eq!(plan.legal_entity_id.value.value, "le_kr_001");
    assert_eq!(plan.employee_id.value.value, "emp_001");
    assert_eq!(plan.approver_id.value.value, "emp_mgr_001");
    assert_eq!(plan.decision.value, LeaveDecision::Approved);
    assert_eq!(plan.routing_mode.value, LeaveRoutingMode::DelegatedApprover);
    assert_eq!(plan.start_date.value, "2026-06-01");
    assert_eq!(plan.end_date.value, "2026-06-03");
    assert_eq!(plan.payroll_period.value, "2026-06");
    assert_eq!(plan.payroll_impact_kind.value, PayrollImpactKind::PaidLeave);
    assert_eq!(plan.workflow_ref.value.value, "workflow/hr-leave/kr");
    assert_eq!(plan.rulepack_ref.value.value, "rulepack/kr-labor-2026");
    assert_eq!(
        plan.routing_evidence_ref.value.value,
        "audit/hr/leave/leave_001/delegation"
    );
    assert_eq!(
        plan.payroll_impact_evidence_ref.value.value,
        "audit/hr/leave/leave_001/payroll-impact"
    );
    assert_eq!(
        plan.payroll_impact_evidence_ref
            .data_class
            .compatibility_data_class(),
        DataClass::Financial
    );
    assert_eq!(
        plan.idempotency_key.value,
        "ten_acme:leave_001:Approved:2026-06"
    );
    assert_eq!(plan.schema_version.value, 1);
}

#[test]
fn test_leave_payroll_impact_requires_rulepack_basis() {
    let error = plan_leave_payroll_impact(LeavePayrollImpactInput {
        rulepack_ref: "policy/kr-labor-2026".to_owned(),
        ..valid_input()
    })
    .expect_err("rulepack basis is required");

    assert_eq!(error, HrDomainError::InvalidRulepackRef);
}

#[test]
fn test_leave_payroll_impact_requires_routing_evidence() {
    let error = plan_leave_payroll_impact(LeavePayrollImpactInput {
        routing_evidence_ref: "audit/".to_owned(),
        ..valid_input()
    })
    .expect_err("delegation/escalation evidence is required");

    assert_eq!(error, HrDomainError::InvalidAuditEvidenceRef);
}

#[test]
fn test_leave_payroll_impact_requires_payroll_evidence() {
    let error = plan_leave_payroll_impact(LeavePayrollImpactInput {
        payroll_impact_evidence_ref: "audit/hr/leave/bearer-token".to_owned(),
        ..valid_input()
    })
    .expect_err("payroll impact evidence cannot look like credentials");

    assert_eq!(error, HrDomainError::InvalidAuditEvidenceRef);
}

#[test]
fn test_leave_payroll_impact_rejects_unsafe_dates_and_periods() {
    let date_error = plan_leave_payroll_impact(LeavePayrollImpactInput {
        start_date: "2026-06-03".to_owned(),
        end_date: "2026-06-01".to_owned(),
        ..valid_input()
    })
    .expect_err("leave start must not be after end");
    assert_eq!(date_error, HrDomainError::InvalidLeaveDate);

    let period_error = plan_leave_payroll_impact(LeavePayrollImpactInput {
        payroll_period: "2026-99".to_owned(),
        ..valid_input()
    })
    .expect_err("payroll period must be YYYY-MM with real month");
    assert_eq!(period_error, HrDomainError::InvalidPayrollPeriod);
}

fn valid_input() -> LeavePayrollImpactInput {
    LeavePayrollImpactInput {
        leave_request_id: "leave_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        employee_id: "emp_001".to_owned(),
        approver_id: "emp_mgr_001".to_owned(),
        decision: LeaveDecision::Approved,
        routing_mode: LeaveRoutingMode::DelegatedApprover,
        start_date: "2026-06-01".to_owned(),
        end_date: "2026-06-03".to_owned(),
        payroll_period: "2026-06".to_owned(),
        payroll_impact_kind: PayrollImpactKind::PaidLeave,
        workflow_ref: "workflow/hr-leave/kr".to_owned(),
        rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        decision_evidence_ref: "audit/hr/leave/leave_001/decision".to_owned(),
        routing_evidence_ref: "audit/hr/leave/leave_001/delegation".to_owned(),
        payroll_impact_evidence_ref: "audit/hr/leave/leave_001/payroll-impact".to_owned(),
        decided_at_epoch_seconds: 1_779_532_800,
    }
}
