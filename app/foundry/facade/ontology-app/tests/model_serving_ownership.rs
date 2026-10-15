#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "model_runtime/support.rs"]
mod support;
use foundry_ontology_app::{compose, compose_with_models};
use foundry_projection_sqlite_draft::SqliteProjectionStore;
use foundry_records_sqlite_draft::SqliteRecordsLog;
use support::Fixture;

#[test]
fn model_serving_refuses_a_standalone_native_handle_opened_first() {
    let fixture = Fixture::new();
    fixture.initialize();
    let standalone = fixture.open();
    assert!(compose_with_models(&fixture.config, Some(&fixture.models)).is_err());
    drop(standalone);
    assert!(compose_with_models(&fixture.config, Some(&fixture.models)).is_ok());
}

#[test]
fn model_serving_excludes_other_facades_and_every_native_writer() {
    let fixture = Fixture::new();
    fixture.initialize();
    let state = compose_with_models(&fixture.config, Some(&fixture.models)).unwrap();
    assert!(compose_with_models(&fixture.config, Some(&fixture.models)).is_err());
    assert!(SqliteRecordsLog::open(&fixture.config.action_log).is_err());
    assert!(SqliteRecordsLog::open(&fixture.config.denial_log).is_err());
    assert!(SqliteProjectionStore::open(&fixture.config.projection_store).is_err());
    assert!(
        foundry_model_sqlite_draft::SqliteModelStore::open_existing(
            &fixture.model_path(),
            Fixture::binding(),
            Fixture::baseline()
        )
        .is_err()
    );
    assert!(
        foundry_ontology_app::model_runtime::initialize_model(
            &fixture.config,
            &fixture.models,
            support::TENANT
        )
        .is_err()
    );
    drop(state);
    drop(fixture.open());
}

#[test]
fn only_verified_serving_composition_observes_an_installed_generation() {
    let fixture = Fixture::new();
    fixture.initialize();
    let mut state = compose_with_models(&fixture.config, Some(&fixture.models)).unwrap();
    let tenant = state.tenants.get_mut(support::TENANT).unwrap().get_mut();
    assert_eq!(tenant.observed_model_generation(), Some(1));
    assert!(tenant.sync_status(support::TENANT).is_ok());
    let registry = tenant.projection().registry_input.clone();
    tenant.inject_registry(registry);
    assert_eq!(tenant.observed_model_generation(), None);
    assert!(tenant.sync_status(support::TENANT).is_err());
    drop(state);

    let mut legacy = compose(&fixture.config).unwrap();
    let model = fixture.open();
    legacy
        .inject_established_model_store(
            support::TENANT,
            Fixture::binding(),
            Fixture::baseline(),
            Box::new(model),
        )
        .unwrap();
    let tenant = legacy.tenants.get_mut(support::TENANT).unwrap().get_mut();
    assert_eq!(tenant.observed_model_generation(), None);
    assert!(tenant.sync_status(support::TENANT).is_ok());
}

#[test]
fn every_supported_mutable_native_input_invalidates_installed_observation() {
    for replacement in 0..5 {
        let fixture = Fixture::new();
        fixture.initialize();
        let mut state = compose_with_models(&fixture.config, Some(&fixture.models)).unwrap();
        let tenant = state.tenants.get_mut(support::TENANT).unwrap().get_mut();
        assert_eq!(tenant.observed_model_generation(), Some(1));
        match replacement {
            0 => {
                tenant.projection_for_injection();
            }
            1 => {
                tenant.projection_store_for_injection();
            }
            2 => {
                tenant.replace_projection_store(Box::new(
                    foundry_projection_draft::MemoryProjectionStore::default(),
                ));
            }
            3 => {
                tenant.replace_action_log(Box::new(
                    SqliteRecordsLog::open(&fixture.directory.join("replacement.sqlite")).unwrap(),
                ));
            }
            _ => {
                tenant.replace_denial_log(Box::new(
                    SqliteRecordsLog::open(&fixture.directory.join("replacement.sqlite")).unwrap(),
                ));
            }
        }
        assert_eq!(tenant.observed_model_generation(), None);
        assert!(tenant.sync_status(support::TENANT).is_err());
    }
}

#[test]
fn failed_composition_releases_the_complete_partial_native_set() {
    let fixture = Fixture::new();
    fixture.initialize();
    let blocker = SqliteProjectionStore::open(&fixture.config.projection_store).unwrap();
    assert!(compose_with_models(&fixture.config, Some(&fixture.models)).is_err());
    drop(fixture.open());
    drop(SqliteRecordsLog::open(&fixture.config.action_log).unwrap());
    drop(SqliteRecordsLog::open(&fixture.config.denial_log).unwrap());
    drop(blocker);
    assert!(compose_with_models(&fixture.config, Some(&fixture.models)).is_ok());
}
