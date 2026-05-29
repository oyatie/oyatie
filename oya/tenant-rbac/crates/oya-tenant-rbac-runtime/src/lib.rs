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

use std::time::Duration;

use oya_http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_runtime_hyper_adapter::{
    ServerConfig, SyncHandler, dispatch as dispatch_http, handler_to_sync,
};
use oya_tenant_rbac_api::{
    ApiErrorEnvelope, CrossServiceWorkflowPlanRequest, GroupCloseRollupRequest,
    IncidentRollbackPlanRequest, ServiceWriteAdmissionRequest, TenantRbacOpsCommandRequest,
};
use oya_tenant_rbac_application::{
    TenantRbacApplicationError, prepare_cross_service_workflow_envelope,
    prepare_incident_rollback_envelope, prepare_tenant_rbac_ops_envelope,
};
use oya_tenant_rbac_domain::{
    TenantRbacDomainError, admit_service_write, plan_cross_service_workflow,
    plan_incident_rollback, roll_up_group_close_status,
};
use serde::Serialize;

pub const TENANT_RBAC_POLICY_ADMISSIONS_PATH: &str = "/tenant-rbac/v1/policy-admissions";
pub const TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH: &str = "/tenant-rbac/v1/group-close-rollups";
pub const TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH: &str =
    "/tenant-rbac/v1/cross-service-workflow-plans";
pub const TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH: &str =
    "/tenant-rbac/v1/incident-rollback-plans";
pub const TENANT_RBAC_OPS_COMMANDS_PATH: &str = "/tenant-rbac/v1/ops-commands";
pub const TENANT_RBAC_HEALTH_PATH: &str = "/tenant-rbac/v1/healthz";

const POLICY_ADMISSION_TOPIC: &str = "policy.tenant-rbac.service-write.admission";
const GROUP_CLOSE_ROLLUP_TOPIC: &str = "projection.tenant-rbac.group-close.rollup";
const JSON_CONTENT_TYPE: &str = "application/json";
const SERVICE_NAME: &str = "tenant-rbac";
const MAX_TENANT_RBAC_BODY_BYTES: usize = 64 * 1024;

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
    let mut router = Router::new();
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_POLICY_ADMISSIONS_PATH,
        handler_to_sync(PolicyAdmissionHandler),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH,
        handler_to_sync(GroupCloseRollupHandler),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH,
        handler_to_sync(CrossServiceWorkflowHandler),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH,
        handler_to_sync(IncidentRollbackHandler),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_OPS_COMMANDS_PATH,
        handler_to_sync(OpsCommandHandler),
    )?;
    router.route(
        HttpMethod::Get,
        TENANT_RBAC_HEALTH_PATH,
        handler_to_sync(HealthHandler),
    )?;
    Ok(router)
}

pub fn dispatch_tenant_rbac_request(request: HttpRequest) -> HttpResponse {
    match tenant_rbac_runtime_router() {
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

struct PolicyAdmissionHandler;
struct GroupCloseRollupHandler;
struct CrossServiceWorkflowHandler;
struct IncidentRollbackHandler;
struct OpsCommandHandler;
struct HealthHandler;

impl oya_http_middleware_kernel::Handler for PolicyAdmissionHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: ServiceWriteAdmissionRequest = parse_json(&req.body)?;
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
                auth_enforcement_runtime: false,
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
