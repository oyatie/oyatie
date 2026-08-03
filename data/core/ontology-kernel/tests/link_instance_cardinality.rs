// ADR-0083 Tier 3: tests use `.unwrap()` / `.expect()` under the cfg(test) exemption.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! Acceptance tests for the ontology-kernel-link-instance-cardinality-enforcement slice.
//!
//! Exercises `OntologyEngine::register_link_instance` across all cardinality variants,
//! the unknown-link-type rejection, and idempotent re-insert behaviour.

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, LinkCardinality,
    LinkInstanceOutcome, LinkTypeDefinition, LinkTypeId, OntologyEngine, OntologyEngineError,
    PropertyTier,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

fn prop(name: &str) -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new(name, PropertyTier::Scalar, internal(), true).unwrap()
}

fn entity(tenant: &str, id: &str) -> EntityTypeDefinition {
    EntityTypeDefinition::new(
        tenant,
        EntityTypeId::new(id).unwrap(),
        id, // display_name equals id for brevity
        vec![prop("name")],
        1,
    )
    .unwrap()
}

/// Build an engine with one registered link type `lty_edge` (OneToOne by default).
/// Returns `(engine, link_type_id)`.
fn engine_with_link_type(cardinality: LinkCardinality) -> (OntologyEngine, LinkTypeId) {
    let mut engine = OntologyEngine::default();
    engine.register_entity_type(entity("ten_t", "ety_from")).unwrap();
    engine.register_entity_type(entity("ten_t", "ety_to")).unwrap();
    let link_id = engine
        .register_link_type(
            LinkTypeDefinition::new(
                "ten_t",
                LinkTypeId::new("lty_edge").unwrap(),
                EntityTypeId::new("ety_from").unwrap(),
                EntityTypeId::new("ety_to").unwrap(),
                cardinality,
                false,
            )
            .unwrap(),
        )
        .unwrap();
    (engine, link_id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Calling `register_link_instance` with a `LinkTypeId` that was never registered
/// for the tenant must return `UnknownLinkType`.
#[test]
fn unknown_link_type_rejected() {
    let mut engine = OntologyEngine::default();
    let phantom = LinkTypeId::new("lty_phantom").unwrap();

    assert_eq!(
        engine.register_link_instance("ten_t", &phantom, "ent_a", "ent_b"),
        Err(OntologyEngineError::UnknownLinkType),
        "unregistered link type must be rejected"
    );
}

/// OneToOne: inserting a second outbound edge from the same `from_entity_id`
/// (to a *different* to) must be rejected.
#[test]
fn one_to_one_second_from_rejected() {
    let (mut engine, link_id) = engine_with_link_type(LinkCardinality::OneToOne);

    // First edge: ok.
    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_a", "ent_b"),
        Ok(LinkInstanceOutcome::Registered)
    );

    // Second edge: same from, different to → violation.
    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_a", "ent_c"),
        Err(OntologyEngineError::CardinalityViolation {
            cardinality: LinkCardinality::OneToOne
        }),
        "OneToOne must reject a second outbound edge from the same from_entity_id"
    );
}

/// OneToOne: inserting a second inbound edge into the same `to_entity_id`
/// (from a *different* from) must be rejected.
#[test]
fn one_to_one_second_to_rejected() {
    let (mut engine, link_id) = engine_with_link_type(LinkCardinality::OneToOne);

    // First edge: ok.
    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_a", "ent_b"),
        Ok(LinkInstanceOutcome::Registered)
    );

    // Second edge: different from, same to → violation.
    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_c", "ent_b"),
        Err(OntologyEngineError::CardinalityViolation {
            cardinality: LinkCardinality::OneToOne
        }),
        "OneToOne must reject a second inbound edge into the same to_entity_id"
    );
}

/// OneToMany: fan-out from the same `from_entity_id` to multiple distinct `to_entity_id`s
/// must all succeed.
#[test]
fn one_to_many_fan_out_allowed() {
    let (mut engine, link_id) = engine_with_link_type(LinkCardinality::OneToMany);

    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_a", "ent_b"),
        Ok(LinkInstanceOutcome::Registered)
    );
    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_a", "ent_c"),
        Ok(LinkInstanceOutcome::Registered),
        "OneToMany must allow fan-out from the same from_entity_id"
    );
}

/// OneToMany: inserting a second inbound edge into the same `to_entity_id`
/// (from a different from) must be rejected.
#[test]
fn one_to_many_second_into_rejected() {
    let (mut engine, link_id) = engine_with_link_type(LinkCardinality::OneToMany);

    // First edge.
    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_a", "ent_b"),
        Ok(LinkInstanceOutcome::Registered)
    );

    // Different from, same to → inbound violation.
    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_c", "ent_b"),
        Err(OntologyEngineError::CardinalityViolation {
            cardinality: LinkCardinality::OneToMany
        }),
        "OneToMany must reject a second inbound edge into the same to_entity_id"
    );
}

/// ManyToMany: all combinations of from/to must be accepted with no violations.
#[test]
fn many_to_many_all_allowed() {
    let (mut engine, link_id) = engine_with_link_type(LinkCardinality::ManyToMany);

    let pairs = [
        ("ent_a", "ent_b"),
        ("ent_a", "ent_c"),
        ("ent_b", "ent_a"),
        ("ent_b", "ent_c"),
        ("ent_c", "ent_a"),
    ];

    for (from, to) in pairs {
        assert_eq!(
            engine.register_link_instance("ten_t", &link_id, from, to),
            Ok(LinkInstanceOutcome::Registered),
            "ManyToMany must allow all combinations; failed on ({from}, {to})"
        );
    }
}

/// Re-inserting the identical `(link_type_id, from_entity_id, to_entity_id)` tuple
/// must return `AlreadyExists` without mutation and without a cardinality error.
#[test]
fn idempotent_reinsert_returns_already_exists() {
    let (mut engine, link_id) = engine_with_link_type(LinkCardinality::OneToOne);

    // First insert.
    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_a", "ent_b"),
        Ok(LinkInstanceOutcome::Registered)
    );

    // Identical insert: must be idempotent.
    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_a", "ent_b"),
        Ok(LinkInstanceOutcome::AlreadyExists),
        "re-inserting the identical edge tuple must return AlreadyExists"
    );

    // OneToOne: a genuinely new edge from the same from must still be rejected (indices unchanged).
    assert_eq!(
        engine.register_link_instance("ten_t", &link_id, "ent_a", "ent_c"),
        Err(OntologyEngineError::CardinalityViolation {
            cardinality: LinkCardinality::OneToOne
        }),
        "cardinality must still be enforced after an idempotent re-insert"
    );
}
