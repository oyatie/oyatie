#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_hr_employment_api::{
    LeaveDecisionDto, LeavePayrollImpactRequest, LeavePayrollImpactResponse, LeaveRoutingModeDto,
    PayrollImpactKindDto,
};
use oya_hr_employment_app::plan_leave_payroll_impact_envelope;
use oya_payroll_run_api::{
    HrLeaveImpactIntakeContext, HrLeaveImpactIntakeRequest, HrLeaveImpactIntakeResponse,
    HrLeaveImpactSourceEnvelope, HrLeaveImpactSourceEnvelopeError,
};
use oya_payroll_run_app::prepare_hr_leave_impact_intake;

#[test]
fn hr_leave_impact_source_envelope_converts_to_payroll_intake_without_calculation() {
    let source = hr_leave_source_envelope();

    let request =
        HrLeaveImpactIntakeRequest::from_hr_leave_impact_source(source, payroll_intake_context())
            .expect("payroll accepts HR-owned leave-impact metadata source");
    let body = serde_json::to_value(&request).expect("serialize payroll intake request");

    assert_eq!(body["runId"], "prun_kr_2026_06");
    assert_eq!(body["payeeId"], "payee_001");
    assert_eq!(body["sourceTopic"], "integration.hr.payroll.leave-impact");
    assert_eq!(
        body["sourceHrIdempotencyKey"],
        "ten_acme:leave_001:Approved:2026-06"
    );
    assert_eq!(body["impactKind"], "PAID_LEAVE");
    assert_eq!(
        body["decisionEvidenceRef"],
        "audit/hr/leave/leave_001/decision"
    );
    assert_eq!(
        body["routingEvidenceRef"],
        "audit/hr/leave/leave_001/delegation"
    );
    assert_eq!(
        body["payrollImpactEvidenceRef"],
        "audit/hr/leave/leave_001/payroll-impact"
    );
    assert_eq!(
        body["payrollIntakeEvidenceRef"],
        "audit/payroll/hr-leave/leave_001/intake"
    );
    assert_eq!(body["rulepackRef"], "rulepack/kr-labor-2026");
    assert_eq!(body["rulepackEffectiveDate"], "2026-01-01");
    assert!(body.get("grossPay").is_none());
    assert!(body.get("calculatedPay").is_none());
    assert!(body.get("payrollCalculationAttached").is_none());

    let outcome = prepare_hr_leave_impact_intake(request.into_domain())
        .expect("app accepts HR source-derived payroll intake");
    assert_eq!(
        outcome.intake.idempotency_key.value,
        "prun_kr_2026_06:payee_001:leave_001:2026-06:PaidLeave:ten_acme:leave_001:Approved:2026-06"
    );

    let response = HrLeaveImpactIntakeResponse::from_intake(&outcome.intake);
    let response_body = serde_json::to_value(response).expect("serialize payroll intake response");
    assert_eq!(
        response_body["integrationTopic"],
        "integration.payroll.hr.leave-impact-intake"
    );
    assert_eq!(response_body["payloadDataClass"], "FINANCIAL");
    assert_eq!(response_body["schemaVersion"], 1);
}

#[test]
fn payroll_rejects_hr_source_envelope_that_claims_runtime_work() {
    let overclaim_cases: [(&str, fn(&mut HrLeaveImpactSourceEnvelope)); 5] = [
        ("payrollCalculationAttached", |source| {
            source.payroll_calculation_attached = true;
        }),
        ("payrollNetworkCall", |source| {
            source.payroll_network_call = true;
        }),
        ("workflowExecution", |source| {
            source.workflow_execution = true;
        }),
        ("storageAttached", |source| {
            source.storage_attached = true;
        }),
        ("runtimeAuditEmission", |source| {
            source.runtime_audit_emission = true;
        }),
    ];

    for (field, mark_overclaim) in overclaim_cases {
        let mut source = hr_leave_source_envelope();
        mark_overclaim(&mut source);

        assert_eq!(
            HrLeaveImpactIntakeRequest::from_hr_leave_impact_source(
                source,
                payroll_intake_context(),
            ),
            Err(HrLeaveImpactSourceEnvelopeError::SourceOverclaimsRuntimeWork),
            "{field} overclaim should be rejected"
        );
    }
}

fn payroll_intake_context() -> HrLeaveImpactIntakeContext {
    HrLeaveImpactIntakeContext {
        run_id: "prun_kr_2026_06".to_owned(),
        payee_id: "payee_001".to_owned(),
        payroll_intake_evidence_ref: "audit/payroll/hr-leave/leave_001/intake".to_owned(),
        received_at_epoch_seconds: 1_779_535_200,
    }
}

fn hr_leave_source_envelope() -> HrLeaveImpactSourceEnvelope {
    let outcome = plan_leave_payroll_impact_envelope(hr_leave_request().into_domain_input())
        .expect("HR app prepares leave-impact envelope");
    let hr_response = LeavePayrollImpactResponse::from_outcome(&outcome);
    let hr_json = serde_json::to_value(hr_response).expect("serialize HR source response");

    serde_json::from_value(hr_json).expect("payroll source envelope accepts HR response JSON")
}

fn hr_leave_request() -> LeavePayrollImpactRequest {
    LeavePayrollImpactRequest {
        leave_request_id: "leave_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        employee_id: "emp_001".to_owned(),
        approver_id: "emp_mgr_001".to_owned(),
        decision: LeaveDecisionDto::Approved,
        routing_mode: LeaveRoutingModeDto::DelegatedApprover,
        start_date: "2026-06-01".to_owned(),
        end_date: "2026-06-03".to_owned(),
        payroll_period: "2026-06".to_owned(),
        payroll_impact_kind: PayrollImpactKindDto::PaidLeave,
        workflow_ref: "workflow/hr-leave/kr".to_owned(),
        rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        decision_evidence_ref: "audit/hr/leave/leave_001/decision".to_owned(),
        routing_evidence_ref: "audit/hr/leave/leave_001/delegation".to_owned(),
        payroll_impact_evidence_ref: "audit/hr/leave/leave_001/payroll-impact".to_owned(),
        decided_at_epoch_seconds: 1_779_534_600,
    }
}
