//! Intelligence assist-draft adapter foundation.
//!
//! This crate defines a deterministic, metadata-only adapter seam for future
//! assist-draft executor integration. It builds executor request envelopes from
//! assist-draft domain/usecase metadata already authorized by the preview
//! foundation and maps executor outcome metadata into stable receipts. It
//! performs no prompt rendering, model calls, provider dispatch, network I/O,
//! builder mutation, filesystem access, credential resolution, durable
//! idempotency, durable audit-chain emission, or queue/runtime processing.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_assist_draft_usecase::{
    AssistDraftAction, AssistDraftAudience, AssistDraftBuilderSurface, AssistDraftDataClass,
    AssistDraftDenialReason, AssistDraftDomainDecision, AssistDraftDomainDenialKind,
    AssistDraftDomainStatus, AssistDraftInvocationMode, AssistDraftKind, AssistDraftPolicyDecision,
    AssistDraftRequest, AssistDraftReviewGate, AssistDraftUsecaseDenialKind,
    AssistDraftUsecaseInput, AssistDraftUsecaseReceipt, AssistDraftUsecaseStatus,
    DomainAssistDraftRequest, IntelligenceAssistDraftUsecase, plan_domain_assist_draft,
};

const ASSIST_DRAFT_EXECUTOR_PATH: &str = "/v1/assist-draft-executions";
const ADAPTER_REFERENCE_REF: &str = "spec://oyatie/intelligence/assist-draft-adapter-foundation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftExecutorHttpMethod {
    Post,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftExecutorTransportMode {
    EnvelopeOnly,
    HostedExecutor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftExecutorAdapterConfig {
    pub endpoint: String,              // data_class: INTERNAL_ONLY
    pub credential_handle_ref: String, // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,         // data_class: INTERNAL_ONLY
    pub executor_audience_ref: String, // data_class: INTERNAL_ONLY
    pub transport_mode: AssistDraftExecutorTransportMode, // data_class: INTERNAL_ONLY
}

impl AssistDraftExecutorAdapterConfig {
    pub fn new(
        endpoint: impl Into<String>,
        credential_handle_ref: impl Into<String>,
        audit_tap_ref: impl Into<String>,
        executor_audience_ref: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            credential_handle_ref: credential_handle_ref.into(),
            audit_tap_ref: audit_tap_ref.into(),
            executor_audience_ref: executor_audience_ref.into(),
            transport_mode: AssistDraftExecutorTransportMode::EnvelopeOnly,
        }
    }

    pub fn with_transport_mode(mut self, transport_mode: AssistDraftExecutorTransportMode) -> Self {
        self.transport_mode = transport_mode;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftExecutorAdapterConfigError {
    EmptyEndpoint,
    NonHttpsEndpoint,
    LocalEndpointDenied,
    EmptyCredentialHandleRef,
    RawCredentialMaterialRejected,
    NonOpaqueCredentialHandleRef,
    EmptyAuditTapRef,
    InvalidAuditTapRef,
    EmptyExecutorAudienceRef,
    InvalidExecutorAudienceRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftExecutorDispatchRequest {
    pub idempotency_key: String,                  // data_class: INTERNAL_ONLY
    pub domain_request: DomainAssistDraftRequest, // data_class: INTERNAL_ONLY
    pub usecase_receipt: AssistDraftUsecaseReceipt, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftExecutorRequestEnvelope {
    pub method: AssistDraftExecutorHttpMethod, // data_class: PUBLIC
    pub endpoint: String,                      // data_class: INTERNAL_ONLY
    pub path: String,                          // data_class: PUBLIC
    pub transport_mode: AssistDraftExecutorTransportMode, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub principal_id: String,                  // data_class: INTERNAL_ONLY
    pub brand_surface_ref: String,             // data_class: INTERNAL_ONLY
    pub locale: String,                        // data_class: INTERNAL_ONLY
    pub idempotency_key: String,               // data_class: INTERNAL_ONLY
    pub context_id: String,                    // data_class: INTERNAL_ONLY
    pub target_builder_ref: String,            // data_class: INTERNAL_ONLY
    pub output_contract_ref: String,           // data_class: INTERNAL_ONLY
    pub builder_surface: AssistDraftBuilderSurface, // data_class: PUBLIC
    pub draft_kind: AssistDraftKind,           // data_class: PUBLIC
    pub audience: AssistDraftAudience,         // data_class: INTERNAL_ONLY
    pub invocation_mode: AssistDraftInvocationMode, // data_class: INTERNAL_ONLY
    pub review_gate: AssistDraftReviewGate,    // data_class: INTERNAL_ONLY
    pub prompt_ref: String,                    // data_class: INTERNAL_ONLY
    pub prompt_context_refs: Vec<String>,      // data_class: INTERNAL_ONLY
    pub consent_grant_ref: String,             // data_class: INTERNAL_ONLY
    pub budget_evidence_ref: String,           // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,          // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,             // data_class: INTERNAL_ONLY
    pub policy_decision_id: String,            // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,           // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: String,           // data_class: INTERNAL_ONLY
    pub prompt_registry_snapshot_ref: String,  // data_class: INTERNAL_ONLY
    pub model_route_ref: String,               // data_class: INTERNAL_ONLY
    pub guardrail_evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub cost_floor_disclosure_ref: String,     // data_class: INTERNAL_ONLY
    pub builder_capability_scope_ref: String,  // data_class: INTERNAL_ONLY
    pub credential_handle_ref: String,         // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,                 // data_class: INTERNAL_ONLY
    pub executor_audience_ref: String,         // data_class: INTERNAL_ONLY
    pub requested_actions: Vec<AssistDraftAction>, // data_class: INTERNAL_ONLY
    pub planned_actions: Vec<AssistDraftAction>, // data_class: INTERNAL_ONLY
    pub data_classes: Vec<AssistDraftDataClass>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
    pub adapter_reference_refs: Vec<String>,   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssistDraftExecutorStatus {
    Accepted {
        executor_request_ref: String,
        draft_ref: String,
        evidence_ref: String,
    },
    Queued {
        executor_request_ref: String,
        queue_ref: String,
        evidence_ref: String,
    },
    Completed {
        executor_request_ref: String,
        draft_ref: String,
        suggested_patch_ref: String,
        evidence_ref: String,
    },
    Denied {
        evidence_ref: String,
    },
    RateLimited {
        evidence_ref: String,
    },
    ExecutorError {
        evidence_ref: String,
    },
    AuthError {
        evidence_ref: String,
    },
    InvalidRequest {
        evidence_ref: String,
    },
    Timeout {
        evidence_ref: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssistDraftExecutorDispatchStatus {
    Accepted,
    Queued,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftExecutorDispatchReceipt {
    pub status: AssistDraftExecutorDispatchStatus, // data_class: PUBLIC
    pub executor_request_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub draft_ref: Option<String>,                 // data_class: INTERNAL_ONLY
    pub queue_ref: Option<String>,                 // data_class: INTERNAL_ONLY
    pub suggested_patch_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssistDraftExecutorDispatchFailure {
    pub reason: String,       // data_class: INTERNAL_ONLY
    pub evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Debug)]
pub struct IntelligenceAssistDraftAdapter {
    config: AssistDraftExecutorAdapterConfig,
    next_status: AssistDraftExecutorStatus,
    last_envelope: Option<AssistDraftExecutorRequestEnvelope>,
}

impl IntelligenceAssistDraftAdapter {
    pub fn try_new(
        config: AssistDraftExecutorAdapterConfig,
        next_status: AssistDraftExecutorStatus,
    ) -> Result<Self, AssistDraftExecutorAdapterConfigError> {
        validate_config(&config)?;
        Ok(Self {
            config,
            next_status,
            last_envelope: None,
        })
    }

    pub fn last_envelope(&self) -> Option<&AssistDraftExecutorRequestEnvelope> {
        self.last_envelope.as_ref()
    }

    pub fn set_next_status(&mut self, next_status: AssistDraftExecutorStatus) {
        self.next_status = next_status;
    }

    pub fn dispatch(
        &mut self,
        request: AssistDraftExecutorDispatchRequest,
    ) -> Result<AssistDraftExecutorDispatchReceipt, AssistDraftExecutorDispatchFailure> {
        validate_dispatch_request(&request)?;
        validate_status_metadata(&self.next_status)?;
        let envelope = self.build_envelope(&request);
        self.last_envelope = Some(envelope);
        receipt_from_status(&self.next_status)
    }

    fn build_envelope(
        &self,
        request: &AssistDraftExecutorDispatchRequest,
    ) -> AssistDraftExecutorRequestEnvelope {
        let domain = &request.domain_request;
        let kernel = &domain.request;
        let policy = &domain.policy_decision;
        AssistDraftExecutorRequestEnvelope {
            method: AssistDraftExecutorHttpMethod::Post,
            endpoint: normalized_endpoint(&self.config.endpoint),
            path: ASSIST_DRAFT_EXECUTOR_PATH.to_owned(),
            transport_mode: self.config.transport_mode,
            tenant_id: kernel.tenant_id.clone(),
            principal_id: domain.principal_id.clone(),
            brand_surface_ref: domain.brand_surface_ref.clone(),
            locale: domain.locale.clone(),
            idempotency_key: request.idempotency_key.clone(),
            context_id: kernel.context_id.clone(),
            target_builder_ref: kernel.target_builder_ref.clone(),
            output_contract_ref: kernel.output_contract_ref.clone(),
            builder_surface: kernel.builder_surface,
            draft_kind: kernel.draft_kind,
            audience: kernel.audience,
            invocation_mode: kernel.invocation_mode,
            review_gate: kernel.review_gate,
            prompt_ref: kernel.prompt_ref.clone(),
            prompt_context_refs: sorted_unique(domain.prompt_context_refs.clone()),
            consent_grant_ref: kernel.consent_grant_ref.clone(),
            budget_evidence_ref: kernel.budget_evidence_ref.clone(),
            request_evidence_ref: kernel.request_evidence_ref.clone(),
            trace_context_ref: kernel.trace_context_ref.clone(),
            policy_decision_id: policy.decision_id.clone(),
            policy_decision_ref: kernel.policy_decision_ref.clone(),
            policy_evidence_ref: policy.evidence_ref.clone(),
            prompt_registry_snapshot_ref: policy.prompt_registry_snapshot_ref.clone(),
            model_route_ref: kernel.model_route_ref.clone(),
            guardrail_evidence_ref: kernel.guardrail_evidence_ref.clone(),
            cost_floor_disclosure_ref: policy.cost_floor_disclosure_ref.clone(),
            builder_capability_scope_ref: policy.builder_capability_scope_ref.clone(),
            credential_handle_ref: self.config.credential_handle_ref.clone(),
            audit_tap_ref: self.config.audit_tap_ref.clone(),
            executor_audience_ref: self.config.executor_audience_ref.clone(),
            requested_actions: sorted_unique_actions(kernel.requested_actions.clone()),
            planned_actions: sorted_unique_actions(request.usecase_receipt.planned_actions.clone()),
            data_classes: sorted_unique_data_classes(kernel.data_classes.clone()),
            evidence_refs: dispatch_evidence_refs(request),
            adapter_reference_refs: vec![ADAPTER_REFERENCE_REF.to_owned()],
        }
    }
}

fn validate_config(
    config: &AssistDraftExecutorAdapterConfig,
) -> Result<(), AssistDraftExecutorAdapterConfigError> {
    let endpoint = config.endpoint.trim();
    if endpoint.is_empty() {
        return Err(AssistDraftExecutorAdapterConfigError::EmptyEndpoint);
    }
    if !endpoint.starts_with("https://") || contains_whitespace(endpoint) {
        return Err(AssistDraftExecutorAdapterConfigError::NonHttpsEndpoint);
    }
    if is_local_endpoint(endpoint) {
        return Err(AssistDraftExecutorAdapterConfigError::LocalEndpointDenied);
    }
    validate_credential_handle_ref(&config.credential_handle_ref)?;
    validate_audit_tap_ref(&config.audit_tap_ref)?;
    validate_executor_audience_ref(&config.executor_audience_ref)?;
    Ok(())
}

fn validate_credential_handle_ref(
    value: &str,
) -> Result<(), AssistDraftExecutorAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AssistDraftExecutorAdapterConfigError::EmptyCredentialHandleRef);
    }
    if contains_raw_secret_material(trimmed) {
        return Err(AssistDraftExecutorAdapterConfigError::RawCredentialMaterialRejected);
    }
    if !is_safe_opaque_ref(trimmed) {
        return Err(AssistDraftExecutorAdapterConfigError::NonOpaqueCredentialHandleRef);
    }
    Ok(())
}

fn validate_audit_tap_ref(value: &str) -> Result<(), AssistDraftExecutorAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AssistDraftExecutorAdapterConfigError::EmptyAuditTapRef);
    }
    if !is_safe_opaque_ref(trimmed) {
        return Err(AssistDraftExecutorAdapterConfigError::InvalidAuditTapRef);
    }
    Ok(())
}

fn validate_executor_audience_ref(
    value: &str,
) -> Result<(), AssistDraftExecutorAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AssistDraftExecutorAdapterConfigError::EmptyExecutorAudienceRef);
    }
    if !is_safe_opaque_ref(trimmed) {
        return Err(AssistDraftExecutorAdapterConfigError::InvalidExecutorAudienceRef);
    }
    Ok(())
}

