//! Workflow-engine trigger-orchestrator API boundary foundation.
//!
//! This crate owns a source-level, framework-free API boundary for trigger
//! orchestration commands that will later be served by REST/gRPC adapters. It
//! validates route, contract-version, tenant, principal, authorization,
//! idempotency, source/kind, and safe metadata bindings; maps API DTOs into the
//! trigger-orchestrator usecase/domain request; delegates only after boundary
//! checks pass; returns stable status/problem DTOs; and preserves in-memory
//! idempotent API replay semantics. It performs no HTTP serving,
//! serialization-framework work, concrete storage, scheduler execution, webhook
//! serving, HMAC verification, event-bus consumption, run creation, network I/O,
//! durable idempotency storage, Kubernetes, cloud, or tenant workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use workflow_trigger_orchestrator_usecase::{
    TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION, TRIGGER_ORCHESTRATOR_DOMAIN_CONTRACT_REF,
    TRIGGER_ORCHESTRATOR_DOMAIN_SURFACE, TRIGGER_ORCHESTRATOR_KERNEL_CONTRACT_REF,
    TRIGGER_ORCHESTRATOR_KERNEL_SURFACE, TRIGGER_ORCHESTRATOR_USECASE_CONTRACT_REF,
    TRIGGER_ORCHESTRATOR_USECASE_SURFACE, TriggerOrchestratorDecisionStatus,
    TriggerOrchestratorDenialReason, TriggerOrchestratorDomainDenialKind,
    TriggerOrchestratorDomainPolicyBinding, TriggerOrchestratorDomainRequest,
    TriggerOrchestratorDomainSource, TriggerOrchestratorDomainStatus,
    TriggerOrchestratorEventEnvelope, TriggerOrchestratorOverlapPolicy,
    TriggerOrchestratorPolicyContext, TriggerOrchestratorRequest,
    TriggerOrchestratorScheduleMetadata, TriggerOrchestratorTriggerKind,
    TriggerOrchestratorUsecase, TriggerOrchestratorUsecaseInput, TriggerOrchestratorUsecaseReceipt,
    TriggerOrchestratorUsecaseStatus, TriggerOrchestratorWebhookMetadata,
};

pub const TRIGGER_ORCHESTRATOR_API_SURFACE: &str = "workflow-engine.trigger-orchestrator.command";
pub const TRIGGER_ORCHESTRATOR_API_DECLARED_VERSION: &str = "2026-05-25";
pub const TRIGGER_ORCHESTRATOR_API_CONTRACT_REF: &str =
    "workflow/workflow-engine/contracts/openapi/workflow-engine.yaml";
pub const TRIGGER_ORCHESTRATOR_API_ROUTE: &str = "/v/2026-05-25/triggers/evaluate";
pub const TRIGGER_ORCHESTRATOR_API_METHOD: &str = "POST";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorApiStatus {
    Accepted,
    BadRequest,
    Forbidden,
    Conflict,
    ServiceUnavailable,
}

impl TriggerOrchestratorApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::ServiceUnavailable => 503,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Accepted => "Accepted",
            Self::BadRequest => "Bad Request",
            Self::Forbidden => "Forbidden",
            Self::Conflict => "Conflict",
            Self::ServiceUnavailable => "Service Unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorApiErrorCode {
    AuthorizationDenied,
    AuthorizationEvidenceInvalid,
    AuthorizationPrincipalMismatch,
    AuthorizationTenantMismatch,
    ContractVersionUnsupported,
    DomainDenied,
    IdempotencyKeyReused,
    MethodNotAllowed,
    RequestIdEmpty,
    RouteMismatch,
    SourceKindMismatch,
    TenantBindingMismatch,
    TenantHeaderEmpty,
    TraceContextInvalid,
    TriggerInvalid,
    UnknownTriggerKind,
    UnknownTriggerSource,
    UnsafeMetadata,
    UsecaseUnavailable,
}

