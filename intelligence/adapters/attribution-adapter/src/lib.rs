//! Intelligence attribution adapter foundation.
//!
//! This crate defines a deterministic, metadata-only adapter seam for future
//! citation renderer integration. It builds renderer request envelopes from the
//! attribution domain/usecase metadata already authorized by the preview
//! foundation and maps renderer outcome metadata into stable receipts. It
//! performs no network I/O, citation text rendering, retrieval execution,
//! filesystem access, credential resolution, durable idempotency, or durable
//! audit-chain emission.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_attribution_usecase::{
    AttributionAudience, AttributionClaim, AttributionDataClass, AttributionDenialKind,
    AttributionDomainDenialKind, AttributionPolicyDecision, AttributionRequest, AttributionSource,
    AttributionSourceKind, AttributionUsecaseDenialKind, AttributionUsecaseReceipt,
    AttributionUsecaseStatus, DomainAttributionRequest,
};

const ATTRIBUTION_RENDERER_PATH: &str = "/v1/attribution/citation-renders";
const ADAPTER_REFERENCE_REF: &str = "spec://oyatie/intelligence/attribution-adapter-foundation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributionRendererHttpMethod {
    Post,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributionRendererTransportMode {
    EnvelopeOnly,
    HostedRenderer,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionRendererAdapterConfig {
    pub endpoint: String,              // data_class: INTERNAL_ONLY
    pub credential_handle_ref: String, // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,         // data_class: INTERNAL_ONLY
    pub renderer_audience_ref: String, // data_class: INTERNAL_ONLY
    pub transport_mode: AttributionRendererTransportMode, // data_class: INTERNAL_ONLY
}

impl AttributionRendererAdapterConfig {
    pub fn new(
        endpoint: impl Into<String>,
        credential_handle_ref: impl Into<String>,
        audit_tap_ref: impl Into<String>,
        renderer_audience_ref: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            credential_handle_ref: credential_handle_ref.into(),
            audit_tap_ref: audit_tap_ref.into(),
            renderer_audience_ref: renderer_audience_ref.into(),
            transport_mode: AttributionRendererTransportMode::EnvelopeOnly,
        }
    }

    pub fn with_transport_mode(mut self, transport_mode: AttributionRendererTransportMode) -> Self {
        self.transport_mode = transport_mode;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributionRendererAdapterConfigError {
    EmptyEndpoint,
    NonHttpsEndpoint,
    LocalEndpointDenied,
    EmptyCredentialHandleRef,
    RawCredentialMaterialRejected,
    NonOpaqueCredentialHandleRef,
    EmptyAuditTapRef,
    InvalidAuditTapRef,
    EmptyRendererAudienceRef,
    InvalidRendererAudienceRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionRendererDispatchRequest {
    pub idempotency_key: String,                  // data_class: INTERNAL_ONLY
    pub domain_request: DomainAttributionRequest, // data_class: INTERNAL_ONLY
    pub usecase_receipt: AttributionUsecaseReceipt, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionRendererRequestEnvelope {
    pub method: AttributionRendererHttpMethod, // data_class: PUBLIC
    pub endpoint: String,                      // data_class: INTERNAL_ONLY
    pub path: String,                          // data_class: PUBLIC
    pub transport_mode: AttributionRendererTransportMode, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub principal_id: String,                  // data_class: INTERNAL_ONLY
    pub attribution_surface: String,           // data_class: INTERNAL_ONLY
    pub idempotency_key: String,               // data_class: INTERNAL_ONLY
    pub output_ref: String,                    // data_class: INTERNAL_ONLY
    pub audience: AttributionAudience,         // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,          // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,             // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,           // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: String,           // data_class: INTERNAL_ONLY
    pub attribution_registry_snapshot_ref: String, // data_class: INTERNAL_ONLY
    pub credential_handle_ref: String,         // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,                 // data_class: INTERNAL_ONLY
    pub renderer_audience_ref: String,         // data_class: INTERNAL_ONLY
    pub citation_count: u32,                   // data_class: INTERNAL_ONLY
    pub citation_resource_refs: Vec<String>,   // data_class: INTERNAL_ONLY
    pub source_resource_refs: Vec<String>,     // data_class: INTERNAL_ONLY
    pub source_title_refs: Vec<String>,        // data_class: INTERNAL_ONLY
    pub source_evidence_refs: Vec<String>,     // data_class: INTERNAL_ONLY
    pub claim_ids: Vec<String>,                // data_class: INTERNAL_ONLY
    pub claim_answer_segment_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub claim_source_ids: Vec<String>,         // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
    pub adapter_reference_refs: Vec<String>,   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttributionRendererStatus {
    Accepted {
        renderer_request_ref: String,
        render_ref: String,
        evidence_ref: String,
    },
    Queued {
        renderer_request_ref: String,
        queue_ref: String,
        evidence_ref: String,
    },
    Completed {
        renderer_request_ref: String,
        render_ref: String,
        citation_bundle_ref: String,
        evidence_ref: String,
    },
    Denied {
        evidence_ref: String,
    },
    RateLimited {
        evidence_ref: String,
    },
    RendererError {
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
pub enum AttributionRendererDispatchStatus {
    Accepted,
    Queued,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionRendererDispatchReceipt {
    pub status: AttributionRendererDispatchStatus, // data_class: PUBLIC
    pub renderer_request_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub render_ref: Option<String>,                // data_class: INTERNAL_ONLY
    pub queue_ref: Option<String>,                 // data_class: INTERNAL_ONLY
    pub citation_bundle_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributionRendererDispatchFailure {
    pub reason: String,       // data_class: INTERNAL_ONLY
    pub evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Debug)]
pub struct IntelligenceAttributionAdapter {
    config: AttributionRendererAdapterConfig,
    next_status: AttributionRendererStatus,
    last_envelope: Option<AttributionRendererRequestEnvelope>,
}

impl IntelligenceAttributionAdapter {
    pub fn try_new(
        config: AttributionRendererAdapterConfig,
        next_status: AttributionRendererStatus,
    ) -> Result<Self, AttributionRendererAdapterConfigError> {
        validate_config(&config)?;
        Ok(Self {
            config,
            next_status,
            last_envelope: None,
        })
    }

    pub fn last_envelope(&self) -> Option<&AttributionRendererRequestEnvelope> {
        self.last_envelope.as_ref()
    }

    pub fn set_next_status(&mut self, next_status: AttributionRendererStatus) {
        self.next_status = next_status;
    }

    pub fn dispatch(
        &mut self,
        request: AttributionRendererDispatchRequest,
    ) -> Result<AttributionRendererDispatchReceipt, AttributionRendererDispatchFailure> {
        validate_dispatch_request(&request)?;
        let envelope = self.build_envelope(&request);
        self.last_envelope = Some(envelope);
        receipt_from_status(&self.next_status)
    }

    fn build_envelope(
        &self,
        request: &AttributionRendererDispatchRequest,
    ) -> AttributionRendererRequestEnvelope {
        let kernel = &request.domain_request.request;
        AttributionRendererRequestEnvelope {
            method: AttributionRendererHttpMethod::Post,
            endpoint: normalized_endpoint(&self.config.endpoint),
            path: ATTRIBUTION_RENDERER_PATH.to_owned(),
            transport_mode: self.config.transport_mode,
            tenant_id: request.domain_request.tenant_id.clone(),
            principal_id: request.domain_request.principal_id.clone(),
            attribution_surface: request.domain_request.attribution_surface.clone(),
            idempotency_key: request.idempotency_key.clone(),
            output_ref: kernel.output_ref.clone(),
            audience: kernel.audience,
            request_evidence_ref: request.domain_request.request_evidence_ref.clone(),
            trace_context_ref: request.domain_request.trace_context_ref.clone(),
            policy_decision_ref: request.domain_request.policy_decision_ref.clone(),
            policy_evidence_ref: kernel.policy_evidence_ref.clone(),
            attribution_registry_snapshot_ref: request
                .domain_request
                .policy_decision
                .attribution_registry_snapshot_ref
                .clone(),
            credential_handle_ref: self.config.credential_handle_ref.clone(),
            audit_tap_ref: self.config.audit_tap_ref.clone(),
            renderer_audience_ref: self.config.renderer_audience_ref.clone(),
            citation_count: request.usecase_receipt.citation_count as u32,
            citation_resource_refs: sorted_unique(
                request.usecase_receipt.citation_resource_refs.clone(),
            ),
            source_resource_refs: sorted_unique(
                kernel
                    .sources
                    .iter()
                    .map(|source| source.resource_ref.clone())
                    .collect(),
            ),
            source_title_refs: sorted_unique(
                kernel
                    .sources
                    .iter()
                    .map(|source| source.title_ref.clone())
                    .collect(),
            ),
            source_evidence_refs: sorted_unique(
                kernel
                    .sources
                    .iter()
                    .map(|source| source.evidence_ref.clone())
                    .collect(),
            ),
            claim_ids: sorted_unique(
                kernel
                    .claims
                    .iter()
                    .map(|claim| claim.claim_id.clone())
                    .collect(),
            ),
            claim_answer_segment_refs: sorted_unique(
                kernel
                    .claims
                    .iter()
                    .map(|claim| claim.answer_segment_ref.clone())
                    .collect(),
            ),
            claim_source_ids: claim_source_ids(&kernel.claims),
            evidence_refs: dispatch_evidence_refs(request),
            adapter_reference_refs: vec![ADAPTER_REFERENCE_REF.to_owned()],
        }
    }
}

fn validate_config(
    config: &AttributionRendererAdapterConfig,
) -> Result<(), AttributionRendererAdapterConfigError> {
    let endpoint = config.endpoint.trim();
    if endpoint.is_empty() {
        return Err(AttributionRendererAdapterConfigError::EmptyEndpoint);
    }
    if !endpoint.starts_with("https://") || contains_whitespace(endpoint) {
        return Err(AttributionRendererAdapterConfigError::NonHttpsEndpoint);
    }
    if is_local_endpoint(endpoint) {
        return Err(AttributionRendererAdapterConfigError::LocalEndpointDenied);
    }
    validate_credential_handle_ref(&config.credential_handle_ref)?;
    validate_audit_tap_ref(&config.audit_tap_ref)?;
    validate_renderer_audience_ref(&config.renderer_audience_ref)?;
    Ok(())
}

fn validate_credential_handle_ref(
    value: &str,
) -> Result<(), AttributionRendererAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AttributionRendererAdapterConfigError::EmptyCredentialHandleRef);
    }
    if !is_safe_double_colon_ref(trimmed) {
        return Err(if contains_raw_secret_material(trimmed) {
            AttributionRendererAdapterConfigError::RawCredentialMaterialRejected
        } else {
            AttributionRendererAdapterConfigError::NonOpaqueCredentialHandleRef
        });
    }
    Ok(())
}

fn validate_audit_tap_ref(value: &str) -> Result<(), AttributionRendererAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AttributionRendererAdapterConfigError::EmptyAuditTapRef);
    }
    if !is_safe_double_colon_ref(trimmed) {
        return Err(AttributionRendererAdapterConfigError::InvalidAuditTapRef);
    }
    Ok(())
}

fn validate_renderer_audience_ref(
    value: &str,
) -> Result<(), AttributionRendererAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AttributionRendererAdapterConfigError::EmptyRendererAudienceRef);
    }
    if !is_safe_double_colon_ref(trimmed) {
        return Err(AttributionRendererAdapterConfigError::InvalidRendererAudienceRef);
    }
    Ok(())
}

