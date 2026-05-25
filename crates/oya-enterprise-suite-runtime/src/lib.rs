//! Enterprise Suite HTTP runtime adapter foundation.
//!
//! This crate binds Enterprise Suite API DTOs to the repo-native Hyper
//! router/middleware foundation without introducing a deployed listener. It
//! validates JSON, invokes suite domain/app metadata planners, and serializes
//! OpenAPI-aligned responses for policy admission, group close rollup,
//! cross-product Workflow planning, incident rollback planning, and ops command
//! metadata. It does not persist suite records, execute Workflow, call child
//! services, run OpenTofu, perform incident rollback, emit runtime audit-chain
//! events, or deploy cloud I/O.
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::time::Duration;

use oya_enterprise_suite_api::{
    ApiErrorEnvelope, ChildWriteAdmissionRequest, CrossProductWorkflowPlanRequest,
    EnterpriseOpsCommandRequest, GroupCloseRollupRequest, IncidentRollbackPlanRequest,
};
use oya_enterprise_suite_app::{
    EnterpriseSuiteAppError, prepare_cross_product_workflow_envelope,
    prepare_enterprise_ops_envelope, prepare_incident_rollback_envelope,
};
use oya_enterprise_suite_domain::{
    EnterpriseSuiteDomainError, admit_child_write, plan_cross_product_workflow,
    plan_incident_rollback, roll_up_group_close_status,
};
use oya_http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_runtime_hyper_adapter::{
    ServerConfig, SyncHandler, dispatch as dispatch_http, handler_to_sync,
};
use serde::Serialize;

pub const ENTERPRISE_SUITE_POLICY_ADMISSIONS_PATH: &str = "/enterprise-suite/v1/policy-admissions";
pub const ENTERPRISE_SUITE_GROUP_CLOSE_ROLLUPS_PATH: &str =
    "/enterprise-suite/v1/group-close-rollups";
pub const ENTERPRISE_SUITE_CROSS_PRODUCT_WORKFLOW_PLANS_PATH: &str =
    "/enterprise-suite/v1/cross-product-workflow-plans";
pub const ENTERPRISE_SUITE_INCIDENT_ROLLBACK_PLANS_PATH: &str =
    "/enterprise-suite/v1/incident-rollback-plans";
pub const ENTERPRISE_SUITE_OPS_COMMANDS_PATH: &str = "/enterprise-suite/v1/ops-commands";
pub const ENTERPRISE_SUITE_HEALTH_PATH: &str = "/enterprise-suite/v1/healthz";

const POLICY_ADMISSION_TOPIC: &str = "policy.enterprise-suite.child-write.admission";
const GROUP_CLOSE_ROLLUP_TOPIC: &str = "projection.enterprise-suite.group-close.rollup";
const JSON_CONTENT_TYPE: &str = "application/json";
const SERVICE_NAME: &str = "enterprise-suite";
const MAX_ENTERPRISE_SUITE_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterpriseSuiteRuntimeRoute {
    pub method: &'static str,              // data_class: INTERNAL_ONLY
    pub path: &'static str,                // data_class: INTERNAL_ONLY
    pub operation_id: &'static str,        // data_class: INTERNAL_ONLY
    pub request_data_class: &'static str,  // data_class: INTERNAL_ONLY
    pub response_data_class: &'static str, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnterpriseSuiteRuntimeError {
    Router(RouterError),
}

impl From<RouterError> for EnterpriseSuiteRuntimeError {
    fn from(error: RouterError) -> Self {
        Self::Router(error)
    }
}

impl std::fmt::Display for EnterpriseSuiteRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnterpriseSuiteRuntimeError::Router(error) => {
                write!(f, "enterprise suite router error: {error:?}")
            }
        }
    }
}

