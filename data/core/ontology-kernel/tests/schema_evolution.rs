// ADR-0083 Tier 3: integration tests use `.expect()` / `.unwrap()` to assert
// invariant setup; these are intentional under the cfg(test) exemption.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! RED tests for the ontology-kernel schema-evolution slice.
//!
//! Acceptance criteria exercised here (subtasks ST1 / ST2):
//!
//! ST1 – backward-compatibility checker over property sets:
//!  • additive_new_property_with_higher_revision_is_accepted
//!  • tier_mutation_on_existing_property_rejected_with_incompatible
//!  • data_class_mutation_on_existing_property_rejected_with_incompatible
//!  • required_flag_flip_on_existing_property_rejected_with_incompatible
//!  • property_removal_rejected_with_incompatible
//!  • multiple_mutations_all_rejected_with_incompatible
//!
//! ST2 – OntologyEngine::evolve_entity_type:
//!  • first_registration_via_evolve_inserts_and_returns_id
//!  • monotonic_additive_evolution_accepted_updates_stored_revision
//!  • equal_revision_rejected_with_non_monotonic_revision
//!  • lower_revision_rejected_with_non_monotonic_revision
//!  • breaking_change_higher_revision_rejected_with_incompatible
//!  • stored_definition_unchanged_after_rejected_evolution
//!  • evolve_does_not_return_duplicate_entity_type_error_on_second_call
//!  • tenant_isolation_evolve_does_not_see_other_tenant_registration
//!
//! Schema evolution precedents honoured:
//!  • Protobuf field-add / reader-writer Avro compatibility: additive-only.
//!  • Confluent Schema Registry FORWARD/BACKWARD compat: field removal forbidden.
//!  • Monotonic schema-version gating (Confluent compatibility level enforcement).

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, OntologyEngine,
    OntologyEngineError, PropertyTier,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

fn pii() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::PiiIdentifying).unwrap()
}

fn quasi() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::PiiQuasiIdentifier).unwrap()
}

/// Build a named property definition with explicit tier/data_class/required.
fn prop(
    name: &str,
    tier: PropertyTier,
    data_class: PrivacyDataClass,
    required: bool,
) -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new(name, tier, data_class, required).unwrap()
}

/// Build an `EntityTypeDefinition` for tenant `ten_test` / id `ety_thing` at
/// `revision` with `name:Scalar/InternalOnly/required=true` as the base
/// property, plus any `extra_props`.
fn base_def(revision: u32, extra_props: Vec<EntityTypePropertyDefinition>) -> EntityTypeDefinition {
    let mut props = vec![prop("name", PropertyTier::Scalar, internal(), true)];
    props.extend(extra_props);
    EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_thing").unwrap(),
        "Thing",
        props,
        revision,
    )
    .unwrap()
}

/// Build a definition for a different tenant so cross-tenant isolation tests
/// have a clean second registration.
fn other_tenant_def(revision: u32) -> EntityTypeDefinition {
    EntityTypeDefinition::new(
        "ten_other",
        EntityTypeId::new("ety_thing").unwrap(),
        "Thing",
        vec![prop("name", PropertyTier::Scalar, internal(), true)],
        revision,
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// ST1 – additive-only backward-compatibility checker
// ---------------------------------------------------------------------------

/// Adding a new property to a definition whose existing properties are all
/// retained unchanged is a valid additive (backward-compatible) evolution.
/// This mirrors Protobuf field-add and Avro reader schema extension.
#[test]
fn additive_new_property_with_higher_revision_is_accepted() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    let v2 = base_def(
        2,
        vec![prop("email", PropertyTier::Scalar, pii(), false)],
    );
    let id = engine
        .evolve_entity_type(v2)
        .expect("additive evolution with higher revision must be accepted");

    let stored = engine.entity_type("ten_test", &id).unwrap();
    assert_eq!(stored.revision, 2, "stored revision must advance to 2");
    assert_eq!(stored.properties.len(), 2, "both properties must be present");
    assert!(
        stored.properties.iter().any(|p| p.name == "email"),
        "new 'email' property must be persisted"
    );
}

/// Adding multiple new properties in a single evolution step is still additive.
#[test]
fn multiple_additive_properties_in_single_evolution_accepted() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    let v2 = base_def(
        2,
        vec![
            prop("email", PropertyTier::Scalar, pii(), false),
            prop("embedding", PropertyTier::Vector, quasi(), false),
            prop("last_seen", PropertyTier::Timeseries, internal(), false),
        ],
    );
    let id = engine
        .evolve_entity_type(v2)
        .expect("multi-property additive evolution must be accepted");

    let stored = engine.entity_type("ten_test", &id).unwrap();
    assert_eq!(stored.revision, 2);
    assert_eq!(stored.properties.len(), 4, "base + 3 new properties");
}

