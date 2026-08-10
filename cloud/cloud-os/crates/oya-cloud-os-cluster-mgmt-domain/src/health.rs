//! Cluster health checks (`pkg/cluster/check`).
//!
//! `talosctl health` runs an ordered list of checks against a cluster: every
//! expected node reports in, etcd is healthy on every control-plane node, all
//! members are known, control-plane static pods are running, and all nodes (and
//! Kubernetes Nodes) are Ready. This module models those checks as a
//! [`HealthCheck`] enum evaluated against a [`ClusterState`] snapshot, with a
//! [`HealthChecker`] that runs them in order and reports the first failure.

use crate::ClusterError;
use os_kernel::machine_type::MachineType;

/// Per-node observed health, as the health checker would scrape it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeHealth {
    /// Node IP / identity.
    pub ip: String,
    /// Role.
    pub machine_type: MachineType,
    /// Whether the node's apid answered (the node "reported in").
    pub apid_reachable: bool,
    /// Whether etcd is healthy on this node (control-plane only).
    pub etcd_healthy: bool,
    /// Whether the kubelet reports the node Ready.
    pub kubelet_ready: bool,
    /// Whether the corresponding Kubernetes Node object is Ready.
    pub k8s_node_ready: bool,
    /// Names of control-plane static pods reported running on this node.
    pub static_pods_running: Vec<String>,
}

impl NodeHealth {
    /// Construct a fully-healthy node health record for a given role.
    pub fn healthy(ip: impl Into<String>, machine_type: MachineType) -> Self {
        let static_pods = if machine_type.is_control_plane() {
            CONTROL_PLANE_STATIC_PODS
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };
        NodeHealth {
            ip: ip.into(),
            machine_type,
            apid_reachable: true,
            etcd_healthy: machine_type.is_control_plane(),
            kubelet_ready: true,
            k8s_node_ready: true,
            static_pods_running: static_pods,
        }
    }
}

/// The expected control-plane static pods (mirrors Talos's set).
pub const CONTROL_PLANE_STATIC_PODS: [&str; 3] = [
    "kube-apiserver",
    "kube-controller-manager",
    "kube-scheduler",
];

/// A snapshot of cluster state the health checks evaluate against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterState {
    /// The set of node IPs the cluster *expects* to exist.
    pub expected_members: Vec<String>,
    /// The observed health of each node that reported in.
    pub nodes: Vec<NodeHealth>,
}

impl ClusterState {
    /// Look up a node's health by IP.
    pub fn node(&self, ip: &str) -> Option<&NodeHealth> {
        self.nodes.iter().find(|n| n.ip == ip)
    }

    /// All control-plane node healths.
    pub fn control_plane_nodes(&self) -> impl Iterator<Item = &NodeHealth> {
        self.nodes
            .iter()
            .filter(|n| n.machine_type.is_control_plane())
    }
}

/// The individual checks `talosctl health` performs, in their run order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthCheck {
    /// Every expected member has reported in via apid.
    AllNodesReportIn,
    /// etcd is healthy on every control-plane node.
    EtcdHealthy,
    /// All cluster members are known (no missing/extra members).
    AllMembersKnown,
    /// Control-plane static pods are running on every control-plane node.
    ControlPlaneStaticPods,
    /// All kubelets report Ready.
    AllKubeletsReady,
    /// All Kubernetes Node objects are Ready.
    AllK8sNodesReady,
}

impl HealthCheck {
    /// The default ordered list of checks Talos runs.
    pub fn default_order() -> [HealthCheck; 6] {
        [
            HealthCheck::AllNodesReportIn,
            HealthCheck::EtcdHealthy,
            HealthCheck::AllMembersKnown,
            HealthCheck::ControlPlaneStaticPods,
            HealthCheck::AllKubeletsReady,
            HealthCheck::AllK8sNodesReady,
        ]
    }

