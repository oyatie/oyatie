use compute_k8s_api::{
    CloudComputeK8sAcceptCreateIntentCommand, CloudComputeK8sAcceptanceRepositoryError as Error,
    CloudComputeK8sAcceptedCreateIntent, CloudComputeK8sOperationKey,
};
use shared_postgres_command_kernel::SET_LOCAL_TENANT_SQL;
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};

const RESERVE: &str = "INSERT INTO compute_k8s_lifecycle.operations (tenant_id, principal_id, surface, idempotency_key, resource_id, request_fingerprint, schema_version, request_contract, operation_state) VALUES ($1, $2, $3, $4, $5, $6, 1, 'pending_intent', 'accepted') ON CONFLICT (tenant_id, principal_id, surface, idempotency_key) DO NOTHING RETURNING extract(epoch FROM created_at)::bigint AS accepted_at_epoch_seconds";
const SELECT: &str = "SELECT resource_id, request_fingerprint, receipt_kind, receipt_json, receipt_digest, schema_version, request_contract, operation_state, extract(epoch FROM created_at)::bigint AS accepted_at_epoch_seconds, completed_at IS NOT NULL AS receipt_complete FROM compute_k8s_lifecycle.operations WHERE tenant_id = $1 AND principal_id = $2 AND surface = $3 AND idempotency_key = $4";

pub(crate) fn unavailable(_: sqlx::Error) -> Error {
    Error::Unavailable
}

pub(crate) async fn begin<'a>(
    pool: &'a PgPool,
    tenant_id: &str,
    read_only: bool,
) -> Result<Transaction<'a, Postgres>, Error> {
    let mut tx = pool.begin().await.map_err(unavailable)?;
    sqlx::query(if read_only {
        "SET TRANSACTION ISOLATION LEVEL READ COMMITTED, READ ONLY"
    } else {
        "SET TRANSACTION ISOLATION LEVEL READ COMMITTED, READ WRITE"
    })
    .execute(&mut *tx)
    .await
    .map_err(unavailable)?;
    crate::catalog_connection::use_catalog_path(&mut tx)
        .await
        .map_err(unavailable)?;
    for sql in [
        "SET LOCAL lock_timeout = '5s'",
        "SET LOCAL statement_timeout = '10s'",
    ] {
        sqlx::query(sql)
            .execute(&mut *tx)
            .await
            .map_err(unavailable)?;
    }
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await
        .map_err(unavailable)?;
    Ok(tx)
}

pub(crate) async fn reserve(
    tx: &mut Transaction<'_, Postgres>,
    command: &CloudComputeK8sAcceptCreateIntentCommand,
    fingerprint: &str,
) -> Result<Option<u64>, Error> {
    let key = &command.operation_key;
    let inserted = sqlx::query(RESERVE)
        .bind(&key.tenant_id)
        .bind(&key.principal_id)
        .bind(&key.surface)
        .bind(&key.idempotency_key)
        .bind(&command.intent.resource_id)
        .bind(fingerprint)
        .fetch_optional(&mut **tx)
        .await
        .map_err(unavailable)?;
    inserted
        .map(|row| {
            let value: i64 = row
                .try_get("accepted_at_epoch_seconds")
                .map_err(|_| Error::IntegrityViolation)?;
            u64::try_from(value).map_err(|_| Error::IntegrityViolation)
        })
        .transpose()
}

pub(crate) async fn select(
    tx: &mut Transaction<'_, Postgres>,
    key: &CloudComputeK8sOperationKey,
    for_update: bool,
) -> Result<Option<PgRow>, Error> {
    let sql = if for_update {
        format!("{SELECT} FOR UPDATE")
    } else {
        SELECT.to_string()
    };
    sqlx::query(&sql)
        .bind(&key.tenant_id)
        .bind(&key.principal_id)
        .bind(&key.surface)
        .bind(&key.idempotency_key)
        .fetch_optional(&mut **tx)
        .await
        .map_err(unavailable)
}

pub(crate) async fn complete(
    tx: &mut Transaction<'_, Postgres>,
    receipt: &CloudComputeK8sAcceptedCreateIntent,
) -> Result<(), Error> {
    let json = serde_json::to_value(receipt).map_err(|_| Error::IntegrityViolation)?;
    let digest =
        crate::canonical_json::json_digest(&json).map_err(|_| Error::IntegrityViolation)?;
    let key = &receipt.operation_key;
    let result = sqlx::query(crate::operation::COMPLETE_OPERATION_SQL)
        .bind(&key.tenant_id)
        .bind(&key.principal_id)
        .bind(&key.surface)
        .bind(&key.idempotency_key)
        .bind("create")
        .bind(json)
        .bind(digest)
        .execute(&mut **tx)
        .await
        .map_err(unavailable)?;
    if result.rows_affected() != 1 {
        return Err(Error::IntegrityViolation);
    }
    Ok(())
}
