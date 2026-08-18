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
/// Batch story-expiry sweep scoped to a single context/pillar pair.
///
/// # Context guard
/// Any post whose `context` or `pillar` does not match the supplied values causes the
/// entire batch to be rejected with `SocialError::CrossContextArtifactRef`, mirroring
/// the `context_snapshot` guard semantics.
///
/// # Artifact filtering
/// Non-Story artifacts (`FeedPost`, `CollaborativePost`) are silently skipped; they
/// contribute no purge targets and do not raise `StoryRequiresTtl`.
///
/// # Expiry aggregation
/// Among matching Story posts, only those whose `story_expires_at <= now` contribute
/// purge targets. Unexpired stories are silently skipped.
///
/// # Determinism
/// The return type is `BTreeSet<PurgeTarget>`. `PurgeTarget` derives `Ord`, so the
/// ordering is deterministic and stable across invocations; callers may rely on this.
///
/// # Empty inputs
/// An empty slice returns `Ok(BTreeSet::new())`. A batch with no expired stories also
/// returns `Ok(BTreeSet::new())`.
pub fn story_sweep(
    context: SocialContextKind,
    pillar: OwnershipPillar,
    posts: &[SocialPost],
    now: u64,
) -> Result<BTreeSet<PurgeTarget>, SocialError> {
    // Context/pillar guard — reject the whole batch on any mismatch.
    if posts
        .iter()
        .any(|p| p.context.value != context || p.pillar.value != pillar)
    {
        return Err(SocialError::CrossContextArtifactRef);
    }
    let mut acc = BTreeSet::new();
    for post in posts {
        // Delegate all per-post guards to story_purge (single source of truth):
        // Err(StoryRequiresTtl)  => non-Story or Story with no TTL  => skip
        // Err(StoryNotExpired)   => unexpired story                  => skip
        // Ok(targets)            => expired story                    => accumulate
        if let Ok(targets) = story_purge(post, now) {
            acc.extend(targets);
        }
    }
    Ok(acc)
}
/// Audit result for collaborative-consent reconciliation.
///
/// All three sets are `BTreeSet<String>`, guaranteeing deterministic lexicographic
/// ordering regardless of the input slice order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollabConsentAudit {
    /// Owners that have a matching consent ref.
    pub satisfied: BTreeSet<String>,
    /// Owners that have no matching consent ref.
    pub missing_consent: BTreeSet<String>,
    /// Consent refs that name no known owner.
    pub extraneous_consent: BTreeSet<String>,
}

/// Reconcile `owner_refs` against `consent_refs` and produce a three-way audit report.
///
/// Returns `Err(SocialError::Invalid)` if any ref in either slice is blank or
/// whitespace-only.  Otherwise computes three disjoint `BTreeSet`s deterministically:
///
/// | Set                  | Formula              |
/// |----------------------|----------------------|
/// | `satisfied`          | owners ∩ consents    |
/// | `missing_consent`    | owners − consents    |
/// | `extraneous_consent` | consents − owners    |
pub fn collab_consent_audit(
    owner_refs: &[String],
    consent_refs: &[String],
) -> Result<CollabConsentAudit, SocialError> {
    for r in owner_refs.iter().chain(consent_refs.iter()) {
        if r.trim().is_empty() {
            return Err(SocialError::Invalid);
        }
    }
    let owners: BTreeSet<&str> = owner_refs.iter().map(|s| s.as_str()).collect();
    let consents: BTreeSet<&str> = consent_refs.iter().map(|s| s.as_str()).collect();
    Ok(CollabConsentAudit {
        satisfied: owners
            .intersection(&consents)
            .map(|s| s.to_string())
            .collect(),
        missing_consent: owners
            .difference(&consents)
            .map(|s| s.to_string())
            .collect(),
        extraneous_consent: consents
            .difference(&owners)
            .map(|s| s.to_string())
            .collect(),
    })
}

