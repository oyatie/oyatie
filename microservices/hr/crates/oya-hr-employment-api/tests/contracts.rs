#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_hr_employment_api::{
    ApiErrorEnvelope, EmploymentStatusDto, HrLifecycleKindDto, JurisdictionDto,
    LaborCompliancePlanRequest, LeaveDecisionDto, LeavePayrollImpactRequest,
    LeavePayrollImpactResponse, LeaveRoutingModeDto, OnboardEmployeeRequest, PayrollImpactKindDto,
    SensitiveHrDataKindDto, SensitiveHrReadPolicyRequest, SensitiveReadLegalBasisDto,
    SensitiveReadPolicyDecisionResponse, SensitiveReadPurposeDto, TenantTierSnapshotDto,
};
use oya_hr_employment_app::{
    onboard_employee, plan_labor_compliance_workflows, plan_leave_payroll_impact_envelope,
    prepare_sensitive_hr_read_envelope,
};
use serde_json::json;

#[test]
fn onboard_employee_request_uses_camel_case_and_stable_enums() {
    let request = onboard_request();
    let body = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(body["employeeId"], "emp_001");
    assert_eq!(body["employmentStatus"], "ACTIVE");
    assert_eq!(body["tenantTierSnapshot"], "ENTERPRISE_GROUP");
    assert_eq!(body["lifecycleKind"], "CREATED");

    let outcome = onboard_employee(request.into_command()).expect("app accepts DTO command");
    assert_eq!(outcome.employee.employee_id.value.value, "emp_001");
}

#[test]
fn labor_compliance_request_converts_to_domain_snapshot() {
    let request = LaborCompliancePlanRequest {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        jurisdiction: JurisdictionDto::Korea,
        active_employee_count: 30,
        rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        workflow_ref: "workflow/hr-compliance/kr".to_owned(),
        evidence_ref: "audit/hr/compliance/kr-threshold".to_owned(),
        evaluated_at_epoch_seconds: 1_779_519_600,
    };

    let outcome = plan_labor_compliance_workflows(request.into_snapshot()).expect("plan");
    assert_eq!(outcome.workflow_dispatches.len(), 2);
}

#[test]
fn leave_payroll_impact_request_uses_camel_case_and_converts_to_domain_input() {
    let request = leave_payroll_impact_request();
    let body = serde_json::to_value(&request).expect("serialize leave payroll request");

    assert_eq!(body["leaveRequestId"], "leave_001");
    assert_eq!(body["approverId"], "emp_mgr_001");
    assert_eq!(body["decision"], "APPROVED");
    assert_eq!(body["routingMode"], "DELEGATED_APPROVER");
    assert_eq!(body["payrollImpactKind"], "PAID_LEAVE");
    assert_eq!(
        body["payrollImpactEvidenceRef"],
        "audit/hr/leave/leave_001/payroll-impact"
    );

    let outcome = plan_leave_payroll_impact_envelope(request.into_domain_input())
        .expect("app accepts leave payroll-impact DTO");
    let response = LeavePayrollImpactResponse::from_outcome(&outcome);
    let response_body = serde_json::to_value(response).expect("serialize leave payroll response");

    assert_eq!(
        response_body["integrationTopic"],
        "integration.hr.payroll.leave-impact"
    );
    assert_eq!(response_body["payrollPeriod"], "2026-06");
    assert_eq!(response_body["payrollImpactKind"], "PAID_LEAVE");
    assert_eq!(response_body["payloadDataClass"], "FINANCIAL");
    assert_eq!(response_body["schemaVersion"], 1);
}

#[test]
fn sensitive_read_policy_request_uses_camel_case_and_converts_to_domain_input() {
    let request = sensitive_read_request();
    let body = serde_json::to_value(&request).expect("serialize sensitive read request");

    assert_eq!(body["actorEmployeeId"], "emp_admin_001");
    assert_eq!(body["subjectEmployeeId"], "emp_001");
    assert_eq!(body["dataKind"], "MEDICAL");
    assert_eq!(body["purpose"], "BENEFITS_ADMINISTRATION");
    assert_eq!(body["legalBasis"], "CONSENT");
    assert_eq!(
        body["consentEvidenceRef"],
        "audit/hr/privacy/emp_001/consent"
    );

    let outcome = prepare_sensitive_hr_read_envelope(request.into_domain_input())
        .expect("app accepts sensitive read policy DTO");
    let response = SensitiveReadPolicyDecisionResponse::from_outcome(&outcome);
    let response_body = serde_json::to_value(response).expect("serialize sensitive read response");

    assert_eq!(response_body["decisionStatus"], "ALLOWED");
    assert_eq!(
        response_body["auditTopic"],
        "audit.hr.sensitive-read.policy"
    );
    assert_eq!(response_body["payloadDataClass"], "PHI");
    assert_eq!(response_body["schemaVersion"], 1);
}

#[test]
fn error_envelope_has_consistent_shape() {
    let envelope =
        ApiErrorEnvelope::validation("Invalid employment request", Some("employeeId".to_owned()));

    assert_eq!(
        serde_json::to_value(envelope).expect("serialize error"),
        json!({
            "error": {
                "code": "VALIDATION_ERROR",
                "message": "Invalid employment request",
                "details": "employeeId"
            }
        })
    );
}

fn onboard_request() -> OnboardEmployeeRequest {
    OnboardEmployeeRequest {
        employee_id: "emp_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        person_ref: "person/acme/001".to_owned(),
        manager_id: Some("emp_mgr_001".to_owned()),
        employment_status: EmploymentStatusDto::Active,
        tenant_tier_snapshot: TenantTierSnapshotDto::EnterpriseGroup,
        audit_evidence_ref: "audit/hr/employee/001".to_owned(),
        version: 1,
        event_id: "hrev_employee_created_001".to_owned(),
        lifecycle_kind: HrLifecycleKindDto::Created,
    }
}

fn leave_payroll_impact_request() -> LeavePayrollImpactRequest {
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

fn sensitive_read_request() -> SensitiveHrReadPolicyRequest {
    SensitiveHrReadPolicyRequest {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        actor_employee_id: "emp_admin_001".to_owned(),
        subject_employee_id: "emp_001".to_owned(),
        data_kind: SensitiveHrDataKindDto::Medical,
        purpose: SensitiveReadPurposeDto::BenefitsAdministration,
        legal_basis: SensitiveReadLegalBasisDto::Consent,
        policy_ref: "policy/hr/sensitive-read/benefits-2026".to_owned(),
        basis_evidence_ref: "audit/hr/privacy/emp_001/basis".to_owned(),
        consent_evidence_ref: Some("audit/hr/privacy/emp_001/consent".to_owned()),
        request_evidence_ref: "audit/hr/privacy/emp_001/request".to_owned(),
        read_log_evidence_ref: "audit/hr/privacy/emp_001/read-log".to_owned(),
        evaluated_at_epoch_seconds: 1_779_534_000,
    }
}
