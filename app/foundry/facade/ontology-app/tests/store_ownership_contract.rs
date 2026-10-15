#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "model_runtime/support.rs"]
mod support;
use foundry_model_api::{ModelDocument, ModelStore};
use foundry_model_sqlite_draft::SqliteModelStore;
use foundry_projection_draft::ProjectionStore;
use foundry_projection_sqlite_draft::SqliteProjectionStore;
use foundry_records_draft::{ActionEnvelope, RecordsLog};
use foundry_records_sqlite_draft::SqliteRecordsLog;
use foundry_store_local_draft::{LocalStoreGuard, LocalStoreSet, StoreAccessError, StoreOpen};
use support::Fixture;

#[test]
fn exclusive_owner_opens_real_bundled_stores_and_retains_exclusion_until_last_handle() {
    let fixture = Fixture::new();
    fixture.initialize();
    let paths = vec![
        (fixture.config.action_log.clone(), StoreOpen::Create),
        (fixture.config.denial_log.clone(), StoreOpen::Create),
        (fixture.config.projection_store.clone(), StoreOpen::Create),
        (fixture.model_path(), StoreOpen::Existing),
    ];
    let set = LocalStoreSet::exclusive(&paths).unwrap();
    let model_guard = set.guard(&fixture.model_path()).unwrap();
    let mut model = SqliteModelStore::open_existing_with_access(
        &fixture.model_path(),
        Fixture::binding(),
        Fixture::baseline(),
        model_guard.clone(),
    )
    .unwrap();
    let second_model = SqliteModelStore::open_existing_with_access(
        &fixture.model_path(),
        Fixture::binding(),
        Fixture::baseline(),
        model_guard.clone(),
    )
    .unwrap();
    assert!(
        model
            .store_access()
            .unwrap()
            .same_owner(second_model.store_access().unwrap())
    );
    let mut action = SqliteRecordsLog::open_with_access(
        &fixture.config.action_log,
        set.guard(&fixture.config.action_log).unwrap(),
    )
    .unwrap();
    let mut denial = SqliteRecordsLog::open_with_access(
        &fixture.config.denial_log,
        set.guard(&fixture.config.denial_log).unwrap(),
    )
    .unwrap();
    let mut projection = SqliteProjectionStore::open_with_access(
        &fixture.config.projection_store,
        set.guard(&fixture.config.projection_store).unwrap(),
    )
    .unwrap();
    let mut document: ModelDocument = Fixture::baseline().document;
    document.entities[0].revision += 1;
    document.entities[0]
        .properties
        .push(foundry_model_api::conformance::property("notes", false));
    let draft = model.create_draft(&Fixture::caller(), 1, document).unwrap();
    let prepared = model.prepare(&Fixture::caller(), draft.id, 1, 1).unwrap();
    model.publish(&Fixture::caller(), &prepared).unwrap();
    assert_eq!(second_model.head(&Fixture::caller()).unwrap().generation, 2);
    let envelope = || {
        ActionEnvelope::new(
            support::TENANT,
            "ent_one",
            "aty_record_write",
            "test-key",
            1,
            vec![0xff],
            1000,
        )
        .unwrap()
    };
    action.append(envelope()).unwrap();
    denial.append(envelope()).unwrap();
    let registry = foundry_ontology_app::seed::registry_for(support::TENANT).unwrap();
    let entries = action.replay(support::TENANT, 1).unwrap();
    let mut folded = foundry_spine::ProjectionState::new(support::TENANT, &registry);
    foundry_spine::project_through(&mut folded, &mut projection, &entries).unwrap();
    assert_eq!(
        foundry_spine::preflight_projection(support::TENANT, &registry, &projection, &entries, 1)
            .unwrap(),
        folded
    );
    assert_eq!(projection.applied_head(support::TENANT).unwrap(), 1);
    assert!(
        SqliteModelStore::open_existing(
            &fixture.model_path(),
            Fixture::binding(),
            Fixture::baseline()
        )
        .is_err()
    );
    assert!(
        SqliteModelStore::initialize(
            &fixture.model_path(),
            Fixture::binding(),
            Fixture::baseline()
        )
        .is_err()
    );
    assert!(SqliteRecordsLog::open(&fixture.config.action_log).is_err());
    assert!(SqliteProjectionStore::open(&fixture.config.projection_store).is_err());
    assert!(matches!(
        LocalStoreSet::exclusive(&paths),
        Err(StoreAccessError::Busy)
    ));
    assert!(
        SqliteProjectionStore::open_with_access(
            &fixture.config.projection_store,
            set.guard(&fixture.config.action_log).unwrap()
        )
        .is_err()
    );
    drop(second_model);
    // Unrelated SQLite descriptors and transaction completion do not release OFD ownership.
    assert!(matches!(
        LocalStoreGuard::shared(&fixture.model_path(), StoreOpen::Existing),
        Err(StoreAccessError::Busy)
    ));
    drop(set);
    drop(model_guard);
    assert!(SqliteRecordsLog::open(&fixture.config.action_log).is_err());
    assert!(SqliteProjectionStore::open(&fixture.config.projection_store).is_err());
    assert!(
        SqliteModelStore::open_existing(
            &fixture.model_path(),
            Fixture::binding(),
            Fixture::baseline()
        )
        .is_err()
    );
    drop(action);
    drop(denial);
    drop(projection);
    drop(model);
    assert!(SqliteRecordsLog::open(&fixture.config.action_log).is_ok());
    assert!(SqliteProjectionStore::open(&fixture.config.projection_store).is_ok());
    assert!(
        SqliteModelStore::open_existing(
            &fixture.model_path(),
            Fixture::binding(),
            Fixture::baseline()
        )
        .is_ok()
    );
}

