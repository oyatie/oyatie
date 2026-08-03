//! Tenant RBAC HTTP runtime adapter foundation.
//!
//! This crate binds Tenant RBAC API DTOs to the repo-native Hyper
//! router/middleware foundation without introducing a deployed listener. It
//! validates JSON, invokes service domain/app metadata planners, and serializes
//! OpenAPI-aligned responses for policy admission, group close rollup,
//! cross-service Workflow planning, incident rollback planning, and ops command
//! metadata. It does not persist service records, execute Workflow, call downstream
//! services, run OpenTofu, perform incident rollback, emit runtime audit-chain
//! events, or deploy cloud I/O.
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use oya_http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_runtime_hyper_adapter::{
    ServerConfig, SyncHandler, dispatch as dispatch_http, handler_to_sync,
};
use oya_shared_pdp_kernel::{EntityRecord, EntitySlice, PolicyDecisionPoint};
use oya_shared_platform_contracts_kernel::pdp::{AuthorizationRequest, Decision, EntityRef};
use oya_tenant_rbac_api::{
    ApiErrorBody, ApiErrorEnvelope, CrossServiceWorkflowPlanRequest, GroupCloseRollupRequest,
    IncidentRollbackPlanRequest, SensitiveHrReadScopeDecisionRequest,
    SensitiveHrReadScopeDecisionResponse, ServiceWriteAdmissionRequest,
    TenantRbacOpsCommandRequest,
};
use oya_tenant_rbac_domain::{
    TenantRbacDomainError, admit_sensitive_hr_read_scope, admit_service_write,
    plan_cross_service_workflow, plan_incident_rollback, roll_up_group_close_status,
};
use oya_tenant_rbac_usecase::{
    TenantRbacApplicationError, prepare_cross_service_workflow_envelope,
    prepare_incident_rollback_envelope, prepare_tenant_rbac_ops_envelope,
};
use serde::Serialize;

pub const TENANT_RBAC_POLICY_ADMISSIONS_PATH: &str = "/tenant-rbac/v1/policy-admissions";
pub const TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH: &str = "/tenant-rbac/v1/group-close-rollups";
pub const TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH: &str =
    "/tenant-rbac/v1/cross-service-workflow-plans";
pub const TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH: &str =
    "/tenant-rbac/v1/incident-rollback-plans";
pub const TENANT_RBAC_OPS_COMMANDS_PATH: &str = "/tenant-rbac/v1/ops-commands";
pub const TENANT_RBAC_HR_SENSITIVE_READ_SCOPE_DECISIONS_PATH: &str =
    "/tenant-rbac/v1/hr-sensitive-read-scope-decisions";
pub const TENANT_RBAC_HEALTH_PATH: &str = "/tenant-rbac/v1/healthz";

const POLICY_ADMISSION_TOPIC: &str = "policy.tenant-rbac.service-write.admission";
const GROUP_CLOSE_ROLLUP_TOPIC: &str = "projection.tenant-rbac.group-close.rollup";
const JSON_CONTENT_TYPE: &str = "application/json";
const SERVICE_NAME: &str = "tenant-rbac";
const MAX_TENANT_RBAC_BODY_BYTES: usize = 64 * 1024;
const PRINCIPAL_SUBJECT_HEADER: &str = "x-oya-principal-subject-id";
const CALLER_TENANT_HEADER: &str = "x-oya-caller-tenant-id";
const REQUEST_ID_HEADER: &str = "x-oya-request-id";

#[derive(Clone)]
pub struct DecisionAuthorizer {
    pdp: Arc<dyn PolicyDecisionPoint>,
}

impl DecisionAuthorizer {
    pub fn new<P>(pdp: Arc<P>) -> Self
    where
        P: PolicyDecisionPoint + 'static,
    {
        Self { pdp }
    }

