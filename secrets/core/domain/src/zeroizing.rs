// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! Memory-zeroing secret holder + vault-path value object.
//!
//! These types are the merge-variant landing of two contracts from
//! `.omc/plans/milestones/M02b-substrate/phases/P06-secrets/impl-plan.md`
//! into the existing `secrets-domain` crate (kept per
//! `F-M02B-PLAN-LIVE-CRATE-RECONCILIATION`). They are additive — sibling
//! types `SecretRef`, `SecretMaterial`, `SecretVersion`, `SecretLease`,
//! `SecretError`, `SecretVault` are unchanged.
//!
//! `ZeroizingSecret`: memory-zeroed-on-drop byte buffer for short-lived
//! in-memory secret values. Distinct from `SecretMaterial` (which carries
//! a `Classified<Vec<u8>>` data-class wrapper and a fingerprint and lives
//! in the persistent-version surface). Use `ZeroizingSecret` for transient
//! values during secret retrieval / rotation; use `SecretMaterial` for
//! persisted vault rows.
//!
//! `VaultPath`: validated OpenBao path. Construction rejects empty paths,
//! `..` traversal, and paths not rooted under `secret/`.

use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ZeroizingSecret(Vec<u8>);

impl ZeroizingSecret {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Debug for ZeroizingSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ZeroizingSecret([REDACTED])")
    }
}

impl std::fmt::Display for ZeroizingSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VaultPath(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultPathError {
    Empty,
    NotUnderSecretRoot,
    ContainsTraversal,
}

impl std::fmt::Display for VaultPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultPathError::Empty => write!(f, "vault path must not be empty"),
            VaultPathError::NotUnderSecretRoot => {
                write!(f, "vault path must start with 'secret/'")
            }
            VaultPathError::ContainsTraversal => {
                write!(f, "vault path must not contain '..' traversal")
            }
        }
    }
}

impl std::error::Error for VaultPathError {}

impl VaultPath {
    pub fn new(path: impl Into<String>) -> Result<Self, VaultPathError> {
        let p = path.into();
        if p.is_empty() {
            return Err(VaultPathError::Empty);
        }
        if !p.starts_with("secret/") {
            return Err(VaultPathError::NotUnderSecretRoot);
        }
        if p.contains("..") {
            return Err(VaultPathError::ContainsTraversal);
        }
        Ok(Self(p))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeroizing_secret_round_trips_bytes() {
        let secret = ZeroizingSecret::new(vec![1, 2, 3, 4]);
        assert_eq!(secret.as_bytes(), &[1, 2, 3, 4]);
        assert_eq!(secret.len(), 4);
        assert!(!secret.is_empty());
    }

    #[test]
    fn zeroizing_secret_empty_buffer_is_empty() {
        let secret = ZeroizingSecret::new(Vec::new());
        assert!(secret.is_empty());
        assert_eq!(secret.len(), 0);
    }

    #[test]
    fn zeroizing_secret_debug_redacts_contents() {
        let secret = ZeroizingSecret::new(b"super-secret-anthropic-api-key".to_vec());
        let rendered = format!("{secret:?}");
        assert_eq!(rendered, "ZeroizingSecret([REDACTED])");
        assert!(!rendered.contains("super-secret"));
        assert!(!rendered.contains("anthropic"));
    }

    #[test]
    fn zeroizing_secret_display_redacts_contents() {
        let secret = ZeroizingSecret::new(b"sk-aaaaaaaaaaaa".to_vec());
        let rendered = format!("{secret}");
        assert_eq!(rendered, "[REDACTED]");
        assert!(!rendered.contains("sk-"));
    }

    #[test]
    fn vault_path_accepts_canonical_openbao_shape() {
        let path = VaultPath::new("secret/data/t/00000000-0000-0000-0000-000000000001/db/primary")
            .expect("canonical OpenBao path must validate");
        assert_eq!(
            path.as_str(),
            "secret/data/t/00000000-0000-0000-0000-000000000001/db/primary"
        );
    }

    #[test]
    fn vault_path_rejects_empty_string() {
        assert_eq!(VaultPath::new(""), Err(VaultPathError::Empty));
    }

    #[test]
    fn vault_path_rejects_paths_outside_secret_root() {
        assert_eq!(
            VaultPath::new("kv/data/t/x"),
            Err(VaultPathError::NotUnderSecretRoot)
        );
    }

    #[test]
    fn vault_path_rejects_dot_dot_traversal() {
        assert_eq!(
            VaultPath::new("secret/data/t/../etc/passwd"),
            Err(VaultPathError::ContainsTraversal)
        );
    }

    #[test]
    fn vault_path_error_display_renders_each_variant() {
        assert_eq!(
            VaultPathError::Empty.to_string(),
            "vault path must not be empty"
        );
        assert_eq!(
            VaultPathError::NotUnderSecretRoot.to_string(),
            "vault path must start with 'secret/'"
        );
        assert_eq!(
            VaultPathError::ContainsTraversal.to_string(),
            "vault path must not contain '..' traversal"
        );
    }
}
