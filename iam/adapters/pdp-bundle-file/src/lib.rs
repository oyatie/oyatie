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
use shared_audit_digest_adapter_awslc::Ed25519ChainVerifier;
use shared_audit_event_kernel::{ChainVerifier, DigestChainError, decode_hex};
use shared_pdp_kernel::PolicyBundle;
use shared_platform_contracts_kernel::pdp::PolicyVersion;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    use shared_audit_digest_adapter_awslc::Ed25519ChainSigner;
    use shared_audit_event_kernel::{ChainSigner, encode_hex};
    use shared_pdp_kernel::{TemplateLink, TemplateSrc};
    use shared_platform_contracts_kernel::pdp::EntityRef;

    /// TEST-SIDE signer only. Production private-key custody is a deferred
    /// founder-gated slice; this slice ships verify-against-trusted-public-keys
    /// + this test signer for fixtures. Reuses the OWNED aws-lc-rs Ed25519
    ///   signer (ADR-0506, ring-free).
    fn unique(tag: &str) -> String {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        format!(
            "{}-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed),
            tag
        )
    }

    fn seed_bundle() -> PolicyBundle {
        PolicyBundle {
            version: PolicyVersion::new("psv-000001").unwrap(),
            schema_src: "schema".to_owned(),
            policies_src: "policies".to_owned(),
            // A non-empty overlay proves the per-tenant field round-trips
            // through the CLOSED schema (deny_unknown_fields) and the
            // version-token re-validation path unchanged.
            tenant_policies: BTreeMap::from([("acme".to_owned(), "// acme overlay\n".to_owned())]),
            templates: vec![TemplateSrc {
                template_id: "pbac-resource-read-grant".to_owned(),
                src: "template".to_owned(),
            }],
            template_links: vec![TemplateLink {
                template_id: "pbac-resource-read-grant".to_owned(),
                link_id: "link-1".to_owned(),
                principal: EntityRef {
                    entity_type: "OyaPlatform::Principal".to_owned(),
                    entity_id: "alice".to_owned(),
                },
                resource: EntityRef {
                    entity_type: "OyaPlatform::TenantResource".to_owned(),
                    entity_id: "doc-1".to_owned(),
                },
            }],
            action_map: BTreeMap::from([(
                "resource.read".to_owned(),
                r#"OyaPlatform::Action::"ReadResource""#.to_owned(),
            )]),
        }
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("iam-pdp-bundle-file-{name}"));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    /// Write a trust-anchor directory containing each (key_id, signer) public
    /// key as `<key_id>.pub` hex. Returns the dir path.
    fn trust_dir_for(tag: &str, keys: &[(&str, &Ed25519ChainSigner)]) -> PathBuf {
        let dir = test_dir(&format!("{}-trust", unique(tag)));
        for (key_id, signer) in keys {
            let hex = encode_hex(&signer.public_key_bytes());
            std::fs::write(dir.join(format!("{key_id}.pub")), hex).expect("write trusted key");
        }
        dir
    }

    /// Sign `inner_bytes` with `signer` under `key_id` into a one-signature
    /// envelope, serialized to JSON.
    fn signed_doc_json(inner_bytes: &str, key_id: &str, signer: &Ed25519ChainSigner) -> String {
        let sig = signer.sign_hex(inner_bytes.as_bytes()).expect("sign");
        let doc = SignedPolicyBundleDoc {
            bundle: inner_bytes.to_owned(),
            signatures: vec![BundleSignature {
                key_id: key_id.to_owned(),
                public_key_hex: encode_hex(&signer.public_key_bytes()),
                signature_hex: sig,
            }],
        };
        serde_json::to_string(&doc).expect("serialize envelope")
    }

    /// Write `contents` to a unique `bundle.json` under a fresh dir; return path.
    fn bundle_file(tag: &str, contents: &str) -> PathBuf {
        let dir = test_dir(&unique(tag));
        let path = dir.join("bundle.json");
        std::fs::write(&path, contents).expect("write bundle");
        path
    }

    // ---- GREEN: signed by a trusted key -> loads ----------------------------

    #[test]
    fn well_formed_signed_bundle_round_trips() {
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        let bundle_path = bundle_file("green", &signed_doc_json(&inner, "psk-1", &signer));
        let trust = trust_dir_for("green", &[("psk-1", &signer)]);

        let store = FilePolicyBundleStore::new(&bundle_path, &trust);
        let loaded = store.load().expect("signed bundle loads");
        assert_eq!(loaded, seed_bundle());
        assert!(
            store
                .describe()
                .contains(&bundle_path.display().to_string())
        );
        assert!(store.describe().contains(&trust.display().to_string()));
    }

    // ---- RED: unsigned (empty signatures) -> rejected -----------------------

    #[test]
    fn unsigned_bundle_fails_closed() {
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        let doc = SignedPolicyBundleDoc {
            bundle: inner,
            signatures: vec![],
        };
        let bundle_path = bundle_file("unsigned", &serde_json::to_string(&doc).unwrap());
        let trust = trust_dir_for("unsigned", &[("psk-1", &signer)]);
        let err = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .unwrap_err();
        assert!(
            matches!(err, BundleStoreError::SignatureRejected { .. }),
            "{err}"
        );
    }

    // ---- RED: tampered inner bytes -> rejected ------------------------------

    #[test]
    fn tampered_inner_bytes_are_rejected() {
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        let doc_json = signed_doc_json(&inner, "psk-1", &signer);
        // Flip one byte of the embedded inner bundle AFTER signing: the stored
        // bytes no longer match the signed bytes.
        let mut doc: SignedPolicyBundleDoc = serde_json::from_str(&doc_json).unwrap();
        let mut tampered = doc.bundle.into_bytes();
        // Mutate a byte deep inside the JSON (the policies_src value region) to a
        // different valid JSON character so the envelope still deserializes.
        let idx = tampered.len() / 2;
        tampered[idx] = if tampered[idx] == b'x' { b'y' } else { b'x' };
        doc.bundle = String::from_utf8(tampered).unwrap();
        let bundle_path = bundle_file("tampered", &serde_json::to_string(&doc).unwrap());
        let trust = trust_dir_for("tampered", &[("psk-1", &signer)]);
        let err = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .unwrap_err();
        assert!(
            matches!(err, BundleStoreError::SignatureRejected { .. }),
            "{err}"
        );
    }

    // ---- RED: signer not in the trust set -> rejected -----------------------

    #[test]
    fn wrong_key_is_rejected() {
        let real = Ed25519ChainSigner::generate("psk-1").unwrap();
        let attacker = Ed25519ChainSigner::generate("psk-1").unwrap(); // same key_id, different key
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        // Signed by the attacker, but the trust set holds the REAL key under the
        // same key_id: the trusted key cannot validate the attacker's signature.
        let bundle_path = bundle_file("wrong-key", &signed_doc_json(&inner, "psk-1", &attacker));
        let trust = trust_dir_for("wrong-key", &[("psk-1", &real)]);
        let err = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .unwrap_err();
        assert!(
            matches!(err, BundleStoreError::SignatureRejected { .. }),
            "{err}"
        );
    }

    #[test]
    fn untrusted_key_id_is_rejected() {
        let signer = Ed25519ChainSigner::generate("rogue").unwrap();
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        // Signed validly by "rogue", but "rogue" is not in the trust set.
        let bundle_path = bundle_file("untrusted", &signed_doc_json(&inner, "rogue", &signer));
        let trusted = Ed25519ChainSigner::generate("psk-1").unwrap();
        let trust = trust_dir_for("untrusted", &[("psk-1", &trusted)]);
        let err = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .unwrap_err();
        assert!(
            matches!(err, BundleStoreError::SignatureRejected { .. }),
            "{err}"
        );
    }

    // ---- version token enforced INSIDE the verified region ------------------

    #[test]
    fn version_token_enforced_inside_verified_region() {
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        // A bundle whose version token is malformed, but VALIDLY SIGNED: the
        // signature passes, then the inner version-token re-validation rejects.
        let mut value = serde_json::to_value(seed_bundle()).unwrap();
        value["version"] = serde_json::json!("has whitespace");
        let inner = value.to_string();
        let bundle_path = bundle_file("bad-version", &signed_doc_json(&inner, "psk-1", &signer));
        let trust = trust_dir_for("bad-version", &[("psk-1", &signer)]);
        let err = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .unwrap_err();
        assert!(matches!(err, BundleStoreError::Malformed { .. }), "{err}");
        assert!(err.to_string().contains("version token rejected"), "{err}");
    }

    #[test]
    fn unknown_inner_fields_rejected_inside_verified_region() {
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        let mut value = serde_json::to_value(seed_bundle()).unwrap();
        value["extra_field"] = serde_json::json!("smuggled");
        let inner = value.to_string();
        let bundle_path = bundle_file("unknown-inner", &signed_doc_json(&inner, "psk-1", &signer));
        let trust = trust_dir_for("unknown-inner", &[("psk-1", &signer)]);
        let err = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .unwrap_err();
        assert!(matches!(err, BundleStoreError::Malformed { .. }), "{err}");
    }

    // ---- unknown ENVELOPE field -> deny_unknown_fields ----------------------

    #[test]
    fn unknown_envelope_field_is_rejected() {
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        let mut value: serde_json::Value =
            serde_json::from_str(&signed_doc_json(&inner, "psk-1", &signer)).unwrap();
        value["skip_verification"] = serde_json::json!(true);
        let bundle_path = bundle_file("unknown-env", &value.to_string());
        let trust = trust_dir_for("unknown-env", &[("psk-1", &signer)]);
        let err = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .unwrap_err();
        assert!(matches!(err, BundleStoreError::Malformed { .. }), "{err}");
    }

    // ---- key rotation: trust set {A, B}; signed by B only -> loads ----------

    #[test]
    fn key_rotation_any_trusted_key_validates() {
        let key_a = Ed25519ChainSigner::generate("psk-a").unwrap();
        let key_b = Ed25519ChainSigner::generate("psk-b").unwrap();
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        // Signed by B only; the trust set holds BOTH A and B (rotation window).
        let bundle_path = bundle_file("rotation", &signed_doc_json(&inner, "psk-b", &key_b));
        let trust = trust_dir_for("rotation", &[("psk-a", &key_a), ("psk-b", &key_b)]);
        let loaded = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .expect("bundle signed by any trusted key loads");
        assert_eq!(loaded, seed_bundle());
    }

    #[test]
    fn first_untrusted_signature_then_trusted_signature_loads() {
        let trusted = Ed25519ChainSigner::generate("psk-1").unwrap();
        let rogue = Ed25519ChainSigner::generate("rogue").unwrap();
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        // Envelope carries a rogue sig FIRST, then a trusted sig: scanning must
        // not stop at the rogue one — any trusted+valid sig admits the bundle.
        let rogue_sig = rogue.sign_hex(inner.as_bytes()).unwrap();
        let trusted_sig = trusted.sign_hex(inner.as_bytes()).unwrap();
        let doc = SignedPolicyBundleDoc {
            bundle: inner,
            signatures: vec![
                BundleSignature {
                    key_id: "rogue".to_owned(),
                    public_key_hex: encode_hex(&rogue.public_key_bytes()),
                    signature_hex: rogue_sig,
                },
                BundleSignature {
                    key_id: "psk-1".to_owned(),
                    public_key_hex: encode_hex(&trusted.public_key_bytes()),
                    signature_hex: trusted_sig,
                },
            ],
        };
        let bundle_path = bundle_file("multi-sig", &serde_json::to_string(&doc).unwrap());
        let trust = trust_dir_for("multi-sig", &[("psk-1", &trusted)]);
        let loaded = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .expect("a trusted+valid signature anywhere admits the bundle");
        assert_eq!(loaded, seed_bundle());
    }

    // ---- bundle / trust-anchor availability + fail-closed -------------------

    #[test]
    fn missing_bundle_file_is_unavailable_not_a_default() {
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        let trust = trust_dir_for("missing-bundle", &[("psk-1", &signer)]);
        let store = FilePolicyBundleStore::new("/nonexistent/pdp/bundle.json", &trust);
        let err = store.load().unwrap_err();
        assert!(matches!(err, BundleStoreError::Unavailable { .. }), "{err}");
    }

    #[test]
    fn malformed_envelope_json_fails_closed() {
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        let bundle_path = bundle_file("garbage", "{ not json");
        let trust = trust_dir_for("garbage", &[("psk-1", &signer)]);
        let err = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .unwrap_err();
        assert!(matches!(err, BundleStoreError::Malformed { .. }), "{err}");
    }

    #[test]
    fn absent_trust_anchor_dir_refuses() {
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        let bundle_path = bundle_file("absent-trust", &signed_doc_json(&inner, "psk-1", &signer));
        let absent = std::env::temp_dir().join(format!(
            "iam-pdp-bundle-file-{}-absent-trust-does-not-exist",
            unique("absent")
        ));
        let _ = std::fs::remove_dir_all(&absent);
        let err = FilePolicyBundleStore::new(&bundle_path, &absent)
            .load()
            .unwrap_err();
        assert!(matches!(err, BundleStoreError::Unavailable { .. }), "{err}");
    }

    #[test]
    fn empty_trust_anchor_dir_refuses() {
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        let bundle_path = bundle_file("empty-trust", &signed_doc_json(&inner, "psk-1", &signer));
        // A trust dir that EXISTS but carries no *.pub keys is fail-closed.
        let empty = test_dir(&format!("{}-empty-keys", unique("empty")));
        let err = FilePolicyBundleStore::new(&bundle_path, &empty)
            .load()
            .unwrap_err();
        assert!(
            matches!(err, BundleStoreError::SignatureRejected { .. }),
            "{err}"
        );
    }

    #[test]
    fn non_hex_trusted_key_is_malformed() {
        let inner = serde_json::to_string(&seed_bundle()).unwrap();
        let signer = Ed25519ChainSigner::generate("psk-1").unwrap();
        let bundle_path = bundle_file("bad-keyhex", &signed_doc_json(&inner, "psk-1", &signer));
        let trust = test_dir(&format!("{}-bad-keyhex", unique("bad-keyhex")));
        std::fs::write(trust.join("psk-1.pub"), "zznothex").unwrap();
        let err = FilePolicyBundleStore::new(&bundle_path, &trust)
            .load()
            .unwrap_err();
        assert!(matches!(err, BundleStoreError::Malformed { .. }), "{err}");
    }
}
