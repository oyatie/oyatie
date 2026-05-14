//! M02-P01 — gemini subscription-auth adapter.
//! In-memory mock impl of ProviderAuthPort. Live-network deferred behind feature flag.
//! No raw secrets — SecretReference only; AuthToken Debug stays redacted.

use oya_foundry_account_domain::SecretReference;
use oya_foundry_adapter_gemini_subscription_kernel::{
    AuthError, AuthToken, PROVIDER_FAMILY, ProviderAuthPort,
};
use std::cell::RefCell;
use std::collections::HashSet;

#[derive(Default)]
pub struct GeminiSubscriptionAdapter {
    revoked_token_ids: RefCell<HashSet<String>>,
    clock_epoch_secs: u64,
    token_lifetime_secs: u64,
}

impl GeminiSubscriptionAdapter {
    pub fn new() -> Self {
        Self {
            revoked_token_ids: RefCell::new(HashSet::new()),
            clock_epoch_secs: 1_000_000,
            token_lifetime_secs: 3600,
        }
    }

    pub fn with_clock(mut self, now_epoch_secs: u64, lifetime_secs: u64) -> Self {
        self.clock_epoch_secs = now_epoch_secs;
        self.token_lifetime_secs = lifetime_secs;
        self
    }

    fn synthesize_token_id(&self, sref: &SecretReference) -> String {
        // Use the Debug repr (already redacted) to derive a stable mock-token-id
        // without exposing the secret. Live impls would call the provider API.
        let dbg = format!("{sref:?}");
        format!("mock-gemini-subscription-{}", dbg.len())
    }
}

impl ProviderAuthPort for GeminiSubscriptionAdapter {
    fn authenticate(&self, sref: &SecretReference) -> Result<AuthToken, AuthError> {
        let token_id = self.synthesize_token_id(sref);
        if self.revoked_token_ids.borrow().contains(&token_id) {
            return Err(AuthError::ProviderRejected(
                "token previously revoked".to_owned(),
            ));
        }
        AuthToken::new(
            self.clock_epoch_secs,
            self.clock_epoch_secs + self.token_lifetime_secs,
            PROVIDER_FAMILY,
            token_id,
        )
    }

    fn revoke(&self, token: &AuthToken) -> Result<(), AuthError> {
        self.revoked_token_ids
            .borrow_mut()
            .insert(token.token_id_redacted().to_owned());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_foundry_account_domain::{ProviderFamily, SecretReference};
    use oya_foundry_adapter_gemini_subscription_kernel::{AUTH_MODE, AuthMode};

    fn sref(s: &str) -> SecretReference {
        SecretReference::new(s.to_owned()).unwrap()
    }

    #[test]
    fn adapter_authenticates_and_returns_token() {
        let a = GeminiSubscriptionAdapter::new();
        let t = a.authenticate(&sref("sref://test-key")).unwrap();
        assert_eq!(t.provider_family(), ProviderFamily::Gemini);
        assert!(t.expires_at_epoch_secs() > t.issued_at_epoch_secs());
    }

    #[test]
    fn adapter_reports_correct_mode_and_family() {
        let a = GeminiSubscriptionAdapter::new();
        assert_eq!(a.auth_mode(), AuthMode::Subscription);
        assert_eq!(a.provider_family(), ProviderFamily::Gemini);
        assert_eq!(AUTH_MODE, AuthMode::Subscription);
    }

    #[test]
    fn adapter_token_debug_redacted() {
        let a = GeminiSubscriptionAdapter::new();
        let t = a.authenticate(&sref("sref://super-secret")).unwrap();
        let dbg = format!("{t:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("super-secret"));
    }

    #[test]
    fn adapter_revoke_blocks_reuse() {
        let a = GeminiSubscriptionAdapter::new();
        let t = a.authenticate(&sref("sref://test-key")).unwrap();
        a.revoke(&t).unwrap();
        // Re-authenticate with same sref should be rejected because synthesized id matches.
        assert!(matches!(
            a.authenticate(&sref("sref://test-key")),
            Err(AuthError::ProviderRejected(_))
        ));
    }

    #[test]
    fn adapter_clock_override_respected() {
        let a = GeminiSubscriptionAdapter::new().with_clock(5_000, 60);
        let t = a.authenticate(&sref("sref://k")).unwrap();
        assert_eq!(t.issued_at_epoch_secs(), 5_000);
        assert_eq!(t.expires_at_epoch_secs(), 5_060);
    }

    #[test]
    fn adapter_does_not_leak_secret_in_token_id() {
        let a = GeminiSubscriptionAdapter::new();
        let t = a
            .authenticate(&sref("sref://very-private-key-material-XYZ"))
            .unwrap();
        assert!(!t.token_id_redacted().contains("very-private"));
        assert!(!t.token_id_redacted().contains("XYZ"));
    }
}