fn validate_dispatch_request(
    request: &AssistDraftExecutorDispatchRequest,
) -> Result<(), AssistDraftExecutorDispatchFailure> {
    require_metadata(
        "assist-draft executor requires idempotency metadata",
        "validation:assist-draft-executor-idempotency-key",
        &request.idempotency_key,
    )?;

    let domain = &request.domain_request;
    let kernel = &domain.request;
    let policy = &domain.policy_decision;

    require_opaque(
        "assist-draft executor requires tenant metadata",
        "validation:assist-draft-executor-tenant",
        &kernel.tenant_id,
    )?;
    require_opaque(
        "assist-draft executor requires principal metadata",
        "validation:assist-draft-executor-principal",
        &domain.principal_id,
    )?;
    require_opaque(
        "assist-draft executor requires kernel principal metadata",
        "validation:assist-draft-executor-kernel-principal",
        &kernel.principal_id,
    )?;
    require_opaque(
        "assist-draft executor requires brand surface metadata",
        "validation:assist-draft-executor-brand-surface",
        &domain.brand_surface_ref,
    )?;
    require_metadata(
        "assist-draft executor requires locale metadata",
        "validation:assist-draft-executor-locale",
        &domain.locale,
    )?;
    require_opaque(
        "assist-draft executor requires context metadata",
        "validation:assist-draft-executor-context",
        &kernel.context_id,
    )?;
    require_opaque(
        "assist-draft executor requires prompt ref metadata",
        "validation:assist-draft-executor-prompt-ref",
        &kernel.prompt_ref,
    )?;
    require_opaque(
        "assist-draft executor requires target builder metadata",
        "validation:assist-draft-executor-target-builder",
        &kernel.target_builder_ref,
    )?;
    require_opaque(
        "assist-draft executor requires output contract metadata",
        "validation:assist-draft-executor-output-contract",
        &kernel.output_contract_ref,
    )?;
    require_opaque(
        "assist-draft executor requires consent metadata",
        "validation:assist-draft-executor-consent",
        &kernel.consent_grant_ref,
    )?;
    require_opaque(
        "assist-draft executor requires budget metadata",
        "validation:assist-draft-executor-budget",
        &kernel.budget_evidence_ref,
    )?;
    require_opaque(
        "assist-draft executor requires request evidence metadata",
        "validation:assist-draft-executor-request-evidence",
        &kernel.request_evidence_ref,
    )?;
    require_opaque(
        "assist-draft executor requires trace metadata",
        "validation:assist-draft-executor-trace",
        &kernel.trace_context_ref,
    )?;
    require_opaque(
        "assist-draft executor requires policy decision metadata",
        "validation:assist-draft-executor-policy-decision",
        &kernel.policy_decision_ref,
    )?;
    require_opaque(
        "assist-draft executor requires model route metadata",
        "validation:assist-draft-executor-model-route",
        &kernel.model_route_ref,
    )?;
    require_opaque(
        "assist-draft executor requires guardrail metadata",
        "validation:assist-draft-executor-guardrail",
        &kernel.guardrail_evidence_ref,
    )?;
    require_metadata(
        "assist-draft executor requires policy decision id metadata",
        "validation:assist-draft-executor-policy-decision-id",
        &policy.decision_id,
    )?;
    require_opaque(
        "assist-draft executor requires policy tenant metadata",
        "validation:assist-draft-executor-policy-tenant",
        &policy.tenant_id,
    )?;
    require_opaque(
        "assist-draft executor requires policy principal metadata",
        "validation:assist-draft-executor-policy-principal",
        &policy.principal_id,
    )?;
    require_opaque(
        "assist-draft executor requires policy evidence metadata",
        "validation:assist-draft-executor-policy-evidence",
        &policy.evidence_ref,
    )?;
    require_opaque(
        "assist-draft executor requires prompt registry metadata",
        "validation:assist-draft-executor-prompt-registry-snapshot",
        &policy.prompt_registry_snapshot_ref,
    )?;
    require_opaque(
        "assist-draft executor requires cost floor metadata",
        "validation:assist-draft-executor-cost-floor",
        &policy.cost_floor_disclosure_ref,
    )?;
    require_opaque(
        "assist-draft executor requires builder capability metadata",
        "validation:assist-draft-executor-builder-capability",
        &policy.builder_capability_scope_ref,
    )?;

    for prompt_context_ref in &domain.prompt_context_refs {
        require_opaque(
            "assist-draft executor requires prompt context metadata refs",
            "validation:assist-draft-executor-prompt-context-ref",
            prompt_context_ref,
        )?;
    }
    for evidence_ref in &kernel.additional_evidence_refs {
        require_opaque(
            "assist-draft executor requires additional evidence metadata refs",
            "validation:assist-draft-executor-additional-evidence-ref",
            evidence_ref,
        )?;
    }
    if kernel.requested_actions.is_empty() {
        return Err(dispatch_failure(
            "assist-draft executor requires requested actions",
            "validation:assist-draft-executor-requested-actions",
        ));
    }
    if kernel.data_classes.is_empty() {
        return Err(dispatch_failure(
            "assist-draft executor requires data class metadata",
            "validation:assist-draft-executor-data-classes",
        ));
    }
    if domain.principal_id != kernel.principal_id
        || policy.tenant_id != kernel.tenant_id
        || policy.principal_id != domain.principal_id
        || kernel.policy_decision_ref != policy.evidence_ref
    {
        return Err(dispatch_failure(
            "assist-draft executor requires policy-bound domain request",
            "validation:assist-draft-executor-policy-binding",
        ));
    }

    validate_receipt_binding(request)
}

