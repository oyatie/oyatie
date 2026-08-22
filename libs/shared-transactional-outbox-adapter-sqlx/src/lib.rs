//! SQLx-backed transactional outbox drain adapter seam.
//!
//! This crate owns SQLx queries for claiming pending outbox rows with
//! `FOR UPDATE SKIP LOCKED` and marking dispatch state transitions. It is a
//! database adapter seam only: it does not publish to a broker, call gRPC, run a
//! background worker, or prove delivery SLOs.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_postgres_command_kernel::SET_LOCAL_TENANT_SQL;
use shared_transactional_outbox_kernel::BackboneOutboxTable;
pub use shared_transactional_outbox_kernel::{
    OutboxClaimBatch, OutboxDispatchState, OutboxMutationReport, PendingOutboxEvent,
};
use sqlx::{PgPool, Row, postgres::PgRow};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SqlxOutboxDrainError {
    MissingTenantScopeRef,
    MissingWorkerRef,
    InvalidClaimLimit {
        requested: u16,
        max_allowed: u16,
    },
    ClaimedEventScopeMismatch {
        expected_tenant_scope_ref: String, // data_class: INTERNAL_ONLY
        actual_tenant_scope_ref: String,   // data_class: INTERNAL_ONLY
    },
    ClaimedEventServiceMismatch {
        expected_service_id: &'static str,
        actual_service_id: String, // data_class: INTERNAL_ONLY
    },
    MissingEventId,
    Sqlx(String),
}

impl From<sqlx::Error> for SqlxOutboxDrainError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error.to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlxOutboxDrainConfig {
    pub worker_ref: String,        // data_class: INTERNAL_ONLY
    pub max_claim_batch_size: u16, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxClaimRequest {
    pub table: BackboneOutboxTable, // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,   // data_class: INTERNAL_ONLY
    pub requested_limit: u16,       // data_class: INTERNAL_ONLY
}

pub struct SqlxTransactionalOutboxDrain {
    pool: PgPool,
    config: SqlxOutboxDrainConfig,
}

impl SqlxOutboxDrainConfig {
    pub fn new(
        worker_ref: impl Into<String>,
        max_claim_batch_size: u16,
    ) -> Result<Self, SqlxOutboxDrainError> {
        let config = Self {
            worker_ref: worker_ref.into(),
            max_claim_batch_size,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), SqlxOutboxDrainError> {
        if self.worker_ref.trim().is_empty() {
            return Err(SqlxOutboxDrainError::MissingWorkerRef);
        }
        if self.max_claim_batch_size == 0 {
            return Err(SqlxOutboxDrainError::InvalidClaimLimit {
                requested: 0,
                max_allowed: 0,
            });
        }
        Ok(())
    }
}

impl OutboxClaimRequest {
    pub fn new(
        table: BackboneOutboxTable,
        tenant_scope_ref: impl Into<String>,
        requested_limit: u16,
        config: &SqlxOutboxDrainConfig,
    ) -> Result<Self, SqlxOutboxDrainError> {
        config.validate()?;
        let request = Self {
            table,
            tenant_scope_ref: tenant_scope_ref.into(),
            requested_limit,
        };
        request.validate(config)?;
        Ok(request)
    }

    pub fn validate(&self, config: &SqlxOutboxDrainConfig) -> Result<(), SqlxOutboxDrainError> {
        if self.tenant_scope_ref.trim().is_empty() {
            return Err(SqlxOutboxDrainError::MissingTenantScopeRef);
        }
        if self.requested_limit == 0 || self.requested_limit > config.max_claim_batch_size {
            return Err(SqlxOutboxDrainError::InvalidClaimLimit {
                requested: self.requested_limit,
                max_allowed: config.max_claim_batch_size,
            });
        }
        Ok(())
    }
}

impl SqlxTransactionalOutboxDrain {
    #[must_use]
    pub fn from_pool(pool: PgPool, config: SqlxOutboxDrainConfig) -> Self {
        Self { pool, config }
    }

