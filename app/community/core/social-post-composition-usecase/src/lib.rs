#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod feed_ranking;

use community_social_domain::{
    OwnershipPillar as SocialOwnershipPillar, PurgeTarget, SocialArtifactKind, SocialContextKind,
    SocialError, SocialPost, SocialPostCreate, story_purge,
};
use community_social_post_composition_api::{
    AuthorizedSocialContext, ComposePostRequest, SocialApiArtifactKind, SocialApiContext,
    SocialApiError, SocialPostReceipt, StoryPurgePlan,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocialUsecaseError {
    Api(SocialApiError),
    Domain(SocialError),
    PrincipalMismatch,
}

pub fn compose_post(
    ctx: &AuthorizedSocialContext,
    req: ComposePostRequest,
) -> Result<(SocialPost, SocialPostReceipt), SocialUsecaseError> {
    ctx.validate().map_err(SocialUsecaseError::Api)?;
    if req.creator_ref != ctx.principal_ref {
        return Err(SocialUsecaseError::PrincipalMismatch);
    }
    let post = SocialPost::new(SocialPostCreate {
        post_id: req.post_id,
        creator_ref: req.creator_ref,
        scope_ref: ctx.scope_ref.clone(),
        context: map_context(ctx.context),
        pillar: map_pillar(ctx.context),
        kind: map_kind(req.kind),
        media_refs: req.media_refs,
        story_expires_at: req.story_expires_at,
        collab_owner_refs: req.collab_owner_refs,
        collab_consent_refs: req.collab_consent_refs,
        workflow_consent_ref: req.workflow_consent_ref,
        ar_biometric_persisted: req.ar_biometric_persisted,
    })
    .map_err(SocialUsecaseError::Domain)?;
    let receipt = SocialPostReceipt {
        post_id: post.post_id.value.clone(),
        event_type: "social.post.created",
        audit_correlation_id: ctx.audit_correlation_id.clone(),
        idempotency_key: ctx.idempotency_key.clone(),
        policy_decision_ref: ctx.policy_decision_ref.clone(),
    };
    Ok((post, receipt))
}

pub fn plan_story_purge(post: &SocialPost, now: u64) -> Result<StoryPurgePlan, SocialUsecaseError> {
    let purge_targets = story_purge(post, now)
        .map_err(SocialUsecaseError::Domain)?
        .into_iter()
        .map(target_name)
        .collect();
    Ok(StoryPurgePlan {
        post_id: post.post_id.value.clone(),
        purge_targets,
    })
}

fn map_context(context: SocialApiContext) -> SocialContextKind {
    match context {
        SocialApiContext::Personal => SocialContextKind::Personal,
        SocialApiContext::Work => SocialContextKind::Work,
    }
}

fn map_pillar(context: SocialApiContext) -> SocialOwnershipPillar {
    match context {
        SocialApiContext::Personal => SocialOwnershipPillar::Personal,
        SocialApiContext::Work => SocialOwnershipPillar::Work,
    }
}

fn map_kind(kind: SocialApiArtifactKind) -> SocialArtifactKind {
    match kind {
        SocialApiArtifactKind::FeedPost => SocialArtifactKind::FeedPost,
        SocialApiArtifactKind::Story => SocialArtifactKind::Story,
        SocialApiArtifactKind::CollaborativePost => SocialArtifactKind::CollaborativePost,
    }
}

fn target_name(target: PurgeTarget) -> &'static str {
    match target {
        PurgeTarget::CdnObject => "cdn_object",
        PurgeTarget::SearchIndex => "search_index",
        PurgeTarget::OntologyNode => "ontology_node",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn personal_ctx() -> AuthorizedSocialContext {
        AuthorizedSocialContext {
            context: SocialApiContext::Personal,
            scope_ref: "person:u".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow:post-compose".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    #[test]
    fn compose_post_binds_principal_to_creator() {
        let req = ComposePostRequest {
            post_id: "p".into(),
            creator_ref: "user:other".into(),
            kind: SocialApiArtifactKind::FeedPost,
            media_refs: vec!["m".into()],
            story_expires_at: None,
            collab_owner_refs: vec![],
            collab_consent_refs: vec![],
            workflow_consent_ref: None,
            ar_biometric_persisted: false,
        };
        assert_eq!(
            compose_post(&personal_ctx(), req),
            Err(SocialUsecaseError::PrincipalMismatch)
        );
    }

    #[test]
    fn work_crosspost_requires_workflow_consent() {
        let ctx = AuthorizedSocialContext {
            context: SocialApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow:post-compose".into(),
            audit_correlation_id: "audit".into(),
        };
        let req = ComposePostRequest {
            post_id: "p".into(),
            creator_ref: "user:u".into(),
            kind: SocialApiArtifactKind::FeedPost,
            media_refs: vec!["m".into()],
            story_expires_at: None,
            collab_owner_refs: vec![],
            collab_consent_refs: vec![],
            workflow_consent_ref: None,
            ar_biometric_persisted: false,
        };
        assert_eq!(
            compose_post(&ctx, req),
            Err(SocialUsecaseError::Domain(
                SocialError::WorkCrosspostRequiresConsent
            ))
        );
    }

    #[test]
    fn story_purge_plan_names_all_external_targets() {
        let req = ComposePostRequest {
            post_id: "story".into(),
            creator_ref: "user:u".into(),
            kind: SocialApiArtifactKind::Story,
            media_refs: vec!["m".into()],
            story_expires_at: Some(10),
            collab_owner_refs: vec![],
            collab_consent_refs: vec![],
            workflow_consent_ref: None,
            ar_biometric_persisted: false,
        };
        let (post, _) = compose_post(&personal_ctx(), req).unwrap();
        let plan = plan_story_purge(&post, 11).unwrap();
        assert_eq!(
            plan.purge_targets,
            vec!["cdn_object", "search_index", "ontology_node"]
        );
    }
}
