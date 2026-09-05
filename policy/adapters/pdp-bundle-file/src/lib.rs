//! # iam-pdp-bundle-file-adapter
//!
//! File-backed [`PolicyBundleStore`] adapter (ADR-0559, G004 slice 1) with
//! cryptographic signature verification at the store boundary (ADR-0536 D-2,
//! G004 bundle-signing slice).
//!
//! ## Posture
//! The policy-bundle transport is a declarative JSON document on a mounted
//! path — a ConfigMap mount in K8s, a plain file in tests. The document is a
//! SIGNED ENVELOPE ([`SignedPolicyBundleDoc`]): a detached Ed25519 signature
//! set over the EXACT stored inner [`PolicyBundle`] bytes. On load the adapter:
//!
//! 1. parses the outer envelope (CLOSED schema, `deny_unknown_fields`);
//! 2. verifies the envelope against the TRUSTED public-key set loaded from the
//!    trust-anchor directory — a bundle verifies iff ANY trusted key whose
//!    `key_id` matches a signature validates the stored inner bytes (key
//!    rotation = several trusted keys). No trusted signature ⇒
//!    [`BundleStoreError::SignatureRejected`] and the inner bundle is NEVER
//!    parsed past this gate;
//! 3. only on success, parses the verified inner bytes as a [`PolicyBundle`]
//!    (CLOSED schema) and re-validates the version token (serde's
//!    `transparent` `PolicyVersion` bypasses constructor validation, so the
//!    adapter re-runs it — INSIDE the verified region, so a malformed version
//!    token in a signed bundle is still rejected).
//!
//! Signing over the exact embedded bytes (sign==verify by construction) avoids
//! any serialize-then-compare canonicalization trap. The trust anchor is
//! fail-closed: an absent/empty trust directory, or one carrying no usable
//! keys, is a boot refusal (a PDP that cannot prove which keys to trust must
//! never serve a decision — the mTLS trust-root precedent).
//!
//! Every error is fail-closed ([`BundleStoreError`]): at boot the service
//! REFUSES TO START (the identity precedent — a serving process is a
//! correctly-configured process), and on reload the serving bundle keeps
//! serving.
//!
//! This adapter is deliberately throwaway (ADR-0550): the destination is the
//! policy-bundle CRD + operator distribution fabric. The envelope + trusted-key
//! verification contract is the SAME at cutover (only the transport changes);
//! the trait does not change. PRODUCTION signing-key custody (KMS/owned-KMS,
//! ADR-0536 D-5) is a separate founder-gated slice — this adapter verifies
//! against configured trusted PUBLIC keys; it never holds a private key.
//!
//! The OWNED aws-lc-rs Ed25519 verifier ([`Ed25519ChainVerifier`], ADR-0506,
//! ring-free) is reused from the audit digest-chain adapter — the first legal
//! `iam/ → libs/shared-audit-*` lib→lib edge (no layer inversion).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use iam_pdp_kernel::{BundleStoreError, PolicyBundleStore};
use serde::{Deserialize, Serialize};
use shared_audit_digest_adapter_awslc::Ed25519ChainVerifier;
use shared_audit_event_kernel::{ChainVerifier, DigestChainError, decode_hex};
use shared_pdp_kernel::PolicyBundle;
use shared_platform_contracts_kernel::pdp::PolicyVersion;

mod publication;
pub use publication::BundlePublishError;

#[cfg(test)]
mod publication_tests;
#[cfg(test)]
mod reader_test_support;
#[cfg(test)]
mod reader_tests;

/// One detached signature over a signed policy-bundle envelope's inner bytes.
///
/// `public_key_hex` is the signer's raw 32-byte Ed25519 Edwards public key
/// (lowercase hex), carried for diagnostics and self-description; it is NOT
/// trusted on the verify path — only the trusted-key set loaded from the trust
/// anchor is consulted (an attacker who embeds their own public key still
/// cannot forge a trusted `key_id`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleSignature {
    /// Stable key identifier; must match a trusted key for the signature to be
    /// considered (rotation = several trusted key_ids).
    pub key_id: String, // data_class: INTERNAL_ONLY
    /// The signer's raw Ed25519 public key (lowercase hex) — self-describing,
    /// never trusted on the verify path.
    pub public_key_hex: String, // data_class: PUBLIC
    /// Ed25519 signature (lowercase hex) over the EXACT inner `bundle` bytes.
    pub signature_hex: String, // data_class: INTERNAL_ONLY
}