/// Changing the `PropertyTier` of an existing property is a breaking schema
/// change — equivalent to changing a Protobuf field type. Must be rejected with
/// `IncompatibleSchemaEvolution`.
#[test]
fn tier_mutation_on_existing_property_rejected_with_incompatible() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    // Mutate "name": Scalar → Vector (breaking).
    let breaking = EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_thing").unwrap(),
        "Thing",
        vec![prop("name", PropertyTier::Vector, internal(), true)],
        2,
    )
    .unwrap();

    assert_eq!(
        engine.evolve_entity_type(breaking),
        Err(OntologyEngineError::IncompatibleSchemaEvolution),
        "tier mutation must be rejected"
    );
}

/// Changing the `data_class` of an existing property violates the privacy
/// contract — equivalent to tightening/loosening Avro field schema. Must be
/// rejected with `IncompatibleSchemaEvolution`.
#[test]
fn data_class_mutation_on_existing_property_rejected_with_incompatible() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    // Mutate "name": InternalOnly → PiiIdentifying (breaking privacy contract).
    let breaking = EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_thing").unwrap(),
        "Thing",
        vec![prop("name", PropertyTier::Scalar, pii(), true)],
        2,
    )
    .unwrap();

    assert_eq!(
        engine.evolve_entity_type(breaking),
        Err(OntologyEngineError::IncompatibleSchemaEvolution),
        "data_class mutation must be rejected"
    );
}

/// Flipping the `required` flag of an existing property is a breaking change
/// (consumers relying on presence will be surprised by absence). Must be
/// rejected with `IncompatibleSchemaEvolution`.
#[test]
fn required_flag_flip_on_existing_property_rejected_with_incompatible() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    // Flip "name" required: true → false.
    let breaking = EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_thing").unwrap(),
        "Thing",
        vec![prop("name", PropertyTier::Scalar, internal(), false)],
        2,
    )
    .unwrap();

    assert_eq!(
        engine.evolve_entity_type(breaking),
        Err(OntologyEngineError::IncompatibleSchemaEvolution),
        "required flag flip must be rejected"
    );
}

/// Removing an existing property is the canonical breaking change in both
/// Protobuf and Avro schema evolution. Must be rejected with
/// `IncompatibleSchemaEvolution`.
#[test]
fn property_removal_rejected_with_incompatible() {
    let mut engine = OntologyEngine::default();
    // v1 has "name" + "code".
    engine
        .evolve_entity_type(base_def(
            1,
            vec![prop("code", PropertyTier::Scalar, internal(), true)],
        ))
        .unwrap();

    // v2 drops "code" — backward-incompatible.
    let dropping = base_def(2, vec![]);

    assert_eq!(
        engine.evolve_entity_type(dropping),
        Err(OntologyEngineError::IncompatibleSchemaEvolution),
        "property removal must be rejected"
    );
}

