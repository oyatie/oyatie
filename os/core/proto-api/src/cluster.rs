//! The `ClusterService` API surface (cluster health checks).
//!
//! Mirrors `pkg/machinery/api/cluster/cluster.proto`: the `HealthCheck`
//! streaming call that walks a fixed sequence of checks (etcd members,
//! control-plane static pods, node readiness, k8s components) and emits a
//! progress message per stage. Modeled here as a state machine over a
//! [`ClusterHealthBackend`].

use crate::common::{ApiError, RequestContext};
use os_kernel::role::Role;

/// A node's membership/role within the cluster health view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterNode {
    /// The node's name/hostname.
    pub name: String,
    /// Whether the node is a control-plane member.
    pub control_plane: bool,
    /// Whether the kubelet reports the node Ready.
    pub ready: bool,
}

/// A `HealthCheck` request, mirroring `cluster.HealthCheckRequest`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HealthCheckRequest {
    /// Expected control-plane node names.
    pub control_plane_nodes: Vec<String>,
    /// Expected worker node names.
    pub worker_nodes: Vec<String>,
    /// Wait for all k8s components to be ready, not just etcd.
    pub wait_for_kubernetes: bool,
}

impl HealthCheckRequest {
    /// All expected node names.
    pub fn all_nodes(&self) -> Vec<String> {
        let mut v = self.control_plane_nodes.clone();
        v.extend(self.worker_nodes.iter().cloned());
        v
    }
}

/// The ordered checks the health stream walks, mirroring the Talos
/// `cluster/check` default check set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthCheck {
    /// etcd has the expected number of healthy members.
    EtcdMembers,
    /// etcd has no alarms (no NOSPACE/CORRUPT).
    EtcdAlarms,
    /// All apid endpoints are reachable.
    ApidReady,
    /// Control-plane static pods are running.
    ControlPlaneStaticPods,
    /// All expected nodes have registered with the API server.
    AllNodesRegistered,
    /// All nodes report Ready.
    AllNodesReady,
    /// kube-system pods (coredns, etc.) are ready.
    KubeSystemReady,
}

impl HealthCheck {
    /// The default sequence of checks. The k8s-specific tail is only run when
    /// `wait_for_kubernetes` is set.
    pub fn sequence(wait_for_kubernetes: bool) -> Vec<HealthCheck> {
        let mut seq = vec![
            HealthCheck::EtcdMembers,
            HealthCheck::EtcdAlarms,
            HealthCheck::ApidReady,
            HealthCheck::ControlPlaneStaticPods,
        ];
        if wait_for_kubernetes {
            seq.extend([
                HealthCheck::AllNodesRegistered,
                HealthCheck::AllNodesReady,
                HealthCheck::KubeSystemReady,
            ]);
        }
        seq
    }

    /// A short human-readable label for progress messages.
    pub fn label(self) -> &'static str {
        match self {
            HealthCheck::EtcdMembers => "etcd members",
            HealthCheck::EtcdAlarms => "etcd alarms",
            HealthCheck::ApidReady => "apid endpoints",
            HealthCheck::ControlPlaneStaticPods => "control-plane static pods",
            HealthCheck::AllNodesRegistered => "all nodes registered",
            HealthCheck::AllNodesReady => "all nodes ready",
            HealthCheck::KubeSystemReady => "kube-system pods ready",
        }
    }
}

/// A single progress message emitted by the health stream, mirroring
/// `cluster.HealthCheckProgress`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthCheckProgress {
    /// The check this message refers to.
    pub check: HealthCheck,
    /// Whether the check passed.
    pub passed: bool,
    /// A human-readable message (e.g. why it failed).
    pub message: String,
}

/// The OS/cluster state the health checker consults, behind a trait so tests
/// can supply an in-memory cluster instead of querying etcd/Kubernetes.
pub trait ClusterHealthBackend {
    /// Number of healthy etcd members vs. expected.
    fn etcd_members(&self) -> (usize, usize);

    /// Whether etcd has any active alarms.
    fn etcd_has_alarms(&self) -> bool;

    /// Whether all apid endpoints answer.
    fn apid_ready(&self) -> bool;

    /// Whether the control-plane static pods are up.
    fn control_plane_pods_ready(&self) -> bool;

