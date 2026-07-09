#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use iam_tenant_rbac_api::{
    CloseBoundaryStateDto, CrossServiceWorkflowPlanRequest, DataClassDto, DeterministicGateDto,
    DeterministicGateEvidenceRequest, GateClosureAuthorityDto, GroupCloseRollupRequest,
    IncidentFirstActionDto, IncidentRemediationRouteDto, IncidentRollbackPlanRequest,
    IncidentTriggerDto, JurisdictionDto, LegalEntityCloseSnapshotRequest,
    ObjectGraphRelationshipOwnerDto, OpsCommandKindDto, ServiceWriteAdmissionRequest,
    TenantRbacOpsCommandRequest, TenantRbacOpsRouteDto, TenantRbacServiceDto,
    TenantRbacWriteKindDto, WorkflowRoutingOwnerDto,
};
use iam_tenant_rbac_app::{
    CallerCredential, ConfiguredBearerPrincipalVerifier, DecisionAuthorizer, PrincipalVerifier,
    TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH, TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH,
    TENANT_RBAC_HEALTH_PATH, TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH,
    TENANT_RBAC_OPS_COMMANDS_PATH, TENANT_RBAC_POLICY_ADMISSIONS_PATH, TenantRbacAuthzProvider,
    TenantRbacMutationAction, TenantRbacMutationAuthorizationError, TenantRbacMutationAuthorizer,
    TenantRbacMutationResource, VerifiedPrincipal, dispatch_tenant_rbac_request,
    tenant_rbac_runtime_routes, tenant_rbac_server_config,
};
use oya_http_middleware_kernel::HttpRequest;
use oya_http_router_kernel::HttpMethod;
use oya_shared_pdp_kernel::{
    DecisionAuditRecord, EntitySlice, PdpError, PdpOutcome, PolicyDecisionPoint,
};
use oya_shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, PolicyVersion,
};

const BEARER_SECRET: &str = "tenant-rbac-break-glass";
const BOUND_TENANT: &str = "ten_acme";

struct AllowAll;
impl TenantRbacMutationAuthorizer for AllowAll {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &TenantRbacMutationResource,
    ) -> Result<(), TenantRbacMutationAuthorizationError> {
        Ok(())
    }
}

struct DenyAll;
impl TenantRbacMutationAuthorizer for DenyAll {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &TenantRbacMutationResource,
    ) -> Result<(), TenantRbacMutationAuthorizationError> {
        Err(TenantRbacMutationAuthorizationError::Denied)
    }
}

struct AssertPolicyFacts;
impl TenantRbacMutationAuthorizer for AssertPolicyFacts {
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &TenantRbacMutationResource,
    ) -> Result<(), TenantRbacMutationAuthorizationError> {
        assert_eq!(principal.principal_id(), "sp_tenant_rbac");
        assert_eq!(principal.tenant_id(), BOUND_TENANT);
        assert_eq!(resource.tenant_id, BOUND_TENANT);
        assert_eq!(resource.action, TenantRbacMutationAction::PolicyAdmission);
        assert_eq!(resource.route_template, TENANT_RBAC_POLICY_ADMISSIONS_PATH);
        Ok(())
    }
}

#[derive(Debug)]
struct RecordingPdp {
    mode: RecordingPdpMode,
    requests: Mutex<Vec<(AuthorizationRequest, EntitySlice)>>,
}

#[derive(Clone, Copy, Debug)]
enum RecordingPdpMode {
    Decision(Decision),
    Error,
}