fn validate_receipt_binding(
    request: &AssistDraftExecutorDispatchRequest,
) -> Result<(), AssistDraftExecutorDispatchFailure> {
    let receipt = &request.usecase_receipt;
    let domain = &request.domain_request;
    let kernel = &domain.request;
    let policy = &domain.policy_decision;

    if receipt.status != AssistDraftUsecaseStatus::Planned || receipt.denial_kind.is_some() {
        return Err(dispatch_failure(
            "assist-draft executor requires planned usecase receipt",
            "validation:assist-draft-executor-usecase-receipt-status",
        ));
    }
    if receipt.idempotency_key != request.idempotency_key
        || receipt.tenant_id != kernel.tenant_id
        || receipt.principal_id != domain.principal_id
        || receipt.brand_surface_ref != domain.brand_surface_ref
        || receipt.locale != domain.locale
        || receipt.target_builder_ref != kernel.target_builder_ref
        || receipt.output_contract_ref != kernel.output_contract_ref
        || receipt.cost_floor_disclosure_ref != policy.cost_floor_disclosure_ref
    {
        return Err(dispatch_failure(
            "assist-draft usecase receipt is not bound to domain request",
            "validation:assist-draft-executor-usecase-receipt-binding",
        ));
    }
    if receipt.planned_actions.is_empty() {
        return Err(dispatch_failure(
            "assist-draft executor requires planned actions",
            "validation:assist-draft-executor-planned-actions",
        ));
    }
    let planned_actions = sorted_unique_actions(receipt.planned_actions.clone());
    for action in sorted_unique_actions(kernel.requested_actions.clone()) {
        if !planned_actions.contains(&action) {
            return Err(dispatch_failure(
                "assist-draft planned actions must cover requested actions",
                "validation:assist-draft-executor-planned-action-binding",
            ));
        }
    }
    for evidence_ref in &receipt.evidence_refs {
        require_opaque(
            "assist-draft executor requires receipt evidence metadata refs",
            "validation:assist-draft-executor-receipt-evidence-ref",
            evidence_ref,
        )?;
    }
    Ok(())
}

