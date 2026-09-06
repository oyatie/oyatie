use compute_k8s_api::{
    CloudComputeK8sLifecycleRepository, CloudComputeK8sLifecycleRepositoryError,
    create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier,
};
use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleMigrator, PgK8sLifecycleRepository, PgK8sLifecycleRuntimeContract,
};
use sqlx::{PgPool, Row};

use super::{legacy_fixture, pending_refusal};
use crate::support::reset_runtime_role;
use crate::support::{create_command, grant_runtime_role};

pub(super) async fn assert_pending_intent_schema(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    reset_runtime_role(setup).await;
    let report = PgK8sLifecycleMigrator::from_pool(setup.clone())
        .migrate()
        .await
        .expect("fresh pending-intent migration");
    assert_eq!(report.applied_versions, [1, 2, 3]);
    let columns = sqlx::query("SELECT column_name, is_nullable, column_default FROM information_schema.columns WHERE table_schema = 'compute_k8s_lifecycle' AND table_name = 'operations' AND column_name IN ('request_contract', 'operation_state') ORDER BY column_name")
        .fetch_all(setup).await.expect("inspect pending columns");
    assert_eq!(columns.len(), 2);
    assert_eq!(
        columns[0].get::<String, _>("column_name"),
        "operation_state"
    );
    assert_eq!(columns[0].get::<String, _>("is_nullable"), "YES");
    assert_eq!(columns[0].get::<Option<String>, _>("column_default"), None);
    assert_eq!(columns[1].get::<String, _>("is_nullable"), "NO");
    assert_eq!(
        columns[1].get::<String, _>("column_default"),
        "'trusted_envelope'::text"
    );
    for (contract, state, surface, accepted) in [
        (Some("trusted_envelope"), None, "create", true),
        (Some("trusted_envelope"), None, "delete", true),
        (Some("trusted_envelope"), Some("accepted"), "delete", false),
        (Some("trusted_envelope"), Some("other"), "create", false),
        (Some("trusted_envelope"), Some("other"), "delete", false),
        (Some("pending_intent"), Some("accepted"), "create", true),
        (Some("pending_intent"), None, "create", false),
        (Some("pending_intent"), None, "delete", false),
        (Some("pending_intent"), Some("accepted"), "delete", false),
        (Some("pending_intent"), Some("other"), "create", false),
        (Some("pending_intent"), Some("other"), "delete", false),
        (Some("trusted_envelope"), Some("accepted"), "create", false),
        (Some("unknown"), None, "create", false),
        (Some("unknown"), None, "delete", false),
        (Some("unknown"), Some("accepted"), "create", false),
        (Some("unknown"), Some("accepted"), "delete", false),
        (Some("unknown"), Some("other"), "create", false),
        (Some("unknown"), Some("other"), "delete", false),
        (None, None, "create", false),
        (None, None, "delete", false),
        (None, Some("accepted"), "create", false),
        (None, Some("accepted"), "delete", false),
        (None, Some("other"), "create", false),
        (None, Some("other"), "delete", false),
    ] {
        let mut tx = setup.begin().await.expect("begin finite state probe");
        let result = sqlx::query("INSERT INTO compute_k8s_lifecycle.operations (tenant_id, principal_id, surface, idempotency_key, resource_id, request_fingerprint, schema_version, request_contract, operation_state) VALUES ('tenant', 'principal', $1, 'key', 'resource', 'fingerprint', 1, $2, $3)")
            .bind(format!("cloud.compute.k8s.cluster.{surface}"))
            .bind(contract).bind(state).execute(&mut *tx).await;
        assert_eq!(
            result.is_ok(),
            accepted,
            "{contract:?}/{state:?}/{surface}: {result:?}"
        );
        if let Err(sqlx::Error::Database(error)) = result {
            assert!(matches!(error.code().as_deref(), Some("23514" | "23502")));
        }
        tx.rollback().await.expect("rollback finite state probe");
    }
    assert_upgrades(setup, app, app_role, runtime_contract).await;
    pending_refusal::assert_refusals(setup, app, app_role, runtime_contract).await;
    assert_legacy_discriminator(setup, app, app_role, runtime_contract).await;
}

