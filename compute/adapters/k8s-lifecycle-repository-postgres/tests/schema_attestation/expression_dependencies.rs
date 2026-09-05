use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleConnectError, PgK8sLifecycleMigrationError, PgK8sLifecycleMigrator,
    PgK8sLifecycleRepository, PgK8sLifecycleRuntimeContract, PgK8sLifecycleSchemaError,
};
use sqlx::PgPool;

use crate::support::{reset_runtime_role, setup_schema};

pub(super) async fn assert_expression_dependencies(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    sqlx::query("CREATE FUNCTION public.lifecycle_clock() RETURNS timestamptz LANGUAGE sql STABLE AS 'SELECT pg_catalog.now()'")
        .execute(setup).await.expect("ordinary application clock function");
    sqlx::query("CREATE FUNCTION public.lifecycle_nonempty(value text) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$ SELECT value <> '' $$")
        .execute(setup).await.expect("ordinary application validation function");
    let mut refusals = Vec::new();
    for change in [
        "ALTER TABLE compute_k8s_lifecycle.clusters ALTER COLUMN updated_at SET DEFAULT public.lifecycle_clock()",
        "ALTER TABLE compute_k8s_lifecycle.clusters DROP CONSTRAINT clusters_desired_state_check, ADD CONSTRAINT clusters_desired_state_check CHECK (public.lifecycle_nonempty(desired_state))",
        "ALTER POLICY clusters_tenant_isolation ON compute_k8s_lifecycle.clusters USING (public.lifecycle_nonempty(tenant_id))",
    ] {
        setup_schema(setup, app_role).await;
        sqlx::query(change)
            .execute(setup)
            .await
            .expect("ordinary noncatalog expression dependency");
        let startup_refused = matches!(
            PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
            Err(PgK8sLifecycleConnectError::Schema(
                PgK8sLifecycleSchemaError::ExpressionDependencyContract
            ))
        );
        let migration_refused = matches!(
            PgK8sLifecycleMigrator::from_pool(setup.clone())
                .migrate()
                .await,
            Err(PgK8sLifecycleMigrationError::Schema(
                PgK8sLifecycleSchemaError::ExpressionDependencyContract
            ))
        );
        refusals.push(startup_refused && migration_refused);
    }
    reset_runtime_role(setup).await;
    sqlx::query("DROP FUNCTION public.lifecycle_clock(), public.lifecycle_nonempty(text)")
        .execute(setup)
        .await
        .expect("remove ordinary test functions");
    assert_eq!(
        refusals, [true; 3],
        "defaults, constraints and policies require catalog dependencies"
    );
}
