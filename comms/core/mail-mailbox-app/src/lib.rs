//! App-layer write orchestration for mail mailbox-store submissions.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use comms_mail_mailbox_api::{
    AuthorizedMailContext, DmarcApiAction, MailApiEnvelope, SubmissionReceipt,
    SubmitMessageRequest, message_sent_event_envelope,
};
use comms_mail_mailbox_postgres::{PersistMailMessageRecord, build_mail_message_write_batch};
use comms_mail_mailbox_usecase::{MailUsecaseError, submit_message};
use shared_postgres_command_kernel::{
    PostgresPoolConfig, SqlCommandError, SqlExecutionPlan, SqlWriteBatch, TenantSqlContext,
};
use shared_protocol_parity_kernel::{ProtocolEventEnvelope, ProtocolParityError};
use shared_protocol_transport_kernel::{
    ProtocolTransportBudget, ProtocolTransportBundle, ProtocolTransportError,
    plan_transport_from_envelope,
};
use shared_transactional_outbox_kernel::{
    BackboneOutboxTable, TransactionalOutboxError, append_outbox_to_batch,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailAppError {
    ScopeMismatch {
        // data_class: INTERNAL_ONLY
        context_scope_ref: String, // data_class: INTERNAL_ONLY
        tenant_id: String,         // data_class: INTERNAL_ONLY
    },
    Usecase(MailUsecaseError),
    Sql(SqlCommandError),
    Protocol(ProtocolParityError),
    Transport(ProtocolTransportError),
    Outbox(TransactionalOutboxError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSubmissionPlan {
    pub receipt: SubmissionReceipt,            // data_class: INTERNAL_ONLY
    pub persistence: SqlWriteBatch,            // data_class: INTERNAL_ONLY
    pub sql_execution: SqlExecutionPlan,       // data_class: INTERNAL_ONLY
    pub protocol_event: ProtocolEventEnvelope, // data_class: INTERNAL_ONLY
    pub transport: ProtocolTransportBundle,    // data_class: INTERNAL_ONLY
}

pub fn plan_submit_message(
    tenant: TenantSqlContext,
    context: AuthorizedMailContext,
    request: SubmitMessageRequest,
) -> Result<MailSubmissionPlan, MailAppError> {
    require_scope_match(&tenant, &context.scope_ref)?;
    let outbox_tenant = tenant.clone();
    let envelope_ref = envelope_ref(&request.envelope);
    let mailbox_id = request.mailbox_id.clone();
    let subject_ref = request.subject_ref.clone();
    let retention_policy_id = request.retention_policy_id.clone();

    let receipt = submit_message(&context, request).map_err(MailAppError::Usecase)?;
    let persistence_record = PersistMailMessageRecord {
        tenant,
        mailbox_id,
        message_id: receipt.message_id.clone(),
        subject_ref,
        envelope_ref,
        retention_policy_id,
        dmarc_action: dmarc_action_name(receipt.dmarc_action).to_string(),
        policy_decision_ref: receipt.policy_decision_ref.clone(),
        idempotency_key: receipt.idempotency_key.clone(),
        audit_correlation_id: receipt.audit_correlation_id.clone(),
    };
    let persistence =
        build_mail_message_write_batch(&persistence_record).map_err(MailAppError::Sql)?;
    let protocol_event =
        message_sent_event_envelope(&context, &receipt).map_err(MailAppError::Protocol)?;
    let persistence = append_outbox_to_batch(
        BackboneOutboxTable::MailMailboxStore,
        &outbox_tenant,
        persistence,
        &protocol_event,
    )
    .map_err(MailAppError::Outbox)?;
    let sql_execution = SqlExecutionPlan::from_batch(
        PostgresPoolConfig::for_microservice("mail", 16).map_err(MailAppError::Sql)?,
        persistence.clone(),
    )
    .map_err(MailAppError::Sql)?;
    let transport =
        plan_transport_from_envelope(&protocol_event, ProtocolTransportBudget::default())
            .map_err(MailAppError::Transport)?;

    Ok(MailSubmissionPlan {
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
) -> Result<(), MailAppError> {
    if tenant.tenant_id == context_scope_ref {
        Ok(())
    } else {
        Err(MailAppError::ScopeMismatch {
            context_scope_ref: context_scope_ref.to_string(),
            tenant_id: tenant.tenant_id.clone(),
        })
    }
}

fn envelope_ref(envelope: &MailApiEnvelope) -> String {
    match envelope {
        MailApiEnvelope::PersonalClientOnly { envelope_ref } => envelope_ref.clone(),
        MailApiEnvelope::TenantDek { dek_ref } => dek_ref.clone(),
        MailApiEnvelope::Imported {
            source_hash,
            evidence_ref,
        } => format!("{source_hash}#{evidence_ref}"),
    }
}

fn dmarc_action_name(action: DmarcApiAction) -> &'static str {
    match action {
        DmarcApiAction::Accept => "accept",
        DmarcApiAction::Quarantine => "quarantine",
        DmarcApiAction::Reject => "reject",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comms_mail_mailbox_api::{DmarcApiPolicy, DmarcCheckRequest, MailApiContext};
    use shared_postgres_command_kernel::SqlParam;

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap()
    }

    fn context() -> AuthorizedMailContext {
        AuthorizedMailContext {
            context: MailApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    fn request() -> SubmitMessageRequest {
        SubmitMessageRequest {
            message_id: "message:m".into(),
            mailbox_id: "mailbox:b".into(),
            subject_ref: "user:u".into(),
            envelope: MailApiEnvelope::TenantDek {
                dek_ref: "dek:d".into(),
            },
            retention_policy_id: "retain".into(),
            dmarc_check: Some(DmarcCheckRequest {
                domain_ref: "domain:d".into(),
                spf_aligned: false,
                dkim_aligned: false,
                policy: DmarcApiPolicy::Quarantine,
                evidence_ref: "evidence:e".into(),
            }),
        }
    }

    #[test]
    fn submit_message_plan_composes_receipt_persistence_and_protocol_event() {
        let plan = plan_submit_message(tenant(), context(), request()).unwrap();

        assert_eq!(plan.receipt.event_type, "mail.message.submitted");
        assert_eq!(plan.receipt.dmarc_action, DmarcApiAction::Quarantine);
        assert_eq!(plan.persistence.statements.len(), 3);
        assert_eq!(plan.sql_execution.total_command_count, 4);
        assert_eq!(plan.sql_execution.pool.application_name, "oyatie-mail");
        assert_eq!(plan.protocol_event.binding.proto_service, "Mail");
        assert_eq!(
            plan.transport.broker_publish.channel_address,
            "workflow-events/mail.message.sent"
        );
        assert_eq!(plan.transport.grpc_unary.rpc, "SendMessage");
        assert_eq!(
            plan.persistence.statements[2].name,
            "insert_transactional_outbox_event"
        );
        assert!(
            plan.persistence.statements[2]
                .sql
                .contains("mail_mailbox_store.protocol_outbox_events")
        );
        assert!(matches!(
            plan.persistence.statements[0].params[9],
            SqlParam::Text(ref value) if value == "quarantine"
        ));
    }

    #[test]
    fn submit_message_plan_rejects_scope_drift_before_building_sql() {
        let mut tenant = tenant();
        tenant.tenant_id = "tenant:other".into();

        assert_eq!(
            plan_submit_message(tenant, context(), request()),
            Err(MailAppError::ScopeMismatch {
                context_scope_ref: "tenant:t".into(),
                tenant_id: "tenant:other".into()
            })
        );
    }
}
