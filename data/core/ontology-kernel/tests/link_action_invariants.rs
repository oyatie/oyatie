// ADR-0083 Tier 3: integration tests use `.expect()` / `.unwrap()` to assert
// invariant setup; these are intentional under the cfg(test) exemption.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! RED tests for the ontology-kernel-link-action-validation slice.
//!
//! Acceptance criteria exercised here (subtasks st1 / st2):
//!
//! st1 – endpoint-reference validation
//!  • link_type_with_dangling_from_endpoint_cross_tenant_rejected
//!  • link_type_with_dangling_to_endpoint_cross_tenant_rejected
//!  • action_type_with_dangling_entity_type_cross_tenant_rejected
//!  • self_referential_link_type_with_both_endpoints_registered_accepted
//!
//! st2 – pillar-consistency + cardinality accessor surface
//!  • registered_link_type_is_queryable_via_accessor
//!  • registered_action_type_is_queryable_via_accessor
//!  • link_type_returns_none_for_unregistered_id
//!  • action_type_returns_none_for_unregistered_id
//!
//! These tests reference `OntologyEngine::link_type` and
//! `OntologyEngine::action_type` query accessors that do not yet exist on the
//! public API, ensuring they fail (RED) until the accessor methods are added.

#[path = "link_action_invariants_support.rs"]
mod support;
use support::*;

/// A link type in ten_b must not accept an EntityTypeId registered only in ten_a.
/// Endpoint resolution is scoped per-tenant; the registration must be rejected
/// with UnknownEntityTypeEndpoint even though the EntityTypeId value matches.
#[test]
fn link_type_with_dangling_from_endpoint_cross_tenant_rejected() {
    let mut engine = OntologyEngine::default();

    // Register "ety_node" only under ten_a.
    engine
        .register_entity_type(entity("ten_a", "ety_node", "Node"))
        .unwrap();

    // Register "ety_edge" under ten_b.
    engine
        .register_entity_type(entity("ten_b", "ety_edge", "Edge"))
        .unwrap();

    // Link type is in ten_b but references "ety_node" which is only in ten_a.
    let link = LinkTypeDefinition::new(
        "ten_b",
        LinkTypeId::new("lty_node_edge").unwrap(),
        EntityTypeId::new("ety_node").unwrap(), // only registered in ten_a
        EntityTypeId::new("ety_edge").unwrap(),
        LinkCardinality::OneToMany,
        false,
    )
    .unwrap();

    assert_eq!(
        engine.register_link_type(link),
        Err(OntologyEngineError::UnknownEntityTypeEndpoint),
        "cross-tenant from-endpoint must be rejected"
    );
}

/// A link type in ten_b must not accept a to-EntityTypeId registered only in ten_a.
#[test]
fn link_type_with_dangling_to_endpoint_cross_tenant_rejected() {
    let mut engine = OntologyEngine::default();

    // Register "ety_node" under ten_b (from-endpoint is fine).
    engine
        .register_entity_type(entity("ten_b", "ety_node", "Node"))
        .unwrap();

    // Register "ety_edge" only under ten_a (to-endpoint is missing in ten_b).
    engine
        .register_entity_type(entity("ten_a", "ety_edge", "Edge"))
        .unwrap();

    let link = LinkTypeDefinition::new(
        "ten_b",
        LinkTypeId::new("lty_node_edge").unwrap(),
        EntityTypeId::new("ety_node").unwrap(),
        EntityTypeId::new("ety_edge").unwrap(), // only registered in ten_a
        LinkCardinality::ManyToMany,
        false,
    )
    .unwrap();

    assert_eq!(
        engine.register_link_type(link),
        Err(OntologyEngineError::UnknownEntityTypeEndpoint),
        "cross-tenant to-endpoint must be rejected"
    );
}

/// An action type in ten_b must not accept an EntityTypeId registered only in ten_a.
#[test]
fn action_type_with_dangling_entity_type_cross_tenant_rejected() {
    let mut engine = OntologyEngine::default();

    // Register entity only in ten_a.
    engine
        .register_entity_type(entity("ten_a", "ety_invoice", "Invoice"))
        .unwrap();

    let action = ActionTypeDefinition::new(
        "ten_b",
        ActionTypeId::new("aty_send_invoice").unwrap(),
        EntityTypeId::new("ety_invoice").unwrap(), // only in ten_a
        "ontology.action.send_invoice",
        AutonomyTier::T1Assist,
        "EVT-INVOICE-SENT",
    )
    .unwrap();

    assert_eq!(
        engine.register_action_type(action),
        Err(OntologyEngineError::UnknownEntityTypeEndpoint),
        "cross-tenant action entity-type endpoint must be rejected"
    );
}

/// A link type where from_entity_type == to_entity_type (self-referential) is
/// structurally valid as long as both references resolve to the same registered
/// entity type within the tenant.
#[test]
fn self_referential_link_type_with_both_endpoints_registered_accepted() {
    let mut engine = OntologyEngine::default();

    engine
        .register_entity_type(entity("ten_org", "ety_employee", "Employee"))
        .unwrap();

    let link = LinkTypeDefinition::new(
        "ten_org",
        LinkTypeId::new("lty_reports_to").unwrap(),
        EntityTypeId::new("ety_employee").unwrap(),
        EntityTypeId::new("ety_employee").unwrap(), // same type on both sides
        LinkCardinality::OneToOne,
        false,
    )
    .unwrap();

    assert!(
        engine.register_link_type(link).is_ok(),
        "self-referential link type must be accepted when entity type is registered"
    );
}

// ---------------------------------------------------------------------------
// st1/st2 – query accessor surface (RED: methods don't exist yet)
//
// `OntologyEngine::link_type(tenant_id, id)` and
// `OntologyEngine::action_type(tenant_id, id)` mirror the existing
// `entity_type` accessor and are required for callers to inspect registered
// link/action types after registration.
// ---------------------------------------------------------------------------

/// After a successful registration the link type must be retrievable via the
/// `link_type` accessor using the same tenant and id.
#[test]
fn registered_link_type_is_queryable_via_accessor() {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(entity("ten_retail", "ety_product", "Product"))
        .unwrap();
    engine
        .register_entity_type(entity("ten_retail", "ety_category", "Category"))
        .unwrap();

    let link_id = engine
        .register_link_type(
            LinkTypeDefinition::new(
                "ten_retail",
                LinkTypeId::new("lty_product_category").unwrap(),
                EntityTypeId::new("ety_product").unwrap(),
                EntityTypeId::new("ety_category").unwrap(),
                LinkCardinality::ManyToMany,
                false,
            )
            .unwrap(),
        )
        .unwrap();

    // RED: `link_type` accessor does not exist yet.
    let stored = engine
        .link_type("ten_retail", &link_id)
        .expect("registered link type must be queryable");

    assert_eq!(stored.id.value, "lty_product_category");
    assert_eq!(stored.cardinality, LinkCardinality::ManyToMany);
    assert!(!stored.allow_cross_tenant);
}
