#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod delivery_receipt;
pub mod mention_fanout;
pub use delivery_receipt::{
    ChannelDeliveryAggregate, DeliveryStatus, RecipientDeliveryState, acknowledge_delivery,
    aggregate_channel_delivery,
};
pub use mention_fanout::{MentionFanout, MentionFanoutInput, derive_mention_fanout};

use comms_messenger_domain::{
    MessageAuditAction, MessageAuditRecord, MessageEnvelope, MessageGovernance,
    MessageGovernanceCreate, MessengerContextKind, MessengerGovernanceError,
    OwnershipPillar as MessengerOwnershipPillar,
};
use comms_messenger_stream_api::{
    AuthorizedMessengerContext, MessageReceipt, MessengerApiContext, MessengerApiEnvelope,
    MessengerApiError, SendMessageRequest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessengerUsecaseError {
    Api(MessengerApiError),
    Domain(MessengerGovernanceError),
    PrincipalMismatch,
    IllegalDeliveryTransition {
        from: DeliveryStatus,
        to: DeliveryStatus,
    },
}

pub fn send_message(
    ctx: &AuthorizedMessengerContext,
    req: SendMessageRequest,
) -> Result<(MessageGovernance, MessageReceipt), MessengerUsecaseError> {
    ctx.validate().map_err(MessengerUsecaseError::Api)?;
    if req.author_ref != ctx.principal_ref {
        return Err(MessengerUsecaseError::PrincipalMismatch);
    }
    let message_id = req.message_id.clone();
    let channel_id = req.channel_id.clone();
    let governance = MessageGovernance::new(MessageGovernanceCreate {
        message_id,
        scope_ref: ctx.scope_ref.clone(),
        context: map_context(ctx.context),
        pillar: map_pillar(ctx.context),
        author_ref: req.author_ref,
        envelope: map_envelope(req.envelope),
        retention_policy_id: req.retention_policy_id,
        legal_hold_ids: req.legal_hold_ids,
    })
    .map_err(MessengerUsecaseError::Domain)?;
    let receipt = MessageReceipt {
        message_id: governance.message_id.value.clone(),
        channel_id,
        event_type: "messenger.message.sent",
        audit_correlation_id: ctx.audit_correlation_id.clone(),
        idempotency_key: ctx.idempotency_key.clone(),
        policy_decision_ref: ctx.policy_decision_ref.clone(),
    };
    Ok((governance, receipt))
}

pub fn prepare_disclosure_audit(
    ctx: &AuthorizedMessengerContext,
    message: &MessageGovernance,
    approved_by: String,
    reason_ref: String,
    audit_chain_ref: String,
) -> Result<MessageAuditRecord, MessengerUsecaseError> {
    ctx.validate().map_err(MessengerUsecaseError::Api)?;
    MessageAuditRecord::new(
        message,
        MessageAuditAction::DisclosureDecrypt,
        ctx.principal_ref.clone(),
        approved_by,
        reason_ref,
        audit_chain_ref,
    )
    .map_err(MessengerUsecaseError::Domain)
}

fn map_context(context: MessengerApiContext) -> MessengerContextKind {
    match context {
        MessengerApiContext::Personal => MessengerContextKind::Personal,
        MessengerApiContext::Work => MessengerContextKind::Work,
    }
}

fn map_pillar(context: MessengerApiContext) -> MessengerOwnershipPillar {
    match context {
        MessengerApiContext::Personal => MessengerOwnershipPillar::Personal,
        MessengerApiContext::Work => MessengerOwnershipPillar::Work,
    }
}

fn map_envelope(envelope: MessengerApiEnvelope) -> MessageEnvelope {
    match envelope {
        MessengerApiEnvelope::PersonalE2e { envelope_ref } => {
            MessageEnvelope::PersonalE2e { envelope_ref }
        }
        MessengerApiEnvelope::TenantDek { dek_ref, four_eyes } => {
            MessageEnvelope::TenantDek { dek_ref, four_eyes }
        }
        MessengerApiEnvelope::CrossOrg {
            local_dek_ref,
            partner_scope_ref,
            partner_dek_ref,
            partner_ediscovery_allowed,
        } => MessageEnvelope::CrossOrg {
            local_dek_ref,
            partner_scope_ref,
            partner_dek_ref,
            partner_ediscovery_allowed,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work_ctx() -> AuthorizedMessengerContext {
        AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow:message-send".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    #[test]
    fn send_message_requires_authorized_principal() {
        let req = SendMessageRequest {
            message_id: "m".into(),
            channel_id: "c".into(),
            author_ref: "user:other".into(),
            envelope: MessengerApiEnvelope::TenantDek {
                dek_ref: "dek".into(),
                four_eyes: true,
            },
            retention_policy_id: "retain".into(),
            legal_hold_ids: vec![],
        };
        assert_eq!(
            send_message(&work_ctx(), req),
            Err(MessengerUsecaseError::PrincipalMismatch)
        );
    }

    #[test]
    fn work_message_requires_four_eyes_envelope() {
        let req = SendMessageRequest {
            message_id: "m".into(),
            channel_id: "c".into(),
            author_ref: "user:u".into(),
            envelope: MessengerApiEnvelope::TenantDek {
                dek_ref: "dek".into(),
                four_eyes: false,
            },
            retention_policy_id: "retain".into(),
            legal_hold_ids: vec![],
        };
        assert_eq!(
            send_message(&work_ctx(), req),
            Err(MessengerUsecaseError::Domain(
                MessengerGovernanceError::FourEyesRequired
            ))
        );
    }

    #[test]
    fn personal_message_not_discoverable_by_disclosure_audit() {
        let ctx = AuthorizedMessengerContext {
            context: MessengerApiContext::Personal,
            scope_ref: "person:u".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow:message-send".into(),
            audit_correlation_id: "audit".into(),
        };
        let req = SendMessageRequest {
            message_id: "m".into(),
            channel_id: "c".into(),
            author_ref: "user:u".into(),
            envelope: MessengerApiEnvelope::PersonalE2e {
                envelope_ref: "sealed".into(),
            },
            retention_policy_id: "retain".into(),
            legal_hold_ids: vec![],
        };
        let (message, _) = send_message(&ctx, req).unwrap();
        assert_eq!(
            prepare_disclosure_audit(
                &ctx,
                &message,
                "user:approver".into(),
                "reason".into(),
                "audit-chain".into()
            ),
            Err(MessengerUsecaseError::Domain(
                MessengerGovernanceError::PersonalNotDiscoverable
            ))
        );
    }
}