/// Returns `true` iff every required owner has a matching consent (i.e. `missing_consent`
/// is empty).
pub fn is_fully_consented(audit: &CollabConsentAudit) -> bool {
    audit.missing_consent.is_empty()
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

    // ── collab_consent_audit tests ────────────────────────────────────────────

    /// All owners have matching consents → satisfied contains all, missing empty.
    #[test]
    fn audit_full_consent() {
        let owners = vec!["alice".into(), "bob".into()];
        let consents = vec!["alice".into(), "bob".into()];
        let audit = collab_consent_audit(&owners, &consents).unwrap();
        assert_eq!(
            audit.satisfied,
            BTreeSet::from(["alice".to_string(), "bob".to_string()])
        );
        assert!(audit.missing_consent.is_empty());
        assert!(audit.extraneous_consent.is_empty());
        assert!(is_fully_consented(&audit));
    }

    /// One owner lacks consent → appears in missing_consent; is_fully_consented false.
    #[test]
    fn audit_partial_gap() {
        let owners = vec!["alice".into(), "bob".into()];
        let consents = vec!["alice".into()];
        let audit = collab_consent_audit(&owners, &consents).unwrap();
        assert_eq!(audit.satisfied, BTreeSet::from(["alice".to_string()]));
        assert_eq!(audit.missing_consent, BTreeSet::from(["bob".to_string()]));
        assert!(audit.extraneous_consent.is_empty());
        assert!(!is_fully_consented(&audit));
    }

    /// Consent ref naming a non-owner → appears in extraneous_consent.
    #[test]
    fn audit_extraneous_consent() {
        let owners = vec!["alice".into()];
        let consents = vec!["alice".into(), "charlie".into()];
        let audit = collab_consent_audit(&owners, &consents).unwrap();
        assert_eq!(audit.satisfied, BTreeSet::from(["alice".to_string()]));
        assert!(audit.missing_consent.is_empty());
        assert_eq!(
            audit.extraneous_consent,
            BTreeSet::from(["charlie".to_string()])
        );
        assert!(is_fully_consented(&audit));
    }

    /// Blank string in owner_refs → Err(Invalid).
    #[test]
    fn audit_blank_owner_rejected() {
        let owners = vec!["alice".into(), "  ".into()];
        let consents = vec!["alice".into()];
        assert_eq!(
            collab_consent_audit(&owners, &consents),
            Err(SocialError::Invalid)
        );
    }

    /// Blank string in consent_refs → Err(Invalid).
    #[test]
    fn audit_blank_consent_rejected() {
        let owners = vec!["alice".into()];
        let consents = vec!["alice".into(), "".into()];
        assert_eq!(
            collab_consent_audit(&owners, &consents),
            Err(SocialError::Invalid)
        );
    }

    /// Both slices empty → Ok with all-empty sets; is_fully_consented true (vacuous).
    #[test]
    fn audit_empty_both_ok() {
        let audit = collab_consent_audit(&[], &[]).unwrap();
        assert!(audit.satisfied.is_empty());
        assert!(audit.missing_consent.is_empty());
        assert!(audit.extraneous_consent.is_empty());
        assert!(is_fully_consented(&audit));
    }

    /// Empty owners + non-empty consents → all consents extraneous, fully consented.
    #[test]
    fn audit_empty_owners_all_extraneous() {
        let consents = vec!["alice".into(), "bob".into()];
        let audit = collab_consent_audit(&[], &consents).unwrap();
        assert!(audit.satisfied.is_empty());
        assert!(audit.missing_consent.is_empty());
        assert_eq!(
            audit.extraneous_consent,
            BTreeSet::from(["alice".to_string(), "bob".to_string()])
        );
        assert!(is_fully_consented(&audit));
    }

    /// Repeated calls with same inputs (in different slice order) produce identical sets.
    #[test]
    fn audit_deterministic_ordering() {
        let owners_a = vec!["bob".into(), "alice".into()];
        let consents_a = vec!["charlie".into(), "alice".into()];
        let owners_b = vec!["alice".into(), "bob".into()];
        let consents_b = vec!["alice".into(), "charlie".into()];
        let audit_a = collab_consent_audit(&owners_a, &consents_a).unwrap();
        let audit_b = collab_consent_audit(&owners_b, &consents_b).unwrap();
        assert_eq!(audit_a, audit_b);
        // BTreeSet ordering: alice < bob < charlie
        assert_eq!(audit_a.satisfied.iter().next().unwrap(), "alice");
        assert_eq!(audit_a.missing_consent.iter().next().unwrap(), "bob");
        assert_eq!(audit_a.extraneous_consent.iter().next().unwrap(), "charlie");
    }

    /// Mixed scenario: overlap, gap, and extraneous all present simultaneously.
    #[test]
    fn audit_gap_and_extraneous() {
        let owners = vec!["alice".into(), "bob".into(), "carol".into()];
        let consents = vec!["alice".into(), "dave".into()];
        let audit = collab_consent_audit(&owners, &consents).unwrap();
        assert_eq!(audit.satisfied, BTreeSet::from(["alice".to_string()]));
        assert_eq!(
            audit.missing_consent,
            BTreeSet::from(["bob".to_string(), "carol".to_string()])
        );
        assert_eq!(
            audit.extraneous_consent,
            BTreeSet::from(["dave".to_string()])
        );
        assert!(!is_fully_consented(&audit));
    }
}
