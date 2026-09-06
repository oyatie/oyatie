use compute_k8s_api::*;
use compute_k8s_lifecycle_repository_postgres::{
    K8S_LIFECYCLE_MIGRATIONS, K8S_LIFECYCLE_REPOSITORY_MIGRATION,
    K8S_LIFECYCLE_RUNTIME_ROLE_MIGRATION,
};
use sha2::{Digest, Sha256};
use shared_postgres_command_kernel::split_migration_statements;
use sqlx::PgPool;

use crate::support::{create_command, reset_runtime_role};

const LEDGER: &str = "CREATE TABLE compute_k8s_lifecycle.schema_migrations (
    version bigint NOT NULL, name text NOT NULL, sha256 text NOT NULL,
    applied_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT schema_migrations_primary_key PRIMARY KEY (version),
    CONSTRAINT schema_migrations_name_unique UNIQUE (name),
    CONSTRAINT schema_migrations_version_positive CHECK (version > 0),
    CONSTRAINT schema_migrations_name_not_empty CHECK (name <> ''),
    CONSTRAINT schema_migrations_sha256_shape CHECK (sha256 ~ '^[0-9a-f]{64}$'))";

pub(super) async fn execute(setup: &PgPool, sql: &str) {
    for statement in split_migration_statements(sql) {
        sqlx::query(&statement)
            .execute(setup)
            .await
            .expect("apply original native fixture");
    }
}

pub(super) async fn prefix(setup: &PgPool, length: usize) {
    reset_runtime_role(setup).await;
    execute(setup, K8S_LIFECYCLE_RUNTIME_ROLE_MIGRATION).await;
    execute(setup, LEDGER).await;
    execute(
        setup,
        "GRANT SELECT ON compute_k8s_lifecycle.schema_migrations TO compute_k8s_lifecycle_runtime",
    )
    .await;
    if length == 2 {
        execute(setup, K8S_LIFECYCLE_REPOSITORY_MIGRATION).await;
    }
    for migration in &K8S_LIFECYCLE_MIGRATIONS[..length] {
        sqlx::query("INSERT INTO compute_k8s_lifecycle.schema_migrations (version, name, sha256) VALUES ($1, $2, $3)")
            .bind(migration.version()).bind(migration.name()).bind(migration.sha256())
            .execute(setup).await.expect("record actual original prefix");
    }
}

pub(super) fn request() -> CloudComputeK8sClusterCreateApiRequest {
    let command = create_command("ten_upgrade", "legacy", "legacy-create");
    CloudComputeK8sClusterCreateApiRequest {
        boundary: CloudComputeK8sApiBoundaryContext {
            request_id: command.request_id,
            tenant_id: command.operation_key.tenant_id.clone(),
            idempotency_key: command.operation_key.idempotency_key,
        },
        principal: CloudComputeK8sApiPrincipal {
            tenant_id: command.operation_key.tenant_id.clone(),
            principal_id: command.operation_key.principal_id.clone(),
        },
        authorization: CloudComputeK8sApiAuthorization {
            tenant_id: command.operation_key.tenant_id,
            principal_id: command.operation_key.principal_id,
            decision_id: "legacy-authorization".to_owned(),
            allowed_surfaces: vec![],
            proof: None,
        },
        path_cluster_id: command.cluster.resource_id,
        body: command.desired_spec,
    }
}

pub(super) fn verifier() -> CloudComputeK8sTrustedAuthorizationVerifier {
    let request = request();
    CloudComputeK8sTrustedAuthorizationVerifier::new(10).with_authorization_proof(
        CloudComputeK8sApiAuthorizationProof {
            tenant_id: request.principal.tenant_id,
            principal_id: request.principal.principal_id,
            surface: CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.to_owned(),
            decision_id: request.authorization.decision_id,
            verified: true,
            issued_at_epoch_seconds: 1,
            expires_at_epoch_seconds: 20,
        },
    )
}

pub(super) struct OriginalSqlRepository(pub PgPool);

impl CloudComputeK8sLifecycleRepository for OriginalSqlRepository {
    fn commit_create<'a>(
        &'a self,
        command: CloudComputeK8sCreateCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sCreateReceipt, CloudComputeK8sLifecycleRepositoryError>,
    > {
        Box::pin(async move {
            let receipt = CloudComputeK8sCreateReceipt {
                cluster: command.cluster.clone(),
                request_id: command.request_id,
            };
            let json = serde_json::to_value(&receipt).expect("serialize original receipt");
            let digest = format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&json).expect("canonical sorted object JSON"))
            );
            let mut tx = self.0.begin().await.expect("begin original SQL fixture");
            sqlx::query("INSERT INTO compute_k8s_lifecycle.operations (tenant_id, principal_id, surface, idempotency_key, resource_id, request_fingerprint, receipt_kind, receipt_json, receipt_digest, schema_version, completed_at) VALUES ($1, $2, $3, $4, $5, $6, 'create', $7, $8, 1, now())")
                .bind(&command.operation_key.tenant_id).bind(&command.operation_key.principal_id)
                .bind(&command.operation_key.surface).bind(&command.operation_key.idempotency_key)
                .bind(&command.cluster.resource_id).bind(&command.fingerprint).bind(json).bind(digest)
                .execute(&mut *tx).await.expect("persist genuine API canonical fingerprint and original receipt");
            sqlx::query("INSERT INTO compute_k8s_lifecycle.clusters (tenant_id, resource_id, desired_spec_json, cluster_json, observed_state, desired_state, schema_version) VALUES ($1, $2, $3, $4, $5, $6, 1)")
                .bind(&command.operation_key.tenant_id).bind(&command.cluster.resource_id)
                .bind(serde_json::to_value(&command.desired_spec).unwrap()).bind(serde_json::to_value(&command.cluster).unwrap())
                .bind(&command.cluster.state).bind(&command.cluster.desired_state)
                .execute(&mut *tx).await.expect("persist original cluster projection");
            tx.commit().await.expect("commit original fixture");
            Ok(receipt)
        })
    }

    fn commit_deletion<'a>(
        &'a self,
        _command: CloudComputeK8sDeleteCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sDeleteReceipt, CloudComputeK8sLifecycleRepositoryError>,
    > {
        Box::pin(async { panic!("legacy upgrade fixture creates only") })
    }
}

pub(super) async fn data_snapshot(setup: &PgPool) -> (Vec<String>, Vec<String>) {
    let operations = sqlx::query_scalar("SELECT (to_jsonb(o) - 'request_contract' - 'operation_state')::text FROM compute_k8s_lifecycle.operations o ORDER BY tenant_id, principal_id, surface, idempotency_key")
        .fetch_all(setup).await.expect("snapshot all original operation fields");
    let clusters = sqlx::query_scalar("SELECT to_jsonb(c)::text FROM compute_k8s_lifecycle.clusters c ORDER BY tenant_id, resource_id")
        .fetch_all(setup).await.expect("snapshot all original cluster fields");
    (operations, clusters)
}