impl RecordingPdp {
    fn allow() -> Self {
        Self {
            mode: RecordingPdpMode::Decision(Decision::Allow),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn deny() -> Self {
        Self {
            mode: RecordingPdpMode::Decision(Decision::Deny),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn error() -> Self {
        Self {
            mode: RecordingPdpMode::Error,
            requests: Mutex::new(Vec::new()),
        }
    }

    fn captured(&self) -> Vec<(AuthorizationRequest, EntitySlice)> {
        self.requests.lock().unwrap().clone()
    }
}

impl PolicyDecisionPoint for RecordingPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        self.requests
            .lock()
            .unwrap()
            .push((request.clone(), entities.clone()));
        match self.mode {
            RecordingPdpMode::Error => Err(PdpError::UnknownAction {
                action: request.action.clone(),
            }),
            RecordingPdpMode::Decision(decision) => {
                let policy_version = self.loaded_policy_version();
                let determining_policy_ids = match decision {
                    Decision::Allow => vec!["tenant-rbac-route-permit".to_owned()],
                    Decision::Deny => Vec::new(),
                };
                Ok(PdpOutcome {
                    response: AuthorizationResponse {
                        decision_id: "dec-tenant-rbac-app-001".to_owned(),
                        request_id: request.request_id.clone(),
                        decision,
                        policy_version: policy_version.clone(),
                        determining_policy_ids: determining_policy_ids.clone(),
                        obligations: Vec::new(),
                    },
                    audit: DecisionAuditRecord {
                        decision_id: "dec-tenant-rbac-app-001".to_owned(),
                        request_id: request.request_id.clone(),
                        tenant_id: request.tenant_id.clone(),
                        principal: request.principal.clone(),
                        action: request.action.clone(),
                        resource: request.resource.clone(),
                        decision,
                        policy_version,
                        determining_policy_ids,
                        cache_hit: false,
                    },
                    cache_hit: false,
                })
            }
        }
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-tenant-rbac-app-001").unwrap()
    }
}

fn test_provider() -> TenantRbacAuthzProvider {
    test_provider_with_authorizer(AllowAll)
}

fn test_provider_with_authorizer(
    authorizer: impl TenantRbacMutationAuthorizer + 'static,
) -> TenantRbacAuthzProvider {
    let verifier =
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, "sp_tenant_rbac", BOUND_TENANT)
            .expect("verifier construction failed");
    TenantRbacAuthzProvider::new(Arc::new(verifier), Arc::new(authorizer))
}

#[test]
fn tenant_rbac_runtime_authorizes_mutation_through_shared_pdp_decision_authorizer() {
    let pdp = Arc::new(RecordingPdp::allow());
    let response = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_POLICY_ADMISSIONS_PATH,
            &service_write_request(),
        ),
        test_provider_with_authorizer(DecisionAuthorizer::new(pdp.clone())),
    );

    assert_eq!(response.status, 202);
    let captured = pdp.captured();
    assert_eq!(captured.len(), 1);
    let (authz_request, entities) = &captured[0];
    assert_eq!(authz_request.tenant_id, BOUND_TENANT);
    assert_eq!(
        authz_request.action,
        TenantRbacMutationAction::PolicyAdmission.surface()
    );
    assert_eq!(authz_request.principal.entity_id, "sp_tenant_rbac");
    assert_eq!(
        authz_request.resource.entity_type,
        "OyaPlatform::TenantRbacMutation"
    );
    assert_eq!(
        authz_request.context["route_template"],
        TENANT_RBAC_POLICY_ADMISSIONS_PATH
    );
    assert!(
        entities
            .entities
            .iter()
            .any(|entity| entity.uid.entity_id == "sp_tenant_rbac")
    );
}

#[test]
fn tenant_rbac_runtime_decision_authorizer_fails_closed_on_pdp_deny_and_error() {
    let denied = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_POLICY_ADMISSIONS_PATH,
            &service_write_request(),
        ),
        test_provider_with_authorizer(DecisionAuthorizer::new(Arc::new(RecordingPdp::deny()))),
    );
    assert_eq!(denied.status, 403);

    let errored = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_POLICY_ADMISSIONS_PATH,
            &service_write_request(),
        ),
        test_provider_with_authorizer(DecisionAuthorizer::new(Arc::new(RecordingPdp::error()))),
    );
    assert_eq!(errored.status, 403);
}

#[test]
fn decision_authorizer_refuses_tenant_mismatch_before_pdp() {
    let verifier =
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, "sp_tenant_rbac", BOUND_TENANT)
            .expect("verifier construction failed");
    let principal = verifier
        .verify_principal(&CallerCredential {
            authorization: Some(format!("Bearer {BEARER_SECRET}")),
        })
        .expect("principal verifies");
    let pdp = Arc::new(RecordingPdp::allow());
    let authorizer = DecisionAuthorizer::new(pdp.clone());
    let resource = TenantRbacMutationResource {
        tenant_id: "ten_other".to_owned(),
        action: TenantRbacMutationAction::PolicyAdmission,
        route_template: TENANT_RBAC_POLICY_ADMISSIONS_PATH.to_owned(),
    };

    assert_eq!(
        authorizer.ensure_authorized(&principal, &resource),
        Err(TenantRbacMutationAuthorizationError::Refused)
    );
    assert!(pdp.captured().is_empty());
}

