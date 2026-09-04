use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleConnectError, PgK8sLifecycleRepository, PgK8sLifecycleRuntimeContract,
    PgK8sLifecycleSchemaError,
};
use sqlx::PgPool;

use crate::support::setup_schema;

pub(super) async fn assert_expression_identity(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    for (change, expected) in [
        (
            "ALTER TABLE compute_k8s_lifecycle.clusters ALTER COLUMN updated_at SET DEFAULT pg_catalog.transaction_timestamp()",
            PgK8sLifecycleSchemaError::ColumnContract,
        ),
        (
            "ALTER TABLE compute_k8s_lifecycle.operations ALTER COLUMN receipt_digest TYPE pg_catalog.varchar",
            PgK8sLifecycleSchemaError::ColumnContract,
        ),
        (
            "ALTER TABLE compute_k8s_lifecycle.clusters DROP CONSTRAINT clusters_database_schema_version, ADD CONSTRAINT clusters_database_schema_version CHECK (schema_version OPERATOR(pg_catalog.<=) 1)",
            PgK8sLifecycleSchemaError::ConstraintContract,
        ),
        (
            "ALTER POLICY clusters_tenant_isolation ON compute_k8s_lifecycle.clusters USING (tenant_id = pg_catalog.current_setting('oyatie.tenant_id'))",
            PgK8sLifecycleSchemaError::PolicyContract,
        ),
        (
            "ALTER TABLE compute_k8s_lifecycle.operations DROP CONSTRAINT operations_receipt_digest_shape, ADD CONSTRAINT operations_receipt_digest_shape CHECK (receipt_digest IS NULL OR receipt_digest COLLATE pg_catalog.\"C\" ~ '^[0-9a-f]{64}$')",
            PgK8sLifecycleSchemaError::ConstraintContract,
        ),
    ] {
        setup_schema(setup, app_role).await;
        sqlx::query(change)
            .execute(setup)
            .await
            .expect("ordinary built-in expression identity change");
        assert!(
            matches!(
                PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
                Err(PgK8sLifecycleConnectError::Schema(actual)) if actual == expected
            ),
            "changed built-in identity must be refused: {change}"
        );
    }
    setup_schema(setup, app_role).await;
    let builtin_types: bool = sqlx::query_scalar(
        "SELECT count(*) = 5 AND bool_and(a.atttypid = expected.type_name::regtype) FROM (VALUES ('schema_migrations', 'version', 'pg_catalog.int8'), ('operations', 'schema_version', 'pg_catalog.int4'), ('operations', 'receipt_json', 'pg_catalog.jsonb'), ('operations', 'receipt_digest', 'pg_catalog.text'), ('operations', 'created_at', 'pg_catalog.timestamptz')) AS expected(table_name, column_name, type_name) JOIN pg_catalog.pg_namespace n ON n.nspname = 'compute_k8s_lifecycle' JOIN pg_catalog.pg_class c ON c.relnamespace = n.oid AND c.relname = expected.table_name JOIN pg_catalog.pg_attribute a ON a.attrelid = c.oid AND a.attname = expected.column_name",
    ).fetch_one(setup).await.expect("resolve native built-in type identities");
    assert!(builtin_types);
    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("restored built-in identities admit startup");
}
