use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleConnectError, PgK8sLifecycleMigrationError, PgK8sLifecycleMigrator,
    PgK8sLifecycleRepository, PgK8sLifecycleRuntimeContract, PgK8sLifecycleSchemaError,
};
use sqlx::{Connection, PgConnection, PgPool};

use crate::support::{quote_identifier, setup_schema};

pub(super) async fn assert_index_placement(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    let mut admin = PgConnection::connect_with(setup.connect_options().as_ref())
        .await
        .expect("dedicated disposable tablespace connection");
    let original_setting: String = sqlx::query_scalar("SHOW allow_in_place_tablespaces")
        .fetch_one(&mut admin)
        .await
        .expect("save dedicated connection setting");
    let default_tablespace: String = sqlx::query_scalar(
        "SELECT t.spcname FROM pg_catalog.pg_database d JOIN pg_catalog.pg_tablespace t ON t.oid = d.dattablespace WHERE d.datname = pg_catalog.current_database()",
    ).fetch_one(&mut admin).await.expect("resolve database default placement");
    sqlx::query("SET allow_in_place_tablespaces = on")
        .execute(&mut admin)
        .await
        .expect("enable disposable-only PostgreSQL test facility");
    sqlx::query("CREATE TABLESPACE compute_lifecycle_index_placement LOCATION ''")
        .execute(&mut admin)
        .await
        .expect("create PostgreSQL-managed fixture tablespace");
    sqlx::query("SELECT pg_catalog.set_config('allow_in_place_tablespaces', $1, false)")
        .bind(&original_setting)
        .execute(&mut admin)
        .await
        .expect("restore connection setting");
    let restored_setting: String = sqlx::query_scalar("SHOW allow_in_place_tablespaces")
        .fetch_one(&mut admin)
        .await
        .expect("verify connection setting restored");
    assert_eq!(restored_setting, original_setting);

    let before = index_definition(&mut admin).await;
    sqlx::query("ALTER INDEX compute_k8s_lifecycle.clusters_reconciliation_scan SET TABLESPACE compute_lifecycle_index_placement")
        .execute(&mut admin).await.expect("relocate ordinary governed index");
    let relocated: bool = sqlx::query_scalar(
        "SELECT reltablespace <> 0 FROM pg_catalog.pg_class WHERE oid = 'compute_k8s_lifecycle.clusters_reconciliation_scan'::regclass",
    ).fetch_one(&mut admin).await.expect("observe native nondefault index placement");
    let after = index_definition(&mut admin).await;
    let startup_refused = matches!(
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
        Err(PgK8sLifecycleConnectError::Schema(
            PgK8sLifecycleSchemaError::IndexContract
        ))
    );
    let migration_refused = matches!(
        PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await,
        Err(PgK8sLifecycleMigrationError::Schema(
            PgK8sLifecycleSchemaError::IndexContract
        ))
    );

    sqlx::query(&format!(
        "ALTER INDEX compute_k8s_lifecycle.clusters_reconciliation_scan SET TABLESPACE {}",
        quote_identifier(&default_tablespace),
    ))
    .execute(&mut admin)
    .await
    .expect("restore database default index placement");
    sqlx::query("DROP TABLESPACE compute_lifecycle_index_placement")
        .execute(&mut admin)
        .await
        .expect("remove only the fixture tablespace");
    admin
        .close()
        .await
        .expect("close dedicated fixture connection");
    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("restored default placement admits startup");
    let report = PgK8sLifecycleMigrator::from_pool(setup.clone())
        .migrate()
        .await
        .expect("restored default placement admits migration");
    assert!(report.applied_versions.is_empty());
    assert!(
        relocated,
        "fixture must actually change physical index placement"
    );
    assert_eq!(
        before, after,
        "index SQL deparsing omits physical placement"
    );
    assert_eq!((startup_refused, migration_refused), (true, true));
}

async fn index_definition(admin: &mut PgConnection) -> String {
    sqlx::query_scalar("SELECT pg_catalog.pg_get_indexdef('compute_k8s_lifecycle.clusters_reconciliation_scan'::regclass)")
        .fetch_one(admin).await.expect("read ordinary index definition")
}
