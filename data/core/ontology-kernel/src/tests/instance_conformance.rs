//! Instance-vs-type conformance: an object instance is validated against the
//! registered definition it claims — required properties present, no
//! undeclared properties, tiers and data classes matching.

use crate::*;
use data_boundary_kernel::{DataClass, PrivacyDataClass};

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

fn pii() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::PiiIdentifying).unwrap()
}

fn engine_with_profile_type() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(
            EntityTypeDefinition::new(
                "ten_test",
                EntityTypeId::new("ety_profile").unwrap(),
                "Profile",
                vec![
                    EntityTypePropertyDefinition::new(
                        "name",
                        PropertyTier::Scalar,
                        internal(),
                        true,
                    )
                    .unwrap(),
                    EntityTypePropertyDefinition::new("email", PropertyTier::Scalar, pii(), false)
                        .unwrap(),
                ],
                1,
            )
            .unwrap(),
        )
        .unwrap();
    engine
}

fn scalar_prop(name: &str, data_class: PrivacyDataClass) -> ObjectProperty {
    ObjectProperty::new(
        name.into(),
        "value".into(),
        PropertyTier::Scalar,
        data_class,
    )
}

fn entity(properties: Vec<ObjectProperty>) -> ObjectEntity {
    ObjectEntity::new(
        "ten_test".into(),
        "ent_profile_1".into(),
        "ety_profile".into(),
        properties,
    )
    .unwrap()
}

/// The conformant fast path: required property present, optional omitted,
/// tier and data class matching the declaration.
#[test]
fn conformant_instance_accepted() {
    let engine = engine_with_profile_type();
    assert_eq!(
        engine.check_instance_conformance(&entity(vec![scalar_prop("name", internal())])),
        Ok(())
    );
}

/// The identity-join contract: a conformant instance carries the registered
/// `EntityTypeId` value as its `entity_type` and a `ten_`-prefixed tenant.
/// The legacy free-string vocabulary resolves to no definition.
#[test]
fn legacy_free_string_entity_type_is_unknown() {
    let engine = engine_with_profile_type();
    let legacy = ObjectEntity::new(
        "tenant_a".into(),
        "ent_profile_1".into(),
        "profile".into(),
        vec![scalar_prop("name", internal())],
    )
    .unwrap();
    assert_eq!(
        engine.check_instance_conformance(&legacy),
        Err(OntologyEngineError::UnknownEntityType)
    );
}

/// A definition property with `required: true` must be present.
#[test]
fn missing_required_property_rejected() {
    let engine = engine_with_profile_type();
    assert_eq!(
        engine.check_instance_conformance(&entity(vec![scalar_prop("email", pii())])),
        Err(OntologyEngineError::MissingRequiredProperty {
            name: "name".into()
        })
    );
}

/// Fail-closed on vocabulary: a property the definition does not declare is
/// rejected rather than ignored — additive-only evolution means a reader can
/// always evolve the type first.
#[test]
fn undeclared_property_rejected() {
    let engine = engine_with_profile_type();
    assert_eq!(
        engine.check_instance_conformance(&entity(vec![
            scalar_prop("name", internal()),
            scalar_prop("nickname", internal()),
        ])),
        Err(OntologyEngineError::UndeclaredProperty {
            name: "nickname".into()
        })
    );
}

/// The instance property's tier must match the declared tier.
#[test]
fn property_tier_mismatch_rejected() {
    let engine = engine_with_profile_type();
    let mismatched = entity(vec![ObjectProperty::new(
        "name".into(),
        "value".into(),
        PropertyTier::Vector,
        internal(),
    )]);
    assert_eq!(
        engine.check_instance_conformance(&mismatched),
        Err(OntologyEngineError::PropertyTierMismatch {
            name: "name".into()
        })
    );
}

/// The instance property's data class must match the declared class.
#[test]
fn property_data_class_mismatch_rejected() {
    let engine = engine_with_profile_type();
    assert_eq!(
        engine.check_instance_conformance(&entity(vec![scalar_prop("name", pii())])),
        Err(OntologyEngineError::PropertyDataClassMismatch {
            name: "name".into()
        })
    );
}