    fn authorize_mutation(
        &self,
        http_request: &HttpRequest,
        projection: MutationAuthorizationProjection<'_>,
    ) -> Result<(), TenantRbacAuthorizationError> {
        let principal_subject_id = required_header(http_request, PRINCIPAL_SUBJECT_HEADER)?;
        let caller_tenant = required_header(http_request, CALLER_TENANT_HEADER)?;
        let request_id = http_request
            .headers
            .get(REQUEST_ID_HEADER)
            .filter(|value| !value.is_empty())
            .cloned()
            .unwrap_or_else(|| projection.request_id_fallback.to_owned());

        let principal = EntityRef {
            entity_type: "OyaPlatform::Principal".to_owned(),
            entity_id: principal_subject_id.to_owned(),
        };
        let resource = EntityRef {
            entity_type: "OyaPlatform::TenantRbacMutation".to_owned(),
            entity_id: projection.target_subject_id.to_owned(),
        };
        let authz_request = AuthorizationRequest {
            request_id,
            tenant_id: projection.target_tenant.to_owned(),
            principal: principal.clone(),
            action: projection.action.to_owned(),
            resource: resource.clone(),
            context: BTreeMap::from([
                (
                    "caller_tenant".to_owned(),
                    serde_json::Value::String(caller_tenant.to_owned()),
                ),
                (
                    "target_tenant".to_owned(),
                    serde_json::Value::String(projection.target_tenant.to_owned()),
                ),
                (
                    "target_subject_id".to_owned(),
                    serde_json::Value::String(projection.target_subject_id.to_owned()),
                ),
                (
                    "method".to_owned(),
                    serde_json::Value::String(projection.method.to_owned()),
                ),
                (
                    "path".to_owned(),
                    serde_json::Value::String(projection.path.to_owned()),
                ),
            ]),
            min_policy_version: None,
        };
        authz_request
            .validate()
            .map_err(TenantRbacAuthorizationError::InvalidRequestProjection)?;
        let entities = EntitySlice {
            entities: vec![
                EntityRecord {
                    uid: principal,
                    attributes: BTreeMap::from([
                        (
                            "caller_tenant".to_owned(),
                            serde_json::Value::String(caller_tenant.to_owned()),
                        ),
                        (
                            "subject_id".to_owned(),
                            serde_json::Value::String(principal_subject_id.to_owned()),
                        ),
                    ]),
                    parents: vec![tenant_entity_ref(caller_tenant)],
                },
                EntityRecord {
                    uid: resource,
                    attributes: BTreeMap::from([
                        (
                            "target_tenant".to_owned(),
                            serde_json::Value::String(projection.target_tenant.to_owned()),
                        ),
                        (
                            "target_subject_id".to_owned(),
                            serde_json::Value::String(projection.target_subject_id.to_owned()),
                        ),
                    ]),
                    parents: vec![tenant_entity_ref(projection.target_tenant)],
                },
            ],
        };
        entities
            .validate()
            .map_err(TenantRbacAuthorizationError::InvalidEntityProjection)?;

        let outcome = self
            .pdp
            .authorize(&authz_request, &entities)
            .map_err(|error| TenantRbacAuthorizationError::Pdp(error.to_string()))?;
        outcome
            .response
            .validate()
            .map_err(TenantRbacAuthorizationError::InvalidPdpResponse)?;
        if outcome.response.decision == Decision::Allow {
            Ok(())
        } else {
            Err(TenantRbacAuthorizationError::Denied)
        }
    }
}

struct MutationAuthorizationProjection<'a> {
    method: &'static str,
    path: &'static str,
    action: &'static str,
    target_tenant: &'a str,
    target_subject_id: &'a str,
    request_id_fallback: &'a str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum TenantRbacAuthorizationError {
    MissingHeader(&'static str),
    InvalidRequestProjection(Vec<oya_shared_platform_contracts_kernel::ContractViolation>),
    InvalidEntityProjection(Vec<oya_shared_platform_contracts_kernel::ContractViolation>),
    InvalidPdpResponse(Vec<oya_shared_platform_contracts_kernel::ContractViolation>),
    Pdp(String),
    Denied,
}