fn validate_dispatch_request(
    request: &AttributionRendererDispatchRequest,
) -> Result<(), AttributionRendererDispatchFailure> {
    require_metadata(
        "citation-renderer:idempotency_key_required",
        "validation:citation-renderer-idempotency-key",
        &request.idempotency_key,
    )?;

    let domain = &request.domain_request;
    let kernel = &domain.request;
    require_opaque(
        "citation-renderer:tenant_required",
        "validation:citation-renderer-tenant",
        &domain.tenant_id,
    )?;
    require_opaque(
        "citation-renderer:principal_required",
        "validation:citation-renderer-principal",
        &domain.principal_id,
    )?;
    require_opaque(
        "citation-renderer:surface_required",
        "validation:citation-renderer-surface",
        &domain.attribution_surface,
    )?;
    require_opaque(
        "citation-renderer:request_evidence_required",
        "validation:citation-renderer-request-evidence",
        &domain.request_evidence_ref,
    )?;
    require_opaque(
        "citation-renderer:trace_context_required",
        "validation:citation-renderer-trace-context",
        &domain.trace_context_ref,
    )?;
    require_opaque(
        "citation-renderer:policy_decision_required",
        "validation:citation-renderer-policy-decision",
        &domain.policy_decision_ref,
    )?;
    require_opaque(
        "citation-renderer:attribution_registry_snapshot_required",
        "validation:citation-renderer-registry-snapshot",
        &domain.policy_decision.attribution_registry_snapshot_ref,
    )?;
    require_opaque(
        "citation-renderer:output_ref_required",
        "validation:citation-renderer-output-ref",
        &kernel.output_ref,
    )?;
    require_opaque(
        "citation-renderer:kernel_policy_evidence_required",
        "validation:citation-renderer-kernel-policy-evidence",
        &kernel.policy_evidence_ref,
    )?;
    require_opaque(
        "citation-renderer:kernel_trace_context_required",
        "validation:citation-renderer-kernel-trace-context",
        &kernel.trace_context_ref,
    )?;
    if kernel.sources.is_empty() || kernel.claims.is_empty() {
        return Err(dispatch_failure(
            "citation-renderer:citation_metadata_required",
            "validation:citation-renderer-citation-metadata",
        ));
    }
    for source in &kernel.sources {
        require_metadata(
            "citation-renderer:source_id_required",
            "validation:citation-renderer-source-id",
            &source.source_id,
        )?;
        require_opaque(
            "citation-renderer:source_resource_ref_required",
            "validation:citation-renderer-source-resource-ref",
            &source.resource_ref,
        )?;
        require_opaque(
            "citation-renderer:source_title_ref_required",
            "validation:citation-renderer-source-title-ref",
            &source.title_ref,
        )?;
        require_opaque(
            "citation-renderer:source_evidence_ref_required",
            "validation:citation-renderer-source-evidence-ref",
            &source.evidence_ref,
        )?;
    }
    for claim in &kernel.claims {
        require_metadata(
            "citation-renderer:claim_id_required",
            "validation:citation-renderer-claim-id",
            &claim.claim_id,
        )?;
        require_opaque(
            "citation-renderer:claim_segment_ref_required",
            "validation:citation-renderer-claim-segment-ref",
            &claim.answer_segment_ref,
        )?;
        for source_id in &claim.source_ids {
            require_metadata(
                "citation-renderer:claim_source_id_required",
                "validation:citation-renderer-claim-source-id",
                source_id,
            )?;
        }
    }

    validate_receipt_binding(request)
}

