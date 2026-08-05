#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use oya_hr_employment_api::{
    EmploymentStatusDto, HrLifecycleKindDto, JurisdictionDto, LaborCompliancePlanRequest,
    LeaveDecisionDto, LeavePayrollImpactRequest, LeaveRoutingModeDto, OnboardEmployeeRequest,
    PayrollImpactKindDto, SensitiveHrDataKindDto, SensitiveHrReadPolicyRequest,
    SensitiveReadLegalBasisDto, SensitiveReadPurposeDto, TenantTierSnapshotDto,
};
use oya_hr_employment_infrastructure::{
    HR_EMPLOYEES_PATH, HR_HEALTH_PATH, HR_LABOR_COMPLIANCE_WORKFLOW_PLANS_PATH,
    HR_LEAVE_PAYROLL_IMPACT_PLANS_PATH, HR_SENSITIVE_READ_POLICY_DECISIONS_PATH,
    dispatch_hr_request, hr_runtime_routes, hr_server_config,
    serve_hr_runtime_n_connections_on_std_listener,
};
use oya_http_middleware_kernel::HttpRequest;
use oya_http_router_kernel::HttpMethod;

#[test]
fn hr_runtime_dispatches_sensitive_read_and_leave() {
    let sensitive = dispatch_hr_request(mock_json_request(
        HttpMethod::Post,
        HR_SENSITIVE_READ_POLICY_DECISIONS_PATH,
        &sensitive_read_request(),
    ));
    let sensitive_body: serde_json::Value =
        serde_json::from_slice(&sensitive.body).expect("sensitive read json");
    assert_eq!(sensitive.status, 200);
    assert_eq!(sensitive_body["decisionStatus"], "ALLOWED");
    assert_eq!(
        sensitive_body["auditTopic"],
        "audit.hr.sensitive-read.policy"
    );
    assert_eq!(sensitive_body["payloadDataClass"], "PHI");

    let leave = dispatch_hr_request(mock_json_request(
        HttpMethod::Post,
        HR_LEAVE_PAYROLL_IMPACT_PLANS_PATH,
        &leave_payroll_impact_request(),
    ));
    let leave_body: serde_json::Value = serde_json::from_slice(&leave.body).expect("leave json");
    assert_eq!(leave.status, 200);
    assert_eq!(
        leave_body["integrationTopic"],
        "integration.hr.payroll.leave-impact"
    );
    assert_eq!(leave_body["payrollPeriod"], "2026-06");
    assert_eq!(leave_body["payloadDataClass"], "FINANCIAL");
}

#[test]
fn hr_runtime_dispatches_onboarding_and_labor_workflow_metadata() {
    let onboard = dispatch_hr_request(mock_json_request(
        HttpMethod::Post,
        HR_EMPLOYEES_PATH,
        &onboard_request(),
    ));
    let onboard_body: serde_json::Value =
        serde_json::from_slice(&onboard.body).expect("onboard json");
    assert_eq!(onboard.status, 202);
    assert_eq!(onboard_body["accepted"], true);
    assert_eq!(onboard_body["auditTopic"], "audit.hr.employment.lifecycle");
    assert_eq!(onboard_body["service"], "hr");

    let labor = dispatch_hr_request(mock_json_request(
        HttpMethod::Post,
        HR_LABOR_COMPLIANCE_WORKFLOW_PLANS_PATH,
        &labor_request(),
    ));
    let labor_body: serde_json::Value = serde_json::from_slice(&labor.body).expect("labor json");
    assert_eq!(labor.status, 200);
    assert_eq!(
        labor_body["workflowDispatches"].as_array().unwrap().len(),
        2
    );
    assert_eq!(labor_body["schemaVersion"], 1);
}