impl TriggerOrchestratorApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AuthorizationDenied => "WORKFLOW_TRIGGER_AUTHORIZATION_DENIED",
            Self::AuthorizationEvidenceInvalid => "WORKFLOW_TRIGGER_AUTHORIZATION_EVIDENCE_INVALID",
            Self::AuthorizationPrincipalMismatch => {
                "WORKFLOW_TRIGGER_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationTenantMismatch => "WORKFLOW_TRIGGER_AUTHORIZATION_TENANT_MISMATCH",
            Self::ContractVersionUnsupported => "WORKFLOW_TRIGGER_CONTRACT_VERSION_UNSUPPORTED",
            Self::DomainDenied => "WORKFLOW_TRIGGER_DOMAIN_DENIED",
            Self::IdempotencyKeyReused => "WORKFLOW_TRIGGER_IDEMPOTENCY_KEY_REUSED",
            Self::MethodNotAllowed => "WORKFLOW_TRIGGER_METHOD_NOT_ALLOWED",
            Self::RequestIdEmpty => "WORKFLOW_TRIGGER_REQUEST_ID_EMPTY",
            Self::RouteMismatch => "WORKFLOW_TRIGGER_ROUTE_MISMATCH",
            Self::SourceKindMismatch => "WORKFLOW_TRIGGER_SOURCE_KIND_MISMATCH",
            Self::TenantBindingMismatch => "WORKFLOW_TRIGGER_TENANT_BINDING_MISMATCH",
            Self::TenantHeaderEmpty => "WORKFLOW_TRIGGER_TENANT_HEADER_EMPTY",
            Self::TraceContextInvalid => "WORKFLOW_TRIGGER_TRACE_CONTEXT_INVALID",
            Self::TriggerInvalid => "WORKFLOW_TRIGGER_INVALID",
            Self::UnknownTriggerKind => "WORKFLOW_TRIGGER_UNKNOWN_KIND",
            Self::UnknownTriggerSource => "WORKFLOW_TRIGGER_UNKNOWN_SOURCE",
            Self::UnsafeMetadata => "WORKFLOW_TRIGGER_UNSAFE_METADATA",
            Self::UsecaseUnavailable => "WORKFLOW_TRIGGER_USECASE_UNAVAILABLE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiBoundaryContext {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String, // data_class: INTERNAL_ONLY
    pub oyatie_version: String,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub evidence_ref: String,          // data_class: INTERNAL_ONLY
    pub policy_bundle_ref: String,     // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiRequest {
    pub boundary: TriggerOrchestratorApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: TriggerOrchestratorApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: TriggerOrchestratorApiAuthorization, // data_class: INTERNAL_ONLY
    pub method: String,                                  // data_class: PUBLIC
    pub route: String,                                   // data_class: PUBLIC
    pub body: TriggerOrchestratorApiTriggerBody,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiTriggerBody {
    pub source: String,                                      // data_class: PUBLIC
    pub trigger_kind: String,                                // data_class: PUBLIC
    pub trigger_id: String,                                  // data_class: INTERNAL_ONLY
    pub workflow_spec_id: String,                            // data_class: INTERNAL_ONLY
    pub version_sha: String,                                 // data_class: INTERNAL_ONLY
    pub active_cell_id: String,                              // data_class: INTERNAL_ONLY
    pub trigger_lineage_ref: String,                         // data_class: INTERNAL_ONLY
    pub run_idempotency_key: String,                         // data_class: INTERNAL_ONLY
    pub authorization_surface_ref: String,                   // data_class: INTERNAL_ONLY
    pub source_evidence_ref: String,                         // data_class: INTERNAL_ONLY
    pub scheduler_evidence_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub webhook_auth_evidence_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub event_contract_ref: Option<String>,                  // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,                            // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,                             // data_class: INTERNAL_ONLY
    pub correlation_ref: String,                             // data_class: INTERNAL_ONLY
    pub idempotency_scope_ref: String,                       // data_class: INTERNAL_ONLY
    pub dry_run_reason_ref: Option<String>,                  // data_class: INTERNAL_ONLY
    pub replay_mode: bool,                                   // data_class: PUBLIC
    pub dry_run: bool,                                       // data_class: PUBLIC
    pub schedule: Option<TriggerOrchestratorApiScheduleDto>, // data_class: INTERNAL_ONLY
    pub webhook: Option<TriggerOrchestratorApiWebhookDto>,   // data_class: INTERNAL_ONLY
    pub event: Option<TriggerOrchestratorApiEventDto>,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiScheduleDto {
    pub cron_expr_ref: String,                 // data_class: INTERNAL_ONLY
    pub timezone_ref: String,                  // data_class: INTERNAL_ONLY
    pub due_epoch_seconds: u64,                // data_class: INTERNAL_ONLY
    pub observed_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub catchup_window_seconds: u64,           // data_class: INTERNAL_ONLY
    pub overlap_policy: String,                // data_class: PUBLIC
    pub paused: bool,                          // data_class: PUBLIC
    pub pause_reason_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub last_fired_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiWebhookDto {
    pub endpoint_ref: String,        // data_class: INTERNAL_ONLY
    pub signature_ref: String,       // data_class: INTERNAL_ONLY
    pub nonce_ref: String,           // data_class: INTERNAL_ONLY
    pub hmac_key_ref: String,        // data_class: INTERNAL_ONLY
    pub received_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub expires_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiEventDto {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub source: String,                 // data_class: INTERNAL_ONLY
    pub event_type: String,             // data_class: INTERNAL_ONLY
    pub specversion: String,            // data_class: PUBLIC
    pub subject_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub event_time_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub correlation_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiSuccessResponse {
    pub status: TriggerOrchestratorApiStatus, // data_class: PUBLIC
    pub route: String,                        // data_class: PUBLIC
    pub trigger: TriggerOrchestratorTriggerDto, // data_class: INTERNAL_ONLY
    pub metadata: TriggerOrchestratorApiResponseMetadata, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

impl TriggerOrchestratorApiSuccessResponse {
    pub fn http_status_code(&self) -> u16 {
        self.status.code()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorTriggerDto {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub trigger_id: String,                             // data_class: INTERNAL_ONLY
    pub workflow_spec_id: String,                       // data_class: INTERNAL_ONLY
    pub usecase_status: String,                         // data_class: PUBLIC
    pub domain_status: Option<String>,                  // data_class: PUBLIC
    pub kernel_status: Option<String>,                  // data_class: PUBLIC
    pub dispatch_required: bool,                        // data_class: PUBLIC
    pub run_idempotency_key: Option<String>,            // data_class: INTERNAL_ONLY
    pub start_run_command_ref: Option<String>,          // data_class: INTERNAL_ONLY
    pub schedule_next_check_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiResponseMetadata {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String, // data_class: INTERNAL_ONLY
    pub surface: String,           // data_class: INTERNAL_ONLY
    pub contract_ref: String,      // data_class: INTERNAL_ONLY
    pub oyatie_version: String,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorApiProblemDetails {
    pub type_ref: String,           // data_class: PUBLIC
    pub status: u16,                // data_class: PUBLIC
    pub code: String,               // data_class: PUBLIC
    pub title: String,              // data_class: PUBLIC
    pub detail_ref: String,         // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TriggerOrchestratorApiError {
    Boundary {
        code: TriggerOrchestratorApiErrorCode,
        status: TriggerOrchestratorApiStatus,
        evidence_ref: String,
    },
    IdempotencyConflict,
    UsecaseDenied {
        status: TriggerOrchestratorApiStatus,
        code: TriggerOrchestratorApiErrorCode,
        evidence_refs: Vec<String>,
    },
}

impl TriggerOrchestratorApiError {
    pub fn status(&self) -> TriggerOrchestratorApiStatus {
        match self {
            Self::Boundary { status, .. } | Self::UsecaseDenied { status, .. } => *status,
            Self::IdempotencyConflict => TriggerOrchestratorApiStatus::Conflict,
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status().code()
    }

    pub fn code(&self) -> TriggerOrchestratorApiErrorCode {
        match self {
            Self::Boundary { code, .. } | Self::UsecaseDenied { code, .. } => *code,
            Self::IdempotencyConflict => TriggerOrchestratorApiErrorCode::IdempotencyKeyReused,
        }
    }

    pub fn problem(&self) -> TriggerOrchestratorApiProblemDetails {
        let status = self.status();
        let code = self.code();
        let evidence_refs = match self {
            Self::Boundary { evidence_ref, .. } => vec![evidence_ref.clone()],
            Self::UsecaseDenied { evidence_refs, .. } => sorted_unique(evidence_refs.clone()),
            Self::IdempotencyConflict => {
                vec!["trigger-orchestrator-api:idempotency-conflict".to_owned()]
            }
        };
        TriggerOrchestratorApiProblemDetails {
            type_ref: format!("problem:workflow-trigger:{}", code.as_str()),
            status: status.code(),
            code: code.as_str().to_owned(),
            title: status.title().to_owned(),
            detail_ref: format!("detail:workflow-trigger:{}", code.as_str()),
            evidence_refs,
        }
    }
}

#[derive(Default)]
pub struct WorkflowTriggerOrchestratorApi {
    usecase: TriggerOrchestratorUsecase,
    responses_by_idempotency: BTreeMap<String, (String, TriggerOrchestratorApiSuccessResponse)>,
}

impl WorkflowTriggerOrchestratorApi {
    pub fn apply_trigger(
        &mut self,
        request: TriggerOrchestratorApiRequest,
    ) -> Result<TriggerOrchestratorApiSuccessResponse, TriggerOrchestratorApiError> {
        validate_boundary(&request)?;
        validate_body_metadata(&request)?;
        let source = parse_source(&request.body.source)?;
        let kind = parse_trigger_kind(&request.body.trigger_kind)?;
        validate_source_kind(source, kind)?;
        let fingerprint = request_fingerprint(&request);
        let cache_key = idempotency_cache_key(&request);
        if let Some((cached_fingerprint, response)) = self.responses_by_idempotency.get(&cache_key)
        {
            if cached_fingerprint == &fingerprint {
                return Ok(response.clone());
            }
            return Err(TriggerOrchestratorApiError::IdempotencyConflict);
        }

        let input = TriggerOrchestratorUsecaseInput {
            request_id: request.boundary.request_id.clone(),
            idempotency_key: request.boundary.idempotency_key.clone(),
            trace_ref: request.boundary.trace_context_ref.clone(),
            domain_request: to_domain_request(&request, source, kind)?,
        };
        let receipt = self.usecase.apply(input);
        let response = map_usecase_receipt(&request, receipt)?;
        self.responses_by_idempotency
            .insert(cache_key, (fingerprint, response.clone()));
        Ok(response)
    }
}

fn to_domain_request(
    request: &TriggerOrchestratorApiRequest,
    source: TriggerOrchestratorDomainSource,
    kind: TriggerOrchestratorTriggerKind,
) -> Result<TriggerOrchestratorDomainRequest, TriggerOrchestratorApiError> {
    let body = &request.body;
    let trigger_request = TriggerOrchestratorRequest {
        kind,
        context: TriggerOrchestratorPolicyContext {
            tenant_id: request.boundary.tenant_id.clone(),
            trigger_id: body.trigger_id.clone(),
            workflow_spec_id: body.workflow_spec_id.clone(),
            version_sha: body.version_sha.clone(),
            active_cell_id: body.active_cell_id.clone(),
            principal_id: request.principal.principal_id.clone(),
            policy_decision_id: request.authorization.decision_id.clone(),
            policy_evidence_ref: request.authorization.evidence_ref.clone(),
            replay_epoch_ref: body.replay_epoch_ref.clone(),
        },
        schedule: body.schedule.as_ref().map(to_schedule).transpose()?,
        webhook: body.webhook.as_ref().map(to_webhook),
        event: body.event.as_ref().map(to_event),
        trigger_lineage_ref: body.trigger_lineage_ref.clone(),
        idempotency_key: body.run_idempotency_key.clone(),
        replay_mode: body.replay_mode,
        dry_run: body.dry_run,
        evidence_refs: body.evidence_refs.clone(),
    };
    Ok(TriggerOrchestratorDomainRequest {
        binding: TriggerOrchestratorDomainPolicyBinding {
            tenant_id: request.boundary.tenant_id.clone(),
            trigger_id: body.trigger_id.clone(),
            workflow_spec_id: body.workflow_spec_id.clone(),
            version_sha: body.version_sha.clone(),
            active_cell_id: body.active_cell_id.clone(),
            principal_id: request.principal.principal_id.clone(),
            allowed_kind: kind,
            policy_decision_id: request.authorization.decision_id.clone(),
            policy_evidence_ref: request.authorization.evidence_ref.clone(),
            policy_bundle_ref: request.authorization.policy_bundle_ref.clone(),
            authorization_surface_ref: body.authorization_surface_ref.clone(),
            source_evidence_ref: body.source_evidence_ref.clone(),
            scheduler_evidence_ref: body.scheduler_evidence_ref.clone(),
            webhook_auth_evidence_ref: body.webhook_auth_evidence_ref.clone(),
            event_contract_ref: body.event_contract_ref.clone(),
            replay_epoch_ref: body.replay_epoch_ref.clone(),
            audit_chain_ref: body.audit_chain_ref.clone(),
        },
        trigger_request,
        source,
        trace_ref: request.boundary.trace_context_ref.clone(),
        correlation_ref: body.correlation_ref.clone(),
        idempotency_scope_ref: body.idempotency_scope_ref.clone(),
        dry_run_reason_ref: body.dry_run_reason_ref.clone(),
        evidence_refs: sorted_unique(
            [
                body.evidence_refs.clone(),
                vec![format!("api-surface:{}", TRIGGER_ORCHESTRATOR_API_SURFACE)],
            ]
            .concat(),
        ),
    })
}

fn to_schedule(
    dto: &TriggerOrchestratorApiScheduleDto,
) -> Result<TriggerOrchestratorScheduleMetadata, TriggerOrchestratorApiError> {
    Ok(TriggerOrchestratorScheduleMetadata {
        cron_expr_ref: dto.cron_expr_ref.clone(),
        timezone_ref: dto.timezone_ref.clone(),
        due_epoch_seconds: dto.due_epoch_seconds,
        observed_epoch_seconds: dto.observed_epoch_seconds,
        catchup_window_seconds: dto.catchup_window_seconds,
        overlap_policy: parse_overlap_policy(&dto.overlap_policy)?,
        paused: dto.paused,
        pause_reason_ref: dto.pause_reason_ref.clone(),
        last_fired_epoch_seconds: dto.last_fired_epoch_seconds,
    })
}

fn to_webhook(dto: &TriggerOrchestratorApiWebhookDto) -> TriggerOrchestratorWebhookMetadata {
    TriggerOrchestratorWebhookMetadata {
        endpoint_ref: dto.endpoint_ref.clone(),
        signature_ref: dto.signature_ref.clone(),
        nonce_ref: dto.nonce_ref.clone(),
        hmac_key_ref: dto.hmac_key_ref.clone(),
        received_epoch_seconds: dto.received_epoch_seconds,
        expires_epoch_seconds: dto.expires_epoch_seconds,
    }
}

fn to_event(dto: &TriggerOrchestratorApiEventDto) -> TriggerOrchestratorEventEnvelope {
    TriggerOrchestratorEventEnvelope {
        event_id: dto.event_id.clone(),
        source: dto.source.clone(),
        event_type: dto.event_type.clone(),
        specversion: dto.specversion.clone(),
        subject_ref: dto.subject_ref.clone(),
        event_time_ref: dto.event_time_ref.clone(),
        correlation_id: dto.correlation_id.clone(),
        idempotency_key: dto.idempotency_key.clone(),
    }
}

fn map_usecase_receipt(
    request: &TriggerOrchestratorApiRequest,
    receipt: TriggerOrchestratorUsecaseReceipt,
) -> Result<TriggerOrchestratorApiSuccessResponse, TriggerOrchestratorApiError> {
    match receipt.status {
        TriggerOrchestratorUsecaseStatus::Accepted
        | TriggerOrchestratorUsecaseStatus::Deferred
        | TriggerOrchestratorUsecaseStatus::Suppressed => {
            let mut evidence_refs = receipt.evidence_refs.clone();
            evidence_refs.push(TRIGGER_ORCHESTRATOR_API_SURFACE.to_owned());
            Ok(TriggerOrchestratorApiSuccessResponse {
                status: TriggerOrchestratorApiStatus::Accepted,
                route: TRIGGER_ORCHESTRATOR_API_ROUTE.to_owned(),
                trigger: TriggerOrchestratorTriggerDto::from_receipt(&receipt),
                metadata: response_metadata(request),
                evidence_refs: sorted_unique(evidence_refs),
                non_claim_refs: sorted_unique(receipt.non_claim_refs),
            })
        }
        TriggerOrchestratorUsecaseStatus::DomainDenied => {
            Err(TriggerOrchestratorApiError::UsecaseDenied {
                status: TriggerOrchestratorApiStatus::Forbidden,
                code: TriggerOrchestratorApiErrorCode::DomainDenied,
                evidence_refs: sorted_unique(receipt.evidence_refs),
            })
        }
        TriggerOrchestratorUsecaseStatus::InvalidInput => {
            Err(TriggerOrchestratorApiError::UsecaseDenied {
                status: TriggerOrchestratorApiStatus::BadRequest,
                code: TriggerOrchestratorApiErrorCode::TriggerInvalid,
                evidence_refs: sorted_unique(receipt.evidence_refs),
            })
        }
        TriggerOrchestratorUsecaseStatus::IdempotencyConflict => {
            Err(TriggerOrchestratorApiError::IdempotencyConflict)
        }
    }
}

impl TriggerOrchestratorTriggerDto {
    fn from_receipt(receipt: &TriggerOrchestratorUsecaseReceipt) -> Self {
        Self {
            tenant_id: receipt.tenant_id.clone(),
            trigger_id: receipt.trigger_id.clone(),
            workflow_spec_id: receipt.workflow_spec_id.clone(),
            usecase_status: receipt.status.as_wire().to_owned(),
            domain_status: receipt
                .domain_status
                .map(domain_status_label)
                .map(str::to_owned),
            kernel_status: receipt
                .kernel_status
                .map(kernel_status_label)
                .map(str::to_owned),
            dispatch_required: receipt.dispatch_required,
            run_idempotency_key: receipt.run_idempotency_key.clone(),
            start_run_command_ref: receipt.start_run_command_ref.clone(),
            schedule_next_check_epoch_seconds: receipt.schedule_next_check_epoch_seconds,
        }
    }
}

fn validate_boundary(
    request: &TriggerOrchestratorApiRequest,
) -> Result<(), TriggerOrchestratorApiError> {
    if request.boundary.request_id.trim().is_empty() {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::RequestIdEmpty,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:request-id-required",
        ));
    }
    if request.boundary.tenant_id.trim().is_empty() {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::TenantHeaderEmpty,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:tenant-required",
        ));
    }
    if request.boundary.oyatie_version != TRIGGER_ORCHESTRATOR_API_DECLARED_VERSION {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::ContractVersionUnsupported,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:unsupported-version",
        ));
    }
    if request.method != TRIGGER_ORCHESTRATOR_API_METHOD {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::MethodNotAllowed,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:method-not-allowed",
        ));
    }
    if request.route != TRIGGER_ORCHESTRATOR_API_ROUTE {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::RouteMismatch,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:route-mismatch",
        ));
    }
    if !is_safe_ref(&request.boundary.request_id)
        || !is_safe_tenant(&request.boundary.tenant_id)
        || !is_safe_ref(&request.boundary.idempotency_key)
    {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::UnsafeMetadata,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:unsafe-boundary-metadata",
        ));
    }
    if !is_safe_ref(&request.boundary.trace_context_ref) {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::TraceContextInvalid,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:trace-context-invalid",
        ));
    }
    if request.principal.tenant_id != request.boundary.tenant_id {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::TenantBindingMismatch,
            TriggerOrchestratorApiStatus::Forbidden,
            "workflow-trigger-api:principal-tenant-mismatch",
        ));
    }
    if request.authorization.tenant_id != request.boundary.tenant_id {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::AuthorizationTenantMismatch,
            TriggerOrchestratorApiStatus::Forbidden,
            "workflow-trigger-api:auth-tenant-mismatch",
        ));
    }
    if request.authorization.principal_id != request.principal.principal_id {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::AuthorizationPrincipalMismatch,
            TriggerOrchestratorApiStatus::Forbidden,
            "workflow-trigger-api:auth-principal-mismatch",
        ));
    }
    if !is_safe_ref(&request.principal.principal_id)
        || !is_safe_ref(&request.authorization.decision_id)
        || !is_safe_ref(&request.authorization.evidence_ref)
        || !is_safe_ref(&request.authorization.policy_bundle_ref)
    {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::AuthorizationEvidenceInvalid,
            TriggerOrchestratorApiStatus::Forbidden,
            "workflow-trigger-api:auth-evidence-invalid",
        ));
    }
    if !request
        .authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == TRIGGER_ORCHESTRATOR_API_SURFACE)
    {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::AuthorizationDenied,
            TriggerOrchestratorApiStatus::Forbidden,
            "workflow-trigger-api:surface-denied",
        ));
    }
    Ok(())
}

