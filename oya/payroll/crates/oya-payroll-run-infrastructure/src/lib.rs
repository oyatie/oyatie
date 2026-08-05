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

use std::time::Duration;

use oya_http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_runtime_hyper_adapter::{
    ServerConfig, SyncHandler, dispatch as dispatch_http, handler_to_sync,
};
use oya_payroll_run_api::{
    ApiErrorEnvelope, HrLeaveImpactIntakeRequest, HrLeaveImpactIntakeResponse,
    HrLeaveImpactKindDto, PayrollJournalDraftRequest, PayrollTrialCloseRequest,
    StatutoryCalculationPreviewRequest, StatutoryCalculationPreviewResponse,
    YearEndSettlementPreviewRequest, YearEndSettlementPreviewResponse,
};
use oya_payroll_run_app::{
    PayrollAppError, close_trial_run, prepare_accounting_dispatch, prepare_hr_leave_impact_intake,
    prepare_statutory_calculation_preview, prepare_year_end_settlement_preview,
};
use serde::Serialize;

pub const PAYROLL_TRIAL_CLOSE_PATH: &str = "/payroll/v1/trial-closes";
pub const PAYROLL_ACCOUNTING_JOURNAL_DRAFT_PATH: &str = "/payroll/v1/accounting-journal-drafts";
pub const PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH: &str = "/payroll/v1/hr-leave-impact-intakes";
pub const PAYROLL_STATUTORY_CALCULATION_PREVIEW_PATH: &str =
    "/payroll/v1/statutory-calculation-previews";
pub const PAYROLL_YEAR_END_SETTLEMENT_PREVIEW_PATH: &str =
    "/payroll/v1/year-end-settlement-previews";
pub const PAYROLL_HEALTH_PATH: &str = "/payroll/v1/healthz";

const JSON_CONTENT_TYPE: &str = "application/json";
const SERVICE_NAME: &str = "payroll";
const MAX_PAYROLL_BODY_BYTES: usize = 64 * 1024;
const LIVE_DEPLOYMENT_NA_RATIONALE: &str = "N/A: no deployable listener or cloud target exists for this Payroll runtime adapter card; evidence is local in-process router replay only.";
const ACCESSIBILITY_NA_RATIONALE: &str =
    "N/A: backend/router-only replay has no Payroll UI surface or browser workflow in this card.";

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
    pub accepted: bool,             // data_class: INTERNAL_ONLY
    pub audit_topic: String,        // data_class: INTERNAL_ONLY
    pub idempotency_key: String,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,        // data_class: PUBLIC
    pub service: String,            // data_class: INTERNAL_ONLY
    pub story_stage: &'static str,  // data_class: PUBLIC
    pub run_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,    // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub source_digest: String,      // data_class: FINANCIAL
    pub payload_data_class: String, // data_class: INTERNAL_ONLY
    #[serde(flatten)]
    pub non_claims: PayrollReplayNonClaims,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollHrLeaveImpactReplayResponse {
    pub story_stage: &'static str,         // data_class: PUBLIC
    pub integration_topic: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub payroll_period: String,            // data_class: FINANCIAL
    pub impact_kind: HrLeaveImpactKindDto, // data_class: FINANCIAL
    pub source_hr_idempotency_key: String, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY + FINANCIAL
    pub payload_data_class: String,        // data_class: INTERNAL_ONLY
    pub schema_version: u32,               // data_class: PUBLIC
    #[serde(flatten)]
    pub non_claims: PayrollReplayNonClaims,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollReplayNonClaims {
    pub local_replay_only: bool,                 // data_class: PUBLIC
    pub deployed_listener: bool,                 // data_class: PUBLIC
    pub storage_attached: bool,                  // data_class: PUBLIC
    pub workflow_dispatch: bool,                 // data_class: PUBLIC
    pub runtime_audit_emission: bool,            // data_class: PUBLIC
    pub external_hr_call: bool,                  // data_class: PUBLIC
    pub external_accounting_call: bool,          // data_class: PUBLIC
    pub statutory_filing_rails: bool,            // data_class: PUBLIC
    pub disbursement_rails: bool,                // data_class: PUBLIC
    pub production_close: bool,                  // data_class: PUBLIC
    pub live_deployment_rationale: &'static str, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollHealthResponse {
    pub status: String,                        // data_class: PUBLIC
    pub service: String,                       // data_class: PUBLIC
    pub runtime_adapter: String,               // data_class: PUBLIC
    pub close_health_gate: String,             // data_class: PUBLIC
    pub rollback_observability: String,        // data_class: PUBLIC
    pub local_replay_only: bool,               // data_class: PUBLIC
    pub live_deployment_status: String,        // data_class: PUBLIC
    pub live_deployment_rationale: String,     // data_class: PUBLIC
    pub accessibility_evidence_status: String, // data_class: PUBLIC
    pub production_close_controller: bool,     // data_class: PUBLIC
    pub deployed_listener: bool,               // data_class: PUBLIC
    pub storage_attached: bool,                // data_class: PUBLIC
    pub workflow_dispatch: bool,               // data_class: PUBLIC
    pub opentofu_ops_convergence: bool,        // data_class: PUBLIC
    pub statutory_filing_rails: bool,          // data_class: PUBLIC
    pub schema_version: u32,                   // data_class: PUBLIC
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
            method: "POST",
            path: PAYROLL_STATUTORY_CALCULATION_PREVIEW_PATH,
            operation_id: "previewPayrollStatutoryCalculation",
            request_data_class: "FINANCIAL",
            response_data_class: "FINANCIAL",
        },
        PayrollRuntimeRoute {
            method: "POST",
            path: PAYROLL_YEAR_END_SETTLEMENT_PREVIEW_PATH,
            operation_id: "previewPayrollYearEndSettlement",
            request_data_class: "PII_IDENTIFYING+FINANCIAL",
            response_data_class: "PII_IDENTIFYING+FINANCIAL",
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

pub fn payroll_runtime_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new()
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
        HttpMethod::Post,
        PAYROLL_STATUTORY_CALCULATION_PREVIEW_PATH,
        handler_to_sync(StatutoryCalculationPreviewHandler),
    )?;
    router.route(
        HttpMethod::Post,
        PAYROLL_YEAR_END_SETTLEMENT_PREVIEW_PATH,
        handler_to_sync(YearEndSettlementPreviewHandler),
    )?;
    router.route(
        HttpMethod::Get,
        PAYROLL_HEALTH_PATH,
        handler_to_sync(HealthHandler),
    )?;
    Ok(router)
}

