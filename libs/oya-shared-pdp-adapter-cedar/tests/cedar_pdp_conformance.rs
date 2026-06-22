//! G004 conformance suite: the embedded Cedar PDP evaluated against the
//! LOCKED FD-001 seed bundle (schema + structural tenant-isolation forbid +
//! RBAC/ABAC policies + PBAC template) through the vendor-neutral
//! `PolicyDecisionPoint` port, on the REAL cedar-policy engine.
//!
//! Test ladder rungs covered here (AMENDMENT 7): contract conformance
//! (locked PDP request/response shapes) + integration against the real
//! engine substrate. The cedar/ fixtures are crate-local copies of the G001
//! contract-lock seeds, guarded against drift by
//! `crate_local_cedar_seeds_match_canonical` (same pattern as the
//! backbone-proto parity test).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use oya_shared_pdp_kernel::{
    EntityRecord, EntitySlice, PdpError, PolicyBundle, PolicyDecisionPoint, TemplateLink,
    TemplateSrc,
};
use oya_shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, Decision, EntityRef, PolicyVersion,
};
use oya_shared_pdp_adapter_cedar::CedarPdp;
use oya_shared_ulid_id_kernel::SeededIdGenerator;

const SCHEMA_SRC: &str = include_str!("../cedar/platform.cedarschema");
const POLICIES_SRC: &str = include_str!("../cedar/platform-policies.cedar");
const TEMPLATE_SRC: &str = include_str!("../cedar/platform-templates.cedar");

const TEMPLATE_ID: &str = "pbac-resource-read-grant";

fn entity_ref(entity_type: &str, entity_id: &str) -> EntityRef {
    EntityRef {
        entity_type: entity_type.to_owned(),
        entity_id: entity_id.to_owned(),
    }
}

