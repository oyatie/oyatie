//! Intelligence credential resolver domain foundation.
//!
//! This crate models provider-credential BYOK references and short-lived
//! sidecar credential handles. It is deliberately pure Rust: no OpenBao client,
//! provider SDK, filesystem, network, raw provider key storage, or credential
//! value accessor lives here.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod credential_handle;
mod secret_reference;

pub use credential_handle::{
    CredentialAudience, CredentialHandle, CredentialHandleIssueError, CredentialHandleIssueRequest,
    MAX_CREDENTIAL_HANDLE_TTL_SECONDS,
};
pub use secret_reference::{
    CredentialProvider, SecretReference, SecretReferenceKind, SecretReferenceValidationError,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_default_reference_is_bound_to_tenant_and_provider() {
        let reference =
            SecretReference::parse("platform-default", "ten_a", CredentialProvider::Anthropic)
                .expect("valid platform default reference");

        assert_eq!(reference.kind(), SecretReferenceKind::PlatformDefault);
        assert_eq!(
            reference.canonical_ref(),
            "platform-default://ten_a/intelligence/provider/anthropic"
        );
    }

    #[test]
    fn handle_rejects_raw_secret_shaped_handle_id() {
        let error = CredentialHandle::issue(CredentialHandleIssueRequest {
            handle_id: "sk-test-secret".to_owned(),
            tenant_id: "ten_a".to_owned(),
            provider: CredentialProvider::OpenAi,
            audience: CredentialAudience::ProviderDispatch,
            issued_at_epoch_seconds: 0,
            expires_at_epoch_seconds: 60,
            generation: 1,
            sidecar_signature_ref: "sigref://openbao/handle/1".to_owned(),
        })
        .expect_err("secret-shaped handle id rejected");

        assert_eq!(error, CredentialHandleIssueError::RawSecretMaterialRejected);
    }

    #[test]
    fn handle_rejects_ambiguous_whitespace_around_handle_refs() {
        let error = CredentialHandle::issue(CredentialHandleIssueRequest {
            handle_id: " handle://ten_a/openai/gen-7".to_owned(),
            tenant_id: "ten_a".to_owned(),
            provider: CredentialProvider::OpenAi,
            audience: CredentialAudience::ProviderDispatch,
            issued_at_epoch_seconds: 0,
            expires_at_epoch_seconds: 60,
            generation: 1,
            sidecar_signature_ref: "sigref://openbao/handle/1".to_owned(),
        })
        .expect_err("whitespace-padded handle id rejected");

        assert_eq!(error, CredentialHandleIssueError::RawSecretMaterialRejected);
    }
}
