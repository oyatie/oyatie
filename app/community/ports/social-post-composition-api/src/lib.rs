#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use shared_protocol_parity_kernel::{
    ProtocolEventEnvelope, ProtocolParityBinding, ProtocolParityBindingSpec, ProtocolParityError,
    require_receipt_event_type,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocialApiError {
    Invalid,
    MissingPersonalScope,
    MissingWorkScope,
    MissingPolicyDecision,
    MissingIdempotencyKey,
    MissingAuditCorrelation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocialApiContext {
    Personal,
    Work,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocialApiArtifactKind {
    FeedPost,
    Story,
    CollaborativePost,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedSocialContext {
    pub context: SocialApiContext,
    pub scope_ref: String,
    pub principal_ref: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
    pub audit_correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComposePostRequest {
    pub post_id: String,
    pub creator_ref: String,
    pub kind: SocialApiArtifactKind,
    pub media_refs: Vec<String>,
    pub story_expires_at: Option<u64>,
    pub collab_owner_refs: Vec<String>,
    pub collab_consent_refs: Vec<String>,
    pub workflow_consent_ref: Option<String>,
    pub ar_biometric_persisted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialPostReceipt {
    pub post_id: String,
    pub event_type: &'static str,
    pub audit_correlation_id: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoryPurgePlan {
    pub post_id: String,
    pub purge_targets: Vec<&'static str>,
}

pub const POST_PUBLISHED_PROTOCOL_SCHEMA_VERSION: &str = "1.0.0";

pub fn post_published_protocol_binding() -> Result<ProtocolParityBinding, ProtocolParityError> {
    ProtocolParityBinding::new(ProtocolParityBindingSpec {
        rest_operation_id: "publishPost",
        asyncapi_operation_id: "emitPostPublished",
        asyncapi_channel_address: "workflow-events/social.post.published",
        asyncapi_message_name: "PostPublished",
        asyncapi_event_kind: "oya.social.post.published.v1",
        receipt_event_type: "social.post.created",
        proto_package: "oya.social.v1",
        proto_service: "PostComposition",
        proto_rpc: "PublishPost",
    })
}

pub fn post_published_event_envelope(
    context: &AuthorizedSocialContext,
    receipt: &SocialPostReceipt,
) -> Result<ProtocolEventEnvelope, ProtocolParityError> {
    let binding = post_published_protocol_binding()?;
    require_receipt_event_type(&binding, receipt.event_type)?;
    ProtocolEventEnvelope::new(
        binding,
        POST_PUBLISHED_PROTOCOL_SCHEMA_VERSION,
        context.scope_ref.clone(),
        receipt.post_id.clone(),
        receipt.audit_correlation_id.clone(),
        Some(receipt.idempotency_key.clone()),
        receipt.policy_decision_ref.clone(),
    )
}

impl AuthorizedSocialContext {
    pub fn validate(&self) -> Result<(), SocialApiError> {
        non_empty(&self.scope_ref)?;
        non_empty(&self.principal_ref)?;
        match self.context {
            SocialApiContext::Personal if !self.scope_ref.starts_with("person:") => {
                return Err(SocialApiError::MissingPersonalScope);
            }
            SocialApiContext::Work if !self.scope_ref.starts_with("tenant:") => {
                return Err(SocialApiError::MissingWorkScope);
            }
            _ => {}
        }
        if self.idempotency_key.trim().is_empty() {
            return Err(SocialApiError::MissingIdempotencyKey);
        }
        if self.policy_decision_ref.trim().is_empty() {
            return Err(SocialApiError::MissingPolicyDecision);
        }
        if self.audit_correlation_id.trim().is_empty() {
            return Err(SocialApiError::MissingAuditCorrelation);
        }
        Ok(())
    }
}

fn non_empty(value: &str) -> Result<(), SocialApiError> {
    if value.trim().is_empty() {
        Err(SocialApiError::Invalid)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_context_requires_work_scope_and_audit_correlation() {
        let ctx = AuthorizedSocialContext {
            context: SocialApiContext::Work,
            scope_ref: "person:u".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(ctx.validate(), Err(SocialApiError::MissingWorkScope));

        let ctx = AuthorizedSocialContext {
            context: SocialApiContext::Personal,
            scope_ref: "person:u".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "".into(),
        };
        assert_eq!(ctx.validate(), Err(SocialApiError::MissingAuditCorrelation));
    }

    #[test]
    fn personal_context_requires_person_scope_and_policy_decision() {
        let ctx = AuthorizedSocialContext {
            context: SocialApiContext::Personal,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(ctx.validate(), Err(SocialApiError::MissingPersonalScope));

        let ctx = AuthorizedSocialContext {
            context: SocialApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "".into(),
            audit_correlation_id: "audit".into(),
        };
        assert_eq!(ctx.validate(), Err(SocialApiError::MissingPolicyDecision));
    }

    #[test]
    fn post_published_protocol_binding_matches_asyncapi_and_proto_contracts() {
        let binding = post_published_protocol_binding().unwrap();

        assert_eq!(binding.rest_operation_id, "publishPost");
        assert_eq!(binding.asyncapi_operation_id, "emitPostPublished");
        assert_eq!(
            binding.asyncapi_channel_address,
            "workflow-events/social.post.published"
        );
        assert_eq!(binding.asyncapi_message_name, "PostPublished");
        assert_eq!(binding.asyncapi_event_kind, "oya.social.post.published.v1");
        assert_eq!(binding.receipt_event_type, "social.post.created");
        assert_eq!(binding.proto_package, "oya.social.v1");
        assert_eq!(binding.proto_service, "PostComposition");
        assert_eq!(binding.proto_rpc, "PublishPost");
    }

    #[test]
    fn post_published_event_envelope_carries_policy_audit_and_idempotency_refs() {
        let context = AuthorizedSocialContext {
            context: SocialApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        };
        let receipt = SocialPostReceipt {
            post_id: "post:p".into(),
            event_type: "social.post.created",
            audit_correlation_id: "audit".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
        };

        let envelope = post_published_event_envelope(&context, &receipt).unwrap();

        assert_eq!(envelope.aggregate_id, "post:p");
        assert_eq!(envelope.tenant_scope_ref, "tenant:t");
        assert_eq!(
            envelope.binding.asyncapi_event_kind,
            "oya.social.post.published.v1"
        );
    }
}