fn validate_body_metadata(
    request: &TriggerOrchestratorApiRequest,
) -> Result<(), TriggerOrchestratorApiError> {
    let body = &request.body;
    let invalid = !is_safe_ref(&body.trigger_id)
        || !is_safe_ref(&body.workflow_spec_id)
        || !is_safe_ref(&body.version_sha)
        || !is_safe_ref(&body.active_cell_id)
        || !is_safe_ref(&body.trigger_lineage_ref)
        || !is_safe_ref(&body.run_idempotency_key)
        || !is_safe_ref(&body.authorization_surface_ref)
        || !is_safe_ref(&body.source_evidence_ref)
        || !is_safe_optional_ref(body.scheduler_evidence_ref.as_deref())
        || !is_safe_optional_ref(body.webhook_auth_evidence_ref.as_deref())
        || !is_safe_optional_ref(body.event_contract_ref.as_deref())
        || !is_safe_ref(&body.replay_epoch_ref)
        || !is_safe_ref(&body.audit_chain_ref)
        || !is_safe_ref(&body.correlation_ref)
        || !is_safe_ref(&body.idempotency_scope_ref)
        || !is_safe_optional_ref(body.dry_run_reason_ref.as_deref())
        || !body.evidence_refs.iter().all(|value| is_safe_ref(value))
        || body
            .schedule
            .as_ref()
            .is_some_and(schedule_has_unsafe_metadata)
        || body
            .webhook
            .as_ref()
            .is_some_and(webhook_has_unsafe_metadata)
        || body.event.as_ref().is_some_and(event_has_unsafe_metadata);
    if invalid {
        return Err(boundary_error(
            TriggerOrchestratorApiErrorCode::UnsafeMetadata,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:unsafe-body-metadata",
        ));
    }
    Ok(())
}