fn validate_status_metadata(
    status: &AssistDraftExecutorStatus,
) -> Result<(), AssistDraftExecutorDispatchFailure> {
    let valid = match status {
        AssistDraftExecutorStatus::Accepted {
            executor_request_ref,
            draft_ref,
            evidence_ref,
        } => [executor_request_ref, draft_ref, evidence_ref]
            .into_iter()
            .all(|value| is_safe_opaque_ref(value)),
        AssistDraftExecutorStatus::Queued {
            executor_request_ref,
            queue_ref,
            evidence_ref,
        } => [executor_request_ref, queue_ref, evidence_ref]
            .into_iter()
            .all(|value| is_safe_opaque_ref(value)),
        AssistDraftExecutorStatus::Completed {
            executor_request_ref,
            draft_ref,
            suggested_patch_ref,
            evidence_ref,
        } => [
            executor_request_ref,
            draft_ref,
            suggested_patch_ref,
            evidence_ref,
        ]
        .into_iter()
        .all(|value| is_safe_opaque_ref(value)),
        AssistDraftExecutorStatus::Denied { evidence_ref }
        | AssistDraftExecutorStatus::RateLimited { evidence_ref }
        | AssistDraftExecutorStatus::ExecutorError { evidence_ref }
        | AssistDraftExecutorStatus::AuthError { evidence_ref }
        | AssistDraftExecutorStatus::InvalidRequest { evidence_ref }
        | AssistDraftExecutorStatus::Timeout { evidence_ref } => is_safe_opaque_ref(evidence_ref),
    };

    if valid {
        Ok(())
    } else {
        Err(dispatch_failure(
            "executor status metadata is invalid",
            "validation:assist-draft-executor-status-metadata",
        ))
    }
}