impl std::fmt::Display for TenantRbacAuthorizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHeader(header) => {
                write!(f, "missing required authorization header {header}")
            }
            Self::InvalidRequestProjection(violations) => {
                write!(f, "invalid PDP request projection: {violations:?}")
            }
            Self::InvalidEntityProjection(violations) => {
                write!(f, "invalid PDP entity projection: {violations:?}")
            }
            Self::InvalidPdpResponse(violations) => {
                write!(f, "invalid PDP response: {violations:?}")
            }
            Self::Pdp(error) => write!(f, "PDP refused Tenant RBAC mutation decision: {error}"),
            Self::Denied => write!(f, "PDP denied Tenant RBAC mutation"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacRuntimeRoute {
    pub method: &'static str,              // data_class: INTERNAL_ONLY
    pub path: &'static str,                // data_class: INTERNAL_ONLY
    pub operation_id: &'static str,        // data_class: INTERNAL_ONLY
    pub request_data_class: &'static str,  // data_class: INTERNAL_ONLY
    pub response_data_class: &'static str, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacRuntimeError {
    Router(RouterError),
}

impl From<RouterError> for TenantRbacRuntimeError {
    fn from(error: RouterError) -> Self {
        Self::Router(error)
    }
}

impl std::fmt::Display for TenantRbacRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantRbacRuntimeError::Router(error) => {
                write!(f, "tenant-rbac router error: {error:?}")
            }
        }
    }
}

impl std::error::Error for TenantRbacRuntimeError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantRbacAcceptedResponse {
    pub accepted: bool,          // data_class: INTERNAL_ONLY
    pub topic: String,           // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub schema_version: u32,     // data_class: PUBLIC
    pub service: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantRbacHealthResponse {
    pub status: String,                     // data_class: PUBLIC
    pub service: String,                    // data_class: PUBLIC
    pub runtime_adapter: String,            // data_class: PUBLIC
    pub deployed_listener: bool,            // data_class: PUBLIC
    pub auth_enforcement_runtime: bool,     // data_class: PUBLIC
    pub storage_attached: bool,             // data_class: PUBLIC
    pub workflow_execution: bool,           // data_class: PUBLIC
    pub open_tofu_execution: bool,          // data_class: PUBLIC
    pub incident_rollback_execution: bool,  // data_class: PUBLIC
    pub downstream_service_calls: bool,     // data_class: PUBLIC
    pub runtime_audit_chain_emission: bool, // data_class: PUBLIC
    pub cloud_service_integration: bool,    // data_class: PUBLIC
    pub schema_version: u32,                // data_class: PUBLIC
}

pub fn tenant_rbac_runtime_routes() -> Vec<TenantRbacRuntimeRoute> {
    vec![
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_POLICY_ADMISSIONS_PATH,
            operation_id: "admitTenantRbacServiceWrite",
            request_data_class: "INTERNAL_ONLY+FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH,
            operation_id: "rollUpTenantRbacGroupClose",
            request_data_class: "INTERNAL_ONLY+FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH,
            operation_id: "planTenantRbacCrossServiceWorkflow",
            request_data_class: "INTERNAL_ONLY",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH,
            operation_id: "planTenantRbacIncidentRollback",
            request_data_class: "INTERNAL_ONLY",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_OPS_COMMANDS_PATH,
            operation_id: "prepareTenantRbacOpsCommand",
            request_data_class: "INTERNAL_ONLY",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_HR_SENSITIVE_READ_SCOPE_DECISIONS_PATH,
            operation_id: "admitTenantRbacHrSensitiveReadScopeDecision",
            request_data_class: "INTERNAL_ONLY+AUDIT",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "GET",
            path: TENANT_RBAC_HEALTH_PATH,
            operation_id: "tenantRbacRuntimeHealth",
            request_data_class: "PUBLIC",
            response_data_class: "PUBLIC",
        },
    ]
}

pub fn tenant_rbac_server_config() -> ServerConfig {
    ServerConfig::default()
        .with_max_body_bytes(MAX_TENANT_RBAC_BODY_BYTES)
        .with_header_read_timeout(Duration::from_secs(10))
        .with_keepalive_timeout(Duration::from_secs(30))
}

pub fn tenant_rbac_runtime_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new()
}

pub fn tenant_rbac_runtime_router() -> Result<Router<SyncHandler>, TenantRbacRuntimeError> {
    tenant_rbac_runtime_router_with_optional_authorizer(None)
}

