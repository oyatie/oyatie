#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use shared_protocol_parity_kernel::{
    ProtocolEventEnvelope, ProtocolParityBinding, ProtocolParityBindingSpec, ProtocolParityError,
    require_receipt_event_type,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunityApiError {
    Invalid,
    MissingTenantScope,
    MissingPolicyDecision,
    MissingIdempotencyKey,
    MissingAuditCorrelation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunityApiMode {
    Reddit,
    Teamblind,
    Handshake,
    KnowledgeBase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoteDirection {
    Up,
    Down,
    Clear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModerationVerb {
    Allow,
    Hide,
    Remove,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedCommunityContext {
    pub tenant_scope_ref: String,
    pub principal_ref: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
    pub audit_correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatePostRequest {
    pub post_id: String,
    pub thread_id: String,
    pub mode: CommunityApiMode,
    pub routine_display_ref: String,
    pub audit_author_ref: String,
    pub disclosure_policy_ref: Option<String>,
    pub body_ref: String,
    pub retention_policy_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CastVoteRequest {
    pub post_id: String,
    pub voter_ref: String,
    pub direction: VoteDirection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeratePostRequest {
    pub policy_ref: String,
    pub evidence_ref: String,
    pub verb: ModerationVerb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostReceipt {
    pub post_id: String,
    pub event_type: &'static str,
    pub audit_correlation_id: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteReceiptEnvelope {
    pub post_id: String,
    pub vote_id: String,
    pub event_type: &'static str,
    pub policy_decision_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModerationReceipt {
    pub post_id: String,
    pub event_type: &'static str,
    pub evidence_ref: String,
    pub policy_decision_ref: String,
}

pub const COMMUNITY_PROTOCOL_SCHEMA_VERSION: &str = "1.0.0";

pub fn post_created_protocol_binding() -> Result<ProtocolParityBinding, ProtocolParityError> {
    ProtocolParityBinding::new(ProtocolParityBindingSpec {
        rest_operation_id: "createPost",
        asyncapi_operation_id: "publishPostCreated",
        asyncapi_channel_address: "community.post.created",
        asyncapi_message_name: "PostCreated",
        asyncapi_event_kind: "oya.community.post.created.v1",
        receipt_event_type: "community.post.created",
        proto_package: "oya.community.v1",
        proto_service: "PostStoreService",
        proto_rpc: "CreatePost",
    })
}

pub fn vote_cast_protocol_binding() -> Result<ProtocolParityBinding, ProtocolParityError> {
    ProtocolParityBinding::new(ProtocolParityBindingSpec {
        rest_operation_id: "castVote",
        asyncapi_operation_id: "publishVoteCast",
        asyncapi_channel_address: "community.vote.cast",
        asyncapi_message_name: "VoteCast",
        asyncapi_event_kind: "oya.community.vote.cast.v1",
        receipt_event_type: "community.vote.cast",
        proto_package: "oya.community.v1",
        proto_service: "VotingEngineService",
        proto_rpc: "CastVote",
    })
}

pub fn moderation_actioned_protocol_binding() -> Result<ProtocolParityBinding, ProtocolParityError>
{
    ProtocolParityBinding::new(ProtocolParityBindingSpec {
        rest_operation_id: "applyModerationAction",
        asyncapi_operation_id: "publishModerationActioned",
        asyncapi_channel_address: "community.moderation.actioned",
        asyncapi_message_name: "ModerationActioned",
        asyncapi_event_kind: "oya.community.moderation.actioned.v1",
        receipt_event_type: "community.moderation.actioned",
        proto_package: "oya.community.v1",
        proto_service: "ModerationQueueService",
        proto_rpc: "ApplyAction",
    })
}

pub fn post_created_event_envelope(
    context: &AuthorizedCommunityContext,
    receipt: &PostReceipt,
) -> Result<ProtocolEventEnvelope, ProtocolParityError> {
    let binding = post_created_protocol_binding()?;
    require_receipt_event_type(&binding, receipt.event_type)?;
    ProtocolEventEnvelope::new(
        binding,
        COMMUNITY_PROTOCOL_SCHEMA_VERSION,
        context.tenant_scope_ref.clone(),
        receipt.post_id.clone(),
        receipt.audit_correlation_id.clone(),
        Some(receipt.idempotency_key.clone()),
        receipt.policy_decision_ref.clone(),
    )
}

pub fn vote_cast_event_envelope(
    context: &AuthorizedCommunityContext,
    receipt: &VoteReceiptEnvelope,
) -> Result<ProtocolEventEnvelope, ProtocolParityError> {
    let binding = vote_cast_protocol_binding()?;
    require_receipt_event_type(&binding, receipt.event_type)?;
    ProtocolEventEnvelope::new(
        binding,
        COMMUNITY_PROTOCOL_SCHEMA_VERSION,
        context.tenant_scope_ref.clone(),
        receipt.post_id.clone(),
        context.audit_correlation_id.clone(),
        Some(context.idempotency_key.clone()),
        receipt.policy_decision_ref.clone(),
    )
}

pub fn moderation_actioned_event_envelope(
    context: &AuthorizedCommunityContext,
    receipt: &ModerationReceipt,
) -> Result<ProtocolEventEnvelope, ProtocolParityError> {
    let binding = moderation_actioned_protocol_binding()?;
    require_receipt_event_type(&binding, receipt.event_type)?;
    ProtocolEventEnvelope::new(
        binding,
        COMMUNITY_PROTOCOL_SCHEMA_VERSION,
        context.tenant_scope_ref.clone(),
        receipt.post_id.clone(),
        context.audit_correlation_id.clone(),
        Some(context.idempotency_key.clone()),
        receipt.policy_decision_ref.clone(),
    )
}

impl AuthorizedCommunityContext {
    pub fn validate(&self) -> Result<(), CommunityApiError> {
        non_empty(&self.tenant_scope_ref)?;
        non_empty(&self.principal_ref)?;
        if !self.tenant_scope_ref.starts_with("tenant:") {
            return Err(CommunityApiError::MissingTenantScope);
        }
        if self.idempotency_key.trim().is_empty() {
            return Err(CommunityApiError::MissingIdempotencyKey);
        }
        if self.policy_decision_ref.trim().is_empty() {
            return Err(CommunityApiError::MissingPolicyDecision);
        }
        if self.audit_correlation_id.trim().is_empty() {
            return Err(CommunityApiError::MissingAuditCorrelation);
        }
        Ok(())
    }
}

fn non_empty(value: &str) -> Result<(), CommunityApiError> {
    if value.trim().is_empty() {
        Err(CommunityApiError::Invalid)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> AuthorizedCommunityContext {
        AuthorizedCommunityContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    #[test]
    fn community_context_requires_tenant_policy_and_idempotency() {
        let ctx = AuthorizedCommunityContext {
            tenant_scope_ref: "person:u".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(ctx.validate(), Err(CommunityApiError::MissingTenantScope));

        let ctx = AuthorizedCommunityContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(
            ctx.validate(),
            Err(CommunityApiError::MissingIdempotencyKey)
        );

        let ctx = AuthorizedCommunityContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(
            ctx.validate(),
            Err(CommunityApiError::MissingPolicyDecision)
        );
    }

    #[test]
    fn post_created_binding_matches_asyncapi_and_proto_contracts() {
        let binding = post_created_protocol_binding().unwrap();

        assert_eq!(binding.rest_operation_id, "createPost");
        assert_eq!(binding.asyncapi_operation_id, "publishPostCreated");
        assert_eq!(binding.asyncapi_channel_address, "community.post.created");
        assert_eq!(binding.asyncapi_message_name, "PostCreated");
        assert_eq!(binding.asyncapi_event_kind, "oya.community.post.created.v1");
        assert_eq!(binding.receipt_event_type, "community.post.created");
        assert_eq!(binding.proto_package, "oya.community.v1");
        assert_eq!(binding.proto_service, "PostStoreService");
        assert_eq!(binding.proto_rpc, "CreatePost");
    }

    #[test]
    fn vote_and_moderation_bindings_match_asyncapi_and_proto_contracts() {
        let vote = vote_cast_protocol_binding().unwrap();
        assert_eq!(vote.rest_operation_id, "castVote");
        assert_eq!(vote.asyncapi_operation_id, "publishVoteCast");
        assert_eq!(vote.asyncapi_channel_address, "community.vote.cast");
        assert_eq!(vote.asyncapi_message_name, "VoteCast");
        assert_eq!(vote.asyncapi_event_kind, "oya.community.vote.cast.v1");
        assert_eq!(vote.receipt_event_type, "community.vote.cast");
        assert_eq!(vote.proto_service, "VotingEngineService");
        assert_eq!(vote.proto_rpc, "CastVote");

        let moderation = moderation_actioned_protocol_binding().unwrap();
        assert_eq!(moderation.rest_operation_id, "applyModerationAction");
        assert_eq!(
            moderation.asyncapi_operation_id,
            "publishModerationActioned"
        );
        assert_eq!(
            moderation.asyncapi_channel_address,
            "community.moderation.actioned"
        );
        assert_eq!(moderation.asyncapi_message_name, "ModerationActioned");
        assert_eq!(
            moderation.asyncapi_event_kind,
            "oya.community.moderation.actioned.v1"
        );
        assert_eq!(
            moderation.receipt_event_type,
            "community.moderation.actioned"
        );
        assert_eq!(moderation.proto_service, "ModerationQueueService");
        assert_eq!(moderation.proto_rpc, "ApplyAction");
    }

    #[test]
    fn protocol_envelopes_carry_policy_audit_and_idempotency_refs() {
        let post_receipt = PostReceipt {
            post_id: "post:p".into(),
            event_type: "community.post.created",
            audit_correlation_id: "audit".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
        };
        let post = post_created_event_envelope(&context(), &post_receipt).unwrap();
        assert_eq!(post.aggregate_id, "post:p");
        assert_eq!(post.tenant_scope_ref, "tenant:t");
        assert_eq!(post.idempotency_key, Some("idem".into()));
        assert_eq!(post.binding.asyncapi_operation_id, "publishPostCreated");

        let vote_receipt = VoteReceiptEnvelope {
            post_id: "post:p".into(),
            vote_id: "vote:v".into(),
            event_type: "community.vote.cast",
            policy_decision_ref: "policy".into(),
        };
        let vote = vote_cast_event_envelope(&context(), &vote_receipt).unwrap();
        assert_eq!(vote.aggregate_id, "post:p");
        assert_eq!(vote.audit_correlation_id, "audit");
        assert_eq!(vote.idempotency_key, Some("idem".into()));
        assert_eq!(vote.binding.proto_rpc, "CastVote");

        let moderation_receipt = ModerationReceipt {
            post_id: "post:p".into(),
            event_type: "community.moderation.actioned",
            evidence_ref: "evidence:e".into(),
            policy_decision_ref: "policy".into(),
        };
        let moderation =
            moderation_actioned_event_envelope(&context(), &moderation_receipt).unwrap();
        assert_eq!(moderation.aggregate_id, "post:p");
        assert_eq!(moderation.policy_decision_ref, "policy");
        assert_eq!(moderation.binding.proto_service, "ModerationQueueService");
    }
}