fn schedule_has_unsafe_metadata(schedule: &TriggerOrchestratorApiScheduleDto) -> bool {
    !is_safe_ref(&schedule.cron_expr_ref)
        || !is_safe_ref(&schedule.timezone_ref)
        || !is_safe_optional_ref(schedule.pause_reason_ref.as_deref())
}

fn webhook_has_unsafe_metadata(webhook: &TriggerOrchestratorApiWebhookDto) -> bool {
    !is_safe_ref(&webhook.endpoint_ref)
        || !is_safe_ref(&webhook.signature_ref)
        || !is_safe_ref(&webhook.nonce_ref)
        || !is_safe_ref(&webhook.hmac_key_ref)
}

fn event_has_unsafe_metadata(event: &TriggerOrchestratorApiEventDto) -> bool {
    !is_safe_ref(&event.event_id)
        || !is_safe_ref(&event.source)
        || !is_safe_metadata(&event.event_type)
        || !is_safe_metadata(&event.specversion)
        || !is_safe_optional_ref(event.subject_ref.as_deref())
        || !is_safe_optional_ref(event.event_time_ref.as_deref())
        || !is_safe_ref(&event.correlation_id)
        || !is_safe_ref(&event.idempotency_key)
}

fn parse_source(
    value: &str,
) -> Result<TriggerOrchestratorDomainSource, TriggerOrchestratorApiError> {
    match value {
        "scheduler" => Ok(TriggerOrchestratorDomainSource::Scheduler),
        "studio-webhook" => Ok(TriggerOrchestratorDomainSource::StudioWebhook),
        "sibling-event-bus" => Ok(TriggerOrchestratorDomainSource::SiblingEventBus),
        "manual-ui" => Ok(TriggerOrchestratorDomainSource::ManualUi),
        "api-command" => Ok(TriggerOrchestratorDomainSource::ApiCommand),
        "workflow-spawn" => Ok(TriggerOrchestratorDomainSource::WorkflowSpawn),
        "ontology-projection" => Ok(TriggerOrchestratorDomainSource::OntologyProjection),
        _ => Err(boundary_error(
            TriggerOrchestratorApiErrorCode::UnknownTriggerSource,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:unknown-source",
        )),
    }
}

