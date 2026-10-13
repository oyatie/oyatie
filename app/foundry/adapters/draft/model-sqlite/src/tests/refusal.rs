use super::*;

#[test]
fn established_open_never_creates_missing_or_partial_baseline() {
    let database = Database::new();
    assert!(matches!(
        SqliteModelStore::open_existing(&database.path(), binding(), baseline()),
        Err(ModelError::Uninitialized)
    ));
    assert!(!database.path().exists());
    let connection = Connection::open(database.path()).unwrap();
    drop(connection);
    assert!(matches!(
        SqliteModelStore::open_existing(&database.path(), binding(), baseline()),
        Err(ModelError::Uninitialized)
    ));
    let store = database.initialize();
    assert!(matches!(
        SqliteModelStore::initialize(&database.path(), binding(), baseline()),
        Err(ModelError::AlreadyInitialized)
    ));
    store
        .connection
        .execute("DELETE FROM ontology_model_publication", [])
        .unwrap();
    drop(store);
    assert!(matches!(
        SqliteModelStore::open_existing(&database.path(), binding(), baseline()),
        Err(ModelError::CorruptState)
    ));
    assert!(matches!(
        SqliteModelStore::initialize(&database.path(), binding(), baseline()),
        Err(ModelError::AlreadyInitialized)
    ));
    let connection = Connection::open(database.path()).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM ontology_model_publication",
                [],
                |row| row.get::<_, u64>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn mismatched_external_binding_or_baseline_is_refused() {
    let database = Database::new();
    drop(database.initialize());
    for field in 0..3 {
        let mut expected = binding();
        match field {
            0 => expected.store_id = "store_other".into(),
            1 => expected.baseline_id = "baseline_other".into(),
            _ => expected.tenant_id = "ten_other".into(),
        }
        assert!(matches!(
            SqliteModelStore::open_existing(&database.path(), expected, baseline()),
            Err(ModelError::BindingMismatch)
        ));
    }
    let mut expected_baseline = baseline();
    expected_baseline.document.entities[0].display_name = "Different".into();
    assert!(matches!(
        SqliteModelStore::open_existing(&database.path(), binding(), expected_baseline),
        Err(ModelError::BindingMismatch)
    ));
}

#[test]
fn corrupt_head_publication_draft_and_receipt_are_refused_on_restart() {
    for corrupt in 0..6 {
        let database = Database::new();
        let mut store = database.initialize();
        let draft = store.create_draft(&caller(), 1, additive_model()).unwrap();
        let prepared = store.prepare(&caller(), draft.id, 1, 1).unwrap();
        store.publish(&caller(), &prepared).unwrap();
        match corrupt {
            0 => {
                store
                    .connection
                    .execute("UPDATE ontology_model_binding SET head = 99", [])
                    .unwrap();
            }
            1 => {
                store
                    .connection
                    .execute(
                        "DELETE FROM ontology_model_publication WHERE generation = 1",
                        [],
                    )
                    .unwrap();
            }
            2 => {
                store
                    .connection
                    .execute("UPDATE ontology_model_draft SET record = X'7B7D'", [])
                    .unwrap();
            }
            3 => {
                assert!(
                    store
                        .connection
                        .execute("DELETE FROM ontology_model_draft", [])
                        .is_err()
                );
                let corrupt_connection = Connection::open(database.path()).unwrap();
                corrupt_connection
                    .execute_batch("PRAGMA foreign_keys = OFF;")
                    .unwrap();
                corrupt_connection
                    .execute("DELETE FROM ontology_model_draft", [])
                    .unwrap();
            }
            4 => {
                let mut receipt = store.open_draft(&caller(), draft.id).unwrap();
                receipt.publication.as_mut().unwrap().draft_version = 99;
                store
                    .connection
                    .execute(
                        "UPDATE ontology_model_draft SET record = ?1",
                        params![encode(&receipt).unwrap()],
                    )
                    .unwrap();
            }
            _ => {
                store
                    .connection
                    .execute("UPDATE ontology_model_binding SET next_draft = 99", [])
                    .unwrap();
            }
        }
        assert!(store.history(&caller()).is_err());
        drop(store);
        assert!(matches!(
            SqliteModelStore::open_existing(&database.path(), binding(), baseline()),
            Err(ModelError::CorruptState)
        ));
    }
}

#[test]
fn native_version_overflow_is_refused_without_changing_saved_document() {
    let database = Database::new();
    let mut store = database.initialize();
    let mut draft = store.create_draft(&caller(), 1, additive_model()).unwrap();
    draft.version = MAX_MODEL_VERSION;
    store
        .connection
        .execute(
            "UPDATE ontology_model_draft SET record = ?1",
            params![encode(&draft).unwrap()],
        )
        .unwrap();
    assert_eq!(
        store.save_draft(
            &caller(),
            draft.id,
            MAX_MODEL_VERSION,
            1,
            baseline().document
        ),
        Err(ModelError::VersionOverflow)
    );
    assert_eq!(store.open_draft(&caller(), draft.id).unwrap(), draft);
}
