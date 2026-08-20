#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod moderation_queue;

use community_post_store_api::{
    AuthorizedCommunityContext, CastVoteRequest, CommunityApiError, CommunityApiMode,
    CreatePostRequest, ModeratePostRequest, ModerationReceipt, ModerationVerb, PostReceipt,
    VoteDirection, VoteReceiptEnvelope,
};
use community_post_store_domain::{
    CommunityAuthor, CommunityError, CommunityMode, CommunityPost, ModerationAction, VoteKind,
    VoteLedger, VoteReceipt, moderation_case,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunityUsecaseError {
    Api(CommunityApiError),
    Domain(CommunityError),
    TenantMismatch,
    PrincipalMismatch,
    PostMismatch,
    VoteClearRequiresAdapterState,
}

pub fn create_post(
    ctx: &AuthorizedCommunityContext,
    req: CreatePostRequest,
) -> Result<(CommunityPost, PostReceipt), CommunityUsecaseError> {
    ctx.validate().map_err(CommunityUsecaseError::Api)?;
    if req.audit_author_ref != ctx.principal_ref {
        return Err(CommunityUsecaseError::PrincipalMismatch);
    }
    let author = CommunityAuthor::new(
        req.routine_display_ref,
        req.audit_author_ref,
        req.disclosure_policy_ref,
    )
    .map_err(CommunityUsecaseError::Domain)?;
    let post = CommunityPost::new(
        req.post_id,
        req.thread_id,
        ctx.tenant_scope_ref.clone(),
        map_mode(req.mode),
        author,
        req.body_ref,
        req.retention_policy_id,
    )
    .map_err(CommunityUsecaseError::Domain)?;
    let receipt = PostReceipt {
        post_id: post.post_id.value.clone(),
        event_type: "community.post.created",
        audit_correlation_id: ctx.audit_correlation_id.clone(),
        idempotency_key: ctx.idempotency_key.clone(),
        policy_decision_ref: ctx.policy_decision_ref.clone(),
    };
    Ok((post, receipt))
}

pub fn cast_vote(
    ctx: &AuthorizedCommunityContext,
    post: &CommunityPost,
    ledger: &mut VoteLedger,
    req: CastVoteRequest,
) -> Result<VoteReceiptEnvelope, CommunityUsecaseError> {
    ctx.validate().map_err(CommunityUsecaseError::Api)?;
    if post.tenant_scope_ref.value != ctx.tenant_scope_ref {
        return Err(CommunityUsecaseError::TenantMismatch);
    }
    if req.post_id != post.post_id.value {
        return Err(CommunityUsecaseError::PostMismatch);
    }
    if req.voter_ref != ctx.principal_ref {
        return Err(CommunityUsecaseError::PrincipalMismatch);
    }
    let kind = match req.direction {
        VoteDirection::Up => VoteKind::Up,
        VoteDirection::Down => VoteKind::Down,
        VoteDirection::Clear => return Err(CommunityUsecaseError::VoteClearRequiresAdapterState),
    };
    let receipt = VoteReceipt {
        vote_id: ctx.idempotency_key.clone(),
        voter_ref: req.voter_ref,
        post_id: req.post_id,
        kind,
    };
    ledger
        .cast(receipt.clone(), post)
        .map_err(CommunityUsecaseError::Domain)?;
    Ok(VoteReceiptEnvelope {
        post_id: receipt.post_id,
        vote_id: receipt.vote_id,
        event_type: "community.vote.cast",
        policy_decision_ref: ctx.policy_decision_ref.clone(),
    })
}

pub fn moderate_post(
    ctx: &AuthorizedCommunityContext,
    post: &CommunityPost,
    req: ModeratePostRequest,
) -> Result<ModerationReceipt, CommunityUsecaseError> {
    ctx.validate().map_err(CommunityUsecaseError::Api)?;
    if post.tenant_scope_ref.value != ctx.tenant_scope_ref {
        return Err(CommunityUsecaseError::TenantMismatch);
    }
    let evidence = moderation_case(
        post,
        map_moderation(req.verb),
        req.policy_ref,
        req.evidence_ref,
    )
    .map_err(CommunityUsecaseError::Domain)?;
    Ok(ModerationReceipt {
        post_id: post.post_id.value.clone(),
        event_type: "community.moderation.actioned",
        evidence_ref: evidence.value,
        policy_decision_ref: ctx.policy_decision_ref.clone(),
    })
}

fn map_mode(mode: CommunityApiMode) -> CommunityMode {
    match mode {
        CommunityApiMode::Reddit => CommunityMode::Reddit,
        CommunityApiMode::Teamblind => CommunityMode::Teamblind,
        CommunityApiMode::Handshake => CommunityMode::Handshake,
        CommunityApiMode::KnowledgeBase => CommunityMode::KnowledgeBase,
    }
}

fn map_moderation(verb: ModerationVerb) -> ModerationAction {
    match verb {
        ModerationVerb::Allow => ModerationAction::Allow,
        ModerationVerb::Hide => ModerationAction::Hide,
        ModerationVerb::Remove => ModerationAction::Remove,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> AuthorizedCommunityContext {
        AuthorizedCommunityContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    fn req() -> CreatePostRequest {
        CreatePostRequest {
            post_id: "p".into(),
            thread_id: "th".into(),
            mode: CommunityApiMode::Teamblind,
            routine_display_ref: "anon".into(),
            audit_author_ref: "user:u".into(),
            disclosure_policy_ref: Some("disclosure".into()),
            body_ref: "body".into(),
            retention_policy_id: "retain".into(),
        }
    }

    #[test]
    fn create_post_requires_teamblind_disclosure_policy() {
        let mut req = req();
        req.disclosure_policy_ref = None;
        assert_eq!(
            create_post(&ctx(), req),
            Err(CommunityUsecaseError::Domain(
                CommunityError::MissingAnonymousDisclosurePolicy
            ))
        );
    }

    #[test]
    fn vote_cast_uses_idempotency_key_and_rejects_duplicates() {
        let mut vote_ctx = ctx();
        vote_ctx.principal_ref = "user:voter".into();
        let (post, _) = create_post(&ctx(), req()).unwrap();
        let mut ledger = VoteLedger::new(&post);
        let vote = CastVoteRequest {
            post_id: "p".into(),
            voter_ref: "user:voter".into(),
            direction: VoteDirection::Up,
        };
        let receipt = cast_vote(&vote_ctx, &post, &mut ledger, vote.clone()).unwrap();
        assert_eq!(receipt.vote_id, "idem");
        assert_eq!(
            cast_vote(&vote_ctx, &post, &mut ledger, vote),
            Err(CommunityUsecaseError::Domain(CommunityError::DuplicateVote))
        );
    }

    #[test]
    fn vote_cast_binds_voter_to_principal() {
        let (post, _) = create_post(&ctx(), req()).unwrap();
        let mut ledger = VoteLedger::new(&post);
        assert_eq!(
            cast_vote(
                &ctx(),
                &post,
                &mut ledger,
                CastVoteRequest {
                    post_id: "p".into(),
                    voter_ref: "user:other".into(),
                    direction: VoteDirection::Up,
                }
            ),
            Err(CommunityUsecaseError::PrincipalMismatch)
        );
    }

    #[test]
    fn moderation_requires_audit_evidence() {
        let (post, _) = create_post(&ctx(), req()).unwrap();
        assert_eq!(
            moderate_post(
                &ctx(),
                &post,
                ModeratePostRequest {
                    policy_ref: "policy".into(),
                    evidence_ref: "".into(),
                    verb: ModerationVerb::Hide,
                }
            ),
            Err(CommunityUsecaseError::Domain(
                CommunityError::ModerationNeedsEvidence
            ))
        );
    }
}
