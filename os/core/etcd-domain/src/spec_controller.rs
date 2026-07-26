//! The etcd lifecycle controllers.
//!
//! Mirrors `internal/app/machined/pkg/controllers/etcd`:
//!
//! * The **spec controller** turns machine-config inputs + COSI preconditions
//!   into a desired [`EtcdConfig`] (bootstrap vs. join, PKI wiring).
//! * The **lifecycle controller** reconciles a member through the
//!   join -> learn -> promote sequence against an [`EtcdClient`], and handles
//!   graceful leave / forfeit-leadership during reset and upgrade.
//! * The **defrag / member-status** reconcilers compute maintenance actions.

use os_kernel::{Error, Result};

use crate::client::EtcdClient;
use crate::config::{BootstrapMode, EtcdConfig, EtcdPki};
use crate::member::{MemberId, MemberPhase, MemberSet};

/// Inputs to the spec controller: the COSI preconditions and config bits Talos
/// gates etcd bring-up on.
#[derive(Debug, Clone)]
pub struct SpecInput {
    /// Node hostname (member name).
    pub hostname: String,
    /// This node's advertised IP.
    pub advertised_ip: String,
    /// Whether the time subsystem reports the clock as synced (etcd will not
    /// start before this).
    pub time_synced: bool,
    /// Whether this node has been told to bootstrap a new cluster.
    pub bootstrap_requested: bool,
    /// Whether a cluster already exists to join (discovered peers present).
    pub existing_cluster: bool,
    /// PKI material available for this node.
    pub pki: EtcdPki,
    /// Known peers (name -> peer URL) for join mode.
    pub peers: std::collections::BTreeMap<String, String>,
}

/// The spec controller: decides the bring-up mode and produces an
/// [`EtcdConfig`], enforcing Talos's preconditions.
#[derive(Debug, Default)]
pub struct EtcdSpecController;

impl EtcdSpecController {
    /// Build the controller.
    pub fn new() -> Self {
        EtcdSpecController
    }

    /// Reconcile the desired etcd config, or return why it is not yet ready.
    pub fn reconcile(&self, input: &SpecInput) -> Result<EtcdConfig> {
        if !input.time_synced {
            return Err(Error::invalid_state(
                "etcd start gated: clock not yet synced",
            ));
        }
        if input.hostname.trim().is_empty() {
            return Err(Error::invalid("etcd spec: empty hostname"));
        }

        let mode = if input.bootstrap_requested {
            BootstrapMode::Bootstrap
        } else if input.existing_cluster {
            BootstrapMode::Join
        } else {
            return Err(Error::invalid_state(
                "etcd start gated: neither bootstrap requested nor existing cluster found",
            ));
        };

        let mut cfg = EtcdConfig::bootstrap(
            input.hostname.clone(),
            input.advertised_ip.clone(),
            input.pki.clone(),
        );
        cfg.bootstrap = mode;

        if matches!(mode, BootstrapMode::Join) {
            // The initial cluster for a joining learner is the existing peers
            // plus this node.
            cfg.initial_cluster = input.peers.clone();
            cfg.initial_cluster
                .insert(input.hostname.clone(), cfg.peer_url());
        }

        cfg.validate()?;
        Ok(cfg)
    }
}

/// Actions the lifecycle controller asks the runtime to perform. Returning an
/// explicit action (rather than performing IO directly) keeps the reconcile
/// function pure and testable, like Talos's controllers emitting resources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleAction {
    /// Nothing to do this tick.
    Noop,
    /// Add this node as a learner to the existing cluster.
    AddAsLearner {
        name: String,
        peer_urls: Vec<String>,
    },
    /// Promote a caught-up learner.
    Promote(MemberId),
    /// Wait for the learner to catch up before promoting.
    WaitForCatchup(MemberId),
    /// Move leadership away before this node leaves.
    ForfeitLeadership { to: MemberId },
    /// Gracefully remove this member from the cluster.
    Remove(MemberId),
}

/// The lifecycle controller reconciling one member against the cluster.
#[derive(Debug)]
pub struct EtcdLifecycleController {
    /// This node's member name.
    pub name: String,
    /// This node's peer URLs.
    pub peer_urls: Vec<String>,
    /// The desired bring-up mode.
    pub mode: BootstrapMode,
}

impl EtcdLifecycleController {
    /// Construct from a config.
    pub fn from_config(cfg: &EtcdConfig) -> Self {
        EtcdLifecycleController {
            name: cfg.name.clone(),
            peer_urls: vec![cfg.peer_url()],
            mode: cfg.bootstrap,
        }
    }