#[test]
fn tenant_rbac_runtime_dispatches_policy_group_workflow_incident_and_ops() {
    let policy = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_POLICY_ADMISSIONS_PATH,
            &service_write_request(),
        ),
        test_provider(),
    );
    let policy_body: serde_json::Value = serde_json::from_slice(&policy.body).expect("policy json");
    assert_eq!(policy.status, 202);
    assert_eq!(policy_body["accepted"], true);
    assert_eq!(
        policy_body["topic"],
        "policy.tenant-rbac.service-write.admission"
    );
    assert_eq!(policy_body["service"], "tenant-rbac");

    let group = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH,
            &group_rollup_request(),
        ),
        test_provider(),
    );
    let group_body: serde_json::Value = serde_json::from_slice(&group.body).expect("group json");
    assert_eq!(group.status, 200);
    assert_eq!(
        group_body["topic"],
        "projection.tenant-rbac.group-close.rollup"
    );
    assert_eq!(
        group_body["idempotencyKey"],
        "ten_acme:grp_acme_kr:group-close-rollup"
    );

    let workflow = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH,
            &workflow_request(),
        ),
        test_provider(),
    );
    let workflow_body: serde_json::Value =
        serde_json::from_slice(&workflow.body).expect("workflow json");
    assert_eq!(workflow.status, 200);
    assert_eq!(
        workflow_body["topic"],
        "workflow.tenant-rbac.cross-service.dispatch"
    );

    let incident = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH,
            &incident_request(),
        ),
        test_provider(),
    );
    let incident_body: serde_json::Value =
        serde_json::from_slice(&incident.body).expect("incident json");
    assert_eq!(incident.status, 202);
    assert_eq!(incident_body["topic"], "incident.tenant-rbac.rollback.plan");

    let ops = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_OPS_COMMANDS_PATH,
            &ops_request(),
        ),
        test_provider(),
    );
    let ops_body: serde_json::Value = serde_json::from_slice(&ops.body).expect("ops json");
    assert_eq!(ops.status, 202);
    assert_eq!(ops_body["topic"], "audit.tenant-rbac.ops.command");
}

#[test]
fn tenant_rbac_runtime_rejects_invalid_json_and_gate_bypass_errors() {
    // Invalid JSON: bearer present so authz passes, handler returns 400.
    let mut headers = BTreeMap::new();
    headers.insert(
        "authorization".to_owned(),
        format!("Bearer {BEARER_SECRET}"),
    );
    let invalid_json = HttpRequest {
        method: HttpMethod::Post,
        path: TENANT_RBAC_POLICY_ADMISSIONS_PATH.to_owned(),
        headers,
        body: b"{not-json".to_vec(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    };
    let invalid_response = dispatch_tenant_rbac_request(invalid_json, test_provider());
    let invalid_body: serde_json::Value =
        serde_json::from_slice(&invalid_response.body).expect("invalid json response");
    assert_eq!(invalid_response.status, 400);
    assert_eq!(invalid_body["error"]["code"], "VALIDATION_ERROR");

    let ai_gate = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH,
            &CrossServiceWorkflowPlanRequest {
                gate_closure_authority: GateClosureAuthorityDto::AiSuggestion,
                ..workflow_request()
            },
        ),
        test_provider(),
    );
    let ai_gate_body: serde_json::Value =
        serde_json::from_slice(&ai_gate.body).expect("ai gate error json");
    assert_eq!(ai_gate.status, 400);
    assert!(
        ai_gate_body["error"]["details"]
            .as_str()
            .unwrap()
            .contains("AiCannotCloseDeterministicGate")
    );

    let manual_ssh = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_OPS_COMMANDS_PATH,
            &TenantRbacOpsCommandRequest {
                route: TenantRbacOpsRouteDto::ManualSsh,
                ..ops_request()
            },
        ),
        test_provider(),
    );
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
fn tenant_rbac_runtime_fails_closed_on_authn_authz_and_tenant_mismatch() {
    let allowed_response = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_POLICY_ADMISSIONS_PATH,
            &service_write_request(),
        ),
        test_provider_with_authorizer(AssertPolicyFacts),
    );
    assert_eq!(allowed_response.status, 202);
    let mut missing_bearer = mock_json_request(
        HttpMethod::Post,
        TENANT_RBAC_POLICY_ADMISSIONS_PATH,
        &service_write_request(),
    );
    missing_bearer.headers.remove("authorization");
    let missing_response = dispatch_tenant_rbac_request(missing_bearer, test_provider());
    assert_eq!(missing_response.status, 401);

    let mut wrong_bearer = mock_json_request(
        HttpMethod::Post,
        TENANT_RBAC_POLICY_ADMISSIONS_PATH,
        &service_write_request(),
    );
    wrong_bearer
        .headers
        .insert("authorization".to_owned(), "Bearer wrong".to_owned());
    let wrong_response = dispatch_tenant_rbac_request(wrong_bearer, test_provider());
    assert_eq!(wrong_response.status, 401);

    let denied_response = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_POLICY_ADMISSIONS_PATH,
            &service_write_request(),
        ),
        test_provider_with_authorizer(DenyAll),
    );
    assert_eq!(denied_response.status, 403);

    let tenant_mismatch_response = dispatch_tenant_rbac_request(
        mock_json_request(
            HttpMethod::Post,
            TENANT_RBAC_POLICY_ADMISSIONS_PATH,
            &ServiceWriteAdmissionRequest {
                tenant_id: "ten_other".to_owned(),
                ..service_write_request()
            },
        ),
        test_provider(),
    );
    assert_eq!(tenant_mismatch_response.status, 403);
}

