//! The etcd client modeled as a trait, plus an in-memory fake used by tests
//! and by the controllers in this crate.
//!
//! Talos's `pkg/etcd` wraps the real `go.etcd.io/etcd/client/v3` and exposes
//! membership, status, defrag, snapshot, and maintenance operations over the
//! etcd gRPC API. Here we capture that surface as the [`EtcdClient`] trait so
//! the lifecycle logic can be exercised offline. [`InMemoryEtcd`] is a small
//! but behaviorally-real simulator: it tracks members, learner state, the Raft
//! leader, an applied/committed index, and the key/value revision used for
//! snapshots.

use std::collections::BTreeMap;
use std::sync::Mutex;

use os_kernel::{Error, Result};

use crate::member::{Member, MemberId, MemberPhase, MemberRole};

/// Status of one member as reported by etcd's `Status` maintenance RPC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberStatus {
    /// The member.
    pub id: MemberId,
    /// etcd server version string.
    pub version: String,
    /// The DB physical size in bytes (what defrag shrinks).
    pub db_size: u64,
    /// The DB logically-used size in bytes.
    pub db_size_in_use: u64,
    /// The current Raft leader's member ID.
    pub leader: MemberId,
    /// The Raft term.
    pub raft_term: u64,
    /// The Raft applied index.
    pub raft_index: u64,
    /// Whether this member is a learner.
    pub is_learner: bool,
    /// Any alarms (e.g. NOSPACE, CORRUPT) currently raised against the member.
    pub alarms: Vec<String>,
}

impl MemberStatus {
    /// Whether this member currently believes it is the leader.
    pub fn is_leader(&self) -> bool {
        !self.leader.is_none() && self.leader == self.id
    }

    /// Fragmentation ratio: physical size over in-use size (>= 1.0). A high
    /// value indicates defrag would reclaim space.
    pub fn fragmentation_ratio(&self) -> f64 {
        if self.db_size_in_use == 0 {
            return 1.0;
        }
        self.db_size as f64 / self.db_size_in_use as f64
    }
}

/// The etcd client surface used by the lifecycle controllers.
pub trait EtcdClient {
    /// List the current cluster members.
    fn member_list(&self) -> Result<Vec<Member>>;

    /// Add a member as a learner, returning its assigned ID.
    fn member_add_as_learner(&self, name: &str, peer_urls: &[String]) -> Result<MemberId>;

    /// Promote a learner to a voting member.
    fn member_promote(&self, id: MemberId) -> Result<()>;

    /// Remove a member from the cluster.
    fn member_remove(&self, id: MemberId) -> Result<()>;

    /// Get the maintenance status for a particular member.
    fn status(&self, id: MemberId) -> Result<MemberStatus>;

    /// The current Raft leader.
    fn leader(&self) -> Result<MemberId>;

    /// Defragment a member's backend, returning the bytes reclaimed.
    fn defragment(&self, id: MemberId) -> Result<u64>;

    /// Move leadership to the target member (used before graceful leave).
    fn move_leader(&self, target: MemberId) -> Result<()>;

    /// Current key-space revision (used by snapshot bookkeeping).
    fn revision(&self) -> Result<u64>;
}

/// An internal record per simulated member.
#[derive(Debug, Clone)]
struct Sim {
    member: Member,
    db_size: u64,
    db_size_in_use: u64,
    applied_index: u64,
}

/// In-memory etcd simulator implementing [`EtcdClient`].
#[derive(Debug)]
pub struct InMemoryEtcd {
    inner: Mutex<State>,
}

#[derive(Debug)]
struct State {
    members: BTreeMap<MemberId, Sim>,
    next_id: u64,
    leader: MemberId,
    term: u64,
    revision: u64,
    /// The leader's commit index, against which learners measure catch-up.
    leader_index: u64,
}

impl InMemoryEtcd {
    /// Create an empty single-cluster simulator with no members.
    pub fn new() -> Self {
        InMemoryEtcd {
            inner: Mutex::new(State {
                members: BTreeMap::new(),
                next_id: 1,
                leader: MemberId::NONE,
                term: 1,
                revision: 1,
                leader_index: 10,
            }),
        }
    }

