//! Intelligence credential-resolver adapter.
//!
//! This crate provides two adapter implementations for the `CredentialHandleIssuerPort`:
//!
//! 1. **`CredentialResolverAdapter`** — deterministic metadata-only sidecar adapter
//!    for testing and envelope-only scenarios.  Validates sidecar configuration and
//!    request metadata, builds sidecar request envelopes, and maps sidecar outcome
//!    metadata into short-lived opaque `CredentialHandle`s or sanitized failures.
//!
//! 2. **`OpenBaoKvAdapter`** — live OpenBao KV-v2 secret-fetch adapter.  Calls
//!    `GET /v1/secret/data/<seat-path>` over a bare hyper HTTP client (ADR-0090
//!    hyper preferred) and resolves the returned credential material into a
//!    short-lived in-memory `CredentialHandle`.  No vault SDK; no raw secret
//!    material ever stored on the returned handle.
//!
//! ADR citations: ADR-0083 (Tier-3 panic-free), ADR-0090 (hyper preferred),
//!                ADR-0509 (flat-clean-arch), ADR-0131.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod openbao_kv;
pub use openbao_kv::{
    OpenBaoKvAdapter, OpenBaoKvAdapterConfig, OpenBaoKvConfigError, RedactedToken,
};

pub use intelligence_credential_resolver_domain::{
    CredentialAudience, CredentialHandle, CredentialHandleIssueRequest, CredentialProvider,
    MAX_CREDENTIAL_HANDLE_TTL_SECONDS, SecretReference, SecretReferenceKind,
};
pub use intelligence_credential_resolver_usecase::{
    CredentialHandleIssueFailure, CredentialHandleIssuerPort, CredentialHandleRequest,
    CredentialResolutionCacheStatus, CredentialResolutionDenialKind, CredentialResolutionInput,
    CredentialResolutionReceipt, CredentialResolutionStatus, CredentialResolverAuditEvent,
    CredentialResolverAuditEventKind, CredentialResolverAuditSink, CredentialResolverUsecase,
    CredentialRotationDenialKind, CredentialRotationEvent, CredentialRotationReceipt,
    CredentialRotationStatus,
};

const ADAPTER_REFERENCE_REF: &str =
    "spec://oyatie/intelligence/credential-resolver-adapter-foundation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialSidecarTransportMode {
    EnvelopeOnly,
    UnixSocketSidecar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialSidecarAdapterConfig {
    pub sidecar_channel_ref: String,  // data_class: INTERNAL_ONLY
    pub handle_signer_ref: String,    // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,        // data_class: INTERNAL_ONLY
    pub sidecar_audience_ref: String, // data_class: INTERNAL_ONLY
    pub max_handle_ttl_seconds: u64,  // data_class: INTERNAL_ONLY
    pub transport_mode: CredentialSidecarTransportMode, // data_class: INTERNAL_ONLY
}

impl CredentialSidecarAdapterConfig {
    pub fn new(
        sidecar_channel_ref: impl Into<String>,
        handle_signer_ref: impl Into<String>,
        audit_tap_ref: impl Into<String>,
        sidecar_audience_ref: impl Into<String>,
    ) -> Self {
        Self {
            sidecar_channel_ref: sidecar_channel_ref.into(),
            handle_signer_ref: handle_signer_ref.into(),
            audit_tap_ref: audit_tap_ref.into(),
            sidecar_audience_ref: sidecar_audience_ref.into(),
            max_handle_ttl_seconds: MAX_CREDENTIAL_HANDLE_TTL_SECONDS,
            transport_mode: CredentialSidecarTransportMode::EnvelopeOnly,
        }
    }

    pub fn with_transport_mode(mut self, mode: CredentialSidecarTransportMode) -> Self {
        self.transport_mode = mode;
        self
    }

