//! Node drain / cordon / uncordon for upgrades.
//!
//! Before Talos upgrades a worker that participates in the cluster it cordons
//! the node (marks it unschedulable) and evicts its pods, honoring
//! PodDisruptionBudgets, so workloads reschedule elsewhere first. After the
//! node comes back it is uncordoned. This mirrors the `talosctl upgrade
//! --stage`/`kubectl drain` behavior implemented in
//! `pkg/cluster/kubernetes` and the machined upgrade preflight.
//!
//! The Kubernetes API surface used (cordon, list pods, evict) is the
//! [`NodeApi`] trait; [`InMemoryNodeApi`] is the deterministic test
//! implementation.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// Scheduling/eviction status of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    /// Schedulable and accepting pods.
    Ready,
    /// Cordoned: still running pods but marked unschedulable.
    Cordoned,
    /// Cordoned and all evictable pods removed.
    Drained,
}

/// A pod tracked for drain purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pod {
    /// Pod name (namespaced uniqueness assumed for the model).
    pub name: String,
    /// Whether the pod is a DaemonSet member (DaemonSet pods are not evicted).
    pub daemonset: bool,
    /// Whether the pod is mirror/static (run by the kubelet, never evicted).
    pub mirror: bool,
}

impl Pod {
    /// A plain evictable workload pod.
    pub fn workload(name: &str) -> Self {
        Pod {
            name: name.to_string(),
            daemonset: false,
            mirror: false,
        }
    }

    /// A DaemonSet pod (skipped by drain).
    pub fn daemonset(name: &str) -> Self {
        Pod {
            name: name.to_string(),
            daemonset: true,
            mirror: false,
        }
    }

    /// A static/mirror pod (skipped by drain).
    pub fn mirror(name: &str) -> Self {
        Pod {
            name: name.to_string(),
            daemonset: false,
            mirror: true,
        }
    }

    /// Whether drain is allowed to evict this pod.
    pub fn is_evictable(&self) -> bool {
        !self.daemonset && !self.mirror
    }
}

/// A PodDisruptionBudget constraint: at least `min_available` matching pods must
/// stay running across the cluster, so eviction is blocked once the count drops
/// to the budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodDisruption {
    /// Names of pods the budget applies to.
    pub pods: Vec<String>,
    /// Minimum number that must remain available.
    pub min_available: usize,
}

/// Options governing a drain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrainOptions {
    /// Evict DaemonSet pods too (default: false, matching kubectl).
    pub ignore_daemonsets: bool,
    /// Force-evict even if PodDisruptionBudgets would block.
    pub force: bool,
    /// Maximum eviction attempts before giving up (models the retry/timeout).
    pub max_attempts: u32,
}

impl Default for DrainOptions {
    fn default() -> Self {
        DrainOptions {
            ignore_daemonsets: true,
            force: false,
            max_attempts: 20,
        }
    }
}

/// Errors raised by the drain controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrainError {
    /// The node does not exist in the API.
    NodeNotFound(String),
    /// Eviction is blocked by a PodDisruptionBudget and `force` is not set.
    DisruptionBudgetBlocked(String),
    /// Drain exhausted its attempt budget without fully evicting.
    Timeout,
}

impl fmt::Display for DrainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DrainError::NodeNotFound(n) => write!(f, "node not found: {n}"),
            DrainError::DisruptionBudgetBlocked(p) => {
                write!(f, "eviction of '{p}' blocked by disruption budget")
            }
            DrainError::Timeout => write!(f, "drain timed out before completion"),
        }
    }
}

/// The Kubernetes API surface drain needs.
pub trait NodeApi {
    /// Mark a node unschedulable.
    fn cordon(&mut self, node: &str) -> Result<(), DrainError>;

    /// Mark a node schedulable again.
    fn uncordon(&mut self, node: &str) -> Result<(), DrainError>;