    /// Seed the simulator with an initial voting member that becomes leader.
    /// Returns the new member's ID.
    pub fn with_bootstrap_member(name: &str, peer_urls: Vec<String>) -> (Self, MemberId) {
        let etcd = Self::new();
        let id = {
            let mut st = etcd.inner.lock().unwrap();
            let id = MemberId(st.next_id);
            st.next_id += 1;
            let mut m = Member::new(name, peer_urls, vec![]);
            m.id = id;
            m.is_learner = false;
            m.phase = MemberPhase::Ready;
            let leader_index = st.leader_index;
            st.members.insert(
                id,
                Sim {
                    member: m,
                    db_size: 1024,
                    db_size_in_use: 1024,
                    applied_index: leader_index,
                },
            );
            st.leader = id;
            id
        };
        (etcd, id)
    }

    /// Advance the leader's commit index, simulating cluster write activity.
    pub fn advance_leader_index(&self, by: u64) {
        let mut st = self.inner.lock().unwrap();
        st.leader_index += by;
        st.revision += by;
    }

    /// Force a learner's replication to catch up to the leader index.
    pub fn sync_member(&self, id: MemberId) {
        let mut st = self.inner.lock().unwrap();
        let leader_index = st.leader_index;
        if let Some(sim) = st.members.get_mut(&id) {
            sim.applied_index = leader_index;
        }
    }

    /// Inflate a member's physical DB size relative to in-use, to simulate
    /// fragmentation that defrag should reclaim.
    pub fn inflate_db(&self, id: MemberId, physical: u64, in_use: u64) {
        let mut st = self.inner.lock().unwrap();
        if let Some(sim) = st.members.get_mut(&id) {
            sim.db_size = physical;
            sim.db_size_in_use = in_use;
        }
    }
}

impl Default for InMemoryEtcd {
    fn default() -> Self {
        Self::new()
    }
}

impl EtcdClient for InMemoryEtcd {
    fn member_list(&self) -> Result<Vec<Member>> {
        let st = self.inner.lock().unwrap();
        Ok(st.members.values().map(|s| s.member.clone()).collect())
    }

    fn member_add_as_learner(&self, name: &str, peer_urls: &[String]) -> Result<MemberId> {
        let mut st = self.inner.lock().unwrap();
        if st.members.values().any(|s| s.member.name == name) {
            return Err(Error::invalid_state(format!(
                "member {name:?} already exists"
            )));
        }
        let id = MemberId(st.next_id);
        st.next_id += 1;
        let mut m = Member::new(name, peer_urls.to_vec(), vec![]);
        m.id = id;
        m.is_learner = true;
        m.phase = MemberPhase::Learning;
        // Learners start behind the leader.
        st.members.insert(
            id,
            Sim {
                member: m,
                db_size: 256,
                db_size_in_use: 256,
                applied_index: 0,
            },
        );
        Ok(id)
    }

    fn member_promote(&self, id: MemberId) -> Result<()> {
        let mut st = self.inner.lock().unwrap();
        let leader_index = st.leader_index;
        let sim = st
            .members
            .get_mut(&id)
            .ok_or_else(|| Error::not_found(format!("member {id}")))?;
        if !sim.member.is_learner {
            return Err(Error::invalid_state("member is not a learner"));
        }
        // etcd rejects promotion if the learner has not caught up.
        if sim.applied_index < leader_index {
            return Err(Error::invalid_state(
                "learner has not caught up; promotion refused",
            ));
        }
        sim.member.is_learner = false;
        sim.member.phase = MemberPhase::Ready;
        Ok(())
    }

    fn member_remove(&self, id: MemberId) -> Result<()> {
        let mut st = self.inner.lock().unwrap();
        if st.members.remove(&id).is_none() {
            return Err(Error::not_found(format!("member {id}")));
        }
        if st.leader == id {
            // Leadership transfers to an arbitrary remaining voter, if any.
            st.leader = st
                .members
                .values()
                .find(|s| !s.member.is_learner)
                .map(|s| s.member.id)
                .unwrap_or(MemberId::NONE);
            st.term += 1;
        }
        Ok(())
    }

