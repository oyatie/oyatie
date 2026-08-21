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

use data_ontology_kernel::{
    ActionTypeDefinition, ActionTypeId, AutonomyTier, EntityTypeDefinition, EntityTypeId,
    EntityTypePropertyDefinition, LinkCardinality, LinkTypeDefinition, LinkTypeId, OntologyEngine,
    OntologyEngineError, OntologyPillar, PropertyTier,
};
use data_boundary_kernel::{DataClass, PrivacyDataClass};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn prop(name: &str) -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new(
        name,
        PropertyTier::Scalar,
        PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
        true,
    )
    .unwrap()
}

fn entity(tenant: &str, id: &str, display: &str) -> EntityTypeDefinition {
    EntityTypeDefinition::new(
        tenant,
        EntityTypeId::new(id).unwrap(),
        display,
        vec![prop("name")],
        1,
    )
    .unwrap()
}

fn entity_with_pillar(
    tenant: &str,
    id: &str,
    display: &str,
    pillar: OntologyPillar,
) -> EntityTypeDefinition {
    entity(tenant, id, display).with_pillar(pillar)
}

// ---------------------------------------------------------------------------
// st1 – cross-tenant endpoint isolation
// ---------------------------------------------------------------------------

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

/// After a successful registration the action type must be retrievable via the
/// `action_type` accessor using the same tenant and id.
#[test]
fn registered_action_type_is_queryable_via_accessor() {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(entity("ten_finance", "ety_invoice", "Invoice"))
        .unwrap();

    let action_id = engine
        .register_action_type(
            ActionTypeDefinition::new(
                "ten_finance",
                ActionTypeId::new("aty_approve_invoice").unwrap(),
                EntityTypeId::new("ety_invoice").unwrap(),
                "ontology.action.approve_invoice",
                AutonomyTier::T2ExecuteWithApproval,
                "EVT-INVOICE-APPROVED",
            )
            .unwrap(),
        )
        .unwrap();

    // RED: `action_type` accessor does not exist yet.
    let stored = engine
        .action_type("ten_finance", &action_id)
        .expect("registered action type must be queryable");

    assert_eq!(stored.id.value, "aty_approve_invoice");
    assert_eq!(
        stored.max_autonomy_tier,
        AutonomyTier::T2ExecuteWithApproval
    );
    assert_eq!(stored.audit_event_type, "EVT-INVOICE-APPROVED");
}

/// `link_type` returns `None` for an id that was never registered in that tenant.
#[test]
fn link_type_returns_none_for_unregistered_id() {
    let engine = OntologyEngine::default();
    let unknown_id = LinkTypeId::new("lty_phantom").unwrap();

    // RED: `link_type` accessor does not exist yet.
    assert!(
        engine.link_type("ten_x", &unknown_id).is_none(),
        "link_type must return None for an unregistered id"
    );
}

/// `action_type` returns `None` for an id that was never registered in that tenant.
#[test]
fn action_type_returns_none_for_unregistered_id() {
    let engine = OntologyEngine::default();
    let unknown_id = ActionTypeId::new("aty_phantom").unwrap();

    // RED: `action_type` accessor does not exist yet.
    assert!(
        engine.action_type("ten_x", &unknown_id).is_none(),
        "action_type must return None for an unregistered id"
    );
}

// ---------------------------------------------------------------------------
// st2 – pillar-consistency: action types are pillar-agnostic
// ---------------------------------------------------------------------------

/// An action type bound to a person-pillar entity type must be accepted; the
/// pillar constraint is only enforced on link types (org/person boundary).
/// Also verifies the action_type accessor returns the correct pillar annotation
/// on the backing entity type — exercises both the accessor and pillar pass-through.
#[test]
fn action_type_on_person_pillar_entity_accepted_and_queryable() {
    let mut engine = OntologyEngine::default();

    engine
        .register_entity_type(entity_with_pillar(
            "ten_hr",
            "ety_employee",
            "Employee",
            OntologyPillar::Person,
        ))
        .unwrap();

    let action_id = engine
        .register_action_type(
            ActionTypeDefinition::new(
                "ten_hr",
                ActionTypeId::new("aty_onboard_employee").unwrap(),
                EntityTypeId::new("ety_employee").unwrap(),
                "ontology.action.onboard_employee",
                AutonomyTier::T0Suggest,
                "EVT-EMPLOYEE-ONBOARDED",
            )
            .unwrap(),
        )
        .expect("action type on person-pillar entity must be accepted");

    // RED: `action_type` accessor does not exist yet.
    let stored = engine
        .action_type("ten_hr", &action_id)
        .expect("action type must be queryable after registration");

    assert_eq!(stored.entity_type.value, "ety_employee");

    // Verify the underlying entity type carries the person pillar.
    let entity_def = engine
        .entity_type("ten_hr", &EntityTypeId::new("ety_employee").unwrap())
        .expect("entity type must be registered");
    assert_eq!(entity_def.pillar, Some(OntologyPillar::Person));
}