    /// Pods currently scheduled on a node.
    fn pods_on(&self, node: &str) -> Result<Vec<Pod>, DrainError>;

    /// Evict a single pod by name from a node. Returns whether the disruption
    /// budget permitted it.
    fn evict(&mut self, node: &str, pod: &str) -> Result<bool, DrainError>;

    /// The schedulable status of a node.
    fn status(&self, node: &str) -> Result<NodeStatus, DrainError>;
}

/// In-memory Kubernetes node API for tests.
#[derive(Debug, Default, Clone)]
pub struct InMemoryNodeApi {
    nodes: BTreeMap<String, NodeStatus>,
    pods: BTreeMap<String, Vec<Pod>>,
    budgets: Vec<PodDisruption>,
    /// Count of pods globally available per budget-tracked pod name.
    available: BTreeMap<String, bool>,
}

impl InMemoryNodeApi {
    /// An empty cluster.
    pub fn new() -> Self {
        InMemoryNodeApi::default()
    }

    /// Register a node with its pods, initially Ready.
    pub fn add_node(&mut self, node: &str, pods: Vec<Pod>) {
        for p in &pods {
            self.available.insert(p.name.clone(), true);
        }
        self.pods.insert(node.to_string(), pods);
        self.nodes.insert(node.to_string(), NodeStatus::Ready);
    }

    /// Register a PodDisruptionBudget.
    pub fn add_budget(&mut self, budget: PodDisruption) {
        self.budgets.push(budget);
    }

    fn available_count(&self, names: &[String]) -> usize {
        names
            .iter()
            .filter(|n| self.available.get(*n).copied().unwrap_or(false))
            .count()
    }

    /// Whether evicting `pod` would violate any disruption budget.
    fn evict_blocked(&self, pod: &str) -> bool {
        for b in &self.budgets {
            if b.pods.iter().any(|p| p == pod) {
                // After eviction, available would drop by one.
                let after = self.available_count(&b.pods).saturating_sub(1);
                if after < b.min_available {
                    return true;
                }
            }
        }
        false
    }
}

impl NodeApi for InMemoryNodeApi {
    fn cordon(&mut self, node: &str) -> Result<(), DrainError> {
        let s = self
            .nodes
            .get_mut(node)
            .ok_or_else(|| DrainError::NodeNotFound(node.to_string()))?;
        if *s == NodeStatus::Ready {
            *s = NodeStatus::Cordoned;
        }
        Ok(())
    }

    fn uncordon(&mut self, node: &str) -> Result<(), DrainError> {
        let s = self
            .nodes
            .get_mut(node)
            .ok_or_else(|| DrainError::NodeNotFound(node.to_string()))?;
        *s = NodeStatus::Ready;
        Ok(())
    }

    fn pods_on(&self, node: &str) -> Result<Vec<Pod>, DrainError> {
        self.pods
            .get(node)
            .cloned()
            .ok_or_else(|| DrainError::NodeNotFound(node.to_string()))
    }

    fn evict(&mut self, node: &str, pod: &str) -> Result<bool, DrainError> {
        if !self.nodes.contains_key(node) {
            return Err(DrainError::NodeNotFound(node.to_string()));
        }
        if self.evict_blocked(pod) {
            return Ok(false);
        }
        if let Some(list) = self.pods.get_mut(node) {
            list.retain(|p| p.name != pod);
        }
        self.available.insert(pod.to_string(), false);
        Ok(true)
    }

    fn status(&self, node: &str) -> Result<NodeStatus, DrainError> {
        self.nodes
            .get(node)
            .copied()
            .ok_or_else(|| DrainError::NodeNotFound(node.to_string()))
    }
}

/// The state of a drain operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainState {
    /// Not started.
    Pending,
    /// Node cordoned, evicting pods.
    Evicting,
    /// All evictable pods removed.
    Drained,
    /// Node uncordoned after upgrade.
    Uncordoned,
}