#[test]
fn tenant_rbac_runtime_manifest_and_health_preserve_honest_non_claims() {
    let routes = tenant_rbac_runtime_routes();
    assert_eq!(routes.len(), 6);
    assert!(
        routes
            .iter()
            .any(|route| route.path == TENANT_RBAC_OPS_COMMANDS_PATH)
    );

    let config = tenant_rbac_server_config();
    assert_eq!(config.max_body_bytes, 64 * 1024);

    // Health is a non-mutation route: middleware passes it through without a
    // bearer token. The response now self-documents auth_enforcement_runtime: true.
    let health = dispatch_tenant_rbac_request(
        HttpRequest {
            method: HttpMethod::Get,
            path: TENANT_RBAC_HEALTH_PATH.to_owned(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: None,
        },
        test_provider(),
    );
    let body: serde_json::Value = serde_json::from_slice(&health.body).expect("health json");
    assert_eq!(health.status, 200);
    assert_eq!(body["runtimeAdapter"], "router-ready");
    assert_eq!(body["deployedListener"], false);
    assert_eq!(body["authEnforcementRuntime"], true);
    assert_eq!(body["storageAttached"], false);
    assert_eq!(body["workflowExecution"], false);
    assert_eq!(body["openTofuExecution"], false);
    assert_eq!(body["incidentRollbackExecution"], false);
    assert_eq!(body["downstreamServiceCalls"], false);
    assert_eq!(body["runtimeAuditChainEmission"], false);
    assert_eq!(body["cloudServiceIntegration"], false);
}

fn mock_json_request<T: serde::Serialize>(
    method: HttpMethod,
    path: &str,
    payload: &T,
) -> HttpRequest {
    // Mutation routes require a verified bearer; the test provider is
    // configured with BEARER_SECRET. Health (GET) passes through unauthenticated
    // but the header is harmless on non-mutation routes.
    let mut headers = BTreeMap::from([
        ("content-type".to_owned(), "application/json".to_owned()),
        (
            "authorization".to_owned(),
            format!("Bearer {BEARER_SECRET}"),
        ),
    ]);
    // For GET requests (health) the authorization header is not required by
    // the middleware, but including it is benign.
    if method == HttpMethod::Get {
        headers.remove("authorization");
    }
    HttpRequest {
        method,
        path: path.to_owned(),
        headers,
        body: serde_json::to_vec(payload).expect("serialize request"),
        path_captures: BTreeMap::new(),
        matched_template: None,
    }
}

fn service_write_request() -> ServiceWriteAdmissionRequest {
    ServiceWriteAdmissionRequest {
        service: TenantRbacServiceDto::Payroll,
        write_kind: TenantRbacWriteKindDto::PayrollClose,
        tenant_id: BOUND_TENANT.to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payload_data_class: DataClassDto::Financial,
        audit_evidence_ref: "audit/tenant-rbac/write/payroll-close".to_owned(),
        policy_gateway_ref: "policy/tenant-rbac/shared-gateway".to_owned(),
        idempotency_key: "ten_acme:le_kr_001:payroll-close".to_owned(),
        sequence: 1,
    }
}

fn group_rollup_request() -> GroupCloseRollupRequest {
    GroupCloseRollupRequest {
        tenant_id: BOUND_TENANT.to_owned(),
        group_id: "grp_acme_kr".to_owned(),
        jurisdiction: JurisdictionDto::Korea,
        dashboard_projection_ref: "projection/tenant-rbac/group/kr".to_owned(),
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
        tenant_id: BOUND_TENANT.to_owned(),
        legal_entity_id: legal_entity_id.to_owned(),
        payroll_close_state: CloseBoundaryStateDto::ProductionClosed,
        accounting_close_state: CloseBoundaryStateDto::ProductionClosed,
        payroll_evidence_ref: format!("audit/payroll/{legal_entity_id}"),
        accounting_evidence_ref: format!("audit/accounting/{legal_entity_id}"),
        payroll_close_version,
        accounting_close_version,
    }
}

fn workflow_request() -> CrossServiceWorkflowPlanRequest {
    CrossServiceWorkflowPlanRequest {
        tenant_id: BOUND_TENANT.to_owned(),
        workflow_ref: "workflow/tenant-rbac/hr-payroll-accounting".to_owned(),
        object_graph_relationship_ref: "object-graph/tenant-rbac/employee-payroll-journal"
            .to_owned(),
        routing_owner: WorkflowRoutingOwnerDto::Workflow,
        relationship_owner: ObjectGraphRelationshipOwnerDto::ObjectGraph,
        services: vec![
            TenantRbacServiceDto::Hr,
            TenantRbacServiceDto::Payroll,
            TenantRbacServiceDto::Accounting,
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
        ai_suggestion_ref: Some("ai/tenant-rbac/advice/001".to_owned()),
        idempotency_key: "ten_acme:workflow:hr-payroll-accounting".to_owned(),
    }
}

fn gate(gate: DeterministicGateDto, suffix: &str) -> DeterministicGateEvidenceRequest {
    DeterministicGateEvidenceRequest {
        gate,
        evidence_ref: format!("audit/tenant-rbac/workflow/{suffix}"),
    }
}

fn incident_request() -> IncidentRollbackPlanRequest {
    IncidentRollbackPlanRequest {
        tenant_id: BOUND_TENANT.to_owned(),
        incident_id: "inc_canary_slo_001".to_owned(),
        trigger: IncidentTriggerDto::CanarySloBreach,
        first_action: IncidentFirstActionDto::Rollback,
        remediation_route: IncidentRemediationRouteDto::OpenTofu,
        canary_evidence_ref: "audit/tenant-rbac/incidents/canary-slo".to_owned(),
        incident_evidence_ref: "audit/tenant-rbac/incidents/inc_canary_slo_001.json".to_owned(),
        rollback_evidence_ref: "audit/tenant-rbac/incidents/rollback-first".to_owned(),
        convergence_ref: "opentofu/tenant-rbac/fixes/inc_canary_slo_001".to_owned(),
        idempotency_key: "ten_acme:incident:inc_canary_slo_001".to_owned(),
    }
}

fn ops_request() -> TenantRbacOpsCommandRequest {
    TenantRbacOpsCommandRequest {
        tenant_id: BOUND_TENANT.to_owned(),
        route: TenantRbacOpsRouteDto::OyaOps,
        command_kind: OpsCommandKindDto::Day2Change,
        evidence_ref: "audit/tenant-rbac/ops/day2".to_owned(),
        change_plan_ref: "opentofu/tenant-rbac/day2-plan".to_owned(),
        idempotency_key: "ten_acme:day2:plan".to_owned(),
    }
}