pub fn tenant_rbac_runtime_router_with_authorizer(
    authorizer: Arc<DecisionAuthorizer>,
) -> Result<Router<SyncHandler>, TenantRbacRuntimeError> {
    tenant_rbac_runtime_router_with_optional_authorizer(Some(authorizer))
}

fn tenant_rbac_runtime_router_with_optional_authorizer(
    authorizer: Option<Arc<DecisionAuthorizer>>,
) -> Result<Router<SyncHandler>, TenantRbacRuntimeError> {
    let mut router = Router::new();
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_POLICY_ADMISSIONS_PATH,
        handler_to_sync(PolicyAdmissionHandler {
            authorizer: authorizer.clone(),
        }),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH,
        handler_to_sync(GroupCloseRollupHandler {
            authorizer: authorizer.clone(),
        }),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH,
        handler_to_sync(CrossServiceWorkflowHandler {
            authorizer: authorizer.clone(),
        }),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH,
        handler_to_sync(IncidentRollbackHandler {
            authorizer: authorizer.clone(),
        }),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_OPS_COMMANDS_PATH,
        handler_to_sync(OpsCommandHandler {
            authorizer: authorizer.clone(),
        }),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_HR_SENSITIVE_READ_SCOPE_DECISIONS_PATH,
        handler_to_sync(HrSensitiveReadScopeDecisionHandler {
            authorizer: authorizer.clone(),
        }),
    )?;
    router.route(
        HttpMethod::Get,
        TENANT_RBAC_HEALTH_PATH,
        handler_to_sync(HealthHandler {
            auth_enforcement_runtime: authorizer.is_some(),
        }),
    )?;
    Ok(router)
}

pub fn dispatch_tenant_rbac_request(request: HttpRequest) -> HttpResponse {
    dispatch_tenant_rbac_request_with_optional_authorizer(request, None)
}

pub fn dispatch_tenant_rbac_request_with_authorizer(
    request: HttpRequest,
    authorizer: Arc<DecisionAuthorizer>,
) -> HttpResponse {
    dispatch_tenant_rbac_request_with_optional_authorizer(request, Some(authorizer))
}

fn dispatch_tenant_rbac_request_with_optional_authorizer(
    request: HttpRequest,
    authorizer: Option<Arc<DecisionAuthorizer>>,
) -> HttpResponse {
    match tenant_rbac_runtime_router_with_optional_authorizer(authorizer) {
        Ok(router) => dispatch_http(request, &router, &tenant_rbac_runtime_chain()),
        Err(error) => json_response(
            500,
            &ApiErrorEnvelope::validation(
                "Tenant RBAC runtime router failed",
                Some(error.to_string()),
            ),
        ),
    }
}

#[derive(Clone)]
struct PolicyAdmissionHandler {
    authorizer: Option<Arc<DecisionAuthorizer>>,
}

#[derive(Clone)]
struct GroupCloseRollupHandler {
    authorizer: Option<Arc<DecisionAuthorizer>>,
}

#[derive(Clone)]
struct CrossServiceWorkflowHandler {
    authorizer: Option<Arc<DecisionAuthorizer>>,
}

#[derive(Clone)]
struct IncidentRollbackHandler {
    authorizer: Option<Arc<DecisionAuthorizer>>,
}

#[derive(Clone)]
struct OpsCommandHandler {
    authorizer: Option<Arc<DecisionAuthorizer>>,
}

#[derive(Clone)]
struct HrSensitiveReadScopeDecisionHandler {
    authorizer: Option<Arc<DecisionAuthorizer>>,
}

#[derive(Clone)]
struct HealthHandler {
    auth_enforcement_runtime: bool,
}

