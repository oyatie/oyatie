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
use std::time::{SystemTime, UNIX_EPOCH};

use audit_chain_domain::{
    AuditAppendInput, AuditChain, AuditChainError, Ed25519SigningKey, Ed25519VerificationKeySet,
    Plane, append as audit_append,
};
use audit_file_adapter::FileAuditLedger;
use iam_pdp_cedar::{
    AuditChainCedarPdp, CedarPdp, PDP_DECISION_AUDIT_SURFACE, PdpAuditChainError,
    PdpDecisionAuditChainLogger,
};
use data_boundary_kernel::{DataClass, Purpose};
use shared_pdp_kernel::{
    EntityRecord, EntitySlice, PdpError, PolicyBundle, PolicyDecisionPoint, TemplateLink,
    TemplateSrc,
};
use shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, Decision, EntityRef, PolicyVersion,
};
use shared_ulid_id_kernel::SeededIdGenerator;

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
            // A NON-restricted acme resource. Ordinary within-tenant read
            // grants (PBAC links, tenant overlays, workload permits) target
            // this doc so they exercise their real intent without colliding
            // with the security-critical `forbid-restricted-read-without-step-up`
            // gate (which only fires on data_class == "restricted").
            EntityRecord {
                uid: entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
                attributes: string_attrs(&[
                    ("tenant_id", "acme"),
                    ("resource_kind", "document"),
                    ("data_class", "internal"),
                    ("cell_id", "cell-001"),
                ]),
                parents: vec![entity_ref("OyaPlatform::Tenant", "acme")],
            },
            // A globex (foreign-tenant) resource: lets a cross-tenant read
            // (acme principal -> globex resource) be exercised so the
            // structural forbid can be proved as the runtime boundary.
            EntityRecord {
                uid: entity_ref("OyaPlatform::TenantResource", "globex-doc-1"),
                attributes: string_attrs(&[
                    ("tenant_id", "globex"),
                    ("resource_kind", "document"),
                    ("data_class", "internal"),
                    ("cell_id", "cell-002"),
                ]),
                parents: vec![entity_ref("OyaPlatform::Tenant", "globex")],
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

fn unique_ledger_path(test_name: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test-side: system clock must be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "pdp-audit-chain-{test_name}-{}-{now}.ledger",
        std::process::id()
    ))
}

