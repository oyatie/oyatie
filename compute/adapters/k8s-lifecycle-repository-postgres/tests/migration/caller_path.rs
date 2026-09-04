use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleConnectError, PgK8sLifecycleMigrationError, PgK8sLifecycleMigrator,
    PgK8sLifecycleRepository, PgK8sLifecycleRuntimeContract, PgK8sLifecycleSchemaError,
};
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::support::{grant_runtime_role, reset_runtime_role};

pub(super) async fn assert_caller_path_preserved(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    let setup = single_connection_pool(setup).await;
    let app = single_connection_pool(app).await;
    for path in [
        "public",
        "compute_k8s_lifecycle, public, pg_catalog",
        "pg_temp, public",
    ] {
        for pool in [&setup, &app] {
            sqlx::query("SELECT pg_catalog.set_config('search_path', $1, false)")
                .bind(path)
                .execute(pool)
                .await
                .expect("configure caller-owned connection");
        }
        reset_runtime_role(&setup).await;
        PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await
            .expect("migration accepts ordinary caller search path");
        assert_path(&setup, path).await;
        grant_runtime_role(&setup, app_role).await;
        let repository = PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
            .await
            .expect("startup accepts ordinary caller search path");
        repository
            .assert_rls_enforceable()
            .await
            .expect("public RLS probe");
        assert_path(&app, path).await;

        sqlx::query(
            "ALTER TABLE compute_k8s_lifecycle.clusters ALTER COLUMN updated_at DROP DEFAULT",
        )
        .execute(&setup)
        .await
        .expect("introduce ordinary default drift");
        assert_eq!(
            PgK8sLifecycleMigrator::from_pool(setup.clone())
                .migrate()
                .await,
            Err(PgK8sLifecycleMigrationError::Schema(
                PgK8sLifecycleSchemaError::ColumnContract
            ))
        );
        assert_path(&setup, path).await;
        assert!(matches!(
            PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
            Err(PgK8sLifecycleConnectError::Schema(
                PgK8sLifecycleSchemaError::ColumnContract
            ))
        ));
        assert_path(&app, path).await;
    }
    setup.close().await;
    app.close().await;
}

async fn single_connection_pool(source: &PgPool) -> PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_with((*source.connect_options()).clone())
        .await
        .expect("connect dedicated caller-owned pool")
}

async fn assert_path(pool: &PgPool, expected: &str) {
    let actual: String = sqlx::query_scalar("SHOW search_path")
        .fetch_one(pool)
        .await
        .expect("read caller path after transaction completion");
    assert_eq!(actual, expected);
}
