//! etcd member identity, roles, and the member lifecycle state machine.
//!
//! Mirrors Talos's notion of an etcd member as managed by `pkg/etcd` and the
//! `internal/app/machined/pkg/controllers/etcd` controllers. A member is added
//! to the cluster as a *learner* (non-voting), then *promoted* to a full
//! voting member once it has caught up with the leader's Raft log. Members can
//! also be *removed* (graceful leave or forfeit during reset/upgrade).

use std::collections::BTreeSet;
use std::fmt;

use os_kernel::{Error, Result};

/// A 64-bit etcd member ID, as returned by the etcd membership API.
///
/// etcd assigns these from a hash of the member's peer URLs and cluster ID, so
/// they are effectively opaque random-looking identifiers. We keep the raw
/// value and render it as the hex string etcd uses in logs and `etcdctl`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemberId(pub u64);

impl MemberId {
    /// The zero member ID, used as a sentinel for "no member".
    pub const NONE: MemberId = MemberId(0);

    /// Returns true if this is the sentinel/zero ID.
    pub fn is_none(self) -> bool {
        self.0 == 0
    }

    /// The hex rendering etcd uses (no `0x` prefix, lowercase).
    pub fn to_hex(self) -> String {
        format!("{:x}", self.0)
    }

    /// Parse a member ID from the hex string form.
    pub fn from_hex(s: &str) -> Result<Self> {
        let s = s.trim().trim_start_matches("0x");
        u64::from_str_radix(s, 16)
            .map(MemberId)
            .map_err(|_| Error::parse(format!("invalid etcd member id: {s:?}")))
    }
}

impl fmt::Display for MemberId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

/// The role of a member in the Raft quorum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberRole {
    /// A non-voting member that is replicating the log to catch up.
    Learner,
    /// A full voting member that participates in quorum and can be leader.
    Voter,
}

impl MemberRole {
    /// Whether this role counts toward quorum.
    pub fn votes(self) -> bool {
        matches!(self, MemberRole::Voter)
    }

    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            MemberRole::Learner => "learner",
            MemberRole::Voter => "voter",
        }
    }
}

/// The lifecycle phase of a member from this node's controller perspective.
///
/// This is the local view of where a member is in the join/promote/remove
/// dance, distinct from the etcd-reported [`MemberRole`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberPhase {
    /// Membership requested but the etcd `MemberAdd` call has not completed.
    Joining,
    /// Added as a learner; replicating but not yet promotable.
    Learning,
    /// Promotion in progress (`MemberPromote` issued).
    Promoting,
    /// A healthy voting member.
    Ready,
    /// A `MemberRemove` has been issued; the member is leaving.
    Leaving,
    /// The member has been removed from the cluster.
    Removed,
}

impl MemberPhase {
    /// Whether the member is in a terminal phase.
    pub fn is_terminal(self) -> bool {
        matches!(self, MemberPhase::Removed)
    }

    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            MemberPhase::Joining => "joining",
            MemberPhase::Learning => "learning",
            MemberPhase::Promoting => "promoting",
            MemberPhase::Ready => "ready",
            MemberPhase::Leaving => "leaving",
            MemberPhase::Removed => "removed",
        }
    }
}

/// A single etcd cluster member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// etcd-assigned member ID (zero until `MemberAdd` returns).
    pub id: MemberId,
    /// Human-readable member name (Talos uses the node hostname).
    pub name: String,
    /// Raft peer URLs (typically `https://<ip>:2380`).
    pub peer_urls: Vec<String>,
    /// Client-facing URLs (typically `https://<ip>:2379`).
    pub client_urls: Vec<String>,
    /// Whether etcd currently classifies this member as a learner.
    pub is_learner: bool,
    /// The local controller phase for this member.
    pub phase: MemberPhase,
}

