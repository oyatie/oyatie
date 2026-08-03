use oya_intelligence_dispatch_usecase::{
    ProviderDispatchFailure, ProviderDispatchPort, ProviderDispatchRequest,
    ProviderDispatchResponse,
};
use intelligence_model_routing_domain::{CredentialMode, ModelProvider};

use crate::modalities::{OpenAiModality, default_openai_modalities, sorted_modalities};
use crate::streaming::OpenAiStreamingMode;

const OPENAI_RESPONSES_PATH: &str = "/v1/responses";
const OPENAI_RESPONSES_DOC: &str =
    "https://developers.openai.com/api/reference/resources/responses/methods/create";
const OPENAI_STREAMING_DOC: &str =
    "https://developers.openai.com/api/docs/guides/streaming-responses";
const OPENAI_AUTH_DOC: &str = "https://developers.openai.com/api/reference/overview#authentication";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAiProviderAdapterConfig {
    pub endpoint: String,                         // data_class: INTERNAL_ONLY
    pub credential_handle_ref: String,            // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,                    // data_class: INTERNAL_ONLY
    pub streaming: OpenAiStreamingMode,           // data_class: INTERNAL_ONLY
    pub safety_identifier_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub declared_modalities: Vec<OpenAiModality>, // data_class: PUBLIC
}