#[test]
fn standalone_native_handles_prevent_exclusive_serving_without_losing_cas() {
    let fixture = Fixture::new();
    fixture.initialize();
    let first = fixture.open();
    let second = fixture.open();
    let paths = [(fixture.model_path(), StoreOpen::Existing)];
    assert!(matches!(
        LocalStoreSet::exclusive(&paths),
        Err(StoreAccessError::Busy)
    ));
    drop(first);
    drop(second);
    assert!(LocalStoreSet::exclusive(&paths).is_ok());
}

#[test]
fn native_adapters_refuse_uri_memory_and_wrong_guard_before_schema_writes() {
    for path in [
        std::path::Path::new(":memory:"),
        std::path::Path::new("file:ignored?mode=memory&cache=shared"),
        std::path::Path::new("file:ignored?immutable=1&vfs=unix-none"),
    ] {
        assert!(SqliteRecordsLog::open(path).is_err());
        assert!(SqliteProjectionStore::open(path).is_err());
        assert!(
            SqliteModelStore::initialize(path, Fixture::binding(), Fixture::baseline()).is_err()
        );
        assert!(!path.exists());
    }
    let fixture = Fixture::new();
    let set = LocalStoreSet::exclusive(&[
        (fixture.config.action_log.clone(), StoreOpen::Create),
        (fixture.config.projection_store.clone(), StoreOpen::Create),
    ])
    .unwrap();
    assert!(
        SqliteRecordsLog::open_with_access(
            &fixture.config.projection_store,
            set.guard(&fixture.config.action_log).unwrap()
        )
        .is_err()
    );
    assert_eq!(
        std::fs::metadata(&fixture.config.projection_store)
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn a_failed_native_model_constructor_releases_its_guard_clone() {
    let fixture = Fixture::new();
    fixture.initialize();
    let set = LocalStoreSet::exclusive(&[(fixture.model_path(), StoreOpen::Existing)]).unwrap();
    let mut wrong = Fixture::baseline();
    wrong.document.entities[0].display_name = "Different baseline".into();
    assert!(
        SqliteModelStore::open_existing_with_access(
            &fixture.model_path(),
            Fixture::binding(),
            wrong,
            set.guard(&fixture.model_path()).unwrap()
        )
        .is_err()
    );
    drop(set);
    assert!(fixture.open().head(&Fixture::caller()).is_ok());
}