    fn status(&self, id: MemberId) -> Result<MemberStatus> {
        let st = self.inner.lock().unwrap();
        let sim = st
            .members
            .get(&id)
            .ok_or_else(|| Error::not_found(format!("member {id}")))?;
        Ok(MemberStatus {
            id,
            version: "3.5.0".to_string(),
            db_size: sim.db_size,
            db_size_in_use: sim.db_size_in_use,
            leader: st.leader,
            raft_term: st.term,
            raft_index: sim.applied_index,
            is_learner: sim.member.is_learner,
            alarms: Vec::new(),
        })
    }

    fn leader(&self) -> Result<MemberId> {
        let st = self.inner.lock().unwrap();
        if st.leader.is_none() {
            return Err(Error::invalid_state("no leader elected"));
        }
        Ok(st.leader)
    }

    fn defragment(&self, id: MemberId) -> Result<u64> {
        let mut st = self.inner.lock().unwrap();
        let sim = st
            .members
            .get_mut(&id)
            .ok_or_else(|| Error::not_found(format!("member {id}")))?;
        let reclaimed = sim.db_size.saturating_sub(sim.db_size_in_use);
        sim.db_size = sim.db_size_in_use;
        Ok(reclaimed)
    }

    fn move_leader(&self, target: MemberId) -> Result<()> {
        let mut st = self.inner.lock().unwrap();
        let sim = st
            .members
            .get(&target)
            .ok_or_else(|| Error::not_found(format!("member {target}")))?;
        if sim.member.is_learner {
            return Err(Error::invalid_state("cannot move leadership to a learner"));
        }
        st.leader = target;
        st.term += 1;
        Ok(())
    }

    fn revision(&self) -> Result<u64> {
        Ok(self.inner.lock().unwrap().revision)
    }
}

/// Whether a member status indicates it is healthy enough to serve.
pub fn member_role(status: &MemberStatus) -> MemberRole {
    if status.is_learner {
        MemberRole::Learner
    } else {
        MemberRole::Voter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_member_is_leader() {
        let (etcd, id) = InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://a:2380".into()]);
        assert_eq!(etcd.leader().unwrap(), id);
        let status = etcd.status(id).unwrap();
        assert!(status.is_leader());
        assert!(!status.is_learner);
    }

    #[test]
    fn add_learner_then_promote_after_catchup() {
        let (etcd, _leader) =
            InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://a:2380".into()]);
        let learner = etcd
            .member_add_as_learner("cp2", &["https://b:2380".to_string()])
            .unwrap();
        // Promotion before catch-up is refused.
        assert!(etcd.member_promote(learner).is_err());
        // After syncing, promotion succeeds.
        etcd.sync_member(learner);
        assert!(etcd.member_promote(learner).is_ok());
        let status = etcd.status(learner).unwrap();
        assert!(!status.is_learner);
    }

    #[test]
    fn duplicate_member_add_rejected() {
        let (etcd, _) = InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://a:2380".into()]);
        assert!(
            etcd.member_add_as_learner("cp1", &["https://a:2380".to_string()])
                .is_err()
        );
    }

    #[test]
    fn remove_leader_transfers_leadership() {
        let (etcd, leader) =
            InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://a:2380".into()]);
        let l2 = etcd
            .member_add_as_learner("cp2", &["https://b:2380".to_string()])
            .unwrap();
        etcd.sync_member(l2);
        etcd.member_promote(l2).unwrap();
        etcd.member_remove(leader).unwrap();
        assert_eq!(etcd.leader().unwrap(), l2);
    }

    #[test]
    fn defrag_reclaims_fragmentation() {
        let (etcd, id) = InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://a:2380".into()]);
        etcd.inflate_db(id, 4096, 1024);
        let status = etcd.status(id).unwrap();
        assert!(status.fragmentation_ratio() > 3.0);
        let reclaimed = etcd.defragment(id).unwrap();
        assert_eq!(reclaimed, 3072);
        let after = etcd.status(id).unwrap();
        assert_eq!(after.fragmentation_ratio(), 1.0);
    }

    #[test]
    fn move_leader_rejects_learner() {
        let (etcd, _) = InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://a:2380".into()]);
        let learner = etcd
            .member_add_as_learner("cp2", &["https://b:2380".to_string()])
            .unwrap();
        assert!(etcd.move_leader(learner).is_err());
    }
}