fn receipt_from_status(
    status: &AssistDraftExecutorStatus,
) -> Result<AssistDraftExecutorDispatchReceipt, AssistDraftExecutorDispatchFailure> {
    match status {
        AssistDraftExecutorStatus::Accepted {
            executor_request_ref,
            draft_ref,
            evidence_ref,
        } => Ok(AssistDraftExecutorDispatchReceipt {
            status: AssistDraftExecutorDispatchStatus::Accepted,
            executor_request_ref: Some(executor_request_ref.clone()),
            draft_ref: Some(draft_ref.clone()),
            queue_ref: None,
            suggested_patch_ref: None,
            evidence_ref: evidence_ref.clone(),
        }),
        AssistDraftExecutorStatus::Queued {
            executor_request_ref,
            queue_ref,
            evidence_ref,
        } => Ok(AssistDraftExecutorDispatchReceipt {
            status: AssistDraftExecutorDispatchStatus::Queued,
            executor_request_ref: Some(executor_request_ref.clone()),
            draft_ref: None,
            queue_ref: Some(queue_ref.clone()),
            suggested_patch_ref: None,
            evidence_ref: evidence_ref.clone(),
        }),
        AssistDraftExecutorStatus::Completed {
            executor_request_ref,
            draft_ref,
            suggested_patch_ref,
            evidence_ref,
        } => Ok(AssistDraftExecutorDispatchReceipt {
            status: AssistDraftExecutorDispatchStatus::Completed,
            executor_request_ref: Some(executor_request_ref.clone()),
            draft_ref: Some(draft_ref.clone()),
            queue_ref: None,
            suggested_patch_ref: Some(suggested_patch_ref.clone()),
            evidence_ref: evidence_ref.clone(),
        }),
        AssistDraftExecutorStatus::Denied { evidence_ref } => Err(dispatch_failure(
            "executor denied assist-draft request",
            evidence_ref,
        )),
        AssistDraftExecutorStatus::RateLimited { evidence_ref } => Err(dispatch_failure(
            "executor rate limited assist-draft request",
            evidence_ref,
        )),
        AssistDraftExecutorStatus::ExecutorError { evidence_ref } => Err(dispatch_failure(
            "executor failed assist-draft request",
            evidence_ref,
        )),
        AssistDraftExecutorStatus::AuthError { evidence_ref } => Err(dispatch_failure(
            "executor auth failed for assist-draft request",
            evidence_ref,
        )),
        AssistDraftExecutorStatus::InvalidRequest { evidence_ref } => Err(dispatch_failure(
            "executor rejected invalid assist-draft request",
            evidence_ref,
        )),
        AssistDraftExecutorStatus::Timeout { evidence_ref } => Err(dispatch_failure(
            "executor timed out assist-draft request",
            evidence_ref,
        )),
    }
}

