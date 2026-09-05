mod expression_dependencies;
mod expression_identity;
mod index_placement;
mod relation_properties;

use compute_k8s_lifecycle_repository_postgres::{
    MIGRATIONS_TABLE, PgK8sLifecycleConnectError, PgK8sLifecycleMigrationError,
    PgK8sLifecycleMigrator, PgK8sLifecycleRepository, PgK8sLifecycleRuntimeContract,
    PgK8sLifecycleSchemaError, SCHEMA_NAME,
};
use sqlx::PgPool;

use crate::support::{quote_identifier, setup_schema};

pub(crate) async fn assert_structural_drift_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    index_placement::assert_index_placement(setup, app, app_role, runtime_contract).await;
    relation_properties::assert_relation_properties(setup, app, app_role, runtime_contract).await;
    expression_identity::assert_expression_identity(setup, app, app_role, runtime_contract).await;
    expression_dependencies::assert_expression_dependencies(setup, app, app_role, runtime_contract)
        .await;
    column_collation_drift_is_refused(setup, app, app_role, runtime_contract).await;
    same_name_constraint_weakening_is_refused(setup, app, app_role, runtime_contract).await;
    same_name_index_drift_is_refused(setup, app, app_role, runtime_contract).await;
    policy_predicate_drift_is_refused(setup, app, app_role, runtime_contract).await;
    column_default_drift_is_refused(setup, app, app_role, runtime_contract).await;
    unexpected_namespace_object_is_refused(setup, app, app_role, runtime_contract).await;
    unexpected_read_grant_is_refused(setup, app, app_role, runtime_contract).await;
    unexpected_column_grant_is_refused(setup, app, app_role, runtime_contract).await;
}

async fn column_collation_drift_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query(
        "ALTER TABLE compute_k8s_lifecycle.operations ALTER COLUMN receipt_digest TYPE text COLLATE pg_catalog.\"C\"",
    )
    .execute(setup)
    .await
    .expect("change ordinary text column collation");
    assert_schema_error(
        app,
        runtime_contract,
        PgK8sLifecycleSchemaError::ColumnContract,
    )
    .await;
    assert_eq!(
        PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await,
        Err(PgK8sLifecycleMigrationError::Schema(
            PgK8sLifecycleSchemaError::ColumnContract
        ))
    );
    sqlx::query(
        "ALTER TABLE compute_k8s_lifecycle.operations ALTER COLUMN receipt_digest TYPE text COLLATE pg_catalog.\"default\"",
    )
    .execute(setup)
    .await
    .expect("restore database default collation");
    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("restored collation admits startup");
}

async fn same_name_constraint_weakening_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query(
        "ALTER TABLE compute_k8s_lifecycle.clusters DROP CONSTRAINT clusters_data_class_matches_desired",
    )
    .execute(setup)
    .await
    .expect("drop exact constraint");
    sqlx::query(
        "ALTER TABLE compute_k8s_lifecycle.clusters ADD CONSTRAINT clusters_data_class_matches_desired CHECK (true)",
    )
    .execute(setup)
    .await
    .expect("replace constraint with same-name weaker definition");
    assert_eq!(
        PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await,
        Err(PgK8sLifecycleMigrationError::Schema(
            PgK8sLifecycleSchemaError::ConstraintContract
        ))
    );
    assert_schema_error(
        app,
        runtime_contract,
        PgK8sLifecycleSchemaError::ConstraintContract,
    )
    .await;
}

async fn same_name_index_drift_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query("DROP INDEX compute_k8s_lifecycle.clusters_reconciliation_scan")
        .execute(setup)
        .await
        .expect("drop exact reconciliation index");
    sqlx::query(
        "CREATE INDEX clusters_reconciliation_scan ON compute_k8s_lifecycle.clusters (resource_id)",
    )
    .execute(setup)
    .await
    .expect("replace index with same-name weaker definition");
    assert_schema_error(
        app,
        runtime_contract,
        PgK8sLifecycleSchemaError::IndexContract,
    )
    .await;
}

async fn policy_predicate_drift_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query(
        "ALTER POLICY clusters_tenant_isolation ON compute_k8s_lifecycle.clusters USING (true) WITH CHECK (true)",
    )
    .execute(setup)
    .await
    .expect("weaken tenant policy predicate");
    assert_schema_error(
        app,
        runtime_contract,
        PgK8sLifecycleSchemaError::PolicyContract,
    )
    .await;
}

async fn column_default_drift_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query("ALTER TABLE compute_k8s_lifecycle.clusters ALTER COLUMN updated_at DROP DEFAULT")
        .execute(setup)
        .await
        .expect("remove column default");
    assert_schema_error(
        app,
        runtime_contract,
        PgK8sLifecycleSchemaError::ColumnContract,
    )
    .await;
}

async fn unexpected_namespace_object_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query(&format!(
        "CREATE FUNCTION {SCHEMA_NAME}.unexpected_runtime_function() RETURNS boolean LANGUAGE sql IMMUTABLE AS 'SELECT true'"
    ))
    .execute(setup)
    .await
    .expect("inject unexpected namespace function");
    assert_schema_error(
        app,
        runtime_contract,
        PgK8sLifecycleSchemaError::NamespaceContract,
    )
    .await;
}

async fn unexpected_read_grant_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query(&format!(
        "GRANT SELECT ON {MIGRATIONS_TABLE} TO {}",
        quote_identifier(app_role)
    ))
    .execute(setup)
    .await
    .expect("inject direct runtime read grant");
    assert_schema_error(
        app,
        runtime_contract,
        PgK8sLifecycleSchemaError::GrantContract,
    )
    .await;
}

async fn unexpected_column_grant_is_refused(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    sqlx::query(&format!(
        "GRANT SELECT (name) ON {MIGRATIONS_TABLE} TO {}",
        quote_identifier(app_role)
    ))
    .execute(setup)
    .await
    .expect("inject direct runtime column grant");
    assert_schema_error(
        app,
        runtime_contract,
        PgK8sLifecycleSchemaError::GrantContract,
    )
    .await;
}

async fn assert_schema_error(
    app: &PgPool,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
    expected: PgK8sLifecycleSchemaError,
) {
    assert!(matches!(
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
        Err(PgK8sLifecycleConnectError::Schema(actual)) if actual == expected
    ));
}
