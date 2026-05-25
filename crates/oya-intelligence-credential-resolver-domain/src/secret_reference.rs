use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CredentialProvider {
    Anthropic,
    AzureOpenAi,
    Bedrock,
    GoogleVertex,
    OpenAi,
}

impl CredentialProvider {
    pub fn path_label(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::AzureOpenAi => "azure-openai",
            Self::Bedrock => "bedrock",
            Self::GoogleVertex => "google-vertex",
            Self::OpenAi => "openai",
        }
    }
}

impl fmt::Display for CredentialProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.path_label())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SecretReferenceKind {
    OpenBaoPath,
    PlatformDefault,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecretReferenceValidationError {
    EmptyReference,
    EmptyTenant,
    InvalidTenant,
    RawSecretMaterialRejected,
    UnsupportedReferenceKind,
    MalformedOpenBaoPath,
    TenantMismatch,
    ProviderMismatch,
}

#[derive(Clone, Eq, PartialEq)]
pub struct SecretReference {
    kind: SecretReferenceKind,    // data_class: INTERNAL_ONLY
    canonical_ref: String,        // data_class: INTERNAL_ONLY
    bound_tenant: String,         // data_class: INTERNAL_ONLY
    provider: CredentialProvider, // data_class: INTERNAL_ONLY
}

impl SecretReference {
    pub fn parse(
        input: &str,
        bound_tenant: &str,
        provider: CredentialProvider,
    ) -> Result<Self, SecretReferenceValidationError> {
        validate_tenant(bound_tenant)?;
        let trimmed_input = input.trim();
        if trimmed_input.is_empty() {
            return Err(SecretReferenceValidationError::EmptyReference);
        }
        if input != trimmed_input
            || contains_raw_secret_material(trimmed_input)
            || contains_whitespace(trimmed_input)
        {
            return Err(SecretReferenceValidationError::RawSecretMaterialRejected);
        }
        let input = trimmed_input;

        if input == "platform-default" || input == "platform-default://intelligence/provider" {
            return Ok(Self {
                kind: SecretReferenceKind::PlatformDefault,
                canonical_ref: format!(
                    "platform-default://{bound_tenant}/intelligence/provider/{provider}"
                ),
                bound_tenant: bound_tenant.to_owned(),
                provider,
            });
        }

        let Some(path) = normalized_openbao_path(input) else {
            return Err(SecretReferenceValidationError::UnsupportedReferenceKind);
        };
        let mut parts = path.split('/');
        match (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            (
                Some("secret"),
                Some(tenant),
                Some("intelligence"),
                Some("provider"),
                Some(provider_label),
                None,
            ) => {
                if tenant != bound_tenant {
                    return Err(SecretReferenceValidationError::TenantMismatch);
                }
                if provider_label != provider.path_label() {
                    return Err(SecretReferenceValidationError::ProviderMismatch);
                }
                Ok(Self {
                    kind: SecretReferenceKind::OpenBaoPath,
                    canonical_ref: format!(
                        "openbao://secret/{tenant}/intelligence/provider/{provider_label}"
                    ),
                    bound_tenant: bound_tenant.to_owned(),
                    provider,
                })
            }
            _ => Err(SecretReferenceValidationError::MalformedOpenBaoPath),
        }
    }

    pub fn kind(&self) -> SecretReferenceKind {
        self.kind
    }

    pub fn canonical_ref(&self) -> &str {
        &self.canonical_ref
    }

    pub fn bound_tenant(&self) -> &str {
        &self.bound_tenant
    }

    pub fn provider(&self) -> CredentialProvider {
        self.provider
    }
}

impl fmt::Debug for SecretReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecretReference")
            .field("kind", &self.kind)
            .field("canonical_ref", &self.canonical_ref)
            .field("bound_tenant", &self.bound_tenant)
            .field("provider", &self.provider)
            .finish()
    }
}

fn validate_tenant(tenant: &str) -> Result<(), SecretReferenceValidationError> {
    let trimmed = tenant.trim();
    if trimmed.is_empty() {
        return Err(SecretReferenceValidationError::EmptyTenant);
    }
    if tenant != trimmed
        || !trimmed.starts_with("ten_")
        || contains_whitespace(trimmed)
        || trimmed.contains('/')
    {
        return Err(SecretReferenceValidationError::InvalidTenant);
    }
    Ok(())
}

fn normalized_openbao_path(input: &str) -> Option<&str> {
    input
        .strip_prefix("${openbao:")
        .and_then(|rest| rest.strip_suffix('}'))
        .or_else(|| input.strip_prefix("openbao://"))
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
