//! Transactional outbox command seam for backbone microservice write plans.
//!
//! The kernel appends a parameterized outbox insert to an existing tenant-scoped
//! SQL write batch so app plans can persist protocol dispatch metadata in the
//! same transaction as business rows. It performs no broker I/O, no payload
//! serialization, and no background polling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_postgres_command_kernel::{
    SqlCommand, SqlCommandError, SqlParam, SqlWriteBatch, TenantSqlContext,
};
use shared_protocol_parity_kernel::{ProtocolEventEnvelope, ProtocolParityError};

pub const INSERT_MESSENGER_OUTBOX_SQL: &str = r#"
INSERT INTO messenger_message_stream.protocol_outbox_events (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  service_id,
  event_id,
  event_kind,
  aggregate_id,
  asyncapi_operation_id,
  asyncapi_channel_address,
  asyncapi_message_name,
  proto_package,
  proto_service,
  proto_rpc,
  schema_version,
  idempotency_key,
  policy_decision_ref,
  audit_correlation_id
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
)
ON CONFLICT (tenant_id, event_id) DO NOTHING
"#;

pub const INSERT_MAIL_OUTBOX_SQL: &str = r#"
INSERT INTO mail_mailbox_store.protocol_outbox_events (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  service_id,
  event_id,
  event_kind,
  aggregate_id,
  asyncapi_operation_id,
  asyncapi_channel_address,
  asyncapi_message_name,
  proto_package,
  proto_service,
  proto_rpc,
  schema_version,
  idempotency_key,
  policy_decision_ref,
  audit_correlation_id
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
)
ON CONFLICT (tenant_id, event_id) DO NOTHING
"#;

pub const INSERT_SOCIAL_OUTBOX_SQL: &str = r#"
INSERT INTO social_post_composition.protocol_outbox_events (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  service_id,
  event_id,
  event_kind,
  aggregate_id,
  asyncapi_operation_id,
  asyncapi_channel_address,
  asyncapi_message_name,
  proto_package,
  proto_service,
  proto_rpc,
  schema_version,
  idempotency_key,
  policy_decision_ref,
  audit_correlation_id
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
)
ON CONFLICT (tenant_id, event_id) DO NOTHING
"#;

