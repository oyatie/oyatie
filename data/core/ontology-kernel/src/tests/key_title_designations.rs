//! Primary-key and title-property designations: each must name a declared
//! property, a key property must be required, and a set key is immutable
//! across revisions.

use crate::*;
use data_boundary_kernel::{DataClass, PrivacyDataClass};

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

fn prop(name: &str, required: bool) -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new(name, PropertyTier::Scalar, internal(), required).unwrap()
}

fn def(revision: u32) -> EntityTypeDefinition {
    EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_account").unwrap(),
        "Account",
        vec![prop("account_number", true), prop("nickname", false)],
        revision,
    )
    .unwrap()
}

/// The conformant fast path: key on a declared required property, title on
/// a declared property.
#[test]
fn valid_designations_register() {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(
            def(1)
                .with_primary_key_property("account_number")
                .with_title_property("nickname"),
        )
        .expect("valid designations must register");
}

/// A designation naming no declared property is rejected.
#[test]
fn undeclared_designation_rejected() {
    let mut engine = OntologyEngine::default();
    assert_eq!(
        engine.register_entity_type(def(1).with_primary_key_property("ghost")),
        Err(OntologyEngineError::DesignatedPropertyNotDeclared {
            name: "ghost".into()
        })
    );
    assert_eq!(
        engine.register_entity_type(def(1).with_title_property("ghost")),
        Err(OntologyEngineError::DesignatedPropertyNotDeclared {
            name: "ghost".into()
        })
    );
}

/// A primary key on an optional property is a contradiction: conformant
/// instances may omit it.
#[test]
fn optional_primary_key_rejected() {
    let mut engine = OntologyEngine::default();
    assert_eq!(
        engine.register_entity_type(def(1).with_primary_key_property("nickname")),
        Err(OntologyEngineError::PrimaryKeyPropertyNotRequired {
            name: "nickname".into()
        })
    );
}

/// Designation integrity holds on the evolve path too, both branches.
#[test]
fn evolve_first_registration_checks_designations() {
    let mut engine = OntologyEngine::default();
    assert_eq!(
        engine.evolve_entity_type(def(1).with_primary_key_property("ghost")),
        Err(OntologyEngineError::DesignatedPropertyNotDeclared {
            name: "ghost".into()
        })
    );
}

/// A set primary key is immutable: changing or removing it re-keys the
/// population. Adoption (None -> Some) stays open.
#[test]
fn primary_key_immutable_once_set() {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(def(1).with_primary_key_property("account_number"))
        .unwrap();

    // Change to another declared required property: rejected.
    let mut with_other_key = def(2).with_primary_key_property("other_id");
    with_other_key.properties.push(prop("other_id", true));
    assert_eq!(
        engine.evolve_entity_type(with_other_key),
        Err(OntologyEngineError::PrimaryKeyChangedOnEvolution)
    );

    // Removal: rejected.
    assert_eq!(
        engine.evolve_entity_type(def(2)),
        Err(OntologyEngineError::PrimaryKeyChangedOnEvolution)
    );

    // Unchanged key evolves freely.
    engine
        .evolve_entity_type(def(2).with_primary_key_property("account_number"))
        .expect("unchanged key must evolve");
}

/// Adopting a key where none was set is allowed, and the title designation
/// may change freely.
#[test]
fn key_adoption_and_title_change_allowed() {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(def(1).with_title_property("nickname"))
        .unwrap();

    engine
        .evolve_entity_type(
            def(2)
                .with_primary_key_property("account_number")
                .with_title_property("account_number"),
        )
        .expect("key adoption and title change must be additive");
}