impl OpenAiProviderAdapterConfig {
    pub fn new(
        endpoint: impl Into<String>,
        credential_handle_ref: impl Into<String>,
        audit_tap_ref: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            credential_handle_ref: credential_handle_ref.into(),
            audit_tap_ref: audit_tap_ref.into(),
            streaming: OpenAiStreamingMode::Disabled,
            safety_identifier_ref: None,
            declared_modalities: default_openai_modalities(),
        }
    }

    pub fn with_streaming(mut self, streaming: OpenAiStreamingMode) -> Self {
        self.streaming = streaming;
        self
    }

    pub fn with_safety_identifier_ref(mut self, safety_identifier_ref: impl Into<String>) -> Self {
        self.safety_identifier_ref = Some(safety_identifier_ref.into());
        self
    }

    pub fn with_declared_modalities(mut self, modalities: Vec<OpenAiModality>) -> Self {
        self.declared_modalities = sorted_modalities(modalities);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenAiAdapterConfigError {
    EmptyEndpoint,
    NonHttpsEndpoint,
    EmptyCredentialHandleRef,
    RawCredentialMaterialRejected,
    NonOpaqueCredentialHandleRef,
    EmptyAuditTapRef,
    InvalidAuditTapRef,
    EmptySafetyIdentifierRef,
    UnsafeSafetyIdentifierRef,
    EmptyModalityDeclaration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenAiHttpMethod {
    Post,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAiProviderRequestEnvelope {
    pub method: OpenAiHttpMethod,                 // data_class: PUBLIC
    pub endpoint: String,                         // data_class: INTERNAL_ONLY
    pub path: String,                             // data_class: PUBLIC
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                  // data_class: INTERNAL_ONLY
    pub client_request_id_ref: String,            // data_class: INTERNAL_ONLY
    pub model_id: String,                         // data_class: INTERNAL_ONLY
    pub input_ref: String,                        // data_class: INTERNAL_ONLY
    pub credential_mode: CredentialMode,          // data_class: INTERNAL_ONLY
    pub credential_handle_ref: String,            // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,             // data_class: INTERNAL_ONLY
    pub route_evidence_refs: Vec<String>,         // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,                    // data_class: INTERNAL_ONLY
    pub streaming: OpenAiStreamingMode,           // data_class: INTERNAL_ONLY
    pub safety_identifier_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub declared_modalities: Vec<OpenAiModality>, // data_class: PUBLIC
    pub api_reference_refs: Vec<String>,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpenAiProviderStatus {
    Accepted {
        provider_request_id_ref: String,
        output_ref: String,
        usage_ref: String,
    },
    RateLimited {
        evidence_ref: String,
    },
    ServerError {
        evidence_ref: String,
    },
    AuthenticationFailed {
        evidence_ref: String,
    },
    InvalidRequest {
        evidence_ref: String,
    },
    ContentPolicyViolation {
        evidence_ref: String,
    },
    ContextLengthExceeded {
        evidence_ref: String,
    },
    ModelNotFound {
        evidence_ref: String,
    },
    Timeout {
        evidence_ref: String,
    },
}

#[derive(Debug)]
pub struct OpenAiProviderAdapter {
    config: OpenAiProviderAdapterConfig,
    next_status: OpenAiProviderStatus,
    last_envelope: Option<OpenAiProviderRequestEnvelope>,
}

impl OpenAiProviderAdapter {
    pub fn try_new(
        config: OpenAiProviderAdapterConfig,
        next_status: OpenAiProviderStatus,
    ) -> Result<Self, OpenAiAdapterConfigError> {
        validate_config(&config)?;
        Ok(Self {
            config,
            next_status,
            last_envelope: None,
        })
    }

    pub fn last_envelope(&self) -> Option<&OpenAiProviderRequestEnvelope> {
        self.last_envelope.as_ref()
    }

    pub fn set_next_status(&mut self, next_status: OpenAiProviderStatus) {
        self.next_status = next_status;
    }

    fn build_envelope(
        &self,
        request: &ProviderDispatchRequest,
    ) -> Result<OpenAiProviderRequestEnvelope, ProviderDispatchFailure> {
        validate_dispatch_request(request)?;

        Ok(OpenAiProviderRequestEnvelope {
            method: OpenAiHttpMethod::Post,
            endpoint: normalized_endpoint(&self.config.endpoint),
            path: OPENAI_RESPONSES_PATH.to_owned(),
            tenant_id: request.tenant_id.clone(),
            idempotency_key: request.idempotency_key.clone(),
            client_request_id_ref: client_request_id_ref(&request.idempotency_key),
            model_id: request.route_selection.model_id.clone(),
            input_ref: request.content_ref.clone(),
            credential_mode: request.route_selection.credential_mode,
            credential_handle_ref: self.config.credential_handle_ref.clone(),
            request_evidence_ref: request.request_evidence_ref.clone(),
            route_evidence_refs: sorted_unique(request.route_selection.evidence_refs.clone()),
            audit_tap_ref: self.config.audit_tap_ref.clone(),
            streaming: self.config.streaming,
            safety_identifier_ref: self.config.safety_identifier_ref.clone(),
            declared_modalities: self.config.declared_modalities.clone(),
            api_reference_refs: vec![
                OPENAI_RESPONSES_DOC.to_owned(),
                OPENAI_STREAMING_DOC.to_owned(),
                OPENAI_AUTH_DOC.to_owned(),
            ],
        })
    }

    fn response_from_status(
        &self,
        status: &OpenAiProviderStatus,
    ) -> Result<ProviderDispatchResponse, ProviderDispatchFailure> {
        match status {
            OpenAiProviderStatus::Accepted {
                provider_request_id_ref,
                output_ref,
                usage_ref,
            } => accepted_response(provider_request_id_ref, output_ref, usage_ref),
            OpenAiProviderStatus::RateLimited { evidence_ref } => {
                Err(provider_failure("openai:rate_limit", evidence_ref))
            }
            OpenAiProviderStatus::ServerError { evidence_ref } => {
                Err(provider_failure("openai:server_error", evidence_ref))
            }
            OpenAiProviderStatus::AuthenticationFailed { evidence_ref } => Err(provider_failure(
                "openai:authentication_failed",
                evidence_ref,
            )),
            OpenAiProviderStatus::InvalidRequest { evidence_ref } => {
                Err(provider_failure("openai:invalid_request", evidence_ref))
            }
            OpenAiProviderStatus::ContentPolicyViolation { evidence_ref } => Err(provider_failure(
                "openai:content_policy_violation",
                evidence_ref,
            )),
            OpenAiProviderStatus::ContextLengthExceeded { evidence_ref } => Err(provider_failure(
                "openai:context_length_exceeded",
                evidence_ref,
            )),
            OpenAiProviderStatus::ModelNotFound { evidence_ref } => {
                Err(provider_failure("openai:model_not_found", evidence_ref))
            }
            OpenAiProviderStatus::Timeout { evidence_ref } => {
                Err(provider_failure("openai:timeout", evidence_ref))
            }
        }
    }
}

impl ProviderDispatchPort for OpenAiProviderAdapter {
    fn dispatch(
        &mut self,
        request: ProviderDispatchRequest,
    ) -> Result<ProviderDispatchResponse, ProviderDispatchFailure> {
        let envelope = self.build_envelope(&request)?;
        self.last_envelope = Some(envelope);
        self.response_from_status(&self.next_status)
    }
}

fn validate_config(config: &OpenAiProviderAdapterConfig) -> Result<(), OpenAiAdapterConfigError> {
    let endpoint = config.endpoint.trim();
    if endpoint.is_empty() {
        return Err(OpenAiAdapterConfigError::EmptyEndpoint);
    }
    if !endpoint.starts_with("https://") || contains_whitespace(endpoint) {
        return Err(OpenAiAdapterConfigError::NonHttpsEndpoint);
    }
    validate_credential_handle_ref(&config.credential_handle_ref)?;
    validate_audit_tap_ref(&config.audit_tap_ref)?;
    validate_safety_identifier_ref(config.safety_identifier_ref.as_deref())?;
    if config.declared_modalities.is_empty() {
        return Err(OpenAiAdapterConfigError::EmptyModalityDeclaration);
    }
    Ok(())
}

fn validate_credential_handle_ref(value: &str) -> Result<(), OpenAiAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(OpenAiAdapterConfigError::EmptyCredentialHandleRef);
    }
    if contains_whitespace(trimmed) || is_openai_secret_like(trimmed) {
        return Err(OpenAiAdapterConfigError::RawCredentialMaterialRejected);
    }
    if !trimmed.contains("://") {
        return Err(OpenAiAdapterConfigError::NonOpaqueCredentialHandleRef);
    }
    Ok(())
}

fn validate_audit_tap_ref(value: &str) -> Result<(), OpenAiAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(OpenAiAdapterConfigError::EmptyAuditTapRef);
    }
    if contains_whitespace(trimmed) || !trimmed.contains("://") || is_openai_secret_like(trimmed) {
        return Err(OpenAiAdapterConfigError::InvalidAuditTapRef);
    }
    Ok(())
}

