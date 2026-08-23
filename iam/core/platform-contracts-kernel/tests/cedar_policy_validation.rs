//! Validates the checked-in `cedar/` seed against the REAL cedar-policy
//! engine (the workspace-pinned 4.11 line): the schema parses, the policy set
//! strict-validates against it, and the RBAC/ABAC/PBAC examples plus the
//! structural tenant-isolation forbid behave exactly as the contract claims.
//!
//! cedar-policy is a dev-dependency only — the production surface of the
//! contracts kernel stays Cedar-free per ADR-0183 policy-engine separation.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use cedar_policy::{
    Authorizer, Context, Decision, Entities, Entity, EntityUid, PolicyId, PolicySet, Request,
    RestrictedExpression, Schema, SlotId, Template, ValidationMode, Validator,
};

const SCHEMA_SRC: &str = include_str!("../cedar/platform.cedarschema");
const POLICIES_SRC: &str = include_str!("../cedar/platform-policies.cedar");
const TEMPLATE_SRC: &str = include_str!("../cedar/platform-templates.cedar");

const TEMPLATE_ID: &str = "pbac-resource-read-grant";

fn schema() -> Schema {
    let (schema, _warnings) =
        Schema::from_cedarschema_str(SCHEMA_SRC).expect("platform.cedarschema must parse");
    schema
}

fn policy_set() -> PolicySet {
    let mut set = PolicySet::from_str(POLICIES_SRC).expect("platform-policies.cedar must parse");
    let template = Template::parse(Some(PolicyId::new(TEMPLATE_ID)), TEMPLATE_SRC)
        .expect("platform-templates.cedar must parse");
    set.add_template(template).expect("template must register");
    set
}

fn string_entity(uid: &str, attrs: &[(&str, &str)], parents: &[&str]) -> Entity {
    let uid = EntityUid::from_str(uid).expect("entity uid must parse");
    let attrs: HashMap<String, RestrictedExpression> = attrs
        .iter()
        .map(|(k, v)| {
            (
                (*k).to_owned(),
                RestrictedExpression::new_string((*v).to_owned()),
            )
        })
        .collect();
    let parents: HashSet<EntityUid> = parents
        .iter()
        .map(|p| EntityUid::from_str(p).expect("parent uid must parse"))
        .collect();
    Entity::new(uid, attrs, parents).expect("entity must build")
}

