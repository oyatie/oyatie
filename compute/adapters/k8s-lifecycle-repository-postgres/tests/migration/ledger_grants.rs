use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleMigrationError, PgK8sLifecycleMigrator, PgK8sLifecycleRepository,
    PgK8sLifecycleRuntimeContract, PgK8sLifecycleSchemaError,
};
use sqlx::PgPool;

use crate::support::setup_schema;

const REVOKE_LEDGER_READ: &str =
    "REVOKE SELECT ON compute_k8s_lifecycle.schema_migrations FROM compute_k8s_lifecycle_runtime";
const GRANT_LEDGER_READ: &str =
    "GRANT SELECT ON compute_k8s_lifecycle.schema_migrations TO compute_k8s_lifecycle_runtime";

pub(super) async fn assert_ledger_grants(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    let mut outcomes = Vec::new();
    for prefix_length in [2_i64, 1, 0] {
        setup_schema(setup, app_role).await;
        if prefix_length < 2 {
            sqlx::query(
                "DROP TABLE compute_k8s_lifecycle.clusters, compute_k8s_lifecycle.operations",
            )
            .execute(setup)
            .await
            .expect("prepare ordinary earlier schema prefix");
            sqlx::query("DELETE FROM compute_k8s_lifecycle.schema_migrations WHERE version > $1")
                .bind(prefix_length)
                .execute(setup)
                .await
                .expect("prepare matching ledger prefix");
        }
        sqlx::query(REVOKE_LEDGER_READ)
            .execute(setup)
            .await
            .expect("revoke required ledger read grant");
        let before = snapshot(setup).await;
        let refused = matches!(
            PgK8sLifecycleMigrator::from_pool(setup.clone())
                .migrate()
                .await,
            Err(PgK8sLifecycleMigrationError::Schema(
                PgK8sLifecycleSchemaError::GrantContract
            ))
        );
        outcomes.push((
            refused,
            snapshot(setup).await == before,
            !ledger_readable(setup).await,
        ));

        sqlx::query(GRANT_LEDGER_READ)
            .execute(setup)
            .await
            .expect("explicitly restore ledger grant");
        PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await
            .expect("restored current or prefix ledger can migrate");
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
            .await
            .expect("explicitly restored grant admits startup");
    }
    setup_schema(setup, app_role).await;
    sqlx::query("DELETE FROM compute_k8s_lifecycle.schema_migrations")
        .execute(setup)
        .await
        .expect("prepare exact legacy schema with empty ledger");
    sqlx::query(REVOKE_LEDGER_READ)
        .execute(setup)
        .await
        .expect("ledger read not yet established for adoption");
    let adopted = PgK8sLifecycleMigrator::from_pool(setup.clone())
        .migrate()
        .await
        .expect("genuine legacy adoption establishes ledger grant atomically");
    assert!(adopted.adopted_unversioned_schema);
    assert!(adopted.applied_versions.is_empty());
    assert!(ledger_readable(setup).await);
    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("adopted ledger grant admits startup");
    assert_eq!(
        outcomes,
        [(true, true, true); 3],
        "current, prefix and empty non-adoption ledgers must refuse revoked grants without changing state"
    );
}

#[derive(Debug, Eq, PartialEq)]
struct SchemaSnapshot {
    ledger_rows: Vec<(i64, String, String, String)>,
    relations: Vec<String>,
    ledger_acl: String,
}

async fn snapshot(setup: &PgPool) -> SchemaSnapshot {
    SchemaSnapshot {
        ledger_rows: sqlx::query_as("SELECT version, name, sha256, applied_at::text FROM compute_k8s_lifecycle.schema_migrations ORDER BY version")
            .fetch_all(setup).await.expect("snapshot exact ledger rows"),
        relations: sqlx::query_scalar("SELECT c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'compute_k8s_lifecycle' ORDER BY c.relname")
            .fetch_all(setup).await.expect("snapshot native relation inventory"),
        ledger_acl: sqlx::query_scalar("SELECT relacl::text FROM pg_catalog.pg_class WHERE oid = 'compute_k8s_lifecycle.schema_migrations'::regclass")
            .fetch_one(setup).await.expect("snapshot exact ledger ACL"),
    }
}

async fn ledger_readable(setup: &PgPool) -> bool {
    sqlx::query_scalar("SELECT pg_catalog.has_table_privilege('compute_k8s_lifecycle_runtime', 'compute_k8s_lifecycle.schema_migrations', 'SELECT')")
        .fetch_one(setup).await.expect("inspect effective policy-role ledger read privilege")
}
