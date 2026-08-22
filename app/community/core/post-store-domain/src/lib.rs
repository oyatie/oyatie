#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
pub mod ranking;
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
pub use ranking::rank_posts;
use std::collections::BTreeSet;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunityError {
    Invalid,
    MissingAnonymousDisclosurePolicy,
    SelfVoteForbidden,
    DuplicateVote,
    ModerationNeedsEvidence,
    NoSuchVote,
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
    pub fn retract(
        &mut self,
        voter_ref: &str,
        _post: &CommunityPost,
    ) -> Result<(), CommunityError> {
        ne(voter_ref)?;
        let before = self.receipts.value.len();
        self.receipts.value.retain(|r| r.voter_ref != voter_ref);
        if self.receipts.value.len() == before {
            return Err(CommunityError::NoSuchVote);
        }
        Ok(())
    }
    pub fn tally(&self) -> i64 {
        self.receipts.value.iter().fold(0i64, |acc, r| {
            acc + match r.kind {
                VoteKind::Up => 1,
                VoteKind::Down => -1,
            }
        })
    }
}
pub fn moderation_case(
    post: &CommunityPost,
    action: ModerationAction,
    policy_ref: String,
    evidence_ref: String,
) -> Result<Classified<String>, CommunityError> {
    ne(&post.post_id.value)?;
    ne(&policy_ref)?;
    match action {
        ModerationAction::Allow => Ok(int(policy_ref)),
        ModerationAction::Hide | ModerationAction::Remove => {
            if evidence_ref.trim().is_empty() {
                return Err(CommunityError::ModerationNeedsEvidence);
            }
            Ok(Classified::new(evidence_ref, DataClass::Audit))
        }
    }
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
    // cvt-1: retract tests
    fn peer_receipt(vote_id: &str) -> VoteReceipt {
        VoteReceipt {
            vote_id: vote_id.into(),
            voter_ref: "user:peer".into(),
            post_id: "p".into(),
            kind: VoteKind::Up,
        }
    }
    #[test]
    fn retract_removes_existing_receipt() {
        let p = post();
        let mut l = VoteLedger::new(&p);
        l.cast(peer_receipt("v1"), &p).unwrap();
        assert_eq!(l.receipts.value.len(), 1);
        l.retract("user:peer", &p).unwrap();
        assert!(l.receipts.value.is_empty());
    }
    #[test]
    fn retract_second_time_errors_no_such_vote() {
        let p = post();
        let mut l = VoteLedger::new(&p);
        l.cast(peer_receipt("v1"), &p).unwrap();
        l.retract("user:peer", &p).unwrap();
        assert_eq!(l.retract("user:peer", &p), Err(CommunityError::NoSuchVote));
    }
    #[test]
    fn retract_empty_voter_ref_errors_invalid() {
        let p = post();
        let mut l = VoteLedger::new(&p);
        assert_eq!(l.retract("", &p), Err(CommunityError::Invalid));
        assert_eq!(l.retract("   ", &p), Err(CommunityError::Invalid));
    }
    #[test]
    fn cast_still_raises_duplicate_vote() {
        let p = post();
        let mut l = VoteLedger::new(&p);
        let r = peer_receipt("v1");
        l.cast(r.clone(), &p).unwrap();
        assert_eq!(l.cast(r, &p), Err(CommunityError::DuplicateVote));
    }
    // cvt-2: tally tests
    #[test]
    fn tally_empty_ledger_is_zero() {
        let p = post();
        let l = VoteLedger::new(&p);
        assert_eq!(l.tally(), 0);
    }
    #[test]
    fn tally_mixed_receipts_net_score() {
        let p = post();
        let mut l = VoteLedger::new(&p);
        for (id, voter, kind) in [
            ("v1", "u1", VoteKind::Up),
            ("v2", "u2", VoteKind::Up),
            ("v3", "u3", VoteKind::Up),
            ("v4", "u4", VoteKind::Down),
            ("v5", "u5", VoteKind::Down),
        ] {
            l.cast(
                VoteReceipt {
                    vote_id: id.into(),
                    voter_ref: voter.into(),
                    post_id: "p".into(),
                    kind,
                },
                &p,
            )
            .unwrap();
        }
        assert_eq!(l.tally(), 1);
    }
    #[test]
    fn tally_all_up() {
        let p = post();
        let mut l = VoteLedger::new(&p);
        for (id, voter) in [("v1", "u1"), ("v2", "u2")] {
            l.cast(
                VoteReceipt {
                    vote_id: id.into(),
                    voter_ref: voter.into(),
                    post_id: "p".into(),
                    kind: VoteKind::Up,
                },
                &p,
            )
            .unwrap();
        }
        assert_eq!(l.tally(), 2);
    }
    #[test]
    fn tally_all_down() {
        let p = post();
        let mut l = VoteLedger::new(&p);
        for (id, voter) in [("v1", "u1"), ("v2", "u2")] {
            l.cast(
                VoteReceipt {
                    vote_id: id.into(),
                    voter_ref: voter.into(),
                    post_id: "p".into(),
                    kind: VoteKind::Down,
                },
                &p,
            )
            .unwrap();
        }
        assert_eq!(l.tally(), -2);
    }
    // cvt-3: action-aware moderation tests
    #[test]
    fn moderation_remove_without_evidence_errors() {
        assert_eq!(
            moderation_case(
                &post(),
                ModerationAction::Remove,
                "policy".into(),
                "".into()
            ),
            Err(CommunityError::ModerationNeedsEvidence)
        );
    }
    #[test]
    fn moderation_remove_with_evidence_tagged_audit() {
        use data_boundary_kernel::{DataClass, DataClassification, OperationalDataClass};
        let result = moderation_case(
            &post(),
            ModerationAction::Remove,
            "policy".into(),
            "evidence-ref".into(),
        )
        .unwrap();
        assert_eq!(result.value, "evidence-ref");
        assert_eq!(
            result.data_class,
            DataClassification::from(DataClass::Audit)
        );
        assert_eq!(
            result.data_class,
            DataClassification::from(OperationalDataClass::Audit)
        );
    }
    #[test]
    fn moderation_allow_passes_without_evidence() {
        assert!(
            moderation_case(&post(), ModerationAction::Allow, "policy".into(), "".into()).is_ok()
        );
    }
    #[test]
    fn moderation_allow_tagged_internal_only() {
        use data_boundary_kernel::{DataClass, DataClassification};
        let result = moderation_case(
            &post(),
            ModerationAction::Allow,
            "policy-ref".into(),
            "".into(),
        )
        .unwrap();
        assert_eq!(result.value, "policy-ref");
        assert_eq!(
            result.data_class,
            DataClassification::from(DataClass::InternalOnly)
        );
    }
    #[test]
    fn moderation_hide_still_requires_evidence() {
        assert_eq!(
            moderation_case(&post(), ModerationAction::Hide, "policy".into(), "".into()),
            Err(CommunityError::ModerationNeedsEvidence)
        );
        let result = moderation_case(
            &post(),
            ModerationAction::Hide,
            "policy".into(),
            "evidence".into(),
        )
        .unwrap();
        use data_boundary_kernel::{DataClass, DataClassification};
        assert_eq!(
            result.data_class,
            DataClassification::from(DataClass::Audit)
        );
    }
}