/// Two tenants in two cells; one cross-tenant-polluted admin group (the exact
/// misconfiguration the structural forbid must neutralize).
fn entities(schema: &Schema) -> Entities {
    let all = vec![
        string_entity(
            r#"OyaPlatform::Tenant::"acme""#,
            &[
                ("tenant_id", "acme"),
                ("cell_id", "cell-001"),
                ("lifecycle_state", "active"),
            ],
            &[],
        ),
        string_entity(
            r#"OyaPlatform::Tenant::"globex""#,
            &[
                ("tenant_id", "globex"),
                ("cell_id", "cell-002"),
                ("lifecycle_state", "active"),
            ],
            &[],
        ),
        string_entity(
            r#"OyaPlatform::Group::"tenant-admins""#,
            &[("tenant_id", "acme")],
            &[r#"OyaPlatform::Tenant::"acme""#],
        ),
        string_entity(
            r#"OyaPlatform::Principal::"alice""#,
            &[
                ("tenant_id", "acme"),
                ("kind", "human"),
                ("step_up_class", "a"),
            ],
            &[r#"OyaPlatform::Group::"tenant-admins""#],
        ),
        // mallory belongs to ANOTHER tenant but is (mis)joined to the same
        // group — the structural forbid must still deny everything.
        string_entity(
            r#"OyaPlatform::Principal::"mallory""#,
            &[
                ("tenant_id", "globex"),
                ("kind", "human"),
                ("step_up_class", "a"),
            ],
            &[r#"OyaPlatform::Group::"tenant-admins""#],
        ),
        string_entity(
            r#"OyaPlatform::WorkloadIdentity::"payments""#,
            &[
                ("tenant_id", "acme"),
                ("spiffe_id", "spiffe://oyatie/acme/payments"),
            ],
            &[r#"OyaPlatform::Tenant::"acme""#],
        ),
        string_entity(
            r#"OyaPlatform::TenantResource::"acme-doc-1""#,
            &[
                ("tenant_id", "acme"),
                ("resource_kind", "document"),
                ("data_class", "restricted"),
                ("cell_id", "cell-001"),
            ],
            &[r#"OyaPlatform::Tenant::"acme""#],
        ),
        // Same tenant and cell as `acme-doc-1`, but NOT restricted. The PBAC template link is
        // exercised against this resource so the assertion isolates template semantics: on a
        // restricted resource the step-up forbid fires first, so the outcome there says nothing
        // about whether the template granted anything.
        string_entity(
            r#"OyaPlatform::TenantResource::"acme-doc-2""#,
            &[
                ("tenant_id", "acme"),
                ("resource_kind", "document"),
                ("data_class", "internal"),
                ("cell_id", "cell-001"),
            ],
            &[r#"OyaPlatform::Tenant::"acme""#],
        ),
    ];
    Entities::from_entities(all, Some(schema)).expect("entities must validate against the schema")
}

fn decide(set: &PolicySet, principal: &str, action: &str, resource: &str) -> Decision {
    let schema = schema();
    let request = Request::new(
        EntityUid::from_str(principal).expect("principal uid"),
        EntityUid::from_str(action).expect("action uid"),
        EntityUid::from_str(resource).expect("resource uid"),
        Context::empty(),
        Some(&schema),
    )
    .expect("request must satisfy the schema");
    Authorizer::new()
        .is_authorized(&request, set, &entities(&schema))
        .decision()
}

#[test]
fn schema_parses_and_policy_set_strict_validates() {
    let result = Validator::new(schema()).validate(&policy_set(), ValidationMode::Strict);
    assert!(
        result.validation_passed(),
        "strict validation failed: {:?}",
        result.validation_errors().collect::<Vec<_>>()
    );
}

#[test]
fn abac_step_up_class_gates_restricted_reads() {
    let set = policy_set();
    // alice asserted step-up class "a" and shares the resource's tenant.
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::Principal::"alice""#,
            r#"OyaPlatform::Action::"ReadResource""#,
            r#"OyaPlatform::TenantResource::"acme-doc-1""#,
        ),
        Decision::Allow
    );
    // The workload has no step_up_class attribute: deny-by-default.
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::WorkloadIdentity::"payments""#,
            r#"OyaPlatform::Action::"ReadResource""#,
            r#"OyaPlatform::TenantResource::"acme-doc-1""#,
        ),
        Decision::Deny
    );
}

#[test]
fn rbac_group_grant_is_tenant_scoped() {
    let set = policy_set();
    // alice (tenant-admins, tenant acme) may administer acme...
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::Principal::"alice""#,
            r#"OyaPlatform::Action::"AdministerTenant""#,
            r#"OyaPlatform::Tenant::"acme""#,
        ),
        Decision::Allow
    );
    // ...but not globex, despite group membership.
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::Principal::"alice""#,
            r#"OyaPlatform::Action::"AdministerTenant""#,
            r#"OyaPlatform::Tenant::"globex""#,
        ),
        Decision::Deny
    );
}

#[test]
fn structural_forbid_overrides_group_membership_cross_tenant() {
    let set = policy_set();
    // mallory is IN the tenant-admins group but belongs to tenant globex:
    // the structural forbid wins over the RBAC permit.
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::Principal::"mallory""#,
            r#"OyaPlatform::Action::"AdministerTenant""#,
            r#"OyaPlatform::Tenant::"acme""#,
        ),
        Decision::Deny
    );
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::Principal::"mallory""#,
            r#"OyaPlatform::Action::"ReadResource""#,
            r#"OyaPlatform::TenantResource::"acme-doc-1""#,
        ),
        Decision::Deny
    );
}

