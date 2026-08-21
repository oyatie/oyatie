#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use shared_protocol_parity_kernel::{
    ProtocolEventEnvelope, ProtocolParityBinding, ProtocolParityBindingSpec, ProtocolParityError,
    require_receipt_event_type,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessengerApiError {
    Invalid,
    MissingPersonalScope,
    MissingTenantScope,
    MissingPolicyDecision,
    MissingIdempotencyKey,
    MissingAuditCorrelation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessengerApiContext {
    Personal,
    Work,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessengerApiEnvelope {
    PersonalE2e {
        envelope_ref: String,
    },
    TenantDek {
        dek_ref: String,
        four_eyes: bool,
    },
    CrossOrg {
        local_dek_ref: String,
        partner_scope_ref: String,
        partner_dek_ref: String,
        partner_ediscovery_allowed: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedMessengerContext {
    pub context: MessengerApiContext,
    pub scope_ref: String,
    pub principal_ref: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
    pub audit_correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SendMessageRequest {
    pub message_id: String,
    pub channel_id: String,
    pub author_ref: String,
    pub envelope: MessengerApiEnvelope,
    pub retention_policy_id: String,
    pub legal_hold_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageReceipt {
    pub message_id: String,
    pub channel_id: String,
    pub event_type: &'static str,
    pub audit_correlation_id: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
}

pub const MESSAGE_POSTED_PROTOCOL_SCHEMA_VERSION: &str = "1.0.0";

pub fn message_posted_protocol_binding() -> Result<ProtocolParityBinding, ProtocolParityError> {
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
}

pub fn message_posted_event_envelope(
    context: &AuthorizedMessengerContext,
    receipt: &MessageReceipt,
) -> Result<ProtocolEventEnvelope, ProtocolParityError> {
    let binding = message_posted_protocol_binding()?;
    require_receipt_event_type(&binding, receipt.event_type)?;
    ProtocolEventEnvelope::new(
        binding,
        MESSAGE_POSTED_PROTOCOL_SCHEMA_VERSION,
        context.scope_ref.clone(),
        receipt.message_id.clone(),
        receipt.audit_correlation_id.clone(),
        Some(receipt.idempotency_key.clone()),
        receipt.policy_decision_ref.clone(),
    )
}

impl AuthorizedMessengerContext {
    pub fn validate(&self) -> Result<(), MessengerApiError> {
        non_empty(&self.scope_ref)?;
        non_empty(&self.principal_ref)?;
        match self.context {
            MessengerApiContext::Personal if !self.scope_ref.starts_with("person:") => {
                return Err(MessengerApiError::MissingPersonalScope);
            }
            MessengerApiContext::Work if !self.scope_ref.starts_with("tenant:") => {
                return Err(MessengerApiError::MissingTenantScope);
            }
            _ => {}
        }
        if self.idempotency_key.trim().is_empty() {
            return Err(MessengerApiError::MissingIdempotencyKey);
        }
        if self.policy_decision_ref.trim().is_empty() {
            return Err(MessengerApiError::MissingPolicyDecision);
        }
        if self.audit_correlation_id.trim().is_empty() {
            return Err(MessengerApiError::MissingAuditCorrelation);
        }
        Ok(())
    }
}

fn non_empty(value: &str) -> Result<(), MessengerApiError> {
    if value.trim().is_empty() {
        Err(MessengerApiError::Invalid)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_context_requires_tenant_scope_and_idempotency() {
        let ctx = AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "person:u".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "k".into(),
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(ctx.validate(), Err(MessengerApiError::MissingTenantScope));

        let ctx = AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "".into(),
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(
            ctx.validate(),
            Err(MessengerApiError::MissingIdempotencyKey)
        );
    }

    #[test]
    fn personal_context_requires_person_scope_and_policy_decision() {
        let ctx = AuthorizedMessengerContext {
            context: MessengerApiContext::Personal,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "k".into(),
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(ctx.validate(), Err(MessengerApiError::MissingPersonalScope));

        let ctx = AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "k".into(),
            policy_decision_ref: "".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(
            ctx.validate(),
            Err(MessengerApiError::MissingPolicyDecision)
        );
    }

    #[test]
    fn message_posted_protocol_binding_matches_asyncapi_and_proto_contracts() {
        let binding = message_posted_protocol_binding().unwrap();

        assert_eq!(binding.rest_operation_id, "postMessage");
        assert_eq!(binding.asyncapi_operation_id, "emitMessagePosted");
        assert_eq!(
            binding.asyncapi_channel_address,
            "workflow-events/messenger.message.posted"
        );
        assert_eq!(binding.asyncapi_message_name, "MessagePosted");
        assert_eq!(
            binding.asyncapi_event_kind,
            "oya.messenger.message.posted.v1"
        );
        assert_eq!(binding.receipt_event_type, "messenger.message.sent");
        assert_eq!(binding.proto_package, "oya.messenger.v1");
        assert_eq!(binding.proto_service, "MessageStream");
        assert_eq!(binding.proto_rpc, "PostMessage");
    }

    #[test]
    fn message_posted_event_envelope_carries_policy_audit_and_idempotency_refs() {
        let context = AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        };
        let receipt = MessageReceipt {
            message_id: "message:m".into(),
            channel_id: "channel:c".into(),
            event_type: "messenger.message.sent",
            audit_correlation_id: "audit".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
        };

        let envelope = message_posted_event_envelope(&context, &receipt).unwrap();

        assert_eq!(envelope.schema_version, "1.0.0");
        assert_eq!(envelope.tenant_scope_ref, "tenant:t");
        assert_eq!(envelope.aggregate_id, "message:m");
        assert_eq!(envelope.audit_correlation_id, "audit");
        assert_eq!(envelope.idempotency_key, Some("idem".into()));
        assert_eq!(envelope.policy_decision_ref, "policy");
    }
}
