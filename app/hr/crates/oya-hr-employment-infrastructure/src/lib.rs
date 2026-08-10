//! HR employment HTTP runtime adapter foundation.
//!
//! This crate binds HR API DTOs to the repo-native Hyper router/middleware
//! foundation without introducing a deployed listener. It validates JSON,
//! invokes HR app-layer metadata planners, and serializes OpenAPI-aligned
//! responses. It does not persist HR records, retrieve sensitive data, execute
//! Workflow, call Payroll, emit runtime audit-chain events, or deploy cloud I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod authz;

pub use authz::{
    AUTHORIZATION_HEADER, AuthzProviderConfigError, CallerCredential,
    ConfiguredBearerPrincipalVerifier, HrAuthzMiddleware, HrAuthzProvider, HrControlPlaneAction,
    HrControlPlaneAuthorizationError, HrControlPlaneAuthorizer, HrControlPlaneResource,
    PrincipalVerificationError, PrincipalVerifier, VERIFIED_TENANT_CAPTURE_KEY, VerifiedPrincipal,
    action_for_template, constant_time_eq,
};

use std::time::Duration;

use oya_hr_employment_api::{
    ApiErrorEnvelope, LaborCompliancePlanRequest, LeavePayrollImpactRequest,
    LeavePayrollImpactResponse, OnboardEmployeeRequest, SensitiveHrReadPolicyRequest,
    SensitiveReadPolicyDecisionResponse,
};
use oya_hr_employment_app::{
    HrAppError, onboard_employee, plan_labor_compliance_workflows,
    plan_leave_payroll_impact_envelope, prepare_sensitive_hr_read_envelope,
};
use oya_hr_employment_domain::HrDomainError;
use oya_http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_runtime_hyper_adapter::{
    ServerConfig, SyncHandler, dispatch as dispatch_http, handler_to_sync,
};
use serde::Serialize;

pub const HR_EMPLOYEES_PATH: &str = "/hr/v1/employees";
pub const HR_LABOR_COMPLIANCE_WORKFLOW_PLANS_PATH: &str = "/hr/v1/labor-compliance-workflow-plans";
pub const HR_SENSITIVE_READ_POLICY_DECISIONS_PATH: &str = "/hr/v1/sensitive-read-policy-decisions";
pub const HR_LEAVE_PAYROLL_IMPACT_PLANS_PATH: &str = "/hr/v1/leave-payroll-impact-plans";
pub const HR_HEALTH_PATH: &str = "/hr/v1/healthz";

const JSON_CONTENT_TYPE: &str = "application/json";
const SERVICE_NAME: &str = "hr";
const MAX_HR_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrRuntimeRoute {
    pub method: &'static str,              // data_class: INTERNAL_ONLY
    pub path: &'static str,                // data_class: INTERNAL_ONLY
    pub operation_id: &'static str,        // data_class: INTERNAL_ONLY
    pub request_data_class: &'static str,  // data_class: INTERNAL_ONLY
    pub response_data_class: &'static str, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HrRuntimeError {
    Router(RouterError),
}

impl From<RouterError> for HrRuntimeError {
    fn from(error: RouterError) -> Self {
        Self::Router(error)
    }
}

impl std::fmt::Display for HrRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HrRuntimeError::Router(error) => write!(f, "hr router error: {error:?}"),
        }
    }
}

