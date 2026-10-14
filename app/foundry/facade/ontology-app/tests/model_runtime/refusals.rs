use crate::support::*;
use foundry_model_api::{ModelBaseline, ModelStore};
use foundry_model_sqlite_draft::SqliteModelStore;
use foundry_ontology_app::{BootError, compose_with_models, model_runtime::ModelRuntimeConfig};

#[test]
fn model_configuration_refuses_partial_ambiguous_and_untrusted_inputs() {
    let fixture = Fixture::new();
    let valid =
        serde_json::json!([{"path":fixture.model_path(),"binding":Fixture::binding()}]).to_string();
    assert!(
        ModelRuntimeConfig::from_values(None, None, None)
            .unwrap()
            .is_none()
    );
    for (models, origin, local) in [
        (None, Some(ORIGIN), Some("true")),
        (Some(valid.as_str()), None, None),
        (None, None, Some("true")),
        (None, None, Some("yes")),
        (Some(""), Some(ORIGIN), Some("true")),
        (Some("[]"), Some(ORIGIN), Some("true")),
        (Some("{}"), Some(ORIGIN), Some("true")),
        (Some(valid.as_str()), Some(ORIGIN), None),
        (
            Some(valid.as_str()),
            Some("http://other.example"),
            Some("true"),
        ),
        (
            Some(valid.as_str()),
            Some("https://shell.example/path"),
            None,
        ),
        (
            Some(valid.as_str()),
            Some("https://user@shell.example"),
            None,
        ),
    ] {
        assert!(ModelRuntimeConfig::from_values(models, origin, local).is_err());
    }
    for malformed in [
        valid.replace("\"binding\":", "\"owner\":\"usr_other\",\"binding\":"),
        valid.replace("\"store_id\":", "\"roles\":[],\"store_id\":"),
        valid.replace("\"path\":", "\"path\":\"duplicate\",\"path\":"),
    ] {
        assert!(
            ModelRuntimeConfig::from_values(Some(&malformed), Some(ORIGIN), Some("true")).is_err()
        );
    }
    assert!(
        ModelRuntimeConfig::from_values(Some(&valid), Some("https://shell.example"), None)
            .unwrap()
            .is_some()
    );
}

#[test]
fn model_bindings_must_be_served_unique_and_distinct_from_all_native_stores() {
    let fixture = Fixture::new();
    let mut repeated = fixture.config.clone();
    repeated.tenants.push(TENANT.into());
    assert!(fixture.models.validate(&repeated).is_err());
    let mut models = fixture.models.clone();
    models.stores[0].binding.tenant_id = "ten_unserved".into();
    assert!(models.validate(&fixture.config).is_err());
    let mut models = fixture.models.clone();
    models.stores.push(models.stores[0].clone());
    assert!(models.validate(&fixture.config).is_err());
    let mut models = fixture.models.clone();
    models.stores[0].path.clear();
    assert!(models.validate(&fixture.config).is_err());
    for path in [
        &fixture.config.action_log,
        &fixture.config.denial_log,
        &fixture.config.projection_store,
    ] {
        let mut models = fixture.models.clone();
        models.stores[0].path = path.clone();
        assert!(matches!(
            compose_with_models(&fixture.config, Some(&models)),
            Err(BootError::ModelConfigurationRefused { .. })
        ));
        assert!(
            !path.exists(),
            "alias refusal happens before any store opens"
        );
    }
    let mut models = fixture.models.clone();
    models.stores[0].binding.store_id.clear();
    assert!(models.validate(&fixture.config).is_err());
}

#[cfg(unix)]
#[test]
fn symlink_hardlink_and_relative_aliases_cannot_initialize_over_another_store() {
    let fixture = Fixture::new();
    std::fs::write(&fixture.config.action_log, b"preserve existing bytes").unwrap();
    let symbolic = fixture.directory.join("symbolic.sqlite");
    std::os::unix::fs::symlink(&fixture.config.action_log, &symbolic).unwrap();
    let hard = fixture.directory.join("hard.sqlite");
    std::fs::hard_link(&fixture.config.action_log, &hard).unwrap();
    std::fs::create_dir(fixture.directory.join("child")).unwrap();
    let relative = fixture.directory.join("child/../actions.sqlite");
    for path in [symbolic, hard, relative] {
        let mut models = fixture.models.clone();
        models.stores[0].path = path;
        assert!(
            foundry_ontology_app::model_runtime::initialize_model(&fixture.config, &models, TENANT)
                .is_err()
        );
        assert_eq!(
            std::fs::read(&fixture.config.action_log).unwrap(),
            b"preserve existing bytes"
        );
    }
}

#[test]
fn serving_never_initializes_missing_partial_or_mismatched_metadata() {
    let fixture = Fixture::new();
    assert!(matches!(
        compose_with_models(&fixture.config, Some(&fixture.models)),
        Err(BootError::ModelStoreRefused { .. })
    ));
    assert!(
        !fixture.model_path().exists(),
        "serve must not CREATE metadata"
    );
    std::fs::write(fixture.model_path(), b"").unwrap();
    assert!(matches!(
        compose_with_models(&fixture.config, Some(&fixture.models)),
        Err(BootError::ModelStoreRefused { .. })
    ));
    std::fs::remove_file(fixture.model_path()).unwrap();
    fixture.initialize();
    let mut wrong = fixture.models.clone();
    wrong.stores[0].binding.baseline_id = "another-independent-baseline".into();
    assert!(matches!(
        compose_with_models(&fixture.config, Some(&wrong)),
        Err(BootError::ModelStoreRefused { .. })
    ));
    assert_eq!(
        fixture.open().head(&Fixture::caller()).unwrap().generation,
        1
    );
    let other = Fixture::new();
    let mut baseline = Fixture::baseline();
    baseline.document.entities[0].display_name = "Another trusted model".into();
    drop(SqliteModelStore::initialize(&other.model_path(), Fixture::binding(), baseline).unwrap());
    assert!(matches!(
        compose_with_models(&other.config, Some(&other.models)),
        Err(BootError::ModelStoreRefused { .. })
    ));
    assert_eq!(
        ModelBaseline::capture(
            &foundry_ontology_app::seed::registry_for(TENANT).unwrap(),
            TENANT
        )
        .unwrap(),
        Fixture::baseline()
    );
}

#[test]
fn model_configured_serve_without_iam_refuses_before_creating_any_store() {
    let fixture = Fixture::new();
    let output = fixture.command().output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("requires explicit IAM"));
    for path in [
        &fixture.model_path(),
        &fixture.config.action_log,
        &fixture.config.denial_log,
        &fixture.config.projection_store,
    ] {
        assert!(!path.exists());
    }
}