    /// The nodes known to the cluster.
    fn nodes(&self) -> Vec<ClusterNode>;

    /// Whether kube-system workloads are ready.
    fn kube_system_ready(&self) -> bool;
}

/// The cluster health-check service.
pub struct ClusterService<B: ClusterHealthBackend> {
    backend: B,
}

impl<B: ClusterHealthBackend> ClusterService<B> {
    /// Wrap a backend.
    pub fn new(backend: B) -> Self {
        ClusterService { backend }
    }

    /// Access the underlying backend.
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Evaluate a single check against the backend, given the request.
    fn evaluate(&self, check: HealthCheck, req: &HealthCheckRequest) -> HealthCheckProgress {
        let (passed, message) = match check {
            HealthCheck::EtcdMembers => {
                let (have, want) = self.backend.etcd_members();
                (
                    have >= want && want > 0,
                    format!("{have}/{want} healthy etcd members"),
                )
            }
            HealthCheck::EtcdAlarms => {
                let alarms = self.backend.etcd_has_alarms();
                (
                    !alarms,
                    if alarms {
                        "etcd alarms present".into()
                    } else {
                        "no alarms".into()
                    },
                )
            }
            HealthCheck::ApidReady => {
                let ok = self.backend.apid_ready();
                (
                    ok,
                    if ok {
                        "all apid endpoints reachable".into()
                    } else {
                        "apid unreachable".into()
                    },
                )
            }
            HealthCheck::ControlPlaneStaticPods => {
                let ok = self.backend.control_plane_pods_ready();
                (
                    ok,
                    if ok {
                        "static pods running".into()
                    } else {
                        "static pods not ready".into()
                    },
                )
            }
            HealthCheck::AllNodesRegistered => {
                let nodes = self.backend.nodes();
                let registered: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
                let missing: Vec<String> = req
                    .all_nodes()
                    .into_iter()
                    .filter(|n| !registered.contains(&n.as_str()))
                    .collect();
                (
                    missing.is_empty(),
                    if missing.is_empty() {
                        "all nodes registered".into()
                    } else {
                        format!("missing nodes: {}", missing.join(", "))
                    },
                )
            }
            HealthCheck::AllNodesReady => {
                let not_ready: Vec<String> = self
                    .backend
                    .nodes()
                    .into_iter()
                    .filter(|n| !n.ready)
                    .map(|n| n.name)
                    .collect();
                (
                    not_ready.is_empty(),
                    if not_ready.is_empty() {
                        "all nodes ready".into()
                    } else {
                        format!("not ready: {}", not_ready.join(", "))
                    },
                )
            }
            HealthCheck::KubeSystemReady => {
                let ok = self.backend.kube_system_ready();
                (
                    ok,
                    if ok {
                        "kube-system ready".into()
                    } else {
                        "kube-system not ready".into()
                    },
                )
            }
        };
        HealthCheckProgress {
            check,
            passed,
            message,
        }
    }

    /// `HealthCheck`: produce the full progress stream. Stops emitting at the
    /// first failed check (mirroring the streaming check's short-circuit), with
    /// the failed check as the final message.
    pub fn health_check(
        &self,
        ctx: &RequestContext,
        req: &HealthCheckRequest,
    ) -> Result<Vec<HealthCheckProgress>, ApiError> {
        ctx.authorize(Role::Reader)?;
        let mut out = Vec::new();
        for check in HealthCheck::sequence(req.wait_for_kubernetes) {
            let progress = self.evaluate(check, req);
            let failed = !progress.passed;
            out.push(progress);
            if failed {
                break;
            }
        }
        Ok(out)
    }

    /// Whether the cluster is fully healthy for the given request.
    pub fn is_healthy(&self, req: &HealthCheckRequest) -> bool {
        HealthCheck::sequence(req.wait_for_kubernetes)
            .into_iter()
            .all(|c| self.evaluate(c, req).passed)
    }
}

/// An in-memory cluster for tests.
#[derive(Debug, Clone)]
pub struct InMemoryCluster {
    /// Healthy etcd members.
    pub etcd_healthy: usize,
    /// Expected etcd members.
    pub etcd_expected: usize,
    /// Whether etcd has alarms.
    pub alarms: bool,
    /// Whether apid endpoints answer.
    pub apid: bool,
    /// Whether static pods are up.
    pub static_pods: bool,
    /// Whether kube-system is ready.
    pub kube_system: bool,
    /// The known nodes.
    pub nodes: Vec<ClusterNode>,
}