/// If multiple existing properties are mutated simultaneously the checker must
/// still return `IncompatibleSchemaEvolution` (first violation is sufficient).
#[test]
fn multiple_mutations_all_rejected_with_incompatible() {
    let mut engine = OntologyEngine::default();
    engine
        .evolve_entity_type(base_def(
            1,
            vec![prop("code", PropertyTier::Scalar, internal(), true)],
        ))
        .unwrap();

    // Mutate both "name" (tier) and "code" (data_class).
    let breaking = EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_thing").unwrap(),
        "Thing",
        vec![
            prop("name", PropertyTier::Vector, internal(), true), // tier changed
            prop("code", PropertyTier::Scalar, pii(), true),      // data_class changed
        ],
        2,
    )
    .unwrap();

    assert_eq!(
        engine.evolve_entity_type(breaking),
        Err(OntologyEngineError::IncompatibleSchemaEvolution),
        "multiple simultaneous mutations must be rejected"
    );
}

// ---------------------------------------------------------------------------
// ST2 – OntologyEngine::evolve_entity_type
// ---------------------------------------------------------------------------

/// First call to `evolve_entity_type` with an unknown id behaves like
/// `register_entity_type`: inserts the definition and returns the id.
/// `DuplicateEntityType` is never surfaced from `evolve_entity_type`.
#[test]
fn first_registration_via_evolve_inserts_and_returns_id() {
    let mut engine = OntologyEngine::default();
    let def = base_def(1, vec![]);
    let id = engine
        .evolve_entity_type(def)
        .expect("first evolve call must succeed as first registration");

    assert_eq!(id.value, "ety_thing");
    assert!(
        engine.entity_type("ten_test", &id).is_some(),
        "definition must be retrievable after first-registration evolve"
    );
}

/// A second call with a strictly higher revision and only additive property
/// changes must succeed, replace the stored definition, and reflect the new
/// revision and property set.
#[test]
fn monotonic_additive_evolution_accepted_updates_stored_revision() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    let v2 = base_def(
        2,
        vec![prop("email", PropertyTier::Scalar, pii(), false)],
    );
    let id = engine
        .evolve_entity_type(v2)
        .expect("monotonic additive evolution must be accepted");

    let stored = engine.entity_type("ten_test", &id).unwrap();
    assert_eq!(stored.revision, 2, "stored revision must be 2 after evolution");
    assert_eq!(
        stored.properties.len(),
        2,
        "stored definition must contain both base and new property"
    );
    assert!(
        stored.properties.iter().any(|p| p.name == "email"),
        "new property 'email' must be persisted in stored definition"
    );
}

/// A large revision jump (e.g. 1 → 100) is permitted by strict-monotonicity;
/// only ordering is enforced, not continuity.
#[test]
fn large_revision_jump_accepted() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    let v100 = base_def(
        100,
        vec![prop("tag", PropertyTier::Scalar, internal(), false)],
    );
    let id = engine
        .evolve_entity_type(v100)
        .expect("large revision jump must be accepted if additive");

    let stored = engine.entity_type("ten_test", &id).unwrap();
    assert_eq!(stored.revision, 100);
}

/// Submitting a candidate with `revision == stored.revision` (same version)
/// must be rejected with `NonMonotonicRevision`. This mirrors Confluent Schema
/// Registry FORWARD compat: you cannot re-register the same version.
#[test]
fn equal_revision_rejected_with_non_monotonic_revision() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    assert_eq!(
        engine.evolve_entity_type(base_def(1, vec![])),
        Err(OntologyEngineError::NonMonotonicRevision),
        "equal revision must be rejected"
    );
}

/// Submitting a candidate with `revision < stored.revision` (downgrade) must
/// be rejected with `NonMonotonicRevision`.
#[test]
fn lower_revision_rejected_with_non_monotonic_revision() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(5, vec![])).unwrap();

    assert_eq!(
        engine.evolve_entity_type(base_def(3, vec![])),
        Err(OntologyEngineError::NonMonotonicRevision),
        "lower revision must be rejected"
    );
}

/// A breaking change (tier mutation) submitted with a higher revision must be
/// rejected with `IncompatibleSchemaEvolution`, not `NonMonotonicRevision`.
/// This confirms the compatibility check runs after the monotonicity gate.
#[test]
fn breaking_change_higher_revision_rejected_with_incompatible() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    let breaking = EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_thing").unwrap(),
        "Thing",
        vec![prop("name", PropertyTier::Vector, internal(), true)],
        2,
    )
    .unwrap();

    assert_eq!(
        engine.evolve_entity_type(breaking),
        Err(OntologyEngineError::IncompatibleSchemaEvolution),
        "breaking change with higher revision must yield IncompatibleSchemaEvolution"
    );
}