/// Drives cordon -> evict -> (later) uncordon for a single node.
#[derive(Debug)]
pub struct DrainController {
    node: String,
    state: DrainState,
    opts: DrainOptions,
    evicted: Vec<String>,
    skipped: Vec<String>,
}

impl DrainController {
    /// A controller for `node` with the given options.
    pub fn new(node: &str, opts: DrainOptions) -> Self {
        DrainController {
            node: node.to_string(),
            state: DrainState::Pending,
            opts,
            evicted: Vec::new(),
            skipped: Vec::new(),
        }
    }

    /// The current drain state.
    pub fn state(&self) -> DrainState {
        self.state
    }

    /// Pod names actually evicted, in order.
    pub fn evicted(&self) -> &[String] {
        &self.evicted
    }

    /// Pod names skipped (DaemonSet/mirror).
    pub fn skipped(&self) -> &[String] {
        &self.skipped
    }

    /// Cordon the node and evict all evictable pods, honoring options.
    pub fn drain<A: NodeApi>(&mut self, api: &mut A) -> Result<DrainState, DrainError> {
        api.cordon(&self.node)?;
        self.state = DrainState::Evicting;

        let mut attempts = 0u32;
        loop {
            let pods = api.pods_on(&self.node)?;
            let pending: Vec<Pod> = pods
                .into_iter()
                .filter(|p| {
                    if p.mirror {
                        return false;
                    }
                    if p.daemonset && self.opts.ignore_daemonsets {
                        return false;
                    }
                    p.is_evictable() || (p.daemonset && !self.opts.ignore_daemonsets)
                })
                .collect();

            if pending.is_empty() {
                break;
            }

            attempts += 1;
            if attempts > self.opts.max_attempts {
                return Err(DrainError::Timeout);
            }

            let mut progressed = false;
            for p in &pending {
                let ok = api.evict(&self.node, &p.name)?;
                if ok {
                    self.evicted.push(p.name.clone());
                    progressed = true;
                } else if self.opts.force {
                    // Force path: still record but the API refused; treat as
                    // blocked permanently.
                    return Err(DrainError::DisruptionBudgetBlocked(p.name.clone()));
                }
            }

            if !progressed {
                // Nothing could be evicted this round -> blocked by PDB.
                let blocked = pending.first().map(|p| p.name.clone()).unwrap_or_default();
                return Err(DrainError::DisruptionBudgetBlocked(blocked));
            }
        }

        // Record any skipped pods for visibility.
        for p in api.pods_on(&self.node)? {
            if !p.is_evictable() {
                self.skipped.push(p.name);
            }
        }

        self.state = DrainState::Drained;
        Ok(self.state)
    }

