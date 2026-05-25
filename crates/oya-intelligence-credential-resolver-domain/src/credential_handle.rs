use std::fmt;

use crate::CredentialProvider;

pub const MAX_CREDENTIAL_HANDLE_TTL_SECONDS: u64 = 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CredentialAudience {
    ProviderDispatch,
    ProviderHealthProbe,
    RotationValidation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CredentialHandleIssueError {
    EmptyHandleId,
    EmptyTenant,
    InvalidTenant,
    EmptySignatureRef,
    RawSecretMaterialRejected,
    NonOpaqueHandleId,
    InvalidSignatureRef,
    ZeroGeneration,
    ExpiryNotAfterIssue,
    TtlExceedsSidecarCeiling,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialHandleIssueRequest {
    pub handle_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub provider: CredentialProvider,  // data_class: INTERNAL_ONLY
    pub audience: CredentialAudience,  // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub generation: u64,               // data_class: INTERNAL_ONLY
    pub sidecar_signature_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Eq, PartialEq)]
pub struct CredentialHandle {
    handle_id: String,                  // data_class: INTERNAL_ONLY
    bound_tenant: String,               // data_class: INTERNAL_ONLY
    bound_provider: CredentialProvider, // data_class: INTERNAL_ONLY
    bound_audience: CredentialAudience, // data_class: INTERNAL_ONLY
    issued_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    expires_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    generation: u64,                    // data_class: INTERNAL_ONLY
    sidecar_signature_ref: String,      // data_class: INTERNAL_ONLY
}

impl CredentialHandle {
    pub fn issue(
        request: CredentialHandleIssueRequest,
    ) -> Result<Self, CredentialHandleIssueError> {
        validate_handle_id(&request.handle_id)?;
        validate_tenant(&request.tenant_id)?;
        validate_signature_ref(&request.sidecar_signature_ref)?;
        if request.generation == 0 {
            return Err(CredentialHandleIssueError::ZeroGeneration);
        }
        if request.expires_at_epoch_seconds <= request.issued_at_epoch_seconds {
            return Err(CredentialHandleIssueError::ExpiryNotAfterIssue);
        }
        if request.expires_at_epoch_seconds - request.issued_at_epoch_seconds
            > MAX_CREDENTIAL_HANDLE_TTL_SECONDS
        {
            return Err(CredentialHandleIssueError::TtlExceedsSidecarCeiling);
        }

        Ok(Self {
            handle_id: request.handle_id,
            bound_tenant: request.tenant_id,
            bound_provider: request.provider,
            bound_audience: request.audience,
            issued_at_epoch_seconds: request.issued_at_epoch_seconds,
            expires_at_epoch_seconds: request.expires_at_epoch_seconds,
            generation: request.generation,
            sidecar_signature_ref: request.sidecar_signature_ref,
        })
    }

    pub fn handle_id_ref(&self) -> &str {
        &self.handle_id
    }

    pub fn bound_tenant(&self) -> &str {
        &self.bound_tenant
    }

    pub fn bound_provider(&self) -> CredentialProvider {
        self.bound_provider
    }

    pub fn bound_audience(&self) -> CredentialAudience {
        self.bound_audience
    }

    pub fn issued_at_epoch_seconds(&self) -> u64 {
        self.issued_at_epoch_seconds
    }

    pub fn expires_at_epoch_seconds(&self) -> u64 {
        self.expires_at_epoch_seconds
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn sidecar_signature_ref(&self) -> &str {
        &self.sidecar_signature_ref
    }

    pub fn is_expired(&self, now_epoch_seconds: u64) -> bool {
        now_epoch_seconds >= self.expires_at_epoch_seconds
    }

    pub fn is_valid_for(
        &self,
        tenant_id: &str,
        provider: CredentialProvider,
        audience: CredentialAudience,
        now_epoch_seconds: u64,
    ) -> bool {
        self.bound_tenant == tenant_id
            && self.bound_provider == provider
            && self.bound_audience == audience
            && !self.is_expired(now_epoch_seconds)
    }
}

impl fmt::Debug for CredentialHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CredentialHandle")
            .field("handle_id", &"REDACTED_OPAQUE_HANDLE_REF")
            .field("bound_tenant", &self.bound_tenant)
            .field("bound_provider", &self.bound_provider)
            .field("bound_audience", &self.bound_audience)
            .field("issued_at_epoch_seconds", &self.issued_at_epoch_seconds)
            .field("expires_at_epoch_seconds", &self.expires_at_epoch_seconds)
            .field("generation", &self.generation)
            .field("sidecar_signature_ref", &"REDACTED_SIGNATURE_REF")
            .finish()
    }
}

fn validate_handle_id(value: &str) -> Result<(), CredentialHandleIssueError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CredentialHandleIssueError::EmptyHandleId);
    }
    if value != trimmed || contains_raw_secret_material(trimmed) || contains_whitespace(trimmed) {
        return Err(CredentialHandleIssueError::RawSecretMaterialRejected);
    }
    if !trimmed.contains("://") {
        return Err(CredentialHandleIssueError::NonOpaqueHandleId);
    }
    Ok(())
}

fn validate_tenant(value: &str) -> Result<(), CredentialHandleIssueError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CredentialHandleIssueError::EmptyTenant);
    }
    if value != trimmed
        || !trimmed.starts_with("ten_")
        || contains_whitespace(trimmed)
        || trimmed.contains('/')
    {
        return Err(CredentialHandleIssueError::InvalidTenant);
    }
    Ok(())
}

fn validate_signature_ref(value: &str) -> Result<(), CredentialHandleIssueError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CredentialHandleIssueError::EmptySignatureRef);
    }
    if value != trimmed
        || contains_raw_secret_material(trimmed)
        || contains_whitespace(trimmed)
        || !trimmed.contains("://")
    {
        return Err(CredentialHandleIssueError::InvalidSignatureRef);
    }
    Ok(())
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
}
