use compute_k8s_lifecycle_repository_postgres::{
    MIGRATIONS_TABLE, PgK8sLifecycleConnectError, PgK8sLifecycleRepository,
    PgK8sLifecycleRuntimeContract, RUNTIME_ROLE, SCHEMA_NAME,
};
use sqlx::PgPool;

use crate::support::{quote_identifier, setup_schema};

pub(super) async fn assert_authority_is_lifecycle_scoped(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    schema_create_is_refused(setup, app, app_role, runtime_contract).await;
    database_create_is_refused(setup, app, app_role, runtime_contract).await;
    ledger_column_mutation_is_refused(setup, app, app_role, runtime_contract).await;
    inherited_cluster_roles_are_refused(setup, app, app_role, runtime_contract).await;
    external_table_access_is_refused(setup, app, app_role, runtime_contract).await;
    executable_security_definer_is_refused(setup, app, app_role, runtime_contract).await;
    parameter_authority_is_scoped(setup, app, app_role, runtime_contract).await;
}

async fn schema_create_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    let role = quote_identifier(app_role);
    sqlx::query(&format!("GRANT CREATE ON SCHEMA {SCHEMA_NAME} TO {role}"))
        .execute(setup)
        .await
        .expect("inject lifecycle DDL authority");
    assert_authority_error(app, app_role, runtime_contract).await;
    sqlx::query(&format!(
        "REVOKE CREATE ON SCHEMA {SCHEMA_NAME} FROM {role}"
    ))
    .execute(setup)
    .await
    .expect("remove lifecycle DDL authority");
}

async fn database_create_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(setup)
        .await
        .expect("read test database name");
    let role = quote_identifier(app_role);
    let database = quote_identifier(&database);
    sqlx::query(&format!("GRANT CREATE ON DATABASE {database} TO {role}"))
        .execute(setup)
        .await
        .expect("inject database-wide create authority");
    assert_authority_error(app, app_role, runtime_contract).await;
    sqlx::query(&format!("REVOKE CREATE ON DATABASE {database} FROM {role}"))
        .execute(setup)
        .await
        .expect("remove database-wide create authority");
}

async fn ledger_column_mutation_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    let role = quote_identifier(app_role);
    sqlx::query(&format!(
        "GRANT UPDATE (sha256) ON {MIGRATIONS_TABLE} TO {role}"
    ))
    .execute(setup)
    .await
    .expect("inject column-level ledger authority");
    assert_authority_error(app, app_role, runtime_contract).await;
    sqlx::query(&format!(
        "REVOKE UPDATE (sha256) ON {MIGRATIONS_TABLE} FROM {role}"
    ))
    .execute(setup)
    .await
    .expect("remove column-level ledger authority");
}

async fn inherited_cluster_roles_are_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    for inherited in ["pg_read_all_data", "pg_write_all_data"] {
        setup_schema(setup, app_role).await;
        let role = quote_identifier(app_role);
        sqlx::query(&format!("GRANT {inherited} TO {role}"))
            .execute(setup)
            .await
            .expect("inject broad inherited role");
        assert_graph_error(app, runtime_contract).await;
        sqlx::query(&format!("REVOKE {inherited} FROM {role}"))
            .execute(setup)
            .await
            .expect("remove broad inherited role");
    }

    setup_schema(setup, app_role).await;
    let role = quote_identifier(app_role);
    sqlx::query(&format!("GRANT {RUNTIME_ROLE} TO {role} WITH ADMIN OPTION"))
        .execute(setup)
        .await
        .expect("inject lifecycle delegation authority");
    assert_graph_error(app, runtime_contract).await;
    sqlx::query(&format!(
        "REVOKE ADMIN OPTION FOR {RUNTIME_ROLE} FROM {role}"
    ))
    .execute(setup)
    .await
    .expect("remove lifecycle delegation authority");
}