fn validate_receipt_binding(
    request: &AttributionRendererDispatchRequest,
) -> Result<(), AttributionRendererDispatchFailure> {
    let receipt = &request.usecase_receipt;
    let domain = &request.domain_request;
    let kernel = &domain.request;
    if receipt.idempotency_key != request.idempotency_key
        || receipt.tenant_id != domain.tenant_id
        || receipt.principal_id != domain.principal_id
        || receipt.attribution_surface != domain.attribution_surface
        || receipt.output_ref != kernel.output_ref
    {
        return Err(dispatch_failure(
            "citation-renderer:usecase_receipt_binding_mismatch",
            "validation:citation-renderer-usecase-receipt-binding",
        ));
    }
    if receipt.status != AttributionUsecaseStatus::Rendered || receipt.denial_kind.is_some() {
        return Err(dispatch_failure(
            "citation-renderer:usecase_receipt_not_rendered",
            "validation:citation-renderer-usecase-receipt-status",
        ));
    }
    if receipt.citation_count == 0 || receipt.citation_count > kernel.max_citations {
        return Err(dispatch_failure(
            "citation-renderer:usecase_receipt_invalid_citation_count",
            "validation:citation-renderer-citation-count",
        ));
    }
    let source_resource_refs: Vec<&str> = kernel
        .sources
        .iter()
        .map(|source| source.resource_ref.as_str())
        .collect();
    for resource_ref in &receipt.citation_resource_refs {
        require_opaque(
            "citation-renderer:receipt_citation_resource_ref_required",
            "validation:citation-renderer-receipt-resource-ref",
            resource_ref,
        )?;
        if !source_resource_refs.contains(&resource_ref.as_str()) {
            return Err(dispatch_failure(
                "citation-renderer:receipt_citation_resource_outside_request",
                "validation:citation-renderer-receipt-resource-binding",
            ));
        }
    }
    for evidence_ref in &receipt.evidence_refs {
        require_opaque(
            "citation-renderer:receipt_evidence_ref_required",
            "validation:citation-renderer-receipt-evidence-ref",
            evidence_ref,
        )?;
    }
    Ok(())
}