pub const INSERT_COMMUNITY_OUTBOX_SQL: &str = r#"
INSERT INTO community_post_store.protocol_outbox_events (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  service_id,
  event_id,
  event_kind,
  aggregate_id,
  asyncapi_operation_id,
  asyncapi_channel_address,
  asyncapi_message_name,
  proto_package,
  proto_service,
  proto_rpc,
  schema_version,
  idempotency_key,
  policy_decision_ref,
  audit_correlation_id
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
)
ON CONFLICT (tenant_id, event_id) DO NOTHING
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackboneOutboxTable {
    MessengerMessageStream,
    MailMailboxStore,
    SocialPostComposition,
    CommunityPostStore,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionalOutboxError {
    Protocol(ProtocolParityError),
    Sql(SqlCommandError),
    TenantScopeMismatch {
        tenant_id: String,          // data_class: INTERNAL_ONLY
        envelope_scope_ref: String, // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionalOutboxEventSummary {
    pub service_id: &'static str,        // data_class: INTERNAL_ONLY
    pub table_name: &'static str,        // data_class: INTERNAL_ONLY
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub event_kind: &'static str,        // data_class: INTERNAL_ONLY
    pub aggregate_id: String,            // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,    // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,     // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingOutboxEvent {
    pub table: BackboneOutboxTable,       // data_class: INTERNAL_ONLY
    pub service_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,         // data_class: INTERNAL_ONLY
    pub event_id: String,                 // data_class: INTERNAL_ONLY
    pub event_kind: String,               // data_class: INTERNAL_ONLY
    pub aggregate_id: String,             // data_class: INTERNAL_ONLY
    pub asyncapi_operation_id: String,    // data_class: INTERNAL_ONLY
    pub asyncapi_channel_address: String, // data_class: INTERNAL_ONLY
    pub asyncapi_message_name: String,    // data_class: INTERNAL_ONLY
    pub proto_package: String,            // data_class: INTERNAL_ONLY
    pub proto_service: String,            // data_class: INTERNAL_ONLY
    pub proto_rpc: String,                // data_class: INTERNAL_ONLY
    pub schema_version: String,           // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>,  // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,      // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,     // data_class: INTERNAL_ONLY
    pub attempt_count: i32,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxClaimBatch {
    pub table: BackboneOutboxTable,      // data_class: INTERNAL_ONLY
    pub table_name: &'static str,        // data_class: INTERNAL_ONLY
    pub worker_ref: String,              // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,        // data_class: INTERNAL_ONLY
    pub claimed_count: usize,            // data_class: INTERNAL_ONLY
    pub event_ids: Vec<String>,          // data_class: INTERNAL_ONLY
    pub events: Vec<PendingOutboxEvent>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutboxDispatchState {
    Published,
    DeadLetter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxMutationReport {
    pub table: BackboneOutboxTable, // data_class: INTERNAL_ONLY
    pub table_name: &'static str,   // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,   // data_class: INTERNAL_ONLY
    pub event_id: String,           // data_class: INTERNAL_ONLY
    pub state: OutboxDispatchState, // data_class: INTERNAL_ONLY
    pub rows_affected: u64,         // data_class: INTERNAL_ONLY
}

impl From<ProtocolParityError> for TransactionalOutboxError {
    fn from(error: ProtocolParityError) -> Self {
        Self::Protocol(error)
    }
}

impl From<SqlCommandError> for TransactionalOutboxError {
    fn from(error: SqlCommandError) -> Self {
        Self::Sql(error)
    }
}

impl BackboneOutboxTable {
    #[must_use]
    pub const fn service_id(self) -> &'static str {
        match self {
            Self::MessengerMessageStream => "messenger-message-stream",
            Self::MailMailboxStore => "mail-mailbox-store",
            Self::SocialPostComposition => "social-post-composition",
            Self::CommunityPostStore => "community-post-store",
        }
    }

    #[must_use]
    pub const fn table_name(self) -> &'static str {
        match self {
            Self::MessengerMessageStream => "messenger_message_stream.protocol_outbox_events",
            Self::MailMailboxStore => "mail_mailbox_store.protocol_outbox_events",
            Self::SocialPostComposition => "social_post_composition.protocol_outbox_events",
            Self::CommunityPostStore => "community_post_store.protocol_outbox_events",
        }
    }

    #[must_use]
    pub const fn insert_sql(self) -> &'static str {
        match self {
            Self::MessengerMessageStream => INSERT_MESSENGER_OUTBOX_SQL,
            Self::MailMailboxStore => INSERT_MAIL_OUTBOX_SQL,
            Self::SocialPostComposition => INSERT_SOCIAL_OUTBOX_SQL,
            Self::CommunityPostStore => INSERT_COMMUNITY_OUTBOX_SQL,
        }
    }
}

pub fn append_outbox_to_batch(
    table: BackboneOutboxTable,
    tenant: &TenantSqlContext,
    batch: SqlWriteBatch,
    envelope: &ProtocolEventEnvelope,
) -> Result<SqlWriteBatch, TransactionalOutboxError> {
    let expected_scope = tenant.tenant_scope_command()?;
    if batch.tenant_scope != expected_scope {
        return Err(TransactionalOutboxError::Sql(
            SqlCommandError::TenantScopeMustPrecedeStatements,
        ));
    }
    let mut statements = batch.statements;
    statements.push(outbox_insert_command(table, tenant, envelope)?);
    SqlWriteBatch::new(tenant, statements).map_err(TransactionalOutboxError::Sql)
}

pub fn outbox_insert_command(
    table: BackboneOutboxTable,
    tenant: &TenantSqlContext,
    envelope: &ProtocolEventEnvelope,
) -> Result<SqlCommand, TransactionalOutboxError> {
    let summary = outbox_event_summary(table, tenant, envelope)?;
    SqlCommand::new(
        "insert_transactional_outbox_event",
        table.insert_sql(),
        outbox_params(tenant, envelope, &summary)?,
    )
    .map_err(TransactionalOutboxError::Sql)
}

pub fn outbox_event_summary(
    table: BackboneOutboxTable,
    tenant: &TenantSqlContext,
    envelope: &ProtocolEventEnvelope,
) -> Result<TransactionalOutboxEventSummary, TransactionalOutboxError> {
    tenant.validate()?;
    envelope.validate()?;
    if tenant.tenant_id != envelope.tenant_scope_ref {
        return Err(TransactionalOutboxError::TenantScopeMismatch {
            tenant_id: tenant.tenant_id.clone(),
            envelope_scope_ref: envelope.tenant_scope_ref.clone(),
        });
    }
    Ok(TransactionalOutboxEventSummary {
        service_id: table.service_id(),
        table_name: table.table_name(),
        event_id: outbox_event_id(envelope),
        event_kind: envelope.binding.asyncapi_event_kind,
        aggregate_id: envelope.aggregate_id.clone(),
        audit_correlation_id: envelope.audit_correlation_id.clone(),
        policy_decision_ref: envelope.policy_decision_ref.clone(),
        idempotency_key: envelope.idempotency_key.clone(),
    })
}

fn outbox_params(
    tenant: &TenantSqlContext,
    envelope: &ProtocolEventEnvelope,
    summary: &TransactionalOutboxEventSummary,
) -> Result<Vec<SqlParam>, SqlCommandError> {
    let mut params = tenant.routing_params()?;
    params.push(SqlParam::text(summary.service_id));
    params.push(SqlParam::text(summary.event_id.clone()));
    params.push(SqlParam::text(summary.event_kind));
    params.push(SqlParam::text(summary.aggregate_id.clone()));
    params.push(SqlParam::text(envelope.binding.asyncapi_operation_id));
    params.push(SqlParam::text(envelope.binding.asyncapi_channel_address));
    params.push(SqlParam::text(envelope.binding.asyncapi_message_name));
    params.push(SqlParam::text(envelope.binding.proto_package));
    params.push(SqlParam::text(envelope.binding.proto_service));
    params.push(SqlParam::text(envelope.binding.proto_rpc));
    params.push(SqlParam::text(envelope.schema_version.clone()));
    params.push(SqlParam::nullable_text(summary.idempotency_key.clone()));
    params.push(SqlParam::text(summary.policy_decision_ref.clone()));
    params.push(SqlParam::text(summary.audit_correlation_id.clone()));
    Ok(params)
}

fn outbox_event_id(envelope: &ProtocolEventEnvelope) -> String {
    let idempotency = envelope
        .idempotency_key
        .as_deref()
        .unwrap_or("no-idempotency");
    format!(
        "{}:{}:{}:{}",
        envelope.binding.asyncapi_event_kind,
        envelope.aggregate_id,
        envelope.audit_correlation_id,
        idempotency
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_postgres_command_kernel::{SqlCommand, SqlParam};
    use shared_protocol_parity_kernel::{ProtocolParityBinding, ProtocolParityBindingSpec};

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap()
    }

    fn envelope() -> ProtocolEventEnvelope {
        ProtocolEventEnvelope::new(
            ProtocolParityBinding::new(ProtocolParityBindingSpec {
                rest_operation_id: "postMessage",
                asyncapi_operation_id: "emitMessagePosted",
                asyncapi_channel_address: "workflow-events/messenger.message.posted",
                asyncapi_message_name: "MessagePosted",
                asyncapi_event_kind: "oya.messenger.message.posted.v1",
                receipt_event_type: "messenger.message.sent",
                proto_package: "oya.messenger.v1",
                proto_service: "MessageStream",
                proto_rpc: "PostMessage",
            })
            .unwrap(),
            "1.0.0",
            "tenant:t",
            "message:m",
            "audit:a",
            Some("idem:i".into()),
            "policy:p",
        )
        .unwrap()
    }

    fn base_batch() -> SqlWriteBatch {
        SqlWriteBatch::new(
            &tenant(),
            vec![SqlCommand::new(
                "insert_business_row",
                "INSERT INTO messenger_message_stream.messages (tenant_id, message_id) VALUES ($1, $2)",
                vec![SqlParam::text("tenant:t"), SqlParam::text("message:m")],
            )
            .unwrap()],
        )
        .unwrap()
    }

    #[test]
    fn appends_outbox_command_after_business_statements() {
        let batch = append_outbox_to_batch(
            BackboneOutboxTable::MessengerMessageStream,
            &tenant(),
            base_batch(),
            &envelope(),
        )
        .unwrap();

        assert_eq!(batch.statements.len(), 2);
        assert_eq!(
            batch.statements[1].name,
            "insert_transactional_outbox_event"
        );
        assert_eq!(batch.statements[1].sql, INSERT_MESSENGER_OUTBOX_SQL);
        assert_eq!(batch.statements[1].params.len(), 18);
        assert!(matches!(
            batch.statements[1].params[15],
            SqlParam::NullableText(Some(ref value)) if value == "idem:i"
        ));
    }

    #[test]
    fn summary_preserves_dispatch_metadata_without_payload_body() {
        let summary = outbox_event_summary(
            BackboneOutboxTable::MessengerMessageStream,
            &tenant(),
            &envelope(),
        )
        .unwrap();

        assert_eq!(summary.service_id, "messenger-message-stream");
        assert_eq!(
            summary.table_name,
            "messenger_message_stream.protocol_outbox_events"
        );
        assert_eq!(summary.event_kind, "oya.messenger.message.posted.v1");
        assert!(summary.event_id.contains("message:m"));
        assert_eq!(summary.audit_correlation_id, "audit:a");
    }

    #[test]
    fn rejects_tenant_scope_drift_between_sql_context_and_protocol_event() {
        let mut other_tenant = tenant();
        other_tenant.tenant_id = "tenant:other".into();

        assert_eq!(
            outbox_event_summary(
                BackboneOutboxTable::MessengerMessageStream,
                &other_tenant,
                &envelope(),
            ),
            Err(TransactionalOutboxError::TenantScopeMismatch {
                tenant_id: "tenant:other".into(),
                envelope_scope_ref: "tenant:t".into(),
            })
        );
    }

    #[test]
    fn table_insert_sql_is_service_specific() {
        assert!(
            BackboneOutboxTable::MailMailboxStore
                .insert_sql()
                .contains("mail_mailbox_store.protocol_outbox_events")
        );
        assert!(
            BackboneOutboxTable::SocialPostComposition
                .insert_sql()
                .contains("social_post_composition.protocol_outbox_events")
        );
        assert!(
            BackboneOutboxTable::CommunityPostStore
                .insert_sql()
                .contains("community_post_store.protocol_outbox_events")
        );
    }
}
