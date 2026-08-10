//! Payroll run HTTP runtime adapter foundation.
//!
//! This crate binds the payroll API DTOs to the repo-native Hyper runtime
//! foundation without introducing an HTTP framework or deployed listener. It
//! constructs a router that can be exercised by tests and later passed to the
//! shared Hyper server entry point. The handlers validate JSON, call the
//! payroll app layer, and return OpenAPI-aligned JSON responses. They do not
//! persist payroll state, calculate wages, submit filings, disburse funds, call
//! HR/accounting services, execute Workflow, or emit runtime audit-chain events.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod authz;

pub use authz::{
    AUTHORIZATION_HEADER, AuthzProviderConfigError, CallerCredential,
    ConfiguredBearerPrincipalVerifier, MoneyMutationAction, MoneyMutationAuthorizationError,
    MoneyMutationAuthorizer, MoneyMutationResource, PayrollAuthzMiddleware, PayrollAuthzProvider,
    PrincipalVerificationError, PrincipalVerifier, VERIFIED_TENANT_CAPTURE_KEY, VerifiedPrincipal,
    action_for_template, constant_time_eq,
};

use std::time::Duration;

use oya_http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_runtime_hyper_adapter::{
    ServerConfig, SyncHandler, dispatch as dispatch_http, handler_to_sync,
};
use oya_payroll_run_api::{
    ApiErrorEnvelope, HrLeaveImpactIntakeRequest, HrLeaveImpactIntakeResponse,
    PayrollJournalDraftRequest, PayrollTrialCloseRequest,
};
use oya_payroll_run_app::{
    PayrollAppError, close_trial_run, prepare_accounting_dispatch, prepare_hr_leave_impact_intake,
};
use serde::Serialize;

pub const PAYROLL_TRIAL_CLOSE_PATH: &str = "/payroll/v1/trial-closes";
pub const PAYROLL_ACCOUNTING_JOURNAL_DRAFT_PATH: &str = "/payroll/v1/accounting-journal-drafts";
pub const PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH: &str = "/payroll/v1/hr-leave-impact-intakes";
pub const PAYROLL_HEALTH_PATH: &str = "/payroll/v1/healthz";

const JSON_CONTENT_TYPE: &str = "application/json";
const SERVICE_NAME: &str = "payroll";
const MAX_PAYROLL_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollRuntimeRoute {
    pub method: &'static str,              // data_class: INTERNAL_ONLY
    pub path: &'static str,                // data_class: INTERNAL_ONLY
    pub operation_id: &'static str,        // data_class: INTERNAL_ONLY
    pub request_data_class: &'static str,  // data_class: INTERNAL_ONLY
    pub response_data_class: &'static str, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PayrollRuntimeError {
    Router(RouterError),
}

impl From<RouterError> for PayrollRuntimeError {
    fn from(error: RouterError) -> Self {
        Self::Router(error)
    }
}

impl std::fmt::Display for PayrollRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayrollRuntimeError::Router(error) => write!(f, "payroll router error: {error:?}"),
        }
    }
}

impl std::error::Error for PayrollRuntimeError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollAcceptedResponse {
    pub accepted: bool,          // data_class: INTERNAL_ONLY
    pub audit_topic: String,     // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub schema_version: u32,     // data_class: PUBLIC
    pub service: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollHealthResponse {
    pub status: String,               // data_class: PUBLIC
    pub service: String,              // data_class: PUBLIC
    pub runtime_adapter: String,      // data_class: PUBLIC
    pub deployed_listener: bool,      // data_class: PUBLIC
    pub storage_attached: bool,       // data_class: PUBLIC
    pub workflow_dispatch: bool,      // data_class: PUBLIC
    pub statutory_filing_rails: bool, // data_class: PUBLIC
    pub schema_version: u32,          // data_class: PUBLIC
}

pub fn payroll_runtime_routes() -> Vec<PayrollRuntimeRoute> {
    vec![
        PayrollRuntimeRoute {
            method: "POST",
            path: PAYROLL_TRIAL_CLOSE_PATH,
            operation_id: "closePayrollTrialRun",
            request_data_class: "PII_IDENTIFYING+FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        PayrollRuntimeRoute {
            method: "POST",
            path: PAYROLL_ACCOUNTING_JOURNAL_DRAFT_PATH,
            operation_id: "preparePayrollAccountingJournalDraft",
            request_data_class: "FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        PayrollRuntimeRoute {
            method: "POST",
            path: PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH,
            operation_id: "intakePayrollHrLeaveImpact",
            request_data_class: "FINANCIAL",
            response_data_class: "FINANCIAL",
        },
        PayrollRuntimeRoute {
            method: "GET",
            path: PAYROLL_HEALTH_PATH,
            operation_id: "payrollRuntimeHealth",
            request_data_class: "PUBLIC",
            response_data_class: "PUBLIC",
        },
    ]
}

pub fn payroll_server_config() -> ServerConfig {
    ServerConfig::default()
        .with_max_body_bytes(MAX_PAYROLL_BODY_BYTES)
        .with_header_read_timeout(Duration::from_secs(10))
        .with_keepalive_timeout(Duration::from_secs(30))
}

/// Build the runtime middleware chain with the AUTH-005 fail-closed authz
/// middleware installed FIRST (ADR-0593). The [`PayrollAuthzMiddleware`] verifies
/// the caller principal and runs the PDP decision on every money-mutation route
/// BEFORE the terminal handler deserializes the body; it short-circuits 401/403.
///
/// There is NO zero-argument / default chain: the composition root MUST supply a
/// [`PayrollAuthzProvider`] (verifier + PDP authorizer), so a money mutation can
/// never be served without both ports running.
#[must_use]
pub fn payroll_runtime_chain(
    provider: PayrollAuthzProvider,
) -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new().push(Box::new(PayrollAuthzMiddleware::new(provider)))
}