#[test]
fn hr_runtime_rejects_invalid_json_and_forbidden_sensitive_purpose() {
    let invalid_json = HttpRequest {
        method: HttpMethod::Post,
        path: HR_SENSITIVE_READ_POLICY_DECISIONS_PATH.to_owned(),
        headers: BTreeMap::new(),
        body: b"{not-json".to_vec(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    };
    let invalid_response = dispatch_hr_request(invalid_json);
    let invalid_body: serde_json::Value =
        serde_json::from_slice(&invalid_response.body).expect("invalid json response");
    assert_eq!(invalid_response.status, 400);
    assert_eq!(invalid_body["error"]["code"], "VALIDATION_ERROR");

    let forbidden = dispatch_hr_request(mock_json_request(
        HttpMethod::Post,
        HR_SENSITIVE_READ_POLICY_DECISIONS_PATH,
        &SensitiveHrReadPolicyRequest {
            purpose: SensitiveReadPurposeDto::GeneralBrowsing,
            ..sensitive_read_request()
        },
    ));
    let forbidden_body: serde_json::Value =
        serde_json::from_slice(&forbidden.body).expect("forbidden json");
    assert_eq!(forbidden.status, 403);
    assert!(
        forbidden_body["error"]["details"]
            .as_str()
            .unwrap()
            .contains("DisallowedSensitiveReadPurpose")
    );
}

#[test]
fn hr_runtime_sensitive_read_fails_closed_without_tenant_rbac_scope() {
    let mut request = sensitive_read_request();
    request.tenant_rbac_scope_evidence_ref = None;

    let response = dispatch_hr_request(mock_json_request(
        HttpMethod::Post,
        HR_SENSITIVE_READ_POLICY_DECISIONS_PATH,
        &request,
    ));
    let body_text = String::from_utf8(response.body.clone()).expect("utf8 error body");
    let body: serde_json::Value = serde_json::from_str(&body_text).expect("scope error json");

    assert_eq!(response.status, 403);
    assert!(
        body["error"]["details"]
            .as_str()
            .unwrap()
            .contains("MissingTenantRbacScopeEvidence")
    );
    assert!(!body_text.contains("rawSensitiveData"));
    assert!(!body_text.contains("PHI"));
}

#[test]
fn hr_runtime_sensitive_read_fails_closed_with_invalid_tenant_rbac_scope() {
    let mut request = sensitive_read_request();
    request.tenant_rbac_scope_evidence_ref =
        Some("audit/tenant-rbac/hr-sensitive-read/../escape".to_owned());

    let response = dispatch_hr_request(mock_json_request(
        HttpMethod::Post,
        HR_SENSITIVE_READ_POLICY_DECISIONS_PATH,
        &request,
    ));
    let body_text = String::from_utf8(response.body.clone()).expect("utf8 error body");
    let body: serde_json::Value = serde_json::from_str(&body_text).expect("scope error json");

    assert_eq!(response.status, 403);
    assert!(
        body["error"]["details"]
            .as_str()
            .unwrap()
            .contains("InvalidTenantRbacScopeEvidence")
    );
    assert!(!body_text.contains("rawSensitiveData"));
    assert!(!body_text.contains("PHI"));
}

#[test]
fn hr_runtime_sensitive_read_fails_closed_without_audit_contract() {
    let mut request = sensitive_read_request();
    request.audit_emission_contract_ref = None;

    let response = dispatch_hr_request(mock_json_request(
        HttpMethod::Post,
        HR_SENSITIVE_READ_POLICY_DECISIONS_PATH,
        &request,
    ));
    let body_text = String::from_utf8(response.body.clone()).expect("utf8 error body");
    let body: serde_json::Value = serde_json::from_str(&body_text).expect("audit error json");

    assert_eq!(response.status, 403);
    assert!(
        body["error"]["details"]
            .as_str()
            .unwrap()
            .contains("MissingSensitiveReadAuditContract")
    );
    assert!(!body_text.contains("rawSensitiveData"));
    assert!(!body_text.contains("PHI"));
}

#[test]
fn hr_runtime_manifest_and_health_preserve_honest_non_claims() {
    let routes = hr_runtime_routes();
    assert_eq!(routes.len(), 5);
    assert!(
        routes
            .iter()
            .any(|route| route.path == HR_SENSITIVE_READ_POLICY_DECISIONS_PATH)
    );

    let config = hr_server_config();
    assert_eq!(config.max_body_bytes, 64 * 1024);

    let health = dispatch_hr_request(HttpRequest {
        method: HttpMethod::Get,
        path: HR_HEALTH_PATH.to_owned(),
        headers: BTreeMap::new(),
        body: Vec::new(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    });
    let body: serde_json::Value = serde_json::from_slice(&health.body).expect("health json");
    assert_eq!(health.status, 200);
    assert_eq!(body["runtimeAdapter"], "listener-boundary-ready");
    assert_eq!(body["listenerBoundary"], "prebound-std-tcp-listener");
    assert_eq!(body["deployedListener"], false);
    assert_eq!(body["storageAttached"], false);
    assert_eq!(body["workflowExecution"], false);
    assert_eq!(body["payrollNetworkCall"], false);
    assert_eq!(body["sensitiveDataFetch"], false);
    assert_eq!(body["runtimeAuditEmission"], false);
    assert_eq!(body["cloudDeployment"], false);
}

#[test]
fn hr_runtime_serves_loopback_health_over_listener_boundary_without_deployment_claims() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback listener");
    let addr = listener.local_addr().expect("listener exposes local addr");
    let server = thread::spawn(move || serve_hr_runtime_n_connections_on_std_listener(listener, 1));

    let mut stream = TcpStream::connect(addr).expect("connect to HR listener boundary");
    stream
        .write_all(b"GET /hr/v1/healthz HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .expect("write HR health request");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("read HR health response");

    server
        .join()
        .expect("HR listener boundary thread joins")
        .expect("HR listener boundary serves one connection");

    assert!(response.starts_with("HTTP/1.1 200 OK"));
    let (_headers, body) = response
        .split_once("\r\n\r\n")
        .expect("HTTP response separates headers and body");
    let body: serde_json::Value = serde_json::from_str(body).expect("health JSON body");
    assert_eq!(body["runtimeAdapter"], "listener-boundary-ready");
    assert_eq!(body["listenerBoundary"], "prebound-std-tcp-listener");
    assert_eq!(body["deployedListener"], false);
    assert_eq!(body["storageAttached"], false);
    assert_eq!(body["workflowExecution"], false);
    assert_eq!(body["payrollNetworkCall"], false);
    assert_eq!(body["sensitiveDataFetch"], false);
    assert_eq!(body["runtimeAuditEmission"], false);
    assert_eq!(body["cloudDeployment"], false);
}

fn mock_json_request<T: serde::Serialize>(
    method: HttpMethod,
    path: &str,
    payload: &T,
) -> HttpRequest {
    HttpRequest {
        method,
        path: path.to_owned(),
        headers: BTreeMap::from([("content-type".to_owned(), "application/json".to_owned())]),
        body: serde_json::to_vec(payload).expect("serialize request"),
        path_captures: BTreeMap::new(),
        matched_template: None,
    }
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

fn labor_request() -> LaborCompliancePlanRequest {
    LaborCompliancePlanRequest {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        jurisdiction: JurisdictionDto::Korea,
        active_employee_count: 30,
        rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        workflow_ref: "workflow/hr-compliance/kr".to_owned(),
        evidence_ref: "audit/hr/compliance/kr-threshold".to_owned(),
        evaluated_at_epoch_seconds: 1_779_519_600,
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
        tenant_rbac_scope_evidence_ref: Some(
            "audit/tenant-rbac/hr-sensitive-read/entset_hr_privacy_kr".to_owned(),
        ),
        audit_emission_contract_ref: Some(
            "audit-event-class/HrSensitiveReadPolicyEvaluated".to_owned(),
        ),
        evaluated_at_epoch_seconds: 1_779_534_000,
    }
}