fn receipt_from_status(
    status: &AttributionRendererStatus,
) -> Result<AttributionRendererDispatchReceipt, AttributionRendererDispatchFailure> {
    match status {
        AttributionRendererStatus::Accepted {
            renderer_request_ref,
            render_ref,
            evidence_ref,
        } => ok_receipt(
            AttributionRendererDispatchStatus::Accepted,
            Some(renderer_request_ref),
            Some(render_ref),
            None,
            None,
            evidence_ref,
        ),
        AttributionRendererStatus::Queued {
            renderer_request_ref,
            queue_ref,
            evidence_ref,
        } => ok_receipt(
            AttributionRendererDispatchStatus::Queued,
            Some(renderer_request_ref),
            None,
            Some(queue_ref),
            None,
            evidence_ref,
        ),
        AttributionRendererStatus::Completed {
            renderer_request_ref,
            render_ref,
            citation_bundle_ref,
            evidence_ref,
        } => ok_receipt(
            AttributionRendererDispatchStatus::Completed,
            Some(renderer_request_ref),
            Some(render_ref),
            None,
            Some(citation_bundle_ref),
            evidence_ref,
        ),
        AttributionRendererStatus::Denied { evidence_ref } => {
            Err(dispatch_failure("citation-renderer:denied", evidence_ref))
        }
        AttributionRendererStatus::RateLimited { evidence_ref } => Err(dispatch_failure(
            "citation-renderer:rate_limited",
            evidence_ref,
        )),
        AttributionRendererStatus::RendererError { evidence_ref } => Err(dispatch_failure(
            "citation-renderer:renderer_error",
            evidence_ref,
        )),
        AttributionRendererStatus::AuthError { evidence_ref } => Err(dispatch_failure(
            "citation-renderer:auth_error",
            evidence_ref,
        )),
        AttributionRendererStatus::InvalidRequest { evidence_ref } => Err(dispatch_failure(
            "citation-renderer:invalid_request",
            evidence_ref,
        )),
        AttributionRendererStatus::Timeout { evidence_ref } => {
            Err(dispatch_failure("citation-renderer:timeout", evidence_ref))
        }
    }
}

