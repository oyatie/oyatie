use compute_k8s_api::{
    CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
    CloudComputeK8sCreateCommand, CloudComputeK8sCreateReceipt, CloudComputeK8sDeleteCommand,
    CloudComputeK8sDeleteReceipt, CloudComputeK8sLifecycleRepositoryError,
    CloudComputeK8sOperationKey,
};
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{Row, postgres::PgRow};

use crate::{SCHEMA_VERSION, error::integrity, error::unavailable};

pub(crate) const RESERVE_OPERATION_SQL: &str = "INSERT INTO compute_k8s_lifecycle.operations (tenant_id, principal_id, surface, idempotency_key, resource_id, request_fingerprint, schema_version) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (tenant_id, principal_id, surface, idempotency_key) DO NOTHING RETURNING 1 AS inserted";
pub(crate) const SELECT_OPERATION_FOR_UPDATE_SQL: &str = "SELECT resource_id, request_fingerprint, receipt_kind, receipt_json, schema_version FROM compute_k8s_lifecycle.operations WHERE tenant_id = $1 AND principal_id = $2 AND surface = $3 AND idempotency_key = $4 FOR UPDATE";
pub(crate) const COMPLETE_OPERATION_SQL: &str = "UPDATE compute_k8s_lifecycle.operations SET receipt_kind = $5, receipt_json = $6, completed_at = now() WHERE tenant_id = $1 AND principal_id = $2 AND surface = $3 AND idempotency_key = $4";

pub(crate) const INSERT_CLUSTER_SQL: &str = "INSERT INTO compute_k8s_lifecycle.clusters (tenant_id, resource_id, desired_spec_json, cluster_json, observed_state, desired_state, schema_version) VALUES ($1, $2, $3, $4, $5, $6, $7)";
pub(crate) const SELECT_CLUSTER_FOR_UPDATE_SQL: &str = "SELECT desired_spec_json, cluster_json, observed_state, desired_state, schema_version FROM compute_k8s_lifecycle.clusters WHERE tenant_id = $1 AND resource_id = $2 FOR UPDATE";
pub(crate) const UPDATE_CLUSTER_SQL: &str = "UPDATE compute_k8s_lifecycle.clusters SET cluster_json = $3, observed_state = $4, desired_state = $5, schema_version = $6, updated_at = now() WHERE tenant_id = $1 AND resource_id = $2";

#[derive(Debug)]
struct StoredOperation {
    resource_id: String,
    request_fingerprint: String,
    receipt_kind: Option<String>,
    receipt_json: Option<serde_json::Value>,
    schema_version: i32,
}

pub(crate) fn encode<T: Serialize>(
    value: &T,
) -> Result<serde_json::Value, CloudComputeK8sLifecycleRepositoryError> {
    serde_json::to_value(value).map_err(integrity)
}

pub(crate) fn decode<T: DeserializeOwned>(
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

pub(crate) fn validate_create_command(
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

pub(crate) fn validate_delete_command(
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

pub(crate) fn replay_create(
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

pub(crate) fn replay_delete(
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

pub(crate) async fn reserve_operation(
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

pub(crate) async fn select_operation_for_update(
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

pub(crate) async fn complete_operation(
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

pub(crate) fn is_unique_violation(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(database) => database.code().as_deref() == Some("23505"),
        _ => false,
    }
}