pub fn dispatch_payroll_request(request: HttpRequest) -> HttpResponse {
    match payroll_runtime_router() {
        Ok(router) => dispatch_http(request, &router, &payroll_runtime_chain()),
        Err(error) => json_response(
            500,
            &ApiErrorEnvelope::validation("Payroll runtime router failed", Some(error.to_string())),
        ),
    }
}

struct TrialCloseHandler;
struct AccountingJournalDraftHandler;
struct HrLeaveImpactIntakeHandler;
struct StatutoryCalculationPreviewHandler;
struct YearEndSettlementPreviewHandler;
struct HealthHandler;

impl oya_http_middleware_kernel::Handler for TrialCloseHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: PayrollTrialCloseRequest = parse_json(&req.body)?;
        let outcome = close_trial_run(request.into_domain()).map_err(app_error_response)?;
        Ok(json_response(
            202,
            &PayrollAcceptedResponse {
                accepted: true,
                audit_topic: outcome.audit_envelope.topic.value.clone(),
                idempotency_key: outcome.audit_envelope.idempotency_key.value.clone(),
                schema_version: outcome.audit_envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
                story_stage: "payroll_trial_close",
                run_id: outcome.audit_envelope.run_id.value.value.clone(),
                tenant_id: outcome.audit_envelope.tenant_id.value.value.clone(),
                legal_entity_id: outcome.audit_envelope.legal_entity_id.value.value.clone(),
                evidence_refs: vec![outcome.audit_envelope.evidence_ref.value.value.clone()],
                source_digest: outcome.audit_envelope.evidence_digest.value.value.clone(),
                payload_data_class: "FINANCIAL".to_owned(),
                non_claims: payroll_replay_non_claims(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for AccountingJournalDraftHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: PayrollJournalDraftRequest = parse_json(&req.body)?;
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
                story_stage: "accounting_journal_draft",
                run_id: outcome.dispatch_envelope.run_id.value.value.clone(),
                tenant_id: outcome.dispatch_envelope.tenant_id.value.value.clone(),
                legal_entity_id: outcome
                    .dispatch_envelope
                    .legal_entity_id
                    .value
                    .value
                    .clone(),
                evidence_refs: vec![
                    outcome
                        .dispatch_envelope
                        .approval_evidence_ref
                        .value
                        .value
                        .clone(),
                    outcome
                        .dispatch_envelope
                        .reversal_required_ref
                        .value
                        .value
                        .clone(),
                ],
                source_digest: outcome
                    .dispatch_envelope
                    .source_payroll_digest
                    .value
                    .value
                    .clone(),
                payload_data_class: "FINANCIAL".to_owned(),
                non_claims: payroll_replay_non_claims(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for HrLeaveImpactIntakeHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: HrLeaveImpactIntakeRequest = parse_json(&req.body)?;
        let outcome =
            prepare_hr_leave_impact_intake(request.into_domain()).map_err(app_error_response)?;
        let response = HrLeaveImpactIntakeResponse::from_intake(&outcome.intake);
        Ok(json_response(
            202,
            &PayrollHrLeaveImpactReplayResponse {
                story_stage: "hr_leave_impact_intake",
                integration_topic: response.integration_topic,
                idempotency_key: response.idempotency_key,
                payroll_period: response.payroll_period,
                impact_kind: response.impact_kind,
                source_hr_idempotency_key: outcome
                    .intake_envelope
                    .source_hr_idempotency_key
                    .value
                    .clone(),
                evidence_refs: vec![
                    outcome.intake.decision_evidence_ref.value.value.clone(),
                    outcome.intake.routing_evidence_ref.value.value.clone(),
                    outcome
                        .intake
                        .payroll_impact_evidence_ref
                        .value
                        .value
                        .clone(),
                    outcome
                        .intake
                        .payroll_intake_evidence_ref
                        .value
                        .value
                        .clone(),
                ],
                payload_data_class: response.payload_data_class,
                schema_version: response.schema_version,
                non_claims: payroll_replay_non_claims(),
            },
        ))
    }
}

impl oya_http_middleware_kernel::Handler for StatutoryCalculationPreviewHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: StatutoryCalculationPreviewRequest = parse_json(&req.body)?;
        let outcome = prepare_statutory_calculation_preview(request.into_domain())
            .map_err(app_error_response)?;
        Ok(json_response(
            202,
            &StatutoryCalculationPreviewResponse::from_draft(&outcome.draft),
        ))
    }
}

impl oya_http_middleware_kernel::Handler for YearEndSettlementPreviewHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: YearEndSettlementPreviewRequest = parse_json(&req.body)?;
        let outcome = prepare_year_end_settlement_preview(request.into_domain())
            .map_err(app_error_response)?;
        Ok(json_response(
            202,
            &YearEndSettlementPreviewResponse::from_prepared(&outcome.prepared),
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
                close_health_gate: "domain-local-only".to_owned(),
                rollback_observability: "metadata-only".to_owned(),
                local_replay_only: true,
                live_deployment_status: "not-deployed-local-router-only".to_owned(),
                live_deployment_rationale: LIVE_DEPLOYMENT_NA_RATIONALE.to_owned(),
                accessibility_evidence_status: ACCESSIBILITY_NA_RATIONALE.to_owned(),
                production_close_controller: false,
                deployed_listener: false,
                storage_attached: false,
                workflow_dispatch: false,
                opentofu_ops_convergence: false,
                statutory_filing_rails: false,
                schema_version: 1,
            },
        ))
    }
}

fn payroll_replay_non_claims() -> PayrollReplayNonClaims {
    PayrollReplayNonClaims {
        local_replay_only: true,
        deployed_listener: false,
        storage_attached: false,
        workflow_dispatch: false,
        runtime_audit_emission: false,
        external_hr_call: false,
        external_accounting_call: false,
        statutory_filing_rails: false,
        disbursement_rails: false,
        production_close: false,
        live_deployment_rationale: LIVE_DEPLOYMENT_NA_RATIONALE,
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
