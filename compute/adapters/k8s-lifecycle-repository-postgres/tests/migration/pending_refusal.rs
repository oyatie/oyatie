use super::legacy_fixture::{execute, prefix};
use crate::support::{grant_runtime_role, setup_schema};
use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleMigrator, PgK8sLifecycleRepository, PgK8sLifecycleRuntimeContract,
};
use sqlx::PgPool;

pub(super) async fn assert_refusals(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    contract: &PgK8sLifecycleRuntimeContract,
) {
    failed_append_rolls_back(setup).await;
    for drift in [
        "ALTER TABLE compute_k8s_lifecycle.operations DROP COLUMN request_contract CASCADE",
        "ALTER TABLE compute_k8s_lifecycle.operations DROP COLUMN operation_state CASCADE",
        "ALTER TABLE compute_k8s_lifecycle.operations ADD COLUMN unexpected text",
        "ALTER TABLE compute_k8s_lifecycle.operations ALTER COLUMN request_contract DROP DEFAULT",
        "ALTER TABLE compute_k8s_lifecycle.operations ALTER COLUMN request_contract DROP NOT NULL",
        "ALTER TABLE compute_k8s_lifecycle.operations ALTER COLUMN operation_state SET DEFAULT 'accepted'",
        "ALTER TABLE compute_k8s_lifecycle.operations ALTER COLUMN operation_state SET NOT NULL",
        "ALTER TABLE compute_k8s_lifecycle.operations ALTER COLUMN operation_state TYPE varchar",
        "ALTER TABLE compute_k8s_lifecycle.operations ADD CONSTRAINT unexpected CHECK (operation_state IS NULL)",
        "ALTER TABLE compute_k8s_lifecycle.operations DROP CONSTRAINT operations_request_contract",
        "ALTER TABLE compute_k8s_lifecycle.operations DROP CONSTRAINT operations_contract_state",
        "ALTER TABLE compute_k8s_lifecycle.operations DROP CONSTRAINT operations_contract_state; ALTER TABLE compute_k8s_lifecycle.operations ADD CONSTRAINT operations_contract_state CHECK (true)",
    ] {
        setup_schema(setup, app_role).await;
        execute(setup, drift).await;
        let before = snapshot(setup).await;
        assert!(
            PgK8sLifecycleRepository::from_pool(app.clone(), contract)
                .await
                .is_err(),
            "{drift}"
        );
        assert!(
            PgK8sLifecycleMigrator::from_pool(setup.clone())
                .migrate()
                .await
                .is_err(),
            "{drift}"
        );
        assert_eq!(
            snapshot(setup).await,
            before,
            "refused drift remains unchanged"
        );
    }
    for length in [0, 1, 2] {
        prefix(setup, length).await;
        grant_runtime_role(setup, app_role).await;
        execute(setup, "ALTER TABLE compute_k8s_lifecycle.schema_migrations ALTER COLUMN applied_at DROP DEFAULT").await;
        let before = snapshot(setup).await;
        assert!(
            PgK8sLifecycleMigrator::from_pool(setup.clone())
                .migrate()
                .await
                .is_err()
        );
        assert_eq!(snapshot(setup).await, before);
    }
    prefix(setup, 0).await;
    execute(
        setup,
        "REVOKE USAGE ON SCHEMA compute_k8s_lifecycle FROM compute_k8s_lifecycle_runtime",
    )
    .await;
    let before = snapshot(setup).await;
    assert!(
        PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await
            .is_err(),
        "runtime migration must not repair a drifted prefix grant"
    );
    assert_eq!(snapshot(setup).await, before);
    for ledger_present in [true, false] {
        setup_schema(setup, app_role).await;
        execute(
            setup,
            if ledger_present {
                "DELETE FROM compute_k8s_lifecycle.schema_migrations"
            } else {
                "DROP TABLE compute_k8s_lifecycle.schema_migrations"
            },
        )
        .await;
        assert!(
            PgK8sLifecycleMigrator::from_pool(setup.clone())
                .migrate()
                .await
                .is_err(),
            "pending layout cannot masquerade as legacy adoption"
        );
        let ledger: Option<String> = sqlx::query_scalar(
            "SELECT to_regclass('compute_k8s_lifecycle.schema_migrations')::text",
        )
        .fetch_one(setup)
        .await
        .unwrap();
        assert_eq!(ledger.is_some(), ledger_present);
    }
}

async fn failed_append_rolls_back(setup: &PgPool) {
    for adoption in [false, true] {
        prefix(setup, 2).await;
        if adoption {
            execute(setup, "DELETE FROM compute_k8s_lifecycle.schema_migrations").await;
        }
        execute(setup, "CREATE FUNCTION public.compute_test_reject_pending_constraints() RETURNS event_trigger LANGUAGE plpgsql AS $$ BEGIN IF EXISTS (SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid = 'compute_k8s_lifecycle.operations'::regclass AND conname = 'operations_contract_state') THEN RAISE EXCEPTION 'injected pending constraint DDL failure after columns and ledger adoption'; END IF; END $$").await;
        execute(setup, "CREATE EVENT TRIGGER compute_test_pending_ddl_failure ON ddl_command_end WHEN TAG IN ('ALTER TABLE') EXECUTE FUNCTION public.compute_test_reject_pending_constraints()").await;
        let before = snapshot(setup).await;
        let result = PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await;
        assert!(
            matches!(result, Err(compute_k8s_lifecycle_repository_postgres::PgK8sLifecycleMigrationError::Sqlx(ref error)) if error.contains("injected pending constraint DDL failure after columns and ledger adoption")),
            "{result:?}"
        );
        assert_eq!(
            snapshot(setup).await,
            before,
            "actual append failure rolls back columns, constraints, ledger rows, and timestamps"
        );
        execute(setup, "DROP EVENT TRIGGER compute_test_pending_ddl_failure; DROP FUNCTION public.compute_test_reject_pending_constraints()").await;
        let retried = PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await
            .expect("retry after removing actual DDL failure");
        assert_eq!(retried.applied_versions, [3]);
        assert_eq!(retried.adopted_unversioned_schema, adoption);
    }
}

pub(super) async fn snapshot(setup: &PgPool) -> Vec<String> {
    sqlx::query_scalar("SELECT 'ledger|' || to_jsonb(m)::text FROM compute_k8s_lifecycle.schema_migrations m UNION ALL SELECT 'column|' || to_jsonb(c)::text FROM information_schema.columns c WHERE table_schema = 'compute_k8s_lifecycle' UNION ALL SELECT 'constraint|' || p.conname || '|' || pg_get_constraintdef(p.oid) FROM pg_constraint p JOIN pg_namespace n ON n.oid = p.connamespace WHERE n.nspname = 'compute_k8s_lifecycle' UNION ALL SELECT 'relation|' || c.relname || '|' || coalesce(c.relacl::text, '') FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'compute_k8s_lifecycle' UNION ALL SELECT 'schema|' || n.nspname || '|' || coalesce(n.nspacl::text, '') FROM pg_namespace n WHERE n.nspname = 'compute_k8s_lifecycle' ORDER BY 1")
        .fetch_all(setup).await.expect("snapshot immutable prefix native shape and ledger")
}