fn dispatch_evidence_refs(request: &AssistDraftExecutorDispatchRequest) -> Vec<String> {
    let kernel = &request.domain_request.request;
    let policy = &request.domain_request.policy_decision;
    let mut refs = vec![
        kernel.consent_grant_ref.clone(),
        kernel.budget_evidence_ref.clone(),
        kernel.request_evidence_ref.clone(),
        kernel.trace_context_ref.clone(),
        kernel.policy_decision_ref.clone(),
        policy.evidence_ref.clone(),
        policy.prompt_registry_snapshot_ref.clone(),
        kernel.model_route_ref.clone(),
        kernel.guardrail_evidence_ref.clone(),
        policy.cost_floor_disclosure_ref.clone(),
        policy.builder_capability_scope_ref.clone(),
    ];
    refs.extend(request.usecase_receipt.evidence_refs.clone());
    refs.extend(kernel.additional_evidence_refs.clone());
    sorted_unique(refs)
}

fn require_metadata(
    reason: &str,
    evidence_ref: &str,
    value: &str,
) -> Result<(), AssistDraftExecutorDispatchFailure> {
    if is_safe_metadata_ref(value) {
        Ok(())
    } else {
        Err(dispatch_failure(reason, evidence_ref))
    }
}

fn require_opaque(
    reason: &str,
    evidence_ref: &str,
    value: &str,
) -> Result<(), AssistDraftExecutorDispatchFailure> {
    if is_safe_opaque_ref(value) {
        Ok(())
    } else {
        Err(dispatch_failure(reason, evidence_ref))
    }
}

fn dispatch_failure(reason: &str, evidence_ref: &str) -> AssistDraftExecutorDispatchFailure {
    AssistDraftExecutorDispatchFailure {
        reason: reason.to_owned(),
        evidence_ref: if is_safe_metadata_ref(evidence_ref) {
            evidence_ref.to_owned()
        } else {
            "assist-draft-executor:error:unsafe-evidence-ref".to_owned()
        },
    }
}

fn normalized_endpoint(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_owned()
}