    /// A stable, human-readable name for the check.
    pub fn name(self) -> &'static str {
        match self {
            HealthCheck::AllNodesReportIn => "all nodes report in",
            HealthCheck::EtcdHealthy => "etcd healthy",
            HealthCheck::AllMembersKnown => "all members known",
            HealthCheck::ControlPlaneStaticPods => "control-plane static pods running",
            HealthCheck::AllKubeletsReady => "all kubelets ready",
            HealthCheck::AllK8sNodesReady => "all Kubernetes nodes ready",
        }
    }

    /// Evaluate this check against a cluster state.
    pub fn evaluate(self, state: &ClusterState) -> Result<(), ClusterError> {
        match self {
            HealthCheck::AllNodesReportIn => {
                for ip in &state.expected_members {
                    match state.node(ip) {
                        None => {
                            return Err(ClusterError::unhealthy(format!(
                                "node {ip} has not reported in"
                            )));
                        }
                        Some(n) if !n.apid_reachable => {
                            return Err(ClusterError::unhealthy(format!(
                                "node {ip} apid is not reachable"
                            )));
                        }
                        Some(_) => {}
                    }
                }
                Ok(())
            }
            HealthCheck::EtcdHealthy => {
                let cp: Vec<&NodeHealth> = state.control_plane_nodes().collect();
                if cp.is_empty() {
                    return Err(ClusterError::unhealthy(
                        "no control-plane nodes present for etcd",
                    ));
                }
                for n in cp {
                    if !n.etcd_healthy {
                        return Err(ClusterError::unhealthy(format!(
                            "etcd unhealthy on {}",
                            n.ip
                        )));
                    }
                }
                Ok(())
            }
            HealthCheck::AllMembersKnown => {
                let mut expected = state.expected_members.clone();
                expected.sort();
                let mut observed: Vec<String> = state.nodes.iter().map(|n| n.ip.clone()).collect();
                observed.sort();
                if expected != observed {
                    return Err(ClusterError::unhealthy(format!(
                        "membership mismatch: expected {expected:?}, observed {observed:?}"
                    )));
                }
                Ok(())
            }
            HealthCheck::ControlPlaneStaticPods => {
                for n in state.control_plane_nodes() {
                    for pod in CONTROL_PLANE_STATIC_PODS {
                        if !n.static_pods_running.iter().any(|p| p == pod) {
                            return Err(ClusterError::unhealthy(format!(
                                "static pod {pod} not running on {}",
                                n.ip
                            )));
                        }
                    }
                }
                Ok(())
            }
            HealthCheck::AllKubeletsReady => {
                for n in &state.nodes {
                    if !n.kubelet_ready {
                        return Err(ClusterError::unhealthy(format!(
                            "kubelet not ready on {}",
                            n.ip
                        )));
                    }
                }
                Ok(())
            }
            HealthCheck::AllK8sNodesReady => {
                for n in &state.nodes {
                    if !n.k8s_node_ready {
                        return Err(ClusterError::unhealthy(format!(
                            "Kubernetes node {} not ready",
                            n.ip
                        )));
                    }
                }
                Ok(())
            }
        }
    }
}

/// The outcome of a single check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    /// Which check ran.
    pub check: HealthCheck,
    /// `None` if it passed, `Some(reason)` if it failed.
    pub failure: Option<String>,
}

impl CheckResult {
    /// Whether the check passed.
    pub fn passed(&self) -> bool {
        self.failure.is_none()
    }
}

/// Runs an ordered set of [`HealthCheck`]s against a cluster state.
#[derive(Debug, Clone)]
pub struct HealthChecker {
    checks: Vec<HealthCheck>,
}

impl Default for HealthChecker {
    fn default() -> Self {
        HealthChecker {
            checks: HealthCheck::default_order().to_vec(),
        }
    }
}

impl HealthChecker {
    /// A checker running the default Talos check order.
    pub fn new() -> Self {
        Self::default()
    }

    /// A checker running a custom subset/order of checks.
    pub fn with_checks(checks: Vec<HealthCheck>) -> Self {
        HealthChecker { checks }
    }

    /// Run all checks, stopping at the first failure (like `talosctl health`),
    /// returning `Ok(())` if the cluster is healthy.
    pub fn run(&self, state: &ClusterState) -> Result<(), ClusterError> {
        for check in &self.checks {
            check.evaluate(state)?;
        }
        Ok(())
    }

    /// Run every check regardless of failures and return all results, useful
    /// for a full health report.
    pub fn run_all(&self, state: &ClusterState) -> Vec<CheckResult> {
        self.checks
            .iter()
            .map(|&check| CheckResult {
                check,
                failure: check.evaluate(state).err().map(|e| e.to_string()),
            })
            .collect()
    }

    /// Whether the cluster is fully healthy.
    pub fn is_healthy(&self, state: &ClusterState) -> bool {
        self.run(state).is_ok()
    }
}