    /// Determine this node's own member ID from the cluster, if present.
    fn own_id<C: EtcdClient>(&self, client: &C) -> Result<Option<MemberId>> {
        let members = client.member_list()?;
        Ok(members
            .into_iter()
            .find(|m| m.name == self.name)
            .map(|m| m.id))
    }

    /// Reconcile one tick toward a healthy voting membership.
    pub fn reconcile_join<C: EtcdClient>(&self, client: &C) -> Result<LifecycleAction> {
        // Bootstrap nodes are already members (they initialized the cluster).
        if matches!(
            self.mode,
            BootstrapMode::Bootstrap | BootstrapMode::RestoreFromSnapshot
        ) {
            return Ok(LifecycleAction::Noop);
        }

        match self.own_id(client)? {
            None => Ok(LifecycleAction::AddAsLearner {
                name: self.name.clone(),
                peer_urls: self.peer_urls.clone(),
            }),
            Some(id) => {
                let status = client.status(id)?;
                if !status.is_learner {
                    // Already a full voter.
                    return Ok(LifecycleAction::Noop);
                }
                // Learner: promote once caught up to the leader's index.
                let leader = client.leader()?;
                let leader_status = client.status(leader)?;
                if status.raft_index >= leader_status.raft_index {
                    Ok(LifecycleAction::Promote(id))
                } else {
                    Ok(LifecycleAction::WaitForCatchup(id))
                }
            }
        }
    }

    /// Reconcile graceful leave: forfeit leadership first if we are the leader,
    /// otherwise remove ourselves.
    pub fn reconcile_leave<C: EtcdClient>(&self, client: &C) -> Result<LifecycleAction> {
        let id = match self.own_id(client)? {
            Some(id) => id,
            None => return Ok(LifecycleAction::Noop), // already gone
        };
        let members = client.member_list()?;
        let set = MemberSet::from_members(members);

        // Refuse to remove the last voter (would destroy the cluster).
        if set.voter_count() <= 1 && set.get(id).map(|m| !m.is_learner).unwrap_or(false) {
            return Err(Error::invalid_state(
                "refusing to remove the last voting etcd member",
            ));
        }

        let leader = client.leader()?;
        if leader == id {
            // Hand leadership to another voter before leaving.
            let target = set
                .members()
                .iter()
                .find(|m| m.id != id && !m.is_learner)
                .map(|m| m.id)
                .ok_or_else(|| Error::invalid_state("no other voter to forfeit leadership to"))?;
            return Ok(LifecycleAction::ForfeitLeadership { to: target });
        }
        Ok(LifecycleAction::Remove(id))
    }

