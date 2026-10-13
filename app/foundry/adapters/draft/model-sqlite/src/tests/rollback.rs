use super::*;

#[test]
fn initialization_failures_roll_back_schema_binding_and_baseline_together() {
    for boundary in ["schema", "binding", "publication", "commit"] {
        let database = Database::new();
        INITIALIZE_FAILURE.with(|fault| fault.set(Some(boundary)));
        assert!(matches!(
            SqliteModelStore::initialize(&database.path(), binding(), baseline()),
            Err(ModelError::Storage { .. })
        ));
        assert_eq!(
            model_table_count(&Connection::open(database.path()).unwrap()).unwrap(),
            0
        );
        assert!(matches!(
            SqliteModelStore::open_existing(&database.path(), binding(), baseline()),
            Err(ModelError::Uninitialized)
        ));
        assert_eq!(
            database
                .initialize()
                .history(&caller())
                .unwrap()
                .head()
                .unwrap()
                .generation,
            1
        );
    }
}

fn snapshot(store: &SqliteModelStore) -> ModelLedger {
    store
        .read(|connection, state| {
            let mut statement = connection
                .prepare("SELECT id FROM ontology_model_draft ORDER BY id")
                .map_err(storage)?;
            let ids = statement
                .query_map([], |row| row.get::<_, u64>(0))
                .map_err(storage)?;
            let drafts = ids
                .map(|id| load::draft(connection, &state.history, id.map_err(storage)?))
                .collect::<Result<Vec<_>, _>>()?;
            ModelLedger::restore(state.history.clone(), state.next_draft, drafts)
        })
        .unwrap()
}

#[test]
fn publication_statement_failures_roll_back_history_head_and_closed_draft_together() {
    for (table, event) in [
        ("ontology_model_publication", "INSERT"),
        ("ontology_model_binding", "UPDATE"),
        ("ontology_model_draft", "UPDATE"),
    ] {
        for timing in ["BEFORE", "AFTER"] {
            let database = Database::new();
            let mut store = database.initialize();
            let draft = store.create_draft(&caller(), 1, additive_model()).unwrap();
            let prepared = store.prepare(&caller(), draft.id, 1, 1).unwrap();
            let before = snapshot(&store);
            store
                .connection
                .execute_batch(&format!(
                    "CREATE TRIGGER refuse_write {timing} {event} ON {table}
                BEGIN SELECT RAISE(ABORT, 'injected model statement failure'); END;"
                ))
                .unwrap();
            assert!(
                matches!(
                    store.publish(&caller(), &prepared),
                    Err(ModelError::Storage { .. })
                ),
                "{timing} {event} {table}"
            );
            assert_eq!(snapshot(&store), before);
            store
                .connection
                .execute_batch("DROP TRIGGER refuse_write;")
                .unwrap();
            drop(store);
            assert_eq!(snapshot(&database.open()), before);
        }
    }
}

#[test]
fn real_failed_commit_rolls_back_all_publication_writes() {
    let database = Database::new();
    let mut store = database.initialize();
    let draft = store.create_draft(&caller(), 1, additive_model()).unwrap();
    let prepared = store.prepare(&caller(), draft.id, 1, 1).unwrap();
    let before = snapshot(&store);
    store.connection.execute_batch("CREATE TABLE model_failure_parent(id INTEGER PRIMARY KEY);
        CREATE TABLE model_failure_child(id INTEGER REFERENCES model_failure_parent(id) DEFERRABLE INITIALLY DEFERRED);
        CREATE TRIGGER refuse_commit AFTER UPDATE ON ontology_model_draft
        BEGIN INSERT INTO model_failure_child VALUES (99); END;").unwrap();
    let result = store.publish(&caller(), &prepared);
    assert!(
        matches!(result, Err(ModelError::Storage { .. })),
        "{result:?}"
    );
    assert_eq!(snapshot(&store), before);
    assert_eq!(
        store
            .connection
            .query_row("SELECT COUNT(*) FROM model_failure_child", [], |row| row
                .get::<_, u64>(0))
            .unwrap(),
        0
    );
    store
        .connection
        .execute_batch("DROP TRIGGER refuse_commit;")
        .unwrap();
    drop(store);
    let mut reopened = database.open();
    assert_eq!(snapshot(&reopened), before);
    assert!(reopened.publish(&caller(), &prepared).is_ok());
}

#[test]
fn draft_create_and_save_failures_leave_no_id_allocation_or_partial_version() {
    for (table, event) in [
        ("ontology_model_draft", "INSERT"),
        ("ontology_model_binding", "UPDATE"),
    ] {
        let database = Database::new();
        let mut store = database.initialize();
        let before = snapshot(&store);
        store
            .connection
            .execute_batch(&format!(
                "CREATE TRIGGER refuse_write AFTER {event} ON {table}
            BEGIN SELECT RAISE(ABORT, 'injected create failure'); END;"
            ))
            .unwrap();
        assert!(store.create_draft(&caller(), 1, additive_model()).is_err());
        assert_eq!(snapshot(&store), before);
    }
    let database = Database::new();
    let mut store = database.initialize();
    let draft = store.create_draft(&caller(), 1, additive_model()).unwrap();
    let before = snapshot(&store);
    store
        .connection
        .execute_batch(
            "CREATE TRIGGER refuse_write AFTER UPDATE ON ontology_model_draft
        BEGIN SELECT RAISE(ABORT, 'injected save failure'); END;",
        )
        .unwrap();
    assert!(
        store
            .save_draft(&caller(), draft.id, 1, 1, additive_model())
            .is_err()
    );
    assert_eq!(snapshot(&store), before);
}