    pub async fn claim_pending_batch(
        &self,
        request: OutboxClaimRequest,
    ) -> Result<OutboxClaimBatch, SqlxOutboxDrainError> {
        request.validate(&self.config)?;
        let mut transaction = self.pool.begin().await?;
        set_tenant_scope_for_rls(&mut transaction, &request.tenant_scope_ref).await?;
        let rows = sqlx::query(&claim_pending_sql(request.table))
            .bind(&request.tenant_scope_ref)
            .bind(i64::from(request.requested_limit))
            .fetch_all(&mut *transaction)
            .await?;
        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            events.push(event_from_row(request.table, &row)?);
        }
        validate_claimed_events(&request, &events)?;
        for event in &events {
            sqlx::query(&mark_publishing_sql(request.table))
                .bind(&request.tenant_scope_ref)
                .bind(&event.event_id)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(claim_batch_from_events(
            request.table,
            &self.config.worker_ref,
            &request.tenant_scope_ref,
            events,
        ))
    }

    pub async fn mark_published(
        &self,
        table: BackboneOutboxTable,
        tenant_scope_ref: &str,
        event_id: &str,
    ) -> Result<OutboxMutationReport, SqlxOutboxDrainError> {
        mutate_dispatch_state(
            &self.pool,
            table,
            tenant_scope_ref,
            event_id,
            OutboxDispatchState::Published,
        )
        .await
    }