    /// Drive the join reconcile to completion, applying actions against the
    /// client. Used by the higher-level service and exercised by tests.
    pub fn run_join<C: EtcdClient>(&self, client: &C, max_ticks: usize) -> Result<MemberPhase> {
        let mut last = MemberPhase::Joining;
        for _ in 0..max_ticks {
            match self.reconcile_join(client)? {
                LifecycleAction::Noop => {
                    last = MemberPhase::Ready;
                    break;
                }
                LifecycleAction::AddAsLearner { name, peer_urls } => {
                    client.member_add_as_learner(&name, &peer_urls)?;
                    last = MemberPhase::Learning;
                }
                LifecycleAction::Promote(id) => {
                    client.member_promote(id)?;
                    last = MemberPhase::Ready;
                }
                LifecycleAction::WaitForCatchup(_) => {
                    last = MemberPhase::Learning;
                    // The caller (or replication) must advance catch-up; a real
                    // controller would requeue. Break to avoid spinning.
                    break;
                }
                other => {
                    return Err(Error::invalid_state(format!(
                        "unexpected action during join: {other:?}"
                    )));
                }
            }
        }
        Ok(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::InMemoryEtcd;
    use std::collections::BTreeMap;

    fn pki() -> EtcdPki {
        EtcdPki {
            ca_cert: "CA".into(),
            ca_key: "CAKEY".into(),
            cert: "C".into(),
            key: "K".into(),
        }
    }

    fn base_input() -> SpecInput {
        SpecInput {
            hostname: "cp1".into(),
            advertised_ip: "10.0.0.1".into(),
            time_synced: true,
            bootstrap_requested: true,
            existing_cluster: false,
            pki: pki(),
            peers: BTreeMap::new(),
        }
    }

    #[test]
    fn spec_gated_on_time_sync() {
        let mut input = base_input();
        input.time_synced = false;
        assert!(EtcdSpecController::new().reconcile(&input).is_err());
    }

    #[test]
    fn spec_bootstrap_mode() {
        let cfg = EtcdSpecController::new().reconcile(&base_input()).unwrap();
        assert_eq!(cfg.bootstrap, BootstrapMode::Bootstrap);
    }

    #[test]
    fn spec_join_mode_builds_initial_cluster() {
        let mut input = base_input();
        input.bootstrap_requested = false;
        input.existing_cluster = true;
        input.hostname = "cp2".into();
        input.advertised_ip = "10.0.0.2".into();
        input.pki.ca_key = String::new(); // joiners need no CA key
        input
            .peers
            .insert("cp1".into(), "https://10.0.0.1:2380".into());
        let cfg = EtcdSpecController::new().reconcile(&input).unwrap();
        assert_eq!(cfg.bootstrap, BootstrapMode::Join);
        assert!(cfg.initial_cluster.contains_key("cp1"));
        assert!(cfg.initial_cluster.contains_key("cp2"));
    }

    #[test]
    fn spec_no_bootstrap_no_cluster_is_gated() {
        let mut input = base_input();
        input.bootstrap_requested = false;
        input.existing_cluster = false;
        assert!(EtcdSpecController::new().reconcile(&input).is_err());
    }

    #[test]
    fn join_flow_adds_learner_then_waits_then_promotes() {
        let (etcd, _leader) =
            InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://10.0.0.1:2380".into()]);
        let mut cfg = EtcdConfig::bootstrap("cp2", "10.0.0.2", {
            let mut p = pki();
            p.ca_key = String::new();
            p
        });
        cfg.bootstrap = BootstrapMode::Join;
        cfg.initial_cluster
            .insert("cp1".into(), "https://10.0.0.1:2380".into());
        let ctrl = EtcdLifecycleController::from_config(&cfg);

        // First run adds the learner and stops at WaitForCatchup.
        let phase = ctrl.run_join(&etcd, 5).unwrap();
        assert_eq!(phase, MemberPhase::Learning);

        // Simulate replication catching up, then run again -> promoted.
        let id = etcd
            .member_list()
            .unwrap()
            .into_iter()
            .find(|m| m.name == "cp2")
            .unwrap()
            .id;
        etcd.sync_member(id);
        let phase = ctrl.run_join(&etcd, 5).unwrap();
        assert_eq!(phase, MemberPhase::Ready);
        assert!(!etcd.status(id).unwrap().is_learner);
    }

    #[test]
    fn bootstrap_node_join_is_noop() {
        let (etcd, _) =
            InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://10.0.0.1:2380".into()]);
        let cfg = EtcdConfig::bootstrap("cp1", "10.0.0.1", pki());
        let ctrl = EtcdLifecycleController::from_config(&cfg);
        assert_eq!(ctrl.reconcile_join(&etcd).unwrap(), LifecycleAction::Noop);
    }

    #[test]
    fn leave_forfeits_leadership_when_leader() {
        let (etcd, leader) =
            InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://10.0.0.1:2380".into()]);
        // Add a second voter so leaving is allowed.
        let l2 = etcd
            .member_add_as_learner("cp2", &["https://10.0.0.2:2380".to_string()])
            .unwrap();
        etcd.sync_member(l2);
        etcd.member_promote(l2).unwrap();

        let cfg = EtcdConfig::bootstrap("cp1", "10.0.0.1", pki());
        let ctrl = EtcdLifecycleController::from_config(&cfg);
        let action = ctrl.reconcile_leave(&etcd).unwrap();
        assert_eq!(action, LifecycleAction::ForfeitLeadership { to: l2 });

        // After moving leadership, leave removes us.
        etcd.move_leader(l2).unwrap();
        let _ = leader;
        let action = ctrl.reconcile_leave(&etcd).unwrap();
        match action {
            LifecycleAction::Remove(_) => {}
            other => panic!("expected Remove, got {other:?}"),
        }
    }

    #[test]
    fn leave_refuses_last_voter() {
        let (etcd, _) =
            InMemoryEtcd::with_bootstrap_member("cp1", vec!["https://10.0.0.1:2380".into()]);
        let cfg = EtcdConfig::bootstrap("cp1", "10.0.0.1", pki());
        let ctrl = EtcdLifecycleController::from_config(&cfg);
        assert!(ctrl.reconcile_leave(&etcd).is_err());
    }
}
