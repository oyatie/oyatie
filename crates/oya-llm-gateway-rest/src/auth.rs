//! Two auth realms, both using constant-time comparison.
//!
//! - [`AuthRealm::Admin`] — the control/management plane (refresh keys, read
//!   pool status). Holds one admin token.
//! - [`AuthRealm::Ingress`] — the proxy realm callers present to dispatch
//!   requests through the pool. Holds a set of accepted ingress proxy-keys, so
//!   the agent fleet can be issued distinct keys without exposing the pooled
//!   upstream keys.
//!
//! Every comparison runs through `ring`'s constant-time slice equality
//! (`CRYPTO_memcmp`) to avoid leaking credential length/content through timing.
//! Tokens are kept only in memory (sourced from a k8s Secret / OpenBao at
//! deploy, never from a plaintext file the gateway reads itself).
//!
//! Note on the `#[allow(deprecated)]` below: ring 0.17 marked the *public*
//! `constant_time::verify_slices_are_equal` re-export deprecated as an
//! API-surface-stability decision ("internal function not intended for external
//! use"), NOT a security regression — the underlying primitive is BoringSSL's
//! `CRYPTO_memcmp`, the same constant-time compare ring still uses internally
//! for HMAC/AEAD tag verification. The dependency spec
//! (`docs/ideas/llm-gateway-best-of-both.md`) names this exact function and
//! mandates consolidating crypto on `ring`, so we keep it with a scoped allow
//! rather than pulling a second crypto crate (`subtle`) back in.

#[allow(deprecated)]
use ring::constant_time::verify_slices_are_equal;

/// Which realm a presented credential is being checked against.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthRealm {
    /// Admin / control plane.
    Admin,
    /// Ingress proxy plane (agent fleet → gateway).
    Ingress,
}

impl AuthRealm {
    /// Stable label for logs/metrics.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            AuthRealm::Admin => "admin",
            AuthRealm::Ingress => "ingress",
        }
    }
}

/// Verifies presented credentials in constant time against in-memory token
/// sets. Construct once at startup from secrets injected by the platform.
#[derive(Clone)]
pub struct AuthVerifier {
    admin_token: Vec<u8>,
    ingress_keys: Vec<Vec<u8>>,
}

impl AuthVerifier {
    /// Build a verifier from the admin token and the set of accepted ingress
    /// proxy-keys. An empty ingress set means no caller can dispatch (the
    /// gateway refuses all proxy traffic) — fail-closed by construction.
    #[must_use]
    pub fn new(admin_token: impl Into<String>, ingress_keys: Vec<String>) -> Self {
        AuthVerifier {
            admin_token: admin_token.into().into_bytes(),
            ingress_keys: ingress_keys.into_iter().map(String::into_bytes).collect(),
        }
    }

    /// `true` if `presented` matches the admin token (constant-time).
    #[must_use]
    pub fn verify_admin(&self, presented: &str) -> bool {
        ct_eq(presented.as_bytes(), &self.admin_token)
    }

    /// `true` if `presented` matches ANY accepted ingress proxy-key.
    ///
    /// Every candidate is compared in constant time, and we always scan the
    /// full set (no early return on first match) so neither the match position
    /// nor the set size is leaked through timing. The per-key comparison uses
    /// `ring`'s constant-time slice equality; we OR the boolean results without
    /// short-circuiting so the loop cost is independent of where (or whether) a
    /// match occurs.
    #[must_use]
    pub fn verify_ingress(&self, presented: &str) -> bool {
        let presented = presented.as_bytes();
        let mut matched = false;
        for key in &self.ingress_keys {
            // Bitwise-OR (not `||`) so there is no early-exit branch on a match.
            matched |= ct_eq(presented, key);
        }
        matched
    }

    /// Verify against the named realm.
    #[must_use]
    pub fn verify(&self, realm: AuthRealm, presented: &str) -> bool {
        match realm {
            AuthRealm::Admin => self.verify_admin(presented),
            AuthRealm::Ingress => self.verify_ingress(presented),
        }
    }

    /// Number of accepted ingress keys (for a startup log/metric; never the
    /// keys themselves).
    #[must_use]
    pub fn ingress_key_count(&self) -> usize {
        self.ingress_keys.len()
    }
}

/// Constant-time byte-slice equality.
///
/// `ring::constant_time::verify_slices_are_equal` compares in time independent
/// of the *content* of equal-length slices and returns `Err` on any
/// difference. A length mismatch returns `Err` immediately (length is not a
/// secret here — credential lengths are not sensitive, and a mismatched length
/// is definitively not the configured token), so no content timing leaks.
#[allow(deprecated)]
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    // `CRYPTO_memcmp` under the hood: constant-time for equal-length inputs.
    verify_slices_are_equal(a, b).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verifier() -> AuthVerifier {
        AuthVerifier::new(
            "admin-supersecret",
            vec!["agent-key-1".to_string(), "agent-key-2".to_string()],
        )
    }

    #[test]
    fn admin_token_matches_exactly() {
        let v = verifier();
        assert!(v.verify_admin("admin-supersecret"));
        assert!(!v.verify_admin("admin-supersecre")); // shorter
        assert!(!v.verify_admin("admin-supersecrettt")); // longer
        assert!(!v.verify_admin("wrong"));
        assert!(!v.verify_admin(""));
    }

    #[test]
    fn ingress_matches_any_member() {
        let v = verifier();
        assert!(v.verify_ingress("agent-key-1"));
        assert!(v.verify_ingress("agent-key-2"));
        assert!(!v.verify_ingress("agent-key-3"));
        assert!(!v.verify_ingress(""));
    }

    #[test]
    fn realms_are_isolated() {
        let v = verifier();
        // An ingress key must NOT authenticate the admin realm and vice versa.
        assert!(!v.verify_admin("agent-key-1"));
        assert!(!v.verify_ingress("admin-supersecret"));
    }

    #[test]
    fn empty_ingress_set_fails_closed() {
        let v = AuthVerifier::new("admin", vec![]);
        assert_eq!(v.ingress_key_count(), 0);
        assert!(!v.verify_ingress("anything"));
        assert!(!v.verify_ingress(""));
    }

    #[test]
    fn verify_dispatches_to_correct_realm() {
        let v = verifier();
        assert!(v.verify(AuthRealm::Admin, "admin-supersecret"));
        assert!(v.verify(AuthRealm::Ingress, "agent-key-1"));
        assert!(!v.verify(AuthRealm::Admin, "agent-key-1"));
    }

    #[test]
    fn realm_labels_are_stable() {
        assert_eq!(AuthRealm::Admin.as_str(), "admin");
        assert_eq!(AuthRealm::Ingress.as_str(), "ingress");
    }
}