fn ok_receipt(
    status: AttributionRendererDispatchStatus,
    renderer_request_ref: Option<&String>,
    render_ref: Option<&String>,
    queue_ref: Option<&String>,
    citation_bundle_ref: Option<&String>,
    evidence_ref: &str,
) -> Result<AttributionRendererDispatchReceipt, AttributionRendererDispatchFailure> {
    let refs = [
        renderer_request_ref,
        render_ref,
        queue_ref,
        citation_bundle_ref,
    ];
    if refs
        .into_iter()
        .flatten()
        .any(|value| !is_safe_opaque_ref(value))
        || !is_safe_opaque_ref(evidence_ref)
    {
        return Err(dispatch_failure(
            "citation-renderer:invalid_renderer_status",
            "validation:citation-renderer-status-metadata",
        ));
    }
    Ok(AttributionRendererDispatchReceipt {
        status,
        renderer_request_ref: renderer_request_ref.cloned(),
        render_ref: render_ref.cloned(),
        queue_ref: queue_ref.cloned(),
        citation_bundle_ref: citation_bundle_ref.cloned(),
        evidence_ref: evidence_ref.to_owned(),
    })
}

fn dispatch_evidence_refs(request: &AttributionRendererDispatchRequest) -> Vec<String> {
    let kernel = &request.domain_request.request;
    let mut refs = vec![
        request.domain_request.request_evidence_ref.clone(),
        request.domain_request.trace_context_ref.clone(),
        request.domain_request.policy_decision_ref.clone(),
        request
            .domain_request
            .policy_decision
            .attribution_registry_snapshot_ref
            .clone(),
        kernel.policy_evidence_ref.clone(),
        kernel.trace_context_ref.clone(),
    ];
    refs.extend(request.usecase_receipt.evidence_refs.clone());
    refs.extend(
        kernel
            .sources
            .iter()
            .map(|source| source.evidence_ref.clone()),
    );
    sorted_unique(refs)
}

fn claim_source_ids(claims: &[AttributionClaim]) -> Vec<String> {
    let mut ids = Vec::new();
    for claim in claims {
        ids.extend(claim.source_ids.clone());
    }
    sorted_unique(ids)
}

fn require_metadata(
    reason: &str,
    evidence_ref: &str,
    value: &str,
) -> Result<(), AttributionRendererDispatchFailure> {
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
) -> Result<(), AttributionRendererDispatchFailure> {
    if is_safe_opaque_ref(value) {
        Ok(())
    } else {
        Err(dispatch_failure(reason, evidence_ref))
    }
}

