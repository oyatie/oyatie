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
