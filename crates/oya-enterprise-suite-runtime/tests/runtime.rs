#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use oya_enterprise_suite_api::{
    ChildWriteAdmissionRequest, CloseBoundaryStateDto, CrossProductWorkflowPlanRequest,
    DataClassDto, DeterministicGateDto, DeterministicGateEvidenceRequest,
    EnterpriseChildProductDto, EnterpriseOpsCommandRequest, EnterpriseOpsRouteDto,
    GateClosureAuthorityDto, GroupCloseRollupRequest, IncidentFirstActionDto,
    IncidentRemediationRouteDto, IncidentRollbackPlanRequest, IncidentTriggerDto, JurisdictionDto,
    LegalEntityCloseSnapshotRequest, ObjectGraphRelationshipOwnerDto, OpsCommandKindDto,
    SuiteWriteKindDto, WorkflowRoutingOwnerDto,
};
use oya_enterprise_suite_runtime::{
    ENTERPRISE_SUITE_CROSS_PRODUCT_WORKFLOW_PLANS_PATH, ENTERPRISE_SUITE_GROUP_CLOSE_ROLLUPS_PATH,
    ENTERPRISE_SUITE_HEALTH_PATH, ENTERPRISE_SUITE_INCIDENT_ROLLBACK_PLANS_PATH,
    ENTERPRISE_SUITE_OPS_COMMANDS_PATH, ENTERPRISE_SUITE_POLICY_ADMISSIONS_PATH,
    dispatch_enterprise_suite_request, enterprise_suite_runtime_routes,
    enterprise_suite_server_config,
};
use oya_http_middleware_kernel::HttpRequest;
use oya_http_router_kernel::HttpMethod;

#[test]
fn enterprise_suite_runtime_dispatches_policy_group_workflow_incident_and_ops() {
    let policy = dispatch_enterprise_suite_request(mock_json_request(
        HttpMethod::Post,
        ENTERPRISE_SUITE_POLICY_ADMISSIONS_PATH,
        &child_write_request(),
    ));
    let policy_body: serde_json::Value = serde_json::from_slice(&policy.body).expect("policy json");
    assert_eq!(policy.status, 202);
    assert_eq!(policy_body["accepted"], true);
    assert_eq!(
        policy_body["topic"],
        "policy.enterprise-suite.child-write.admission"
    );
    assert_eq!(policy_body["service"], "enterprise-suite");

    let group = dispatch_enterprise_suite_request(mock_json_request(
        HttpMethod::Post,
        ENTERPRISE_SUITE_GROUP_CLOSE_ROLLUPS_PATH,
        &group_rollup_request(),
    ));
    let group_body: serde_json::Value = serde_json::from_slice(&group.body).expect("group json");
    assert_eq!(group.status, 200);
    assert_eq!(
        group_body["topic"],
        "projection.enterprise-suite.group-close.rollup"
    );
    assert_eq!(
        group_body["idempotencyKey"],
        "ten_acme:grp_acme_kr:group-close-rollup"
    );

    let workflow = dispatch_enterprise_suite_request(mock_json_request(
        HttpMethod::Post,
        ENTERPRISE_SUITE_CROSS_PRODUCT_WORKFLOW_PLANS_PATH,
        &workflow_request(),
    ));
    let workflow_body: serde_json::Value =
        serde_json::from_slice(&workflow.body).expect("workflow json");
    assert_eq!(workflow.status, 200);
    assert_eq!(
        workflow_body["topic"],
        "workflow.enterprise-suite.cross-product.dispatch"
    );

    let incident = dispatch_enterprise_suite_request(mock_json_request(
        HttpMethod::Post,
        ENTERPRISE_SUITE_INCIDENT_ROLLBACK_PLANS_PATH,
        &incident_request(),
    ));
    let incident_body: serde_json::Value =
        serde_json::from_slice(&incident.body).expect("incident json");
    assert_eq!(incident.status, 202);
    assert_eq!(
        incident_body["topic"],
        "incident.enterprise-suite.rollback.plan"
    );

    let ops = dispatch_enterprise_suite_request(mock_json_request(
        HttpMethod::Post,
        ENTERPRISE_SUITE_OPS_COMMANDS_PATH,
        &ops_request(),
    ));
    let ops_body: serde_json::Value = serde_json::from_slice(&ops.body).expect("ops json");
    assert_eq!(ops.status, 202);
    assert_eq!(ops_body["topic"], "audit.enterprise-suite.ops.command");
}

