//! Foundry account-auth kernel — neutral identity + reference value types.
//!
//! Per ADR-0056 (12-layer enum, port-in-kernel): the kernel holds the
//! value types that ports exchange across product boundaries. Adapter
//! kernels in outer rings consume directly from here; the domain
//! re-exports for backwards-compat so existing call sites stay valid.
//!
//! No I/O. No provider-specific code. No state-machine behavior — that
//! lives in `intelligence-account-domain`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AccountId(pub String);

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SessionId(pub String);

/// Allowlisted provider family. Adding a family requires an ADR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderFamily {
    Aws,
    Oci,
    Claude,
    OpenAiOrCodex,
    Gemini,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderFamilyError(pub String);

impl fmt::Display for ProviderFamilyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "provider family not allowlisted: {}", self.0)
    }
}

impl TryFrom<&str> for ProviderFamily {
    type Error = ProviderFamilyError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "AWS" => Ok(Self::Aws),
            "OCI" => Ok(Self::Oci),
            "Claude" => Ok(Self::Claude),
            "OpenAIOrCodex" => Ok(Self::OpenAiOrCodex),
            "Gemini" => Ok(Self::Gemini),
            other => Err(ProviderFamilyError(other.to_owned())),
        }
    }
}

/// Reference to a secret in some external store. Carries no raw
/// bytes — `Debug` redacts, no `Display` impl. `Hash` is derived so
/// `SecretReference` can be used directly as a map key by the same-value
/// invariant: two refs hash equal iff the underlying `sref://` strings are
/// equal. The inner string is NOT exposed by `Hash` (the trait derives over
/// the field but produces only a hash code, not the value).
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SecretReference(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretReferenceError(pub String);

impl fmt::Display for SecretReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid secret reference: {}", self.0)
    }
}

impl SecretReference {
    pub fn new(sref: String) -> Result<Self, SecretReferenceError> {
        if !sref.starts_with("sref://") {
            return Err(SecretReferenceError("must use sref:// scheme".to_owned()));
        }
        if sref.len() <= 7 {
            return Err(SecretReferenceError("reference body is empty".to_owned()));
        }
        Ok(Self(sref))
    }
}

impl fmt::Debug for SecretReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretReference(sref://[REDACTED])")
    }
}

/// `AuthToken` returned by an authenticated provider session.
///
/// SECURITY: inner bytes are never exposed — `Debug` renders redacted and no
/// `Display` is implemented, so a token cannot reach a log line by accident.
#[derive(Clone, Eq, PartialEq)]
pub struct AuthToken {
    issued_at_epoch_secs: u64,
    expires_at_epoch_secs: u64,
    provider_family: ProviderFamily,
    token_id_redacted: String,
}

impl AuthToken {
    pub fn new(
        issued_at_epoch_secs: u64,
        expires_at_epoch_secs: u64,
        provider_family: ProviderFamily,
        token_id_redacted: String,
    ) -> Result<Self, AuthError> {
        if expires_at_epoch_secs <= issued_at_epoch_secs {
            return Err(AuthError::Expired);
        }
        if token_id_redacted.is_empty() {
            return Err(AuthError::InvalidToken);
        }
        Ok(Self {
            issued_at_epoch_secs,
            expires_at_epoch_secs,
            provider_family,
            token_id_redacted,
        })
    }

    pub fn issued_at_epoch_secs(&self) -> u64 {
        self.issued_at_epoch_secs
    }
    pub fn expires_at_epoch_secs(&self) -> u64 {
        self.expires_at_epoch_secs
    }
    pub fn provider_family(&self) -> ProviderFamily {
        self.provider_family
    }
    pub fn token_id_redacted(&self) -> &str {
        &self.token_id_redacted
    }
}

impl fmt::Debug for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AuthToken(provider={:?}, issued={}, expires={}, token=[REDACTED])",
            self.provider_family, self.issued_at_epoch_secs, self.expires_at_epoch_secs
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthError {
    InvalidSecretReference,
    Expired,
    InvalidToken,
    ProviderRejected(String),
    NetworkUnavailable,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSecretReference => write!(f, "invalid secret reference"),
            Self::Expired => write!(f, "auth token expired"),
            Self::InvalidToken => write!(f, "invalid auth token"),
            Self::ProviderRejected(s) => write!(f, "provider rejected auth: {s}"),
            Self::NetworkUnavailable => write!(f, "network unavailable; live-smoke deferred"),
        }
    }
}

/// The auth modality an adapter serves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthMode {
    Api,
    Subscription,
}

/// `ProviderAuthPort` — the single port trait every provider auth adapter satisfies.
/// Implementations exchange a [`SecretReference`] for an [`AuthToken`].
///
/// ADR-0020 specifies ONE parameterised contract rather than one crate per
/// (provider, mode) pair. The parameters are the two associated consts: each
/// implementor declares its family and modality at compile time, and the
/// provided method bodies surface them at runtime for call sites that need a
/// value. This replaces six byte-identical 169-line kernel crates that differed
/// only in those two constants.
///
/// Not object-safe by construction — associated consts are the point. Nothing in
/// the tree uses `dyn ProviderAuthPort`, so compile-time parameterisation costs
/// nothing here and buys a static guarantee that a family/mode is always declared.
pub trait ProviderAuthPort {
    const PROVIDER_FAMILY: ProviderFamily;
    const AUTH_MODE: AuthMode;