impl std::error::Error for EnterpriseSuiteRuntimeError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnterpriseSuiteAcceptedResponse {
    pub accepted: bool,          // data_class: INTERNAL_ONLY
    pub topic: String,           // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub schema_version: u32,     // data_class: PUBLIC
    pub service: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnterpriseSuiteHealthResponse {
    pub status: String,                     // data_class: PUBLIC
    pub service: String,                    // data_class: PUBLIC
    pub runtime_adapter: String,            // data_class: PUBLIC
    pub deployed_listener: bool,            // data_class: PUBLIC
    pub auth_enforcement_runtime: bool,     // data_class: PUBLIC
    pub storage_attached: bool,             // data_class: PUBLIC
    pub workflow_execution: bool,           // data_class: PUBLIC
    pub open_tofu_execution: bool,          // data_class: PUBLIC
    pub incident_rollback_execution: bool,  // data_class: PUBLIC
    pub child_service_calls: bool,          // data_class: PUBLIC
    pub runtime_audit_chain_emission: bool, // data_class: PUBLIC
    pub cloud_service_integration: bool,    // data_class: PUBLIC
    pub schema_version: u32,                // data_class: PUBLIC
}

pub fn enterprise_suite_runtime_routes() -> Vec<EnterpriseSuiteRuntimeRoute> {
    vec![
        EnterpriseSuiteRuntimeRoute {
            method: "POST",
            path: ENTERPRISE_SUITE_POLICY_ADMISSIONS_PATH,
            operation_id: "admitEnterpriseSuiteChildWrite",
            request_data_class: "INTERNAL_ONLY+FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        EnterpriseSuiteRuntimeRoute {
            method: "POST",
            path: ENTERPRISE_SUITE_GROUP_CLOSE_ROLLUPS_PATH,
            operation_id: "rollUpEnterpriseSuiteGroupClose",
            request_data_class: "INTERNAL_ONLY+FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        EnterpriseSuiteRuntimeRoute {
            method: "POST",
            path: ENTERPRISE_SUITE_CROSS_PRODUCT_WORKFLOW_PLANS_PATH,
            operation_id: "planEnterpriseSuiteCrossProductWorkflow",
            request_data_class: "INTERNAL_ONLY",
            response_data_class: "INTERNAL_ONLY",
        },
        EnterpriseSuiteRuntimeRoute {
            method: "POST",
            path: ENTERPRISE_SUITE_INCIDENT_ROLLBACK_PLANS_PATH,
            operation_id: "planEnterpriseSuiteIncidentRollback",
            request_data_class: "INTERNAL_ONLY",
            response_data_class: "INTERNAL_ONLY",
        },
        EnterpriseSuiteRuntimeRoute {
            method: "POST",
            path: ENTERPRISE_SUITE_OPS_COMMANDS_PATH,
            operation_id: "prepareEnterpriseSuiteOpsCommand",
            request_data_class: "INTERNAL_ONLY",
            response_data_class: "INTERNAL_ONLY",
        },
        EnterpriseSuiteRuntimeRoute {
            method: "GET",
            path: ENTERPRISE_SUITE_HEALTH_PATH,
            operation_id: "enterpriseSuiteRuntimeHealth",
            request_data_class: "PUBLIC",
            response_data_class: "PUBLIC",
        },
    ]
}

pub fn enterprise_suite_server_config() -> ServerConfig {
    ServerConfig::default()
        .with_max_body_bytes(MAX_ENTERPRISE_SUITE_BODY_BYTES)
        .with_header_read_timeout(Duration::from_secs(10))
        .with_keepalive_timeout(Duration::from_secs(30))
}

pub fn enterprise_suite_runtime_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new()
}