impl Member {
    /// Construct a fresh member in the [`MemberPhase::Joining`] phase.
    pub fn new(name: impl Into<String>, peer_urls: Vec<String>, client_urls: Vec<String>) -> Self {
        Member {
            id: MemberId::NONE,
            name: name.into(),
            peer_urls,
            client_urls,
            is_learner: true,
            phase: MemberPhase::Joining,
        }
    }

    /// The effective Raft role.
    pub fn role(&self) -> MemberRole {
        if self.is_learner {
            MemberRole::Learner
        } else {
            MemberRole::Voter
        }
    }

    /// Validate that the member has the URLs etcd requires.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("etcd member name must not be empty"));
        }
        if self.peer_urls.is_empty() {
            return Err(Error::invalid(format!(
                "etcd member {:?} has no peer URLs",
                self.name
            )));
        }
        for url in self.peer_urls.iter().chain(self.client_urls.iter()) {
            validate_url(url)?;
        }
        Ok(())
    }

    /// Whether this member is eligible for promotion: it must currently be a
    /// learner and have caught up (the caller supplies the catch-up decision).
    pub fn can_promote(&self, caught_up: bool) -> bool {
        self.is_learner && caught_up && self.phase != MemberPhase::Removed
    }
}

fn validate_url(url: &str) -> Result<()> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| Error::invalid(format!("etcd URL must have scheme: {url:?}")))?;
    let host_port = rest.split('/').next().unwrap_or("");
    let (host, port) = host_port
        .rsplit_once(':')
        .ok_or_else(|| Error::invalid(format!("etcd URL must have host:port: {url:?}")))?;
    if host.is_empty() {
        return Err(Error::invalid(format!("etcd URL has empty host: {url:?}")));
    }
    port.parse::<u16>()
        .map_err(|_| Error::invalid(format!("etcd URL has invalid port: {url:?}")))?;
    Ok(())
}

/// The whole-cluster membership view, used to reason about quorum.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemberSet {
    members: Vec<Member>,
}

impl MemberSet {
    /// Empty member set.
    pub fn new() -> Self {
        MemberSet {
            members: Vec::new(),
        }
    }

    /// Build from an existing list of members.
    pub fn from_members(members: Vec<Member>) -> Self {
        MemberSet { members }
    }

    /// All members.
    pub fn members(&self) -> &[Member] {
        &self.members
    }

    /// Number of members of any role.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Insert or replace a member by ID (or by name when ID is unset).
    pub fn upsert(&mut self, member: Member) {
        if let Some(existing) = self.members.iter_mut().find(|m| {
            // Match by ID when both carry one, otherwise fall back to name so a
            // member that has just been assigned an ID by etcd updates the
            // name-keyed entry in place rather than duplicating it.
            (!member.id.is_none() && !m.id.is_none() && m.id == member.id) || m.name == member.name
        }) {
            *existing = member;
        } else {
            self.members.push(member);
        }
    }

    /// Remove a member by ID, returning it if present.
    pub fn remove(&mut self, id: MemberId) -> Option<Member> {
        if let Some(pos) = self.members.iter().position(|m| m.id == id) {
            Some(self.members.remove(pos))
        } else {
            None
        }
    }

    /// Look up a member by ID.
    pub fn get(&self, id: MemberId) -> Option<&Member> {
        self.members.iter().find(|m| m.id == id)
    }

