//! The etcd member spec used to bootstrap and join the cluster's datastore.
//!
//! Mirrors Talos `etcd` controllers: each control-plane node runs an etcd
//! member as a static pod, and the member moves through a small join/bootstrap
//! lifecycle. We model the member spec (peer/client URLs, initial cluster) and
//! the state machine guarding bootstrap-vs-join.

use crate::error::{K8sError, Result};

/// Default etcd client port.
pub const ETCD_CLIENT_PORT: u16 = 2379;
/// Default etcd peer port.
pub const ETCD_PEER_PORT: u16 = 2380;

/// The lifecycle state of an etcd member on this node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtcdMemberState {
    /// No member configured yet.
    Uninitialized,
    /// This member is the first member, bootstrapping a new cluster.
    Bootstrapping,
    /// This member is joining an existing cluster.
    Joining,
    /// The member is part of a running cluster.
    Running,
    /// The member has been removed from the cluster.
    Removed,
}

impl EtcdMemberState {
    /// Whether the member is actively serving.
    pub fn is_active(self) -> bool {
        matches!(self, EtcdMemberState::Running)
    }
}

/// The etcd member spec for this node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EtcdSpec {
    /// The member name (usually the node name).
    pub name: String,
    /// The IP/host this member advertises.
    pub advertise_ip: String,
    /// `initial-cluster` entries: `(name, peer_url)` for every member.
    pub initial_cluster: Vec<(String, String)>,
    /// Current lifecycle state.
    pub state: EtcdMemberState,
}

impl EtcdSpec {
    /// Build an uninitialized spec for `name` advertising `ip`.
    pub fn new(name: impl Into<String>, advertise_ip: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let advertise_ip = advertise_ip.into();
        if name.trim().is_empty() {
            return Err(K8sError::InvalidConfig(
                "etcd member name empty".to_string(),
            ));
        }
        if advertise_ip.trim().is_empty() {
            return Err(K8sError::InvalidConfig(
                "etcd advertise IP empty".to_string(),
            ));
        }
        Ok(EtcdSpec {
            name,
            advertise_ip,
            initial_cluster: Vec::new(),
            state: EtcdMemberState::Uninitialized,
        })
    }

    /// This member's peer URL.
    pub fn peer_url(&self) -> String {
        format!("https://{}:{}", self.advertise_ip, ETCD_PEER_PORT)
    }

    /// This member's client URL.
    pub fn client_url(&self) -> String {
        format!("https://{}:{}", self.advertise_ip, ETCD_CLIENT_PORT)
    }

    /// Render the `--initial-cluster` flag value.
    pub fn initial_cluster_arg(&self) -> String {
        self.initial_cluster
            .iter()
            .map(|(n, url)| format!("{n}={url}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Bootstrap a brand-new cluster with this member as the sole initial
    /// member. Only valid from [`EtcdMemberState::Uninitialized`].
    pub fn bootstrap(&mut self) -> Result<()> {
        if self.state != EtcdMemberState::Uninitialized {
            return Err(K8sError::EtcdState(format!(
                "cannot bootstrap from {:?}",
                self.state
            )));
        }
        self.initial_cluster = vec![(self.name.clone(), self.peer_url())];
        self.state = EtcdMemberState::Bootstrapping;
        Ok(())
    }

    /// Join an existing cluster. `existing` is the current member set; this
    /// member is appended. Only valid from [`EtcdMemberState::Uninitialized`].
    pub fn join(&mut self, existing: &[(String, String)]) -> Result<()> {
        if self.state != EtcdMemberState::Uninitialized {
            return Err(K8sError::EtcdState(format!(
                "cannot join from {:?}",
                self.state
            )));
        }
        if existing.is_empty() {
            return Err(K8sError::EtcdState(
                "cannot join an empty cluster; bootstrap instead".to_string(),
            ));
        }
        let mut cluster: Vec<(String, String)> = existing.to_vec();
        cluster.push((self.name.clone(), self.peer_url()));
        self.initial_cluster = cluster;
        self.state = EtcdMemberState::Joining;
        Ok(())
    }

    /// Mark the member as running once it has reported healthy.
    pub fn promote_running(&mut self) -> Result<()> {
        match self.state {
            EtcdMemberState::Bootstrapping | EtcdMemberState::Joining => {
                self.state = EtcdMemberState::Running;
                Ok(())
            }
            other => Err(K8sError::EtcdState(format!(
                "cannot promote to Running from {other:?}"
            ))),
        }
    }

    /// Remove this member from the cluster.
    pub fn remove(&mut self) -> Result<()> {
        if !self.state.is_active() {
            return Err(K8sError::EtcdState(format!(
                "cannot remove member in state {:?}",
                self.state
            )));
        }
        self.state = EtcdMemberState::Removed;
        Ok(())
    }

    /// The `initial-cluster-state` flag value etcd expects.
    pub fn initial_cluster_state(&self) -> &'static str {
        match self.state {
            EtcdMemberState::Joining => "existing",
            _ => "new",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_render() {
        let s = EtcdSpec::new("cp-1", "10.0.0.1").unwrap();
        assert_eq!(s.peer_url(), "https://10.0.0.1:2380");
        assert_eq!(s.client_url(), "https://10.0.0.1:2379");
    }

    #[test]
    fn bootstrap_sets_single_member() {
        let mut s = EtcdSpec::new("cp-1", "10.0.0.1").unwrap();
        s.bootstrap().unwrap();
        assert_eq!(s.state, EtcdMemberState::Bootstrapping);
        assert_eq!(s.initial_cluster_arg(), "cp-1=https://10.0.0.1:2380");
        assert_eq!(s.initial_cluster_state(), "new");
        // Cannot bootstrap twice.
        assert!(s.bootstrap().is_err());
    }

    #[test]
    fn join_appends_to_existing() {
        let mut s = EtcdSpec::new("cp-2", "10.0.0.2").unwrap();
        let existing = vec![("cp-1".to_string(), "https://10.0.0.1:2380".to_string())];
        s.join(&existing).unwrap();
        assert_eq!(s.state, EtcdMemberState::Joining);
        assert_eq!(s.initial_cluster.len(), 2);
        assert_eq!(s.initial_cluster_state(), "existing");
    }

    #[test]
    fn join_empty_cluster_fails() {
        let mut s = EtcdSpec::new("cp-2", "10.0.0.2").unwrap();
        let err = s.join(&[]).unwrap_err();
        assert_eq!(err.kind(), "etcd_state");
    }

    #[test]
    fn promote_and_remove_lifecycle() {
        let mut s = EtcdSpec::new("cp-1", "10.0.0.1").unwrap();
        s.bootstrap().unwrap();
        s.promote_running().unwrap();
        assert!(s.state.is_active());
        s.remove().unwrap();
        assert_eq!(s.state, EtcdMemberState::Removed);
        // Cannot remove again.
        assert!(s.remove().is_err());
    }

    #[test]
    fn promote_from_uninitialized_fails() {
        let mut s = EtcdSpec::new("cp-1", "10.0.0.1").unwrap();
        assert!(s.promote_running().is_err());
    }
}
