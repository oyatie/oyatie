//! PostgreSQL-backed Kubernetes lifecycle intent and idempotency repository.
//!
//! Each operation runs in one transaction, sets the canonical tenant GUC before
//! touching tenant data, and commits the cluster intent with its replay receipt.
//! Construction verifies the serving role and FORCE RLS posture before returning
//! a usable repository.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use compute_k8s_api::{
    CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
    CloudComputeK8sClusterCreateRequest, CloudComputeK8sClusterRecord,
    CloudComputeK8sCreateCommand, CloudComputeK8sCreateReceipt, CloudComputeK8sDeleteCommand,
    CloudComputeK8sDeleteReceipt, CloudComputeK8sLifecycleRepository,
    CloudComputeK8sLifecycleRepositoryError, CloudComputeK8sOperationKey,
    CloudComputeK8sRepositoryFuture,
};
use serde::{Serialize, de::DeserializeOwned};
use shared_postgres_command_adapter_sqlx::assert_rls_enforceable;
use shared_postgres_command_kernel::{RlsEnforceabilityError, SET_LOCAL_TENANT_SQL};
use sqlx::{PgPool, Row, postgres::PgPoolOptions, postgres::PgRow};

pub const SCHEMA_NAME: &str = "compute_k8s_lifecycle";
pub const CLUSTERS_TABLE: &str = "compute_k8s_lifecycle.clusters";
pub const OPERATIONS_TABLE: &str = "compute_k8s_lifecycle.operations";
pub const GOVERNED_TABLES: &[&str] = &[CLUSTERS_TABLE, OPERATIONS_TABLE];
pub const RUNTIME_ROLE: &str = "compute_k8s_lifecycle_runtime";
pub const SCHEMA_VERSION: i32 = 1;

const RESERVE_OPERATION_SQL: &str = "INSERT INTO compute_k8s_lifecycle.operations (tenant_id, principal_id, surface, idempotency_key, resource_id, request_fingerprint, schema_version) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (tenant_id, principal_id, surface, idempotency_key) DO NOTHING RETURNING 1 AS inserted";
const SELECT_OPERATION_FOR_UPDATE_SQL: &str = "SELECT resource_id, request_fingerprint, receipt_kind, receipt_json, schema_version FROM compute_k8s_lifecycle.operations WHERE tenant_id = $1 AND principal_id = $2 AND surface = $3 AND idempotency_key = $4 FOR UPDATE";
const COMPLETE_OPERATION_SQL: &str = "UPDATE compute_k8s_lifecycle.operations SET receipt_kind = $5, receipt_json = $6, completed_at = now() WHERE tenant_id = $1 AND principal_id = $2 AND surface = $3 AND idempotency_key = $4";

const INSERT_CLUSTER_SQL: &str = "INSERT INTO compute_k8s_lifecycle.clusters (tenant_id, resource_id, desired_spec_json, cluster_json, observed_state, desired_state, schema_version) VALUES ($1, $2, $3, $4, $5, $6, $7)";
const SELECT_CLUSTER_FOR_UPDATE_SQL: &str = "SELECT desired_spec_json, cluster_json, observed_state, desired_state, schema_version FROM compute_k8s_lifecycle.clusters WHERE tenant_id = $1 AND resource_id = $2 FOR UPDATE";
const UPDATE_CLUSTER_SQL: &str = "UPDATE compute_k8s_lifecycle.clusters SET cluster_json = $3, observed_state = $4, desired_state = $5, schema_version = $6, updated_at = now() WHERE tenant_id = $1 AND resource_id = $2";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PgK8sLifecycleConnectError {
    MissingDatabaseUrl,
    Sqlx(String),
    RlsUnenforceable { role: String },
    RlsRoleMismatch { role: String, expected: String },
    RlsNotForcedOnTable { table: String },
}

impl core::fmt::Display for PgK8sLifecycleConnectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingDatabaseUrl => write!(f, "database url is empty"),
            Self::Sqlx(detail) => write!(f, "PostgreSQL connection validation failed: {detail}"),
            Self::RlsUnenforceable { role } => {
                write!(f, "runtime role '{role}' can bypass row-level security")
            }
            Self::RlsRoleMismatch { role, expected } => write!(
                f,
                "runtime role '{role}' is not a member of policy role '{expected}'"
            ),
            Self::RlsNotForcedOnTable { table } => {
                write!(f, "governed table '{table}' is not protected by FORCE RLS")
            }
        }
    }
}

