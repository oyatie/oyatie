use compute_k8s_api::{
    CloudComputeK8sLifecycleRepository, CloudComputeK8sLifecycleRepositoryError,
};
use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleRepository, PgK8sLifecycleRuntimeContract,
};
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::support::{create_command, delete_command, setup_schema};

pub(super) async fn assert_runtime_path_preserved(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    let app = PgPoolOptions::new()
        .max_connections(1)
        .connect_with((*app.connect_options()).clone())
        .await
        .expect("dedicated runtime pool");
    sqlx::query("SET search_path = public")
        .execute(&app)
        .await
        .expect("caller path");
    let repository = PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("admit runtime before test instrumentation");
    sqlx::query(
        "CREATE FUNCTION compute_k8s_lifecycle.check_transaction_path() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF pg_catalog.current_setting('search_path') <> 'pg_catalog, pg_temp' THEN RAISE EXCEPTION 'unexpected transaction search_path'; END IF; RETURN NEW; END $$",
    ).execute(setup).await.expect("install benign transaction-setting assertion");
    sqlx::query(
        "CREATE TRIGGER check_transaction_path BEFORE INSERT OR UPDATE ON compute_k8s_lifecycle.operations FOR EACH ROW EXECUTE FUNCTION compute_k8s_lifecycle.check_transaction_path()",
    ).execute(setup).await.expect("observe actual repository transaction setting");
    repository
        .commit_create(create_command("ten_path", "path", "create-path"))
        .await
        .expect("create uses trusted path");
    assert_caller_state(&app).await;
    repository
        .commit_deletion(delete_command("ten_path", "path", "delete-path"))
        .await
        .expect("deletion uses trusted path");
    assert_caller_state(&app).await;
    assert_eq!(
        repository
            .commit_deletion(delete_command("ten_path", "missing", "missing-path"))
            .await,
        Err(CloudComputeK8sLifecycleRepositoryError::ClusterNotFound)
    );
    assert_caller_state(&app).await;
    app.close().await;
}

async fn assert_caller_state(app: &PgPool) {
    let path: String = sqlx::query_scalar("SHOW search_path")
        .fetch_one(app)
        .await
        .expect("caller path restored");
    assert_eq!(path, "public");
    let tenant: Option<String> =
        sqlx::query_scalar("SELECT pg_catalog.current_setting('oyatie.tenant_id', true)")
            .fetch_one(app)
            .await
            .expect("tenant scope cleared");
    assert!(tenant.is_none_or(|value| value.is_empty()));
}
