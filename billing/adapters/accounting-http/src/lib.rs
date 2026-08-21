//! Accounting journal HTTP runtime adapter foundation.
//!
//! This crate binds accounting API DTOs to the repo-native Hyper
//! router/middleware foundation without introducing a deployed listener. It
//! validates JSON, invokes accounting app-layer metadata planners, and
//! serializes OpenAPI-aligned responses. It does not persist ledgers, execute
//! payments, call Payroll, execute Workflow, submit VAT filings, emit runtime
//! audit-chain events, or deploy cloud I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod authz;

pub use authz::{
    AUTHORIZATION_HEADER, AccountingAuthzMiddleware, AccountingAuthzProvider,
    AccountingMutationAction, AccountingMutationAuthorizationError, AccountingMutationAuthorizer,
    AccountingMutationResource, AuthzProviderConfigError, CallerCredential,
    ConfiguredBearerPrincipalVerifier, PrincipalVerificationError, PrincipalVerifier,
    VERIFIED_TENANT_CAPTURE_KEY, VerifiedPrincipal, action_for_template, constant_time_eq,
};

use std::time::Duration;

use billing_accounting_api::{
    ApiErrorEnvelope, JournalPostRequest, PayrollPostingRequest, VatDeadlineRequest,
};
use billing_accounting_app::{
    AccountingAppError, plan_vat_workflow, post_journal_with_audit, record_payroll_posting,
};
use http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use http_router_kernel::{HttpMethod, Router, RouterError};
use http_runtime_hyper_adapter::{
    ServerConfig, SyncHandler, dispatch as dispatch_http, handler_to_sync,
};
use serde::Serialize;

pub const ACCOUNTING_JOURNALS_PATH: &str = "/accounting/v1/journals";
pub const ACCOUNTING_PAYROLL_POSTINGS_PATH: &str = "/accounting/v1/payroll-postings";
pub const ACCOUNTING_VAT_WORKFLOW_PLANS_PATH: &str = "/accounting/v1/vat-workflow-plans";
pub const ACCOUNTING_HEALTH_PATH: &str = "/accounting/v1/healthz";

const JSON_CONTENT_TYPE: &str = "application/json";
const SERVICE_NAME: &str = "accounting";
const MAX_ACCOUNTING_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingRuntimeRoute {
    pub method: &'static str,              // data_class: INTERNAL_ONLY
    pub path: &'static str,                // data_class: INTERNAL_ONLY
    pub operation_id: &'static str,        // data_class: INTERNAL_ONLY
    pub request_data_class: &'static str,  // data_class: INTERNAL_ONLY
    pub response_data_class: &'static str, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccountingRuntimeError {
    Router(RouterError),
}

impl From<RouterError> for AccountingRuntimeError {
    fn from(error: RouterError) -> Self {
        Self::Router(error)
    }
}

impl std::fmt::Display for AccountingRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountingRuntimeError::Router(error) => {
                write!(f, "accounting router error: {error:?}")
            }
        }
    }
}

