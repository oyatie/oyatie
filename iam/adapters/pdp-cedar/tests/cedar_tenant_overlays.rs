//! Per-tenant overlay policies: confinement, fail-closed load, and the forbid they cannot escape.
//!
//! Part of the G004 Cedar conformance suite; shared fixtures in `conformance/`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod conformance;

use conformance::*;

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
