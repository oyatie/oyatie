use crate::*;
use data_boundary_kernel::PrivacyDataClass;
fn definition(tenant: &str, id: &str, members: &[&str]) -> NamedStringTypeDefinition {
    NamedStringTypeDefinition::new(
        NamedStringTypeRef::new(tenant, NamedStringTypeId::new(id).unwrap(), 1).unwrap(),
        "Test status",
        members.iter().map(|s| s.to_string()).collect(),
    )
    .unwrap()
}
fn entity(declaration: ValueTypeDeclaration) -> EntityTypeDefinition {
    EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_test").unwrap(),
        "Test",
        vec![
            EntityTypePropertyDefinition::typed(
                "status",
                declaration,
                PrivacyDataClass::internal_only(),
                true,
            )
            .unwrap(),
        ],
        1,
    )
    .unwrap()
}
fn action(declaration: ValueTypeDeclaration) -> ActionTypeDefinition {
    ActionTypeDefinition::new(
        "ten_test",
        ActionTypeId::new("aty_test").unwrap(),
        EntityTypeId::new("ety_test").unwrap(),
        "ops-console",
        AutonomyTier::T1Assist,
        "test.written",
    )
    .unwrap()
    .with_parameters(vec![
        ActionParameterDefinition::typed(
            "status",
            declaration,
            PrivacyDataClass::internal_only(),
            true,
        )
        .unwrap(),
    ])
}
#[test]
fn canonical_identity_is_required_at_every_entity_and_action_admission_station() {
    let accepted = definition("ten_test", "vty_status", &["TestActive", "TestLeave"]);
    let conflicting = definition("ten_test", "vty_status", &["TestInvalid"]);
    let foreign = definition("ten_other", "vty_status", &["TestActive", "TestLeave"]);
    let mut engine = OntologyEngine::default();
    engine.register_named_string_type(accepted.clone()).unwrap();
    let baseline = engine.clone();
    for bad in [conflicting, foreign] {
        for evolve in [false, true] {
            let mut isolated = baseline.clone();
            let result = if evolve {
                isolated.evolve_entity_type(entity(ValueTypeDeclaration::NamedString(bad.clone())))
            } else {
                isolated
                    .register_entity_type(entity(ValueTypeDeclaration::NamedString(bad.clone())))
            };
            assert!(result.is_err());
            assert_eq!(isolated, baseline);
        }
    }
    let declaration = engine
        .named_string_declaration(accepted.reference())
        .unwrap();
    engine
        .register_entity_type(entity(declaration.clone()))
        .unwrap();
    engine
        .register_action_type(action(declaration.clone()))
        .unwrap();
    let baseline = engine.clone();
    for bad in [
        definition("ten_test", "vty_status", &["TestInvalid"]),
        definition("ten_other", "vty_status", &["TestActive"]),
    ] {
        let mut isolated = baseline.clone();
        let mut candidate = action(ValueTypeDeclaration::NamedString(bad.clone()));
        candidate.id = ActionTypeId::new("aty_new").unwrap();
        assert!(isolated.register_action_type(candidate).is_err());
        assert_eq!(isolated, baseline);
        let mut candidate = action(ValueTypeDeclaration::NamedString(bad.clone()));
        candidate.revision = 2;
        assert!(isolated.evolve_action_type(candidate).is_err());
        assert_eq!(isolated, baseline);
        let mut candidate = entity(ValueTypeDeclaration::NamedString(bad));
        candidate.revision = 2;
        assert!(isolated.evolve_entity_type(candidate).is_err());
        assert_eq!(isolated, baseline);
    }
    assert_eq!(
        engine.register_named_string_type(accepted.clone()),
        Err(OntologyEngineError::DuplicateNamedStringType)
    );
    let other = definition("ten_test", "vty_equal_set", &["TestActive", "TestLeave"]);
    engine.register_named_string_type(other.clone()).unwrap();
    assert_ne!(
        engine.named_string_declaration(other.reference()).unwrap(),
        declaration
    );
    assert_eq!(engine.named_string_types("ten_other").count(), 0);
}
#[test]
fn exact_allowed_string_members_use_the_shared_property_and_parameter_station_without_coercion() {
    let accepted = definition("ten_test", "vty_status", &["TestActive", "é"]);
    let mut engine = OntologyEngine::default();
    engine.register_named_string_type(accepted.clone()).unwrap();
    let declaration = engine
        .named_string_declaration(accepted.reference())
        .unwrap();
    engine
        .register_entity_type(entity(declaration.clone()))
        .unwrap();
    engine.register_action_type(action(declaration)).unwrap();
    for (value, allowed) in [
        (PropertyValue::String("TestActive".into()), true),
        (PropertyValue::String("é".into()), true),
        (PropertyValue::String("e\u{301}".into()), false),
        (PropertyValue::String("testactive".into()), false),
        (PropertyValue::String(" TestActive".into()), false),
        (PropertyValue::Integer(1), false),
    ] {
        let submitted = [ObjectProperty::typed(
            "status".into(),
            value,
            PrivacyDataClass::internal_only(),
        )];
        let object = ObjectEntity::new(
            "ten_test".into(),
            "ent_test".into(),
            "ety_test".into(),
            submitted.to_vec(),
        )
        .unwrap();
        let property = engine.check_instance_conformance(&object);
        let parameter = engine.check_action_parameter_conformance(
            "ten_test",
            &ActionTypeId::new("aty_test").unwrap(),
            &submitted,
        );
        assert_eq!(property.is_ok(), allowed);
        assert_eq!(parameter.is_ok(), allowed);
        if !allowed {
            assert!(matches!(
                property,
                Err(OntologyEngineError::PropertyValueTypeMismatch {
                    expected: "named_string",
                    ..
                })
            ));
            assert!(matches!(
                parameter,
                Err(OntologyEngineError::ParameterValueTypeMismatch {
                    expected: "named_string",
                    ..
                })
            ));
        }
    }
}
#[test]
fn invalid_sets_revisions_and_nested_positions_are_refused_without_normalizing_members() {
    let reference =
        NamedStringTypeRef::new("ten_test", NamedStringTypeId::new("vty_test").unwrap(), 1)
            .unwrap();
    for members in [
        vec![],
        vec![" ".into()],
        vec!["Test".into(), "Test".into()],
        vec!["x".repeat(MAX_NAMED_STRING_TEXT_BYTES + 1)],
    ] {
        assert_eq!(
            NamedStringTypeDefinition::new(reference.clone(), "Test", members),
            Err(OntologyEngineError::InvalidNamedStringMembers)
        );
    }
    assert!(NamedStringTypeRef::new("ten_test", reference.id().clone(), 0).is_err());
    let late = NamedStringTypeDefinition::new(
        NamedStringTypeRef::new("ten_test", reference.id().clone(), 2).unwrap(),
        "Test",
        vec!["Test".into()],
    )
    .unwrap();
    assert_eq!(
        OntologyEngine::default().register_named_string_type(late),
        Err(OntologyEngineError::InvalidNamedStringRevision)
    );
    let nested = ValueTypeDeclaration::Array {
        element: Box::new(ValueTypeDeclaration::NamedString(definition(
            "ten_test",
            "vty_test",
            &["Test"],
        ))),
    };
    assert_eq!(nested.validate(), Err(ValueTypeError::NestedNamedString));
}