fn dispatch_failure(reason: &str, evidence_ref: &str) -> AttributionRendererDispatchFailure {
    AttributionRendererDispatchFailure {
        reason: reason.to_owned(),
        evidence_ref: if is_safe_metadata_ref(evidence_ref) {
            evidence_ref.to_owned()
        } else {
            "citation-renderer:error:unsafe-evidence-ref".to_owned()
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
        || lower.contains("raw output")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_metadata_only_citation_renderer_envelope() {
        let mut adapter = valid_adapter(AttributionRendererStatus::Accepted {
            renderer_request_ref: "citation-renderer://requests/req-1".to_owned(),
            render_ref: "citation-renderer://renders/render-1".to_owned(),
            evidence_ref: "citation-renderer:evidence:accepted".to_owned(),
        });

        let receipt = adapter
            .dispatch(valid_request())
            .expect("renderer accepts request");
        let envelope = adapter.last_envelope().expect("envelope recorded");

        assert_eq!(envelope.method, AttributionRendererHttpMethod::Post);
        assert_eq!(envelope.path, "/v1/attribution/citation-renders");
        assert_eq!(envelope.tenant_id, "tenant:alpha");
        assert_eq!(envelope.output_ref, "answer://responses/resp-adapter-1");
        assert_eq!(envelope.citation_count, 2);
        assert_eq!(
            envelope.transport_mode,
            AttributionRendererTransportMode::EnvelopeOnly
        );
        assert_eq!(
            envelope.credential_handle_ref,
            "secretref://ten_a/citation-renderer/byok"
        );
        assert!(
            envelope
                .evidence_refs
                .contains(&"attribution-registry:snapshot:adapter:1".to_owned())
        );
        assert_eq!(receipt.status, AttributionRendererDispatchStatus::Accepted);
        assert_eq!(
            receipt.render_ref,
            Some("citation-renderer://renders/render-1".to_owned())
        );
    }

    #[test]
    fn rejects_raw_secret_like_credential_handles() {
        let config = AttributionRendererAdapterConfig::new(
            "https://citation-renderer.oyatie.internal",
            "sk-test-raw-secret",
            "audit://tap/intelligence/attribution",
            "audience://intelligence/citation-renderer",
        );

        let error = IntelligenceAttributionAdapter::try_new(
            config,
            AttributionRendererStatus::Timeout {
                evidence_ref: "citation-renderer:error:timeout".to_owned(),
            },
        )
        .expect_err("raw secret handle rejected");

        assert_eq!(
            error,
            AttributionRendererAdapterConfigError::RawCredentialMaterialRejected
        );
    }

    #[test]
    fn rejects_non_https_and_localhost_endpoints() {
        let non_https = AttributionRendererAdapterConfig::new(
            "http://citation-renderer.oyatie.internal",
            "secretref://ten_a/citation-renderer/byok",
            "audit://tap/intelligence/attribution",
            "audience://intelligence/citation-renderer",
        );
        let local = AttributionRendererAdapterConfig::new(
            "https://localhost:9444",
            "secretref://ten_a/citation-renderer/byok",
            "audit://tap/intelligence/attribution",
            "audience://intelligence/citation-renderer",
        );

        assert_eq!(
            IntelligenceAttributionAdapter::try_new(
                non_https,
                AttributionRendererStatus::Timeout {
                    evidence_ref: "citation-renderer:error:timeout".to_owned(),
                }
            )
            .expect_err("non-https rejected"),
            AttributionRendererAdapterConfigError::NonHttpsEndpoint
        );
        assert_eq!(
            IntelligenceAttributionAdapter::try_new(
                local,
                AttributionRendererStatus::Timeout {
                    evidence_ref: "citation-renderer:error:timeout".to_owned(),
                }
            )
            .expect_err("localhost rejected"),
            AttributionRendererAdapterConfigError::LocalEndpointDenied
        );
    }

    #[test]
    fn rejects_raw_prompt_output_or_document_shaped_refs_before_envelope() {
        let mut adapter = valid_adapter(AttributionRendererStatus::Accepted {
            renderer_request_ref: "citation-renderer://requests/req-raw".to_owned(),
            render_ref: "citation-renderer://renders/render-raw".to_owned(),
            evidence_ref: "citation-renderer:evidence:accepted".to_owned(),
        });
        let mut request = valid_request();
        request.domain_request.request.output_ref = "raw output model answer".to_owned();
        request.domain_request.request.sources[0].resource_ref =
            "document text refund policy".to_owned();

        let failure = adapter
            .dispatch(request)
            .expect_err("raw content refs denied before envelope");
        let debug = format!("{failure:?}{:?}", adapter.last_envelope());

        assert_eq!(
            failure.evidence_ref,
            "validation:citation-renderer-output-ref"
        );
        assert!(adapter.last_envelope().is_none());
        assert!(!debug.contains("raw output model answer"));
        assert!(!debug.contains("document text refund policy"));
    }

    #[test]
    fn rejects_denied_or_mismatched_usecase_receipts_before_envelope() {
        let mut adapter = valid_adapter(AttributionRendererStatus::Accepted {
            renderer_request_ref: "citation-renderer://requests/req-binding".to_owned(),
            render_ref: "citation-renderer://renders/render-binding".to_owned(),
            evidence_ref: "citation-renderer:evidence:accepted".to_owned(),
        });
        let mut denied = valid_request();
        denied.usecase_receipt.status = AttributionUsecaseStatus::Denied;
        denied.usecase_receipt.denial_kind = Some(AttributionUsecaseDenialKind::DomainDenied);

        let failure = adapter
            .dispatch(denied)
            .expect_err("denied usecase receipt rejected");
        assert_eq!(
            failure.reason,
            "citation-renderer:usecase_receipt_not_rendered"
        );
        assert!(adapter.last_envelope().is_none());

        let mut mismatched = valid_request();
        mismatched.usecase_receipt.output_ref = "answer://responses/other".to_owned();
        let failure = adapter
            .dispatch(mismatched)
            .expect_err("receipt binding mismatch rejected");
        assert_eq!(
            failure.reason,
            "citation-renderer:usecase_receipt_binding_mismatch"
        );
        assert!(adapter.last_envelope().is_none());
    }

    #[test]
    fn maps_renderer_outcomes_distinctly() {
        let mut adapter = valid_adapter(AttributionRendererStatus::Queued {
            renderer_request_ref: "citation-renderer://requests/req-queued".to_owned(),
            queue_ref: "citation-renderer://queues/q-1".to_owned(),
            evidence_ref: "citation-renderer:evidence:queued".to_owned(),
        });
        let queued = adapter.dispatch(valid_request()).expect("queued");
        assert_eq!(queued.status, AttributionRendererDispatchStatus::Queued);
        assert_eq!(
            queued.queue_ref,
            Some("citation-renderer://queues/q-1".to_owned())
        );

        adapter.set_next_status(AttributionRendererStatus::Completed {
            renderer_request_ref: "citation-renderer://requests/req-complete".to_owned(),
            render_ref: "citation-renderer://renders/render-complete".to_owned(),
            citation_bundle_ref: "citation-renderer://bundles/bundle-1".to_owned(),
            evidence_ref: "citation-renderer:evidence:completed".to_owned(),
        });
        let completed = adapter.dispatch(valid_request()).expect("completed");
        assert_eq!(
            completed.status,
            AttributionRendererDispatchStatus::Completed
        );
        assert_eq!(
            completed.citation_bundle_ref,
            Some("citation-renderer://bundles/bundle-1".to_owned())
        );

        adapter.set_next_status(AttributionRendererStatus::RateLimited {
            evidence_ref: "citation-renderer:error:rate-limit".to_owned(),
        });
        let failure = adapter
            .dispatch(valid_request())
            .expect_err("rate limited maps to failure");
        assert_eq!(failure.reason, "citation-renderer:rate_limited");
    }

    #[test]
    fn envelope_and_receipts_never_contain_raw_prompt_output_document_or_secret_bytes() {
        let mut adapter = valid_adapter(AttributionRendererStatus::Completed {
            renderer_request_ref: "citation-renderer://requests/req-safe".to_owned(),
            render_ref: "citation-renderer://renders/render-safe".to_owned(),
            citation_bundle_ref: "citation-renderer://bundles/bundle-safe".to_owned(),
            evidence_ref: "citation-renderer:evidence:completed".to_owned(),
        });

        let receipt = adapter.dispatch(valid_request()).expect("completed");
        let debug = format!("{:?}{receipt:?}", adapter.last_envelope());

        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("raw output"));
        assert!(!debug.contains("document text"));
        assert!(!debug.contains("sk-"));
    }

    #[test]
    fn rejects_invalid_renderer_status_metadata() {
        let mut adapter = valid_adapter(AttributionRendererStatus::Completed {
            renderer_request_ref: "citation-renderer://requests/req-invalid".to_owned(),
            render_ref: "raw output rendered citation".to_owned(),
            citation_bundle_ref: "citation-renderer://bundles/bundle-invalid".to_owned(),
            evidence_ref: "citation-renderer:evidence:completed".to_owned(),
        });

        let failure = adapter
            .dispatch(valid_request())
            .expect_err("unsafe renderer refs rejected");

        assert_eq!(failure.reason, "citation-renderer:invalid_renderer_status");
    }

    fn valid_adapter(next_status: AttributionRendererStatus) -> IntelligenceAttributionAdapter {
        IntelligenceAttributionAdapter::try_new(
            AttributionRendererAdapterConfig::new(
                "https://citation-renderer.oyatie.internal/",
                "secretref://ten_a/citation-renderer/byok",
                "audit://tap/intelligence/attribution",
                "audience://intelligence/citation-renderer",
            ),
            next_status,
        )
        .expect("valid adapter config")
    }

    fn valid_request() -> AttributionRendererDispatchRequest {
        let domain_request = sample_domain_request();
        AttributionRendererDispatchRequest {
            idempotency_key: "idem:attribution-renderer:1".to_owned(),
            usecase_receipt: AttributionUsecaseReceipt {
                idempotency_key: "idem:attribution-renderer:1".to_owned(),
                tenant_id: domain_request.tenant_id.clone(),
                principal_id: domain_request.principal_id.clone(),
                attribution_surface: domain_request.attribution_surface.clone(),
                output_ref: domain_request.request.output_ref.clone(),
                status: AttributionUsecaseStatus::Rendered,
                denial_kind: None,
                domain_denial_kind: None,
                kernel_denial_kind: None,
                citation_count: 2,
                citation_resource_refs: vec![
                    "kg://entity/accounting-policy".to_owned(),
                    "doc://help-center/refund-policy".to_owned(),
                ],
                evidence_refs: vec!["attribution-usecase:evidence:rendered".to_owned()],
            },
            domain_request,
        }
    }

    fn sample_domain_request() -> DomainAttributionRequest {
        DomainAttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-owner".to_owned(),
            attribution_surface: "surface:dispatch-response".to_owned(),
            request_evidence_ref: "request:evidence:attribution-renderer:1".to_owned(),
            trace_context_ref: "trace:attribution-renderer:1".to_owned(),
            policy_decision_ref: "policy:evidence:attribution-renderer:1".to_owned(),
            policy_decision: sample_policy(),
            request: sample_kernel_request(),
        }
    }

    fn sample_policy() -> AttributionPolicyDecision {
        AttributionPolicyDecision {
            decision_id: "attribution-policy-decision:adapter:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:attribution-owner".to_owned(),
            allowed_surfaces: vec!["surface:dispatch-response".to_owned()],
            allowed_audiences: vec![AttributionAudience::External, AttributionAudience::Internal],
            allowed_source_kinds: vec![
                AttributionSourceKind::KnowledgeGraph,
                AttributionSourceKind::PolicyDocument,
                AttributionSourceKind::RetrievalDocument,
            ],
            allowed_data_classes: vec![
                AttributionDataClass::Public,
                AttributionDataClass::Internal,
            ],
            max_citations: 8,
            min_confidence_bps: 7_000,
            evidence_ref: "policy:evidence:attribution-renderer:1".to_owned(),
            attribution_registry_snapshot_ref: "attribution-registry:snapshot:adapter:1".to_owned(),
        }
    }

    fn sample_kernel_request() -> AttributionRequest {
        AttributionRequest {
            tenant_id: "tenant:alpha".to_owned(),
            output_ref: "answer://responses/resp-adapter-1".to_owned(),
            audience: AttributionAudience::External,
            policy_evidence_ref: "policy:evidence:attribution-renderer:1".to_owned(),
            trace_context_ref: "trace:attribution-renderer:1".to_owned(),
            max_citations: 8,
            max_citations_per_claim: 8,
            sources: vec![
                AttributionSource {
                    source_id: "src-kg-policy".to_owned(),
                    resource_ref: "kg://entity/accounting-policy".to_owned(),
                    title_ref: "title://knowledge/accounting-policy".to_owned(),
                    source_kind: AttributionSourceKind::KnowledgeGraph,
                    data_class: AttributionDataClass::Public,
                    evidence_ref: "evidence:kg:accounting-policy".to_owned(),
                    freshness_epoch_seconds: 1_779_523_200,
                },
                AttributionSource {
                    source_id: "src-doc-refund".to_owned(),
                    resource_ref: "doc://help-center/refund-policy".to_owned(),
                    title_ref: "title://help/refund-policy".to_owned(),
                    source_kind: AttributionSourceKind::RetrievalDocument,
                    data_class: AttributionDataClass::Public,
                    evidence_ref: "evidence:doc:refund-policy".to_owned(),
                    freshness_epoch_seconds: 1_779_523_201,
                },
            ],
            claims: vec![
                AttributionClaim {
                    claim_id: "claim-2".to_owned(),
                    answer_segment_ref: "answer-segment://resp-adapter-1/2".to_owned(),
                    source_ids: vec!["src-doc-refund".to_owned()],
                    confidence_bps: 9_000,
                },
                AttributionClaim {
                    claim_id: "claim-1".to_owned(),
                    answer_segment_ref: "answer-segment://resp-adapter-1/1".to_owned(),
                    source_ids: vec!["src-kg-policy".to_owned()],
                    confidence_bps: 9_200,
                },
            ],
        }
    }
}
