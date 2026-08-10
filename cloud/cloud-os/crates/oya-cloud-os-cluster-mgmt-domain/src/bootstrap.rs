//! Cluster bootstrap orchestration (`talosctl bootstrap` + apply-config flow).
//!
//! After nodes are provisioned, `talosctl cluster create` applies machine
//! configs to every node, then issues a one-time `bootstrap` call to a single
//! control-plane node which initializes etcd. This module models that
//! orchestration as a small state machine ([`BootstrapPhase`]) driven by a
//! [`BootstrapOrchestrator`].

use crate::ClusterError;
use std::collections::BTreeMap;
use os_kernel::machine_type::MachineType;

/// Phases of bringing a cluster from "nodes provisioned" to "etcd bootstrapped".
///
/// The order is strict: configs must be applied before bootstrap may run, and
/// bootstrap may only be issued once (a second attempt is rejected, matching
/// Talos's `AlreadyExists` guard on the bootstrap API).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BootstrapPhase {
    /// No configs applied yet.
    Pending,
    /// Machine configs have been applied to all nodes.
    ConfigsApplied,
    /// The bootstrap call has been issued to one control-plane node; etcd is
    /// initializing.
    Bootstrapping,
    /// etcd is up and the control plane is forming.
    Bootstrapped,
}

impl BootstrapPhase {
    /// Whether `self` may legally advance to `next`.
    pub fn can_advance_to(self, next: BootstrapPhase) -> bool {
        use BootstrapPhase::*;
        matches!(
            (self, next),
            (Pending, ConfigsApplied)
                | (ConfigsApplied, Bootstrapping)
                | (Bootstrapping, Bootstrapped)
        )
    }
}

/// State tracked for a single node during bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeBootState {
    machine_type: MachineType,
    config_applied: bool,
    bootstrap_requested: bool,
}

/// Drives a cluster from provisioned nodes to a bootstrapped control plane.
#[derive(Debug)]
pub struct BootstrapOrchestrator {
    cluster_name: String,
    phase: BootstrapPhase,
    nodes: BTreeMap<String, NodeBootState>,
    bootstrap_node: Option<String>,
}

impl BootstrapOrchestrator {
    /// Create an orchestrator for a named cluster with a set of `(ip, type)`
    /// nodes.
    pub fn new(
        cluster_name: impl Into<String>,
        nodes: impl IntoIterator<Item = (String, MachineType)>,
    ) -> Result<Self, ClusterError> {
        let mut map = BTreeMap::new();
        for (ip, machine_type) in nodes {
            map.insert(
                ip,
                NodeBootState {
                    machine_type,
                    config_applied: false,
                    bootstrap_requested: false,
                },
            );
        }
        if map.is_empty() {
            return Err(ClusterError::invalid(
                "bootstrap requires at least one node",
            ));
        }
        if !map.values().any(|n| n.machine_type.is_control_plane()) {
            return Err(ClusterError::invalid(
                "bootstrap requires at least one control-plane node",
            ));
        }
        Ok(BootstrapOrchestrator {
            cluster_name: cluster_name.into(),
            phase: BootstrapPhase::Pending,
            nodes: map,
            bootstrap_node: None,
        })
    }

    /// Current phase.
    pub fn phase(&self) -> BootstrapPhase {
        self.phase
    }

    /// The cluster name.
    pub fn cluster_name(&self) -> &str {
        &self.cluster_name
    }

    /// The node that received the bootstrap call, if any.
    pub fn bootstrap_node(&self) -> Option<&str> {
        self.bootstrap_node.as_deref()
    }

    /// Mark a single node's machine config as applied.
    pub fn apply_config(&mut self, node_ip: &str) -> Result<(), ClusterError> {
        let node = self
            .nodes
            .get_mut(node_ip)
            .ok_or_else(|| ClusterError::not_found(format!("node {node_ip:?} not in cluster")))?;
        node.config_applied = true;
        Ok(())
    }

    /// Apply config to every node and advance to [`BootstrapPhase::ConfigsApplied`].
    pub fn apply_all_configs(&mut self) -> Result<(), ClusterError> {
        for node in self.nodes.values_mut() {
            node.config_applied = true;
        }
        self.transition(BootstrapPhase::ConfigsApplied)
    }

    /// Whether every node has had its config applied.
    pub fn all_configs_applied(&self) -> bool {
        self.nodes.values().all(|n| n.config_applied)
    }

    /// Issue the one-time bootstrap call to a chosen control-plane node.
    ///
    /// Fails if configs are not yet applied, if the node is not control-plane,
    /// or if bootstrap was already issued (matching Talos's idempotency guard).
    pub fn bootstrap(&mut self, node_ip: &str) -> Result<(), ClusterError> {
        if self.phase == BootstrapPhase::Bootstrapping || self.phase == BootstrapPhase::Bootstrapped
        {
            return Err(ClusterError::invalid_state(
                "etcd has already been bootstrapped for this cluster",
            ));
        }
        if !self.all_configs_applied() {
            return Err(ClusterError::invalid_state(
                "cannot bootstrap before all node configs are applied",
            ));
        }
        let node = self
            .nodes
            .get_mut(node_ip)
            .ok_or_else(|| ClusterError::not_found(format!("node {node_ip:?} not in cluster")))?;
        if !node.machine_type.is_control_plane() {
            return Err(ClusterError::invalid(format!(
                "node {node_ip:?} is not a control-plane node and cannot be bootstrapped"
            )));
        }
        node.bootstrap_requested = true;
        self.bootstrap_node = Some(node_ip.to_string());
        self.transition(BootstrapPhase::Bootstrapping)
    }

    /// Mark the cluster as fully bootstrapped (etcd up).
    pub fn mark_bootstrapped(&mut self) -> Result<(), ClusterError> {
        self.transition(BootstrapPhase::Bootstrapped)
    }

    /// Run the full happy-path bootstrap, choosing the first control-plane node.
    pub fn run(&mut self) -> Result<(), ClusterError> {
        if self.phase == BootstrapPhase::Pending {
            self.apply_all_configs()?;
        }
        let cp_ip = self
            .nodes
            .iter()
            .find(|(_, n)| n.machine_type.is_control_plane())
            .map(|(ip, _)| ip.clone())
            .ok_or_else(|| ClusterError::invalid("no control-plane node available"))?;
        self.bootstrap(&cp_ip)?;
        self.mark_bootstrapped()
    }

    fn transition(&mut self, next: BootstrapPhase) -> Result<(), ClusterError> {
        if self.phase == next {
            return Ok(());
        }
        if !self.phase.can_advance_to(next) {
            return Err(ClusterError::invalid_state(format!(
                "cannot advance bootstrap from {:?} to {next:?}",
                self.phase
            )));
        }
        self.phase = next;
        Ok(())
    }
}
