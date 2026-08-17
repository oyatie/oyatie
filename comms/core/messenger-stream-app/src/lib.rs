//! App-layer write orchestration for messenger message-stream.
//!
//! This crate composes the API/usecase, persistence-command, and protocol-parity
//! seams into one runtime-neutral write plan. It still performs no database I/O,
//! broker publish, gateway routing, or async runtime work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use comms_messenger_stream_api::{
    AuthorizedMessengerContext, MessageReceipt, MessengerApiEnvelope, SendMessageRequest,
    message_posted_event_envelope,
};
use comms_messenger_stream_postgres::{PersistMessageRecord, build_message_write_batch};
use comms_messenger_stream_usecase::{MessengerUsecaseError, send_message};
use oya_shared_postgres_command_kernel::{
    PostgresPoolConfig, SqlCommandError, SqlExecutionPlan, SqlWriteBatch, TenantSqlContext,
};
use oya_shared_protocol_parity_kernel::{ProtocolEventEnvelope, ProtocolParityError};
use oya_shared_protocol_transport_kernel::{
    ProtocolTransportBudget, ProtocolTransportBundle, ProtocolTransportError,
    plan_transport_from_envelope,
};
use oya_shared_transactional_outbox_kernel::{
    BackboneOutboxTable, TransactionalOutboxError, append_outbox_to_batch,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessengerAppError {
    ScopeMismatch {
        // data_class: INTERNAL_ONLY
        context_scope_ref: String, // data_class: INTERNAL_ONLY
        tenant_id: String,         // data_class: INTERNAL_ONLY
    },
    Usecase(MessengerUsecaseError),
    Sql(SqlCommandError),
    Protocol(ProtocolParityError),
    Transport(ProtocolTransportError),
    Outbox(TransactionalOutboxError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessengerWritePlan {
    pub receipt: MessageReceipt,               // data_class: INTERNAL_ONLY
    pub persistence: SqlWriteBatch,            // data_class: INTERNAL_ONLY
    pub sql_execution: SqlExecutionPlan,       // data_class: INTERNAL_ONLY
    pub protocol_event: ProtocolEventEnvelope, // data_class: INTERNAL_ONLY
    pub transport: ProtocolTransportBundle,    // data_class: INTERNAL_ONLY
}

pub fn plan_send_message(
    tenant: TenantSqlContext,
    context: AuthorizedMessengerContext,
    request: SendMessageRequest,
) -> Result<MessengerWritePlan, MessengerAppError> {
    require_scope_match(&tenant, &context.scope_ref)?;
    let outbox_tenant = tenant.clone();
    let envelope_ref = envelope_ref(&request.envelope);
    let persistence_record = PersistMessageRecord {
        tenant,
        channel_id: request.channel_id.clone(),
        message_id: request.message_id.clone(),
        author_ref: request.author_ref.clone(),
        envelope_ref,
        retention_policy_id: request.retention_policy_id.clone(),
        legal_hold_ids: request.legal_hold_ids.clone(),
        policy_decision_ref: context.policy_decision_ref.clone(),
        idempotency_key: context.idempotency_key.clone(),
        audit_correlation_id: context.audit_correlation_id.clone(),
    };

    let (_, receipt) = send_message(&context, request).map_err(MessengerAppError::Usecase)?;
    let persistence =
        build_message_write_batch(&persistence_record).map_err(MessengerAppError::Sql)?;
    let protocol_event =
        message_posted_event_envelope(&context, &receipt).map_err(MessengerAppError::Protocol)?;
    let persistence = append_outbox_to_batch(
        BackboneOutboxTable::MessengerMessageStream,
        &outbox_tenant,
        persistence,
        &protocol_event,
    )
    .map_err(MessengerAppError::Outbox)?;
    let sql_execution = SqlExecutionPlan::from_batch(
        PostgresPoolConfig::for_microservice("messenger", 16).map_err(MessengerAppError::Sql)?,
        persistence.clone(),
    )
    .map_err(MessengerAppError::Sql)?;
    let transport =
        plan_transport_from_envelope(&protocol_event, ProtocolTransportBudget::default())
            .map_err(MessengerAppError::Transport)?;

    Ok(MessengerWritePlan {
        receipt,
        persistence,
        sql_execution,
        protocol_event,
        transport,
    })
}

fn require_scope_match(
    tenant: &TenantSqlContext,
    context_scope_ref: &str,
) -> Result<(), MessengerAppError> {
    if tenant.tenant_id == context_scope_ref {
        Ok(())
    } else {
        Err(MessengerAppError::ScopeMismatch {
            context_scope_ref: context_scope_ref.to_string(),
            tenant_id: tenant.tenant_id.clone(),
        })
    }
}

fn envelope_ref(envelope: &MessengerApiEnvelope) -> String {
    match envelope {
        MessengerApiEnvelope::PersonalE2e { envelope_ref } => envelope_ref.clone(),
        MessengerApiEnvelope::TenantDek { dek_ref, .. } => dek_ref.clone(),
        MessengerApiEnvelope::CrossOrg { local_dek_ref, .. } => local_dek_ref.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comms_messenger_stream_api::MessengerApiContext;
    use oya_shared_postgres_command_kernel::SqlParam;

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap()
    }

    fn context() -> AuthorizedMessengerContext {
        AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    fn request() -> SendMessageRequest {
        SendMessageRequest {
            message_id: "message:m".into(),
            channel_id: "channel:c".into(),
            author_ref: "user:u".into(),
            envelope: MessengerApiEnvelope::TenantDek {
                dek_ref: "dek:d".into(),
                four_eyes: true,
            },
            retention_policy_id: "retain".into(),
            legal_hold_ids: vec!["hold:h".into()],
        }
    }

    #[test]
    fn send_message_plan_composes_receipt_persistence_and_protocol_event() {
        let plan = plan_send_message(tenant(), context(), request()).unwrap();

        assert_eq!(plan.receipt.event_type, "messenger.message.sent");
        assert_eq!(
            plan.persistence.tenant_scope.name,
            "set_local_oyatie_tenant"
        );
        assert_eq!(plan.persistence.statements.len(), 3);
        assert_eq!(plan.sql_execution.total_command_count, 4);
        assert_eq!(plan.sql_execution.pool.application_name, "oyatie-messenger");
        assert_eq!(plan.protocol_event.binding.proto_rpc, "PostMessage");
        assert_eq!(
            plan.transport.broker_publish.event_kind,
            "oya.messenger.message.posted.v1"
        );
        assert_eq!(plan.transport.grpc_unary.rpc, "PostMessage");
        assert_eq!(plan.protocol_event.policy_decision_ref, "policy");
        assert_eq!(
            plan.persistence.statements[2].name,
            "insert_transactional_outbox_event"
        );
        assert!(
            plan.persistence.statements[2]
                .sql
                .contains("messenger_message_stream.protocol_outbox_events")
        );
        assert!(matches!(
            plan.persistence.statements[0].params[7],
            SqlParam::Text(ref value) if value == "dek:d"
        ));
    }

    #[test]
    fn send_message_plan_rejects_scope_drift_before_building_sql() {
        let mut tenant = tenant();
        tenant.tenant_id = "tenant:other".into();

        assert_eq!(
            plan_send_message(tenant, context(), request()),
            Err(MessengerAppError::ScopeMismatch {
                context_scope_ref: "tenant:t".into(),
                tenant_id: "tenant:other".into()
            })
        );
    }
}
