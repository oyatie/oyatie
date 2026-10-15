use super::super::*;

#[test]
fn named_string_migrations_copy_only_the_same_exact_declaration_and_never_convert_or_default_as_string()
 {
    use data_ontology_kernel::{
        EntityTypePropertyDefinition, NamedStringTypeDefinition, NamedStringTypeId,
        NamedStringTypeRef, ValueTypeDeclaration,
    };
    let mut engine = OntologyEngine::default();
    let reference = NamedStringTypeRef::new(
        "ten_test",
        NamedStringTypeId::new("vty_test_status").unwrap(),
        1,
    )
    .unwrap();
    engine
        .register_named_string_type(
            NamedStringTypeDefinition::new(
                reference.clone(),
                "Test status",
                vec!["TestActive".into()],
            )
            .unwrap(),
        )
        .unwrap();
    let declaration = engine.named_string_declaration(&reference).unwrap();
    let named = |name: &str| {
        EntityTypePropertyDefinition::typed(
            name,
            declaration.clone(),
            migration_plan_support::internal(),
            false,
        )
        .unwrap()
    };
    let mut old = rev1_properties();
    old.push(named("named_source"));
    engine
        .register_entity_type(definition(1, old.clone()))
        .unwrap();
    let mut new = old;
    new.push(named("named_target"));
    new.push(migration_plan_support::typed(
        "free_string",
        data_ontology_kernel::ScalarType::String,
    ));
    engine.evolve_entity_type(definition(2, new)).unwrap();
    engine.register_action_type(action()).unwrap();
    assert_eq!(
        plan(vec![copy("named_source", "named_target")]).validate(&engine),
        Ok(())
    );
    for transform in [
        copy("named_source", "free_string"),
        copy("note", "named_target"),
        UpcastTransform::ConvertAs {
            from: "score".into(),
            to: "named_target".into(),
            conversion: ValueConversion::IntegerToString,
        },
        UpcastTransform::DefaultTo {
            to: "named_target".into(),
            value: DefaultValue::String("TestActive".into()),
        },
    ] {
        assert!(matches!(
            plan(vec![transform]).validate(&engine),
            Err(PlanError::TypeIncompatible { .. })
        ));
    }
    assert!(matches!(declaration, ValueTypeDeclaration::NamedString(_)));
}