impl std::error::Error for PgK8sLifecycleConnectError {}

impl From<RlsEnforceabilityError> for PgK8sLifecycleConnectError {
    fn from(error: RlsEnforceabilityError) -> Self {
        match error {
            RlsEnforceabilityError::Unenforceable { role } => Self::RlsUnenforceable { role },
            RlsEnforceabilityError::RoleMismatch { role, expected } => {
                Self::RlsRoleMismatch { role, expected }
            }
            RlsEnforceabilityError::RlsNotForced { table, .. }
            | RlsEnforceabilityError::GovernedTableMissing { table } => {
                Self::RlsNotForcedOnTable { table }
            }
            RlsEnforceabilityError::RoleSwitchInEffect { .. }
            | RlsEnforceabilityError::ProbeFailed { .. } => Self::Sqlx(error.to_string()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PgK8sLifecycleRepository {
    pool: PgPool,
}

impl PgK8sLifecycleRepository {
    pub async fn connect(database_url: &str) -> Result<Self, PgK8sLifecycleConnectError> {
        validate_database_url(database_url)?;
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(database_url)
            .await
            .map_err(|error| PgK8sLifecycleConnectError::Sqlx(error.to_string()))?;
        Self::from_pool(pool).await
    }

    pub async fn from_pool(pool: PgPool) -> Result<Self, PgK8sLifecycleConnectError> {
        let repository = Self { pool };
        repository.assert_rls_enforceable().await?;
        Ok(repository)
    }

    pub async fn assert_rls_enforceable(&self) -> Result<(), PgK8sLifecycleConnectError> {
        assert_rls_enforceable(&self.pool, RUNTIME_ROLE, GOVERNED_TABLES)
            .await
            .map_err(PgK8sLifecycleConnectError::from)
    }
}

#[derive(Debug)]
struct StoredOperation {
    resource_id: String,
    request_fingerprint: String,
    receipt_kind: Option<String>,
    receipt_json: Option<serde_json::Value>,
    schema_version: i32,
}

fn validate_database_url(database_url: &str) -> Result<(), PgK8sLifecycleConnectError> {
    if database_url.trim().is_empty() {
        return Err(PgK8sLifecycleConnectError::MissingDatabaseUrl);
    }
    Ok(())
}

fn unavailable<T>(_error: T) -> CloudComputeK8sLifecycleRepositoryError {
    CloudComputeK8sLifecycleRepositoryError::Unavailable
}

fn integrity<T>(_error: T) -> CloudComputeK8sLifecycleRepositoryError {
    CloudComputeK8sLifecycleRepositoryError::IntegrityViolation
}

fn encode<T: Serialize>(
    value: &T,
) -> Result<serde_json::Value, CloudComputeK8sLifecycleRepositoryError> {
    serde_json::to_value(value).map_err(integrity)
}

fn decode<T: DeserializeOwned>(
    value: serde_json::Value,
) -> Result<T, CloudComputeK8sLifecycleRepositoryError> {
    serde_json::from_value(value).map_err(integrity)
}

fn validate_operation_key(
    operation_key: &CloudComputeK8sOperationKey,
    expected_surface: &str,
) -> Result<(), CloudComputeK8sLifecycleRepositoryError> {
    if operation_key.tenant_id.trim().is_empty()
        || operation_key.principal_id.trim().is_empty()
        || operation_key.idempotency_key.trim().is_empty()
        || operation_key.surface != expected_surface
    {
        return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
    }
    Ok(())
}

fn validate_create_command(
    command: &CloudComputeK8sCreateCommand,
) -> Result<(), CloudComputeK8sLifecycleRepositoryError> {
    validate_operation_key(
        &command.operation_key,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
    )?;
    if command.fingerprint.trim().is_empty()
        || command.request_id.trim().is_empty()
        || command.cluster.resource_id != command.desired_spec.resource_id
        || command.cluster.tenant_id != command.desired_spec.tenant_id
        || command.cluster.tenant_id != command.operation_key.tenant_id
        || command.cluster.state.trim().is_empty()
        || command.cluster.desired_state != "present"
        || command.cluster.schema_version == 0
    {
        return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
    }
    Ok(())
}

fn validate_delete_command(
    command: &CloudComputeK8sDeleteCommand,
) -> Result<(), CloudComputeK8sLifecycleRepositoryError> {
    validate_operation_key(
        &command.operation_key,
        CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
    )?;
    let resource_tenant = command.resource_id.tenant_id().map_err(integrity)?;
    if command.request_id.trim().is_empty()
        || command.resource_id.value.trim().is_empty()
        || resource_tenant != command.operation_key.tenant_id
    {
        return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
    }
    Ok(())
}

fn stored_operation(
    row: &PgRow,
) -> Result<StoredOperation, CloudComputeK8sLifecycleRepositoryError> {
    Ok(StoredOperation {
        resource_id: row.try_get("resource_id").map_err(integrity)?,
        request_fingerprint: row.try_get("request_fingerprint").map_err(integrity)?,
        receipt_kind: row.try_get("receipt_kind").map_err(integrity)?,
        receipt_json: row.try_get("receipt_json").map_err(integrity)?,
        schema_version: row.try_get("schema_version").map_err(integrity)?,
    })
}

fn replay_create(
    row: &PgRow,
    command: &CloudComputeK8sCreateCommand,
) -> Result<CloudComputeK8sCreateReceipt, CloudComputeK8sLifecycleRepositoryError> {
    let stored = stored_operation(row)?;
    if stored.resource_id != command.cluster.resource_id
        || stored.request_fingerprint != command.fingerprint
    {
        return Err(
            CloudComputeK8sLifecycleRepositoryError::IdempotencyKeyReused {
                idempotency_key: command.operation_key.idempotency_key.clone(),
            },
        );
    }
    if stored.schema_version != SCHEMA_VERSION || stored.receipt_kind.as_deref() != Some("create") {
        return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
    }
    let receipt: CloudComputeK8sCreateReceipt = decode(
        stored
            .receipt_json
            .ok_or(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation)?,
    )?;
    if receipt.cluster != command.cluster {
        return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
    }
    Ok(receipt)
}

fn replay_delete(
    row: &PgRow,
    command: &CloudComputeK8sDeleteCommand,
) -> Result<CloudComputeK8sDeleteReceipt, CloudComputeK8sLifecycleRepositoryError> {
    let stored = stored_operation(row)?;
    if stored.resource_id != command.resource_id.value
        || stored.request_fingerprint != command.resource_id.value
    {
        return Err(
            CloudComputeK8sLifecycleRepositoryError::IdempotencyKeyReused {
                idempotency_key: command.operation_key.idempotency_key.clone(),
            },
        );
    }
    if stored.schema_version != SCHEMA_VERSION || stored.receipt_kind.as_deref() != Some("delete") {
        return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
    }
    let receipt: CloudComputeK8sDeleteReceipt = decode(
        stored
            .receipt_json
            .ok_or(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation)?,
    )?;
    if receipt.cluster.resource_id != command.resource_id.value
        || receipt.cluster.tenant_id != command.operation_key.tenant_id
        || receipt.cluster.desired_state != "deleted"
    {
        return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
    }
    Ok(receipt)
}

async fn reserve_operation(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    operation_key: &CloudComputeK8sOperationKey,
    resource_id: &str,
    request_fingerprint: &str,
) -> Result<bool, CloudComputeK8sLifecycleRepositoryError> {
    let inserted = sqlx::query(RESERVE_OPERATION_SQL)
        .bind(&operation_key.tenant_id)
        .bind(&operation_key.principal_id)
        .bind(&operation_key.surface)
        .bind(&operation_key.idempotency_key)
        .bind(resource_id)
        .bind(request_fingerprint)
        .bind(SCHEMA_VERSION)
        .fetch_optional(&mut **tx)
        .await
        .map_err(unavailable)?;
    Ok(inserted.is_some())
}

async fn select_operation_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    operation_key: &CloudComputeK8sOperationKey,
) -> Result<PgRow, CloudComputeK8sLifecycleRepositoryError> {
    sqlx::query(SELECT_OPERATION_FOR_UPDATE_SQL)
        .bind(&operation_key.tenant_id)
        .bind(&operation_key.principal_id)
        .bind(&operation_key.surface)
        .bind(&operation_key.idempotency_key)
        .fetch_optional(&mut **tx)
        .await
        .map_err(unavailable)?
        .ok_or(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation)
}

async fn complete_operation(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    operation_key: &CloudComputeK8sOperationKey,
    receipt_kind: &str,
    receipt_json: &serde_json::Value,
) -> Result<(), CloudComputeK8sLifecycleRepositoryError> {
    let result = sqlx::query(COMPLETE_OPERATION_SQL)
        .bind(&operation_key.tenant_id)
        .bind(&operation_key.principal_id)
        .bind(&operation_key.surface)
        .bind(&operation_key.idempotency_key)
        .bind(receipt_kind)
        .bind(receipt_json)
        .execute(&mut **tx)
        .await
        .map_err(unavailable)?;
    if result.rows_affected() != 1 {
        return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
    }
    Ok(())
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(database) => database.code().as_deref() == Some("23505"),
        _ => false,
    }
}

impl CloudComputeK8sLifecycleRepository for PgK8sLifecycleRepository {
    fn commit_create<'a>(
        &'a self,
        command: CloudComputeK8sCreateCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sCreateReceipt, CloudComputeK8sLifecycleRepositoryError>,
    > {
        Box::pin(async move {
            validate_create_command(&command)?;
            let desired_spec_json = encode(&command.desired_spec)?;
            let cluster_json = encode(&command.cluster)?;
            let mut tx = self.pool.begin().await.map_err(unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&command.operation_key.tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(unavailable)?;

            let inserted = reserve_operation(
                &mut tx,
                &command.operation_key,
                &command.cluster.resource_id,
                &command.fingerprint,
            )
            .await?;
            if !inserted {
                let row = select_operation_for_update(&mut tx, &command.operation_key).await?;
                let receipt = replay_create(&row, &command)?;
                tx.commit().await.map_err(unavailable)?;
                return Ok(receipt);
            }

            let insert = sqlx::query(INSERT_CLUSTER_SQL)
                .bind(&command.operation_key.tenant_id)
                .bind(&command.cluster.resource_id)
                .bind(&desired_spec_json)
                .bind(&cluster_json)
                .bind(&command.cluster.state)
                .bind(&command.cluster.desired_state)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await;
            if let Err(error) = insert {
                if is_unique_violation(&error) {
                    return Err(CloudComputeK8sLifecycleRepositoryError::ClusterAlreadyExists);
                }
                return Err(CloudComputeK8sLifecycleRepositoryError::Unavailable);
            }

            let receipt = CloudComputeK8sCreateReceipt {
                cluster: command.cluster,
                request_id: command.request_id,
            };
            let receipt_json = encode(&receipt)?;
            complete_operation(&mut tx, &command.operation_key, "create", &receipt_json).await?;
            tx.commit().await.map_err(unavailable)?;
            Ok(receipt)
        })
    }

    fn commit_deletion<'a>(
        &'a self,
        command: CloudComputeK8sDeleteCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sDeleteReceipt, CloudComputeK8sLifecycleRepositoryError>,
    > {
        Box::pin(async move {
            validate_delete_command(&command)?;
            let mut tx = self.pool.begin().await.map_err(unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&command.operation_key.tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(unavailable)?;

            let inserted = reserve_operation(
                &mut tx,
                &command.operation_key,
                &command.resource_id.value,
                &command.resource_id.value,
            )
            .await?;
            if !inserted {
                let row = select_operation_for_update(&mut tx, &command.operation_key).await?;
                let receipt = replay_delete(&row, &command)?;
                tx.commit().await.map_err(unavailable)?;
                return Ok(receipt);
            }

            let row = sqlx::query(SELECT_CLUSTER_FOR_UPDATE_SQL)
                .bind(&command.operation_key.tenant_id)
                .bind(&command.resource_id.value)
                .fetch_optional(&mut *tx)
                .await
                .map_err(unavailable)?
                .ok_or(CloudComputeK8sLifecycleRepositoryError::ClusterNotFound)?;
            let desired_spec_json: serde_json::Value =
                row.try_get("desired_spec_json").map_err(integrity)?;
            let cluster_json: serde_json::Value = row.try_get("cluster_json").map_err(integrity)?;
            let observed_state: String = row.try_get("observed_state").map_err(integrity)?;
            let desired_state: String = row.try_get("desired_state").map_err(integrity)?;
            let schema_version: i32 = row.try_get("schema_version").map_err(integrity)?;
            let desired_spec: CloudComputeK8sClusterCreateRequest = decode(desired_spec_json)?;
            let mut cluster: CloudComputeK8sClusterRecord = decode(cluster_json)?;
            if schema_version != SCHEMA_VERSION
                || desired_spec.resource_id != command.resource_id.value
                || desired_spec.tenant_id != command.operation_key.tenant_id
                || cluster.resource_id != command.resource_id.value
                || cluster.tenant_id != command.operation_key.tenant_id
                || cluster.state != observed_state
                || cluster.desired_state != desired_state
            {
                return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
            }

            cluster.desired_state = "deleted".to_string();
            let updated_cluster_json = encode(&cluster)?;
            let update = sqlx::query(UPDATE_CLUSTER_SQL)
                .bind(&command.operation_key.tenant_id)
                .bind(&command.resource_id.value)
                .bind(&updated_cluster_json)
                .bind(&cluster.state)
                .bind(&cluster.desired_state)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(unavailable)?;
            if update.rows_affected() != 1 {
                return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
            }

            let receipt = CloudComputeK8sDeleteReceipt {
                cluster,
                request_id: command.request_id,
            };
            let receipt_json = encode(&receipt)?;
            complete_operation(&mut tx, &command.operation_key, "delete", &receipt_json).await?;
            tx.commit().await.map_err(unavailable)?;
            Ok(receipt)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_postgres_command_kernel::force_rls_tables;

    #[test]
    fn connect_rejects_blank_database_url() {
        assert_eq!(
            validate_database_url("   "),
            Err(PgK8sLifecycleConnectError::MissingDatabaseUrl)
        );
    }

    #[test]
    fn operation_reservation_serializes_replicas_without_overwriting_first_receipt() {
        assert!(RESERVE_OPERATION_SQL.contains("ON CONFLICT"));
        assert!(RESERVE_OPERATION_SQL.contains("DO NOTHING"));
        assert!(SELECT_OPERATION_FOR_UPDATE_SQL.contains("FOR UPDATE"));
        assert!(SELECT_CLUSTER_FOR_UPDATE_SQL.contains("FOR UPDATE"));
        assert!(!COMPLETE_OPERATION_SQL.contains("INSERT"));
    }

    #[test]
    fn every_data_statement_is_explicitly_tenant_scoped() {
        for statement in [
            RESERVE_OPERATION_SQL,
            SELECT_OPERATION_FOR_UPDATE_SQL,
            COMPLETE_OPERATION_SQL,
            INSERT_CLUSTER_SQL,
            SELECT_CLUSTER_FOR_UPDATE_SQL,
            UPDATE_CLUSTER_SQL,
        ] {
            assert!(statement.contains("tenant_id"), "{statement}");
        }
        assert_eq!(
            SET_LOCAL_TENANT_SQL,
            "SELECT set_config('oyatie.tenant_id', $1, true)"
        );
    }

    #[test]
    fn governed_tables_exactly_match_force_rls_migration() {
        let migration = include_str!("../migrations/0001_k8s_lifecycle_repository.sql");
        let mut forced = force_rls_tables(migration);
        forced.sort();
        let mut governed: Vec<String> = GOVERNED_TABLES
            .iter()
            .map(|table| (*table).to_string())
            .collect();
        governed.sort();
        assert_eq!(governed, forced);
    }

    #[test]
    fn runtime_role_is_shared_by_role_and_policy_migrations() {
        let role_migration = include_str!("../migrations/0000_runtime_role.sql");
        let table_migration = include_str!("../migrations/0001_k8s_lifecycle_repository.sql");
        assert!(role_migration.contains(RUNTIME_ROLE));
        assert!(table_migration.contains(&format!("TO {RUNTIME_ROLE}")));
    }
}