impl std::error::Error for HrRuntimeError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HrAcceptedResponse {
    pub accepted: bool,          // data_class: INTERNAL_ONLY
    pub audit_topic: String,     // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub schema_version: u32,     // data_class: PUBLIC
    pub service: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaborCompliancePlanResponse {
    pub workflow_dispatches: Vec<WorkflowDispatchResponse>, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDispatchResponse {
    pub topic: String,           // data_class: INTERNAL_ONLY
    pub workflow_ref: String,    // data_class: INTERNAL_ONLY
    pub obligation_kind: String, // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HrHealthResponse {
    pub status: String,             // data_class: PUBLIC
    pub service: String,            // data_class: PUBLIC
    pub runtime_adapter: String,    // data_class: PUBLIC
    pub deployed_listener: bool,    // data_class: PUBLIC
    pub storage_attached: bool,     // data_class: PUBLIC
    pub workflow_execution: bool,   // data_class: PUBLIC
    pub payroll_network_call: bool, // data_class: PUBLIC
    pub sensitive_data_fetch: bool, // data_class: PUBLIC
    pub schema_version: u32,        // data_class: PUBLIC
}

pub fn hr_runtime_routes() -> Vec<HrRuntimeRoute> {
    vec![
        HrRuntimeRoute {
            method: "POST",
            path: HR_EMPLOYEES_PATH,
            operation_id: "onboardHrEmployee",
            request_data_class: "PII_IDENTIFYING",
            response_data_class: "INTERNAL_ONLY",
        },
        HrRuntimeRoute {
            method: "POST",
            path: HR_LABOR_COMPLIANCE_WORKFLOW_PLANS_PATH,
            operation_id: "planHrLaborComplianceWorkflows",
            request_data_class: "INTERNAL_ONLY",
            response_data_class: "INTERNAL_ONLY",
        },
        HrRuntimeRoute {
            method: "POST",
            path: HR_SENSITIVE_READ_POLICY_DECISIONS_PATH,
            operation_id: "evaluateHrSensitiveReadPolicy",
            request_data_class: "SENSITIVE_PIPA_ART23",
            response_data_class: "SENSITIVE_PIPA_ART23",
        },
        HrRuntimeRoute {
            method: "POST",
            path: HR_LEAVE_PAYROLL_IMPACT_PLANS_PATH,
            operation_id: "planHrLeavePayrollImpact",
            request_data_class: "FINANCIAL",
            response_data_class: "FINANCIAL",
        },
        HrRuntimeRoute {
            method: "GET",
            path: HR_HEALTH_PATH,
            operation_id: "hrRuntimeHealth",
            request_data_class: "PUBLIC",
            response_data_class: "PUBLIC",
        },
    ]
}

pub fn hr_server_config() -> ServerConfig {
    ServerConfig::default()
        .with_max_body_bytes(MAX_HR_BODY_BYTES)
        .with_header_read_timeout(Duration::from_secs(10))
        .with_keepalive_timeout(Duration::from_secs(30))
}

#[must_use]
pub fn hr_runtime_chain(provider: HrAuthzProvider) -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new().push(Box::new(HrAuthzMiddleware::new(provider)))
}

pub fn hr_runtime_router() -> Result<Router<SyncHandler>, HrRuntimeError> {
    let mut router = Router::new();
    router.route(
        HttpMethod::Post,
        HR_EMPLOYEES_PATH,
        handler_to_sync(OnboardEmployeeHandler),
    )?;
    router.route(
        HttpMethod::Post,
        HR_LABOR_COMPLIANCE_WORKFLOW_PLANS_PATH,
        handler_to_sync(LaborComplianceHandler),
    )?;
    router.route(
        HttpMethod::Post,
        HR_SENSITIVE_READ_POLICY_DECISIONS_PATH,
        handler_to_sync(SensitiveReadHandler),
    )?;
    router.route(
        HttpMethod::Post,
        HR_LEAVE_PAYROLL_IMPACT_PLANS_PATH,
        handler_to_sync(LeavePayrollImpactHandler),
    )?;
    router.route(
        HttpMethod::Get,
        HR_HEALTH_PATH,
        handler_to_sync(HealthHandler),
    )?;
    Ok(router)
}

pub fn dispatch_hr_request(request: HttpRequest, provider: HrAuthzProvider) -> HttpResponse {
    match hr_runtime_router() {
        Ok(router) => dispatch_http(request, &router, &hr_runtime_chain(provider)),
        Err(error) => json_response(
            500,
            &ApiErrorEnvelope::validation("HR runtime router failed", Some(error.to_string())),
        ),
    }
}

fn enforce_body_tenant_matches_verified(
    req: &HttpRequest,
    body_tenant_id: &str,
) -> Result<(), HttpResponse> {
    match req.path_captures.get(VERIFIED_TENANT_CAPTURE_KEY) {
        Some(verified) if constant_time_eq(verified.as_bytes(), body_tenant_id.as_bytes()) => {
            Ok(())
        }
        _ => Err(json_response(
            403,
            &ApiErrorEnvelope::validation(
                "HR mutation tenant does not match the verified principal",
                None,
            ),
        )),
    }
}

struct OnboardEmployeeHandler;
struct LaborComplianceHandler;
struct SensitiveReadHandler;
struct LeavePayrollImpactHandler;
struct HealthHandler;

impl oya_http_middleware_kernel::Handler for OnboardEmployeeHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: OnboardEmployeeRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let outcome = onboard_employee(request.into_command()).map_err(app_error_response)?;
        Ok(json_response(
            202,
            &HrAcceptedResponse {
                accepted: true,
                audit_topic: outcome.audit_envelope.topic.value.clone(),
                idempotency_key: outcome.audit_envelope.idempotency_key.value.clone(),
                schema_version: outcome.audit_envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for LaborComplianceHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: LaborCompliancePlanRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let outcome =
            plan_labor_compliance_workflows(request.into_snapshot()).map_err(app_error_response)?;
        let workflow_dispatches = outcome
            .workflow_dispatches
            .iter()
            .map(|dispatch| WorkflowDispatchResponse {
                topic: dispatch.topic.value.clone(),
                workflow_ref: dispatch.workflow_ref.value.value.clone(),
                obligation_kind: format!("{:?}", dispatch.obligation_kind.value),
                idempotency_key: dispatch.idempotency_key.value.clone(),
            })
            .collect();
        Ok(json_response(
            200,
            &LaborCompliancePlanResponse {
                workflow_dispatches,
                schema_version: 1,
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for SensitiveReadHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: SensitiveHrReadPolicyRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let outcome = prepare_sensitive_hr_read_envelope(request.into_domain_input())
            .map_err(app_error_response)?;
        Ok(json_response(
            200,
            &SensitiveReadPolicyDecisionResponse::from_outcome(&outcome),
        ))
    }
}

impl oya_http_middleware_kernel::Handler for LeavePayrollImpactHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: LeavePayrollImpactRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let outcome = plan_leave_payroll_impact_envelope(request.into_domain_input())
            .map_err(app_error_response)?;
        Ok(json_response(
            200,
            &LeavePayrollImpactResponse::from_outcome(&outcome),
        ))
    }
}

impl oya_http_middleware_kernel::Handler for HealthHandler {
    type Error = HttpResponse;

    fn call(&self, _req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        Ok(json_response(
            200,
            &HrHealthResponse {
                status: "ok".to_owned(),
                service: SERVICE_NAME.to_owned(),
                runtime_adapter: "router-ready".to_owned(),
                deployed_listener: false,
                storage_attached: false,
                workflow_execution: false,
                payroll_network_call: false,
                sensitive_data_fetch: false,
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
            &ApiErrorEnvelope::validation("Invalid HR JSON request", Some(error.to_string())),
        )
    })
}

fn app_error_response(error: HrAppError) -> HttpResponse {
    let status = match error {
        HrAppError::Domain(HrDomainError::DisallowedSensitiveReadPurpose) => 403,
        _ => 400,
    };
    json_response(
        status,
        &ApiErrorEnvelope::validation("Invalid HR command", Some(format!("{error:?}"))),
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