/// After a rejected evolution the stored definition must remain at its prior
/// revision and property set — atomic rollback on rejection.
#[test]
fn stored_definition_unchanged_after_rejected_evolution() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    // Attempt a breaking evolution (revision is higher but tier mutated).
    let breaking = EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_thing").unwrap(),
        "Thing",
        vec![prop("name", PropertyTier::Vector, internal(), true)],
        2,
    )
    .unwrap();
    let _ = engine.evolve_entity_type(breaking);

    // Stored definition must still be at revision 1 with the original property.
    let id = EntityTypeId::new("ety_thing").unwrap();
    let stored = engine
        .entity_type("ten_test", &id)
        .expect("definition must still be registered after rejection");
    assert_eq!(
        stored.revision, 1,
        "revision must not change after rejected evolution"
    );
    assert_eq!(
        stored.properties.len(),
        1,
        "property count must not change after rejected evolution"
    );
    assert_eq!(
        stored.properties[0].tier,
        PropertyTier::Scalar,
        "property tier must not change after rejected evolution"
    );
}

/// `evolve_entity_type` must never return `DuplicateEntityType`. A second
/// call with a higher revision and additive changes returns `Ok(id)`.
#[test]
fn evolve_does_not_return_duplicate_entity_type_error_on_second_call() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();

    let result = engine.evolve_entity_type(base_def(
        2,
        vec![prop("tag", PropertyTier::Scalar, internal(), false)],
    ));

    assert!(
        result != Err(OntologyEngineError::DuplicateEntityType),
        "evolve_entity_type must never return DuplicateEntityType"
    );
    assert!(result.is_ok(), "valid evolution must return Ok(id)");
}

/// Entity type registrations are scoped per-tenant. Registering `ety_thing`
/// under `ten_other` and then evolving it under `ten_test` must be treated as
/// a first registration for `ten_test` (no cross-tenant leakage).
#[test]
fn tenant_isolation_evolve_does_not_see_other_tenant_registration() {
    let mut engine = OntologyEngine::default();

    // Register ety_thing under ten_other at revision 5.
    engine.evolve_entity_type(other_tenant_def(5)).unwrap();

    // Evolve ety_thing under ten_test at revision 1 — must be a first registration.
    let id = engine
        .evolve_entity_type(base_def(1, vec![]))
        .expect("evolve under ten_test must not see ten_other's registration");

    let stored = engine.entity_type("ten_test", &id).unwrap();
    assert_eq!(
        stored.revision, 1,
        "ten_test definition must start at revision 1, not inherit ten_other's 5"
    );

    // ten_other definition must be unaffected.
    let other_id = EntityTypeId::new("ety_thing").unwrap();
    let other_stored = engine.entity_type("ten_other", &other_id).unwrap();
    assert_eq!(other_stored.revision, 5, "ten_other definition must be unchanged");
}

/// The revision field of the stored definition must be readable via the
/// `entity_type` accessor and reflect the latest successful evolution.
#[test]
fn entity_type_accessor_reflects_latest_revision_after_evolution() {
    let mut engine = OntologyEngine::default();
    engine.evolve_entity_type(base_def(1, vec![])).unwrap();
    engine
        .evolve_entity_type(base_def(
            2,
            vec![prop("code", PropertyTier::Scalar, internal(), false)],
        ))
        .unwrap();
    engine
        .evolve_entity_type(base_def(
            3,
            vec![
                prop("code", PropertyTier::Scalar, internal(), false),
                prop("tag", PropertyTier::Scalar, internal(), false),
            ],
        ))
        .unwrap();

    let id = EntityTypeId::new("ety_thing").unwrap();
    let stored = engine.entity_type("ten_test", &id).unwrap();
    assert_eq!(stored.revision, 3, "accessor must return latest revision");
    assert_eq!(stored.properties.len(), 3, "all three properties must be present");
}