impl oya_http_middleware_kernel::Handler for PolicyAdmissionHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: ServiceWriteAdmissionRequest = parse_json(&req.body)?;
        require_authorized(
            self.authorizer.as_deref(),
            &req,
            MutationAuthorizationProjection {
                method: "POST",
                path: TENANT_RBAC_POLICY_ADMISSIONS_PATH,
                action: "tenant-rbac.policy-admissions.write",
                target_tenant: &request.tenant_id,
                target_subject_id: &request.legal_entity_id,
                request_id_fallback: &request.idempotency_key,
            },
        )?;
        let decision = admit_service_write(request.into_domain()).map_err(domain_error_response)?;
        Ok(json_response(
            202,
            &TenantRbacAcceptedResponse {
                accepted: true,
                topic: POLICY_ADMISSION_TOPIC.to_owned(),
                idempotency_key: decision.idempotency_key.value.clone(),
                schema_version: decision.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for GroupCloseRollupHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: GroupCloseRollupRequest = parse_json(&req.body)?;
        let fallback_request_id = format!(
            "{}:{}:group-close-rollup",
            request.tenant_id, request.group_id
        );
        require_authorized(
            self.authorizer.as_deref(),
            &req,
            MutationAuthorizationProjection {
                method: "POST",
                path: TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH,
                action: "tenant-rbac.group-close-rollups.write",
                target_tenant: &request.tenant_id,
                target_subject_id: &request.group_id,
                request_id_fallback: &fallback_request_id,
            },
        )?;
        let rollup =
            roll_up_group_close_status(request.into_domain()).map_err(domain_error_response)?;
        Ok(json_response(
            200,
            &TenantRbacAcceptedResponse {
                accepted: true,
                topic: GROUP_CLOSE_ROLLUP_TOPIC.to_owned(),
                idempotency_key: format!(
                    "{}:{}:group-close-rollup",
                    rollup.tenant_id.value.value, rollup.group_id.value.value
                ),
                schema_version: rollup.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for CrossServiceWorkflowHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: CrossServiceWorkflowPlanRequest = parse_json(&req.body)?;
        require_authorized(
            self.authorizer.as_deref(),
            &req,
            MutationAuthorizationProjection {
                method: "POST",
                path: TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH,
                action: "tenant-rbac.cross-service-workflow-plans.write",
                target_tenant: &request.tenant_id,
                target_subject_id: &request.workflow_ref,
                request_id_fallback: &request.idempotency_key,
            },
        )?;
        let plan =
            plan_cross_service_workflow(request.into_domain()).map_err(domain_error_response)?;
        let envelope = prepare_cross_service_workflow_envelope(&plan);
        Ok(json_response(
            200,
            &TenantRbacAcceptedResponse {
                accepted: true,
                topic: envelope.topic.value.clone(),
                idempotency_key: envelope.idempotency_key.value.clone(),
                schema_version: envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for IncidentRollbackHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: IncidentRollbackPlanRequest = parse_json(&req.body)?;
        require_authorized(
            self.authorizer.as_deref(),
            &req,
            MutationAuthorizationProjection {
                method: "POST",
                path: TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH,
                action: "tenant-rbac.incident-rollback-plans.write",
                target_tenant: &request.tenant_id,
                target_subject_id: &request.incident_id,
                request_id_fallback: &request.idempotency_key,
            },
        )?;
        let plan = plan_incident_rollback(request.into_domain()).map_err(domain_error_response)?;
        let envelope = prepare_incident_rollback_envelope(&plan);
        Ok(json_response(
            202,
            &TenantRbacAcceptedResponse {
                accepted: true,
                topic: envelope.topic.value.clone(),
                idempotency_key: envelope.idempotency_key.value.clone(),
                schema_version: envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for OpsCommandHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: TenantRbacOpsCommandRequest = parse_json(&req.body)?;
        require_authorized(
            self.authorizer.as_deref(),
            &req,
            MutationAuthorizationProjection {
                method: "POST",
                path: TENANT_RBAC_OPS_COMMANDS_PATH,
                action: "tenant-rbac.ops-commands.write",
                target_tenant: &request.tenant_id,
                target_subject_id: &request.change_plan_ref,
                request_id_fallback: &request.idempotency_key,
            },
        )?;
        let envelope =
            prepare_tenant_rbac_ops_envelope(request.into_app()).map_err(app_error_response)?;
        Ok(json_response(
            202,
            &TenantRbacAcceptedResponse {
                accepted: true,
                topic: envelope.topic.value.clone(),
                idempotency_key: envelope.idempotency_key.value.clone(),
                schema_version: envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for HrSensitiveReadScopeDecisionHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: SensitiveHrReadScopeDecisionRequest = parse_json(&req.body)?;
        require_authorized(
            self.authorizer.as_deref(),
            &req,
            MutationAuthorizationProjection {
                method: "POST",
                path: TENANT_RBAC_HR_SENSITIVE_READ_SCOPE_DECISIONS_PATH,
                action: "tenant-rbac.hr-sensitive-read-scope-decisions.write",
                target_tenant: &request.tenant_id,
                target_subject_id: &request.entitlement_set_id,
                request_id_fallback: &request.idempotency_key,
            },
        )?;
        let decision =
            admit_sensitive_hr_read_scope(request.into_domain()).map_err(domain_error_response)?;
        Ok(json_response(
            200,
            &SensitiveHrReadScopeDecisionResponse::from_decision(&decision),
        ))
    }
}

impl oya_http_middleware_kernel::Handler for HealthHandler {
    type Error = HttpResponse;

    fn call(&self, _req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        Ok(json_response(
            200,
            &TenantRbacHealthResponse {
                status: "ok".to_owned(),
                service: SERVICE_NAME.to_owned(),
                runtime_adapter: "router-ready".to_owned(),
                deployed_listener: false,
                auth_enforcement_runtime: self.auth_enforcement_runtime,
                storage_attached: false,
                workflow_execution: false,
                open_tofu_execution: false,
                incident_rollback_execution: false,
                downstream_service_calls: false,
                runtime_audit_chain_emission: false,
                cloud_service_integration: false,
                schema_version: 1,
            },
        ))
    }
}

fn parse_json<T>(body: &[u8]) -> Result<T, HttpResponse>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_slice(body).map_err(|error| {
        json_response(
            400,
            &ApiErrorEnvelope::validation(
                "Invalid Tenant RBAC JSON request",
                Some(error.to_string()),
            ),
        )
    })
}

fn require_authorized(
    authorizer: Option<&DecisionAuthorizer>,
    http_request: &HttpRequest,
    projection: MutationAuthorizationProjection<'_>,
) -> Result<(), HttpResponse> {
    let Some(authorizer) = authorizer else {
        return Err(authorization_denied_response(
            "Tenant RBAC mutation requires a DecisionAuthorizer-backed PDP",
        ));
    };
    authorizer
        .authorize_mutation(http_request, projection)
        .map_err(|error| authorization_denied_response(error.to_string()))
}

fn required_header<'a>(
    request: &'a HttpRequest,
    header: &'static str,
) -> Result<&'a str, TenantRbacAuthorizationError> {
    request
        .headers
        .get(header)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or(TenantRbacAuthorizationError::MissingHeader(header))
}

fn tenant_entity_ref(tenant_id: &str) -> EntityRef {
    EntityRef {
        entity_type: "OyaPlatform::Tenant".to_owned(),
        entity_id: tenant_id.to_owned(),
    }
}

fn domain_error_response(error: TenantRbacDomainError) -> HttpResponse {
    json_response(
        400,
        &ApiErrorEnvelope::validation("Invalid Tenant RBAC command", Some(format!("{error:?}"))),
    )
}

fn app_error_response(error: TenantRbacApplicationError) -> HttpResponse {
    json_response(
        400,
        &ApiErrorEnvelope::validation("Invalid Tenant RBAC command", Some(format!("{error:?}"))),
    )
}

fn authorization_denied_response(details: impl Into<String>) -> HttpResponse {
    json_response(
        403,
        &ApiErrorEnvelope {
            error: ApiErrorBody {
                code: "AUTHORIZATION_DENIED".to_owned(),
                message: "Tenant RBAC authorization denied".to_owned(),
                details: Some(details.into()),
            },
        },
    )
}

fn json_response<T>(status: u16, body: &T) -> HttpResponse
where
    T: Serialize,
{
    match serde_json::to_vec(body) {
        Ok(bytes) => HttpResponse::new(status)
            .with_header("content-type", JSON_CONTENT_TYPE)
            .with_body(bytes),
        Err(error) => HttpResponse::new(500)
            .with_header("content-type", "text/plain; charset=utf-8")
            .with_body(format!("json serialization failed: {error}").into_bytes()),
    }
}
