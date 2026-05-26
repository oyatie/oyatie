//! M02-P01 — Anthropic Subscription-auth kernel.
//! Port-trait crate: defines ProviderAuthPort that adapter implementations satisfy.
//! No raw secrets — SecretReference only.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_intelligence_account_kernel::{ProviderFamily, SecretReference};
use std::fmt;

/// AuthToken returned by an authenticated provider session.
/// SECURITY: inner bytes never exposed; Debug shows redacted; no Display.
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

/// AuthMode identifies the auth modality this kernel serves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthMode {
    Api,
    Subscription,
}

pub const AUTH_MODE: AuthMode = AuthMode::Subscription;
pub const PROVIDER_FAMILY: ProviderFamily = ProviderFamily::Claude;

/// ProviderAuthPort — port trait that adapter impls satisfy.
/// Implementations exchange a SecretReference for an AuthToken.
pub trait ProviderAuthPort {
    fn provider_family(&self) -> ProviderFamily {
        PROVIDER_FAMILY
    }
    fn auth_mode(&self) -> AuthMode {
        AUTH_MODE
    }
    fn authenticate(&self, sref: &SecretReference) -> Result<AuthToken, AuthError>;
    fn revoke(&self, token: &AuthToken) -> Result<(), AuthError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_mode_matches_crate_intent() {
        assert_eq!(AUTH_MODE, AuthMode::Subscription);
    }

    #[test]
    fn provider_family_matches_crate_intent() {
        assert_eq!(PROVIDER_FAMILY, ProviderFamily::Claude);
    }

    #[test]
    fn auth_token_valid() {
        let t = AuthToken::new(100, 200, PROVIDER_FAMILY, "tok-id-1".to_owned()).unwrap();
        assert_eq!(t.provider_family(), PROVIDER_FAMILY);
        assert_eq!(t.issued_at_epoch_secs(), 100);
        assert_eq!(t.expires_at_epoch_secs(), 200);
    }

    #[test]
    fn auth_token_rejects_expired() {
        assert_eq!(
            AuthToken::new(200, 100, PROVIDER_FAMILY, "x".to_owned()),
            Err(AuthError::Expired)
        );
    }

    #[test]
    fn auth_token_rejects_empty_id() {
        assert_eq!(
            AuthToken::new(100, 200, PROVIDER_FAMILY, String::new()),
            Err(AuthError::InvalidToken)
        );
    }

    #[test]
    fn auth_token_debug_is_redacted() {
        let t =
            AuthToken::new(100, 200, PROVIDER_FAMILY, "very-secret-token-id".to_owned()).unwrap();
        let dbg = format!("{t:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("very-secret-token-id"));
    }

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
}
