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

fn composites() -> [ValueTypeDeclaration; 2] {
    [
        ValueTypeDeclaration::Array {
            element: Box::new(ValueTypeDeclaration::Scalar(ScalarType::String)),
        },
        ValueTypeDeclaration::Struct(StructSchema {
            fields: vec![StructFieldDeclaration {
                name: "code".into(),
                required: true,
                value_type: ValueTypeDeclaration::Scalar(ScalarType::Integer),
            }],
        }),
    ]
}

fn with_declaration(
    mut definition: EntityTypeDefinition,
    declaration: ValueTypeDeclaration,
) -> EntityTypeDefinition {
    definition.properties[0].tier = declaration.tier();
    definition.properties[0].value_type = Some(declaration);
    definition
}

#[test]
fn composite_keys_refuse_registration_and_evolution_without_changing_current_or_history() {
    for declaration in composites() {
        let candidate =
            with_declaration(def(1), declaration).with_primary_key_property("account_number");
        for evolve in [false, true] {
            let mut engine = OntologyEngine::default();
            let before = engine.clone();
            let result = if evolve {
                engine.evolve_entity_type(candidate.clone())
            } else {
                engine.register_entity_type(candidate.clone())
            };
            assert_eq!(
                result,
                Err(OntologyEngineError::PrimaryKeyPropertyNotScalar {
                    name: "account_number".into()
                })
            );
            assert_eq!(
                engine, before,
                "refusal must retain neither current definition nor revision"
            );
        }
        let mut prior = candidate.clone();
        prior.primary_key_property = None;
        prior.title_property = Some("account_number".into());
        let mut engine = OntologyEngine::default();
        engine
            .register_entity_type(prior)
            .expect("composite non-key and title remain valid");
        let before = engine.clone();
        let mut next = candidate.clone();
        next.revision = 2;
        assert_eq!(
            engine.evolve_entity_type(next),
            Err(OntologyEngineError::PrimaryKeyPropertyNotScalar {
                name: "account_number".into()
            })
        );
        assert_eq!(
            engine, before,
            "refused adoption preserves current and all accepted revisions"
        );
        let mut optional =
            with_declaration(def(1), candidate.properties[0].value_type.clone().unwrap())
                .with_primary_key_property("account_number");
        optional.properties[0].required = false;
        let mut engine = OntologyEngine::default();
        assert_eq!(
            engine.register_entity_type(optional.clone()),
            Err(OntologyEngineError::PrimaryKeyPropertyNotRequired {
                name: "account_number".into()
            })
        );
        optional.title_property = Some("ghost".into());
        assert_eq!(
            engine.register_entity_type(optional),
            Err(OntologyEngineError::DesignatedPropertyNotDeclared {
                name: "ghost".into()
            })
        );
    }
}

#[test]
fn all_scalar_and_legacy_tier_keys_preserve_registration_evolution_and_value_conformance() {
    let scalars = [
        (ScalarType::String, PropertyValue::String(String::new())),
        (ScalarType::Integer, PropertyValue::Integer(0)),
        (
            ScalarType::Double,
            PropertyValue::Double(FiniteDouble::new(-0.0).unwrap()),
        ),
        (ScalarType::Boolean, PropertyValue::Boolean(false)),
        (
            ScalarType::Date,
            PropertyValue::Date(CalendarDate::new(2026, 10, 13).unwrap()),
        ),
        (
            ScalarType::Timestamp,
            PropertyValue::Timestamp { epoch_millis: 0 },
        ),
    ];
    let cases = scalars
        .into_iter()
        .map(|(kind, value)| {
            (
                PropertyTier::Scalar,
                Some(ValueTypeDeclaration::Scalar(kind)),
                value,
            )
        })
        .chain(
            PropertyTier::all_tiers()
                .into_iter()
                .map(|tier| (tier, None, PropertyValue::String(String::new()))),
        );
    for (tier, declaration, value) in cases {
        let mut keyed = def(1).with_primary_key_property("account_number");
        keyed.properties[0].tier = tier;
        keyed.properties[0].value_type = declaration;
        let mut property = ObjectProperty::typed("account_number".into(), value, internal());
        property.tier = tier;
        let entity = ObjectEntity::new(
            "ten_test".into(),
            "ent_account".into(),
            "ety_account".into(),
            vec![property],
        )
        .unwrap();
        for first_evolve in [false, true] {
            let mut engine = OntologyEngine::default();
            if first_evolve {
                engine.evolve_entity_type(keyed.clone())
            } else {
                engine.register_entity_type(keyed.clone())
            }
            .unwrap();
            assert_eq!(engine.check_instance_conformance(&entity), Ok(()));
            let mut next = keyed.clone();
            next.revision = 2;
            engine.evolve_entity_type(next).unwrap();
            assert_eq!(engine.check_instance_conformance(&entity), Ok(()));
        }
        let mut unkeyed = keyed.clone();
        unkeyed.primary_key_property = None;
        let mut engine = OntologyEngine::default();
        engine.register_entity_type(unkeyed).unwrap();
        keyed.revision = 2;
        engine
            .evolve_entity_type(keyed)
            .expect("legacy and scalar key adoption remains valid");
        assert_eq!(engine.check_instance_conformance(&entity), Ok(()));
    }
}
