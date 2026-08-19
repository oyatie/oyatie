//! Intelligence assist-draft API boundary foundation.
//!
//! This crate owns the source-level, framework-free API boundary for advisory
//! assist-draft requests. It validates tenant/principal/authorization/request
//! binding, preserves idempotent API responses in memory, delegates
//! metadata-only planning to the assist-draft usecase, and dispatches planned
//! receipts through the metadata-only assist-draft executor adapter seam. It
//! performs no HTTP serving, prompt rendering, model/provider calls, builder
//! mutation, network I/O, durable idempotency storage, durable audit-chain
//! emission, queue processing, or runtime execution.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use intelligence_assist_draft_adapter::*;

pub const ASSIST_DRAFT_API_SURFACE: &str = "intelligence.assist-draft.request";
pub const ASSIST_DRAFT_API_CONTRACT_REF: &str =
    "contracts/openapi/intelligence-assist-draft-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftApiStatus {
    Accepted,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
    FailedDependency,
    TooManyRequests,
    ServiceUnavailable,
    GatewayTimeout,
}

impl AssistDraftApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
            Self::FailedDependency => 424,
            Self::TooManyRequests => 429,
            Self::ServiceUnavailable => 503,
            Self::GatewayTimeout => 504,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    TraceContextInvalid,
    PrincipalIdEmpty,
    AuthorizationDecisionIdEmpty,
    AuthorizationEvidenceInvalid,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    TenantBindingMismatch,
    PrincipalBindingMismatch,
    PolicyBindingMismatch,
    UnsafeMetadata,
    UsecaseDenied,
    IdempotencyKeyReused,
    ExecutorDenied,
    ExecutorRateLimited,
    ExecutorFailed,
    ExecutorAuthFailed,
    ExecutorInvalidRequest,
    ExecutorTimeout,
}

impl AssistDraftApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "ASSIST_DRAFT_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "ASSIST_DRAFT_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "ASSIST_DRAFT_IDEMPOTENCY_KEY_EMPTY",
            Self::TraceContextInvalid => "ASSIST_DRAFT_TRACE_CONTEXT_INVALID",
            Self::PrincipalIdEmpty => "ASSIST_DRAFT_PRINCIPAL_ID_EMPTY",
            Self::AuthorizationDecisionIdEmpty => "ASSIST_DRAFT_AUTH_DECISION_ID_EMPTY",
            Self::AuthorizationEvidenceInvalid => "ASSIST_DRAFT_AUTH_EVIDENCE_INVALID",
            Self::AuthorizationTenantMismatch => "ASSIST_DRAFT_AUTH_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "ASSIST_DRAFT_AUTH_PRINCIPAL_MISMATCH",
            Self::AuthorizationDenied => "ASSIST_DRAFT_AUTHORIZATION_DENIED",
            Self::TenantBindingMismatch => "ASSIST_DRAFT_TENANT_BINDING_MISMATCH",
            Self::PrincipalBindingMismatch => "ASSIST_DRAFT_PRINCIPAL_BINDING_MISMATCH",
            Self::PolicyBindingMismatch => "ASSIST_DRAFT_POLICY_BINDING_MISMATCH",
            Self::UnsafeMetadata => "ASSIST_DRAFT_UNSAFE_METADATA",
            Self::UsecaseDenied => "ASSIST_DRAFT_USECASE_DENIED",
            Self::IdempotencyKeyReused => "ASSIST_DRAFT_IDEMPOTENCY_KEY_REUSED",
            Self::ExecutorDenied => "ASSIST_DRAFT_EXECUTOR_DENIED",
            Self::ExecutorRateLimited => "ASSIST_DRAFT_EXECUTOR_RATE_LIMITED",
            Self::ExecutorFailed => "ASSIST_DRAFT_EXECUTOR_FAILED",
            Self::ExecutorAuthFailed => "ASSIST_DRAFT_EXECUTOR_AUTH_FAILED",
            Self::ExecutorInvalidRequest => "ASSIST_DRAFT_EXECUTOR_INVALID_REQUEST",
            Self::ExecutorTimeout => "ASSIST_DRAFT_EXECUTOR_TIMEOUT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftApiBoundaryContext {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub evidence_ref: String,          // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftApiRequest {
    pub boundary: AssistDraftApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: AssistDraftApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: AssistDraftApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: DomainAssistDraftRequest,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftApiSuccessResponse {
    pub status: AssistDraftApiStatus,               // data_class: PUBLIC
    pub usecase_receipt: AssistDraftUsecaseReceipt, // data_class: INTERNAL_ONLY
    pub dispatch_receipt: AssistDraftExecutorDispatchReceipt, // data_class: INTERNAL_ONLY
    pub metadata: AssistDraftApiResponseMetadata,   // data_class: INTERNAL_ONLY
}