async fn external_table_access_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query("DROP SCHEMA IF EXISTS compute_k8s_external_authority CASCADE")
        .execute(setup)
        .await
        .expect("drop prior external authority fixture");
    sqlx::query("CREATE SCHEMA compute_k8s_external_authority")
        .execute(setup)
        .await
        .expect("create external authority fixture");
    sqlx::query("CREATE TABLE compute_k8s_external_authority.records (id bigint)")
        .execute(setup)
        .await
        .expect("create external table fixture");
    sqlx::query(&format!(
        "GRANT USAGE ON SCHEMA compute_k8s_external_authority TO {}",
        quote_identifier(app_role)
    ))
    .execute(setup)
    .await
    .expect("grant external schema usage");
    sqlx::query(&format!(
        "GRANT SELECT ON compute_k8s_external_authority.records TO {}",
        quote_identifier(app_role)
    ))
    .execute(setup)
    .await
    .expect("inject external table authority");
    assert_authority_error(app, app_role, runtime_contract).await;
    sqlx::query("DROP SCHEMA compute_k8s_external_authority CASCADE")
        .execute(setup)
        .await
        .expect("remove external authority fixture");
}

async fn executable_security_definer_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query(
        "CREATE FUNCTION public.oyatie_compute_migration_probe() RETURNS boolean LANGUAGE sql SECURITY DEFINER AS 'SELECT true'",
    )
    .execute(setup)
    .await
    .expect("inject executable security-definer function");
    assert_authority_error(app, app_role, runtime_contract).await;
    sqlx::query("DROP FUNCTION public.oyatie_compute_migration_probe()")
        .execute(setup)
        .await
        .expect("remove executable security-definer function");
}

async fn parameter_authority_is_scoped(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    for target in [app_role, RUNTIME_ROLE] {
        let target = quote_identifier(target);
        sqlx::query(&format!("GRANT SET ON PARAMETER work_mem TO {target}"))
            .execute(setup)
            .await
            .expect("inject ordinary session-parameter authority");
        assert_runtime_admitted(app, runtime_contract).await;
        sqlx::query(&format!("REVOKE SET ON PARAMETER work_mem FROM {target}"))
            .execute(setup)
            .await
            .expect("remove ordinary session-parameter authority");

        sqlx::query(&format!(
            "GRANT ALTER SYSTEM ON PARAMETER work_mem TO {target}"
        ))
        .execute(setup)
        .await
        .expect("inject cluster-wide parameter authority");
        assert_authority_error(app, app_role, runtime_contract).await;
        sqlx::query(&format!(
            "REVOKE ALTER SYSTEM ON PARAMETER work_mem FROM {target}"
        ))
        .execute(setup)
        .await
        .expect("remove cluster-wide parameter authority");
        assert_runtime_admitted(app, runtime_contract).await;

        sqlx::query(&format!(
            "GRANT SET ON PARAMETER session_replication_role TO {target}"
        ))
        .execute(setup)
        .await
        .expect("inject privileged session-parameter authority");
        assert_authority_error(app, app_role, runtime_contract).await;
        sqlx::query(&format!(
            "REVOKE SET ON PARAMETER session_replication_role FROM {target}"
        ))
        .execute(setup)
        .await
        .expect("remove privileged session-parameter authority");
        assert_runtime_admitted(app, runtime_contract).await;
    }
}

async fn assert_runtime_admitted(app: &PgPool, runtime_contract: &PgK8sLifecycleRuntimeContract) {
    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("runtime authority is admitted after remediation");
}

async fn assert_authority_error(
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    assert!(matches!(
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
        Err(PgK8sLifecycleConnectError::PrivilegedAuthorityPresent { role })
            if role == app_role
    ));
}

async fn assert_graph_error(app: &PgPool, runtime_contract: &PgK8sLifecycleRuntimeContract) {
    assert!(matches!(
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
        Err(PgK8sLifecycleConnectError::ServingRoleGraphMismatch)
    ));
}