pub fn payroll_runtime_router() -> Result<Router<SyncHandler>, PayrollRuntimeError> {
    let mut router = Router::new();
    router.route(
        HttpMethod::Post,
        PAYROLL_TRIAL_CLOSE_PATH,
        handler_to_sync(TrialCloseHandler),
    )?;
    router.route(
        HttpMethod::Post,
        PAYROLL_ACCOUNTING_JOURNAL_DRAFT_PATH,
        handler_to_sync(AccountingJournalDraftHandler),
    )?;
    router.route(
        HttpMethod::Post,
        PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH,
        handler_to_sync(HrLeaveImpactIntakeHandler),
    )?;
    router.route(
        HttpMethod::Get,
        PAYROLL_HEALTH_PATH,
        handler_to_sync(HealthHandler),
    )?;
    Ok(router)
}

/// Dispatch a payroll runtime request through the fail-closed authz chain
/// (ADR-0593). The `provider` (principal verifier + PDP authorizer) is required —
/// money-mutation routes are gated 401/403 before any handler runs. Health is
/// passed through unauthenticated by the middleware.
pub fn dispatch_payroll_request(
    request: HttpRequest,
    provider: PayrollAuthzProvider,
) -> HttpResponse {
    match payroll_runtime_router() {
        Ok(router) => dispatch_http(request, &router, &payroll_runtime_chain(provider)),
        Err(error) => json_response(
            500,
            &ApiErrorEnvelope::validation("Payroll runtime router failed", Some(error.to_string())),
        ),
    }
}

/// Cross-check that the body-claimed `tenant_id` equals the VERIFIED tenant the
/// authz middleware injected into `path_captures`. A mismatch is a cross-tenant
/// body substitution attempt (true blast radius): tenant A presenting a verified
/// credential but a body naming tenant B. Fail-closed -> HTTP 403.
///
/// Returns `Err(403)` when the verified tenant is absent (the mutation reached a
/// handler without passing the authz middleware — defense in depth) or differs
/// from the body tenant.
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
                "Payroll mutation tenant does not match the verified principal",
                None,
            ),
        )),
    }
}

struct TrialCloseHandler;
struct AccountingJournalDraftHandler;
struct HrLeaveImpactIntakeHandler;
struct HealthHandler;

impl oya_http_middleware_kernel::Handler for TrialCloseHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: PayrollTrialCloseRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let outcome = close_trial_run(request.into_domain()).map_err(app_error_response)?;
        Ok(json_response(
            202,
            &PayrollAcceptedResponse {
                accepted: true,
                audit_topic: outcome.audit_envelope.topic.value.clone(),
                idempotency_key: outcome.audit_envelope.idempotency_key.value.clone(),
                schema_version: outcome.audit_envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for AccountingJournalDraftHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: PayrollJournalDraftRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let outcome =
            prepare_accounting_dispatch(request.into_domain()).map_err(app_error_response)?;
        Ok(json_response(
            202,
            &PayrollAcceptedResponse {
                accepted: true,
                audit_topic: outcome.dispatch_envelope.topic.value.clone(),
                idempotency_key: outcome.dispatch_envelope.idempotency_key.value.clone(),
                schema_version: outcome.dispatch_envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for HrLeaveImpactIntakeHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: HrLeaveImpactIntakeRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let outcome =
            prepare_hr_leave_impact_intake(request.into_domain()).map_err(app_error_response)?;
        Ok(json_response(
            202,
            &HrLeaveImpactIntakeResponse::from_intake(&outcome.intake),
        ))
    }
}

impl oya_http_middleware_kernel::Handler for HealthHandler {
    type Error = HttpResponse;

    fn call(&self, _req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        Ok(json_response(
            200,
            &PayrollHealthResponse {
                status: "ok".to_owned(),
                service: SERVICE_NAME.to_owned(),
                runtime_adapter: "router-ready".to_owned(),
                deployed_listener: false,
                storage_attached: false,
                workflow_dispatch: false,
                statutory_filing_rails: false,
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
            &ApiErrorEnvelope::validation("Invalid payroll JSON request", Some(error.to_string())),
        )
    })
}

fn app_error_response(error: PayrollAppError) -> HttpResponse {
    json_response(
        400,
        &ApiErrorEnvelope::validation("Invalid payroll command", Some(format!("{error:?}"))),
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
