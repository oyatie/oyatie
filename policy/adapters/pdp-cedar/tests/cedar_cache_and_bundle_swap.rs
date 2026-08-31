//! Decision cache freshness, atomic bundle swap, and refusal to serve a rejected bundle.
//!
//! Part of the G004 Cedar conformance suite; shared fixtures in `conformance/`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod conformance;

use conformance::*;

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