fn is_local_endpoint(endpoint: &str) -> bool {
    let lower = endpoint.to_ascii_lowercase();
    lower.starts_with("https://localhost")
        || lower.starts_with("https://127.")
        || lower.starts_with("https://[::1]")
        || lower.starts_with("https://0.0.0.0")
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

fn sorted_unique_actions(mut values: Vec<AssistDraftAction>) -> Vec<AssistDraftAction> {
    values.sort();
    values.dedup();
    values
}

fn sorted_unique_data_classes(mut values: Vec<AssistDraftDataClass>) -> Vec<AssistDraftDataClass> {
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_metadata_only_assist_draft_executor_envelope() {
        let mut adapter = IntelligenceAssistDraftAdapter::try_new(
            AssistDraftExecutorAdapterConfig::new(
                "https://assist-draft-executor.internal",
                "credential-handle:assist-draft:1",
                "audit-tap:assist-draft:1",
                "draft-executor:assist-draft:workflow-studio",
            ),
            AssistDraftExecutorStatus::Accepted {
                executor_request_ref: "draft-request:assist-draft:1".to_owned(),
                draft_ref: "draft:assist-draft:1".to_owned(),
                evidence_ref: "executor-evidence:assist-draft:accepted".to_owned(),
            },
        )
        .expect("config");

        let receipt = adapter
            .dispatch(valid_dispatch_request())
            .expect("dispatch accepted");

        assert_eq!(receipt.status, AssistDraftExecutorDispatchStatus::Accepted);
        assert_eq!(receipt.draft_ref, Some("draft:assist-draft:1".to_owned()));
        let envelope = adapter.last_envelope().expect("envelope");
        assert_eq!(envelope.method, AssistDraftExecutorHttpMethod::Post);
        assert_eq!(envelope.endpoint, "https://assist-draft-executor.internal");
        assert_eq!(envelope.path, "/v1/assist-draft-executions");
        assert_eq!(
            envelope.transport_mode,
            AssistDraftExecutorTransportMode::EnvelopeOnly
        );
        assert_eq!(envelope.tenant_id, "tenant:alpha");
        assert_eq!(envelope.principal_id, "principal:builder-owner");
        assert_eq!(envelope.locale, "en-US");
        assert_eq!(
            envelope.target_builder_ref,
            "builder://workflow-studio/canvas-1"
        );
        assert_eq!(envelope.output_contract_ref, "workflow-spec://contracts/v1");
        assert_eq!(
            envelope.cost_floor_disclosure_ref,
            "cost-floor:assist-draft:workflow-studio"
        );
        assert_eq!(
            envelope.prompt_context_refs,
            vec!["context-snippet:workflow-studio:canvas-1".to_owned()]
        );
        assert_eq!(
            envelope.planned_actions,
            vec![
                AssistDraftAction::CreateDraft,
                AssistDraftAction::ExplainDraft
            ]
        );
        assert!(
            envelope
                .evidence_refs
                .contains(&"policy:assist-draft:allow".to_owned())
        );
        assert!(
            envelope
                .adapter_reference_refs
                .contains(&"spec://oyatie/intelligence/assist-draft-adapter-foundation".to_owned())
        );
    }

    #[test]
    fn rejects_non_https_and_localhost_endpoints() {
        assert_eq!(
            IntelligenceAssistDraftAdapter::try_new(
                AssistDraftExecutorAdapterConfig::new(
                    "http://assist-draft-executor.internal",
                    "credential-handle:assist-draft:1",
                    "audit-tap:assist-draft:1",
                    "draft-executor:assist-draft:workflow-studio",
                ),
                accepted_status(),
            )
            .unwrap_err(),
            AssistDraftExecutorAdapterConfigError::NonHttpsEndpoint
        );
        assert_eq!(
            IntelligenceAssistDraftAdapter::try_new(
                AssistDraftExecutorAdapterConfig::new(
                    "https://localhost:8080",
                    "credential-handle:assist-draft:1",
                    "audit-tap:assist-draft:1",
                    "draft-executor:assist-draft:workflow-studio",
                ),
                accepted_status(),
            )
            .unwrap_err(),
            AssistDraftExecutorAdapterConfigError::LocalEndpointDenied
        );
    }

    #[test]
    fn rejects_raw_secret_like_credential_handles() {
        assert_eq!(
            IntelligenceAssistDraftAdapter::try_new(
                AssistDraftExecutorAdapterConfig::new(
                    "https://assist-draft-executor.internal",
                    "sk-secret",
                    "audit-tap:assist-draft:1",
                    "draft-executor:assist-draft:workflow-studio",
                ),
                accepted_status(),
            )
            .unwrap_err(),
            AssistDraftExecutorAdapterConfigError::RawCredentialMaterialRejected
        );
    }

    #[test]
    fn rejects_raw_prompt_output_or_document_shaped_refs_before_envelope() {
        let mut adapter = valid_adapter();
        let mut request = valid_dispatch_request();
        request.domain_request.request.prompt_ref =
            "raw prompt: write an email with secret".to_owned();
        request
            .domain_request
            .prompt_context_refs
            .push("document=raw output".to_owned());

        let failure = adapter.dispatch(request).expect_err("invalid dispatch");

        assert_eq!(adapter.last_envelope(), None);
        let debug = format!("{failure:?}");
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("document=raw output"));
    }

    #[test]
    fn rejects_denied_or_mismatched_usecase_receipts_before_envelope() {
        let mut adapter = valid_adapter();
        let mut denied = valid_dispatch_request();
        denied.usecase_receipt.status = AssistDraftUsecaseStatus::Denied;
        denied.usecase_receipt.denial_kind = Some(AssistDraftUsecaseDenialKind::DomainDenied);

        let failure = adapter.dispatch(denied).expect_err("denied receipt");
        assert!(failure.reason.contains("planned usecase receipt"));
        assert_eq!(adapter.last_envelope(), None);

        let mut mismatched = valid_dispatch_request();
        mismatched.usecase_receipt.tenant_id = "tenant:other".to_owned();
        let failure = adapter
            .dispatch(mismatched)
            .expect_err("mismatched receipt");
        assert!(failure.reason.contains("bound to domain request"));
        assert_eq!(adapter.last_envelope(), None);
    }

    #[test]
    fn rejects_policy_drifted_domain_request_before_envelope() {
        let mut adapter = valid_adapter();
        let mut request = valid_dispatch_request();
        request.domain_request.policy_decision.tenant_id = "tenant:other".to_owned();

        let failure = adapter.dispatch(request).expect_err("policy drift");

        assert_eq!(
            failure.reason,
            "assist-draft executor requires policy-bound domain request"
        );
        assert_eq!(adapter.last_envelope(), None);
    }

    #[test]
    fn maps_executor_outcomes_distinctly() {
        let statuses = vec![
            (
                accepted_status(),
                Ok(AssistDraftExecutorDispatchStatus::Accepted),
            ),
            (
                AssistDraftExecutorStatus::Queued {
                    executor_request_ref: "draft-request:assist-draft:1".to_owned(),
                    queue_ref: "queue:assist-draft:1".to_owned(),
                    evidence_ref: "executor-evidence:assist-draft:queued".to_owned(),
                },
                Ok(AssistDraftExecutorDispatchStatus::Queued),
            ),
            (
                AssistDraftExecutorStatus::Completed {
                    executor_request_ref: "draft-request:assist-draft:1".to_owned(),
                    draft_ref: "draft:assist-draft:1".to_owned(),
                    suggested_patch_ref: "suggested-patch:assist-draft:1".to_owned(),
                    evidence_ref: "executor-evidence:assist-draft:completed".to_owned(),
                },
                Ok(AssistDraftExecutorDispatchStatus::Completed),
            ),
            (
                AssistDraftExecutorStatus::Denied {
                    evidence_ref: "executor-evidence:assist-draft:denied".to_owned(),
                },
                Err("executor denied assist-draft request"),
            ),
            (
                AssistDraftExecutorStatus::RateLimited {
                    evidence_ref: "executor-evidence:assist-draft:rate-limited".to_owned(),
                },
                Err("executor rate limited assist-draft request"),
            ),
            (
                AssistDraftExecutorStatus::ExecutorError {
                    evidence_ref: "executor-evidence:assist-draft:error".to_owned(),
                },
                Err("executor failed assist-draft request"),
            ),
            (
                AssistDraftExecutorStatus::AuthError {
                    evidence_ref: "executor-evidence:assist-draft:auth".to_owned(),
                },
                Err("executor auth failed for assist-draft request"),
            ),
            (
                AssistDraftExecutorStatus::InvalidRequest {
                    evidence_ref: "executor-evidence:assist-draft:invalid".to_owned(),
                },
                Err("executor rejected invalid assist-draft request"),
            ),
            (
                AssistDraftExecutorStatus::Timeout {
                    evidence_ref: "executor-evidence:assist-draft:timeout".to_owned(),
                },
                Err("executor timed out assist-draft request"),
            ),
        ];

        for (status, expected) in statuses {
            let mut adapter =
                IntelligenceAssistDraftAdapter::try_new(valid_config(), status).expect("config");
            let result = adapter.dispatch(valid_dispatch_request());
            match expected {
                Ok(dispatch_status) => assert_eq!(result.expect("success").status, dispatch_status),
                Err(reason) => assert_eq!(result.expect_err("failure").reason, reason),
            }
        }
    }

    #[test]
    fn rejects_invalid_executor_status_metadata() {
        let mut adapter = IntelligenceAssistDraftAdapter::try_new(
            valid_config(),
            AssistDraftExecutorStatus::Completed {
                executor_request_ref: "draft-request:assist-draft:1".to_owned(),
                draft_ref: "raw output: generated draft".to_owned(),
                suggested_patch_ref: "suggested-patch:assist-draft:1".to_owned(),
                evidence_ref: "executor-evidence:assist-draft:completed".to_owned(),
            },
        )
        .expect("config");

        let failure = adapter
            .dispatch(valid_dispatch_request())
            .expect_err("invalid status");
        assert_eq!(failure.reason, "executor status metadata is invalid");
        assert_eq!(adapter.last_envelope(), None);
        let debug = format!("{failure:?}");
        assert!(!debug.contains("raw output"));
    }

    #[test]
    fn envelope_and_receipts_never_contain_raw_prompt_output_document_or_secret_bytes() {
        let mut adapter = valid_adapter();
        let receipt = adapter
            .dispatch(valid_dispatch_request())
            .expect("dispatch");
        let rendered = format!("{:?}{:?}", adapter.last_envelope(), receipt);

        assert!(!rendered.contains("raw prompt"));
        assert!(!rendered.contains("raw output"));
        assert!(!rendered.contains("document="));
        assert!(!rendered.contains("sk-secret"));
        assert!(!rendered.contains("write an email"));
    }

    fn valid_adapter() -> IntelligenceAssistDraftAdapter {
        IntelligenceAssistDraftAdapter::try_new(valid_config(), accepted_status()).expect("config")
    }

    fn valid_config() -> AssistDraftExecutorAdapterConfig {
        AssistDraftExecutorAdapterConfig::new(
            "https://assist-draft-executor.internal",
            "credential-handle:assist-draft:1",
            "audit-tap:assist-draft:1",
            "draft-executor:assist-draft:workflow-studio",
        )
    }

    fn accepted_status() -> AssistDraftExecutorStatus {
        AssistDraftExecutorStatus::Accepted {
            executor_request_ref: "draft-request:assist-draft:1".to_owned(),
            draft_ref: "draft:assist-draft:1".to_owned(),
            evidence_ref: "executor-evidence:assist-draft:accepted".to_owned(),
        }
    }

    fn valid_dispatch_request() -> AssistDraftExecutorDispatchRequest {
        AssistDraftExecutorDispatchRequest {
            idempotency_key: "idempotency:assist-draft:1".to_owned(),
            domain_request: valid_domain_request(),
            usecase_receipt: valid_usecase_receipt(),
        }
    }

    fn valid_usecase_receipt() -> AssistDraftUsecaseReceipt {
        AssistDraftUsecaseReceipt {
            idempotency_key: "idempotency:assist-draft:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:builder-owner".to_owned(),
            brand_surface_ref: "brand-surface:workflow-studio:assist".to_owned(),
            locale: "en-US".to_owned(),
            target_builder_ref: "builder://workflow-studio/canvas-1".to_owned(),
            output_contract_ref: "workflow-spec://contracts/v1".to_owned(),
            cost_floor_disclosure_ref: "cost-floor:assist-draft:workflow-studio".to_owned(),
            status: AssistDraftUsecaseStatus::Planned,
            denial_kind: None,
            domain_denial_kind: None,
            kernel_reasons: Vec::new(),
            denial_reasons: Vec::new(),
            planned_actions: vec![
                AssistDraftAction::CreateDraft,
                AssistDraftAction::ExplainDraft,
            ],
            refusal_banner: None,
            evidence_refs: vec![
                "budget:assist-draft:1".to_owned(),
                "consent:assist-draft:1".to_owned(),
                "cost-floor:assist-draft:workflow-studio".to_owned(),
                "guardrail:assist-draft:allow".to_owned(),
                "model-route:assist-draft:1".to_owned(),
                "policy:assist-draft:allow".to_owned(),
                "prompt-registry:assist-draft:v1".to_owned(),
                "request:assist-draft:1".to_owned(),
                "trace:assist-draft:1".to_owned(),
            ],
        }
    }

    fn valid_domain_request() -> DomainAssistDraftRequest {
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