impl AssistDraftApiSuccessResponse {
    pub fn http_status_code(&self) -> u16 {
        self.status.code()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftApiResponseMetadata {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub principal_id: String,      // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String, // data_class: INTERNAL_ONLY
    pub surface: String,           // data_class: INTERNAL_ONLY
    pub contract_ref: String,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftApiErrorResponse {
    pub error: AssistDraftApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftApiErrorBody {
    pub code: String,                            // data_class: INTERNAL_ONLY
    pub message: String,                         // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,       // data_class: INTERNAL_ONLY
    pub request_id: String,                      // data_class: INTERNAL_ONLY
    pub details: Vec<AssistDraftApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssistDraftApiError {
    Boundary {
        code: AssistDraftApiErrorCode,
        status: AssistDraftApiStatus,
        evidence_ref: String,
    },
    IdempotencyConflict {
        evidence_ref: String,
    },
    UsecaseDenied {
        denial_kind: Option<AssistDraftUsecaseDenialKind>,
        evidence_refs: Vec<String>,
    },
    ExecutorFailure {
        code: AssistDraftApiErrorCode,
        status: AssistDraftApiStatus,
        evidence_ref: String,
    },
}

impl AssistDraftApiError {
    pub fn status(&self) -> AssistDraftApiStatus {
        match self {
            Self::Boundary { status, .. } | Self::ExecutorFailure { status, .. } => *status,
            Self::IdempotencyConflict { .. } => AssistDraftApiStatus::Conflict,
            Self::UsecaseDenied { .. } => AssistDraftApiStatus::Forbidden,
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status().code()
    }

    pub fn code(&self) -> AssistDraftApiErrorCode {
        match self {
            Self::Boundary { code, .. } | Self::ExecutorFailure { code, .. } => *code,
            Self::IdempotencyConflict { .. } => AssistDraftApiErrorCode::IdempotencyKeyReused,
            Self::UsecaseDenied { .. } => AssistDraftApiErrorCode::UsecaseDenied,
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> AssistDraftApiErrorResponse {
        AssistDraftApiErrorResponse {
            error: AssistDraftApiErrorBody {
                code: self.code().as_str().to_owned(),
                message: self.message().to_owned(),
                message_localized: None,
                request_id: sanitize_response_metadata(request_id.into()),
                details: self.details(),
                retry_after_seconds: if self.status() == AssistDraftApiStatus::TooManyRequests {
                    Some(1)
                } else {
                    None
                },
            },
        }
    }

    fn message(&self) -> &'static str {
        match self.code() {
            AssistDraftApiErrorCode::RequestIdEmpty => "X-Request-Id header is required",
            AssistDraftApiErrorCode::TenantHeaderEmpty => "X-Tenant-Id header is required",
            AssistDraftApiErrorCode::IdempotencyKeyEmpty => "Idempotency-Key header is required",
            AssistDraftApiErrorCode::TraceContextInvalid => "Trace context reference is invalid",
            AssistDraftApiErrorCode::PrincipalIdEmpty => "Authenticated principal is required",
            AssistDraftApiErrorCode::AuthorizationDecisionIdEmpty => {
                "Authorization decision id is required"
            }
            AssistDraftApiErrorCode::AuthorizationEvidenceInvalid => {
                "Authorization evidence reference is invalid"
            }
            AssistDraftApiErrorCode::AuthorizationTenantMismatch => {
                "Authorization tenant must match the request tenant"
            }
            AssistDraftApiErrorCode::AuthorizationPrincipalMismatch => {
                "Authorization principal must match the authenticated principal"
            }
            AssistDraftApiErrorCode::AuthorizationDenied => {
                "Authorization does not allow assist-draft requests"
            }
            AssistDraftApiErrorCode::TenantBindingMismatch => {
                "Tenant header must match principal, policy, and request body"
            }
            AssistDraftApiErrorCode::PrincipalBindingMismatch => {
                "Authenticated principal must match policy and request body"
            }
            AssistDraftApiErrorCode::PolicyBindingMismatch => {
                "Policy decision must be bound to the assist-draft request"
            }
            AssistDraftApiErrorCode::UnsafeMetadata => "Assist-draft request metadata is invalid",
            AssistDraftApiErrorCode::UsecaseDenied => "Assist-draft request was denied",
            AssistDraftApiErrorCode::IdempotencyKeyReused => {
                "Idempotency key was already used with a different request"
            }
            AssistDraftApiErrorCode::ExecutorDenied => "Assist-draft executor denied the request",
            AssistDraftApiErrorCode::ExecutorRateLimited => {
                "Assist-draft executor rate limited the request"
            }
            AssistDraftApiErrorCode::ExecutorFailed => "Assist-draft executor failed the request",
            AssistDraftApiErrorCode::ExecutorAuthFailed => {
                "Assist-draft executor authentication failed"
            }
            AssistDraftApiErrorCode::ExecutorInvalidRequest => {
                "Assist-draft executor rejected the request"
            }
            AssistDraftApiErrorCode::ExecutorTimeout => "Assist-draft executor timed out",
        }
    }

    fn details(&self) -> Vec<AssistDraftApiErrorDetail> {
        let field = match self.code() {
            AssistDraftApiErrorCode::RequestIdEmpty => "headers.x-request-id",
            AssistDraftApiErrorCode::TenantHeaderEmpty => "headers.x-tenant-id",
            AssistDraftApiErrorCode::IdempotencyKeyEmpty => "headers.idempotency-key",
            AssistDraftApiErrorCode::TraceContextInvalid => "headers.trace-context-ref",
            AssistDraftApiErrorCode::PrincipalIdEmpty => "principal.principal_id",
            AssistDraftApiErrorCode::AuthorizationDecisionIdEmpty => "authorization.decision_id",
            AssistDraftApiErrorCode::AuthorizationEvidenceInvalid => "authorization.evidence_ref",
            AssistDraftApiErrorCode::AuthorizationTenantMismatch => "authorization.tenant_id",
            AssistDraftApiErrorCode::AuthorizationPrincipalMismatch => "authorization.principal_id",
            AssistDraftApiErrorCode::AuthorizationDenied => "authorization.allowed_surfaces",
            AssistDraftApiErrorCode::TenantBindingMismatch => "body.request.tenant_id",
            AssistDraftApiErrorCode::PrincipalBindingMismatch => "body.request.principal_id",
            AssistDraftApiErrorCode::PolicyBindingMismatch => "body.policy_decision",
            AssistDraftApiErrorCode::UnsafeMetadata => "body",
            AssistDraftApiErrorCode::UsecaseDenied => "body",
            AssistDraftApiErrorCode::IdempotencyKeyReused => "headers.idempotency-key",
            AssistDraftApiErrorCode::ExecutorDenied
            | AssistDraftApiErrorCode::ExecutorRateLimited
            | AssistDraftApiErrorCode::ExecutorFailed
            | AssistDraftApiErrorCode::ExecutorAuthFailed
            | AssistDraftApiErrorCode::ExecutorInvalidRequest
            | AssistDraftApiErrorCode::ExecutorTimeout => "executor",
        };
        vec![AssistDraftApiErrorDetail {
            field: field.to_owned(),
            issue: self.code().as_str().to_owned(),
        }]
    }
}

pub struct IntelligenceAssistDraftApi {
    usecase: IntelligenceAssistDraftUsecase,
    adapter: IntelligenceAssistDraftAdapter,
    ledger: BTreeMap<AssistDraftApiLedgerKey, AssistDraftApiLedgerEntry>,
    dispatch_count: usize,
}

impl IntelligenceAssistDraftApi {
    pub fn new(adapter: IntelligenceAssistDraftAdapter) -> Self {
        Self {
            usecase: IntelligenceAssistDraftUsecase::default(),
            adapter,
            ledger: BTreeMap::new(),
            dispatch_count: 0,
        }
    }

    pub fn submit(
        &mut self,
        request: AssistDraftApiRequest,
    ) -> Result<AssistDraftApiSuccessResponse, AssistDraftApiError> {
        validate_api_request(&request)?;
        let key = AssistDraftApiLedgerKey::from_request(&request);
        let fingerprint = AssistDraftApiFingerprint::from_request(&request);
        if let Some(entry) = self.ledger.get(&key) {
            if entry.fingerprint == fingerprint {
                return Ok(entry.response.clone());
            }
            return Err(AssistDraftApiError::IdempotencyConflict {
                evidence_ref: "validation:assist-draft-api-idempotency-conflict".to_owned(),
            });
        }

        let usecase_receipt = self.usecase.plan(AssistDraftUsecaseInput {
            idempotency_key: request.boundary.idempotency_key.clone(),
            request: request.body.clone(),
        });
        if usecase_receipt.status != AssistDraftUsecaseStatus::Planned {
            return Err(AssistDraftApiError::UsecaseDenied {
                denial_kind: usecase_receipt.denial_kind,
                evidence_refs: sorted_unique(usecase_receipt.evidence_refs.clone()),
            });
        }

        self.dispatch_count += 1;
        let dispatch_receipt = self
            .adapter
            .dispatch(AssistDraftExecutorDispatchRequest {
                idempotency_key: request.boundary.idempotency_key.clone(),
                domain_request: request.body.clone(),
                usecase_receipt: usecase_receipt.clone(),
            })
            .map_err(api_error_from_executor_failure)?;

        let response = AssistDraftApiSuccessResponse {
            status: api_status_from_dispatch(dispatch_receipt.status),
            usecase_receipt,
            dispatch_receipt,
            metadata: AssistDraftApiResponseMetadata {
                request_id: request.boundary.request_id.clone(),
                tenant_id: request.boundary.tenant_id.clone(),
                principal_id: request.principal.principal_id.clone(),
                idempotency_key: request.boundary.idempotency_key.clone(),
                trace_context_ref: request.boundary.trace_context_ref.clone(),
                surface: ASSIST_DRAFT_API_SURFACE.to_owned(),
                contract_ref: ASSIST_DRAFT_API_CONTRACT_REF.to_owned(),
            },
        };
        self.ledger.insert(
            key,
            AssistDraftApiLedgerEntry {
                fingerprint,
                response: response.clone(),
            },
        );
        Ok(response)
    }

    pub fn dispatch_count(&self) -> usize {
        self.dispatch_count
    }

    pub fn adapter_last_envelope(&self) -> Option<&AssistDraftExecutorRequestEnvelope> {
        self.adapter.last_envelope()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct AssistDraftApiLedgerKey {
    tenant_id: String,
    principal_id: String,
    surface: String,
    idempotency_key: String,
}

impl AssistDraftApiLedgerKey {
    fn from_request(request: &AssistDraftApiRequest) -> Self {
        Self {
            tenant_id: request.boundary.tenant_id.clone(),
            principal_id: request.principal.principal_id.clone(),
            surface: ASSIST_DRAFT_API_SURFACE.to_owned(),
            idempotency_key: request.boundary.idempotency_key.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AssistDraftApiLedgerEntry {
    fingerprint: AssistDraftApiFingerprint,
    response: AssistDraftApiSuccessResponse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AssistDraftApiFingerprint {
    canonical: String,
}

impl AssistDraftApiFingerprint {
    fn from_request(request: &AssistDraftApiRequest) -> Self {
        let domain = &request.body;
        let kernel = &domain.request;
        let policy = &domain.policy_decision;
        let mut entries = vec![
            canonical_entry("request_id", &request.boundary.request_id),
            canonical_entry("tenant", &request.boundary.tenant_id),
            canonical_entry("principal", &request.principal.principal_id),
            canonical_entry("trace", &request.boundary.trace_context_ref),
            canonical_entry("auth_decision", &request.authorization.decision_id),
            canonical_entry("auth_evidence", &request.authorization.evidence_ref),
            canonical_vec_entry(
                "auth_surfaces",
                &sorted_unique(request.authorization.allowed_surfaces.clone()),
            ),
            canonical_entry("brand_surface", &domain.brand_surface_ref),
            canonical_entry("locale", &domain.locale),
            canonical_vec_entry(
                "prompt_context_refs",
                &sorted_unique(domain.prompt_context_refs.clone()),
            ),
            canonical_entry("policy_decision_id", &policy.decision_id),
            canonical_entry("policy_tenant", &policy.tenant_id),
            canonical_entry("policy_principal", &policy.principal_id),
            canonical_entry("policy_evidence", &policy.evidence_ref),
            canonical_entry("prompt_registry", &policy.prompt_registry_snapshot_ref),
            canonical_entry("cost_floor", &policy.cost_floor_disclosure_ref),
            canonical_entry("builder_scope", &policy.builder_capability_scope_ref),
            canonical_entry("kernel_tenant", &kernel.tenant_id),
            canonical_entry("kernel_principal", &kernel.principal_id),
            canonical_entry("context", &kernel.context_id),
            canonical_entry("prompt_ref", &kernel.prompt_ref),
            canonical_entry("target_builder", &kernel.target_builder_ref),
            canonical_entry("output_contract", &kernel.output_contract_ref),
            canonical_entry("consent", &kernel.consent_grant_ref),
            canonical_entry("budget", &kernel.budget_evidence_ref),
            canonical_entry("policy_ref", &kernel.policy_decision_ref),
            canonical_entry("model_route", &kernel.model_route_ref),
            canonical_entry("guardrail", &kernel.guardrail_evidence_ref),
            canonical_entry("request_evidence", &kernel.request_evidence_ref),
            canonical_entry("kernel_trace", &kernel.trace_context_ref),
            canonical_vec_entry(
                "additional_evidence",
                &sorted_unique(kernel.additional_evidence_refs.clone()),
            ),
        ];
        entries.push(canonical_entry(
            "builder_surface",
            &format!("{:?}", kernel.builder_surface),
        ));
        entries.push(canonical_entry(
            "draft_kind",
            &format!("{:?}", kernel.draft_kind),
        ));
        entries.push(canonical_entry(
            "audience",
            &format!("{:?}", kernel.audience),
        ));
        entries.push(canonical_entry(
            "invocation",
            &format!("{:?}", kernel.invocation_mode),
        ));
        entries.push(canonical_entry(
            "review_gate",
            &format!("{:?}", kernel.review_gate),
        ));
        entries.push(canonical_vec_entry(
            "actions",
            &action_entries(&kernel.requested_actions),
        ));
        entries.push(canonical_vec_entry(
            "data_classes",
            &data_class_entries(&kernel.data_classes),
        ));
        entries.sort();
        Self {
            canonical: entries.join("|"),
        }
    }
}

fn validate_api_request(request: &AssistDraftApiRequest) -> Result<(), AssistDraftApiError> {
    require_metadata(
        AssistDraftApiErrorCode::RequestIdEmpty,
        AssistDraftApiStatus::BadRequest,
        "validation:assist-draft-api-request-id",
        &request.boundary.request_id,
    )?;
    require_opaque(
        AssistDraftApiErrorCode::TenantHeaderEmpty,
        AssistDraftApiStatus::BadRequest,
        "validation:assist-draft-api-tenant-header",
        &request.boundary.tenant_id,
    )?;
    require_metadata(
        AssistDraftApiErrorCode::IdempotencyKeyEmpty,
        AssistDraftApiStatus::BadRequest,
        "validation:assist-draft-api-idempotency-key",
        &request.boundary.idempotency_key,
    )?;
    require_opaque(
        AssistDraftApiErrorCode::TraceContextInvalid,
        AssistDraftApiStatus::BadRequest,
        "validation:assist-draft-api-trace-context",
        &request.boundary.trace_context_ref,
    )?;
    require_opaque(
        AssistDraftApiErrorCode::PrincipalIdEmpty,
        AssistDraftApiStatus::Unauthorized,
        "validation:assist-draft-api-principal",
        &request.principal.principal_id,
    )?;
    require_metadata(
        AssistDraftApiErrorCode::AuthorizationDecisionIdEmpty,
        AssistDraftApiStatus::Forbidden,
        "validation:assist-draft-api-auth-decision",
        &request.authorization.decision_id,
    )?;
    require_opaque(
        AssistDraftApiErrorCode::AuthorizationEvidenceInvalid,
        AssistDraftApiStatus::Forbidden,
        "validation:assist-draft-api-auth-evidence",
        &request.authorization.evidence_ref,
    )?;
    if request.authorization.tenant_id != request.boundary.tenant_id {
        return Err(boundary_error(
            AssistDraftApiErrorCode::AuthorizationTenantMismatch,
            AssistDraftApiStatus::Forbidden,
            "validation:assist-draft-api-auth-tenant",
        ));
    }
    if request.authorization.principal_id != request.principal.principal_id {
        return Err(boundary_error(
            AssistDraftApiErrorCode::AuthorizationPrincipalMismatch,
            AssistDraftApiStatus::Forbidden,
            "validation:assist-draft-api-auth-principal",
        ));
    }
    if !request
        .authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == ASSIST_DRAFT_API_SURFACE)
    {
        return Err(boundary_error(
            AssistDraftApiErrorCode::AuthorizationDenied,
            AssistDraftApiStatus::Forbidden,
            "validation:assist-draft-api-auth-surface",
        ));
    }

    let domain = &request.body;
    let kernel = &domain.request;
    let policy = &domain.policy_decision;
    if request.boundary.tenant_id != request.principal.tenant_id
        || request.boundary.tenant_id != kernel.tenant_id
        || request.boundary.tenant_id != policy.tenant_id
    {
        return Err(boundary_error(
            AssistDraftApiErrorCode::TenantBindingMismatch,
            AssistDraftApiStatus::Forbidden,
            "validation:assist-draft-api-tenant-binding",
        ));
    }
    if request.principal.principal_id != domain.principal_id
        || request.principal.principal_id != kernel.principal_id
        || request.principal.principal_id != policy.principal_id
    {
        return Err(boundary_error(
            AssistDraftApiErrorCode::PrincipalBindingMismatch,
            AssistDraftApiStatus::Forbidden,
            "validation:assist-draft-api-principal-binding",
        ));
    }
    if request.authorization.decision_id != policy.decision_id
        || request.authorization.evidence_ref != policy.evidence_ref
        || kernel.policy_decision_ref != policy.evidence_ref
    {
        return Err(boundary_error(
            AssistDraftApiErrorCode::PolicyBindingMismatch,
            AssistDraftApiStatus::Forbidden,
            "validation:assist-draft-api-policy-binding",
        ));
    }
    validate_domain_metadata(domain)
}

fn validate_domain_metadata(domain: &DomainAssistDraftRequest) -> Result<(), AssistDraftApiError> {
    let kernel = &domain.request;
    let policy = &domain.policy_decision;
    let opaque_refs = [
        domain.principal_id.as_str(),
        domain.brand_surface_ref.as_str(),
        kernel.tenant_id.as_str(),
        kernel.principal_id.as_str(),
        kernel.context_id.as_str(),
        kernel.prompt_ref.as_str(),
        kernel.target_builder_ref.as_str(),
        kernel.output_contract_ref.as_str(),
        kernel.consent_grant_ref.as_str(),
        kernel.budget_evidence_ref.as_str(),
        kernel.policy_decision_ref.as_str(),
        kernel.model_route_ref.as_str(),
        kernel.guardrail_evidence_ref.as_str(),
        kernel.request_evidence_ref.as_str(),
        kernel.trace_context_ref.as_str(),
        policy.evidence_ref.as_str(),
        policy.prompt_registry_snapshot_ref.as_str(),
        policy.cost_floor_disclosure_ref.as_str(),
        policy.builder_capability_scope_ref.as_str(),
    ];
    if opaque_refs
        .into_iter()
        .any(|value| !is_safe_opaque_ref(value))
        || !is_safe_metadata_ref(&domain.locale)
        || !is_safe_metadata_ref(&policy.decision_id)
        || domain
            .prompt_context_refs
            .iter()
            .any(|value| !is_safe_opaque_ref(value))
        || kernel
            .additional_evidence_refs
            .iter()
            .any(|value| !is_safe_opaque_ref(value))
    {
        return Err(boundary_error(
            AssistDraftApiErrorCode::UnsafeMetadata,
            AssistDraftApiStatus::BadRequest,
            "validation:assist-draft-api-domain-metadata",
        ));
    }
    Ok(())
}

fn api_status_from_dispatch(status: AssistDraftExecutorDispatchStatus) -> AssistDraftApiStatus {
    match status {
        AssistDraftExecutorDispatchStatus::Accepted
        | AssistDraftExecutorDispatchStatus::Queued
        | AssistDraftExecutorDispatchStatus::Completed => AssistDraftApiStatus::Accepted,
    }
}

fn api_error_from_executor_failure(
    failure: AssistDraftExecutorDispatchFailure,
) -> AssistDraftApiError {
    let lower = failure.reason.to_ascii_lowercase();
    let (code, status) = if lower.contains("rate limited") {
        (
            AssistDraftApiErrorCode::ExecutorRateLimited,
            AssistDraftApiStatus::TooManyRequests,
        )
    } else if lower.contains("timed out") {
        (
            AssistDraftApiErrorCode::ExecutorTimeout,
            AssistDraftApiStatus::GatewayTimeout,
        )
    } else if lower.contains("auth failed") {
        (
            AssistDraftApiErrorCode::ExecutorAuthFailed,
            AssistDraftApiStatus::FailedDependency,
        )
    } else if lower.contains("invalid") {
        (
            AssistDraftApiErrorCode::ExecutorInvalidRequest,
            AssistDraftApiStatus::BadRequest,
        )
    } else if lower.contains("denied") {
        (
            AssistDraftApiErrorCode::ExecutorDenied,
            AssistDraftApiStatus::Forbidden,
        )
    } else {
        (
            AssistDraftApiErrorCode::ExecutorFailed,
            AssistDraftApiStatus::ServiceUnavailable,
        )
    };
    AssistDraftApiError::ExecutorFailure {
        code,
        status,
        evidence_ref: sanitize_response_metadata(failure.evidence_ref),
    }
}

fn require_metadata(
    code: AssistDraftApiErrorCode,
    status: AssistDraftApiStatus,
    evidence_ref: &str,
    value: &str,
) -> Result<(), AssistDraftApiError> {
    if is_safe_metadata_ref(value) {
        Ok(())
    } else {
        Err(boundary_error(code, status, evidence_ref))
    }
}

fn require_opaque(
    code: AssistDraftApiErrorCode,
    status: AssistDraftApiStatus,
    evidence_ref: &str,
    value: &str,
) -> Result<(), AssistDraftApiError> {
    if is_safe_opaque_ref(value) {
        Ok(())
    } else {
        Err(boundary_error(code, status, evidence_ref))
    }
}

fn boundary_error(
    code: AssistDraftApiErrorCode,
    status: AssistDraftApiStatus,
    evidence_ref: &str,
) -> AssistDraftApiError {
    AssistDraftApiError::Boundary {
        code,
        status,
        evidence_ref: sanitize_response_metadata(evidence_ref.to_owned()),
    }
}

fn sanitize_response_metadata(value: String) -> String {
    if is_safe_metadata_ref(&value) {
        value
    } else {
        "assist-draft-api:redacted-unsafe-metadata".to_owned()
    }
}

fn is_safe_opaque_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.contains(':')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn is_safe_metadata_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn contains_whitespace(value: &str) -> bool {
    value.chars().any(char::is_whitespace)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("secret=")
        || lower.contains("password=")
        || lower.contains("token=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw output")
        || lower.contains("customer message")
        || lower.contains("write an email")
        || lower.contains("model answer")
        || lower.contains("document text")
        || lower.contains("document=")
        || lower.contains("prompt=")
        || lower.contains("completion=")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn action_entries(values: &[AssistDraftAction]) -> Vec<String> {
    let mut entries: Vec<String> = values.iter().map(|value| format!("{value:?}")).collect();
    entries.sort();
    entries.dedup();
    entries
}

fn data_class_entries(values: &[AssistDraftDataClass]) -> Vec<String> {
    let mut entries: Vec<String> = values.iter().map(|value| format!("{value:?}")).collect();
    entries.sort();
    entries.dedup();
    entries
}

fn canonical_entry(key: &str, value: &str) -> String {
    format!("{key}={value}")
}

fn canonical_vec_entry(key: &str, values: &[String]) -> String {
    format!("{key}=[{}]", values.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_authorized_request_and_dispatches_metadata_only_executor() {
        let mut api = IntelligenceAssistDraftApi::new(valid_adapter());
        let response = api.submit(valid_api_request()).expect("accepted");
        assert_eq!(response.status, AssistDraftApiStatus::Accepted);
        assert_eq!(response.http_status_code(), 202);
        assert_eq!(response.metadata.request_id, "request:api:assist-draft:1");
        assert_eq!(
            api.adapter_last_envelope().expect("envelope").tenant_id,
            "tenant:alpha"
        );
    }

    #[test]
    fn rejects_tenant_or_principal_drift_before_usecase_or_adapter_side_effects() {
        let mut api = IntelligenceAssistDraftApi::new(valid_adapter());
        let mut request = valid_api_request();
        request.boundary.tenant_id = "tenant:other".to_owned();
        let error = api.submit(request).expect_err("tenant drift");
        assert_eq!(error.status(), AssistDraftApiStatus::Forbidden);
        assert_eq!(api.adapter_last_envelope(), None);
    }

    #[test]
    fn idempotent_replay_returns_cached_response_and_conflict_does_not_dispatch_again() {
        let mut api = IntelligenceAssistDraftApi::new(valid_adapter());
        let first = api.submit(valid_api_request()).expect("first");
        let replay = api.submit(valid_api_request()).expect("replay");
        assert_eq!(first, replay);
        assert_eq!(api.dispatch_count(), 1);
        let mut conflict = valid_api_request();
        conflict.body.request.output_contract_ref = "workflow-spec://contracts/v2".to_owned();
        let error = api.submit(conflict).expect_err("conflict");
        assert_eq!(error.status(), AssistDraftApiStatus::Conflict);
        assert_eq!(api.dispatch_count(), 1);
    }

    #[test]
    fn maps_executor_failures_to_stable_http_statuses() {
        let cases = [
            (
                AssistDraftExecutorStatus::RateLimited {
                    evidence_ref: "executor-evidence:assist-draft:rate-limited".to_owned(),
                },
                AssistDraftApiStatus::TooManyRequests,
            ),
            (
                AssistDraftExecutorStatus::Timeout {
                    evidence_ref: "executor-evidence:assist-draft:timeout".to_owned(),
                },
                AssistDraftApiStatus::GatewayTimeout,
            ),
            (
                AssistDraftExecutorStatus::InvalidRequest {
                    evidence_ref: "executor-evidence:assist-draft:invalid".to_owned(),
                },
                AssistDraftApiStatus::BadRequest,
            ),
        ];
        for (status, expected) in cases {
            let mut api = IntelligenceAssistDraftApi::new(valid_adapter_with_status(status));
            let error = api.submit(valid_api_request()).expect_err("mapped failure");
            assert_eq!(error.status(), expected);
        }
    }

    #[test]
    fn structured_errors_do_not_echo_raw_prompt_output_document_or_secret_material() {
        let mut api = IntelligenceAssistDraftApi::new(valid_adapter());
        let mut request = valid_api_request();
        request.body.request.prompt_ref = "raw prompt: write an email with sk-secret".to_owned();
        let error = api.submit(request).expect_err("unsafe");
        let rendered = format!("{:?}", error.error_response("request:api:assist-draft:1"));
        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("write an email"));
        assert!(!rendered.contains("sk-secret"));
    }

    pub(crate) fn valid_adapter() -> IntelligenceAssistDraftAdapter {
        valid_adapter_with_status(AssistDraftExecutorStatus::Accepted {
            executor_request_ref: "draft-request:assist-draft:1".to_owned(),
            draft_ref: "draft:assist-draft:1".to_owned(),
            evidence_ref: "executor-evidence:assist-draft:accepted".to_owned(),
        })
    }

    pub(crate) fn valid_adapter_with_status(
        status: AssistDraftExecutorStatus,
    ) -> IntelligenceAssistDraftAdapter {
        IntelligenceAssistDraftAdapter::try_new(
            AssistDraftExecutorAdapterConfig::new(
                "https://assist-draft-executor.internal",
                "credential-handle:assist-draft:1",
                "audit-tap:assist-draft:1",
                "draft-executor:assist-draft:workflow-studio",
            ),
            status,
        )
        .expect("adapter")
    }

    pub(crate) fn valid_api_request() -> AssistDraftApiRequest {
        AssistDraftApiRequest {
            boundary: AssistDraftApiBoundaryContext {
                request_id: "request:api:assist-draft:1".to_owned(),
                tenant_id: "tenant:alpha".to_owned(),
                idempotency_key: "idempotency:assist-draft:1".to_owned(),
                trace_context_ref: "trace:assist-draft:1".to_owned(),
            },
            principal: AssistDraftApiPrincipal {
                tenant_id: "tenant:alpha".to_owned(),
                principal_id: "principal:builder-owner".to_owned(),
            },
            authorization: AssistDraftApiAuthorization {
                tenant_id: "tenant:alpha".to_owned(),
                principal_id: "principal:builder-owner".to_owned(),
                decision_id: "decision:assist-draft:1".to_owned(),
                evidence_ref: "policy:assist-draft:allow".to_owned(),
                allowed_surfaces: vec![ASSIST_DRAFT_API_SURFACE.to_owned()],
            },
            body: valid_domain_request(),
        }
    }

    pub(crate) fn valid_domain_request() -> DomainAssistDraftRequest {
        DomainAssistDraftRequest {
            principal_id: "principal:builder-owner".to_owned(),
            brand_surface_ref: "brand-surface:workflow-studio:assist".to_owned(),
            locale: "en-US".to_owned(),
            prompt_context_refs: vec!["context-snippet:workflow-studio:canvas-1".to_owned()],
            policy_decision: AssistDraftPolicyDecision {
                decision_id: "decision:assist-draft:1".to_owned(),
                tenant_id: "tenant:alpha".to_owned(),
                principal_id: "principal:builder-owner".to_owned(),
                ai_assist_enabled: true,
                explicit_automation_allowed: false,
                allowed_builder_surfaces: vec![AssistDraftBuilderSurface::WorkflowStudio],
                allowed_draft_kinds: vec![AssistDraftKind::WorkflowDraft],
                allowed_audiences: vec![AssistDraftAudience::TenantBuilder],
                allowed_data_classes: vec![
                    AssistDraftDataClass::Internal,
                    AssistDraftDataClass::Public,
                ],
                allowed_actions: vec![
                    AssistDraftAction::CreateDraft,
                    AssistDraftAction::ExplainDraft,
                ],
                allowed_locales: vec!["en-US".to_owned()],
                max_prompt_context_refs: 4,
                evidence_ref: "policy:assist-draft:allow".to_owned(),
                prompt_registry_snapshot_ref: "prompt-registry:assist-draft:v1".to_owned(),
                cost_floor_disclosure_ref: "cost-floor:assist-draft:workflow-studio".to_owned(),
                builder_capability_scope_ref: "builder-scope:workflow-studio:draft".to_owned(),
            },
            request: AssistDraftRequest {
                tenant_id: "tenant:alpha".to_owned(),
                principal_id: "principal:builder-owner".to_owned(),
                context_id: "context://workflow-studio/canvas-1".to_owned(),
                builder_surface: AssistDraftBuilderSurface::WorkflowStudio,
                draft_kind: AssistDraftKind::WorkflowDraft,
                audience: AssistDraftAudience::TenantBuilder,
                invocation_mode: AssistDraftInvocationMode::UserInvoked,
                review_gate: AssistDraftReviewGate::HumanReviewRequired,
                prompt_ref: "prompt://assist-draft/req-1".to_owned(),
                target_builder_ref: "builder://workflow-studio/canvas-1".to_owned(),
                output_contract_ref: "workflow-spec://contracts/v1".to_owned(),
                consent_grant_ref: "consent:assist-draft:1".to_owned(),
                budget_evidence_ref: "budget:assist-draft:1".to_owned(),
                policy_decision_ref: "policy:assist-draft:allow".to_owned(),
                model_route_ref: "model-route:assist-draft:1".to_owned(),
                guardrail_evidence_ref: "guardrail:assist-draft:allow".to_owned(),
                request_evidence_ref: "request:assist-draft:1".to_owned(),
                trace_context_ref: "trace:assist-draft:1".to_owned(),
                data_classes: vec![AssistDraftDataClass::Internal, AssistDraftDataClass::Public],
                requested_actions: vec![
                    AssistDraftAction::CreateDraft,
                    AssistDraftAction::ExplainDraft,
                ],
                additional_evidence_refs: Vec::new(),
            },
        }
    }
}
