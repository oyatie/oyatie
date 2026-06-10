//! Network-readiness and etcd-readiness conditions.
//!
//! Mirrors the network/etcd readiness predicates the Talos boot sequence waits
//! on. In Talos these are expressed as resource conditions:
//! `network.StatusReady` (the `NetworkStatus` resource flips `AddressReady` /
//! `ConnectivityReady` / `HostnameReady` / `EtcFilesReady`), and etcd readiness
//! is "the local etcd member is a healthy voting member of the quorum".
//!
//! We model the network status as a bitset-ish struct and etcd as a small
//! membership/quorum view, both behind probe traits with in-memory impls.

use crate::condition::{Condition, Poll};

/// The readiness flags of Talos's `NetworkStatus` resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NetworkStatus {
    /// At least one routable address is configured.
    pub address_ready: bool,
    /// Default route / outbound connectivity is up.
    pub connectivity_ready: bool,
    /// Hostname has been resolved/applied.
    pub hostname_ready: bool,
    /// `/etc/hosts` and `/etc/resolv.conf` have been rendered.
    pub etc_files_ready: bool,
}

impl NetworkStatus {
    /// A fully-ready network.
    pub fn all_ready() -> Self {
        NetworkStatus {
            address_ready: true,
            connectivity_ready: true,
            hostname_ready: true,
            etc_files_ready: true,
        }
    }

    /// True when every readiness flag is set (Talos's `StatusReady`).
    pub fn is_ready(&self) -> bool {
        self.address_ready && self.connectivity_ready && self.hostname_ready && self.etc_files_ready
    }

    /// Human-readable list of the components that are not yet ready.
    pub fn pending_components(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if !self.address_ready {
            out.push("address");
        }
        if !self.connectivity_ready {
            out.push("connectivity");
        }
        if !self.hostname_ready {
            out.push("hostname");
        }
        if !self.etc_files_ready {
            out.push("etc-files");
        }
        out
    }
}

/// OS boundary: read the current [`NetworkStatus`].
pub trait NetworkProbe {
    /// Snapshot the current network status.
    fn network_status(&self) -> NetworkStatus;
}

/// In-memory [`NetworkProbe`].
#[derive(Debug, Default, Clone)]
pub struct InMemoryNetwork {
    status: NetworkStatus,
}

impl InMemoryNetwork {
    /// Network with nothing ready.
    pub fn new() -> Self {
        InMemoryNetwork {
            status: NetworkStatus::default(),
        }
    }

    /// Replace the status.
    pub fn set(&mut self, status: NetworkStatus) {
        self.status = status;
    }

    /// Mutable access to the status for incremental updates.
    pub fn status_mut(&mut self) -> &mut NetworkStatus {
        &mut self.status
    }
}

impl NetworkProbe for InMemoryNetwork {
    fn network_status(&self) -> NetworkStatus {
        self.status
    }
}

/// Wait for the network to be fully ready (`network.StatusReady`).
pub struct WaitForNetworkReady<'a, P: NetworkProbe> {
    probe: &'a P,
}

impl<'a, P: NetworkProbe> WaitForNetworkReady<'a, P> {
    /// Construct a network-ready condition.
    pub fn new(probe: &'a P) -> Self {
        WaitForNetworkReady { probe }
    }
}

impl<P: NetworkProbe> Condition for WaitForNetworkReady<'_, P> {
    fn poll(&self) -> Poll {
        if self.probe.network_status().is_ready() {
            Poll::Ready
        } else {
            Poll::Pending(self.describe())
        }
    }

    fn describe(&self) -> String {
        format!(
            "network to be ready (pending: {:?})",
            self.probe.network_status().pending_components()
        )
    }
}

/// A view of the local etcd member's place in the cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EtcdStatus {
    /// The local member has joined the cluster and has a member id.
    pub member_joined: bool,
    /// The local member is a voting (non-learner) member.
    pub is_voter: bool,
    /// A leader currently exists (the cluster has quorum).
    pub has_leader: bool,
    /// Total number of members; quorum needs a strict majority alive.
    pub members: u32,
    /// Number of members currently reachable/healthy.
    pub healthy_members: u32,
}