fn link(set: &mut PolicySet, link_id: &str, principal: &str, resource: &str) {
    let mut values = HashMap::new();
    values.insert(
        SlotId::principal(),
        EntityUid::from_str(principal).expect("slot principal uid"),
    );
    values.insert(
        SlotId::resource(),
        EntityUid::from_str(resource).expect("slot resource uid"),
    );
    set.link(PolicyId::new(TEMPLATE_ID), PolicyId::new(link_id), values)
        .expect("template link must succeed");
}

/// A template link grants exactly the scoped read it names — and nothing the forbids withhold.
///
/// This previously linked `acme-doc-1`, whose `data_class` is `restricted`, and asserted Allow.
/// That can never hold: `forbid-restricted-read-without-step-up` denies every restricted read by
/// a principal lacking `step_up_class == "a"`, and the `payments` workload has no such attribute.
/// Cedar is forbid-overrides-permit, so the link was irrelevant to the outcome and the assertion
/// contradicted a gate the seed documents as unconditional. The fix belongs in the test, not the
/// policy: exercise the template on a non-restricted resource, then re-assert that the very same
/// link still cannot reach the restricted one.
#[test]
fn pbac_template_link_grants_scoped_read() {
    let mut set = policy_set();
    // Before linking: the workload cannot read (deny-by-default).
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::WorkloadIdentity::"payments""#,
            r#"OyaPlatform::Action::"ReadResource""#,
            r#"OyaPlatform::TenantResource::"acme-doc-2""#,
        ),
        Decision::Deny
    );
    link(
        &mut set,
        "pbac-link-payments-doc2",
        r#"OyaPlatform::WorkloadIdentity::"payments""#,
        r#"OyaPlatform::TenantResource::"acme-doc-2""#,
    );
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::WorkloadIdentity::"payments""#,
            r#"OyaPlatform::Action::"ReadResource""#,
            r#"OyaPlatform::TenantResource::"acme-doc-2""#,
        ),
        Decision::Allow
    );

    // The grant is scoped to the linked resource: a sibling in the same tenant and cell stays
    // denied because no link names it.
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::WorkloadIdentity::"payments""#,
            r#"OyaPlatform::Action::"ReadResource""#,
            r#"OyaPlatform::TenantResource::"acme-doc-1""#,
        ),
        Decision::Deny
    );

    // Forbid-overrides-permit holds across template links: even an explicit link to the
    // restricted resource cannot defeat the step-up gate.
    link(
        &mut set,
        "pbac-link-payments-doc1",
        r#"OyaPlatform::WorkloadIdentity::"payments""#,
        r#"OyaPlatform::TenantResource::"acme-doc-1""#,
    );
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::WorkloadIdentity::"payments""#,
            r#"OyaPlatform::Action::"ReadResource""#,
            r#"OyaPlatform::TenantResource::"acme-doc-1""#,
        ),
        Decision::Deny
    );
}

#[test]
fn pbac_template_link_cannot_defeat_structural_isolation() {
    let mut set = policy_set();
    // Even an explicit (mis-issued) cross-tenant link stays denied: forbid
    // overrides permit unconditionally.
    link(
        &mut set,
        "pbac-link-mallory-doc1",
        r#"OyaPlatform::Principal::"mallory""#,
        r#"OyaPlatform::TenantResource::"acme-doc-1""#,
    );
    assert_eq!(
        decide(
            &set,
            r#"OyaPlatform::Principal::"mallory""#,
            r#"OyaPlatform::Action::"ReadResource""#,
            r#"OyaPlatform::TenantResource::"acme-doc-1""#,
        ),
        Decision::Deny
    );
}
