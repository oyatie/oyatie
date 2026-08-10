//! Signing backend abstraction.
//!
//! Real Talos delegates certificate signing to a private key held by the CA
//! (ECDSA/Ed25519 via crypto/x509). The [`SigningBackend`] trait models that
//! *boundary*. Two implementations live here:
//!
//! * [`EcdsaP256Signer`] — the REAL crypto backend (G002 slice-1b-i; ADR-0561
//!   D5 promotion, ADR-0506): an ECDSA P-256 private key minted by `rcgen` on
//!   the `aws-lc-rs` backend (NO ring). `sign` produces a real ASN.1/DER ECDSA
//!   signature over the to-be-signed bytes; `verify` checks it with AWS-LC
//!   against the signer's public point. This is what production issuance uses,
//!   and what makes the issued leaves carry real DER + real signatures.
//! * `InMemorySigner` — a deterministic keyed-hash MAC retained for the
//!   shape-model unit tests (it has no host-entropy / external-crate needs). It
//!   is NOT a real signature and never produces a real DER leaf. It is behind
//!   the non-default `modeled-crypto` feature, so a production build cannot
//!   link it (unlinked here rather than intra-doc-linked, since the item does
//!   not exist off-feature).
//!
//! The trait shape (`sign`/`verify`/`key_id`) is already asymmetric-compatible,
//! so swapping `InMemorySigner` for `EcdsaP256Signer` does not change the CA,
//! the `TrustBundle`, the `SecurityService`, or any kernel port — the seam holds
//! exactly as ADR-0561 D4 (signer cutover) requires.

use std::sync::Arc;

use aws_lc_rs::signature::{ECDSA_P256_SHA256_ASN1, UnparsedPublicKey};
use rcgen::{KeyPair, PKCS_ECDSA_P256_SHA256, PublicKeyData, SigningKey};

use crate::error::{Result, TrustError};
use crate::x509::hex_encode;

/// Length of an uncompressed SEC1 P-256 public point: `0x04 || X(32) || Y(32)`.
const P256_UNCOMPRESSED_POINT_LEN: usize = 65;

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
///
/// This is a *modeled* signing backend, not a weak real one: the "signature" is
/// 8 bytes of FNV-1a, and [`InMemorySigner::from_seed`] makes the private key
/// literally equal the seed bytes — so anyone who knows the seed string forges
/// any signature this backend accepts. It satisfies the same [`SigningBackend`]
/// bound [`crate::ca::CertificateAuthority::bootstrap`] takes, so an un-gated
/// copy lets a production build stand up a CA issuing forgeable certificates.
/// Hence the non-default `modeled-crypto` feature: no production target enables
/// it, so production cannot link this type. Production signs with
/// [`EcdsaP256Signer`].
// The gate sits below the `derive` so it is textually adjacent to the item it
// guards, which is what `token::tests` asserts.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(test, feature = "modeled-crypto"))]
pub struct InMemorySigner {
    private_key: Vec<u8>,
}

#[cfg(any(test, feature = "modeled-crypto"))]
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

#[cfg(any(test, feature = "modeled-crypto"))]
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

/// The REAL ECDSA P-256 signing backend (G002 slice-1b-i).
///
/// Wraps an `rcgen` [`KeyPair`] generated on the `aws-lc-rs` backend (ADR-0506:
/// AWS-LC, NO ring). The private key signs the to-be-signed bytes producing a
/// real ASN.1/DER ECDSA-with-SHA-256 signature; verification recovers the
/// uncompressed SEC1 public point from the key's SubjectPublicKeyInfo and checks
/// the signature with AWS-LC. The key is held behind an [`Arc`] so the same
/// signer can be `Clone`d into the CA, the [`crate::bundle::TrustBundle`], and a
/// verification anchor without re-serialising the key material.
///
/// Issuance of the real X.509 DER leaf (the artifact a peer presents) is driven
/// by [`crate::der`] using the SAME `rcgen` key this signer holds, so the leaf's
/// signature and this backend's `sign`/`verify` are produced by one private key.
#[derive(Clone)]
pub struct EcdsaP256Signer {
    key: Arc<KeyPair>,
    /// Cached SubjectPublicKeyInfo DER of the public half (for `key_id` + verify).
    spki_der: Arc<Vec<u8>>,
}

impl core::fmt::Debug for EcdsaP256Signer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Never render private key material; surface only the stable public id.
        f.debug_struct("EcdsaP256Signer")
            .field("key_id", &self.key_id())
            .finish()
    }
}

