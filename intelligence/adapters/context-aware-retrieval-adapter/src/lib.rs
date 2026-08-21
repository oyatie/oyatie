//! Intelligence context-aware retrieval adapter foundation.
//!
//! This crate defines a deterministic, metadata-only adapter seam for future
//! context retrieval executor integration. It builds executor request envelopes
//! from the retrieval domain/usecase metadata already authorized by the preview
//! foundation and maps executor outcome metadata into stable receipts. It
//! performs no network I/O, vector-store calls, embedding generation,
//! ontology/KG execution, document fetch, filesystem access, credential
//! resolution, durable idempotency, or durable audit-chain emission.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_context_aware_retrieval_usecase::{
    ContextAudience, ContextCandidate, ContextDataClass, ContextRetrievalDomainDenialKind,
    ContextRetrievalPolicyDecision, ContextRetrievalRequest, ContextRetrievalUsecaseDenialKind,
    ContextRetrievalUsecaseReceipt, ContextRetrievalUsecaseStatus, ContextSourceKind,
    DomainContextRetrievalRequest,
};

const CONTEXT_RETRIEVAL_EXECUTOR_PATH: &str = "/v1/context-retrieval/executions";
const ADAPTER_REFERENCE_REF: &str =
    "spec://oyatie/intelligence/context-aware-retrieval-adapter-foundation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRetrievalExecutorHttpMethod {
    Post,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRetrievalExecutorTransportMode {
    EnvelopeOnly,
    HostedExecutor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalExecutorAdapterConfig {
    pub endpoint: String,              // data_class: INTERNAL_ONLY
    pub credential_handle_ref: String, // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,         // data_class: INTERNAL_ONLY
    pub executor_audience_ref: String, // data_class: INTERNAL_ONLY
    pub transport_mode: ContextRetrievalExecutorTransportMode, // data_class: INTERNAL_ONLY
}

impl ContextRetrievalExecutorAdapterConfig {
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
            transport_mode: ContextRetrievalExecutorTransportMode::EnvelopeOnly,
        }
    }

    pub fn with_transport_mode(
        mut self,
        transport_mode: ContextRetrievalExecutorTransportMode,
    ) -> Self {
        self.transport_mode = transport_mode;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRetrievalExecutorAdapterConfigError {
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
pub struct ContextRetrievalExecutorDispatchRequest {
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub domain_request: DomainContextRetrievalRequest, // data_class: INTERNAL_ONLY
    pub usecase_receipt: ContextRetrievalUsecaseReceipt, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalExecutorRequestEnvelope {
    pub method: ContextRetrievalExecutorHttpMethod, // data_class: PUBLIC
    pub endpoint: String,                           // data_class: INTERNAL_ONLY
    pub path: String,                               // data_class: PUBLIC
    pub transport_mode: ContextRetrievalExecutorTransportMode, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub principal_id: String,                       // data_class: INTERNAL_ONLY
    pub query_surface: String,                      // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                    // data_class: INTERNAL_ONLY
    pub query_ref: String,                          // data_class: INTERNAL_ONLY
    pub audience: ContextAudience,                  // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,                  // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,                // data_class: INTERNAL_ONLY
    pub retrieval_index_snapshot_ref: String,       // data_class: INTERNAL_ONLY
    pub credential_handle_ref: String,              // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,                      // data_class: INTERNAL_ONLY
    pub executor_audience_ref: String,              // data_class: INTERNAL_ONLY
    pub max_context_items: usize,                   // data_class: INTERNAL_ONLY
    pub freshness_floor_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub planned_context_count: usize,               // data_class: INTERNAL_ONLY
    pub plan_resource_refs: Vec<String>,            // data_class: INTERNAL_ONLY
    pub candidate_resource_refs: Vec<String>,       // data_class: INTERNAL_ONLY
    pub candidate_evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
    pub candidate_source_kinds: Vec<ContextSourceKind>, // data_class: INTERNAL_ONLY
    pub allowed_source_kinds: Vec<ContextSourceKind>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                 // data_class: INTERNAL_ONLY
    pub adapter_reference_refs: Vec<String>,        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextRetrievalExecutorStatus {
    Accepted {
        executor_request_ref: String,
        retrieval_ref: String,
        evidence_ref: String,
    },
    Queued {
        executor_request_ref: String,
        queue_ref: String,
        evidence_ref: String,
    },
    Completed {
        executor_request_ref: String,
        retrieval_ref: String,
        context_bundle_ref: String,
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
pub enum ContextRetrievalExecutorDispatchStatus {
    Accepted,
    Queued,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalExecutorDispatchReceipt {
    pub status: ContextRetrievalExecutorDispatchStatus, // data_class: PUBLIC
    pub executor_request_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub retrieval_ref: Option<String>,                  // data_class: INTERNAL_ONLY
    pub queue_ref: Option<String>,                      // data_class: INTERNAL_ONLY
    pub context_bundle_ref: Option<String>,             // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRetrievalExecutorDispatchFailure {
    pub reason: String,       // data_class: INTERNAL_ONLY
    pub evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Debug)]
pub struct IntelligenceContextAwareRetrievalAdapter {
    config: ContextRetrievalExecutorAdapterConfig,
    next_status: ContextRetrievalExecutorStatus,
    last_envelope: Option<ContextRetrievalExecutorRequestEnvelope>,
}

impl IntelligenceContextAwareRetrievalAdapter {
    pub fn try_new(
        config: ContextRetrievalExecutorAdapterConfig,
        next_status: ContextRetrievalExecutorStatus,
    ) -> Result<Self, ContextRetrievalExecutorAdapterConfigError> {
        validate_config(&config)?;
        Ok(Self {
            config,
            next_status,
            last_envelope: None,
        })
    }

    pub fn last_envelope(&self) -> Option<&ContextRetrievalExecutorRequestEnvelope> {
        self.last_envelope.as_ref()
    }

    pub fn set_next_status(&mut self, next_status: ContextRetrievalExecutorStatus) {
        self.next_status = next_status;
    }

    pub fn dispatch(
        &mut self,
        request: ContextRetrievalExecutorDispatchRequest,
    ) -> Result<ContextRetrievalExecutorDispatchReceipt, ContextRetrievalExecutorDispatchFailure>
    {
        validate_dispatch_request(&request)?;
        let envelope = self.build_envelope(&request);
        self.last_envelope = Some(envelope);
        receipt_from_status(&self.next_status)
    }

    fn build_envelope(
        &self,
        request: &ContextRetrievalExecutorDispatchRequest,
    ) -> ContextRetrievalExecutorRequestEnvelope {
        let retrieval_request = &request.domain_request.request;
        ContextRetrievalExecutorRequestEnvelope {
            method: ContextRetrievalExecutorHttpMethod::Post,
            endpoint: normalized_endpoint(&self.config.endpoint),
            path: CONTEXT_RETRIEVAL_EXECUTOR_PATH.to_owned(),
            transport_mode: self.config.transport_mode,
            tenant_id: retrieval_request.tenant_id.clone(),
            principal_id: request.domain_request.principal_id.clone(),
            query_surface: request.domain_request.query_surface.clone(),
            idempotency_key: request.idempotency_key.clone(),
            query_ref: retrieval_request.query_ref.clone(),
            audience: retrieval_request.audience,
            request_evidence_ref: retrieval_request.request_evidence_ref.clone(),
            trace_context_ref: retrieval_request.trace_context_ref.clone(),
            policy_decision_ref: retrieval_request.policy_decision_ref.clone(),
            retrieval_index_snapshot_ref: request
                .domain_request
                .policy_decision
                .retrieval_index_snapshot_ref
                .clone(),
            credential_handle_ref: self.config.credential_handle_ref.clone(),
            audit_tap_ref: self.config.audit_tap_ref.clone(),
            executor_audience_ref: self.config.executor_audience_ref.clone(),
            max_context_items: retrieval_request.max_context_items,
            freshness_floor_epoch_seconds: retrieval_request.freshness_floor_epoch_seconds,
            planned_context_count: request.usecase_receipt.plan_resource_refs.len(),
            plan_resource_refs: sorted_unique(request.usecase_receipt.plan_resource_refs.clone()),
            candidate_resource_refs: sorted_unique(
                retrieval_request
                    .candidates
                    .iter()
                    .map(|candidate| candidate.resource_ref.clone())
                    .collect(),
            ),
            candidate_evidence_refs: sorted_unique(
                retrieval_request
                    .candidates
                    .iter()
                    .map(|candidate| candidate.evidence_ref.clone())
                    .collect(),
            ),
            candidate_source_kinds: sorted_unique_source_kinds(
                retrieval_request
                    .candidates
                    .iter()
                    .map(|candidate| candidate.source_kind)
                    .collect(),
            ),
            allowed_source_kinds: sorted_unique_source_kinds(
                retrieval_request.allowed_source_kinds.clone(),
            ),
            evidence_refs: dispatch_evidence_refs(request),
            adapter_reference_refs: vec![ADAPTER_REFERENCE_REF.to_owned()],
        }
    }
}

fn validate_config(
    config: &ContextRetrievalExecutorAdapterConfig,
) -> Result<(), ContextRetrievalExecutorAdapterConfigError> {
    let endpoint = config.endpoint.trim();
    if endpoint.is_empty() {
        return Err(ContextRetrievalExecutorAdapterConfigError::EmptyEndpoint);
    }
    if !endpoint.starts_with("https://") || contains_whitespace(endpoint) {
        return Err(ContextRetrievalExecutorAdapterConfigError::NonHttpsEndpoint);
    }
    if is_local_endpoint(endpoint) {
        return Err(ContextRetrievalExecutorAdapterConfigError::LocalEndpointDenied);
    }
    validate_credential_handle_ref(&config.credential_handle_ref)?;
    validate_audit_tap_ref(&config.audit_tap_ref)?;
    validate_executor_audience_ref(&config.executor_audience_ref)?;
    Ok(())
}

fn validate_credential_handle_ref(
    value: &str,
) -> Result<(), ContextRetrievalExecutorAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ContextRetrievalExecutorAdapterConfigError::EmptyCredentialHandleRef);
    }
    if !is_safe_double_colon_ref(trimmed) {
        return Err(if contains_raw_secret_material(trimmed) {
            ContextRetrievalExecutorAdapterConfigError::RawCredentialMaterialRejected
        } else {
            ContextRetrievalExecutorAdapterConfigError::NonOpaqueCredentialHandleRef
        });
    }
    Ok(())
}

fn validate_audit_tap_ref(value: &str) -> Result<(), ContextRetrievalExecutorAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ContextRetrievalExecutorAdapterConfigError::EmptyAuditTapRef);
    }
    if !is_safe_double_colon_ref(trimmed) {
        return Err(ContextRetrievalExecutorAdapterConfigError::InvalidAuditTapRef);
    }
    Ok(())
}