pub fn enterprise_suite_runtime_router() -> Result<Router<SyncHandler>, EnterpriseSuiteRuntimeError>
{
    let mut router = Router::new();
    router.route(
        HttpMethod::Post,
        ENTERPRISE_SUITE_POLICY_ADMISSIONS_PATH,
        handler_to_sync(PolicyAdmissionHandler),
    )?;
    router.route(
        HttpMethod::Post,
        ENTERPRISE_SUITE_GROUP_CLOSE_ROLLUPS_PATH,
        handler_to_sync(GroupCloseRollupHandler),
    )?;
    router.route(
        HttpMethod::Post,
        ENTERPRISE_SUITE_CROSS_PRODUCT_WORKFLOW_PLANS_PATH,
        handler_to_sync(CrossProductWorkflowHandler),
    )?;
    router.route(
        HttpMethod::Post,
        ENTERPRISE_SUITE_INCIDENT_ROLLBACK_PLANS_PATH,
        handler_to_sync(IncidentRollbackHandler),
    )?;
    router.route(
        HttpMethod::Post,
        ENTERPRISE_SUITE_OPS_COMMANDS_PATH,
        handler_to_sync(OpsCommandHandler),
    )?;
    router.route(
        HttpMethod::Get,
        ENTERPRISE_SUITE_HEALTH_PATH,
        handler_to_sync(HealthHandler),
    )?;
    Ok(router)
}

pub fn dispatch_enterprise_suite_request(request: HttpRequest) -> HttpResponse {
    match enterprise_suite_runtime_router() {
        Ok(router) => dispatch_http(request, &router, &enterprise_suite_runtime_chain()),
        Err(error) => json_response(
            500,
            &ApiErrorEnvelope::validation(
                "Enterprise Suite runtime router failed",
                Some(error.to_string()),
            ),
        ),
    }
}

struct PolicyAdmissionHandler;
struct GroupCloseRollupHandler;
struct CrossProductWorkflowHandler;
struct IncidentRollbackHandler;
struct OpsCommandHandler;
struct HealthHandler;

impl oya_http_middleware_kernel::Handler for PolicyAdmissionHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: ChildWriteAdmissionRequest = parse_json(&req.body)?;
        let decision = admit_child_write(request.into_domain()).map_err(domain_error_response)?;
        Ok(json_response(
            202,
            &EnterpriseSuiteAcceptedResponse {
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
            &EnterpriseSuiteAcceptedResponse {
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

impl oya_http_middleware_kernel::Handler for CrossProductWorkflowHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: CrossProductWorkflowPlanRequest = parse_json(&req.body)?;
        let plan =
            plan_cross_product_workflow(request.into_domain()).map_err(domain_error_response)?;
        let envelope = prepare_cross_product_workflow_envelope(&plan);
        Ok(json_response(
            200,
            &EnterpriseSuiteAcceptedResponse {
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
            &EnterpriseSuiteAcceptedResponse {
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
        let request: EnterpriseOpsCommandRequest = parse_json(&req.body)?;
        let envelope =
            prepare_enterprise_ops_envelope(request.into_app()).map_err(app_error_response)?;
        Ok(json_response(
            202,
            &EnterpriseSuiteAcceptedResponse {
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
            &EnterpriseSuiteHealthResponse {
                status: "ok".to_owned(),
                service: SERVICE_NAME.to_owned(),
                runtime_adapter: "router-ready".to_owned(),
                deployed_listener: false,
                auth_enforcement_runtime: false,
                storage_attached: false,
                workflow_execution: false,
                open_tofu_execution: false,
                incident_rollback_execution: false,
                child_service_calls: false,
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
                "Invalid Enterprise Suite JSON request",
                Some(error.to_string()),
            ),
        )
    })
}

fn domain_error_response(error: EnterpriseSuiteDomainError) -> HttpResponse {
    json_response(
        400,
        &ApiErrorEnvelope::validation(
            "Invalid Enterprise Suite command",
            Some(format!("{error:?}")),
        ),
    )
}

fn app_error_response(error: EnterpriseSuiteAppError) -> HttpResponse {
    json_response(
        400,
        &ApiErrorEnvelope::validation(
            "Invalid Enterprise Suite command",
            Some(format!("{error:?}")),
        ),
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
