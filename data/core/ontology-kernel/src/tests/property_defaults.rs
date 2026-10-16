use crate::*;
use data_boundary_kernel::PrivacyDataClass;

fn property(
    kind: Option<ValueTypeDeclaration>,
    value: PropertyValue,
) -> EntityTypePropertyDefinition {
    let mut property = EntityTypePropertyDefinition::new(
        "value",
        kind.as_ref()
            .map_or(PropertyTier::Scalar, ValueTypeDeclaration::tier),
        PrivacyDataClass::internal_only(),
        true,
    )
    .unwrap();
    property.value_type = kind;
    property.default_value = Some(value);
    property
}

fn entity(property: EntityTypePropertyDefinition, revision: u32) -> EntityTypeDefinition {
    EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_defaults").unwrap(),
        "Defaults",
        vec![property],
        revision,
    )
    .unwrap()
}

fn invalid(path: &str, expected: &'static str, found: &'static str) -> OntologyEngineError {
    OntologyEngineError::InvalidPropertyDefault {
        name: "value".into(),
        cause: ValueTypeViolation {
            path: path.into(),
            expected,
            found,
        },
    }
}

#[test]
fn scalar_defaults_require_exact_carriers_on_registration_and_evolution() {
    for (kind, value) in [
        (ScalarType::String, PropertyValue::String(String::new())),
        (ScalarType::Integer, PropertyValue::Integer(0)),
        (
            ScalarType::Double,
            PropertyValue::Double(FiniteDouble::new(0.0).unwrap()),
        ),
        (ScalarType::Boolean, PropertyValue::Boolean(false)),
        (
            ScalarType::Date,
            PropertyValue::Date(CalendarDate::new(2000, 2, 29).unwrap()),
        ),
        (
            ScalarType::Timestamp,
            PropertyValue::Timestamp {
                epoch_millis: i64::MIN,
            },
        ),
    ] {
        let mut engine = OntologyEngine::default();
        let good = entity(property(Some(ValueTypeDeclaration::Scalar(kind)), value), 1);
        engine.register_entity_type(good.clone()).unwrap();
        let mut bad = good.clone();
        bad.properties[0].default_value = Some(PropertyValue::Array(vec![]));
        let expected = invalid(
            "",
            good.properties[0].value_type.as_ref().unwrap().type_label(),
            "array",
        );
        assert_eq!(
            OntologyEngine::default().register_entity_type(bad.clone()),
            Err(expected.clone())
        );
        bad.revision = 2;
        assert_eq!(engine.evolve_entity_type(bad), Err(expected));
        assert_eq!(engine.entity_type("ten_test", &good.id), Some(&good));
    }
    let good = property(None, PropertyValue::String(String::new()));
    OntologyEngine::default()
        .register_entity_type(entity(good, 1))
        .unwrap();
    assert_eq!(
        OntologyEngine::default()
            .evolve_entity_type(entity(property(None, PropertyValue::Integer(0)), 1)),
        Err(invalid("", "string", "integer"))
    );
}

#[test]
fn named_defaults_preserve_exact_members_without_coercion() {
    let definition = NamedStringTypeDefinition::new(
        NamedStringTypeRef::new(
            "ten_test",
            NamedStringTypeId::new("vty_defaults").unwrap(),
            1,
        )
        .unwrap(),
        "Default vocabulary",
        vec!["é".into(), "TestActive".into()],
    )
    .unwrap();
    let mut registry = OntologyEngine::default();
    registry
        .register_named_string_type(definition.clone())
        .unwrap();
    let kind = registry
        .named_string_declaration(definition.reference())
        .unwrap();
    for (value, admitted) in [
        (PropertyValue::String("é".into()), true),
        (PropertyValue::String("e\u{301}".into()), false),
        (PropertyValue::String("testactive".into()), false),
        (PropertyValue::String(" TestActive".into()), false),
        (PropertyValue::Integer(0), false),
    ] {
        let mut engine = registry.clone();
        let candidate = entity(property(Some(kind.clone()), value.clone()), 1);
        let result = engine.register_entity_type(candidate);
        if admitted {
            assert!(result.is_ok());
        } else {
            assert_eq!(result, Err(invalid("", "named_string", value.type_label())));
            assert_eq!(engine, registry);
        }
    }
}