    pub fn with_max_handle_ttl_seconds(mut self, ttl_seconds: u64) -> Self {
        self.max_handle_ttl_seconds = ttl_seconds;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialSidecarAdapterConfigError {
    EmptySidecarChannelRef,
    InvalidSidecarChannelRef,
    EmptyHandleSignerRef,
    InvalidHandleSignerRef,
    EmptyAuditTapRef,
    InvalidAuditTapRef,
    EmptySidecarAudienceRef,
    InvalidSidecarAudienceRef,
    InvalidHandleTtl,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CredentialSidecarStatus {
    Issued {
        handle_id_ref: String,
        generation: u64,
        sidecar_signature_ref: String,
        evidence_ref: String,
    },
    Denied {
        evidence_ref: String,
    },
    RateLimited {
        evidence_ref: String,
    },
    SidecarError {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialSidecarRequestEnvelope {
    pub transport_mode: CredentialSidecarTransportMode, // data_class: INTERNAL_ONLY
    pub sidecar_channel_ref: String,                    // data_class: INTERNAL_ONLY
    pub handle_signer_ref: String,                      // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,                          // data_class: INTERNAL_ONLY
    pub sidecar_audience_ref: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub provider: CredentialProvider,                   // data_class: INTERNAL_ONLY
    pub audience: CredentialAudience,                   // data_class: INTERNAL_ONLY
    pub secret_reference_kind: SecretReferenceKind,     // data_class: INTERNAL_ONLY
    pub secret_reference_ref: String,                   // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,                   // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,                // data_class: INTERNAL_ONLY
    pub requested_ttl_seconds: u64,                     // data_class: INTERNAL_ONLY
    pub adapter_reference_refs: Vec<String>,            // data_class: PUBLIC
}

#[derive(Debug)]
pub struct CredentialResolverAdapter {
    config: CredentialSidecarAdapterConfig,
    next_status: CredentialSidecarStatus,
    last_envelope: Option<CredentialSidecarRequestEnvelope>,
}

impl CredentialResolverAdapter {
    pub fn try_new(
        config: CredentialSidecarAdapterConfig,
        next_status: CredentialSidecarStatus,
    ) -> Result<Self, CredentialSidecarAdapterConfigError> {
        validate_config(&config)?;
        Ok(Self {
            config,
            next_status,
            last_envelope: None,
        })
    }

    pub fn last_envelope(&self) -> Option<&CredentialSidecarRequestEnvelope> {
        self.last_envelope.as_ref()
    }

    pub fn set_next_status(&mut self, status: CredentialSidecarStatus) {
        self.next_status = status;
    }

    fn build_envelope(
        &self,
        request: &CredentialHandleRequest,
    ) -> CredentialSidecarRequestEnvelope {
        CredentialSidecarRequestEnvelope {
            transport_mode: self.config.transport_mode,
            sidecar_channel_ref: self.config.sidecar_channel_ref.clone(),
            handle_signer_ref: self.config.handle_signer_ref.clone(),
            audit_tap_ref: self.config.audit_tap_ref.clone(),
            sidecar_audience_ref: self.config.sidecar_audience_ref.clone(),
            tenant_id: request.tenant_id.clone(),
            provider: request.provider,
            audience: request.audience,
            secret_reference_kind: request.secret_reference.kind(),
            secret_reference_ref: request.secret_reference.canonical_ref().to_owned(),
            request_evidence_ref: request.request_evidence_ref.clone(),
            requested_at_epoch_seconds: request.now_epoch_seconds,
            requested_ttl_seconds: self.config.max_handle_ttl_seconds,
            adapter_reference_refs: vec![ADAPTER_REFERENCE_REF.to_owned()],
        }
    }
}

impl CredentialHandleIssuerPort for CredentialResolverAdapter {
    fn issue_handle(
        &mut self,
        request: CredentialHandleRequest,
    ) -> Result<CredentialHandle, CredentialHandleIssueFailure> {
        validate_request(&request)?;
        validate_status_metadata(&self.next_status)?;
        let envelope = self.build_envelope(&request);
        self.last_envelope = Some(envelope);
        handle_from_status(&request, &self.config, &self.next_status)
    }
}

fn validate_config(
    config: &CredentialSidecarAdapterConfig,
) -> Result<(), CredentialSidecarAdapterConfigError> {
    require_config_ref(
        &config.sidecar_channel_ref,
        CredentialSidecarAdapterConfigError::EmptySidecarChannelRef,
        CredentialSidecarAdapterConfigError::InvalidSidecarChannelRef,
    )?;
    require_config_ref(
        &config.handle_signer_ref,
        CredentialSidecarAdapterConfigError::EmptyHandleSignerRef,
        CredentialSidecarAdapterConfigError::InvalidHandleSignerRef,
    )?;
    require_config_ref(
        &config.audit_tap_ref,
        CredentialSidecarAdapterConfigError::EmptyAuditTapRef,
        CredentialSidecarAdapterConfigError::InvalidAuditTapRef,
    )?;
    require_config_ref(
        &config.sidecar_audience_ref,
        CredentialSidecarAdapterConfigError::EmptySidecarAudienceRef,
        CredentialSidecarAdapterConfigError::InvalidSidecarAudienceRef,
    )?;
    if config.max_handle_ttl_seconds == 0
        || config.max_handle_ttl_seconds > MAX_CREDENTIAL_HANDLE_TTL_SECONDS
    {
        return Err(CredentialSidecarAdapterConfigError::InvalidHandleTtl);
    }
    Ok(())
}

fn require_config_ref(
    value: &str,
    empty: CredentialSidecarAdapterConfigError,
    invalid: CredentialSidecarAdapterConfigError,
) -> Result<(), CredentialSidecarAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(empty);
    }
    if is_safe_opaque_ref(value) {
        Ok(())
    } else {
        Err(invalid)
    }
}

fn validate_request(request: &CredentialHandleRequest) -> Result<(), CredentialHandleIssueFailure> {
    require_tenant(&request.tenant_id, "validation:credential-sidecar-tenant")?;
    if request.secret_reference.bound_tenant() != request.tenant_id
        || request.secret_reference.provider() != request.provider
    {
        return Err(dispatch_failure(
            "credential-sidecar request is not bound to secret reference metadata",
            "validation:credential-sidecar-secret-binding",
        ));
    }
    require_opaque(
        request.secret_reference.canonical_ref(),
        "validation:credential-sidecar-secret-reference",
    )?;
    require_opaque(
        &request.request_evidence_ref,
        "validation:credential-sidecar-request-evidence",
    )?;
    Ok(())
}

fn validate_status_metadata(
    status: &CredentialSidecarStatus,
) -> Result<(), CredentialHandleIssueFailure> {
    let valid = match status {
        CredentialSidecarStatus::Issued {
            handle_id_ref,
            generation,
            sidecar_signature_ref,
            evidence_ref,
        } => {
            *generation > 0
                && is_safe_handle_ref(handle_id_ref)
                && is_safe_opaque_ref(sidecar_signature_ref)
                && is_safe_opaque_ref(evidence_ref)
        }
        CredentialSidecarStatus::Denied { evidence_ref }
        | CredentialSidecarStatus::RateLimited { evidence_ref }
        | CredentialSidecarStatus::SidecarError { evidence_ref }
        | CredentialSidecarStatus::AuthError { evidence_ref }
        | CredentialSidecarStatus::InvalidRequest { evidence_ref }
        | CredentialSidecarStatus::Timeout { evidence_ref } => is_safe_opaque_ref(evidence_ref),
    };
    if valid {
        Ok(())
    } else {
        Err(dispatch_failure(
            "credential-sidecar status metadata is invalid",
            "validation:credential-sidecar-status-metadata",
        ))
    }
}

fn handle_from_status(
    request: &CredentialHandleRequest,
    config: &CredentialSidecarAdapterConfig,
    status: &CredentialSidecarStatus,
) -> Result<CredentialHandle, CredentialHandleIssueFailure> {
    match status {
        CredentialSidecarStatus::Issued {
            handle_id_ref,
            generation,
            sidecar_signature_ref,
            evidence_ref: _,
        } => CredentialHandle::issue(CredentialHandleIssueRequest {
            handle_id: handle_id_ref.clone(),
            tenant_id: request.tenant_id.clone(),
            provider: request.provider,
            audience: request.audience,
            issued_at_epoch_seconds: request.now_epoch_seconds,
            expires_at_epoch_seconds: request
                .now_epoch_seconds
                .saturating_add(config.max_handle_ttl_seconds),
            generation: *generation,
            sidecar_signature_ref: sidecar_signature_ref.clone(),
        })
        .map_err(|_| {
            dispatch_failure(
                "credential-sidecar returned invalid handle metadata",
                "validation:credential-sidecar-issued-handle",
            )
        }),
        CredentialSidecarStatus::Denied { evidence_ref } => {
            Err(dispatch_failure("credential-sidecar:denied", evidence_ref))
        }
        CredentialSidecarStatus::RateLimited { evidence_ref } => Err(dispatch_failure(
            "credential-sidecar:rate_limited",
            evidence_ref,
        )),
        CredentialSidecarStatus::SidecarError { evidence_ref } => Err(dispatch_failure(
            "credential-sidecar:sidecar_error",
            evidence_ref,
        )),
        CredentialSidecarStatus::AuthError { evidence_ref } => Err(dispatch_failure(
            "credential-sidecar:auth_error",
            evidence_ref,
        )),
        CredentialSidecarStatus::InvalidRequest { evidence_ref } => Err(dispatch_failure(
            "credential-sidecar:invalid_request",
            evidence_ref,
        )),
        CredentialSidecarStatus::Timeout { evidence_ref } => {
            Err(dispatch_failure("credential-sidecar:timeout", evidence_ref))
        }
    }
}

fn require_tenant(value: &str, evidence_ref: &str) -> Result<(), CredentialHandleIssueFailure> {
    if is_safe_tenant_id(value) {
        Ok(())
    } else {
        Err(dispatch_failure(
            "credential-sidecar requires tenant metadata",
            evidence_ref,
        ))
    }
}

fn require_opaque(value: &str, evidence_ref: &str) -> Result<(), CredentialHandleIssueFailure> {
    if is_safe_opaque_ref(value) {
        Ok(())
    } else {
        Err(dispatch_failure(
            "credential-sidecar requires opaque metadata refs",
            evidence_ref,
        ))
    }
}

fn dispatch_failure(reason: &str, evidence_ref: &str) -> CredentialHandleIssueFailure {
    CredentialHandleIssueFailure {
        reason: safe_failure_reason(reason),
        evidence_ref: safe_evidence_ref(evidence_ref),
    }
}

fn safe_failure_reason(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() || contains_raw_secret_material(trimmed) {
        "credential-sidecar failed".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn safe_evidence_ref(value: &str) -> String {
    let trimmed = value.trim();
    if is_safe_opaque_ref(trimmed) || is_safe_metadata_ref(trimmed) {
        trimmed.to_owned()
    } else {
        "credential-sidecar:unsafe-evidence-ref".to_owned()
    }
}

fn is_safe_handle_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.starts_with("handle://")
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
        || lower.contains("password=")
        || lower.contains("token=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw provider key")
        || lower.contains("raw secret")
        || lower.contains("credential value")
        || lower.contains("private key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct AuditCollector {
        events: Vec<CredentialResolverAuditEvent>,
    }

    impl CredentialResolverAuditSink for AuditCollector {
        fn record(&mut self, event: CredentialResolverAuditEvent) {
            self.events.push(event);
        }
    }

    #[test]
    fn builds_metadata_only_sidecar_envelope_and_issues_bound_handle() {
        let mut adapter = valid_adapter(CredentialSidecarStatus::Issued {
            handle_id_ref: "handle://ten_a/openai/gen-1".to_owned(),
            generation: 1,
            sidecar_signature_ref: "sigref://openbao/handle/1".to_owned(),
            evidence_ref: "sidecar:evidence:issued".to_owned(),
        });

        let handle = adapter
            .issue_handle(valid_handle_request())
            .expect("handle");

        assert_eq!(handle.bound_tenant(), "ten_a");
        assert_eq!(handle.bound_provider(), CredentialProvider::OpenAi);
        assert_eq!(handle.generation(), 1);
        let envelope = adapter.last_envelope().expect("envelope");
        assert_eq!(envelope.provider, CredentialProvider::OpenAi);
        assert_eq!(
            envelope.secret_reference_ref,
            "openbao://secret/ten_a/intelligence/provider/openai"
        );
        assert_eq!(
            envelope.transport_mode,
            CredentialSidecarTransportMode::EnvelopeOnly
        );
        assert_eq!(envelope.requested_ttl_seconds, 60);
    }

    #[test]
    fn usecase_resolves_through_adapter_without_exposing_secret_material() {
        let adapter = valid_adapter(CredentialSidecarStatus::Issued {
            handle_id_ref: "handle://ten_a/openai/gen-1".to_owned(),
            generation: 1,
            sidecar_signature_ref: "sigref://openbao/handle/1".to_owned(),
            evidence_ref: "sidecar:evidence:issued".to_owned(),
        });
        let mut usecase = CredentialResolverUsecase::new(adapter, AuditCollector::default());

        let receipt = usecase.resolve(valid_resolution_input("idem-adapter-1"));
        let (adapter, audit) = usecase.into_parts();
        let debug = format!("{receipt:?}{:?}{:?}", adapter.last_envelope(), audit.events);

        assert_eq!(receipt.status, CredentialResolutionStatus::Resolved);
        assert_eq!(
            receipt.cache_status,
            Some(CredentialResolutionCacheStatus::MissIssued)
        );
        assert!(adapter.last_envelope().is_some());
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("raw provider key"));
        assert!(!debug.contains("credential value"));
    }

    #[test]
    fn rejects_invalid_config_or_raw_secret_like_refs() {
        assert_eq!(
            CredentialResolverAdapter::try_new(
                valid_config().with_max_handle_ttl_seconds(61),
                issued_status(),
            )
            .unwrap_err(),
            CredentialSidecarAdapterConfigError::InvalidHandleTtl
        );
        assert_eq!(
            CredentialResolverAdapter::try_new(
                CredentialSidecarAdapterConfig::new(
                    "sk-test-secret",
                    "credential-signer:openbao:active",
                    "audit-tap:credential-resolver:1",
                    "audience:credential-resolver:sidecar",
                ),
                issued_status(),
            )
            .unwrap_err(),
            CredentialSidecarAdapterConfigError::InvalidSidecarChannelRef
        );
    }

    #[test]
    fn rejects_invalid_request_metadata_before_envelope() {
        let mut adapter = valid_adapter(issued_status());
        let mut request = valid_handle_request();
        request.request_evidence_ref = "sk-test raw provider key".to_owned();

        let failure = adapter.issue_handle(request).expect_err("invalid request");

        assert_eq!(adapter.last_envelope(), None);
        let debug = format!("{failure:?}");
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("raw provider key"));
    }

    #[test]
    fn maps_sidecar_outcomes_to_sanitized_failures() {
        let cases = [
            (
                CredentialSidecarStatus::Denied {
                    evidence_ref: "sidecar:evidence:denied".to_owned(),
                },
                "credential-sidecar:denied",
            ),
            (
                CredentialSidecarStatus::RateLimited {
                    evidence_ref: "sidecar:evidence:rate-limited".to_owned(),
                },
                "credential-sidecar:rate_limited",
            ),
            (
                CredentialSidecarStatus::SidecarError {
                    evidence_ref: "sidecar:evidence:error".to_owned(),
                },
                "credential-sidecar:sidecar_error",
            ),
            (
                CredentialSidecarStatus::AuthError {
                    evidence_ref: "sidecar:evidence:auth".to_owned(),
                },
                "credential-sidecar:auth_error",
            ),
            (
                CredentialSidecarStatus::InvalidRequest {
                    evidence_ref: "sidecar:evidence:invalid".to_owned(),
                },
                "credential-sidecar:invalid_request",
            ),
            (
                CredentialSidecarStatus::Timeout {
                    evidence_ref: "sidecar:evidence:timeout".to_owned(),
                },
                "credential-sidecar:timeout",
            ),
        ];

        for (status, reason) in cases {
            let mut adapter = valid_adapter(status);
            let failure = adapter
                .issue_handle(valid_handle_request())
                .expect_err("sidecar failure");
            assert_eq!(failure.reason, reason);
        }
    }

    #[test]
    fn invalid_sidecar_status_metadata_fails_before_envelope() {
        let mut adapter = valid_adapter(CredentialSidecarStatus::Issued {
            handle_id_ref: "sk-test-raw-secret".to_owned(),
            generation: 1,
            sidecar_signature_ref: "sigref://openbao/handle/1".to_owned(),
            evidence_ref: "sidecar:evidence:issued".to_owned(),
        });

        let failure = adapter
            .issue_handle(valid_handle_request())
            .expect_err("invalid status");

        assert_eq!(
            failure.reason,
            "credential-sidecar status metadata is invalid"
        );
        assert_eq!(adapter.last_envelope(), None);
        let debug = format!("{failure:?}");
        assert!(!debug.contains("sk-test-raw-secret"));
    }

    #[test]
    fn envelope_and_failures_never_contain_raw_secret_bytes() {
        let mut adapter = valid_adapter(CredentialSidecarStatus::Issued {
            handle_id_ref: "handle://ten_a/openai/gen-1".to_owned(),
            generation: 1,
            sidecar_signature_ref: "sigref://openbao/handle/1".to_owned(),
            evidence_ref: "sidecar:evidence:issued".to_owned(),
        });
        let handle = adapter
            .issue_handle(valid_handle_request())
            .expect("handle");
        let rendered = format!("{handle:?}{:?}", adapter.last_envelope());

        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw provider key"));
        assert!(!rendered.contains("credential value"));
        assert!(!rendered.contains("Authorization:"));
    }

    fn valid_adapter(status: CredentialSidecarStatus) -> CredentialResolverAdapter {
        CredentialResolverAdapter::try_new(valid_config(), status).expect("valid config")
    }

    fn valid_config() -> CredentialSidecarAdapterConfig {
        CredentialSidecarAdapterConfig::new(
            "sidecar-channel:credential-resolver:local",
            "credential-signer:openbao:active",
            "audit-tap:credential-resolver:1",
            "audience:credential-resolver:sidecar",
        )
    }

    fn issued_status() -> CredentialSidecarStatus {
        CredentialSidecarStatus::Issued {
            handle_id_ref: "handle://ten_a/openai/gen-1".to_owned(),
            generation: 1,
            sidecar_signature_ref: "sigref://openbao/handle/1".to_owned(),
            evidence_ref: "sidecar:evidence:issued".to_owned(),
        }
    }

    fn valid_handle_request() -> CredentialHandleRequest {
        CredentialHandleRequest {
            tenant_id: "ten_a".to_owned(),
            provider: CredentialProvider::OpenAi,
            audience: CredentialAudience::ProviderDispatch,
            secret_reference: SecretReference::parse(
                "${openbao:secret/ten_a/intelligence/provider/openai}",
                "ten_a",
                CredentialProvider::OpenAi,
            )
            .expect("secret ref"),
            request_evidence_ref: "request:credential:1".to_owned(),
            now_epoch_seconds: 1_000,
        }
    }

    fn valid_resolution_input(idempotency_key: &str) -> CredentialResolutionInput {
        CredentialResolutionInput {
            idempotency_key: idempotency_key.to_owned(),
            tenant_id: "ten_a".to_owned(),
            provider: CredentialProvider::OpenAi,
            audience: CredentialAudience::ProviderDispatch,
            secret_reference_text: "${openbao:secret/ten_a/intelligence/provider/openai}"
                .to_owned(),
            request_evidence_ref: "request:credential:1".to_owned(),
            now_epoch_seconds: 1_000,
        }
    }
}
