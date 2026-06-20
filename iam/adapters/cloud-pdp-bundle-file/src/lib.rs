//! # oya-cloud-iam-pdp-bundle-file-adapter
//!
//! File-backed [`PolicyBundleStore`] adapter (ADR-0559, G004 slice 1).
//!
//! ## Posture
//! The slice-1 policy-bundle transport is a declarative JSON document on a
//! mounted path — a ConfigMap mount in K8s, a plain file in tests — parsed
//! with a CLOSED schema (`deny_unknown_fields` on every [`PolicyBundle`]
//! field) and re-validated against the locked contract invariants after
//! deserialization (serde's `transparent` `PolicyVersion` bypasses
//! constructor validation, so the adapter re-runs it; a bundle with a
//! whitespace/empty version token is malformed, not loadable).
//!
//! Every error is fail-closed ([`BundleStoreError`]): at boot the service
//! REFUSES TO START (the oya-identity precedent — a serving process is a
//! correctly-configured process), and on reload the serving bundle keeps
//! serving.
//!
//! This adapter is deliberately throwaway (ADR-0550): the destination is the
//! policy-bundle CRD + operator distribution fabric with signature
//! verification at the store boundary (ADR-0536 D-2), which lands behind the
//! SAME kernel port in a follow-up slice. The trait does not change at
//! cutover; this crate is deleted as a unit.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use iam_cloud_pdp_kernel::{BundleStoreError, PolicyBundleStore};
use oya_shared_pdp_kernel::PolicyBundle;
use oya_shared_platform_contracts_kernel::pdp::PolicyVersion;

/// [`PolicyBundleStore`] over one JSON document at a fixed path.
#[derive(Debug, Clone)]
pub struct FilePolicyBundleStore {
    path: PathBuf,
}

impl FilePolicyBundleStore {
    /// A store reading from `path` (ConfigMap mount in K8s deployments).
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The backing path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Parse + invariant-check one bundle document. Exposed for reuse by tests
/// and by the future reload path; every failure is a [`BundleStoreError`].
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

impl PolicyBundleStore for FilePolicyBundleStore {
    fn load(&self) -> Result<PolicyBundle, BundleStoreError> {
        let raw =
            std::fs::read_to_string(&self.path).map_err(|e| BundleStoreError::Unavailable {
                detail: format!("cannot read {}: {e}", self.path.display()),
            })?;
        parse_bundle(&raw)
    }

    fn describe(&self) -> String {
        format!("file:{}", self.path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeMap;

    use oya_shared_pdp_kernel::{TemplateLink, TemplateSrc};
    use oya_shared_platform_contracts_kernel::pdp::EntityRef;

    fn seed_bundle() -> PolicyBundle {
        PolicyBundle {
            version: PolicyVersion::new("psv-000001").unwrap(),
            schema_src: "schema".to_owned(),
            policies_src: "policies".to_owned(),
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

    fn temp_file(name: &str, contents: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "oya-cloud-iam-pdp-bundle-file-{}-{name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("bundle.json");
        std::fs::write(&path, contents).expect("write bundle");
        path
    }

    #[test]
    fn well_formed_bundle_round_trips() {
        let serialized = serde_json::to_string(&seed_bundle()).unwrap();
        let path = temp_file("green", &serialized);
        let store = FilePolicyBundleStore::new(&path);
        let loaded = store.load().expect("bundle loads");
        assert_eq!(loaded, seed_bundle());
        assert_eq!(store.describe(), format!("file:{}", path.display()));
    }

    #[test]
    fn missing_file_is_unavailable_not_a_default() {
        let store = FilePolicyBundleStore::new("/nonexistent/oya-pdp/bundle.json");
        let err = store.load().unwrap_err();
        assert!(matches!(err, BundleStoreError::Unavailable { .. }), "{err}");
    }

    #[test]
    fn malformed_json_fails_closed() {
        let path = temp_file("garbage", "{ not json");
        let err = FilePolicyBundleStore::new(path).load().unwrap_err();
        assert!(matches!(err, BundleStoreError::Malformed { .. }), "{err}");
    }

    #[test]
    fn unknown_fields_are_rejected_closed_schema() {
        let mut value = serde_json::to_value(seed_bundle()).unwrap();
        value["extra_field"] = serde_json::json!("smuggled");
        let path = temp_file("unknown-field", &value.to_string());
        let err = FilePolicyBundleStore::new(path).load().unwrap_err();
        assert!(matches!(err, BundleStoreError::Malformed { .. }), "{err}");
    }

    #[test]
    fn whitespace_version_token_is_rejected_post_deserialization() {
        let mut value = serde_json::to_value(seed_bundle()).unwrap();
        value["version"] = serde_json::json!("has whitespace");
        let path = temp_file("bad-version", &value.to_string());
        let err = FilePolicyBundleStore::new(path).load().unwrap_err();
        assert!(matches!(err, BundleStoreError::Malformed { .. }), "{err}");
        assert!(err.to_string().contains("version token rejected"), "{err}");
    }
}