#[test]
fn composite_default_violations_keep_struct_and_array_paths() {
    let kind = ValueTypeDeclaration::Struct(StructSchema {
        fields: vec![StructFieldDeclaration {
            name: "counts".into(),
            required: true,
            value_type: ValueTypeDeclaration::Array {
                element: Box::new(ValueTypeDeclaration::Scalar(ScalarType::Integer)),
            },
        }],
    });
    let good = PropertyValue::Struct(
        [(
            "counts".into(),
            PropertyValue::Array(vec![PropertyValue::Integer(0)]),
        )]
        .into(),
    );
    OntologyEngine::default()
        .register_entity_type(entity(property(Some(kind.clone()), good), 1))
        .unwrap();
    for (value, expected) in [
        (
            PropertyValue::Struct(
                [(
                    "counts".into(),
                    PropertyValue::Array(vec![PropertyValue::Boolean(false)]),
                )]
                .into(),
            ),
            invalid("counts[0]", "integer", "boolean"),
        ),
        (
            PropertyValue::Struct(Default::default()),
            invalid("counts", "array", "absent"),
        ),
        (
            PropertyValue::Struct(
                [
                    ("counts".into(), PropertyValue::Array(vec![])),
                    ("extra".into(), PropertyValue::Integer(0)),
                ]
                .into(),
            ),
            invalid("extra", "absent", "integer"),
        ),
    ] {
        assert_eq!(
            OntologyEngine::default()
                .evolve_entity_type(entity(property(Some(kind.clone()), value), 1)),
            Err(expected)
        );
    }
}

#[test]
fn default_metadata_changes_require_higher_revisions_and_never_relax_required_or_frozen_fields() {
    let mut original = property(
        Some(ValueTypeDeclaration::Scalar(ScalarType::Integer)),
        PropertyValue::Integer(0),
    );
    original.default_value = None;
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(entity(original.clone(), 1))
        .unwrap();
    let mut changed = original.clone();
    for (revision, value) in [
        (2, Some(PropertyValue::Integer(0))),
        (3, Some(PropertyValue::Integer(1))),
        (4, None),
    ] {
        changed.default_value = value;
        assert_eq!(
            engine.evolve_entity_type(entity(changed.clone(), revision - 1)),
            Err(OntologyEngineError::NonMonotonicRevision)
        );
        engine
            .evolve_entity_type(entity(changed.clone(), revision))
            .unwrap();
    }
    changed.default_value = Some(PropertyValue::Integer(0));
    engine
        .evolve_entity_type(entity(changed.clone(), 5))
        .unwrap();
    let object = ObjectEntity {
        tenant_id: "ten_test".into(),
        id: "ent_test".into(),
        entity_type: data_boundary_kernel::Classified::new(
            "ety_defaults".into(),
            data_boundary_kernel::DataClass::InternalOnly,
        ),
        properties: Default::default(),
    };
    assert_eq!(
        engine.check_instance_conformance(&object),
        Err(OntologyEngineError::MissingRequiredProperty {
            name: "value".into()
        })
    );
    for mutation in 0..6 {
        let mut candidate = entity(changed.clone(), 6);
        match mutation {
            0 => candidate.properties[0].required = false,
            1 => candidate.properties[0].value_type = None,
            2 => {
                candidate.properties[0].data_class =
                    PrivacyDataClass::new(data_boundary_kernel::DataClass::Public).unwrap()
            }
            3 => candidate.properties.clear(),
            4 => candidate.properties[0].tier = PropertyTier::Vector,
            _ => {
                let mut added = changed.clone();
                added.name = "new_required".into();
                candidate.properties.push(added);
            }
        }
        assert!(engine.evolve_entity_type(candidate).is_err());
    }
    assert_eq!(
        engine
            .entity_type_at_revision("ten_test", &EntityTypeId::new("ety_defaults").unwrap(), 1)
            .unwrap()
            .properties[0]
            .default_value,
        None
    );
}