    fn provider_family(&self) -> ProviderFamily {
        Self::PROVIDER_FAMILY
    }
    fn auth_mode(&self) -> AuthMode {
        Self::AUTH_MODE
    }
    fn authenticate(&self, sref: &SecretReference) -> Result<AuthToken, AuthError>;
    fn revoke(&self, token: &AuthToken) -> Result<(), AuthError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_family_aws() {
        assert_eq!(ProviderFamily::try_from("AWS"), Ok(ProviderFamily::Aws));
    }

    #[test]
    fn provider_family_claude() {
        assert_eq!(
            ProviderFamily::try_from("Claude"),
            Ok(ProviderFamily::Claude)
        );
    }

    #[test]
    fn provider_family_rejects_unknown() {
        assert!(ProviderFamily::try_from("Anthropic").is_err());
    }

    #[test]
    fn provider_family_error_carries_input() {
        let err = ProviderFamily::try_from("BadProvider").unwrap_err();
        assert!(err.0.contains("BadProvider"));
    }

    #[test]
    fn secret_ref_valid_sref_scheme() {
        assert!(SecretReference::new("sref://my-secret-id".to_owned()).is_ok());
    }

    #[test]
    fn secret_ref_rejects_non_sref_scheme() {
        assert!(SecretReference::new("http://my-secret".to_owned()).is_err());
    }

    #[test]
    fn secret_ref_rejects_bare_sref() {
        assert!(SecretReference::new("sref://".to_owned()).is_err());
    }

    #[test]
    fn secret_ref_debug_is_redacted() {
        let r = SecretReference::new("sref://very-secret-value".to_owned()).unwrap();
        let dbg = format!("{r:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("very-secret"));
    }

    // --- ProviderAuthPort, consolidated from six per-provider kernel crates ---

    #[test]
    fn auth_token_valid() {
        let t = AuthToken::new(100, 200, ProviderFamily::Claude, "tok-id-1".to_owned()).unwrap();
        assert_eq!(t.provider_family(), ProviderFamily::Claude);
        assert_eq!(t.issued_at_epoch_secs(), 100);
        assert_eq!(t.expires_at_epoch_secs(), 200);
    }

    #[test]
    fn auth_token_rejects_expired() {
        assert_eq!(
            AuthToken::new(200, 100, ProviderFamily::Claude, "x".to_owned()),
            Err(AuthError::Expired)
        );
    }

    #[test]
    fn auth_token_rejects_empty_id() {
        assert_eq!(
            AuthToken::new(100, 200, ProviderFamily::Claude, String::new()),
            Err(AuthError::InvalidToken)
        );
    }

    /// SECURITY INVARIANT carried over from the retired kernels: a token id must
    /// never be renderable. Losing this test in a refactor would silently permit
    /// secrets in logs.
    #[test]
    fn auth_token_debug_is_redacted() {
        let t = AuthToken::new(
            100,
            200,
            ProviderFamily::Claude,
            "very-secret-token-id".to_owned(),
        )
        .unwrap();
        let dbg = format!("{t:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("very-secret-token-id"));
    }

    /// Carried over from the retired kernels: each variant must be
    /// distinguishable in an operator-facing message.
    #[test]
    fn auth_error_display_distinct() {
        let messages: Vec<String> = vec![
            format!("{}", AuthError::InvalidSecretReference),
            format!("{}", AuthError::Expired),
            format!("{}", AuthError::InvalidToken),
            format!("{}", AuthError::ProviderRejected("boom".to_owned())),
            format!("{}", AuthError::NetworkUnavailable),
        ];
        let unique: std::collections::HashSet<_> = messages.iter().collect();
        assert_eq!(unique.len(), messages.len());
    }

    /// The consolidation invariant: ONE trait now expresses every
    /// (provider, mode) pair that previously required its own crate. Two
    /// implementors differing only in their associated consts prove the
    /// parameterisation actually carries the distinction.
    #[test]
    fn associated_consts_parameterise_family_and_mode() {
        struct ClaudeSub;
        struct OpenAiApi;

        fn stub(f: ProviderFamily) -> Result<AuthToken, AuthError> {
            AuthToken::new(0, 1, f, "stub".to_owned())
        }

        impl ProviderAuthPort for ClaudeSub {
            const PROVIDER_FAMILY: ProviderFamily = ProviderFamily::Claude;
            const AUTH_MODE: AuthMode = AuthMode::Subscription;
            fn authenticate(&self, _s: &SecretReference) -> Result<AuthToken, AuthError> {
                stub(Self::PROVIDER_FAMILY)
            }
            fn revoke(&self, _t: &AuthToken) -> Result<(), AuthError> {
                Ok(())
            }
        }

        impl ProviderAuthPort for OpenAiApi {
            const PROVIDER_FAMILY: ProviderFamily = ProviderFamily::OpenAiOrCodex;
            const AUTH_MODE: AuthMode = AuthMode::Api;
            fn authenticate(&self, _s: &SecretReference) -> Result<AuthToken, AuthError> {
                stub(Self::PROVIDER_FAMILY)
            }
            fn revoke(&self, _t: &AuthToken) -> Result<(), AuthError> {
                Ok(())
            }
        }

        assert_eq!(ClaudeSub.provider_family(), ProviderFamily::Claude);
        assert_eq!(ClaudeSub.auth_mode(), AuthMode::Subscription);
        assert_eq!(OpenAiApi.provider_family(), ProviderFamily::OpenAiOrCodex);
        assert_eq!(OpenAiApi.auth_mode(), AuthMode::Api);

        let sref = SecretReference::new("sref://x".to_owned()).unwrap();
        assert_eq!(
            ClaudeSub.authenticate(&sref).unwrap().provider_family(),
            ProviderFamily::Claude
        );
    }
}