fn validate_executor_audience_ref(
    value: &str,
) -> Result<(), ContextRetrievalExecutorAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ContextRetrievalExecutorAdapterConfigError::EmptyExecutorAudienceRef);
    }
    if !is_safe_double_colon_ref(trimmed) {
        return Err(ContextRetrievalExecutorAdapterConfigError::InvalidExecutorAudienceRef);
    }
    Ok(())
}

fn validate_dispatch_request(
    request: &ContextRetrievalExecutorDispatchRequest,
) -> Result<(), ContextRetrievalExecutorDispatchFailure> {
    require_metadata(
        "context-retrieval-executor:idempotency_key_required",
        "validation:context-retrieval-executor-idempotency-key",
        &request.idempotency_key,
    )?;

    let domain = &request.domain_request;
    let retrieval_request = &domain.request;
    require_tenant(
        "context-retrieval-executor:tenant_required",
        "validation:context-retrieval-executor-tenant",
        &retrieval_request.tenant_id,
    )?;
    require_metadata(
        "context-retrieval-executor:principal_required",
        "validation:context-retrieval-executor-principal",
        &domain.principal_id,
    )?;
    require_metadata(
        "context-retrieval-executor:surface_required",
        "validation:context-retrieval-executor-surface",
        &domain.query_surface,
    )?;
    require_opaque(
        "context-retrieval-executor:query_ref_required",
        "validation:context-retrieval-executor-query-ref",
        &retrieval_request.query_ref,
    )?;
    require_opaque(
        "context-retrieval-executor:request_evidence_required",
        "validation:context-retrieval-executor-request-evidence",
        &retrieval_request.request_evidence_ref,
    )?;
    require_opaque(
        "context-retrieval-executor:trace_context_required",
        "validation:context-retrieval-executor-trace-context",
        &retrieval_request.trace_context_ref,
    )?;
    require_opaque(
        "context-retrieval-executor:policy_decision_required",
        "validation:context-retrieval-executor-policy-decision",
        &retrieval_request.policy_decision_ref,
    )?;
    require_opaque(
        "context-retrieval-executor:retrieval_index_snapshot_required",
        "validation:context-retrieval-executor-index-snapshot",
        &domain.policy_decision.retrieval_index_snapshot_ref,
    )?;
    if retrieval_request.allowed_source_kinds.is_empty() {
        return Err(dispatch_failure(
            "context-retrieval-executor:allowed_source_kinds_required",
            "validation:context-retrieval-executor-source-kinds",
        ));
    }
    if retrieval_request.candidates.is_empty() {
        return Err(dispatch_failure(
            "context-retrieval-executor:candidate_metadata_required",
            "validation:context-retrieval-executor-candidate-metadata",
        ));
    }
    for candidate in &retrieval_request.candidates {
        require_tenant(
            "context-retrieval-executor:candidate_tenant_required",
            "validation:context-retrieval-executor-candidate-tenant",
            &candidate.tenant_id,
        )?;
        require_opaque(
            "context-retrieval-executor:candidate_resource_ref_required",
            "validation:context-retrieval-executor-candidate-resource-ref",
            &candidate.resource_ref,
        )?;
        require_opaque(
            "context-retrieval-executor:candidate_evidence_ref_required",
            "validation:context-retrieval-executor-candidate-evidence-ref",
            &candidate.evidence_ref,
        )?;
        if candidate.relevance_millis > 1000 {
            return Err(dispatch_failure(
                "context-retrieval-executor:candidate_relevance_invalid",
                "validation:context-retrieval-executor-candidate-relevance",
            ));
        }
    }

    validate_receipt_binding(request)
}

