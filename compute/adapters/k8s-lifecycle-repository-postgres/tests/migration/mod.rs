mod caller_path;
mod ledger_grants;
mod runtime_path;

use compute_k8s_lifecycle_repository_postgres::{
    CURRENT_MIGRATION_VERSION, MIGRATIONS_TABLE, PgK8sLifecycleConnectError,
    PgK8sLifecycleMigrationError, PgK8sLifecycleMigrator, PgK8sLifecycleRepository,
    PgK8sLifecycleRuntimeContract, SCHEMA_NAME,
};
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::support::{
    apply_unversioned_schema, grant_runtime_role, reset_runtime_role, reset_schema, setup_schema,
};

pub(crate) async fn assert_migration_contract(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    ledger_grants::assert_ledger_grants(setup, app, app_role, runtime_contract).await;
    caller_path::assert_caller_path_preserved(setup, app, app_role, runtime_contract).await;
    runtime_path::assert_runtime_path_preserved(setup, app, app_role, runtime_contract).await;
    concurrent_migrators_serialize(setup, app, app_role, runtime_contract).await;
    runtime_refuses_incomplete_and_drifted_ledgers(setup, app, app_role, runtime_contract).await;
    migrator_adopts_only_exact_unversioned_schema(setup, app, app_role, runtime_contract).await;
    failed_adoption_rolls_back_ledger(setup).await;
}

async fn concurrent_migrators_serialize(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    reset_runtime_role(setup).await;
    let inherited_isolation = PgPoolOptions::new()
        .max_connections(2)
        .after_connect(|connection, _| {
            Box::pin(async move {
                sqlx::query("SET default_transaction_isolation = 'repeatable read'")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect_with((*setup.connect_options()).clone())
        .await
        .expect("migration caller defaults to repeatable read");
    let left = PgK8sLifecycleMigrator::from_pool(inherited_isolation.clone());
    let right = left.clone();
    let (left, right) = tokio::join!(left.migrate(), right.migrate());
    let reports = [
        left.expect("left migration"),
        right.expect("right migration"),
    ];
    assert_eq!(
        reports
            .iter()
            .filter(|report| report.applied_versions == [1, 2])
            .count(),
        1
    );
    assert_eq!(
        reports
            .iter()
            .filter(|report| report.applied_versions.is_empty())
            .count(),
        1
    );
    assert!(reports.iter().all(|report| {
        report.current_version == CURRENT_MIGRATION_VERSION && !report.adopted_unversioned_schema
    }));
    grant_runtime_role(setup, app_role).await;
    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("runtime accepts the fully migrated schema");
    let isolation: String = sqlx::query_scalar("SHOW default_transaction_isolation")
        .fetch_one(&inherited_isolation)
        .await
        .expect("caller isolation preserved");
    assert_eq!(isolation, "repeatable read");
    inherited_isolation.close().await;
}

async fn runtime_refuses_incomplete_and_drifted_ledgers(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query(&format!("DROP TABLE {MIGRATIONS_TABLE}"))
        .execute(setup)
        .await
        .expect("drop ledger");
    assert!(matches!(
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
        Err(PgK8sLifecycleConnectError::Schema(_))
    ));

    setup_schema(setup, app_role).await;
    sqlx::query(&format!("DELETE FROM {MIGRATIONS_TABLE} WHERE version = 2"))
        .execute(setup)
        .await
        .expect("remove latest ledger row");
    assert!(matches!(
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
        Err(PgK8sLifecycleConnectError::Schema(_))
    ));

    setup_schema(setup, app_role).await;
    sqlx::query(&format!(
        "INSERT INTO {MIGRATIONS_TABLE} (version, name, sha256) VALUES (3, 'unsupported-future', repeat('a', 64))"
    ))
    .execute(setup)
    .await
    .expect("inject future migration row");
    assert!(matches!(
        PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await,
        Err(PgK8sLifecycleMigrationError::DatabaseAhead {
            observed: 3,
            supported: CURRENT_MIGRATION_VERSION
        })
    ));
    assert!(matches!(
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
        Err(PgK8sLifecycleConnectError::Schema(_))
    ));

    setup_schema(setup, app_role).await;
    sqlx::query(&format!(
        "UPDATE {MIGRATIONS_TABLE} SET sha256 = repeat('0', 64) WHERE version = 2"
    ))
    .execute(setup)
    .await
    .expect("tamper migration digest");
    assert!(matches!(
        PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await,
        Err(PgK8sLifecycleMigrationError::AppliedMigrationDrift { version: 2 })
    ));
    assert!(matches!(
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
        Err(PgK8sLifecycleConnectError::Schema(_))
    ));
}

async fn migrator_adopts_only_exact_unversioned_schema(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    reset_schema(setup).await;
    apply_unversioned_schema(setup).await;
    let report = PgK8sLifecycleMigrator::from_pool(setup.clone())
        .migrate()
        .await
        .expect("exact unversioned schema is adopted");
    assert!(report.adopted_unversioned_schema);
    assert!(report.applied_versions.is_empty());
    grant_runtime_role(setup, app_role).await;
    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("adopted unversioned schema passes runtime attestation");

    reset_runtime_role(setup).await;
    sqlx::query(&format!("CREATE SCHEMA {SCHEMA_NAME}"))
        .execute(setup)
        .await
        .expect("install partial unversioned schema namespace");
    sqlx::query(&format!("CREATE TABLE {SCHEMA_NAME}.clusters (id bigint)"))
        .execute(setup)
        .await
        .expect("install partial unversioned schema");
    assert_eq!(
        PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await,
        Err(PgK8sLifecycleMigrationError::SchemaStateAmbiguous)
    );
    let ledger: Option<String> = sqlx::query_scalar("SELECT to_regclass($1)::text")
        .bind(MIGRATIONS_TABLE)
        .fetch_one(setup)
        .await
        .expect("probe ledger after refusal");
    assert!(ledger.is_none());
}

async fn failed_adoption_rolls_back_ledger(setup: &PgPool) {
    reset_runtime_role(setup).await;
    PgK8sLifecycleMigrator::from_pool(setup.clone())
        .migrate()
        .await
        .expect("prepare exact schema");
    sqlx::query(&format!("DELETE FROM {MIGRATIONS_TABLE}"))
        .execute(setup)
        .await
        .expect("prepare empty adoption ledger");
    sqlx::query(&format!(
        "CREATE FUNCTION {SCHEMA_NAME}.reject_ledger_write() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected ledger write failure'; END $$"
    ))
    .execute(setup)
    .await
    .expect("install ledger failure function");
    sqlx::query(&format!(
        "CREATE TRIGGER reject_ledger_write BEFORE INSERT ON {MIGRATIONS_TABLE} FOR EACH ROW EXECUTE FUNCTION {SCHEMA_NAME}.reject_ledger_write()"
    ))
    .execute(setup)
    .await
    .expect("install ledger failure trigger");
    assert!(matches!(
        PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await,
        Err(PgK8sLifecycleMigrationError::Sqlx(_))
    ));
    let rows: i64 = sqlx::query_scalar(&format!("SELECT count(*)::bigint FROM {MIGRATIONS_TABLE}"))
        .fetch_one(setup)
        .await
        .expect("count rolled-back ledger rows");
    assert_eq!(rows, 0);
}