    /// Uncordon the node after the upgrade completes.
    pub fn uncordon<A: NodeApi>(&mut self, api: &mut A) -> Result<DrainState, DrainError> {
        api.uncordon(&self.node)?;
        self.state = DrainState::Uncordoned;
        Ok(self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cluster() -> InMemoryNodeApi {
        let mut api = InMemoryNodeApi::new();
        api.add_node(
            "worker-1",
            alloc::vec![
                Pod::workload("nginx-a"),
                Pod::workload("nginx-b"),
                Pod::daemonset("kube-proxy"),
                Pod::mirror("static-pod"),
            ],
        );
        api
    }

    #[test]
    fn pod_evictability() {
        assert!(Pod::workload("x").is_evictable());
        assert!(!Pod::daemonset("x").is_evictable());
        assert!(!Pod::mirror("x").is_evictable());
    }

    #[test]
    fn drain_evicts_only_workloads_and_cordons() {
        let mut api = cluster();
        let mut ctrl = DrainController::new("worker-1", DrainOptions::default());
        let state = ctrl.drain(&mut api).unwrap();
        assert_eq!(state, DrainState::Drained);
        assert_eq!(api.status("worker-1").unwrap(), NodeStatus::Cordoned);

        let mut evicted = ctrl.evicted().to_vec();
        evicted.sort();
        assert_eq!(
            evicted,
            alloc::vec!["nginx-a".to_string(), "nginx-b".to_string()]
        );

        // DaemonSet + mirror remain.
        let remaining: Vec<String> = api
            .pods_on("worker-1")
            .unwrap()
            .into_iter()
            .map(|p| p.name)
            .collect();
        assert!(remaining.contains(&"kube-proxy".to_string()));
        assert!(remaining.contains(&"static-pod".to_string()));
    }

    #[test]
    fn drain_then_uncordon_restores_ready() {
        let mut api = cluster();
        let mut ctrl = DrainController::new("worker-1", DrainOptions::default());
        ctrl.drain(&mut api).unwrap();
        ctrl.uncordon(&mut api).unwrap();
        assert_eq!(ctrl.state(), DrainState::Uncordoned);
        assert_eq!(api.status("worker-1").unwrap(), NodeStatus::Ready);
    }

    #[test]
    fn drain_unknown_node_errors() {
        let mut api = InMemoryNodeApi::new();
        let mut ctrl = DrainController::new("ghost", DrainOptions::default());
        assert_eq!(
            ctrl.drain(&mut api),
            Err(DrainError::NodeNotFound("ghost".to_string()))
        );
    }

    #[test]
    fn disruption_budget_blocks_eviction() {
        let mut api = InMemoryNodeApi::new();
        api.add_node("worker-1", alloc::vec![Pod::workload("critical")]);
        api.add_budget(PodDisruption {
            pods: alloc::vec!["critical".to_string()],
            min_available: 1,
        });
        let mut ctrl = DrainController::new("worker-1", DrainOptions::default());
        let err = ctrl.drain(&mut api).unwrap_err();
        assert_eq!(
            err,
            DrainError::DisruptionBudgetBlocked("critical".to_string())
        );
        // Pod was not evicted.
        assert_eq!(api.pods_on("worker-1").unwrap().len(), 1);
    }

    #[test]
    fn budget_allows_eviction_when_replicas_spare() {
        let mut api = InMemoryNodeApi::new();
        api.add_node("worker-1", alloc::vec![Pod::workload("web-1")]);
        // Budget tracks two replicas but only requires one available; with
        // web-2 (not on this node) staying available, web-1 can go.
        api.available.insert("web-2".to_string(), true);
        api.add_budget(PodDisruption {
            pods: alloc::vec!["web-1".to_string(), "web-2".to_string()],
            min_available: 1,
        });
        let mut ctrl = DrainController::new("worker-1", DrainOptions::default());
        assert_eq!(ctrl.drain(&mut api).unwrap(), DrainState::Drained);
        assert_eq!(ctrl.evicted(), &["web-1".to_string()]);
    }

    #[test]
    fn ignore_daemonsets_false_evicts_them() {
        let mut api = InMemoryNodeApi::new();
        api.add_node(
            "worker-1",
            alloc::vec![Pod::workload("app"), Pod::daemonset("logger")],
        );
        let opts = DrainOptions {
            ignore_daemonsets: false,
            ..DrainOptions::default()
        };
        let mut ctrl = DrainController::new("worker-1", opts);
        ctrl.drain(&mut api).unwrap();
        let mut evicted = ctrl.evicted().to_vec();
        evicted.sort();
        assert_eq!(
            evicted,
            alloc::vec!["app".to_string(), "logger".to_string()]
        );
    }

    #[test]
    fn empty_node_drains_immediately() {
        let mut api = InMemoryNodeApi::new();
        api.add_node("empty", Vec::new());
        let mut ctrl = DrainController::new("empty", DrainOptions::default());
        assert_eq!(ctrl.drain(&mut api).unwrap(), DrainState::Drained);
        assert!(ctrl.evicted().is_empty());
    }
}