impl InMemoryCluster {
    /// A fully-healthy three-node control plane.
    pub fn healthy_three_node() -> Self {
        InMemoryCluster {
            etcd_healthy: 3,
            etcd_expected: 3,
            alarms: false,
            apid: true,
            static_pods: true,
            kube_system: true,
            nodes: ["cp-1", "cp-2", "cp-3"]
                .into_iter()
                .map(|n| ClusterNode {
                    name: n.into(),
                    control_plane: true,
                    ready: true,
                })
                .collect(),
        }
    }
}

impl ClusterHealthBackend for InMemoryCluster {
    fn etcd_members(&self) -> (usize, usize) {
        (self.etcd_healthy, self.etcd_expected)
    }
    fn etcd_has_alarms(&self) -> bool {
        self.alarms
    }
    fn apid_ready(&self) -> bool {
        self.apid
    }
    fn control_plane_pods_ready(&self) -> bool {
        self.static_pods
    }
    fn nodes(&self) -> Vec<ClusterNode> {
        self.nodes.clone()
    }
    fn kube_system_ready(&self) -> bool {
        self.kube_system
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Code;

    fn req(wait_k8s: bool) -> HealthCheckRequest {
        HealthCheckRequest {
            control_plane_nodes: vec!["cp-1".into(), "cp-2".into(), "cp-3".into()],
            worker_nodes: vec![],
            wait_for_kubernetes: wait_k8s,
        }
    }

    #[test]
    fn sequence_extends_with_kubernetes() {
        assert_eq!(HealthCheck::sequence(false).len(), 4);
        assert_eq!(HealthCheck::sequence(true).len(), 7);
        assert_eq!(HealthCheck::EtcdMembers.label(), "etcd members");
    }

    #[test]
    fn healthy_cluster_passes_all() {
        let svc = ClusterService::new(InMemoryCluster::healthy_three_node());
        let progress = svc
            .health_check(&RequestContext::admin_local(), &req(true))
            .unwrap();
        assert_eq!(progress.len(), 7);
        assert!(progress.iter().all(|p| p.passed));
        assert!(svc.is_healthy(&req(true)));
    }

    #[test]
    fn short_circuits_on_first_failure() {
        let mut cluster = InMemoryCluster::healthy_three_node();
        cluster.etcd_healthy = 1; // lost quorum
        let svc = ClusterService::new(cluster);
        let progress = svc
            .health_check(&RequestContext::admin_local(), &req(true))
            .unwrap();
        // First check fails, stream stops there.
        assert_eq!(progress.len(), 1);
        assert_eq!(progress[0].check, HealthCheck::EtcdMembers);
        assert!(!progress[0].passed);
        assert!(!svc.is_healthy(&req(true)));
    }

    #[test]
    fn detects_missing_and_not_ready_nodes() {
        let mut cluster = InMemoryCluster::healthy_three_node();
        cluster.nodes.pop(); // cp-3 missing
        let svc = ClusterService::new(cluster);
        let progress = svc
            .health_check(&RequestContext::admin_local(), &req(true))
            .unwrap();
        let last = progress.last().unwrap();
        assert_eq!(last.check, HealthCheck::AllNodesRegistered);
        assert!(!last.passed);
        assert!(last.message.contains("cp-3"));
    }

    #[test]
    fn not_ready_node_fails_readiness() {
        let mut cluster = InMemoryCluster::healthy_three_node();
        cluster.nodes[2].ready = false;
        let svc = ClusterService::new(cluster);
        let progress = svc
            .health_check(&RequestContext::admin_local(), &req(true))
            .unwrap();
        let last = progress.last().unwrap();
        assert_eq!(last.check, HealthCheck::AllNodesReady);
        assert!(last.message.contains("cp-3"));
    }

    #[test]
    fn health_check_requires_read_role() {
        let svc = ClusterService::new(InMemoryCluster::healthy_three_node());
        let nobody = RequestContext::with_roles(os_kernel::role::RoleSet::new());
        assert_eq!(
            svc.health_check(&nobody, &req(false)).unwrap_err().code,
            Code::PermissionDenied
        );
    }
}
