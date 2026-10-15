use super::*;

fn named_key_registry() -> OntologyEngine {
    use data_ontology_kernel::{NamedStringTypeDefinition, NamedStringTypeId, NamedStringTypeRef};
    let legacy = registry();
    let id = EntityTypeId::new("ety_reading").unwrap();
    let mut row = legacy.entity_type("ten_test", &id).unwrap().clone();
    let reference = NamedStringTypeRef::new(
        "ten_test",
        NamedStringTypeId::new("vty_test_name").unwrap(),
        1,
    )
    .unwrap();
    let mut engine = OntologyEngine::default();
    engine
        .register_named_string_type(
            NamedStringTypeDefinition::new(
                reference.clone(),
                "Test names",
                vec!["Ada".into(), "Grace".into()],
            )
            .unwrap(),
        )
        .unwrap();
    row.properties[0].value_type = Some(engine.named_string_declaration(&reference).unwrap());
    row.primary_key_property = Some("name".into());
    engine.register_entity_type(row).unwrap();
    engine
        .register_action_type(
            legacy
                .action_type("ten_test", &ActionTypeId::new("aty_calibrate").unwrap())
                .unwrap()
                .clone(),
        )
        .unwrap();
    engine
}
#[test]
fn a_logged_nonmember_poison_is_deterministic_and_cannot_create_an_object() {
    use foundry_spine::{FoldOutcome, PoisonReason};
    let engine = named_key_registry();
    let entry = sealed(
        "ent_bad_named",
        1,
        vec![
            OntologyEdit::create_object(
                "ety_reading",
                vec![prop("name", WireValue::String("Unlisted".into()))],
            )
            .unwrap(),
        ],
    );
    let mut state = ProjectionState::new("ten_test", &engine);
    let result = apply_sealed(&mut state, &entry);
    assert!(matches!(
        result,
        FoldOutcome::Poisoned(PoisonReason::Conformance(
            data_ontology_kernel::OntologyEngineError::PropertyValueTypeMismatch {
                expected: "named_string",
                ..
            }
        ))
    ));
    let replayed = fold_from_scratch("ten_test", &engine, [&entry]);
    assert_eq!(replayed, state);
    assert!(state.bindings.is_empty());
    assert!(state.history.is_empty());
    assert_eq!(state.objects, data_ontology_kernel::ObjectGraph::default());
}
#[test]
fn named_string_keys_keep_requiredness_and_exact_string_uniqueness_in_the_projection_store() {
    use foundry_projection_draft::{MemoryProjectionStore, ProjectionStore, ProjectionStoreError};
    use foundry_spine::{WriteThroughError, project_through};
    let engine = named_key_registry();
    let entries = [
        sealed(
            "ent_a",
            1,
            vec![
                OntologyEdit::create_object(
                    "ety_reading",
                    vec![prop("name", WireValue::String("Ada".into()))],
                )
                .unwrap(),
            ],
        ),
        sealed(
            "ent_b",
            2,
            vec![
                OntologyEdit::create_object(
                    "ety_reading",
                    vec![prop("name", WireValue::String("Ada".into()))],
                )
                .unwrap(),
            ],
        ),
    ];
    let mut state = ProjectionState::new("ten_test", &engine);
    let mut store = MemoryProjectionStore::default();
    assert!(matches!(
        project_through(&mut state, &mut store, &entries),
        Err(WriteThroughError::Store {
            ordinal: 2,
            error: ProjectionStoreError::DuplicatePrimaryKey { .. }
        })
    ));
    assert_eq!(store.applied_head("ten_test").unwrap(), 1);
    assert!(store.get("ten_test", "ent_b").unwrap().is_none());
    let id = EntityTypeId::new("ety_reading").unwrap();
    let mut optional = engine.entity_type("ten_test", &id).unwrap().clone();
    optional.properties[0].required = false;
    let mut fresh = engine.clone();
    assert!(matches!(
        fresh.evolve_entity_type(optional),
        Err(data_ontology_kernel::OntologyEngineError::PrimaryKeyPropertyNotRequired { .. })
    ));
}