async fn assert_upgrades(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    contract: &PgK8sLifecycleRuntimeContract,
) {
    for length in [0, 1, 2] {
        legacy_fixture::prefix(setup, length).await;
        grant_runtime_role(setup, app_role).await;
        assert!(
            PgK8sLifecycleRepository::from_pool(app.clone(), contract)
                .await
                .is_err(),
            "writer startup refuses original prefix {length}"
        );
        let result = PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await
            .expect("attested original prefix upgrades");
        assert_eq!(
            result.applied_versions,
            match length {
                0 => vec![1, 2, 3],
                1 => vec![2, 3],
                _ => vec![3],
            }
        );
        assert!(!result.adopted_unversioned_schema);
        assert!(
            PgK8sLifecycleMigrator::from_pool(setup.clone())
                .migrate()
                .await
                .unwrap()
                .applied_versions
                .is_empty()
        );
    }
    for ledger in ["versioned", "absent", "empty"] {
        legacy_fixture::prefix(setup, 2).await;
        if ledger == "absent" {
            legacy_fixture::execute(setup, "DROP TABLE compute_k8s_lifecycle.schema_migrations")
                .await;
        } else if ledger == "empty" {
            legacy_fixture::execute(setup, "DELETE FROM compute_k8s_lifecycle.schema_migrations")
                .await;
        }
        let request = legacy_fixture::request();
        let verifier = legacy_fixture::verifier();
        let first = create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
            &legacy_fixture::OriginalSqlRepository(setup.clone()),
            request.clone(),
            &verifier,
        )
        .await
        .expect("actual legacy API creates original SQL fixture");
        let before = legacy_fixture::data_snapshot(setup).await;
        let report = PgK8sLifecycleMigrator::from_pool(setup.clone())
            .migrate()
            .await
            .expect("populated original schema upgrades");
        assert_eq!(report.adopted_unversioned_schema, ledger != "versioned");
        assert_eq!(report.applied_versions, [3]);
        assert_eq!(legacy_fixture::data_snapshot(setup).await, before);
        grant_runtime_role(setup, app_role).await;
        let repository = PgK8sLifecycleRepository::from_pool(app.clone(), contract)
            .await
            .unwrap();
        let replay = create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
            &repository,
            request,
            &verifier,
        )
        .await
        .expect("actual legacy API replays preserved canonical receipt");
        assert_eq!(replay, first);
        assert_eq!(legacy_fixture::data_snapshot(setup).await, before);
    }
}

async fn assert_legacy_discriminator(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    contract: &PgK8sLifecycleRuntimeContract,
) {
    crate::support::setup_schema(setup, app_role).await;
    let repository = PgK8sLifecycleRepository::from_pool(app.clone(), contract)
        .await
        .unwrap();
    let command = create_command("ten_contract", "bridge", "bridge");
    repository.commit_create(command.clone()).await.unwrap();
    for (request_contract, state, reused) in [
        ("pending_intent", Some("accepted"), true),
        ("pending_intent", None, false),
        ("pending_intent", Some("unknown"), false),
        ("trusted_envelope", Some("accepted"), false),
        ("unknown", None, false),
    ] {
        let mut tx = setup.begin().await.unwrap();
        sqlx::query("ALTER TABLE compute_k8s_lifecycle.operations DROP CONSTRAINT operations_contract_state, DROP CONSTRAINT operations_request_contract").execute(&mut *tx).await.unwrap();
        sqlx::query("UPDATE compute_k8s_lifecycle.operations SET request_contract = $1, operation_state = $2").bind(request_contract).bind(state).execute(&mut *tx).await.unwrap();
        tx.commit().await.unwrap();
        let result = repository.commit_create(command.clone()).await;
        if reused {
            assert!(matches!(
                result,
                Err(CloudComputeK8sLifecycleRepositoryError::IdempotencyKeyReused { .. })
            ));
        } else {
            assert_eq!(
                result,
                Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation)
            );
        }
        legacy_fixture::execute(setup, "UPDATE compute_k8s_lifecycle.operations SET request_contract = 'trusted_envelope', operation_state = NULL; ALTER TABLE compute_k8s_lifecycle.operations ADD CONSTRAINT operations_contract_state CHECK (true), ADD CONSTRAINT operations_request_contract CHECK (true)").await;
    }
}
