//! Signing backend abstraction.
//!
//! Real Talos delegates certificate signing to a private key held by the CA
//! (ECDSA/Ed25519 via crypto/x509). Performing actual asymmetric cryptography
//! would require external crates and host entropy, so we model the *boundary* as
//! a trait. Tests use [`InMemorySigner`], a deterministic keyed-hash MAC that
//! behaves like a signature for the purpose of issuer/leaf verification: only a
//! signer holding the matching key can produce a signature that verifies.

use crate::error::{Result, TrustError};
use crate::x509::hex_encode;

/// A pluggable signing backend. A real implementation wraps an ECDSA/Ed25519
/// private key; the in-memory implementation uses a keyed hash.
pub trait SigningBackend {
    /// Produce a signature over `tbs` (the to-be-signed bytes).
    fn sign(&self, tbs: &[u8]) -> Vec<u8>;

    /// Verify that `signature` is a valid signature over `tbs` produced by the
    /// key paired with this backend's public identity.
    fn verify(&self, tbs: &[u8], signature: &[u8]) -> bool;

    /// A stable identifier of the public half of this signer, used to bind a
    /// certificate to the CA that signed it.
    fn key_id(&self) -> String;
}

/// Deterministic in-memory signer: an FNV-1a keyed hash over the private key
/// concatenated with the message. Only a signer constructed from the same
/// private key produces a signature that verifies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemorySigner {
    private_key: Vec<u8>,
}

impl InMemorySigner {
    /// Construct from raw private key bytes.
    pub fn new(private_key: impl Into<Vec<u8>>) -> Self {
        InMemorySigner {
            private_key: private_key.into(),
        }
    }

    /// Derive a signer deterministically from a textual seed.
    pub fn from_seed(seed: &str) -> Self {
        InMemorySigner {
            private_key: seed.as_bytes().to_vec(),
        }
    }

    /// Raw private key material for the sealed CA persistence boundary.
    pub(crate) fn private_key_material(&self) -> &[u8] {
        &self.private_key
    }

    fn mac(&self, tbs: &[u8]) -> [u8; 8] {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in self
            .private_key
            .iter()
            .chain(b"\x00sig\x00")
            .chain(tbs.iter())
        {
            hash ^= u64::from(b);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        hash.to_be_bytes()
    }
}

impl SigningBackend for InMemorySigner {
    fn sign(&self, tbs: &[u8]) -> Vec<u8> {
        self.mac(tbs).to_vec()
    }

    fn verify(&self, tbs: &[u8], signature: &[u8]) -> bool {
        // Constant-shape comparison (length-checked) against the recomputed MAC.
        let expected = self.mac(tbs);
        signature.len() == expected.len()
            && signature.iter().zip(expected.iter()).all(|(a, b)| a == b)
    }

    fn key_id(&self) -> String {
        // Public id derived from the private key (one-way), so it never leaks
        // the key but is stable per signer.
        let mut hash: u64 = 0x8422_2325_cbf2_9ce4;
        for &b in &self.private_key {
            hash ^= u64::from(b);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        hex_encode(&hash.to_be_bytes())
    }
}

/// Verify a signature, returning a descriptive error rather than a bool. Used by
/// higher layers that want to propagate a [`TrustError`].
pub fn verify_or_err(backend: &dyn SigningBackend, tbs: &[u8], signature: &[u8]) -> Result<()> {
    if backend.verify(tbs, signature) {
        Ok(())
    } else {
        Err(TrustError::verification_failed(
            "signature does not verify against issuer key",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_round_trips() {
        let s = InMemorySigner::from_seed("ca-key");
        let sig = s.sign(b"hello");
        assert!(s.verify(b"hello", &sig));
    }

    #[test]
    fn wrong_key_does_not_verify() {
        let signer = InMemorySigner::from_seed("ca-key");
        let attacker = InMemorySigner::from_seed("other-key");
        let sig = signer.sign(b"payload");
        assert!(!attacker.verify(b"payload", &sig));
    }

    #[test]
    fn tampered_message_fails() {
        let s = InMemorySigner::from_seed("ca-key");
        let sig = s.sign(b"payload");
        assert!(!s.verify(b"PAYLOAD", &sig));
        assert!(verify_or_err(&s, b"PAYLOAD", &sig).is_err());
    }

    #[test]
    fn key_id_is_stable_and_hides_key() {
        let s = InMemorySigner::from_seed("ca-key");
        assert_eq!(s.key_id(), InMemorySigner::from_seed("ca-key").key_id());
        assert_ne!(s.key_id(), InMemorySigner::from_seed("ca-key2").key_id());
    }
}