fn parse_trigger_kind(
    value: &str,
) -> Result<TriggerOrchestratorTriggerKind, TriggerOrchestratorApiError> {
    match value {
        "cron" => Ok(TriggerOrchestratorTriggerKind::Cron),
        "webhook" => Ok(TriggerOrchestratorTriggerKind::Webhook),
        "event-bus" => Ok(TriggerOrchestratorTriggerKind::EventBus),
        "manual" => Ok(TriggerOrchestratorTriggerKind::Manual),
        "api" => Ok(TriggerOrchestratorTriggerKind::Api),
        "workflow-spawn" => Ok(TriggerOrchestratorTriggerKind::WorkflowSpawn),
        "ontology" => Ok(TriggerOrchestratorTriggerKind::Ontology),
        _ => Err(boundary_error(
            TriggerOrchestratorApiErrorCode::UnknownTriggerKind,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:unknown-trigger-kind",
        )),
    }
}

fn validate_source_kind(
    source: TriggerOrchestratorDomainSource,
    kind: TriggerOrchestratorTriggerKind,
) -> Result<(), TriggerOrchestratorApiError> {
    if source.expected_kind() == kind {
        Ok(())
    } else {
        Err(boundary_error(
            TriggerOrchestratorApiErrorCode::SourceKindMismatch,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:source-kind-mismatch",
        ))
    }
}