impl std::error::Error for AccountingRuntimeError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingAcceptedResponse {
    pub accepted: bool,          // data_class: INTERNAL_ONLY
    pub audit_topic: String,     // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub schema_version: u32,     // data_class: PUBLIC
    pub service: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VatWorkflowPlanResponse {
    pub opened: bool,                 // data_class: INTERNAL_ONLY
    pub workflow_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub required_steps: Vec<String>,  // data_class: INTERNAL_ONLY
    pub schema_version: u32,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingHealthResponse {
    pub status: String,               // data_class: PUBLIC
    pub service: String,              // data_class: PUBLIC
    pub runtime_adapter: String,      // data_class: PUBLIC
    pub deployed_listener: bool,      // data_class: PUBLIC
    pub storage_attached: bool,       // data_class: PUBLIC
    pub workflow_execution: bool,     // data_class: PUBLIC
    pub statutory_filing_rails: bool, // data_class: PUBLIC
    pub payment_execution: bool,      // data_class: PUBLIC
    pub payroll_network_call: bool,   // data_class: PUBLIC
    pub schema_version: u32,          // data_class: PUBLIC
}

pub fn accounting_runtime_routes() -> Vec<AccountingRuntimeRoute> {
    vec![
        AccountingRuntimeRoute {
            method: "POST",
            path: ACCOUNTING_JOURNALS_PATH,
            operation_id: "postAccountingJournal",
            request_data_class: "FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        AccountingRuntimeRoute {
            method: "POST",
            path: ACCOUNTING_PAYROLL_POSTINGS_PATH,
            operation_id: "recordAccountingPayrollPosting",
            request_data_class: "FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        AccountingRuntimeRoute {
            method: "POST",
            path: ACCOUNTING_VAT_WORKFLOW_PLANS_PATH,
            operation_id: "planAccountingVatWorkflow",
            request_data_class: "FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        AccountingRuntimeRoute {
            method: "GET",
            path: ACCOUNTING_HEALTH_PATH,
            operation_id: "accountingRuntimeHealth",
            request_data_class: "PUBLIC",
            response_data_class: "PUBLIC",
        },
    ]
}

pub fn accounting_server_config() -> ServerConfig {
    ServerConfig::default()
        .with_max_body_bytes(MAX_ACCOUNTING_BODY_BYTES)
        .with_header_read_timeout(Duration::from_secs(10))
        .with_keepalive_timeout(Duration::from_secs(30))
}

/// Build the runtime middleware chain with the AUTH-005 fail-closed authz
/// middleware installed FIRST (ADR-0593). The [`AccountingAuthzMiddleware`]
/// verifies the caller principal and runs the PDP decision on every
/// money-mutation route BEFORE the terminal handler deserializes the body; it
/// short-circuits 401/403.
///
/// There is NO zero-argument / default chain: the composition root MUST supply
/// an [`AccountingAuthzProvider`] (verifier + PDP authorizer), so a money
/// mutation can never be served without both ports running.
#[must_use]
pub fn accounting_runtime_chain(
    provider: AccountingAuthzProvider,
) -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new().push(Box::new(AccountingAuthzMiddleware::new(provider)))
}

pub fn accounting_runtime_router() -> Result<Router<SyncHandler>, AccountingRuntimeError> {
    let mut router = Router::new();
    router.route(
        HttpMethod::Post,
        ACCOUNTING_JOURNALS_PATH,
        handler_to_sync(JournalPostHandler),
    )?;
    router.route(
        HttpMethod::Post,
        ACCOUNTING_PAYROLL_POSTINGS_PATH,
        handler_to_sync(PayrollPostingHandler),
    )?;
    router.route(
        HttpMethod::Post,
        ACCOUNTING_VAT_WORKFLOW_PLANS_PATH,
        handler_to_sync(VatWorkflowHandler),
    )?;
    router.route(
        HttpMethod::Get,
        ACCOUNTING_HEALTH_PATH,
        handler_to_sync(HealthHandler),
    )?;
    Ok(router)
}

/// Dispatch an accounting runtime request through the fail-closed authz chain
/// (ADR-0593). The `provider` (principal verifier + PDP authorizer) is required —
/// money-mutation routes are gated 401/403 before any handler runs. Health is
/// passed through unauthenticated by the middleware.
pub fn dispatch_accounting_request(
    request: HttpRequest,
    provider: AccountingAuthzProvider,
) -> HttpResponse {
    match accounting_runtime_router() {
        Ok(router) => dispatch_http(request, &router, &accounting_runtime_chain(provider)),
        Err(error) => json_response(
            500,
            &ApiErrorEnvelope::validation(
                "Accounting runtime router failed",
                Some(error.to_string()),
            ),
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
                "Accounting mutation tenant does not match the verified principal",
                None,
            ),
        )),
    }
}

struct JournalPostHandler;
struct PayrollPostingHandler;
struct VatWorkflowHandler;
struct HealthHandler;

impl http_middleware_kernel::Handler for JournalPostHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: JournalPostRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let outcome = post_journal_with_audit(request.into_domain()).map_err(app_error_response)?;
        Ok(json_response(
            202,
            &AccountingAcceptedResponse {
                accepted: true,
                audit_topic: outcome.audit_envelope.topic.value.clone(),
                idempotency_key: outcome.audit_envelope.idempotency_key.value.clone(),
                schema_version: outcome.audit_envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl http_middleware_kernel::Handler for PayrollPostingHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: PayrollPostingRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let outcome = record_payroll_posting(request.into_domain()).map_err(app_error_response)?;
        Ok(json_response(
            202,
            &AccountingAcceptedResponse {
                accepted: true,
                audit_topic: outcome.audit_envelope.topic.value.clone(),
                idempotency_key: outcome.audit_envelope.idempotency_key.value.clone(),
                schema_version: outcome.audit_envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl http_middleware_kernel::Handler for VatWorkflowHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: VatDeadlineRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let outcome = plan_vat_workflow(request.into_domain()).map_err(app_error_response)?;
        let response = match outcome.dispatch_envelope {
            Some(envelope) => VatWorkflowPlanResponse {
                opened: true,
                workflow_ref: Some(envelope.workflow_ref.value.value.clone()),
                required_steps: envelope
                    .required_steps
                    .value
                    .iter()
                    .map(|step| format!("{step:?}"))
                    .collect(),
                schema_version: envelope.schema_version.value,
            },
            None => VatWorkflowPlanResponse {
                opened: false,
                workflow_ref: None,
                required_steps: Vec::new(),
                schema_version: 1,
            },
        };
        Ok(json_response(200, &response))
    }
}

impl http_middleware_kernel::Handler for HealthHandler {
    type Error = HttpResponse;

    fn call(&self, _req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        Ok(json_response(
            200,
            &AccountingHealthResponse {
                status: "ok".to_owned(),
                service: SERVICE_NAME.to_owned(),
                runtime_adapter: "router-ready".to_owned(),
                deployed_listener: false,
                storage_attached: false,
                workflow_execution: false,
                statutory_filing_rails: false,
                payment_execution: false,
                payroll_network_call: false,
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
                "Invalid accounting JSON request",
                Some(error.to_string()),
            ),
        )
    })
}

fn app_error_response(error: AccountingAppError) -> HttpResponse {
    json_response(
        400,
        &ApiErrorEnvelope::validation("Invalid accounting command", Some(format!("{error:?}"))),
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
