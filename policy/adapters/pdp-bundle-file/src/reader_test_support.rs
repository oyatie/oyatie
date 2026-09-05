use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use shared_audit_digest_adapter_awslc::Ed25519ChainSigner;
use shared_audit_event_kernel::{ChainSigner, encode_hex};
use shared_pdp_kernel::{PolicyBundle, TemplateLink, TemplateSrc};
use shared_platform_contracts_kernel::pdp::{EntityRef, PolicyVersion};

use super::{BundleSignature, SignedPolicyBundleDoc};

/// TEST-SIDE signer only. Production private-key custody is a deferred
/// founder-gated slice; this slice ships verify-against-trusted-public-keys
/// + this test signer for fixtures. Reuses the OWNED aws-lc-rs Ed25519
///   signer (ADR-0506, ring-free).
pub(super) fn unique(tag: &str) -> String {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed),
        tag
    )
}

pub(super) fn seed_bundle() -> PolicyBundle {
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

pub(super) fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("iam-pdp-bundle-file-{name}"));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

/// Write a trust-anchor directory containing each (key_id, signer) public
/// key as `<key_id>.pub` hex. Returns the dir path.
pub(super) fn trust_dir_for(tag: &str, keys: &[(&str, &Ed25519ChainSigner)]) -> PathBuf {
    let dir = test_dir(&format!("{}-trust", unique(tag)));
    for (key_id, signer) in keys {
        let hex = encode_hex(&signer.public_key_bytes());
        std::fs::write(dir.join(format!("{key_id}.pub")), hex).expect("write trusted key");
    }
    dir
}

/// Sign `inner_bytes` with `signer` under `key_id` into a one-signature
/// envelope, serialized to JSON.
pub(super) fn signed_doc_json(
    inner_bytes: &str,
    key_id: &str,
    signer: &Ed25519ChainSigner,
) -> String {
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
pub(super) fn bundle_file(tag: &str, contents: &str) -> PathBuf {
    let dir = test_dir(&unique(tag));
    let path = dir.join("bundle.json");
    std::fs::write(&path, contents).expect("write bundle");
    path
}