fn validate_receipt_binding(
    request: &ContextRetrievalExecutorDispatchRequest,
) -> Result<(), ContextRetrievalExecutorDispatchFailure> {
    let receipt = &request.usecase_receipt;
    let domain = &request.domain_request;
    let retrieval_request = &domain.request;
    if receipt.idempotency_key != request.idempotency_key
        || receipt.tenant_id != retrieval_request.tenant_id
        || receipt.principal_id != domain.principal_id
        || receipt.query_surface != domain.query_surface
        || receipt.query_ref != retrieval_request.query_ref
    {
        return Err(dispatch_failure(
            "context-retrieval-executor:usecase_receipt_binding_mismatch",
            "validation:context-retrieval-executor-usecase-receipt-binding",
        ));
    }
    if receipt.status != ContextRetrievalUsecaseStatus::Planned || receipt.denial_kind.is_some() {
        return Err(dispatch_failure(
            "context-retrieval-executor:usecase_receipt_not_planned",
            "validation:context-retrieval-executor-usecase-receipt-status",
        ));
    }
    if receipt.plan_resource_refs.is_empty()
        || receipt.plan_resource_refs.len() > retrieval_request.max_context_items
    {
        return Err(dispatch_failure(
            "context-retrieval-executor:usecase_receipt_invalid_plan_count",
            "validation:context-retrieval-executor-plan-count",
        ));
    }
    let candidate_resource_refs: Vec<&str> = retrieval_request
        .candidates
        .iter()
        .map(|candidate| candidate.resource_ref.as_str())
        .collect();
    for resource_ref in &receipt.plan_resource_refs {
        require_opaque(
            "context-retrieval-executor:receipt_plan_resource_ref_required",
            "validation:context-retrieval-executor-receipt-resource-ref",
            resource_ref,
        )?;
        if !candidate_resource_refs.contains(&resource_ref.as_str()) {
            return Err(dispatch_failure(
                "context-retrieval-executor:receipt_plan_resource_outside_request",
                "validation:context-retrieval-executor-receipt-resource-binding",
            ));
        }
    }
    for evidence_ref in &receipt.evidence_refs {
        require_opaque(
            "context-retrieval-executor:receipt_evidence_ref_required",
            "validation:context-retrieval-executor-receipt-evidence-ref",
            evidence_ref,
        )?;
    }
    Ok(())
}

