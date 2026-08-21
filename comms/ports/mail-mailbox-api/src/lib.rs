#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use shared_protocol_parity_kernel::{
    ProtocolEventEnvelope, ProtocolParityBinding, ProtocolParityBindingSpec, ProtocolParityError,
    require_receipt_event_type,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailApiError {
    Invalid,
    MissingPersonalScope,
    MissingTenantScope,
    MissingPolicyDecision,
    MissingIdempotencyKey,
    MissingAuditCorrelation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailApiContext {
    Personal,
    Work,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailApiEnvelope {
    PersonalClientOnly {
        envelope_ref: String,
    },
    TenantDek {
        dek_ref: String,
    },
    Imported {
        source_hash: String,
        evidence_ref: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedMailContext {
    pub context: MailApiContext,
    pub scope_ref: String,
    pub principal_ref: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
    pub audit_correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowHandoffRequest {
    pub message_id: String,
    pub mailbox_id: String,
    pub subject_ref: String,
    pub envelope: MailApiEnvelope,
    pub retention_policy_id: String,
    pub legal_hold_ids: Vec<String>,
    pub lawful_basis_ref: String,
    pub policy_snapshot_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiAssistRequest {
    pub message_id: String,
    pub mailbox_id: String,
    pub subject_ref: String,
    pub envelope: MailApiEnvelope,
    pub retention_policy_id: String,
    pub encrypted_content_ref: String,
    pub plaintext_in_prompt: bool,
    pub training_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DmarcCheckRequest {
    pub domain_ref: String,
    pub spf_aligned: bool,
    pub dkim_aligned: bool,
    pub policy: DmarcApiPolicy,
    pub evidence_ref: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmarcApiPolicy {
    None,
    Quarantine,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmarcApiAction {
    Accept,
    Quarantine,
    Reject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffReceipt {
    pub message_id: String,
    pub event_type: &'static str,
    pub audit_correlation_id: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmitMessageRequest {
    pub message_id: String,
    pub mailbox_id: String,
    pub subject_ref: String,
    pub envelope: MailApiEnvelope,
    pub retention_policy_id: String,
    pub dmarc_check: Option<DmarcCheckRequest>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmissionReceipt {
    pub message_id: String,
    pub event_type: &'static str,
    pub audit_correlation_id: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
    pub dmarc_action: DmarcApiAction,
}

pub const MESSAGE_SENT_PROTOCOL_SCHEMA_VERSION: &str = "1.0.0";

pub fn message_sent_protocol_binding() -> Result<ProtocolParityBinding, ProtocolParityError> {
    ProtocolParityBinding::new(ProtocolParityBindingSpec {
        rest_operation_id: "sendMessage",
        asyncapi_operation_id: "emitMessageSent",
        asyncapi_channel_address: "workflow-events/mail.message.sent",
        asyncapi_message_name: "MessageSent",
        asyncapi_event_kind: "oya.mail.message.sent.v1",
        receipt_event_type: "mail.message.submitted",
        proto_package: "oya.mail.v1",
        proto_service: "Mail",
        proto_rpc: "SendMessage",
    })
}

pub fn message_sent_event_envelope(
    context: &AuthorizedMailContext,
    receipt: &SubmissionReceipt,
) -> Result<ProtocolEventEnvelope, ProtocolParityError> {
    let binding = message_sent_protocol_binding()?;
    require_receipt_event_type(&binding, receipt.event_type)?;
    ProtocolEventEnvelope::new(
        binding,
        MESSAGE_SENT_PROTOCOL_SCHEMA_VERSION,
        context.scope_ref.clone(),
        receipt.message_id.clone(),
        receipt.audit_correlation_id.clone(),
        Some(receipt.idempotency_key.clone()),
        receipt.policy_decision_ref.clone(),
    )
}

impl AuthorizedMailContext {
    pub fn validate(&self) -> Result<(), MailApiError> {
        non_empty(&self.scope_ref)?;
        non_empty(&self.principal_ref)?;
        match self.context {
            MailApiContext::Personal if !self.scope_ref.starts_with("person:") => {
                return Err(MailApiError::MissingPersonalScope);
            }
            MailApiContext::Work if !self.scope_ref.starts_with("tenant:") => {
                return Err(MailApiError::MissingTenantScope);
            }
            _ => {}
        }
        if self.idempotency_key.trim().is_empty() {
            return Err(MailApiError::MissingIdempotencyKey);
        }
        if self.policy_decision_ref.trim().is_empty() {
            return Err(MailApiError::MissingPolicyDecision);
        }
        if self.audit_correlation_id.trim().is_empty() {
            return Err(MailApiError::MissingAuditCorrelation);
        }
        Ok(())
    }
}

fn non_empty(value: &str) -> Result<(), MailApiError> {
    if value.trim().is_empty() {
        Err(MailApiError::Invalid)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_context_requires_tenant_scope_and_idempotency() {
        let ctx = AuthorizedMailContext {
            context: MailApiContext::Work,
            scope_ref: "person:u".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(ctx.validate(), Err(MailApiError::MissingTenantScope));

        let ctx = AuthorizedMailContext {
            context: MailApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "".into(),
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(ctx.validate(), Err(MailApiError::MissingIdempotencyKey));
    }

    #[test]
    fn personal_context_requires_person_scope_and_policy_decision() {
        let ctx = AuthorizedMailContext {
            context: MailApiContext::Personal,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(ctx.validate(), Err(MailApiError::MissingPersonalScope));

        let ctx = AuthorizedMailContext {
            context: MailApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(ctx.validate(), Err(MailApiError::MissingPolicyDecision));
    }

    #[test]
    fn message_sent_protocol_binding_matches_asyncapi_and_proto_contracts() {
        let binding = message_sent_protocol_binding().unwrap();

        assert_eq!(binding.rest_operation_id, "sendMessage");
        assert_eq!(binding.asyncapi_operation_id, "emitMessageSent");
        assert_eq!(
            binding.asyncapi_channel_address,
            "workflow-events/mail.message.sent"
        );
        assert_eq!(binding.asyncapi_message_name, "MessageSent");
        assert_eq!(binding.asyncapi_event_kind, "oya.mail.message.sent.v1");
        assert_eq!(binding.receipt_event_type, "mail.message.submitted");
        assert_eq!(binding.proto_package, "oya.mail.v1");
        assert_eq!(binding.proto_service, "Mail");
        assert_eq!(binding.proto_rpc, "SendMessage");
    }

    #[test]
    fn message_sent_event_envelope_carries_policy_audit_and_idempotency_refs() {
        let context = AuthorizedMailContext {
            context: MailApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        };
        let receipt = SubmissionReceipt {
            message_id: "message:m".into(),
            event_type: "mail.message.submitted",
            audit_correlation_id: "audit".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            dmarc_action: DmarcApiAction::Accept,
        };

        let envelope = message_sent_event_envelope(&context, &receipt).unwrap();

        assert_eq!(envelope.aggregate_id, "message:m");
        assert_eq!(envelope.tenant_scope_ref, "tenant:t");
        assert_eq!(envelope.idempotency_key, Some("idem".into()));
        assert_eq!(envelope.binding.asyncapi_operation_id, "emitMessageSent");
    }
}