fn validate_safety_identifier_ref(value: Option<&str>) -> Result<(), OpenAiAdapterConfigError> {
    let Some(value) = value else {
        return Ok(());
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(OpenAiAdapterConfigError::EmptySafetyIdentifierRef);
    }
    if contains_whitespace(trimmed) || is_openai_secret_like(trimmed) {
        return Err(OpenAiAdapterConfigError::UnsafeSafetyIdentifierRef);
    }
    Ok(())
}

fn validate_dispatch_request(
    request: &ProviderDispatchRequest,
) -> Result<(), ProviderDispatchFailure> {
    if request.route_selection.provider != ModelProvider::OpenAi {
        return Err(provider_failure(
            "openai:route-provider-mismatch",
            "validation:openai-route-provider",
        ));
    }
    if request.idempotency_key.trim().is_empty() {
        return Err(provider_failure(
            "openai:idempotency_key_required",
            "validation:openai-idempotency-key",
        ));
    }
    if request.tenant_id.trim().is_empty() {
        return Err(provider_failure(
            "openai:tenant_required",
            "validation:openai-tenant",
        ));
    }
    if request.content_ref.trim().is_empty() {
        return Err(provider_failure(
            "openai:content_ref_required",
            "validation:openai-content-ref",
        ));
    }
    if !is_safe_opaque_ref(&request.content_ref) {
        return Err(provider_failure(
            "openai:content_ref_must_be_opaque",
            "validation:openai-content-ref-opaque",
        ));
    }
    if request.route_selection.model_id.trim().is_empty() {
        return Err(provider_failure(
            "openai:model_id_required",
            "validation:openai-model-id",
        ));
    }
    if request.request_evidence_ref.trim().is_empty() {
        return Err(provider_failure(
            "openai:request_evidence_required",
            "validation:openai-request-evidence",
        ));
    }
    if request.route_selection.evidence_refs.is_empty()
        || request
            .route_selection
            .evidence_refs
            .iter()
            .any(|evidence_ref| evidence_ref.trim().is_empty())
    {
        return Err(provider_failure(
            "openai:route_evidence_required",
            "validation:openai-route-evidence",
        ));
    }
    Ok(())
}

fn accepted_response(
    provider_request_id_ref: &str,
    output_ref: &str,
    usage_ref: &str,
) -> Result<ProviderDispatchResponse, ProviderDispatchFailure> {
    if provider_request_id_ref.trim().is_empty()
        || output_ref.trim().is_empty()
        || usage_ref.trim().is_empty()
        || !is_safe_opaque_ref(output_ref)
        || !is_safe_metadata_ref(provider_request_id_ref)
        || !is_safe_metadata_ref(usage_ref)
    {
        return Err(provider_failure(
            "openai:invalid_provider_status",
            "validation:openai-accepted-status",
        ));
    }

    Ok(ProviderDispatchResponse {
        output_ref: output_ref.to_owned(),
        provider_evidence_ref: format!("openai:response:{provider_request_id_ref}:{usage_ref}"),
        output_guardrail_findings: Vec::new(),
    })
}

fn provider_failure(reason: &str, evidence_ref: &str) -> ProviderDispatchFailure {
    let evidence_ref = if is_safe_metadata_ref(evidence_ref) {
        non_empty_evidence_ref(evidence_ref)
    } else {
        "openai:error:unsafe-evidence-ref".to_owned()
    };

    ProviderDispatchFailure {
        reason: reason.to_owned(),
        evidence_ref,
    }
}

fn non_empty_evidence_ref(evidence_ref: &str) -> String {
    if evidence_ref.trim().is_empty() {
        "openai:error:missing-evidence-ref".to_owned()
    } else {
        evidence_ref.to_owned()
    }
}

fn client_request_id_ref(idempotency_key: &str) -> String {
    format!("x-client-request-id://{idempotency_key}")
}

fn normalized_endpoint(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_owned()
}

fn contains_whitespace(value: &str) -> bool {
    value.chars().any(char::is_whitespace)
}

fn is_safe_opaque_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.contains("://")
        && !contains_whitespace(trimmed)
        && !is_openai_secret_like(trimmed)
}

fn is_safe_metadata_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && !contains_whitespace(trimmed) && !is_openai_secret_like(trimmed)
}

fn is_openai_secret_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("openai_api_key")
        || lower.contains("authorization:")
        || lower.contains("bearer ")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}
