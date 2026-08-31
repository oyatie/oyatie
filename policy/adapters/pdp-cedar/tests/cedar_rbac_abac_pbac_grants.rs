//! The three grant paths and the structural forbid that overrides them.
//!
//! Part of the G004 Cedar conformance suite; shared fixtures in `conformance/`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod conformance;

use conformance::*;

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
