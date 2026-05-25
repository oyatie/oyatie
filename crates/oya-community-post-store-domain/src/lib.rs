#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunityError {
    Invalid,
    MissingAnonymousDisclosurePolicy,
    SelfVoteForbidden,
    DuplicateVote,
    ModerationNeedsEvidence,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunityMode {
    Reddit,
    Teamblind,
    Handshake,
    KnowledgeBase,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VoteKind {
    Up,
    Down,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModerationAction {
    Allow,
    Hide,
    Remove,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunityAuthor {
    pub routine_display_ref: Classified<String>,
    pub audit_author_ref: Classified<String>,
    pub disclosure_policy_ref: Classified<Option<String>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunityPost {
    pub post_id: Classified<String>,
    pub thread_id: Classified<String>,
    pub tenant_scope_ref: Classified<String>,
    pub mode: Classified<CommunityMode>,
    pub author: Classified<CommunityAuthor>,
    pub body_ref: Classified<String>,
    pub retention_policy_id: Classified<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VoteReceipt {
    pub vote_id: String,
    pub voter_ref: String,
    pub post_id: String,
    pub kind: VoteKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteLedger {
    pub post_id: Classified<String>,
    pub receipts: Classified<BTreeSet<VoteReceipt>>,
}
impl CommunityAuthor {
    pub fn new(
        display: String,
        audit: String,
        policy: Option<String>,
    ) -> Result<Self, CommunityError> {
        ne(&display)?;
        ne(&audit)?;
        Ok(Self {
            routine_display_ref: Classified::new(display, PrivacyDataClass::pii_quasi_identifier()),
            audit_author_ref: Classified::new(audit, PrivacyDataClass::pii_identifying()),
            disclosure_policy_ref: int(policy),
        })
    }
}
impl CommunityPost {
    pub fn new(
        post_id: String,
        thread_id: String,
        tenant_scope_ref: String,
        mode: CommunityMode,
        author: CommunityAuthor,
        body_ref: String,
        retention_policy_id: String,
    ) -> Result<Self, CommunityError> {
        for s in [
            &post_id,
            &thread_id,
            &tenant_scope_ref,
            &body_ref,
            &retention_policy_id,
        ] {
            ne(s)?
        }
        if matches!(mode, CommunityMode::Teamblind | CommunityMode::Handshake)
            && author
                .disclosure_policy_ref
                .value
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
        {
            return Err(CommunityError::MissingAnonymousDisclosurePolicy);
        };
        Ok(Self {
            post_id: int(post_id),
            thread_id: int(thread_id),
            tenant_scope_ref: int(tenant_scope_ref),
            mode: int(mode),
            author: Classified::new(author, PrivacyDataClass::pii_identifying()),
            body_ref: Classified::new(body_ref, PrivacyDataClass::pii_identifying()),
            retention_policy_id: int(retention_policy_id),
        })
    }
}
impl VoteLedger {
    pub fn new(post: &CommunityPost) -> Self {
        Self {
            post_id: int(post.post_id.value.clone()),
            receipts: int(BTreeSet::new()),
        }
    }
    pub fn cast(&mut self, r: VoteReceipt, post: &CommunityPost) -> Result<(), CommunityError> {
        ne(&r.vote_id)?;
        ne(&r.voter_ref)?;
        if r.voter_ref == post.author.value.audit_author_ref.value {
            return Err(CommunityError::SelfVoteForbidden);
        };
        if !self.receipts.value.insert(r) {
            return Err(CommunityError::DuplicateVote);
        };
        Ok(())
    }
}
pub fn moderation_case(
    post: &CommunityPost,
    action: ModerationAction,
    policy_ref: String,
    evidence_ref: String,
) -> Result<Classified<String>, CommunityError> {
    let _ = action;
    ne(&post.post_id.value)?;
    ne(&policy_ref)?;
    if evidence_ref.trim().is_empty() {
        return Err(CommunityError::ModerationNeedsEvidence);
    };
    Ok(Classified::new(evidence_ref, DataClass::Audit))
}
fn ne(s: &str) -> Result<(), CommunityError> {
    if s.trim().is_empty() {
        Err(CommunityError::Invalid)
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
    fn author() -> CommunityAuthor {
        CommunityAuthor::new("anon".into(), "user:real".into(), Some("policy".into())).unwrap()
    }
    fn post() -> CommunityPost {
        CommunityPost::new(
            "p".into(),
            "t".into(),
            "tenant".into(),
            CommunityMode::Teamblind,
            author(),
            "body".into(),
            "retain".into(),
        )
        .unwrap()
    }
    #[test]
    fn teamblind_mode_requires_anonymous_disclosure_policy() {
        let a = CommunityAuthor::new("anon".into(), "user".into(), None).unwrap();
        assert_eq!(
            CommunityPost::new(
                "p".into(),
                "t".into(),
                "tenant".into(),
                CommunityMode::Teamblind,
                a,
                "body".into(),
                "r".into()
            ),
            Err(CommunityError::MissingAnonymousDisclosurePolicy)
        )
    }
    #[test]
    fn voting_is_idempotent_and_self_vote_denied() {
        let p = post();
        let mut l = VoteLedger::new(&p);
        assert_eq!(
            l.cast(
                VoteReceipt {
                    vote_id: "v0".into(),
                    voter_ref: "user:real".into(),
                    post_id: "p".into(),
                    kind: VoteKind::Up
                },
                &p
            ),
            Err(CommunityError::SelfVoteForbidden)
        );
        let r = VoteReceipt {
            vote_id: "v1".into(),
            voter_ref: "user:peer".into(),
            post_id: "p".into(),
            kind: VoteKind::Up,
        };
        l.cast(r.clone(), &p).unwrap();
        assert_eq!(l.cast(r, &p), Err(CommunityError::DuplicateVote))
    }
    #[test]
    fn moderation_action_requires_evidence() {
        assert_eq!(
            moderation_case(&post(), ModerationAction::Hide, "policy".into(), "".into()),
            Err(CommunityError::ModerationNeedsEvidence)
        );
        assert!(
            moderation_case(
                &post(),
                ModerationAction::Hide,
                "policy".into(),
                "audit".into()
            )
            .is_ok()
        )
    }
}