impl EtcdStatus {
    /// True when the cluster has quorum: a strict majority is healthy.
    pub fn has_quorum(&self) -> bool {
        self.members > 0 && self.healthy_members * 2 > self.members
    }

    /// True when the local etcd is ready to serve: joined, voting, quorum,
    /// and a leader exists.
    pub fn is_ready(&self) -> bool {
        self.member_joined && self.is_voter && self.has_leader && self.has_quorum()
    }
}

/// OS boundary: read the current [`EtcdStatus`].
pub trait EtcdProbe {
    /// Snapshot etcd status.
    fn etcd_status(&self) -> EtcdStatus;
}

/// In-memory [`EtcdProbe`].
#[derive(Debug, Default, Clone)]
pub struct InMemoryEtcd {
    status: EtcdStatus,
}

impl InMemoryEtcd {
    /// Etcd with nothing ready.
    pub fn new() -> Self {
        InMemoryEtcd {
            status: EtcdStatus::default(),
        }
    }

    /// Replace the status.
    pub fn set(&mut self, status: EtcdStatus) {
        self.status = status;
    }
}

impl EtcdProbe for InMemoryEtcd {
    fn etcd_status(&self) -> EtcdStatus {
        self.status
    }
}

/// Wait for the local etcd member to be ready (joined, voting, quorum).
pub struct WaitForEtcdReady<'a, P: EtcdProbe> {
    probe: &'a P,
}

impl<'a, P: EtcdProbe> WaitForEtcdReady<'a, P> {
    /// Construct an etcd-ready condition.
    pub fn new(probe: &'a P) -> Self {
        WaitForEtcdReady { probe }
    }
}

impl<P: EtcdProbe> Condition for WaitForEtcdReady<'_, P> {
    fn poll(&self) -> Poll {
        if self.probe.etcd_status().is_ready() {
            Poll::Ready
        } else {
            Poll::Pending(self.describe())
        }
    }

    fn describe(&self) -> String {
        "etcd to be a healthy voting member with quorum".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_pending_until_all_flags() {
        let mut net = InMemoryNetwork::new();
        {
            let cond = WaitForNetworkReady::new(&net);
            assert!(matches!(cond.poll(), Poll::Pending(_)));
        }
        net.set(NetworkStatus::all_ready());
        let cond = WaitForNetworkReady::new(&net);
        assert_eq!(cond.poll(), Poll::Ready);
    }

    #[test]
    fn network_pending_components_listed() {
        let mut net = InMemoryNetwork::new();
        net.status_mut().address_ready = true;
        net.status_mut().connectivity_ready = true;
        let pending = net.network_status().pending_components();
        assert_eq!(pending, vec!["hostname", "etc-files"]);
    }

    #[test]
    fn etcd_quorum_math() {
        // 3 members, 2 healthy -> quorum (2*2=4 > 3).
        let s = EtcdStatus {
            members: 3,
            healthy_members: 2,
            ..Default::default()
        };
        assert!(s.has_quorum());
        // 3 members, 1 healthy -> no quorum.
        let s = EtcdStatus {
            members: 3,
            healthy_members: 1,
            ..Default::default()
        };
        assert!(!s.has_quorum());
        // 0 members -> no quorum.
        assert!(!EtcdStatus::default().has_quorum());
    }

    #[test]
    fn etcd_ready_requires_all() {
        let mut etcd = InMemoryEtcd::new();
        {
            let cond = WaitForEtcdReady::new(&etcd);
            assert!(matches!(cond.poll(), Poll::Pending(_)));
        }
        etcd.set(EtcdStatus {
            member_joined: true,
            is_voter: true,
            has_leader: true,
            members: 3,
            healthy_members: 2,
        });
        let cond = WaitForEtcdReady::new(&etcd);
        assert_eq!(cond.poll(), Poll::Ready);
    }

    #[test]
    fn etcd_learner_not_ready() {
        let etcd = {
            let mut e = InMemoryEtcd::new();
            e.set(EtcdStatus {
                member_joined: true,
                is_voter: false, // learner
                has_leader: true,
                members: 3,
                healthy_members: 3,
            });
            e
        };
        let cond = WaitForEtcdReady::new(&etcd);
        assert!(matches!(cond.poll(), Poll::Pending(_)));
    }
}
