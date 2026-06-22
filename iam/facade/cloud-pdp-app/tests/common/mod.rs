//! Shared fixtures for the cloud-iam PDP service test ladder.
//!
//! One fixture set drives the REST contract suite, the gRPC contract suite,
//! and the live-socket E2E, so every surface provably exercises the same
//! seed bundle: the crate-local copies of the canonical FD-001 platform
//! Cedar seeds (parity-guarded by tests/seed_parity.rs), covering RBAC
//! (group grant), ABAC (step-up attribute condition), PBAC (template link),
//! and the structural tenant-isolation forbid.

#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test fixtures; not every consumer uses every helper"
)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use iam_cloud_pdp_app::PdpState;
use iam_cloud_pdp_kernel::InMemoryDecisionAuditSink;
use oya_shared_pdp_adapter_cedar::CedarPdp;
use oya_shared_pdp_kernel::{EntityRecord, EntitySlice, PolicyBundle, TemplateLink, TemplateSrc};
use oya_shared_platform_contracts_kernel::pdp::{AuthorizationRequest, EntityRef, PolicyVersion};
use oya_shared_ulid_id_kernel::SeededIdGenerator;

pub const SCHEMA_SRC: &str = include_str!("../../cedar/platform.cedarschema");
pub const POLICIES_SRC: &str = include_str!("../../cedar/platform-policies.cedar");
pub const TEMPLATE_SRC: &str = include_str!("../../cedar/platform-templates.cedar");

pub const TEMPLATE_ID: &str = "pbac-resource-read-grant";
pub const SEED_VERSION: &str = "psv-000001";

pub fn entity_ref(entity_type: &str, entity_id: &str) -> EntityRef {
    EntityRef {
        entity_type: entity_type.to_owned(),
        entity_id: entity_id.to_owned(),
    }
}

pub fn action_map() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "resource.read".to_owned(),
            r#"OyaPlatform::Action::"ReadResource""#.to_owned(),
        ),
        (
            "resource.write".to_owned(),
            r#"OyaPlatform::Action::"WriteResource""#.to_owned(),
        ),
        (
            "tenant.administer".to_owned(),
            r#"OyaPlatform::Action::"AdministerTenant""#.to_owned(),
        ),
    ])
}

/// The slice-1 seed bundle over the canonical platform Cedar seeds.
pub fn seed_bundle(version: &str, template_links: Vec<TemplateLink>) -> PolicyBundle {
    PolicyBundle {
        version: PolicyVersion::new(version).unwrap(),
        schema_src: SCHEMA_SRC.to_owned(),
        policies_src: POLICIES_SRC.to_owned(),
        tenant_policies: BTreeMap::new(),
        templates: vec![TemplateSrc {
            template_id: TEMPLATE_ID.to_owned(),
            src: TEMPLATE_SRC.to_owned(),
        }],
        template_links,
        action_map: action_map(),
    }
}

/// A PBAC link granting bob a scoped read on acme-doc-1.
pub fn bob_read_link() -> TemplateLink {
    TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-bob-acme-doc-1".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "bob"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    }
}

/// Two tenants; alice = acme tenant-admin with step-up "a"; bob = acme
/// principal with NO group and NO step-up (only a PBAC link can let him
/// read); mallory = globex principal (mis)joined into acme's admin group —
/// the structural forbid must neutralize her entirely.
pub fn entity_slice() -> EntitySlice {
    let string_attrs = |pairs: &[(&str, &str)]| -> BTreeMap<String, serde_json::Value> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), serde_json::json!(v)))
            .collect()
    };
    EntitySlice {
        entities: vec![
            EntityRecord {
                uid: entity_ref("OyaPlatform::Tenant", "acme"),
                attributes: string_attrs(&[
                    ("tenant_id", "acme"),
                    ("cell_id", "cell-001"),
                    ("lifecycle_state", "active"),
                ]),
                parents: vec![],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::Tenant", "globex"),
                attributes: string_attrs(&[
                    ("tenant_id", "globex"),
                    ("cell_id", "cell-002"),
                    ("lifecycle_state", "active"),
                ]),
                parents: vec![],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::Group", "tenant-admins"),
                attributes: string_attrs(&[("tenant_id", "acme")]),
                parents: vec![entity_ref("OyaPlatform::Tenant", "acme")],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::Principal", "alice"),
                attributes: string_attrs(&[
                    ("tenant_id", "acme"),
                    ("kind", "human"),
                    ("step_up_class", "a"),
                ]),
                parents: vec![entity_ref("OyaPlatform::Group", "tenant-admins")],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::Principal", "bob"),
                attributes: string_attrs(&[("tenant_id", "acme"), ("kind", "human")]),
                parents: vec![],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::Principal", "mallory"),
                attributes: string_attrs(&[
                    ("tenant_id", "globex"),
                    ("kind", "human"),
                    ("step_up_class", "a"),
                ]),
                parents: vec![entity_ref("OyaPlatform::Group", "tenant-admins")],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
                attributes: string_attrs(&[
                    ("tenant_id", "acme"),
                    ("resource_kind", "document"),
                    ("data_class", "restricted"),
                    ("cell_id", "cell-001"),
                ]),
                parents: vec![entity_ref("OyaPlatform::Tenant", "acme")],
            },
        ],
    }
}

pub fn request(
    request_id: &str,
    tenant_id: &str,
    principal: EntityRef,
    action: &str,
    resource: EntityRef,
) -> AuthorizationRequest {
    AuthorizationRequest {
        request_id: request_id.to_owned(),
        tenant_id: tenant_id.to_owned(),
        principal,
        action: action.to_owned(),
        resource,
        context: BTreeMap::new(),
        min_policy_version: None,
    }
}

/// Build the shared service state over the seed bundle with an inspectable
/// in-memory audit sink (deterministic ids; tests never depend on wall-clock
/// entropy).
pub fn seeded_state(links: Vec<TemplateLink>) -> (Arc<PdpState>, Arc<InMemoryDecisionAuditSink>) {
    let pdp = CedarPdp::load(
        &seed_bundle(SEED_VERSION, links),
        Arc::new(SeededIdGenerator::default()),
        64,
    )
    .expect("seed bundle must load");
    let sink = Arc::new(InMemoryDecisionAuditSink::new());
    let state = Arc::new(PdpState::new(pdp, Arc::clone(&sink) as Arc<_>));
    (state, sink)
}

/// Write `contents` into a unique temp file and return its path (E2E bundle
/// transport fixture — the ConfigMap stand-in).
pub fn temp_bundle_file(tag: &str, contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "oya-cloud-iam-pdp-e2e-{}-{tag}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("bundle.json");
    std::fs::write(&path, contents).expect("write bundle");
    path
}