    /// Look up a member by name.
    pub fn get_by_name(&self, name: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.name == name)
    }

    /// The number of voting members (those that count toward quorum).
    pub fn voter_count(&self) -> usize {
        self.members.iter().filter(|m| !m.is_learner).count()
    }

    /// The number of learners.
    pub fn learner_count(&self) -> usize {
        self.members.iter().filter(|m| m.is_learner).count()
    }

    /// The quorum size required for the current voter count: `floor(n/2)+1`.
    pub fn quorum(&self) -> usize {
        self.voter_count() / 2 + 1
    }

    /// Whether quorum is retained if `lost` voting members become unavailable.
    pub fn quorum_retained_losing(&self, lost: usize) -> bool {
        let remaining = self.voter_count().saturating_sub(lost);
        remaining >= self.quorum()
    }

    /// Whether it is safe to remove `count` voters without losing quorum.
    ///
    /// This is the check Talos applies before scaling down / resetting a node:
    /// removing a voter must not drop the surviving voters below quorum.
    pub fn safe_to_remove_voters(&self, count: usize) -> bool {
        if count == 0 {
            return true;
        }
        let after = self.voter_count().saturating_sub(count);
        // After removal the quorum recomputes against the smaller set.
        after > 0 && after >= (after / 2 + 1)
    }

    /// Distinct member names; used to detect duplicate joins.
    pub fn names(&self) -> BTreeSet<&str> {
        self.members.iter().map(|m| m.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voter(name: &str) -> Member {
        let mut m = Member::new(
            name,
            vec![format!("https://{name}:2380")],
            vec![format!("https://{name}:2379")],
        );
        m.is_learner = false;
        m.phase = MemberPhase::Ready;
        m
    }

    #[test]
    fn member_id_hex_roundtrip() {
        let id = MemberId(0xdead_beef);
        assert_eq!(id.to_hex(), "deadbeef");
        assert_eq!(MemberId::from_hex("deadbeef").unwrap(), id);
        assert_eq!(MemberId::from_hex("0xDEADBEEF").unwrap(), id);
        assert!(MemberId::from_hex("zz").is_err());
    }

    #[test]
    fn validate_rejects_bad_urls() {
        let mut m = Member::new("n1", vec!["ftp://x:1".into()], vec![]);
        m.is_learner = false;
        assert!(m.validate().is_err());

        let m2 = Member::new("n2", vec!["https://10.0.0.1:2380".into()], vec![]);
        assert!(m2.validate().is_ok());

        let empty = Member::new("", vec!["https://x:1".into()], vec![]);
        assert!(empty.validate().is_err());

        let no_peers = Member::new("n3", vec![], vec![]);
        assert!(no_peers.validate().is_err());
    }

    #[test]
    fn quorum_math() {
        let mut set = MemberSet::new();
        set.upsert(voter("a"));
        assert_eq!(set.quorum(), 1);
        set.upsert(voter("b"));
        set.upsert(voter("c"));
        assert_eq!(set.voter_count(), 3);
        assert_eq!(set.quorum(), 2);
        // losing one of three voters keeps quorum.
        assert!(set.quorum_retained_losing(1));
        // losing two does not.
        assert!(!set.quorum_retained_losing(2));
    }

    #[test]
    fn safe_to_remove_voters_logic() {
        let mut set = MemberSet::new();
        set.upsert(voter("a"));
        set.upsert(voter("b"));
        set.upsert(voter("c"));
        // 3 -> 2 voters: 2 >= quorum(2) ok.
        assert!(set.safe_to_remove_voters(1));
        // 3 -> 1 voter: still has a quorum of 1.
        assert!(set.safe_to_remove_voters(2));
        // 3 -> 0 voters: never safe.
        assert!(!set.safe_to_remove_voters(3));
    }

    #[test]
    fn upsert_by_name_then_id() {
        let mut set = MemberSet::new();
        let m = Member::new("a", vec!["https://a:2380".into()], vec![]);
        set.upsert(m);
        assert_eq!(set.len(), 1);
        // Now etcd assigns an ID; upserting the same name updates in place.
        let mut m2 = Member::new("a", vec!["https://a:2380".into()], vec![]);
        m2.id = MemberId(42);
        set.upsert(m2);
        assert_eq!(set.len(), 1);
        assert_eq!(set.get(MemberId(42)).unwrap().name, "a");
    }

    #[test]
    fn learner_promotion_eligibility() {
        let mut m = Member::new("a", vec!["https://a:2380".into()], vec![]);
        assert!(!m.role().votes());
        assert!(!m.can_promote(false));
        assert!(m.can_promote(true));
        m.is_learner = false;
        assert!(!m.can_promote(true));
    }
}
