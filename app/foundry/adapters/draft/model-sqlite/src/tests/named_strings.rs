use super::*;
fn blobs(store: &SqliteModelStore) -> Vec<Vec<u8>> {
    let mut result = vec![];
    for query in [
        "SELECT binding FROM ontology_model_binding",
        "SELECT baseline FROM ontology_model_binding",
        "SELECT record FROM ontology_model_draft ORDER BY id",
        "SELECT record FROM ontology_model_publication ORDER BY generation",
    ] {
        let mut statement = store.connection.prepare(query).unwrap();
        result.extend(
            statement
                .query_map([], |row| row.get::<_, Vec<u8>>(0))
                .unwrap()
                .map(Result::unwrap),
        );
    }
    result
}
#[test]
fn named_publication_preserves_legacy_blobs_and_retains_unused_canonical_rows_after_durable_reopen()
{
    let database = Database::new();
    let mut store = database.initialize();
    let legacy = blobs(&store);
    let mut model = named_string_model();
    let mut unused = model.named_string_types[0].clone();
    unused.id = "vty_test_unused".into();
    model.named_string_types.push(unused);
    let saved = store.create_draft(&caller(), 1, model.clone()).unwrap();
    let prepared = store.prepare(&caller(), saved.id, 1, 1).unwrap();
    let receipt = store.publish(&caller(), &prepared).unwrap();
    let current = blobs(&store);
    assert_eq!(current[0], legacy[0]);
    assert_eq!(current[1], legacy[1]);
    assert!(current.contains(&legacy[2]));
    let registry = store.history(&caller()).unwrap().registry().unwrap();
    drop(store);
    let reopened = database.open();
    assert_eq!(
        reopened.open_draft(&caller(), saved.id).unwrap().document,
        model
    );
    assert_eq!(
        reopened.history(&caller()).unwrap().registry().unwrap(),
        registry
    );
    assert_eq!(registry.named_string_types("ten_model").count(), 2);
    assert_eq!(
        reopened.receipt(&caller(), saved.id, 1, 1).unwrap(),
        Some(receipt)
    );
    assert_eq!(blobs(&reopened), current);
}
#[test]
fn invalid_saved_named_draft_reopens_exactly_and_native_prepare_has_no_durable_effect() {
    let database = Database::new();
    let mut store = database.initialize();
    let mut model = named_string_model();
    model.named_string_types[0]
        .members
        .push("TestActive".into());
    let saved = store.create_draft(&caller(), 1, model.clone()).unwrap();
    drop(store);
    let mut store = database.open();
    assert_eq!(
        store.open_draft(&caller(), saved.id).unwrap().document,
        model
    );
    let before = blobs(&store);
    assert!(store.prepare(&caller(), saved.id, 1, 1).is_err());
    assert_eq!(blobs(&store), before);
    assert_eq!(store.head(&caller()).unwrap().generation, 1);
    let fixed = store
        .save_draft(&caller(), saved.id, 1, 1, named_string_model())
        .unwrap();
    assert!(store.prepare(&caller(), fixed.id, fixed.version, 1).is_ok());
}
#[test]
fn amplified_schema_metadata_refuses_before_publication_and_keeps_all_rows_identical() {
    let database = Database::new();
    let mut store = database.initialize();
    let mut model = named_string_model();
    model.named_string_types[0].members = (0..64)
        .map(|i| format!("Test{i:03}{}", "x".repeat(4000)))
        .collect();
    for index in 0..4 {
        let mut property = property(&format!("test_reuse_{index}"), false);
        property.value_type = Some(ModelValueType::NamedString {
            type_id: "vty_test_status".into(),
            revision: 1,
        });
        model.entities[0].properties.push(property);
    }
    let saved = store.create_draft(&caller(), 1, model).unwrap();
    let before = blobs(&store);
    assert!(
        matches!(store.prepare(&caller(),saved.id,1,1),Err(ModelError::Limit{path})if path=="schemas")
    );
    assert_eq!(blobs(&store), before);
    assert_eq!(store.head(&caller()).unwrap().generation, 1);
}