    pub async fn mark_dead_letter(
        &self,
        table: BackboneOutboxTable,
        tenant_scope_ref: &str,
        event_id: &str,
    ) -> Result<OutboxMutationReport, SqlxOutboxDrainError> {
        mutate_dispatch_state(
            &self.pool,
            table,
            tenant_scope_ref,
            event_id,
            OutboxDispatchState::DeadLetter,
        )
        .await
    }
}

async fn mutate_dispatch_state(
    pool: &PgPool,
    table: BackboneOutboxTable,
    tenant_scope_ref: &str,
    event_id: &str,
    state: OutboxDispatchState,
) -> Result<OutboxMutationReport, SqlxOutboxDrainError> {
    validate_event_key(tenant_scope_ref, event_id)?;
    let sql = match state {
        OutboxDispatchState::Published => mark_published_sql(table),
        OutboxDispatchState::DeadLetter => mark_dead_letter_sql(table),
    };
    let mut transaction = pool.begin().await?;
    set_tenant_scope_for_rls(&mut transaction, tenant_scope_ref).await?;
    let result = sqlx::query(&sql)
        .bind(tenant_scope_ref)
        .bind(event_id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(OutboxMutationReport {
        table,
        table_name: table.table_name(),
        tenant_scope_ref: tenant_scope_ref.to_string(),
        event_id: event_id.to_string(),
        state,
        rows_affected: result.rows_affected(),
    })
}

fn event_from_row(
    table: BackboneOutboxTable,
    row: &PgRow,
) -> Result<PendingOutboxEvent, SqlxOutboxDrainError> {
    Ok(PendingOutboxEvent {
        table,
        service_id: row.try_get("service_id")?,
        tenant_scope_ref: row.try_get("tenant_id")?,
        event_id: row.try_get("event_id")?,
        event_kind: row.try_get("event_kind")?,
        aggregate_id: row.try_get("aggregate_id")?,
        asyncapi_operation_id: row.try_get("asyncapi_operation_id")?,
        asyncapi_channel_address: row.try_get("asyncapi_channel_address")?,
        asyncapi_message_name: row.try_get("asyncapi_message_name")?,
        proto_package: row.try_get("proto_package")?,
        proto_service: row.try_get("proto_service")?,
        proto_rpc: row.try_get("proto_rpc")?,
        schema_version: row.try_get("schema_version")?,
        idempotency_key: row.try_get("idempotency_key")?,
        policy_decision_ref: row.try_get("policy_decision_ref")?,
        audit_correlation_id: row.try_get("audit_correlation_id")?,
        attempt_count: row.try_get("attempt_count")?,
    })
}

pub fn validate_claimed_events(
    request: &OutboxClaimRequest,
    events: &[PendingOutboxEvent],
) -> Result<(), SqlxOutboxDrainError> {
    for event in events {
        if event.tenant_scope_ref != request.tenant_scope_ref {
            return Err(SqlxOutboxDrainError::ClaimedEventScopeMismatch {
                expected_tenant_scope_ref: request.tenant_scope_ref.clone(),
                actual_tenant_scope_ref: event.tenant_scope_ref.clone(),
            });
        }
        if event.service_id != request.table.service_id() {
            return Err(SqlxOutboxDrainError::ClaimedEventServiceMismatch {
                expected_service_id: request.table.service_id(),
                actual_service_id: event.service_id.clone(),
            });
        }
    }
    Ok(())
}

async fn set_tenant_scope_for_rls(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_scope_ref: &str,
) -> Result<(), SqlxOutboxDrainError> {
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind(tenant_scope_ref)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

pub fn claim_batch_from_events(
    table: BackboneOutboxTable,
    worker_ref: &str,
    tenant_scope_ref: &str,
    events: Vec<PendingOutboxEvent>,
) -> OutboxClaimBatch {
    let event_ids = events.iter().map(|event| event.event_id.clone()).collect();
    OutboxClaimBatch {
        table,
        table_name: table.table_name(),
        worker_ref: worker_ref.to_string(),
        tenant_scope_ref: tenant_scope_ref.to_string(),
        claimed_count: events.len(),
        event_ids,
        events,
    }
}

fn validate_event_key(tenant_scope_ref: &str, event_id: &str) -> Result<(), SqlxOutboxDrainError> {
    if tenant_scope_ref.trim().is_empty() {
        return Err(SqlxOutboxDrainError::MissingTenantScopeRef);
    }
    if event_id.trim().is_empty() {
        return Err(SqlxOutboxDrainError::MissingEventId);
    }
    Ok(())
}

#[must_use]
pub fn claim_pending_sql(table: BackboneOutboxTable) -> String {
    format!(
        "SELECT service_id, tenant_id, event_id, event_kind, aggregate_id, asyncapi_operation_id, asyncapi_channel_address, asyncapi_message_name, proto_package, proto_service, proto_rpc, schema_version, idempotency_key, policy_decision_ref, audit_correlation_id, attempt_count \
         FROM {} \
         WHERE tenant_id = $1 AND dispatch_state = 'pending' \
         ORDER BY created_at \
         LIMIT $2 \
         FOR UPDATE SKIP LOCKED",
        table.table_name()
    )
}

#[must_use]
pub fn mark_publishing_sql(table: BackboneOutboxTable) -> String {
    format!(
        "UPDATE {} SET dispatch_state = 'publishing', attempt_count = attempt_count + 1 WHERE tenant_id = $1 AND event_id = $2",
        table.table_name()
    )
}

#[must_use]
pub fn mark_published_sql(table: BackboneOutboxTable) -> String {
    format!(
        "UPDATE {} SET dispatch_state = 'published', published_at = now() WHERE tenant_id = $1 AND event_id = $2",
        table.table_name()
    )
}

#[must_use]
pub fn mark_dead_letter_sql(table: BackboneOutboxTable) -> String {
    format!(
        "UPDATE {} SET dispatch_state = 'dead_letter' WHERE tenant_id = $1 AND event_id = $2",
        table.table_name()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> SqlxOutboxDrainConfig {
        SqlxOutboxDrainConfig::new("outbox-worker:a", 50).unwrap()
    }

    fn request() -> OutboxClaimRequest {
        OutboxClaimRequest::new(
            BackboneOutboxTable::MessengerMessageStream,
            "tenant:t",
            10,
            &config(),
        )
        .unwrap()
    }

    fn event() -> PendingOutboxEvent {
        PendingOutboxEvent {
            table: BackboneOutboxTable::MessengerMessageStream,
            service_id: "messenger-message-stream".into(),
            tenant_scope_ref: "tenant:t".into(),
            event_id: "event:e".into(),
            event_kind: "oya.messenger.message.posted.v1".into(),
            aggregate_id: "message:m".into(),
            asyncapi_operation_id: "emitMessagePosted".into(),
            asyncapi_channel_address: "workflow-events/messenger.message.posted".into(),
            asyncapi_message_name: "MessagePosted".into(),
            proto_package: "oya.messenger.v1".into(),
            proto_service: "MessageStream".into(),
            proto_rpc: "PostMessage".into(),
            schema_version: "1.0.0".into(),
            idempotency_key: Some("idem:i".into()),
            policy_decision_ref: "policy:p".into(),
            audit_correlation_id: "audit:a".into(),
            attempt_count: 0,
        }
    }

    #[test]
    fn claim_request_rejects_empty_or_excessive_limits() {
        assert_eq!(
            SqlxOutboxDrainConfig::new("worker", 0),
            Err(SqlxOutboxDrainError::InvalidClaimLimit {
                requested: 0,
                max_allowed: 0,
            })
        );
        assert_eq!(
            OutboxClaimRequest::new(
                BackboneOutboxTable::MessengerMessageStream,
                "tenant:t",
                51,
                &config(),
            ),
            Err(SqlxOutboxDrainError::InvalidClaimLimit {
                requested: 51,
                max_allowed: 50,
            })
        );
    }

    #[test]
    fn claim_sql_uses_service_table_and_skip_locked_queue_claim() {
        let sql = claim_pending_sql(BackboneOutboxTable::MessengerMessageStream);

        assert!(sql.contains("messenger_message_stream.protocol_outbox_events"));
        assert!(sql.contains("dispatch_state = 'pending'"));
        assert!(sql.contains("ORDER BY created_at"));
        assert!(sql.contains("FOR UPDATE SKIP LOCKED"));
        assert!(sql.contains("LIMIT $2"));
    }

    #[test]
    fn mutation_sql_is_table_scoped_and_parameterized() {
        assert_eq!(
            mark_publishing_sql(BackboneOutboxTable::MailMailboxStore),
            "UPDATE mail_mailbox_store.protocol_outbox_events SET dispatch_state = 'publishing', attempt_count = attempt_count + 1 WHERE tenant_id = $1 AND event_id = $2"
        );
        assert_eq!(
            mark_published_sql(BackboneOutboxTable::SocialPostComposition),
            "UPDATE social_post_composition.protocol_outbox_events SET dispatch_state = 'published', published_at = now() WHERE tenant_id = $1 AND event_id = $2"
        );
        assert_eq!(
            mark_dead_letter_sql(BackboneOutboxTable::CommunityPostStore),
            "UPDATE community_post_store.protocol_outbox_events SET dispatch_state = 'dead_letter' WHERE tenant_id = $1 AND event_id = $2"
        );
    }

    #[test]
    fn claimed_events_reject_scope_and_service_drift() {
        let mut wrong_scope = event();
        wrong_scope.tenant_scope_ref = "tenant:other".into();
        assert_eq!(
            validate_claimed_events(&request(), &[wrong_scope]),
            Err(SqlxOutboxDrainError::ClaimedEventScopeMismatch {
                expected_tenant_scope_ref: "tenant:t".into(),
                actual_tenant_scope_ref: "tenant:other".into(),
            })
        );

        let mut wrong_service = event();
        wrong_service.service_id = "mail-mailbox-store".into();
        assert_eq!(
            validate_claimed_events(&request(), &[wrong_service]),
            Err(SqlxOutboxDrainError::ClaimedEventServiceMismatch {
                expected_service_id: "messenger-message-stream",
                actual_service_id: "mail-mailbox-store".into(),
            })
        );
    }

    #[test]
    fn claim_batch_summary_preserves_worker_and_event_ids() {
        let batch = claim_batch_from_events(
            BackboneOutboxTable::MessengerMessageStream,
            "worker:a",
            "tenant:t",
            vec![event()],
        );

        assert_eq!(batch.worker_ref, "worker:a");
        assert_eq!(batch.claimed_count, 1);
        assert_eq!(batch.event_ids, vec!["event:e"]);
        assert_eq!(
            batch.table_name,
            "messenger_message_stream.protocol_outbox_events"
        );
    }

    #[test]
    fn event_key_validation_rejects_blank_mutations() {
        assert_eq!(
            validate_event_key("", "event:e"),
            Err(SqlxOutboxDrainError::MissingTenantScopeRef)
        );
        assert_eq!(
            validate_event_key("tenant:t", " "),
            Err(SqlxOutboxDrainError::MissingEventId)
        );
    }
}