fn parse_overlap_policy(
    value: &str,
) -> Result<TriggerOrchestratorOverlapPolicy, TriggerOrchestratorApiError> {
    match value {
        "skip" => Ok(TriggerOrchestratorOverlapPolicy::Skip),
        "buffer-one" => Ok(TriggerOrchestratorOverlapPolicy::BufferOne),
        "buffer-all" => Ok(TriggerOrchestratorOverlapPolicy::BufferAll),
        "cancel-other" => Ok(TriggerOrchestratorOverlapPolicy::CancelOther),
        "terminate-other" => Ok(TriggerOrchestratorOverlapPolicy::TerminateOther),
        "allow-all" => Ok(TriggerOrchestratorOverlapPolicy::AllowAll),
        _ => Err(boundary_error(
            TriggerOrchestratorApiErrorCode::TriggerInvalid,
            TriggerOrchestratorApiStatus::BadRequest,
            "workflow-trigger-api:unknown-overlap-policy",
        )),
    }
}

fn response_metadata(
    request: &TriggerOrchestratorApiRequest,
) -> TriggerOrchestratorApiResponseMetadata {
    TriggerOrchestratorApiResponseMetadata {
        request_id: request.boundary.request_id.clone(),
        tenant_id: request.boundary.tenant_id.clone(),
        idempotency_key: request.boundary.idempotency_key.clone(),
        trace_context_ref: request.boundary.trace_context_ref.clone(),
        surface: TRIGGER_ORCHESTRATOR_API_SURFACE.to_owned(),
        contract_ref: TRIGGER_ORCHESTRATOR_API_CONTRACT_REF.to_owned(),
        oyatie_version: TRIGGER_ORCHESTRATOR_API_DECLARED_VERSION.to_owned(),
    }
}

fn request_fingerprint(request: &TriggerOrchestratorApiRequest) -> String {
    format!("{request:?}")
}

fn idempotency_cache_key(request: &TriggerOrchestratorApiRequest) -> String {
    format!(
        "{}|{}",
        request.boundary.tenant_id, request.boundary.idempotency_key
    )
}

fn boundary_error(
    code: TriggerOrchestratorApiErrorCode,
    status: TriggerOrchestratorApiStatus,
    evidence_ref: &str,
) -> TriggerOrchestratorApiError {
    TriggerOrchestratorApiError::Boundary {
        code,
        status,
        evidence_ref: evidence_ref.to_owned(),
    }
}

fn domain_status_label(status: TriggerOrchestratorDomainStatus) -> &'static str {
    match status {
        TriggerOrchestratorDomainStatus::Accepted => "accepted",
        TriggerOrchestratorDomainStatus::Denied => "denied",
        TriggerOrchestratorDomainStatus::Deferred => "deferred",
        TriggerOrchestratorDomainStatus::Suppressed => "suppressed",
    }
}

fn kernel_status_label(status: TriggerOrchestratorDecisionStatus) -> &'static str {
    match status {
        TriggerOrchestratorDecisionStatus::Accepted => "accepted",
        TriggerOrchestratorDecisionStatus::Denied => "denied",
        TriggerOrchestratorDecisionStatus::Deferred => "deferred",
        TriggerOrchestratorDecisionStatus::Suppressed => "suppressed",
    }
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
}

