//! aws-lc-rs adapter for the audit digest-chain ports — ADR-0506
//! canonical crypto backend, transitional custody per ADR-0510.
//!
//! Implements [`Digester`] with SHA-256 (CloudTrail digest-file algorithm)
//! and [`ChainSigner`]/[`ChainVerifier`] with Ed25519 (RFC 8032,
//! deterministic signing — no ambient RNG at sign time). Key custody here
//! is in-process and transitional; the W5 destination moves signing behind
//! the owned KMS interface (ADR-0536 D-5) without changing the ports.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use aws_lc_rs::digest::{SHA256, digest};
use aws_lc_rs::signature::{ED25519, Ed25519KeyPair, KeyPair, UnparsedPublicKey};
use oya_shared_audit_event_kernel::{
    ChainSigner, ChainVerifier, DigestChainError, Digester, decode_hex, encode_hex,
};

/// Self-describing digest prefix (house convention `sha256:<hex>`).
pub const SHA256_DIGEST_PREFIX: &str = "sha256:";

/// SHA-256 [`Digester`] backed by aws-lc-rs.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sha256Digester;

impl Digester for Sha256Digester {
    fn algorithm(&self) -> &'static str {
        "sha256"
    }

    fn digest_hex(&self, bytes: &[u8]) -> String {
        let d = digest(&SHA256, bytes);
        format!("{SHA256_DIGEST_PREFIX}{}", encode_hex(d.as_ref()))
    }
}

/// Ed25519 [`ChainSigner`] holding an in-process key pair (transitional
/// custody — see crate docs).
pub struct Ed25519ChainSigner {
    key_pair: Ed25519KeyPair, // data_class: SECRET
    key_id: String,           // data_class: INTERNAL_ONLY
}

impl Ed25519ChainSigner {
    /// Generate a fresh signing key under `key_id`.
    ///
    /// # Errors
    /// [`DigestChainError::SigningFailed`] when key generation fails.
    pub fn generate(key_id: impl Into<String>) -> Result<Self, DigestChainError> {
        let key_pair = Ed25519KeyPair::generate()
            .map_err(|e| DigestChainError::SigningFailed(format!("ed25519 keygen: {e}")))?;
        Ok(Self {
            key_pair,
            key_id: key_id.into(),
        })
    }

    /// Raw 32-byte Edwards public key for registration with a verifier.
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.key_pair.public_key().as_ref().to_vec()
    }
}

impl ChainSigner for Ed25519ChainSigner {
    fn key_id(&self) -> &str {
        &self.key_id
    }

    fn sign_hex(&self, message: &[u8]) -> Result<String, DigestChainError> {
        // Ed25519 signing is deterministic per RFC 8032 — no RNG input.
        Ok(encode_hex(self.key_pair.sign(message).as_ref()))
    }
}

/// Ed25519 [`ChainVerifier`] over a key_id → public-key registry.
#[derive(Clone, Debug, Default)]
pub struct Ed25519ChainVerifier {
    keys: BTreeMap<String, Vec<u8>>, // data_class: PUBLIC
}

impl Ed25519ChainVerifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `public_key` (raw 32-byte Edwards point) under `key_id`.
    #[must_use]
    pub fn with_key(mut self, key_id: impl Into<String>, public_key: Vec<u8>) -> Self {
        self.keys.insert(key_id.into(), public_key);
        self
    }
}

impl ChainVerifier for Ed25519ChainVerifier {
    fn verify(
        &self,
        key_id: &str,
        message: &[u8],
        signature_hex: &str,
    ) -> Result<(), DigestChainError> {
        let public_key = self
            .keys
            .get(key_id)
            .ok_or_else(|| DigestChainError::UnknownKeyId(key_id.to_owned()))?;
        let signature = decode_hex(signature_hex)?;
        UnparsedPublicKey::new(&ED25519, public_key)
            .verify(message, &signature)
            .map_err(|_| DigestChainError::SignatureInvalid { sequence: 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_digest_is_self_describing_and_stable() {
        let d = Sha256Digester.digest_hex(b"abc");
        // NIST FIPS 180-2 test vector for "abc".
        assert_eq!(
            d,
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sign_verify_round_trip() {
        let signer = Ed25519ChainSigner::generate("key-1").unwrap();
        let verifier = Ed25519ChainVerifier::new().with_key("key-1", signer.public_key_bytes());
        let sig = signer.sign_hex(b"message").unwrap();
        verifier.verify("key-1", b"message", &sig).unwrap();
    }

    #[test]
    fn tampered_message_fails_verification() {
        let signer = Ed25519ChainSigner::generate("key-1").unwrap();
        let verifier = Ed25519ChainVerifier::new().with_key("key-1", signer.public_key_bytes());
        let sig = signer.sign_hex(b"message").unwrap();
        assert!(matches!(
            verifier.verify("key-1", b"forged", &sig),
            Err(DigestChainError::SignatureInvalid { .. })
        ));
    }

    #[test]
    fn unknown_key_id_is_rejected() {
        let verifier = Ed25519ChainVerifier::new();
        assert_eq!(
            verifier.verify("nope", b"m", "00"),
            Err(DigestChainError::UnknownKeyId("nope".into()))
        );
    }
}