/// The signed outer envelope around a serialized [`PolicyBundle`]. The
/// `bundle` field holds the EXACT inner bytes the signatures cover (verbatim,
/// not a re-serialization), so verify operates on the same bytes that are then
/// parsed — sign==verify by construction, no canonicalization trap. CLOSED
/// schema: unknown envelope fields are rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedPolicyBundleDoc {
    /// The exact serialized inner [`PolicyBundle`] bytes the signatures cover.
    pub bundle: String, // data_class: INTERNAL_ONLY
    /// At least one detached signature; an empty set is fail-closed-rejected.
    pub signatures: Vec<BundleSignature>, // data_class: INTERNAL_ONLY
}

/// [`PolicyBundleStore`] over one signed JSON document at a fixed path,
/// verified against a trusted public-key set loaded from a trust-anchor dir.
#[derive(Debug, Clone)]
pub struct FilePolicyBundleStore {
    path: PathBuf,
    trust_dir: PathBuf,
}

impl FilePolicyBundleStore {
    /// A store reading the signed bundle document from `path` (ConfigMap mount
    /// in K8s) and the trusted signing public keys from `trust_dir` (a separate
    /// ConfigMap-projected directory of hex-encoded Ed25519 public-key files).
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, trust_dir: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            trust_dir: trust_dir.into(),
        }
    }

    /// The backing bundle path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The trust-anchor directory.
    #[must_use]
    pub fn trust_dir(&self) -> &Path {
        &self.trust_dir
    }
}

/// File extension trusted-key files must carry (defensive: skip stray files in
/// a ConfigMap mount such as `..data` symlinks Kubernetes projects).
const TRUST_KEY_EXTENSION: &str = "pub";

/// Load every trusted Ed25519 signing public key from `trust_dir` into a
/// verifier. Each `*.pub` file's stem is the `key_id` and its contents are the
/// raw 32-byte Edwards public key as lowercase hex. Fail-closed: an unreadable
/// directory, or one carrying no usable key, is a [`BundleStoreError`] (a PDP
/// that cannot prove which keys to trust must never serve).
///
/// Returns the verifier plus the set of loaded key_ids (for diagnostics).
fn load_trust_anchor(
    trust_dir: &Path,
) -> Result<(Ed25519ChainVerifier, Vec<String>), BundleStoreError> {
    let entries = std::fs::read_dir(trust_dir).map_err(|e| BundleStoreError::Unavailable {
        detail: format!("cannot read trust anchor dir {}: {e}", trust_dir.display()),
    })?;
    let mut verifier = Ed25519ChainVerifier::new();
    let mut key_ids = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| BundleStoreError::Unavailable {
            detail: format!("trust anchor dir {} entry: {e}", trust_dir.display()),
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some(TRUST_KEY_EXTENSION) {
            continue;
        }
        let Some(key_id) = path.file_stem().and_then(|s| s.to_str()).map(str::to_owned) else {
            continue;
        };
        let hex = std::fs::read_to_string(&path).map_err(|e| BundleStoreError::Unavailable {
            detail: format!("cannot read trusted key {}: {e}", path.display()),
        })?;
        let public_key = decode_hex(hex.trim()).map_err(|e| BundleStoreError::Malformed {
            detail: format!("trusted key {} is not valid hex: {e}", path.display()),
        })?;
        verifier = verifier.with_key(key_id.clone(), public_key);
        key_ids.push(key_id);
    }
    if key_ids.is_empty() {
        // Fail-closed: a present-but-keyless trust anchor proves nothing.
        return Err(BundleStoreError::SignatureRejected {
            detail: format!(
                "trust anchor dir {} carried no trusted signing keys ({TRUST_KEY_EXTENSION} files)",
                trust_dir.display()
            ),
        });
    }
    Ok((verifier, key_ids))
}