fn receipt_from_status(
    status: &ContextRetrievalExecutorStatus,
) -> Result<ContextRetrievalExecutorDispatchReceipt, ContextRetrievalExecutorDispatchFailure> {
    match status {
        ContextRetrievalExecutorStatus::Accepted {
            executor_request_ref,
            retrieval_ref,
            evidence_ref,
        } => ok_receipt(
            ContextRetrievalExecutorDispatchStatus::Accepted,
            Some(executor_request_ref),
            Some(retrieval_ref),
            None,
            None,
            evidence_ref,
        ),
        ContextRetrievalExecutorStatus::Queued {
            executor_request_ref,
            queue_ref,
            evidence_ref,
        } => ok_receipt(
            ContextRetrievalExecutorDispatchStatus::Queued,
            Some(executor_request_ref),
            None,
            Some(queue_ref),
            None,
            evidence_ref,
        ),
        ContextRetrievalExecutorStatus::Completed {
            executor_request_ref,
            retrieval_ref,
            context_bundle_ref,
            evidence_ref,
        } => ok_receipt(
            ContextRetrievalExecutorDispatchStatus::Completed,
            Some(executor_request_ref),
            Some(retrieval_ref),
            None,
            Some(context_bundle_ref),
            evidence_ref,
        ),
        ContextRetrievalExecutorStatus::Denied { evidence_ref } => Err(dispatch_failure(
            "context-retrieval-executor:denied",
            evidence_ref,
        )),
        ContextRetrievalExecutorStatus::RateLimited { evidence_ref } => Err(dispatch_failure(
            "context-retrieval-executor:rate_limited",
            evidence_ref,
        )),
        ContextRetrievalExecutorStatus::ExecutorError { evidence_ref } => Err(dispatch_failure(
            "context-retrieval-executor:executor_error",
            evidence_ref,
        )),
        ContextRetrievalExecutorStatus::AuthError { evidence_ref } => Err(dispatch_failure(
            "context-retrieval-executor:auth_error",
            evidence_ref,
        )),
        ContextRetrievalExecutorStatus::InvalidRequest { evidence_ref } => Err(dispatch_failure(
            "context-retrieval-executor:invalid_request",
            evidence_ref,
        )),
        ContextRetrievalExecutorStatus::Timeout { evidence_ref } => Err(dispatch_failure(
            "context-retrieval-executor:timeout",
            evidence_ref,
        )),
    }
}