#[test]
fn enterprise_suite_runtime_rejects_invalid_json_and_gate_bypass_errors() {
    let invalid_json = HttpRequest {
        method: HttpMethod::Post,
        path: ENTERPRISE_SUITE_POLICY_ADMISSIONS_PATH.to_owned(),
        headers: BTreeMap::new(),
        body: b"{not-json".to_vec(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    };
    let invalid_response = dispatch_enterprise_suite_request(invalid_json);
    let invalid_body: serde_json::Value =
        serde_json::from_slice(&invalid_response.body).expect("invalid json response");
    assert_eq!(invalid_response.status, 400);
    assert_eq!(invalid_body["error"]["code"], "VALIDATION_ERROR");

    let ai_gate = dispatch_enterprise_suite_request(mock_json_request(
        HttpMethod::Post,
        ENTERPRISE_SUITE_CROSS_PRODUCT_WORKFLOW_PLANS_PATH,
        &CrossProductWorkflowPlanRequest {
            gate_closure_authority: GateClosureAuthorityDto::AiSuggestion,
            ..workflow_request()
        },
    ));
    let ai_gate_body: serde_json::Value =
        serde_json::from_slice(&ai_gate.body).expect("ai gate error json");
    assert_eq!(ai_gate.status, 400);
    assert!(
        ai_gate_body["error"]["details"]
            .as_str()
            .unwrap()
            .contains("AiCannotCloseDeterministicGate")
    );

    let manual_ssh = dispatch_enterprise_suite_request(mock_json_request(
        HttpMethod::Post,
        ENTERPRISE_SUITE_OPS_COMMANDS_PATH,
        &EnterpriseOpsCommandRequest {
            route: EnterpriseOpsRouteDto::ManualSsh,
            ..ops_request()
        },
    ));
    let manual_ssh_body: serde_json::Value =
        serde_json::from_slice(&manual_ssh.body).expect("manual ssh error json");
    assert_eq!(manual_ssh.status, 400);
    assert!(
        manual_ssh_body["error"]["details"]
            .as_str()
            .unwrap()
            .contains("ManualSshRefused")
    );
}

#[test]
fn enterprise_suite_runtime_manifest_and_health_preserve_honest_non_claims() {
    let routes = enterprise_suite_runtime_routes();
    assert_eq!(routes.len(), 6);
    assert!(
        routes
            .iter()
            .any(|route| route.path == ENTERPRISE_SUITE_OPS_COMMANDS_PATH)
    );

    let config = enterprise_suite_server_config();
    assert_eq!(config.max_body_bytes, 64 * 1024);

    let health = dispatch_enterprise_suite_request(HttpRequest {
        method: HttpMethod::Get,
        path: ENTERPRISE_SUITE_HEALTH_PATH.to_owned(),
        headers: BTreeMap::new(),
        body: Vec::new(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    });
    let body: serde_json::Value = serde_json::from_slice(&health.body).expect("health json");
    assert_eq!(health.status, 200);
    assert_eq!(body["runtimeAdapter"], "router-ready");
    assert_eq!(body["deployedListener"], false);
    assert_eq!(body["authEnforcementRuntime"], false);
    assert_eq!(body["storageAttached"], false);
    assert_eq!(body["workflowExecution"], false);
    assert_eq!(body["openTofuExecution"], false);
    assert_eq!(body["incidentRollbackExecution"], false);
    assert_eq!(body["childServiceCalls"], false);
    assert_eq!(body["runtimeAuditChainEmission"], false);
    assert_eq!(body["cloudServiceIntegration"], false);
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

fn child_write_request() -> ChildWriteAdmissionRequest {
    ChildWriteAdmissionRequest {
        child_product: EnterpriseChildProductDto::Payroll,
        write_kind: SuiteWriteKindDto::PayrollClose,
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payload_data_class: DataClassDto::Financial,
        audit_evidence_ref: "audit/enterprise-suite/write/payroll-close".to_owned(),
        policy_gateway_ref: "policy/enterprise-suite/shared-gateway".to_owned(),
        idempotency_key: "ten_acme:le_kr_001:payroll-close".to_owned(),
        sequence: 1,
    }
}

fn group_rollup_request() -> GroupCloseRollupRequest {
    GroupCloseRollupRequest {
        tenant_id: "ten_acme".to_owned(),
        group_id: "grp_acme_kr".to_owned(),
        jurisdiction: JurisdictionDto::Korea,
        dashboard_projection_ref: "projection/enterprise-suite/group/kr".to_owned(),
        legal_entities: vec![
            closed_snapshot("le_kr_001", 7, 11),
            closed_snapshot("le_kr_002", 3, 5),
        ],
    }
}

fn closed_snapshot(
    legal_entity_id: &str,
    payroll_close_version: u64,
    accounting_close_version: u64,
) -> LegalEntityCloseSnapshotRequest {
    LegalEntityCloseSnapshotRequest {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: legal_entity_id.to_owned(),
        payroll_close_state: CloseBoundaryStateDto::ProductionClosed,
        accounting_close_state: CloseBoundaryStateDto::ProductionClosed,
        payroll_evidence_ref: format!("audit/payroll/{legal_entity_id}"),
        accounting_evidence_ref: format!("audit/accounting/{legal_entity_id}"),
        payroll_close_version,
        accounting_close_version,
    }
}

fn workflow_request() -> CrossProductWorkflowPlanRequest {
    CrossProductWorkflowPlanRequest {
        tenant_id: "ten_acme".to_owned(),
        workflow_ref: "workflow/enterprise-suite/hr-payroll-accounting".to_owned(),
        object_graph_relationship_ref: "object-graph/enterprise-suite/employee-payroll-journal"
            .to_owned(),
        routing_owner: WorkflowRoutingOwnerDto::Workflow,
        relationship_owner: ObjectGraphRelationshipOwnerDto::ObjectGraph,
        child_products: vec![
            EnterpriseChildProductDto::Hr,
            EnterpriseChildProductDto::Payroll,
            EnterpriseChildProductDto::Accounting,
        ],
        gate_evidence_refs: vec![
            gate(DeterministicGateDto::HumanApproval, "approval"),
            gate(DeterministicGateDto::EvidenceAttached, "evidence"),
            gate(DeterministicGateDto::RollbackPlanAttached, "rollback"),
            gate(
                DeterministicGateDto::LegalEntityBoundaryChecked,
                "entity-boundary",
            ),
        ],
        gate_closure_authority: GateClosureAuthorityDto::DeterministicGateSet,
        ai_suggestion_ref: Some("ai/enterprise-suite/advice/001".to_owned()),
        idempotency_key: "ten_acme:workflow:hr-payroll-accounting".to_owned(),
    }
}

fn gate(gate: DeterministicGateDto, suffix: &str) -> DeterministicGateEvidenceRequest {
    DeterministicGateEvidenceRequest {
        gate,
        evidence_ref: format!("audit/enterprise-suite/workflow/{suffix}"),
    }
}

fn incident_request() -> IncidentRollbackPlanRequest {
    IncidentRollbackPlanRequest {
        tenant_id: "ten_acme".to_owned(),
        incident_id: "inc_canary_slo_001".to_owned(),
        trigger: IncidentTriggerDto::CanarySloBreach,
        first_action: IncidentFirstActionDto::Rollback,
        remediation_route: IncidentRemediationRouteDto::OpenTofu,
        canary_evidence_ref: "audit/enterprise-suite/incidents/canary-slo".to_owned(),
        incident_evidence_ref: "audit/enterprise-suite/incidents/inc_canary_slo_001.json"
            .to_owned(),
        rollback_evidence_ref: "audit/enterprise-suite/incidents/rollback-first".to_owned(),
        convergence_ref: "opentofu/enterprise-suite/fixes/inc_canary_slo_001".to_owned(),
        idempotency_key: "ten_acme:incident:inc_canary_slo_001".to_owned(),
    }
}

fn ops_request() -> EnterpriseOpsCommandRequest {
    EnterpriseOpsCommandRequest {
        tenant_id: "ten_acme".to_owned(),
        route: EnterpriseOpsRouteDto::OyaOps,
        command_kind: OpsCommandKindDto::Day2Change,
        evidence_ref: "audit/enterprise-suite/ops/day2".to_owned(),
        change_plan_ref: "opentofu/enterprise-suite/day2-plan".to_owned(),
        idempotency_key: "ten_acme:day2:plan".to_owned(),
    }
}