fn audit_input(tenant_id: &str, decision: &str) -> AuditAppendInput {
    AuditAppendInput {
        tenant_id: tenant_id.to_owned(),
        surface: PDP_DECISION_AUDIT_SURFACE.to_owned(),
        plane: Plane::Control,
        purpose: Purpose::CoreService,
        data_classes: vec![DataClass::InternalOnly, DataClass::Audit],
        decision: decision.to_owned(),
    }
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
    // acme-doc-2 is NON-restricted: this isolates the PBAC grant from the
    // step-up forbid (which only gates restricted reads).
    let link = TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-bob-doc2".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "bob"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
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
                entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Allow);
    assert_eq!(
        outcome.response.determining_policy_ids,
        vec!["pbac-link-bob-doc2".to_owned()],
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
    // bob, an acme principal, reading a NON-restricted acme resource: the acme
    // overlay grants it, attributed to the overlay's namespaced id. (acme-doc-2
    // is non-restricted so the step-up forbid does not apply; the forbid's
    // effect on a restricted read is proven by
    // `tenant_overlay_permit_cannot_bypass_step_up_forbid`.)
    let outcome = pdp
        .authorize(
            &request(
                "req-ovl-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
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
            BTreeMap::from([("acme".to_owned(), "permit (this is not cedar".to_owned())]),
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
    let pdp = pdp_with_overlays(BTreeMap::from([("globex".to_owned(), overlay.to_owned())]));
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

// -------------------- the REAL tenant-isolation boundary (G004 audit) ----
//
// The sole, formally-verified tenant-isolation boundary is the global
// `structural-tenant-isolation` forbid (forbid-overrides-permit) over the
// schema-required `tenant_id` attribute — NOT any load-time overlay check.
// These tests lock that boundary directly so a maintainer cannot remove the
// forbid (or weaken the schema) without a red suite.

/// The 5 cross-tenant evasion permit shapes from the G004 security audit. Each
/// carries the same-tenant equality as a NON-binding token (behind `||`, in an
/// `unless`, behind `!`, etc.), so a substring/EST-presence detector would
/// wrongly accept them. The sound detector REJECTS all 5 at load
/// (`sound_detector_rejects_every_audit_evasion_overlay`); and even if one were
/// admitted, the runtime forbid still denies the cross-tenant read
/// (`structural_forbid_denies_cross_tenant_read_for_any_permit_shape`).
const EVASION_PERMITS: &[(&str, &str)] = &[
    (
        "evasion-or-true",
        r#"permit (principal, action == OyaPlatform::Action::"ReadResource", resource)
           when { principal.tenant_id == resource.tenant_id || true };"#,
    ),
    (
        "evasion-unless-true",
        r#"permit (principal, action == OyaPlatform::Action::"ReadResource", resource)
           when { principal.tenant_id == resource.tenant_id } unless { true };"#,
    ),
    (
        "evasion-guard-in-unless",
        r#"permit (principal, action == OyaPlatform::Action::"ReadResource", resource)
           unless { principal.tenant_id == resource.tenant_id };"#,
    ),
    (
        "evasion-negated",
        r#"permit (principal, action == OyaPlatform::Action::"ReadResource", resource)
           when { !(principal.tenant_id == resource.tenant_id) };"#,
    ),
    (
        "evasion-or-tautology",
        r#"permit (principal, action == OyaPlatform::Action::"ReadResource", resource)
           when { (principal.tenant_id == resource.tenant_id) || (1 == 1) };"#,
    ),
];

/// Legitimate, genuinely tenant-confined permit shapes the sound detector MUST
/// keep accepting: canonical, operand-swapped, parenthesized (parens are
/// transparent in the EST), and `&&`-nested.
const LEGITIMATE_PERMITS: &[(&str, &str)] = &[
    (
        "ok-canonical",
        r#"permit (principal, action == OyaPlatform::Action::"ReadResource", resource)
           when { principal.tenant_id == resource.tenant_id };"#,
    ),
    (
        "ok-operand-swap",
        r#"permit (principal, action == OyaPlatform::Action::"ReadResource", resource)
           when { resource.tenant_id == principal.tenant_id };"#,
    ),
    (
        "ok-parenthesized",
        r#"permit (principal, action == OyaPlatform::Action::"ReadResource", resource)
           when { (principal.tenant_id == resource.tenant_id) };"#,
    ),
    (
        "ok-and-nested",
        r#"permit (principal, action == OyaPlatform::Action::"ReadResource", resource)
           when { resource.resource_kind == "document" && principal.tenant_id == resource.tenant_id };"#,
    ),
];

#[test]
fn structural_forbid_present_in_every_per_tenant_merged_set() {
    // Load benign overlays for BOTH tenants, then prove the structural forbid
    // governs EACH per-tenant merged set: a cross-tenant read (acme principal
    // -> globex resource) is denied AND the deny is attributed to the
    // structural forbid, which can only be true if the forbid is present and
    // armed in the per-tenant set actually evaluated.
    let acme_overlay = r#"
@id("ovl-acme-benign")
permit (
  principal == OyaPlatform::Principal::"bob",
  action == OyaPlatform::Action::"ReadResource",
  resource
)
when { principal.tenant_id == resource.tenant_id };
"#;
    let globex_overlay = r#"
@id("ovl-globex-benign")
permit (
  principal == OyaPlatform::Principal::"mallory",
  action == OyaPlatform::Action::"ReadResource",
  resource
)
when { principal.tenant_id == resource.tenant_id };
"#;
    let pdp = pdp_with_overlays(BTreeMap::from([
        ("acme".to_owned(), acme_overlay.to_owned()),
        ("globex".to_owned(), globex_overlay.to_owned()),
    ]));
    // acme's merged set: bob (acme) reading a globex resource crosses the
    // boundary and must be denied by the structural forbid.
    let acme_outcome = pdp
        .authorize(
            &request(
                "req-merged-acme",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "globex-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(acme_outcome.response.decision, Decision::Deny);
    assert!(
        acme_outcome
            .response
            .determining_policy_ids
            .iter()
            .any(|id| id == "structural-tenant-isolation"),
        "the cross-tenant deny in acme's merged set must be attributed to the \
         structural forbid (present + armed), got {:?}",
        acme_outcome.response.determining_policy_ids
    );
    // globex's merged set: mallory (globex) reading an acme resource is the
    // symmetric cross-tenant read; same forbid must govern.
    let globex_outcome = pdp
        .authorize(
            &request(
                "req-merged-globex",
                "globex",
                entity_ref("OyaPlatform::Principal", "mallory"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(globex_outcome.response.decision, Decision::Deny);
    assert!(
        globex_outcome
            .response
            .determining_policy_ids
            .iter()
            .any(|id| id == "structural-tenant-isolation"),
        "the cross-tenant deny in globex's merged set must be attributed to the \
         structural forbid (present + armed), got {:?}",
        globex_outcome.response.determining_policy_ids
    );
}

#[test]
fn entity_slice_missing_tenant_id_is_rejected_by_schema_validation() {
    // The structural forbid is only sound because the schema makes `tenant_id`
    // a REQUIRED attribute on every principal/resource. Drop it from one
    // entity and the slice must be rejected by schema validation (fail closed),
    // never silently evaluated.
    let mut slice = entity_slice();
    // Strip tenant_id from the acme-doc-1 resource record.
    let doc = slice
        .entities
        .iter_mut()
        .find(|e| e.uid.entity_id == "acme-doc-1")
        .expect("fixture must contain acme-doc-1");
    doc.attributes.remove("tenant_id");
    let pdp = pdp(vec![]);
    let err = pdp
        .authorize(
            &request(
                "req-missing-tid",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &slice,
        )
        .unwrap_err();
    assert!(
        matches!(err, PdpError::Evaluation { .. }),
        "an entity slice missing the schema-required tenant_id must be rejected, got {err:?}"
    );
}

#[test]
fn structural_forbid_denies_cross_tenant_read_for_any_permit_shape() {
    // Prove the RUNTIME forbid is the boundary, independent of the load-time
    // overlay detector: inject each audit evasion permit DIRECTLY into the
    // GLOBAL policy set (bypassing the overlay path entirely), then show a
    // cross-tenant read (bob@acme -> globex resource) is still denied. The
    // forbid wins over every permit shape, so the boundary holds even if a
    // detector were defeated.
    for (id, body) in EVASION_PERMITS {
        let mut bundle = locked_seed_bundle("psv-000001", vec![]);
        bundle
            .policies_src
            .push_str(&format!("\n@id(\"{id}\")\n{body}\n"));
        let pdp = CedarPdp::load(&bundle, Arc::new(SeededIdGenerator::default()), 64)
            .unwrap_or_else(|e| panic!("global injection of {id} must still load: {e}"));
        let outcome = pdp
            .authorize(
                &request(
                    "req-evasion-global",
                    "acme",
                    entity_ref("OyaPlatform::Principal", "bob"),
                    "resource.read",
                    entity_ref("OyaPlatform::TenantResource", "globex-doc-1"),
                ),
                &entity_slice(),
            )
            .unwrap();
        assert_eq!(
            outcome.response.decision,
            Decision::Deny,
            "{id}: the structural forbid must deny the cross-tenant read whatever \
             shape the permit takes"
        );
    }
}

#[test]
fn sound_detector_rejects_every_audit_evasion_overlay() {
    // Each evasion overlay carries the same-tenant equality as a NON-binding
    // token (behind ||, in an unless, behind !, etc.). The sound load-time
    // detector must REJECT all 5 — a non-binding accept-on-presence detector
    // would (wrongly) admit them.
    for (id, body) in EVASION_PERMITS {
        let overlay = format!("@id(\"{id}\")\n{body}");
        let result = CedarPdp::load(
            &locked_seed_bundle_with_overlays(
                "psv-000001",
                vec![],
                BTreeMap::from([("acme".to_owned(), overlay)]),
            ),
            Arc::new(SeededIdGenerator::default()),
            64,
        );
        assert!(
            matches!(result, Err(PdpError::BundleRejected { .. })),
            "{id}: a non-binding same-tenant guard must be rejected at load, got {:?}",
            result.map(|_| "loaded")
        );
    }
}

#[test]
fn sound_detector_accepts_every_legitimate_overlay_shape() {
    // The sound detector must keep accepting genuinely tenant-confined permits:
    // canonical, operand-swapped, parenthesized, and &&-nested.
    for (id, body) in LEGITIMATE_PERMITS {
        let overlay = format!("@id(\"{id}\")\n{body}");
        let result = CedarPdp::load(
            &locked_seed_bundle_with_overlays(
                "psv-000001",
                vec![],
                BTreeMap::from([("acme".to_owned(), overlay)]),
            ),
            Arc::new(SeededIdGenerator::default()),
            64,
        );
        assert!(
            result.is_ok(),
            "{id}: a legitimately tenant-confined permit must load, got {:?}",
            result.err()
        );
    }
}

#[test]
fn tenant_overlay_permit_cannot_bypass_step_up_forbid() {
    // MAJOR (G004 audit): an overlay `permit` must NOT defeat the global,
    // security-critical step-up gate on restricted reads. The gate is encoded
    // as a FORBID (`forbid-restricted-read-without-step-up`), so even a
    // perfectly tenant-confined overlay permit cannot grant bob (NO
    // step_up_class) a read of the RESTRICTED acme-doc-1. Forbid wins.
    let overlay = r#"
@id("ovl-bob-restricted")
permit (
  principal == OyaPlatform::Principal::"bob",
  action == OyaPlatform::Action::"ReadResource",
  resource
)
when { principal.tenant_id == resource.tenant_id };
"#;
    let pdp = pdp_with_overlays(BTreeMap::from([("acme".to_owned(), overlay.to_owned())]));
    let outcome = pdp
        .authorize(
            &request(
                "req-stepup-bypass",
                "acme",
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
        "an overlay permit must not bypass the step-up forbid on restricted reads"
    );
    // The SAME overlay grants the non-restricted acme-doc-2 (the legitimate
    // overlay purpose is preserved — only the security gate is non-bypassable).
    let allowed = pdp
        .authorize(
            &request(
                "req-stepup-ok",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(
        allowed.response.decision,
        Decision::Allow,
        "the overlay still grants an ordinary (non-restricted) within-tenant read"
    );
}

#[test]
fn step_up_forbid_still_allows_a_stepped_up_restricted_read() {
    // The forbid's `unless` exception holds: alice (step_up_class "a") still
    // reads the restricted acme-doc-1 — the gate denies only the non-stepped-up.
    let pdp = pdp(vec![]);
    let outcome = pdp
        .authorize(
            &request(
                "req-stepup-allow",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Allow);
}

#[test]
fn step_up_forbid_blocks_a_pbac_link_to_a_restricted_read() {
    // MAJOR (G004 audit): a PBAC template-link must NOT defeat the global,
    // security-critical step-up gate on restricted reads. This locks the
    // template-link grant path specifically (the overlay path is locked by
    // `tenant_overlay_permit_cannot_bypass_step_up_forbid`, and the
    // template-link path against the STRUCTURAL forbid by
    // `structural_forbid_overrides_misissued_cross_tenant_template_link`; this
    // is the template-link path against the STEP-UP forbid). The gate is
    // encoded as a FORBID (`forbid-restricted-read-without-step-up`), so even an
    // explicit link granting bob (NO step_up_class) a read of the RESTRICTED
    // acme-doc-1 stays denied — forbid overrides permit.
    let link = TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-bob-restricted".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "bob"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    };
    let pdp_restricted = pdp(vec![link]);
    let denied = pdp_restricted
        .authorize(
            &request(
                "req-pbac-restricted-deny",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(
        denied.response.decision,
        Decision::Deny,
        "a PBAC template-link must not bypass the step-up forbid on restricted reads"
    );

    // The SAME link grants the NON-restricted acme-doc-2 (the legitimate link
    // purpose is preserved — only the restricted-read security gate is
    // non-bypassable).
    let link_ok = TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-bob-restricted".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "bob"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
    };
    let pdp_ok = pdp(vec![link_ok]);
    let allowed = pdp_ok
        .authorize(
            &request(
                "req-pbac-restricted-ok",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(
        allowed.response.decision,
        Decision::Allow,
        "the link still grants an ordinary (non-restricted) within-tenant read"
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
    // acme-doc-2 is non-restricted: the pre-swap grant is an ordinary read,
    // isolated from the step-up forbid.
    let link = TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-bob-doc2".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "bob"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
    };
    let pdp = pdp(vec![link]);
    let req = request(
        "req-revoke-1",
        "acme",
        entity_ref("OyaPlatform::Principal", "bob"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
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
    broken.policies_src =
        "permit (principal, action, resource) when { principal.nonexistent_attr == \"x\" };"
            .to_owned();
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
fn audited_pdp_persists_signed_audit_chain_event_with_decision_lineage() {
    let ledger_path = unique_ledger_path("decision-lineage");
    let signer = Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [7_u8; 32]).unwrap();
    let trusted_keys = Ed25519VerificationKeySet::single(signer.verification_key()).unwrap();
    let logger = PdpDecisionAuditChainLogger::new(
        FileAuditLedger::new(ledger_path.clone()),
        signer,
        trusted_keys.clone(),
    )
    .unwrap();
    let pdp = AuditChainCedarPdp::load(
        &locked_seed_bundle("psv-000001", vec![]),
        Arc::new(SeededIdGenerator::default()),
        64,
        logger,
    )
    .unwrap();

    let outcome = pdp
        .authorize(
            &request(
                "req-audit-chain-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();

    let persisted = FileAuditLedger::new(ledger_path.clone())
        .load_with_trusted_keys(&trusted_keys)
        .unwrap();
    let events = persisted.events();
    assert_eq!(
        events.len(),
        1,
        "exactly one durable audit-chain event is appended"
    );
    let event = &events[0];
    assert_eq!(event.tenant_id, "acme");
    assert_eq!(event.surface, PDP_DECISION_AUDIT_SURFACE);
    assert_eq!(event.plane, Plane::Control);
    assert_eq!(event.purpose, Purpose::CoreService);
    assert_eq!(
        event.data_classes,
        vec![DataClass::InternalOnly, DataClass::Audit]
    );
    assert!(
        event.ed25519_signature.is_some(),
        "audit-chain event must be signed"
    );

    let lineage: serde_json::Value = serde_json::from_str(&event.decision).unwrap();
    assert_eq!(lineage["decision_id"], outcome.response.decision_id);
    assert_eq!(lineage["request_id"], "req-audit-chain-1");
    assert_eq!(lineage["tenant_id"], "acme");
    assert_eq!(lineage["resource"]["entity_id"], "acme-doc-1");
    assert_eq!(lineage["action"], "resource.read");
    assert_eq!(lineage["decision"], "allow");
    assert_eq!(lineage["policy_version"], "psv-000001");

    std::fs::remove_file(ledger_path).ok();
}

#[test]
fn audited_pdp_refuses_unsigned_prior_multi_tenant_ledger_replay() {
    let ledger_path = unique_ledger_path("unsigned-prior-ledger");
    let ledger = FileAuditLedger::new(ledger_path.clone());
    let mut chain = AuditChain::multi_tenant_shards();
    audit_append(&mut chain, audit_input("acme", "preexisting-acme"), None).unwrap();
    audit_append(
        &mut chain,
        audit_input("globex", "preexisting-globex"),
        None,
    )
    .unwrap();
    ledger.append_chain(&chain).unwrap();

    let signer = Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [7_u8; 32]).unwrap();
    let trusted_keys = Ed25519VerificationKeySet::single(signer.verification_key()).unwrap();
    let err = PdpDecisionAuditChainLogger::new(ledger, signer, trusted_keys).unwrap_err();

    assert!(
        matches!(
            err,
            PdpAuditChainError::TrustedSignatureReplay(
                AuditChainError::MissingEd25519Signature { .. }
            )
        ),
        "unsigned prior ledger must fail closed at startup, got {err:?}"
    );

    std::fs::remove_file(ledger_path).ok();
}

#[test]
fn audited_pdp_refuses_attacker_signed_prior_multi_tenant_ledger_replay() {
    let ledger_path = unique_ledger_path("attacker-signed-prior-ledger");
    let ledger = FileAuditLedger::new(ledger_path.clone());
    let trusted_signer =
        Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [7_u8; 32]).unwrap();
    let attacker_signer =
        Ed25519SigningKey::from_seed_bytes("attacker-pdp-audit-key", [9_u8; 32]).unwrap();
    let mut chain = AuditChain::multi_tenant_shards();
    audit_append(
        &mut chain,
        audit_input("acme", "attacker-signed-acme"),
        Some(&attacker_signer),
    )
    .unwrap();
    audit_append(
        &mut chain,
        audit_input("globex", "attacker-signed-globex"),
        Some(&attacker_signer),
    )
    .unwrap();
    ledger.append_chain(&chain).unwrap();

    let trusted_keys =
        Ed25519VerificationKeySet::single(trusted_signer.verification_key()).unwrap();
    let err = PdpDecisionAuditChainLogger::new(ledger, trusted_signer, trusted_keys).unwrap_err();

    assert!(
        matches!(
            err,
            PdpAuditChainError::TrustedSignatureReplay(
                AuditChainError::MissingTrustedEd25519Key { .. }
            )
        ),
        "attacker-signed prior ledger must fail closed at startup, got {err:?}"
    );

    std::fs::remove_file(ledger_path).ok();
}

#[test]
fn audited_pdp_refuses_serving_signer_when_trusted_key_material_differs() {
    let ledger_path = unique_ledger_path("untrusted-serving-signer");
    let signer = Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [7_u8; 32]).unwrap();
    let key_id_collision =
        Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [9_u8; 32]).unwrap();
    let trusted_keys =
        Ed25519VerificationKeySet::single(key_id_collision.verification_key()).unwrap();
    let err = PdpDecisionAuditChainLogger::new(
        FileAuditLedger::new(ledger_path.clone()),
        signer,
        trusted_keys,
    )
    .unwrap_err();

    assert!(
        matches!(
            err,
            PdpAuditChainError::UntrustedSigner(
                AuditChainError::Ed25519SignatureKeyMismatch { .. }
            )
        ),
        "serving signer must match trusted key material, got {err:?}"
    );

    std::fs::remove_file(ledger_path).ok();
}

#[test]
fn audited_pdp_maps_untrusted_persisted_ledger_to_audit_chain_emission() {
    let ledger_path = unique_ledger_path("audit-emission-fail-closed");
    let signer = Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [7_u8; 32]).unwrap();
    let trusted_keys = Ed25519VerificationKeySet::single(signer.verification_key()).unwrap();
    let logger = PdpDecisionAuditChainLogger::new(
        FileAuditLedger::new(ledger_path.clone()),
        signer,
        trusted_keys,
    )
    .unwrap();
    let pdp = AuditChainCedarPdp::load(
        &locked_seed_bundle("psv-000001", vec![]),
        Arc::new(SeededIdGenerator::default()),
        64,
        logger,
    )
    .unwrap();

    let mut untrusted_chain = AuditChain::multi_tenant_shards();
    audit_append(
        &mut untrusted_chain,
        audit_input("acme", "post-startup-unsigned"),
        None,
    )
    .unwrap();
    FileAuditLedger::new(ledger_path.clone())
        .append_chain(&untrusted_chain)
        .unwrap();

    let err = pdp
        .authorize(
            &request(
                "req-audit-chain-fail-closed",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap_err();

    match err {
        PdpError::AuditChainEmission { detail } => {
            assert!(
                detail.contains("trusted signature replay failed"),
                "audit-chain emission detail must name trusted replay failure, got {detail:?}"
            );
            assert!(
                detail.contains("MissingEd25519Signature"),
                "audit-chain emission detail must preserve signature cause, got {detail:?}"
            );
        }
        other => {
            panic!("untrusted persisted ledger must fail closed as audit emission, got {other:?}")
        }
    }

    std::fs::remove_file(ledger_path).ok();
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
    // acme-doc-2 is non-restricted: the obligation rides out on an ordinary
    // grant, undisturbed by the step-up forbid.
    let outcome = pdp
        .authorize(
            &request(
                "req-obl-1",
                "acme",
                entity_ref("OyaPlatform::WorkloadIdentity", "payments"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
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
            "libs/shared-platform-contracts-kernel/cedar/platform.cedarschema",
        ),
        (
            "cedar/platform-policies.cedar",
            "libs/shared-platform-contracts-kernel/cedar/platform-policies.cedar",
        ),
        (
            "cedar/platform-templates.cedar",
            "libs/shared-platform-contracts-kernel/cedar/platform-templates.cedar",
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
            panic!(
                "canonical cedar seed missing: {}: {e}",
                canonical_path.display()
            )
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