fn ok_receipt(
    status: ContextRetrievalExecutorDispatchStatus,
    executor_request_ref: Option<&String>,
    retrieval_ref: Option<&String>,
    queue_ref: Option<&String>,
    context_bundle_ref: Option<&String>,
    evidence_ref: &str,
) -> Result<ContextRetrievalExecutorDispatchReceipt, ContextRetrievalExecutorDispatchFailure> {
    let refs = [
        executor_request_ref,
        retrieval_ref,
        queue_ref,
        context_bundle_ref,
    ];
    if refs
        .into_iter()
        .flatten()
        .any(|value| !is_safe_opaque_ref(value))
        || !is_safe_opaque_ref(evidence_ref)
    {
        return Err(dispatch_failure(
            "context-retrieval-executor:invalid_executor_status",
            "validation:context-retrieval-executor-status-metadata",
        ));
    }
    Ok(ContextRetrievalExecutorDispatchReceipt {
        status,
        executor_request_ref: executor_request_ref.cloned(),
        retrieval_ref: retrieval_ref.cloned(),
        queue_ref: queue_ref.cloned(),
        context_bundle_ref: context_bundle_ref.cloned(),
        evidence_ref: evidence_ref.to_owned(),
    })
}

fn dispatch_evidence_refs(request: &ContextRetrievalExecutorDispatchRequest) -> Vec<String> {
    let retrieval_request = &request.domain_request.request;
    let mut refs = vec![
        retrieval_request.request_evidence_ref.clone(),
        retrieval_request.trace_context_ref.clone(),
        retrieval_request.policy_decision_ref.clone(),
        request.domain_request.policy_decision.evidence_ref.clone(),
        request
            .domain_request
            .policy_decision
            .retrieval_index_snapshot_ref
            .clone(),
    ];
    refs.extend(request.usecase_receipt.evidence_refs.clone());
    refs.extend(
        retrieval_request
            .candidates
            .iter()
            .map(|candidate| candidate.evidence_ref.clone()),
    );
    sorted_unique(refs)
}

fn require_tenant(
    reason: &str,
    evidence_ref: &str,
    value: &str,
) -> Result<(), ContextRetrievalExecutorDispatchFailure> {
    if is_safe_tenant_id(value) {
        Ok(())
    } else {
        Err(dispatch_failure(reason, evidence_ref))
    }
}

fn require_metadata(
    reason: &str,
    evidence_ref: &str,
    value: &str,
) -> Result<(), ContextRetrievalExecutorDispatchFailure> {
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
) -> Result<(), ContextRetrievalExecutorDispatchFailure> {
    if is_safe_opaque_ref(value) {
        Ok(())
    } else {
        Err(dispatch_failure(reason, evidence_ref))
    }
}