/// Verify a signed envelope against the trusted key set and return the verified
/// inner bytes. Fail-closed: empty signatures, or no signature whose `key_id`
/// is trusted AND validates the inner bytes, is [`BundleStoreError::SignatureRejected`].
fn verify_envelope<'a>(
    doc: &'a SignedPolicyBundleDoc,
    verifier: &Ed25519ChainVerifier,
) -> Result<&'a str, BundleStoreError> {
    if doc.signatures.is_empty() {
        return Err(BundleStoreError::SignatureRejected {
            detail: "signed bundle carries no signatures".to_owned(),
        });
    }
    let inner = doc.bundle.as_bytes();
    // A bundle verifies iff ANY trusted key whose key_id matches a signature
    // validates the EXACT stored inner bytes (rotation = several trusted keys).
    // An unknown key_id is not an error here — it just is not the trusted key;
    // we keep scanning. The bundle is rejected only if NONE of the signatures
    // is both trusted and valid.
    for sig in &doc.signatures {
        match verifier.verify(&sig.key_id, inner, &sig.signature_hex) {
            Ok(()) => return Ok(&doc.bundle),
            Err(DigestChainError::UnknownKeyId(_)) => {} // not a trusted key; keep scanning
            Err(DigestChainError::SignatureInvalid { .. }) => {} // trusted key, bad sig; keep scanning
            Err(other) => {
                // Malformed signature hex (or any other structural fault) is a
                // hard reject — a well-formed signed bundle never carries one.
                return Err(BundleStoreError::SignatureRejected {
                    detail: format!("signature for key_id {:?} unusable: {other}", sig.key_id),
                });
            }
        }
    }
    Err(BundleStoreError::SignatureRejected {
        detail: "no trusted key produced a valid signature over the bundle".to_owned(),
    })
}

/// Parse + invariant-check the VERIFIED inner bundle bytes. Runs strictly
/// AFTER signature verification, so the version-token re-validation lives
/// INSIDE the verified region. Exposed for reuse by tests and the reload path;
/// every failure is a [`BundleStoreError`].
fn parse_bundle(raw: &str) -> Result<PolicyBundle, BundleStoreError> {
    let bundle: PolicyBundle =
        serde_json::from_str(raw).map_err(|e| BundleStoreError::Malformed {
            detail: e.to_string(),
        })?;
    // serde(transparent) deserialization bypasses PolicyVersion::new's
    // opaque-token invariants; re-run them so a malformed version token can
    // never become a serving bundle (it would corrupt zookie comparisons and
    // decision-cache keys downstream).
    PolicyVersion::new(bundle.version.as_str()).map_err(|violations| {
        BundleStoreError::Malformed {
            detail: format!("bundle version token rejected: {violations:?}"),
        }
    })?;
    Ok(bundle)
}

/// Parse the signed envelope, verify it against `verifier`, then parse the
/// VERIFIED inner bytes. Exposed for the reload path and tests; every failure
/// is a [`BundleStoreError`] and the inner bundle is parsed only after verify.
fn parse_signed_bundle(
    raw: &str,
    verifier: &Ed25519ChainVerifier,
) -> Result<PolicyBundle, BundleStoreError> {
    let doc: SignedPolicyBundleDoc =
        serde_json::from_str(raw).map_err(|e| BundleStoreError::Malformed {
            detail: format!("signed bundle envelope malformed: {e}"),
        })?;
    let verified_inner = verify_envelope(&doc, verifier)?;
    parse_bundle(verified_inner)
}

impl PolicyBundleStore for FilePolicyBundleStore {
    fn load(&self) -> Result<PolicyBundle, BundleStoreError> {
        // The trust anchor is read first: a PDP that cannot establish its
        // trusted-key set must refuse before it even reads the bundle.
        let (verifier, _key_ids) = load_trust_anchor(&self.trust_dir)?;
        let raw =
            std::fs::read_to_string(&self.path).map_err(|e| BundleStoreError::Unavailable {
                detail: format!("cannot read {}: {e}", self.path.display()),
            })?;
        parse_signed_bundle(&raw, &verifier)
    }

    fn describe(&self) -> String {
        format!(
            "file:{} (trust:{})",
            self.path.display(),
            self.trust_dir.display()
        )
    }
}
