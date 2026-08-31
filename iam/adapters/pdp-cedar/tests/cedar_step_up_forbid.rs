//! Step-up authentication as a forbid no permit shape can bypass.
//!
//! Part of the G004 Cedar conformance suite; shared fixtures in `conformance/`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod conformance;

use conformance::*;

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