fn dispatch_failure(reason: &str, evidence_ref: &str) -> ContextRetrievalExecutorDispatchFailure {
    ContextRetrievalExecutorDispatchFailure {
        reason: reason.to_owned(),
        evidence_ref: if is_safe_metadata_ref(evidence_ref) {
            evidence_ref.to_owned()
        } else {
            "context-retrieval-executor:error:unsafe-evidence-ref".to_owned()
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

fn is_safe_double_colon_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.contains("://")
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
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

fn is_safe_tenant_id(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.starts_with("ten_")
        && !trimmed.contains('/')
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
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw query")
        || lower.contains("customer message")
        || lower.contains("write an email")
        || lower.contains("model answer")
        || lower.contains("document text")
        || lower.contains("prompt=")
        || lower.contains("completion=")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn sorted_unique_source_kinds(mut values: Vec<ContextSourceKind>) -> Vec<ContextSourceKind> {
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_metadata_only_retrieval_executor_envelope() {
        let mut adapter = valid_adapter(ContextRetrievalExecutorStatus::Accepted {
            executor_request_ref: "retrieval-executor://requests/req-1".to_owned(),
            retrieval_ref: "retrieval-executor://retrievals/run-1".to_owned(),
            evidence_ref: "retrieval-executor:evidence:accepted".to_owned(),
        });

        let receipt = adapter
            .dispatch(valid_request())
            .expect("executor accepts request");
        let envelope = adapter.last_envelope().expect("envelope recorded");

        assert_eq!(envelope.method, ContextRetrievalExecutorHttpMethod::Post);
        assert_eq!(envelope.path, "/v1/context-retrieval/executions");
        assert_eq!(envelope.tenant_id, "ten_a");
        assert_eq!(envelope.query_ref, "queryref://opaque/adapter-1");
        assert_eq!(envelope.planned_context_count, 2);
        assert_eq!(
            envelope.transport_mode,
            ContextRetrievalExecutorTransportMode::EnvelopeOnly
        );
        assert_eq!(
            envelope.credential_handle_ref,
            "secretref://ten_a/retrieval-executor/byok"
        );
        assert!(
            envelope
                .evidence_refs
                .contains(&"retrieval-index:snapshot:adapter:1".to_owned())
        );
        assert_eq!(
            receipt.status,
            ContextRetrievalExecutorDispatchStatus::Accepted
        );
        assert_eq!(
            receipt.retrieval_ref,
            Some("retrieval-executor://retrievals/run-1".to_owned())
        );
    }

    #[test]
    fn rejects_raw_secret_like_credential_handles() {
        let config = ContextRetrievalExecutorAdapterConfig::new(
            "https://retrieval-executor.oyatie.internal",
            "sk-test-raw-secret",
            "audit://tap/intelligence/context-aware-retrieval",
            "audience://intelligence/context-retrieval-executor",
        );

        let error = IntelligenceContextAwareRetrievalAdapter::try_new(
            config,
            ContextRetrievalExecutorStatus::Timeout {
                evidence_ref: "retrieval-executor:error:timeout".to_owned(),
            },
        )
        .expect_err("raw secret handle rejected");

        assert_eq!(
            error,
            ContextRetrievalExecutorAdapterConfigError::RawCredentialMaterialRejected
        );
    }

    #[test]
    fn rejects_non_https_and_localhost_endpoints() {
        let non_https = ContextRetrievalExecutorAdapterConfig::new(
            "http://retrieval-executor.oyatie.internal",
            "secretref://ten_a/retrieval-executor/byok",
            "audit://tap/intelligence/context-aware-retrieval",
            "audience://intelligence/context-retrieval-executor",
        );
        let local = ContextRetrievalExecutorAdapterConfig::new(
            "https://localhost:9445",
            "secretref://ten_a/retrieval-executor/byok",
            "audit://tap/intelligence/context-aware-retrieval",
            "audience://intelligence/context-retrieval-executor",
        );

        assert_eq!(
            IntelligenceContextAwareRetrievalAdapter::try_new(
                non_https,
                ContextRetrievalExecutorStatus::Timeout {
                    evidence_ref: "retrieval-executor:error:timeout".to_owned(),
                }
            )
            .expect_err("non-https rejected"),
            ContextRetrievalExecutorAdapterConfigError::NonHttpsEndpoint
        );
        assert_eq!(
            IntelligenceContextAwareRetrievalAdapter::try_new(
                local,
                ContextRetrievalExecutorStatus::Timeout {
                    evidence_ref: "retrieval-executor:error:timeout".to_owned(),
                }
            )
            .expect_err("localhost rejected"),
            ContextRetrievalExecutorAdapterConfigError::LocalEndpointDenied
        );
    }

    #[test]
    fn rejects_raw_query_or_document_refs_before_envelope() {
        let mut adapter = valid_adapter(ContextRetrievalExecutorStatus::Accepted {
            executor_request_ref: "retrieval-executor://requests/req-raw".to_owned(),
            retrieval_ref: "retrieval-executor://retrievals/run-raw".to_owned(),
            evidence_ref: "retrieval-executor:evidence:accepted".to_owned(),
        });
        let mut request = valid_request();
        request.domain_request.request.query_ref = "raw query customer message".to_owned();
        request.domain_request.request.candidates[0].resource_ref =
            "document text account policy".to_owned();

        let failure = adapter
            .dispatch(request)
            .expect_err("raw query refs denied before envelope");
        let debug = format!("{failure:?}{:?}", adapter.last_envelope());

        assert_eq!(
            failure.evidence_ref,
            "validation:context-retrieval-executor-query-ref"
        );
        assert!(adapter.last_envelope().is_none());
        assert!(!debug.contains("raw query customer message"));
        assert!(!debug.contains("document text account policy"));
    }

    #[test]
    fn rejects_denied_or_mismatched_usecase_receipts_before_envelope() {
        let mut adapter = valid_adapter(ContextRetrievalExecutorStatus::Accepted {
            executor_request_ref: "retrieval-executor://requests/req-binding".to_owned(),
            retrieval_ref: "retrieval-executor://retrievals/run-binding".to_owned(),
            evidence_ref: "retrieval-executor:evidence:accepted".to_owned(),
        });
        let mut denied = valid_request();
        denied.usecase_receipt.status = ContextRetrievalUsecaseStatus::Denied;
        denied.usecase_receipt.denial_kind = Some(ContextRetrievalUsecaseDenialKind::DomainDenied);

        let failure = adapter
            .dispatch(denied)
            .expect_err("denied usecase receipt rejected");
        assert_eq!(
            failure.reason,
            "context-retrieval-executor:usecase_receipt_not_planned"
        );
        assert!(adapter.last_envelope().is_none());

        let mut mismatched = valid_request();
        mismatched.usecase_receipt.query_ref = "queryref://opaque/other".to_owned();
        let failure = adapter
            .dispatch(mismatched)
            .expect_err("receipt binding mismatch rejected");
        assert_eq!(
            failure.reason,
            "context-retrieval-executor:usecase_receipt_binding_mismatch"
        );
        assert!(adapter.last_envelope().is_none());
    }

    #[test]
    fn maps_executor_outcomes_distinctly() {
        let mut adapter = valid_adapter(ContextRetrievalExecutorStatus::Queued {
            executor_request_ref: "retrieval-executor://requests/req-queued".to_owned(),
            queue_ref: "retrieval-executor://queues/q-1".to_owned(),
            evidence_ref: "retrieval-executor:evidence:queued".to_owned(),
        });
        let queued = adapter.dispatch(valid_request()).expect("queued");
        assert_eq!(
            queued.status,
            ContextRetrievalExecutorDispatchStatus::Queued
        );
        assert_eq!(
            queued.queue_ref,
            Some("retrieval-executor://queues/q-1".to_owned())
        );

        adapter.set_next_status(ContextRetrievalExecutorStatus::Completed {
            executor_request_ref: "retrieval-executor://requests/req-complete".to_owned(),
            retrieval_ref: "retrieval-executor://retrievals/run-complete".to_owned(),
            context_bundle_ref: "retrieval-executor://bundles/bundle-1".to_owned(),
            evidence_ref: "retrieval-executor:evidence:completed".to_owned(),
        });
        let completed = adapter.dispatch(valid_request()).expect("completed");
        assert_eq!(
            completed.status,
            ContextRetrievalExecutorDispatchStatus::Completed
        );
        assert_eq!(
            completed.context_bundle_ref,
            Some("retrieval-executor://bundles/bundle-1".to_owned())
        );

        adapter.set_next_status(ContextRetrievalExecutorStatus::RateLimited {
            evidence_ref: "retrieval-executor:error:rate-limit".to_owned(),
        });
        let failure = adapter
            .dispatch(valid_request())
            .expect_err("rate limited maps to failure");
        assert_eq!(failure.reason, "context-retrieval-executor:rate_limited");
    }

    #[test]
    fn envelope_and_receipts_never_contain_raw_query_document_or_secret_bytes() {
        let mut adapter = valid_adapter(ContextRetrievalExecutorStatus::Completed {
            executor_request_ref: "retrieval-executor://requests/req-safe".to_owned(),
            retrieval_ref: "retrieval-executor://retrievals/run-safe".to_owned(),
            context_bundle_ref: "retrieval-executor://bundles/bundle-safe".to_owned(),
            evidence_ref: "retrieval-executor:evidence:completed".to_owned(),
        });

        let receipt = adapter.dispatch(valid_request()).expect("completed");
        let debug = format!("{:?}{receipt:?}", adapter.last_envelope());

        assert!(!debug.contains("raw query"));
        assert!(!debug.contains("document text"));
        assert!(!debug.contains("customer message"));
        assert!(!debug.contains("sk-"));
    }

    #[test]
    fn rejects_invalid_executor_status_metadata() {
        let mut adapter = valid_adapter(ContextRetrievalExecutorStatus::Completed {
            executor_request_ref: "retrieval-executor://requests/req-invalid".to_owned(),
            retrieval_ref: "raw query retrieval result".to_owned(),
            context_bundle_ref: "retrieval-executor://bundles/bundle-invalid".to_owned(),
            evidence_ref: "retrieval-executor:evidence:completed".to_owned(),
        });

        let failure = adapter
            .dispatch(valid_request())
            .expect_err("unsafe executor refs rejected");

        assert_eq!(
            failure.reason,
            "context-retrieval-executor:invalid_executor_status"
        );
    }

    fn valid_adapter(
        next_status: ContextRetrievalExecutorStatus,
    ) -> IntelligenceContextAwareRetrievalAdapter {
        IntelligenceContextAwareRetrievalAdapter::try_new(
            ContextRetrievalExecutorAdapterConfig::new(
                "https://retrieval-executor.oyatie.internal/",
                "secretref://ten_a/retrieval-executor/byok",
                "audit://tap/intelligence/context-aware-retrieval",
                "audience://intelligence/context-retrieval-executor",
            ),
            next_status,
        )
        .expect("valid adapter config")
    }

    fn valid_request() -> ContextRetrievalExecutorDispatchRequest {
        let domain_request = sample_domain_request();
        ContextRetrievalExecutorDispatchRequest {
            idempotency_key: "idem-ctx-executor-1".to_owned(),
            usecase_receipt: ContextRetrievalUsecaseReceipt {
                idempotency_key: "idem-ctx-executor-1".to_owned(),
                tenant_id: domain_request.request.tenant_id.clone(),
                principal_id: domain_request.principal_id.clone(),
                query_surface: domain_request.query_surface.clone(),
                query_ref: domain_request.request.query_ref.clone(),
                status: ContextRetrievalUsecaseStatus::Planned,
                denial_kind: None,
                denial_reasons: Vec::new(),
                plan_resource_refs: vec![
                    "kgref://subgraph/adapter-1".to_owned(),
                    "entityref://org/adapter-2".to_owned(),
                ],
                evidence_refs: vec!["retrieval-usecase:evidence:planned".to_owned()],
            },
            domain_request,
        }
    }

    fn sample_domain_request() -> DomainContextRetrievalRequest {
        DomainContextRetrievalRequest {
            principal_id: "principal-ctx-adapter-1".to_owned(),
            query_surface: "intelligence.context-aware-retrieval.pre".to_owned(),
            request: sample_retrieval_request(),
            policy_decision: sample_policy(),
        }
    }

    fn sample_policy() -> ContextRetrievalPolicyDecision {
        ContextRetrievalPolicyDecision {
            decision_id: "policy-ctx-adapter-1".to_owned(),
            tenant_id: "ten_a".to_owned(),
            principal_id: "principal-ctx-adapter-1".to_owned(),
            allowed_surfaces: vec!["intelligence.context-aware-retrieval.pre".to_owned()],
            allowed_source_kinds: vec![
                ContextSourceKind::OntologyEntity,
                ContextSourceKind::KnowledgeGraphSubgraph,
            ],
            max_context_items: 2,
            freshness_floor_epoch_seconds: 100,
            evidence_ref: "cedar:ctx-adapter:allow".to_owned(),
            retrieval_index_snapshot_ref: "retrieval-index:snapshot:adapter:1".to_owned(),
        }
    }

    fn sample_retrieval_request() -> ContextRetrievalRequest {
        ContextRetrievalRequest {
            tenant_id: "ten_a".to_owned(),
            query_ref: "queryref://opaque/adapter-1".to_owned(),
            request_evidence_ref: "req:ctx-adapter:1".to_owned(),
            trace_context_ref: "trace:ctx-adapter:1".to_owned(),
            policy_decision_ref: "cedar:ctx-adapter:allow".to_owned(),
            audience: ContextAudience::TenantOperator,
            allowed_source_kinds: vec![
                ContextSourceKind::OntologyEntity,
                ContextSourceKind::KnowledgeGraphSubgraph,
            ],
            max_context_items: 2,
            freshness_floor_epoch_seconds: 100,
            candidates: vec![
                ContextCandidate {
                    tenant_id: "ten_a".to_owned(),
                    source_kind: ContextSourceKind::OntologyEntity,
                    resource_ref: "entityref://org/adapter-2".to_owned(),
                    evidence_ref: "ctx:entity:adapter:2".to_owned(),
                    data_class: ContextDataClass::InternalOnly,
                    observed_at_epoch_seconds: 125,
                    relevance_millis: 920,
                },
                ContextCandidate {
                    tenant_id: "ten_a".to_owned(),
                    source_kind: ContextSourceKind::KnowledgeGraphSubgraph,
                    resource_ref: "kgref://subgraph/adapter-1".to_owned(),
                    evidence_ref: "ctx:kg:adapter:1".to_owned(),
                    data_class: ContextDataClass::InternalOnly,
                    observed_at_epoch_seconds: 130,
                    relevance_millis: 930,
                },
            ],
        }
    }
}