fn action_map() -> BTreeMap<String, String> {
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

fn locked_seed_bundle(version: &str, template_links: Vec<TemplateLink>) -> PolicyBundle {
    locked_seed_bundle_with_overlays(version, template_links, BTreeMap::new())
}

fn locked_seed_bundle_with_overlays(
    version: &str,
    template_links: Vec<TemplateLink>,
    tenant_policies: BTreeMap<String, String>,
) -> PolicyBundle {
    PolicyBundle {
        version: PolicyVersion::new(version).unwrap(),
        schema_src: SCHEMA_SRC.to_owned(),
        policies_src: POLICIES_SRC.to_owned(),
        tenant_policies,
        templates: vec![TemplateSrc {
            template_id: TEMPLATE_ID.to_owned(),
            src: TEMPLATE_SRC.to_owned(),
        }],
        template_links,
        action_map: action_map(),
    }
}

/// Two tenants in two cells; one cross-tenant-polluted admin group — the
/// exact misconfiguration the structural forbid must neutralize (mirrors the
/// locked contract-kernel validation fixture).
fn entity_slice() -> EntitySlice {
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
            // bob: acme principal WITHOUT step-up class and WITHOUT group
            // membership — only a PBAC template link can let him read.
            EntityRecord {
                uid: entity_ref("OyaPlatform::Principal", "bob"),
                attributes: string_attrs(&[("tenant_id", "acme"), ("kind", "human")]),
                parents: vec![],
            },
            // mallory belongs to ANOTHER tenant but is (mis)joined to the
            // same group — the structural forbid must still deny everything.
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
                uid: entity_ref("OyaPlatform::WorkloadIdentity", "payments"),
                attributes: string_attrs(&[
                    ("tenant_id", "acme"),
                    ("spiffe_id", "spiffe://oyatie/acme/payments"),
                ]),
                parents: vec![entity_ref("OyaPlatform::Tenant", "acme")],
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

fn request(
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

fn pdp(links: Vec<TemplateLink>) -> CedarPdp {
    CedarPdp::load(
        &locked_seed_bundle("psv-000001", links),
        Arc::new(SeededIdGenerator::default()),
        64,
    )
    .expect("locked seed bundle must load")
}

// ---------------------------------------------------------------- RBAC ----

#[test]
fn rbac_group_grant_allows_tenant_admin_within_tenant() {
    let pdp = pdp(vec![]);
    let outcome = pdp
        .authorize(
            &request(
                "req-rbac-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "tenant.administer",
                entity_ref("OyaPlatform::Tenant", "acme"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Allow);
    assert_eq!(
        outcome.response.determining_policy_ids,
        vec!["rbac-tenant-admin-group".to_owned()],
        "every allow is attributable to its permit policy"
    );
}

#[test]
fn rbac_non_member_is_denied_by_default() {
    let pdp = pdp(vec![]);
    let outcome = pdp
        .authorize(
            &request(
                "req-rbac-2",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "tenant.administer",
                entity_ref("OyaPlatform::Tenant", "acme"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Deny);
}

// ---------------------------------------------------------------- ABAC ----

#[test]
fn abac_step_up_class_gates_restricted_reads() {
    let pdp = pdp(vec![]);
    // alice asserted step-up class "a": allow, attributed to the ABAC policy.
    let allowed = pdp
        .authorize(
            &request(
                "req-abac-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(allowed.response.decision, Decision::Allow);
    assert_eq!(
        allowed.response.determining_policy_ids,
        vec!["abac-step-up-restricted-read".to_owned()]
    );
    // bob has no step_up_class attribute: deny-by-default.
    let denied = pdp
        .authorize(
            &request(
                "req-abac-2",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(denied.response.decision, Decision::Deny);
}

// ---------------------------------------------------------------- PBAC ----

#[test]
fn pbac_template_link_grants_scoped_read() {
    let link = TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-bob-doc1".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "bob"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    };
    // Without the link: deny-by-default (proved by the ABAC test above).
    let pdp = pdp(vec![link]);
    let outcome = pdp
        .authorize(
            &request(
                "req-pbac-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Allow);
    assert_eq!(
        outcome.response.determining_policy_ids,
        vec!["pbac-link-bob-doc1".to_owned()],
        "the allow is attributable to the template instantiation"
    );
}

// ---------------------------------------- structural tenant isolation ----

#[test]
fn structural_forbid_overrides_cross_tenant_group_membership() {
    let pdp = pdp(vec![]);
    for (req_id, action, resource) in [
        (
            "req-iso-1",
            "tenant.administer",
            entity_ref("OyaPlatform::Tenant", "acme"),
        ),
        (
            "req-iso-2",
            "resource.read",
            entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
        ),
    ] {
        let outcome = pdp
            .authorize(
                &request(
                    req_id,
                    "globex",
                    entity_ref("OyaPlatform::Principal", "mallory"),
                    action,
                    resource,
                ),
                &entity_slice(),
            )
            .unwrap();
        assert_eq!(
            outcome.response.decision,
            Decision::Deny,
            "{req_id}: structural forbid must override every permit"
        );
    }
}

#[test]
fn structural_forbid_overrides_misissued_cross_tenant_template_link() {
    let link = TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-mallory-doc1".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "mallory"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    };
    let pdp = pdp(vec![link]);
    let outcome = pdp
        .authorize(
            &request(
                "req-iso-3",
                "globex",
                entity_ref("OyaPlatform::Principal", "mallory"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(
        outcome.response.decision,
        Decision::Deny,
        "a mis-issued cross-tenant grant can never defeat the structural forbid"
    );
}

// ------------------------------------------- per-tenant overlays (G004) ----

/// A legitimate acme overlay: grant bob ReadResource, tenant-confined by the
/// canonical same-tenant guard. Without it bob is deny-by-default (proved by
/// `abac_step_up_class_gates_restricted_reads`).
const ACME_OVERLAY_BOB_READ: &str = r#"
@id("ovl-bob-read")
permit (
  principal == OyaPlatform::Principal::"bob",
  action == OyaPlatform::Action::"ReadResource",
  resource
)
when { principal.tenant_id == resource.tenant_id };
"#;

fn pdp_with_overlays(tenant_policies: BTreeMap<String, String>) -> CedarPdp {
    CedarPdp::load(
        &locked_seed_bundle_with_overlays("psv-000001", vec![], tenant_policies),
        Arc::new(SeededIdGenerator::default()),
        64,
    )
    .expect("bundle with overlays must load")
}

#[test]
fn tenant_overlay_permit_applies_only_within_owning_tenant() {
    let pdp = pdp_with_overlays(BTreeMap::from([(
        "acme".to_owned(),
        ACME_OVERLAY_BOB_READ.to_owned(),
    )]));
    // bob, an acme principal, reading an acme resource: the acme overlay grants
    // it, attributed to the overlay's namespaced id.
    let outcome = pdp
        .authorize(
            &request(
                "req-ovl-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Allow);
    assert_eq!(
        outcome.response.determining_policy_ids,
        vec!["acme/ovl-bob-read".to_owned()],
        "the allow is attributable to the tenant-namespaced overlay policy"
    );
}

#[test]
fn tenant_overlay_does_not_leak_to_other_tenant() {
    // The SAME acme overlay; a request whose SVID-bound tenant is globex must
    // NOT see it (the selection is keyed by the request's own tenant_id).
    let pdp = pdp_with_overlays(BTreeMap::from([(
        "acme".to_owned(),
        ACME_OVERLAY_BOB_READ.to_owned(),
    )]));
    let outcome = pdp
        .authorize(
            &request(
                "req-ovl-2",
                "globex",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(
        outcome.response.decision,
        Decision::Deny,
        "globex must never be evaluated against acme's overlay"
    );
}

#[test]
fn fail_closed_default_deny_with_empty_overlays() {
    // Empty tenant_policies + no global permit for bob: deny-by-default.
    let pdp = pdp_with_overlays(BTreeMap::new());
    let outcome = pdp
        .authorize(
            &request(
                "req-ovl-3",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Deny);
}

#[test]
fn malformed_tenant_overlay_rejects_whole_bundle_fail_closed() {
    // An overlay that is not valid Cedar must reject the WHOLE bundle at load.
    let result = CedarPdp::load(
        &locked_seed_bundle_with_overlays(
            "psv-000001",
            vec![],
            BTreeMap::from([(
                "acme".to_owned(),
                "permit (this is not cedar".to_owned(),
            )]),
        ),
        Arc::new(SeededIdGenerator::default()),
        64,
    );
    assert!(
        matches!(result, Err(PdpError::BundleRejected { .. })),
        "a malformed overlay must fail closed, got {:?}",
        result.map(|_| "loaded")
    );
}

#[test]
fn tenant_overlay_authoring_cross_tenant_permit_is_rejected_at_load() {
    // An acme overlay permit WITHOUT the same-tenant guard could grant across a
    // tenant boundary — reject at LOAD so isolation is structural, not emergent.
    let cross_tenant_permit = r#"
@id("ovl-cross")
permit (
  principal is OyaPlatform::Principal,
  action == OyaPlatform::Action::"ReadResource",
  resource
);
"#;
    let result = CedarPdp::load(
        &locked_seed_bundle_with_overlays(
            "psv-000001",
            vec![],
            BTreeMap::from([("acme".to_owned(), cross_tenant_permit.to_owned())]),
        ),
        Arc::new(SeededIdGenerator::default()),
        64,
    );
    assert!(
        matches!(result, Err(PdpError::BundleRejected { .. })),
        "an overlay permit that is not tenant-confined must fail closed at load, got {:?}",
        result.map(|_| "loaded")
    );

    // And an overlay that NAMES another tenant by literal is likewise rejected.
    let foreign_literal_permit = r#"
@id("ovl-foreign")
permit (
  principal is OyaPlatform::Principal,
  action == OyaPlatform::Action::"ReadResource",
  resource
)
when { resource.tenant_id == "globex" };
"#;
    let result = CedarPdp::load(
        &locked_seed_bundle_with_overlays(
            "psv-000001",
            vec![],
            BTreeMap::from([
                ("acme".to_owned(), foreign_literal_permit.to_owned()),
                ("globex".to_owned(), String::new()),
            ]),
        ),
        Arc::new(SeededIdGenerator::default()),
        64,
    );
    assert!(
        matches!(result, Err(PdpError::BundleRejected { .. })),
        "an overlay naming another known tenant must fail closed at load, got {:?}",
        result.map(|_| "loaded")
    );
}

#[test]
fn tenant_overlay_cannot_escape_structural_forbid() {
    // Even a same-tenant-guarded overlay grant cannot defeat the structural
    // forbid: mallory is a globex principal mis-joined to the acme group; an
    // acme overlay permitting mallory must still be denied cross-tenant.
    let overlay = r#"
@id("ovl-mallory")
permit (
  principal == OyaPlatform::Principal::"mallory",
  action == OyaPlatform::Action::"ReadResource",
  resource
)
when { principal.tenant_id == resource.tenant_id };
"#;
    let pdp = pdp_with_overlays(BTreeMap::from([(
        "globex".to_owned(),
        overlay.to_owned(),
    )]));
    let outcome = pdp
        .authorize(
            &request(
                "req-ovl-forbid",
                "globex",
                entity_ref("OyaPlatform::Principal", "mallory"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(
        outcome.response.decision,
        Decision::Deny,
        "the structural forbid overrides every overlay permit across a tenant boundary"
    );
}

// ------------------------------------------------- zookie freshness ----

#[test]
fn stale_policy_version_pin_is_refused_not_answered() {
    let pdp = pdp(vec![]);
    let mut req = request(
        "req-zookie-1",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    );
    req.min_policy_version = Some(PolicyVersion::new("psv-000002").unwrap());
    let err = pdp.authorize(&req, &entity_slice()).unwrap_err();
    assert!(
        matches!(err, PdpError::StalePolicyVersion { .. }),
        "expected stale-version refusal, got {err:?}"
    );
    // Pinning the LOADED version succeeds (read-your-writes satisfied).
    req.min_policy_version = Some(pdp.loaded_policy_version());
    assert!(pdp.authorize(&req, &entity_slice()).is_ok());
}

// --------------------------------------------- decision cache + swap ----

#[test]
fn cache_replays_decision_content_with_fresh_decision_ids() {
    let pdp = pdp(vec![]);
    let req = request(
        "req-cache-1",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    );
    let first = pdp.authorize(&req, &entity_slice()).unwrap();
    assert!(!first.cache_hit);
    let mut replay = req.clone();
    replay.request_id = "req-cache-2".to_owned();
    let second = pdp.authorize(&replay, &entity_slice()).unwrap();
    assert!(second.cache_hit, "identical decision surface must hit");
    assert_eq!(second.response.decision, first.response.decision);
    assert_eq!(
        second.response.determining_policy_ids,
        first.response.determining_policy_ids
    );
    assert_ne!(
        second.response.decision_id, first.response.decision_id,
        "every decision — cached or not — gets its own audit-chain key"
    );
    assert_eq!(second.response.request_id, "req-cache-2");
}

#[test]
fn bundle_swap_revokes_immediately_and_disarms_prior_cache() {
    let link = TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-bob-doc1".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "bob"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    };
    let pdp = pdp(vec![link]);
    let req = request(
        "req-revoke-1",
        "acme",
        entity_ref("OyaPlatform::Principal", "bob"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    );
    let before = pdp.authorize(&req, &entity_slice()).unwrap();
    assert_eq!(before.response.decision, Decision::Allow);
    // Revocation: the policy store pushes a bundle WITHOUT the grant. The
    // swap is atomic; the version change makes every cached allow
    // unreachable (sub-60s revocation = bundle propagation latency).
    pdp.swap_bundle(&locked_seed_bundle("psv-000002", vec![]))
        .unwrap();
    let mut after_req = req.clone();
    after_req.request_id = "req-revoke-2".to_owned();
    let after = pdp.authorize(&after_req, &entity_slice()).unwrap();
    assert_eq!(after.response.decision, Decision::Deny);
    assert!(!after.cache_hit, "prior cache entries must be unreachable");
    assert_eq!(after.response.policy_version.as_str(), "psv-000002");
}

#[test]
fn rejected_bundle_never_replaces_a_serving_one() {
    let pdp = pdp(vec![]);
    let mut broken = locked_seed_bundle("psv-000099", vec![]);
    broken.policies_src = "permit (principal, action, resource) when { principal.nonexistent_attr == \"x\" };".to_owned();
    let err = pdp.swap_bundle(&broken).unwrap_err();
    assert!(matches!(err, PdpError::BundleRejected { .. }));
    assert_eq!(
        pdp.loaded_policy_version().as_str(),
        "psv-000001",
        "the serving bundle must keep serving after a rejected swap"
    );
}

// --------------------------------------------------- audit + errors ----

#[test]
fn every_decision_yields_an_attributable_audit_record() {
    let pdp = pdp(vec![]);
    let allow = pdp
        .authorize(
            &request(
                "req-audit-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(allow.audit.decision_id, allow.response.decision_id);
    assert_eq!(allow.audit.decision, Decision::Allow);
    assert_eq!(allow.audit.tenant_id, "acme");
    assert_eq!(allow.audit.policy_version.as_str(), "psv-000001");
    assert!(!allow.audit.determining_policy_ids.is_empty());

    let deny = pdp
        .authorize(
            &request(
                "req-audit-2",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.write",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(deny.audit.decision, Decision::Deny);
    assert_ne!(deny.audit.decision_id, allow.audit.decision_id);
}

#[test]
fn unknown_action_fails_closed() {
    let pdp = pdp(vec![]);
    let err = pdp
        .authorize(
            &request(
                "req-err-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.purge",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap_err();
    assert!(matches!(err, PdpError::UnknownAction { .. }));
}

#[test]
fn obligations_ride_out_with_annotated_permits() {
    let mut bundle = locked_seed_bundle("psv-000001", vec![]);
    bundle.policies_src.push_str(
        "\n@id(\"workload-read-grant\")\n@obligation(\"emit-step-up-audit\")\npermit (\n  principal is OyaPlatform::WorkloadIdentity,\n  action == OyaPlatform::Action::\"ReadResource\",\n  resource\n)\nwhen { principal.tenant_id == resource.tenant_id };\n",
    );
    let pdp = CedarPdp::load(&bundle, Arc::new(SeededIdGenerator::default()), 64).unwrap();
    let outcome = pdp
        .authorize(
            &request(
                "req-obl-1",
                "acme",
                entity_ref("OyaPlatform::WorkloadIdentity", "payments"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Allow);
    assert_eq!(
        outcome
            .response
            .obligations
            .iter()
            .map(|o| o.obligation_id.as_str())
            .collect::<Vec<_>>(),
        vec!["emit-step-up-audit"]
    );
}

// ------------------------------------------------ seed parity guard ----

// option_env!, not env!: CARGO_MANIFEST_DIR is undefined at buck2 compile
// time (hermetic sandbox), and the buck2 lane must still COMPILE this target
// (FRIC-019). The cargo lane enforces parity; buck2 skips with a notice.
fn manifest_dir() -> Option<&'static Path> {
    option_env!("CARGO_MANIFEST_DIR").map(Path::new)
}

fn repo_root() -> Option<PathBuf> {
    let mut dir = manifest_dir()?.to_path_buf();
    loop {
        if dir.join("Cargo.toml").is_file() && dir.join("docs/decisions").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[test]
fn crate_local_cedar_seeds_match_canonical() {
    const PAIRS: &[(&str, &str)] = &[
        (
            "cedar/platform.cedarschema",
            "libs/oya-shared-platform-contracts-kernel/cedar/platform.cedarschema",
        ),
        (
            "cedar/platform-policies.cedar",
            "libs/oya-shared-platform-contracts-kernel/cedar/platform-policies.cedar",
        ),
        (
            "cedar/platform-templates.cedar",
            "libs/oya-shared-platform-contracts-kernel/cedar/platform-templates.cedar",
        ),
    ];
    let (Some(crate_dir), Some(root)) = (manifest_dir(), repo_root()) else {
        eprintln!(
            "cedar_seed_parity: repo root marker not reachable (hermetic sandbox); \
             skipped {} pairs — cargo CI lane enforces parity",
            PAIRS.len()
        );
        return;
    };
    let mut mismatches = Vec::new();
    for (local, canonical) in PAIRS {
        let local_bytes = std::fs::read(crate_dir.join(local))
            .unwrap_or_else(|e| panic!("crate-local cedar seed missing: {local}: {e}"));
        let canonical_path = root.join(canonical);
        let canonical_bytes = std::fs::read(&canonical_path).unwrap_or_else(|e| {
            panic!("canonical cedar seed missing: {}: {e}", canonical_path.display())
        });
        if local_bytes != canonical_bytes {
            mismatches.push(format!("{local} != {canonical}"));
        }
    }
    assert!(
        mismatches.is_empty(),
        "crate-local cedar seed copies drifted from the canonical contract-lock \
         sources (canonical wins; sync the crate copy in the same change):\n  {}",
        mismatches.join("\n  ")
    );
}