fn is_safe_optional_ref(value: Option<&str>) -> bool {
    value.is_none_or(is_safe_ref)
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && !value.chars().any(char::is_whitespace)
        && !contains_raw_secret_material(value)
        && !contains_raw_content_material(value)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("private key")
        || lower.contains("-----begin")
        || lower.contains("secret=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
        || lower.contains("payload")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn api_status_and_error_codes_are_stable_and_unique() {
        let statuses = [
            TriggerOrchestratorApiStatus::Accepted,
            TriggerOrchestratorApiStatus::BadRequest,
            TriggerOrchestratorApiStatus::Forbidden,
            TriggerOrchestratorApiStatus::Conflict,
            TriggerOrchestratorApiStatus::ServiceUnavailable,
        ];
        assert_eq!(statuses[0].code(), 202);
        let status_codes: BTreeSet<u16> = statuses.iter().map(|status| status.code()).collect();
        assert_eq!(status_codes.len(), statuses.len());

        let codes = [
            TriggerOrchestratorApiErrorCode::AuthorizationDenied,
            TriggerOrchestratorApiErrorCode::AuthorizationEvidenceInvalid,
            TriggerOrchestratorApiErrorCode::AuthorizationPrincipalMismatch,
            TriggerOrchestratorApiErrorCode::AuthorizationTenantMismatch,
            TriggerOrchestratorApiErrorCode::ContractVersionUnsupported,
            TriggerOrchestratorApiErrorCode::DomainDenied,
            TriggerOrchestratorApiErrorCode::IdempotencyKeyReused,
            TriggerOrchestratorApiErrorCode::MethodNotAllowed,
            TriggerOrchestratorApiErrorCode::RequestIdEmpty,
            TriggerOrchestratorApiErrorCode::RouteMismatch,
            TriggerOrchestratorApiErrorCode::SourceKindMismatch,
            TriggerOrchestratorApiErrorCode::TenantBindingMismatch,
            TriggerOrchestratorApiErrorCode::TenantHeaderEmpty,
            TriggerOrchestratorApiErrorCode::TraceContextInvalid,
            TriggerOrchestratorApiErrorCode::TriggerInvalid,
            TriggerOrchestratorApiErrorCode::UnknownTriggerKind,
            TriggerOrchestratorApiErrorCode::UnknownTriggerSource,
            TriggerOrchestratorApiErrorCode::UnsafeMetadata,
            TriggerOrchestratorApiErrorCode::UsecaseUnavailable,
        ];
        let wire: Vec<&str> = codes.iter().map(|code| code.as_str()).collect();
        let unique: BTreeSet<&str> = wire.iter().copied().collect();
        assert_eq!(wire.len(), unique.len());
    }

    #[test]
    fn accepts_authorized_scheduler_trigger_and_returns_metadata_only_response() {
        let mut api = WorkflowTriggerOrchestratorApi::default();
        let response = api
            .apply_trigger(authorized_request("idem:api:1", "scheduler", "cron"))
            .expect("accepted");

        assert_eq!(response.status, TriggerOrchestratorApiStatus::Accepted);
        assert_eq!(response.http_status_code(), 202);
        assert_eq!(response.route, TRIGGER_ORCHESTRATOR_API_ROUTE);
        assert_eq!(response.trigger.usecase_status, "accepted");
        assert_eq!(response.trigger.domain_status.as_deref(), Some("accepted"));
        assert_eq!(response.trigger.kernel_status.as_deref(), Some("accepted"));
        assert!(response.trigger.dispatch_required);
        assert_eq!(response.metadata.surface, TRIGGER_ORCHESTRATOR_API_SURFACE);
        assert!(
            response
                .evidence_refs
                .contains(&TRIGGER_ORCHESTRATOR_API_SURFACE.to_owned())
        );
        assert!(
            response
                .non_claim_refs
                .contains(&"no-run-creation".to_owned())
        );
    }

    #[test]
    fn idempotent_replay_returns_cached_response_and_drift_conflicts_without_redelegating() {
        let mut api = WorkflowTriggerOrchestratorApi::default();
        let first_request = authorized_request("idem:api:replay", "scheduler", "cron");
        let first = api.apply_trigger(first_request.clone()).unwrap();
        let second = api.apply_trigger(first_request).unwrap();
        assert_eq!(first, second);

        let mut drifted = authorized_request("idem:api:replay", "scheduler", "cron");
        drifted.body.workflow_spec_id = "workflow:other".to_owned();
        let error = api.apply_trigger(drifted).expect_err("conflict");
        assert_eq!(error.status(), TriggerOrchestratorApiStatus::Conflict);
        assert_eq!(
            error.code(),
            TriggerOrchestratorApiErrorCode::IdempotencyKeyReused
        );
    }

    #[test]
    fn boundary_route_version_and_authorization_fail_before_trigger_evaluation() {
        let mut api = WorkflowTriggerOrchestratorApi::default();
        let mut version = authorized_request("idem:api:bad-version", "scheduler", "cron");
        version.boundary.oyatie_version = "2020-01-01".to_owned();
        assert_eq!(
            api.apply_trigger(version).unwrap_err().code(),
            TriggerOrchestratorApiErrorCode::ContractVersionUnsupported
        );

        let mut route = authorized_request("idem:api:bad-route", "scheduler", "cron");
        route.route = "/wrong".to_owned();
        assert_eq!(
            api.apply_trigger(route).unwrap_err().code(),
            TriggerOrchestratorApiErrorCode::RouteMismatch
        );

        let mut denied = authorized_request("idem:api:denied", "scheduler", "cron");
        denied.authorization.allowed_surfaces.clear();
        assert_eq!(
            api.apply_trigger(denied).unwrap_err().code(),
            TriggerOrchestratorApiErrorCode::AuthorizationDenied
        );
    }

    #[test]
    fn unknown_source_kind_and_mismatch_return_stable_problem_details() {
        let mut api = WorkflowTriggerOrchestratorApi::default();
        let unknown = api
            .apply_trigger(authorized_request("idem:api:unknown", "scheduler", "bogus"))
            .unwrap_err();
        let problem = unknown.problem();
        assert_eq!(problem.status, 400);
        assert_eq!(
            problem.code,
            TriggerOrchestratorApiErrorCode::UnknownTriggerKind.as_str()
        );
        assert!(problem.detail_ref.contains("WORKFLOW_TRIGGER_UNKNOWN_KIND"));

        let mismatch = api
            .apply_trigger(authorized_request(
                "idem:api:mismatch",
                "scheduler",
                "webhook",
            ))
            .unwrap_err();
        assert_eq!(
            mismatch.code(),
            TriggerOrchestratorApiErrorCode::SourceKindMismatch
        );
    }

    #[test]
    fn domain_denial_maps_to_forbidden_without_raw_echo() {
        let mut api = WorkflowTriggerOrchestratorApi::default();
        let mut request = authorized_request("idem:api:domain-denied", "scheduler", "cron");
        request.body.scheduler_evidence_ref = None;

        let error = api.apply_trigger(request).unwrap_err();
        assert_eq!(error.status(), TriggerOrchestratorApiStatus::Forbidden);
        assert_eq!(error.code(), TriggerOrchestratorApiErrorCode::DomainDenied);
        let problem = error.problem();
        assert!(
            problem
                .evidence_refs
                .contains(&"validation:scheduler-evidence-required".to_owned())
        );
        assert!(!format!("{problem:?}").contains("raw prompt"));
    }

    #[test]
    fn deferred_suppressed_and_webhook_event_sources_map_to_api_success_without_runtime_claims() {
        let mut api = WorkflowTriggerOrchestratorApi::default();
        let mut paused = authorized_request("idem:api:paused", "scheduler", "cron");
        paused.body.schedule.as_mut().unwrap().paused = true;
        paused.body.schedule.as_mut().unwrap().pause_reason_ref =
            Some("pause:maintenance".to_owned());
        let deferred = api.apply_trigger(paused).unwrap();
        assert_eq!(deferred.trigger.usecase_status, "deferred");
        assert!(!deferred.trigger.dispatch_required);

        let mut replay =
            authorized_request("idem:api:replay-mode", "sibling-event-bus", "event-bus");
        replay.body.schedule = None;
        replay.body.event = Some(event_valid());
        replay.body.scheduler_evidence_ref = None;
        replay.body.event_contract_ref = Some("event-contract:cloudevents-v1".to_owned());
        replay.body.replay_mode = true;
        let suppressed = api.apply_trigger(replay).unwrap();
        assert_eq!(suppressed.trigger.usecase_status, "suppressed");
        assert!(!suppressed.trigger.dispatch_required);

        let mut webhook = authorized_request("idem:api:webhook", "studio-webhook", "webhook");
        webhook.body.schedule = None;
        webhook.body.webhook = Some(webhook_valid());
        webhook.body.scheduler_evidence_ref = None;
        webhook.body.webhook_auth_evidence_ref = Some("webhook-auth:hmac-nonce-bound".to_owned());
        let accepted = api.apply_trigger(webhook).unwrap();
        assert_eq!(accepted.trigger.usecase_status, "accepted");
        assert!(
            accepted
                .non_claim_refs
                .contains(&"no-webhook-server".to_owned())
        );
    }

    #[test]
    fn unsafe_boundary_or_body_metadata_is_rejected_without_echo() {
        let mut api = WorkflowTriggerOrchestratorApi::default();
        let mut request = authorized_request("idem:api:unsafe", "scheduler", "cron");
        request.body.correlation_ref =
            "corr:raw prompt Authorization: Bearer sk-test payload".to_owned();

        let error = api.apply_trigger(request).unwrap_err();
        assert_eq!(error.status(), TriggerOrchestratorApiStatus::BadRequest);
        assert_eq!(
            error.code(),
            TriggerOrchestratorApiErrorCode::UnsafeMetadata
        );
        let rendered = format!("{:?}", error.problem());
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("payload"));
    }

    fn authorized_request(
        idempotency_key: &str,
        source: &str,
        kind: &str,
    ) -> TriggerOrchestratorApiRequest {
        TriggerOrchestratorApiRequest {
            boundary: TriggerOrchestratorApiBoundaryContext {
                request_id: format!("request:trigger-api:{idempotency_key}"),
                tenant_id: "ten_foundry".to_owned(),
                idempotency_key: idempotency_key.to_owned(),
                trace_context_ref: "trace:trigger-api".to_owned(),
                oyatie_version: TRIGGER_ORCHESTRATOR_API_DECLARED_VERSION.to_owned(),
            },
            principal: TriggerOrchestratorApiPrincipal {
                tenant_id: "ten_foundry".to_owned(),
                principal_id: "principal:workflow-operator".to_owned(),
            },
            authorization: TriggerOrchestratorApiAuthorization {
                tenant_id: "ten_foundry".to_owned(),
                principal_id: "principal:workflow-operator".to_owned(),
                decision_id: "policy-decision:allow-trigger".to_owned(),
                evidence_ref: "policy-evidence:cedar-allow".to_owned(),
                policy_bundle_ref: "policy-bundle:trigger-v1".to_owned(),
                allowed_surfaces: vec![TRIGGER_ORCHESTRATOR_API_SURFACE.to_owned()],
            },
            method: TRIGGER_ORCHESTRATOR_API_METHOD.to_owned(),
            route: TRIGGER_ORCHESTRATOR_API_ROUTE.to_owned(),
            body: TriggerOrchestratorApiTriggerBody {
                source: source.to_owned(),
                trigger_kind: kind.to_owned(),
                trigger_id: "trigger:daily-invoice".to_owned(),
                workflow_spec_id: "workflow:invoice-approval".to_owned(),
                version_sha: "sha:abc123".to_owned(),
                active_cell_id: "cell:use1-a".to_owned(),
                trigger_lineage_ref: "lineage:trigger-parent".to_owned(),
                run_idempotency_key: "idem:trigger-run".to_owned(),
                authorization_surface_ref: "authz-surface:trigger-admission".to_owned(),
                source_evidence_ref: "source-evidence:trigger-admission".to_owned(),
                scheduler_evidence_ref: Some("scheduler:durable-clock-window".to_owned()),
                webhook_auth_evidence_ref: None,
                event_contract_ref: None,
                replay_epoch_ref: "replay-epoch:2026-05-25T000000Z".to_owned(),
                audit_chain_ref: "audit-chain:trigger-api".to_owned(),
                correlation_ref: "corr:trigger-api".to_owned(),
                idempotency_scope_ref: "idem-scope:tenant-trigger".to_owned(),
                dry_run_reason_ref: None,
                replay_mode: false,
                dry_run: false,
                schedule: Some(schedule_due()),
                webhook: None,
                event: None,
                evidence_refs: vec!["evidence:api-unit-test".to_owned()],
            },
        }
    }

    fn schedule_due() -> TriggerOrchestratorApiScheduleDto {
        TriggerOrchestratorApiScheduleDto {
            cron_expr_ref: "cron:every-hour".to_owned(),
            timezone_ref: "tz:America-New_York".to_owned(),
            due_epoch_seconds: 1_750_000_000,
            observed_epoch_seconds: 1_750_000_008,
            catchup_window_seconds: 10,
            overlap_policy: "buffer-one".to_owned(),
            paused: false,
            pause_reason_ref: None,
            last_fired_epoch_seconds: Some(1_749_996_400),
        }
    }

    fn webhook_valid() -> TriggerOrchestratorApiWebhookDto {
        TriggerOrchestratorApiWebhookDto {
            endpoint_ref: "endpoint:webhook-invoice".to_owned(),
            signature_ref: "signature:webhook-headers".to_owned(),
            nonce_ref: "nonce:webhook-001".to_owned(),
            hmac_key_ref: "hmac-key:webhook-signing".to_owned(),
            received_epoch_seconds: 1_750_000_001,
            expires_epoch_seconds: 1_750_000_061,
        }
    }

    fn event_valid() -> TriggerOrchestratorApiEventDto {
        TriggerOrchestratorApiEventDto {
            event_id: "event:invoice-approved-001".to_owned(),
            source: "https://events.oyatie.example/workflow".to_owned(),
            event_type: "com.oyatie.workflow.invoice_approved".to_owned(),
            specversion: TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION.to_owned(),
            subject_ref: Some("subject:invoice-123".to_owned()),
            event_time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
            correlation_id: "corr:invoice-123".to_owned(),
            idempotency_key: "idem:event-001".to_owned(),
        }
    }
}
