use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleConnectError, PgK8sLifecycleMigrationError, PgK8sLifecycleMigrator,
    PgK8sLifecycleRepository, PgK8sLifecycleRoleDatabaseClaimError, PgK8sLifecycleRuntimeContract,
    PgK8sLifecycleSchemaError,
};
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::support::{reset_runtime_role, setup_schema};

pub(crate) async fn assert_database_claim(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    reset_runtime_role(setup).await;
    sqlx::query("CREATE ROLE compute_k8s_lifecycle_runtime NOLOGIN")
        .execute(setup)
        .await
        .expect("ordinary precreated unclaimed role");
    let unclaimed_refused =
        migration_refused(setup, PgK8sLifecycleRoleDatabaseClaimError::Unclaimed).await;
    reset_runtime_role(setup).await;
    setup_schema(setup, app_role).await;
    sqlx::query("GRANT CREATE ON TABLESPACE pg_default TO compute_k8s_lifecycle_runtime")
        .execute(setup)
        .await
        .expect("ordinary shared tablespace dependency");
    let shared_refused = migration_refused(
        setup,
        PgK8sLifecycleRoleDatabaseClaimError::UnsupportedSharedDependency,
    )
    .await
        && matches!(
            PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
            Err(PgK8sLifecycleConnectError::Schema(
                PgK8sLifecycleSchemaError::RuntimeRoleDatabaseClaim(
                    PgK8sLifecycleRoleDatabaseClaimError::UnsupportedSharedDependency
                )
            ))
        );
    sqlx::query("REVOKE CREATE ON TABLESPACE pg_default FROM compute_k8s_lifecycle_runtime")
        .execute(setup)
        .await
        .expect("restore shared tablespace contract");
    let mut foreign_refusals = Vec::new();
    for change in [
        "GRANT SELECT ON public.claim_probe TO compute_k8s_lifecycle_runtime",
        "ALTER TABLE public.claim_probe OWNER TO compute_k8s_lifecycle_runtime",
        "CREATE POLICY claim_policy ON public.claim_probe TO compute_k8s_lifecycle_runtime USING (true)",
        "GRANT CONNECT ON DATABASE compute_claim_foreign TO compute_k8s_lifecycle_runtime",
        "ALTER DATABASE compute_claim_foreign OWNER TO compute_k8s_lifecycle_runtime",
    ] {
        let foreign = create_database(setup, "compute_claim_foreign").await;
        sqlx::query("CREATE TABLE public.claim_probe (id integer)")
            .execute(&foreign)
            .await
            .expect("ordinary foreign database table");
        sqlx::query(change)
            .execute(&foreign)
            .await
            .expect("ordinary foreign database dependency");
        foreign_refusals.push(
            migration_refused(
                setup,
                PgK8sLifecycleRoleDatabaseClaimError::ForeignOrUnresolvedDatabase,
            )
            .await
                && matches!(
                    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
                    Err(PgK8sLifecycleConnectError::Schema(
                        PgK8sLifecycleSchemaError::RuntimeRoleDatabaseClaim(
                            PgK8sLifecycleRoleDatabaseClaimError::ForeignOrUnresolvedDatabase
                        )
                    ))
                ),
        );
        foreign.close().await;
        drop_database(setup, "compute_claim_foreign").await;
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
            .await
            .expect("current database claim remains admitted after foreign dependency removal");
    }
    reset_runtime_role(setup).await;
    let first = create_database(setup, "compute_claim_first").await;
    let second = create_database(setup, "compute_claim_second").await;
    let left = PgK8sLifecycleMigrator::from_pool(first.clone());
    let right = PgK8sLifecycleMigrator::from_pool(second.clone());
    let (left_result, right_result) = tokio::join!(left.migrate(), right.migrate());
    let winners = usize::from(left_result.is_ok()) + usize::from(right_result.is_ok());
    let loser = if left_result.is_ok() { &second } else { &first };
    let retry_refused = migration_refused(
        loser,
        PgK8sLifecycleRoleDatabaseClaimError::ForeignOrUnresolvedDatabase,
    )
    .await;
    let loser_schema: Option<String> =
        sqlx::query_scalar("SELECT pg_catalog.to_regnamespace('compute_k8s_lifecycle')::text")
            .fetch_one(loser)
            .await
            .expect("probe rolled-back losing installation");
    first.close().await;
    second.close().await;
    drop_database(setup, "compute_claim_first").await;
    drop_database(setup, "compute_claim_second").await;
    reset_runtime_role(setup).await;
    assert_eq!(
        (
            unclaimed_refused,
            shared_refused,
            foreign_refusals,
            winners,
            retry_refused,
            loser_schema.is_none()
        ),
        (true, true, vec![true; 5], 1, true, true),
        "unclaimed/foreign roles refuse; exactly one fresh install commits; loser retry refuses with no schema"
    );
}

async fn migration_refused(pool: &PgPool, expected: PgK8sLifecycleRoleDatabaseClaimError) -> bool {
    matches!(PgK8sLifecycleMigrator::from_pool(pool.clone()).migrate().await,
        Err(PgK8sLifecycleMigrationError::Schema(PgK8sLifecycleSchemaError::RuntimeRoleDatabaseClaim(actual))) if actual == expected)
}

async fn create_database(setup: &PgPool, name: &str) -> PgPool {
    sqlx::query(&format!(
        "CREATE DATABASE {}",
        crate::support::quote_identifier(name)
    ))
    .execute(setup)
    .await
    .expect("create disposable claim-test database");
    PgPoolOptions::new()
        .max_connections(2)
        .connect_with((*setup.connect_options()).clone().database(name))
        .await
        .expect("connect disposable claim-test database")
}

async fn drop_database(setup: &PgPool, name: &str) {
    sqlx::query(&format!(
        "DROP DATABASE {}",
        crate::support::quote_identifier(name)
    ))
    .execute(setup)
    .await
    .expect("remove disposable claim-test database");
}