impl EcdsaP256Signer {
    /// Generate a fresh random ECDSA P-256 signer (AWS-LC entropy).
    ///
    /// # Errors
    /// [`TrustError`] if key generation fails (entropy/backend error).
    pub fn generate() -> Result<Self> {
        let key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256)
            .map_err(|e| TrustError::Other(format!("ECDSA P-256 keygen failed: {e}")))?;
        Ok(Self::from_key_pair(key))
    }

    /// Reconstruct a signer from a PKCS#8 private-key DER previously produced by
    /// [`EcdsaP256Signer::private_key_der`]. Lets a CA persist + reload its key
    /// across the `SigningBackend` seam without changing the trait.
    ///
    /// # Errors
    /// [`TrustError`] if the DER is not a valid ECDSA P-256 PKCS#8 key.
    pub fn from_pkcs8_der(pkcs8_der: &[u8]) -> Result<Self> {
        let key = KeyPair::try_from(pkcs8_der)
            .map_err(|e| TrustError::Other(format!("invalid ECDSA P-256 PKCS#8 key: {e}")))?;
        Ok(Self::from_key_pair(key))
    }

    fn from_key_pair(key: KeyPair) -> Self {
        let spki_der = key.subject_public_key_info();
        Self {
            key: Arc::new(key),
            spki_der: Arc::new(spki_der),
        }
    }

    /// The PKCS#8 DER of the private key, so the CA can persist its signing key.
    #[must_use]
    pub fn private_key_der(&self) -> Vec<u8> {
        self.key.serialize_der()
    }

    /// The SubjectPublicKeyInfo DER of the public half (real SPKI, the value a
    /// certificate embeds as its `public_key_der`).
    #[must_use]
    pub fn public_key_spki_der(&self) -> Vec<u8> {
        self.spki_der.as_ref().clone()
    }

    /// Borrow the underlying `rcgen` key pair (used by [`crate::der`] issuance to
    /// sign the real certificate with the same key as this backend).
    #[must_use]
    pub fn key_pair(&self) -> &KeyPair {
        &self.key
    }

    /// The uncompressed SEC1 public point (`0x04 || X || Y`) carried in the tail
    /// of the SubjectPublicKeyInfo. For P-256 the SPKI ends with the 65-byte
    /// uncompressed point; returns `None` if the SPKI is too short to contain it.
    fn public_point(&self) -> Option<&[u8]> {
        let spki = self.spki_der.as_ref();
        spki.len()
            .checked_sub(P256_UNCOMPRESSED_POINT_LEN)
            .map(|start| &spki[start..])
            .filter(|point| point.first() == Some(&0x04))
    }
}

impl SigningBackend for EcdsaP256Signer {
    fn sign(&self, tbs: &[u8]) -> Vec<u8> {
        // rcgen's SigningKey::sign produces a real ASN.1/DER ECDSA-with-SHA-256
        // signature. The trait is infallible by shape; a backend signing error
        // (never expected for an in-memory AWS-LC key) yields an empty signature,
        // which `Certificate::validate` rejects as unsigned — fail-closed, never a
        // panic in production code (ADR-0083 Tier-3).
        self.key.sign(tbs).unwrap_or_default()
    }

    fn verify(&self, tbs: &[u8], signature: &[u8]) -> bool {
        let Some(point) = self.public_point() else {
            return false;
        };
        UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, point)
            .verify(tbs, signature)
            .is_ok()
    }

    fn key_id(&self) -> String {
        // A stable public id: the FNV-1a digest of the real SubjectPublicKeyInfo.
        // One-way and derived only from the public half, so it never leaks the
        // private key but is identical across clones of the same signer.
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in self.spki_der.as_ref() {
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

    // ---- EcdsaP256Signer: REAL crypto (G002 slice-1b-i) --------------------

    #[test]
    fn ecdsa_sign_then_verify_round_trips() {
        let s = EcdsaP256Signer::generate().unwrap();
        let sig = s.sign(b"hello-real-crypto");
        // A real ECDSA/DER signature is not the 8-byte MAC of the shape model.
        assert!(sig.len() > 8);
        assert!(s.verify(b"hello-real-crypto", &sig));
    }

    #[test]
    fn ecdsa_wrong_key_does_not_verify() {
        let signer = EcdsaP256Signer::generate().unwrap();
        let attacker = EcdsaP256Signer::generate().unwrap();
        let sig = signer.sign(b"payload");
        // A different real key cannot verify the signature.
        assert!(!attacker.verify(b"payload", &sig));
    }

    #[test]
    fn ecdsa_tampered_message_fails() {
        let s = EcdsaP256Signer::generate().unwrap();
        let sig = s.sign(b"payload");
        assert!(!s.verify(b"PAYLOAD", &sig));
        assert!(verify_or_err(&s, b"PAYLOAD", &sig).is_err());
    }

    #[test]
    fn ecdsa_key_id_stable_across_clone_and_reload() {
        let s = EcdsaP256Signer::generate().unwrap();
        // A clone shares the key (Arc) and yields the identical public id.
        assert_eq!(s.key_id(), s.clone().key_id());
        // Reloading from the PKCS#8 DER reproduces the same public id.
        let reloaded = EcdsaP256Signer::from_pkcs8_der(&s.private_key_der()).unwrap();
        assert_eq!(s.key_id(), reloaded.key_id());
        // and the reloaded signer verifies signatures from the original.
        let sig = s.sign(b"x");
        assert!(reloaded.verify(b"x", &sig));
    }

    #[test]
    fn ecdsa_two_keys_have_distinct_ids() {
        let a = EcdsaP256Signer::generate().unwrap();
        let b = EcdsaP256Signer::generate().unwrap();
        assert_ne!(a.key_id(), b.key_id());
    }

    #[test]
    fn ecdsa_public_spki_is_real_and_nonempty() {
        let s = EcdsaP256Signer::generate().unwrap();
        let spki = s.public_key_spki_der();
        // A real P-256 SPKI is ~91 bytes and ends with the 65-byte point.
        assert!(spki.len() >= P256_UNCOMPRESSED_POINT_LEN);
        assert_eq!(spki[spki.len() - P256_UNCOMPRESSED_POINT_LEN], 0x04);
    }
}
