#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocialError {
    Invalid,
    ContextPillarMismatch,
    StoryRequiresTtl,
    StoryNotExpired,
    CrossContextArtifactRef,
    WorkCrosspostRequiresConsent,
    ArBiometricPersistenceForbidden,
    CollabRequiresConsent,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocialContextKind {
    Personal,
    Work,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnershipPillar {
    Personal,
    Work,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocialArtifactKind {
    FeedPost,
    Story,
    CollaborativePost,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PurgeTarget {
    CdnObject,
    SearchIndex,
    OntologyNode,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialPostCreate {
    pub post_id: String,
    pub creator_ref: String,
    pub scope_ref: String,
    pub context: SocialContextKind,
    pub pillar: OwnershipPillar,
    pub kind: SocialArtifactKind,
    pub media_refs: Vec<String>,
    pub story_expires_at: Option<u64>,
    pub collab_owner_refs: Vec<String>,
    pub collab_consent_refs: Vec<String>,
    pub workflow_consent_ref: Option<String>,
    pub ar_biometric_persisted: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialPost {
    pub post_id: Classified<String>,
    pub creator_ref: Classified<String>,
    pub scope_ref: Classified<String>,
    pub context: Classified<SocialContextKind>,
    pub pillar: Classified<OwnershipPillar>,
    pub kind: Classified<SocialArtifactKind>,
    pub media_refs: Classified<Vec<String>>,
    pub story_expires_at: Classified<Option<u64>>,
    pub collab_consent_refs: Classified<Vec<String>>,
    pub workflow_consent_ref: Classified<Option<String>>,
}
impl SocialPost {
    pub fn new(i: SocialPostCreate) -> Result<Self, SocialError> {
        ne(&i.post_id)?;
        ne(&i.creator_ref)?;
        ne(&i.scope_ref)?;
        if i.media_refs.is_empty() {
            return Err(SocialError::Invalid);
        };
        match (i.context, i.pillar) {
            (SocialContextKind::Personal, OwnershipPillar::Personal)
            | (SocialContextKind::Work, OwnershipPillar::Work) => (),
            _ => return Err(SocialError::ContextPillarMismatch),
        };
        if i.kind == SocialArtifactKind::Story && i.story_expires_at.is_none() {
            return Err(SocialError::StoryRequiresTtl);
        };
        if i.context == SocialContextKind::Work
            && i.workflow_consent_ref
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
        {
            return Err(SocialError::WorkCrosspostRequiresConsent);
        };
        if i.ar_biometric_persisted {
            return Err(SocialError::ArBiometricPersistenceForbidden);
        };
        if !i.collab_owner_refs.is_empty()
            && i.collab_consent_refs.len() < i.collab_owner_refs.len()
        {
            return Err(SocialError::CollabRequiresConsent);
        };
        Ok(Self {
            post_id: int(i.post_id),
            creator_ref: Classified::new(i.creator_ref, PrivacyDataClass::pii_identifying()),
            scope_ref: int(i.scope_ref),
            context: int(i.context),
            pillar: int(i.pillar),
            kind: int(i.kind),
            media_refs: Classified::new(i.media_refs, PrivacyDataClass::pii_identifying()),
            story_expires_at: int(i.story_expires_at),
            collab_consent_refs: int(i.collab_consent_refs),
            workflow_consent_ref: int(i.workflow_consent_ref),
        })
    }
}
pub fn context_snapshot(
    context: SocialContextKind,
    pillar: OwnershipPillar,
    posts: &[SocialPost],
) -> Result<Vec<String>, SocialError> {
    if posts
        .iter()
        .any(|p| p.context.value != context || p.pillar.value != pillar)
    {
        return Err(SocialError::CrossContextArtifactRef);
    };
    Ok(posts.iter().map(|p| p.post_id.value.clone()).collect())
}
pub fn story_purge(post: &SocialPost, now: u64) -> Result<BTreeSet<PurgeTarget>, SocialError> {
    let Some(exp) = post.story_expires_at.value else {
        return Err(SocialError::StoryRequiresTtl);
    };
    if now < exp {
        return Err(SocialError::StoryNotExpired);
    };
    Ok(BTreeSet::from([
        PurgeTarget::CdnObject,
        PurgeTarget::SearchIndex,
        PurgeTarget::OntologyNode,
    ]))
}
fn ne(s: &str) -> Result<(), SocialError> {
    if s.trim().is_empty() {
        Err(SocialError::Invalid)
    } else {
        Ok(())
    }
}
fn int<T>(v: T) -> Classified<T> {
    Classified::new(v, DataClass::InternalOnly)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn personal(k: SocialArtifactKind) -> SocialPost {
        SocialPost::new(SocialPostCreate {
            post_id: "p1".into(),
            creator_ref: "u".into(),
            scope_ref: "person:u".into(),
            context: SocialContextKind::Personal,
            pillar: OwnershipPillar::Personal,
            kind: k,
            media_refs: vec!["m".into()],
            story_expires_at: (k == SocialArtifactKind::Story).then_some(10),
            collab_owner_refs: vec![],
            collab_consent_refs: vec![],
            workflow_consent_ref: None,
            ar_biometric_persisted: false,
        })
        .unwrap()
    }

    // ── sweep test helpers ────────────────────────────────────────────────────

    /// Build a Personal/Personal Story with a specific expiry and post_id.
    fn personal_story(post_id: &str, expires_at: u64) -> SocialPost {
        SocialPost::new(SocialPostCreate {
            post_id: post_id.into(),
            creator_ref: "u".into(),
            scope_ref: "person:u".into(),
            context: SocialContextKind::Personal,
            pillar: OwnershipPillar::Personal,
            kind: SocialArtifactKind::Story,
            media_refs: vec!["m".into()],
            story_expires_at: Some(expires_at),
            collab_owner_refs: vec![],
            collab_consent_refs: vec![],
            workflow_consent_ref: None,
            ar_biometric_persisted: false,
        })
        .unwrap()
    }

    /// Build a Work/Work Story with a specific expiry and post_id.
    fn work_story(post_id: &str, expires_at: u64) -> SocialPost {
        SocialPost::new(SocialPostCreate {
            post_id: post_id.into(),
            creator_ref: "u".into(),
            scope_ref: "tenant:t".into(),
            context: SocialContextKind::Work,
            pillar: OwnershipPillar::Work,
            kind: SocialArtifactKind::Story,
            media_refs: vec!["m".into()],
            story_expires_at: Some(expires_at),
            collab_owner_refs: vec![],
            collab_consent_refs: vec![],
            workflow_consent_ref: Some("consent-ref".into()),
            ar_biometric_persisted: false,
        })
        .unwrap()
    }

    /// Build a Personal/Personal FeedPost.
    fn personal_feed(post_id: &str) -> SocialPost {
        SocialPost::new(SocialPostCreate {
            post_id: post_id.into(),
            creator_ref: "u".into(),
            scope_ref: "person:u".into(),
            context: SocialContextKind::Personal,
            pillar: OwnershipPillar::Personal,
            kind: SocialArtifactKind::FeedPost,
            media_refs: vec!["m".into()],
            story_expires_at: None,
            collab_owner_refs: vec![],
            collab_consent_refs: vec![],
            workflow_consent_ref: None,
            ar_biometric_persisted: false,
        })
        .unwrap()
    }

    // ── [scs-1] tests ─────────────────────────────────────────────────────────

    /// A post whose context does not match the sweep scope must cause the entire
    /// batch to be rejected with CrossContextArtifactRef.
    #[test]
    fn sweep_mismatched_context_yields_cross_context_artifact_ref() {
        // work_story is Work/Work; sweeping with Personal context → mismatch
        let mismatched = work_story("w1", 5);
        let result = story_sweep(
            SocialContextKind::Personal,
            OwnershipPillar::Personal,
            &[mismatched],
            100,
        );
        assert_eq!(result, Err(SocialError::CrossContextArtifactRef));
    }

    /// Two stories with the same context/pillar, one expired and one not —
    /// only the expired story contributes purge targets.
    #[test]
    fn sweep_mixed_expiry_returns_only_expired_purge_targets() {
        let expired = personal_story("s-expired", 10); // expires_at=10
        let fresh = personal_story("s-fresh", 200); // expires_at=200
        let now = 100; // expired < now; fresh > now

        let result = story_sweep(
            SocialContextKind::Personal,
            OwnershipPillar::Personal,
            &[expired, fresh],
            now,
        )
        .expect("should not error");

        // Must contain all three purge targets from the expired story
        assert!(result.contains(&PurgeTarget::CdnObject));
        assert!(result.contains(&PurgeTarget::SearchIndex));
        assert!(result.contains(&PurgeTarget::OntologyNode));
        // Result came only from expired post; set size is 3 (no duplicates from fresh)
        assert_eq!(result.len(), 3);
    }

    // ── [scs-2] tests ─────────────────────────────────────────────────────────

    /// A heterogeneous batch containing a FeedPost and an expired Story must
    /// return purge targets only for the Story; no StoryRequiresTtl error.
    #[test]
    fn sweep_heterogeneous_batch_skips_non_story() {
        let feed = personal_feed("f1");
        let story = personal_story("s1", 10); // expires_at=10
        let now = 50; // story is expired

        let result = story_sweep(
            SocialContextKind::Personal,
            OwnershipPillar::Personal,
            &[feed, story],
            now,
        );

        // Must succeed — no StoryRequiresTtl raised for the FeedPost
        assert!(
            result.is_ok(),
            "expected Ok but got {:?}",
            result.unwrap_err()
        );
        let targets = result.unwrap();
        // Exactly three purge targets from the one expired Story
        assert_eq!(targets.len(), 3);
        assert!(targets.contains(&PurgeTarget::CdnObject));
        assert!(targets.contains(&PurgeTarget::SearchIndex));
        assert!(targets.contains(&PurgeTarget::OntologyNode));
    }

    // ── [scs-3] tests ─────────────────────────────────────────────────────────

    /// An empty post slice must return Ok(empty set) immediately.
    #[test]
    fn sweep_empty_slice_returns_empty_set() {
        let result = story_sweep(
            SocialContextKind::Personal,
            OwnershipPillar::Personal,
            &[],
            100,
        );
        assert_eq!(result, Ok(BTreeSet::new()));
    }

    /// A batch where every story is unexpired must return Ok(empty set).
    #[test]
    fn sweep_all_unexpired_returns_empty_set() {
        let s1 = personal_story("s1", 500);
        let s2 = personal_story("s2", 1000);
        let now = 100; // both stories still live

        let result = story_sweep(
            SocialContextKind::Personal,
            OwnershipPillar::Personal,
            &[s1, s2],
            now,
        );
        assert_eq!(result, Ok(BTreeSet::new()));
    }
    #[test]
    fn test_personal_post_pillar_immutable() {
        let p = personal(SocialArtifactKind::FeedPost);
        assert_eq!(p.context.value, SocialContextKind::Personal)
    }
    #[test]
    fn test_context_switch_reconstructs_all_surfaces() {
        let p = personal(SocialArtifactKind::FeedPost);
        assert_eq!(
            context_snapshot(SocialContextKind::Personal, OwnershipPillar::Personal, &[p]).unwrap(),
            vec!["p1".to_string()]
        )
    }
    #[test]
    fn test_ephemeral_story_purge_within_ttl() {
        assert!(
            story_purge(&personal(SocialArtifactKind::Story), 11)
                .unwrap()
                .contains(&PurgeTarget::OntologyNode)
        )
    }
    #[test]
    fn test_workflow_crosspost_requires_consent_and_stays_in_work_context() {
        assert_eq!(
            SocialPost::new(SocialPostCreate {
                post_id: "w".into(),
                creator_ref: "u".into(),
                scope_ref: "tenant:t".into(),
                context: SocialContextKind::Work,
                pillar: OwnershipPillar::Work,
                kind: SocialArtifactKind::FeedPost,
                media_refs: vec!["m".into()],
                story_expires_at: None,
                collab_owner_refs: vec![],
                collab_consent_refs: vec![],
                workflow_consent_ref: None,
                ar_biometric_persisted: false
            }),
            Err(SocialError::WorkCrosspostRequiresConsent)
        )
    }
    #[test]
    fn test_ar_no_biometric_persistence() {
        assert_eq!(
            SocialPost::new(SocialPostCreate {
                post_id: "a".into(),
                creator_ref: "u".into(),
                scope_ref: "person:u".into(),
                context: SocialContextKind::Personal,
                pillar: OwnershipPillar::Personal,
                kind: SocialArtifactKind::FeedPost,
                media_refs: vec!["m".into()],
                story_expires_at: None,
                collab_owner_refs: vec![],
                collab_consent_refs: vec![],
                workflow_consent_ref: None,
                ar_biometric_persisted: true
            }),
            Err(SocialError::ArBiometricPersistenceForbidden)
        )
    }
    #[test]
    fn test_collab_post_ownership_and_consent() {
        assert!(
            SocialPost::new(SocialPostCreate {
                post_id: "c".into(),
                creator_ref: "u".into(),
                scope_ref: "person:u".into(),
                context: SocialContextKind::Personal,
                pillar: OwnershipPillar::Personal,
                kind: SocialArtifactKind::CollaborativePost,
                media_refs: vec!["m".into()],
                story_expires_at: None,
                collab_owner_refs: vec!["u".into(), "v".into()],
                collab_consent_refs: vec!["cu".into(), "cv".into()],
                workflow_consent_ref: None,
                ar_biometric_persisted: false
            })
            .is_ok()
        )
    }
}
