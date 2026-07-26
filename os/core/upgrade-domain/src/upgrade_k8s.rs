//! `talosctl upgrade-k8s`: in-place Kubernetes component upgrades.
//!
//! Distinct from the OS upgrade, `upgrade-k8s` bumps the control-plane
//! components (kube-apiserver, kube-controller-manager, kube-scheduler) and the
//! kubelet to a new Kubernetes version without touching the OS. The real flow
//! (`pkg/cluster/kubernetes/upgrade.go`) is roughly:
//!
//! 1. Validate the target version is reachable (no skipping a minor).
//! 2. Pre-pull the new component images to every node.
//! 3. Upgrade the static-pod manifests for each control-plane component, one at
//!    a time, waiting for each to become healthy.
//! 4. Bump the kubelet version across nodes.
//! 5. Update bootstrap manifests (kube-proxy, CoreDNS, etc.).
//!
//! This module models the upgrade plan, ordering, and the per-component state
//! machine. The cluster API boundary is [`K8sApi`]; [`InMemoryK8sApi`] is the
//! deterministic test implementation.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use os_kernel::Version;

/// A Kubernetes control-plane / node component that gets version-bumped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentKind {
    /// kube-apiserver (upgraded first).
    ApiServer,
    /// kube-controller-manager.
    ControllerManager,
    /// kube-scheduler.
    Scheduler,
    /// kubelet on every node.
    Kubelet,
    /// Bootstrap manifests (kube-proxy, CoreDNS).
    BootstrapManifests,
}

impl ComponentKind {
    /// Stable component name.
    pub fn name(self) -> &'static str {
        match self {
            ComponentKind::ApiServer => "kube-apiserver",
            ComponentKind::ControllerManager => "kube-controller-manager",
            ComponentKind::Scheduler => "kube-scheduler",
            ComponentKind::Kubelet => "kubelet",
            ComponentKind::BootstrapManifests => "bootstrap-manifests",
        }
    }

    /// Whether this component is a control-plane static pod (vs. node-level).
    pub fn is_control_plane(self) -> bool {
        matches!(
            self,
            ComponentKind::ApiServer | ComponentKind::ControllerManager | ComponentKind::Scheduler
        )
    }

    /// The canonical upgrade order Talos applies.
    pub fn upgrade_order() -> [ComponentKind; 5] {
        [
            ComponentKind::ApiServer,
            ComponentKind::ControllerManager,
            ComponentKind::Scheduler,
            ComponentKind::Kubelet,
            ComponentKind::BootstrapManifests,
        ]
    }
}

/// The currently-running and target versions for each component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentVersions {
    versions: BTreeMap<ComponentKind, Version>,
}

impl ComponentVersions {
    /// All components at one uniform version.
    pub fn uniform(v: Version) -> Self {
        let mut versions = BTreeMap::new();
        for c in ComponentKind::upgrade_order() {
            versions.insert(c, v.clone());
        }
        ComponentVersions { versions }
    }

    /// The version of a component.
    pub fn get(&self, c: ComponentKind) -> Option<&Version> {
        self.versions.get(&c)
    }

    /// Set a component's version.
    pub fn set(&mut self, c: ComponentKind, v: Version) {
        self.versions.insert(c, v);
    }

    /// Whether every component is at exactly `target`.
    pub fn all_at(&self, target: &Version) -> bool {
        ComponentKind::upgrade_order()
            .iter()
            .all(|c| self.versions.get(c) == Some(target))
    }
}

/// Errors raised by `upgrade-k8s`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum K8sUpgradeError {
    /// Target skips more than one minor version (disallowed).
    UnsupportedSkip { from: Version, to: Version },
    /// Target is not strictly newer than current.
    NotAnUpgrade { from: Version, to: Version },
    /// A required image failed to pre-pull.
    PrePullFailed(String),
    /// A component failed to become healthy after the manifest bump.
    ComponentUnhealthy(ComponentKind),
    /// A node could not be reached.
    NodeUnreachable(String),
}

impl fmt::Display for K8sUpgradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            K8sUpgradeError::UnsupportedSkip { from, to } => {
                write!(f, "cannot upgrade from {from} to {to}: skips a minor")
            }
            K8sUpgradeError::NotAnUpgrade { from, to } => {
                write!(f, "{to} is not newer than {from}")
            }
            K8sUpgradeError::PrePullFailed(img) => write!(f, "failed to pre-pull {img}"),
            K8sUpgradeError::ComponentUnhealthy(c) => {
                write!(f, "component {} did not become healthy", c.name())
            }
            K8sUpgradeError::NodeUnreachable(n) => write!(f, "node unreachable: {n}"),
        }
    }
}

/// The cluster operations `upgrade-k8s` performs.
pub trait K8sApi {
    /// Pre-pull `image` on `node`. Returns Err if the pull fails.
    fn pre_pull(&mut self, node: &str, image: &str) -> Result<(), K8sUpgradeError>;

    /// Update the static-pod manifest for `component` to `version` and report
    /// whether it became healthy.
    fn update_manifest(
        &mut self,
        component: ComponentKind,
        version: &Version,
    ) -> Result<bool, K8sUpgradeError>;

    /// Bump the kubelet version on `node`.
    fn bump_kubelet(&mut self, node: &str, version: &Version) -> Result<(), K8sUpgradeError>;

    /// The set of node names in the cluster.
    fn nodes(&self) -> Vec<String>;
}

/// In-memory cluster API for tests.
#[derive(Debug, Clone)]
pub struct InMemoryK8sApi {
    nodes: Vec<String>,
    /// Images that should fail to pull (to model registry errors).
    failing_images: Vec<String>,
    /// Components that should report unhealthy after a manifest update.
    unhealthy: Vec<ComponentKind>,
    /// Recorded pulled images per node.
    pulled: BTreeMap<String, Vec<String>>,
    /// Recorded kubelet versions per node.
    kubelet_versions: BTreeMap<String, Version>,
    /// Recorded manifest updates.
    manifest_updates: Vec<(ComponentKind, Version)>,
}

impl InMemoryK8sApi {
    /// A cluster with the given node names.
    pub fn new(nodes: &[&str]) -> Self {
        InMemoryK8sApi {
            nodes: nodes.iter().map(|n| n.to_string()).collect(),
            failing_images: Vec::new(),
            unhealthy: Vec::new(),
            pulled: BTreeMap::new(),
            kubelet_versions: BTreeMap::new(),
            manifest_updates: Vec::new(),
        }
    }

    /// Mark an image substring as failing to pull.
    pub fn fail_image(&mut self, image: &str) {
        self.failing_images.push(image.to_string());
    }

    /// Mark a component as failing its health check.
    pub fn fail_health(&mut self, c: ComponentKind) {
        self.unhealthy.push(c);
    }

    /// Images pulled on a node.
    pub fn pulled_on(&self, node: &str) -> Vec<String> {
        self.pulled.get(node).cloned().unwrap_or_default()
    }

    /// Recorded manifest updates, in order applied.
    pub fn manifest_updates(&self) -> &[(ComponentKind, Version)] {
        &self.manifest_updates
    }

    /// Kubelet version recorded for a node.
    pub fn kubelet_version(&self, node: &str) -> Option<&Version> {
        self.kubelet_versions.get(node)
    }
}

impl K8sApi for InMemoryK8sApi {
    fn pre_pull(&mut self, node: &str, image: &str) -> Result<(), K8sUpgradeError> {
        if !self.nodes.iter().any(|n| n == node) {
            return Err(K8sUpgradeError::NodeUnreachable(node.to_string()));
        }
        if self.failing_images.iter().any(|i| image.contains(i)) {
            return Err(K8sUpgradeError::PrePullFailed(image.to_string()));
        }
        self.pulled
            .entry(node.to_string())
            .or_default()
            .push(image.to_string());
        Ok(())
    }

    fn update_manifest(
        &mut self,
        component: ComponentKind,
        version: &Version,
    ) -> Result<bool, K8sUpgradeError> {
        self.manifest_updates.push((component, version.clone()));
        Ok(!self.unhealthy.contains(&component))
    }

    fn bump_kubelet(&mut self, node: &str, version: &Version) -> Result<(), K8sUpgradeError> {
        if !self.nodes.iter().any(|n| n == node) {
            return Err(K8sUpgradeError::NodeUnreachable(node.to_string()));
        }
        self.kubelet_versions
            .insert(node.to_string(), version.clone());
        Ok(())
    }

    fn nodes(&self) -> Vec<String> {
        self.nodes.clone()
    }
}

/// A validated plan to move the cluster from `current` to `target`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sUpgradePlan {
    /// Version the cluster is moving away from.
    pub current: Version,
    /// Version the cluster is moving to.
    pub target: Version,
    /// Registry prefix for component images (e.g. `registry.k8s.io`).
    pub registry: String,
    /// Component upgrade order.
    pub order: Vec<ComponentKind>,
}

impl K8sUpgradePlan {
    /// Build and validate a plan. Enforces the "no skipping a minor" rule.
    pub fn new(current: Version, target: Version, registry: &str) -> Result<Self, K8sUpgradeError> {
        if target <= current {
            return Err(K8sUpgradeError::NotAnUpgrade {
                from: current,
                to: target,
            });
        }
        if !current.is_upgrade_allowed_to(&target) {
            return Err(K8sUpgradeError::UnsupportedSkip {
                from: current,
                to: target,
            });
        }
        Ok(K8sUpgradePlan {
            current,
            target,
            registry: registry.to_string(),
            order: ComponentKind::upgrade_order().to_vec(),
        })
    }

    /// The image reference for a control-plane component at the target version.
    pub fn image_for(&self, c: ComponentKind) -> String {
        alloc::format!(
            "{}/{}:v{}.{}.{}",
            self.registry,
            c.name(),
            self.target.major,
            self.target.minor,
            self.target.patch
        )
    }
}

/// Phases of the upgrade-k8s controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum K8sUpgradePhase {
    /// Not started.
    Pending,
    /// Pre-pulling images to all nodes.
    PrePulling,
    /// Upgrading control-plane static pods in order.
    UpgradingControlPlane,
    /// Bumping kubelet on all nodes.
    UpgradingKubelet,
    /// Applying bootstrap manifests.
    BootstrapManifests,
    /// Done.
    Complete,
}

/// Orchestrates `upgrade-k8s` against a [`K8sApi`].
#[derive(Debug)]
pub struct UpgradeK8sController {
    plan: K8sUpgradePlan,
    phase: K8sUpgradePhase,
    upgraded: Vec<ComponentKind>,
}

impl UpgradeK8sController {
    /// Build a controller from a validated plan.
    pub fn new(plan: K8sUpgradePlan) -> Self {
        UpgradeK8sController {
            plan,
            phase: K8sUpgradePhase::Pending,
            upgraded: Vec::new(),
        }
    }

    /// Current phase.
    pub fn phase(&self) -> K8sUpgradePhase {
        self.phase
    }

    /// Components successfully upgraded, in order.
    pub fn upgraded(&self) -> &[ComponentKind] {
        &self.upgraded
    }

    /// The plan being executed.
    pub fn plan(&self) -> &K8sUpgradePlan {
        &self.plan
    }

    /// Run the entire upgrade against `api`.
    pub fn run<A: K8sApi>(&mut self, api: &mut A) -> Result<K8sUpgradePhase, K8sUpgradeError> {
        let nodes = api.nodes();

        // Phase 1: pre-pull control-plane images on every node.
        self.phase = K8sUpgradePhase::PrePulling;
        for c in self.plan.order.clone() {
            if !c.is_control_plane() {
                continue;
            }
            let image = self.plan.image_for(c);
            for node in &nodes {
                api.pre_pull(node, &image)?;
            }
        }

        // Phase 2: control-plane static pods, one at a time, health-gated.
        self.phase = K8sUpgradePhase::UpgradingControlPlane;
        for c in self.plan.order.clone() {
            if !c.is_control_plane() {
                continue;
            }
            let healthy = api.update_manifest(c, &self.plan.target)?;
            if !healthy {
                return Err(K8sUpgradeError::ComponentUnhealthy(c));
            }
            self.upgraded.push(c);
        }

        // Phase 3: kubelet on every node.
        self.phase = K8sUpgradePhase::UpgradingKubelet;
        for node in &nodes {
            api.bump_kubelet(node, &self.plan.target)?;
        }
        self.upgraded.push(ComponentKind::Kubelet);

        // Phase 4: bootstrap manifests.
        self.phase = K8sUpgradePhase::BootstrapManifests;
        let healthy = api.update_manifest(ComponentKind::BootstrapManifests, &self.plan.target)?;
        if !healthy {
            return Err(K8sUpgradeError::ComponentUnhealthy(
                ComponentKind::BootstrapManifests,
            ));
        }
        self.upgraded.push(ComponentKind::BootstrapManifests);

        self.phase = K8sUpgradePhase::Complete;
        Ok(self.phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(maj: u64, min: u64, patch: u64) -> Version {
        Version::new(maj, min, patch)
    }

    #[test]
    fn component_order_and_names() {
        let order = ComponentKind::upgrade_order();
        assert_eq!(order[0], ComponentKind::ApiServer);
        assert_eq!(order[4], ComponentKind::BootstrapManifests);
        assert_eq!(ComponentKind::ApiServer.name(), "kube-apiserver");
        assert!(ComponentKind::ApiServer.is_control_plane());
        assert!(!ComponentKind::Kubelet.is_control_plane());
    }

    #[test]
    fn plan_validation_rejects_downgrade_and_skip() {
        assert!(matches!(
            K8sUpgradePlan::new(v(1, 29, 0), v(1, 29, 0), "registry.k8s.io"),
            Err(K8sUpgradeError::NotAnUpgrade { .. })
        ));
        assert!(matches!(
            K8sUpgradePlan::new(v(1, 29, 0), v(1, 28, 0), "registry.k8s.io"),
            Err(K8sUpgradeError::NotAnUpgrade { .. })
        ));
        assert!(matches!(
            K8sUpgradePlan::new(v(1, 28, 0), v(1, 30, 0), "registry.k8s.io"),
            Err(K8sUpgradeError::UnsupportedSkip { .. })
        ));
        assert!(K8sUpgradePlan::new(v(1, 28, 5), v(1, 29, 0), "registry.k8s.io").is_ok());
    }

    #[test]
    fn image_ref_formatting() {
        let plan = K8sUpgradePlan::new(v(1, 28, 0), v(1, 29, 2), "registry.k8s.io").unwrap();
        assert_eq!(
            plan.image_for(ComponentKind::ApiServer),
            "registry.k8s.io/kube-apiserver:v1.29.2"
        );
    }

    #[test]
    fn full_upgrade_runs_all_phases() {
        let plan = K8sUpgradePlan::new(v(1, 28, 0), v(1, 29, 0), "registry.k8s.io").unwrap();
        let mut ctrl = UpgradeK8sController::new(plan);
        let mut api = InMemoryK8sApi::new(&["cp-1", "worker-1"]);

        let phase = ctrl.run(&mut api).unwrap();
        assert_eq!(phase, K8sUpgradePhase::Complete);

        // Control plane upgraded in order, then kubelet, then manifests.
        assert_eq!(
            ctrl.upgraded(),
            &[
                ComponentKind::ApiServer,
                ComponentKind::ControllerManager,
                ComponentKind::Scheduler,
                ComponentKind::Kubelet,
                ComponentKind::BootstrapManifests,
            ]
        );

        // Every node pre-pulled the three control-plane images.
        assert_eq!(api.pulled_on("cp-1").len(), 3);
        assert_eq!(api.pulled_on("worker-1").len(), 3);

        // Kubelet bumped on all nodes.
        assert_eq!(api.kubelet_version("cp-1"), Some(&v(1, 29, 0)));
        assert_eq!(api.kubelet_version("worker-1"), Some(&v(1, 29, 0)));
    }

    #[test]
    fn pre_pull_failure_aborts_before_manifests() {
        let plan = K8sUpgradePlan::new(v(1, 28, 0), v(1, 29, 0), "registry.k8s.io").unwrap();
        let mut ctrl = UpgradeK8sController::new(plan);
        let mut api = InMemoryK8sApi::new(&["cp-1"]);
        api.fail_image("kube-scheduler");

        let err = ctrl.run(&mut api).unwrap_err();
        assert!(matches!(err, K8sUpgradeError::PrePullFailed(_)));
        // No manifests should have been applied.
        assert!(api.manifest_updates().is_empty());
        assert_eq!(ctrl.phase(), K8sUpgradePhase::PrePulling);
    }

    #[test]
    fn unhealthy_component_halts_upgrade() {
        let plan = K8sUpgradePlan::new(v(1, 28, 0), v(1, 29, 0), "registry.k8s.io").unwrap();
        let mut ctrl = UpgradeK8sController::new(plan);
        let mut api = InMemoryK8sApi::new(&["cp-1"]);
        api.fail_health(ComponentKind::ControllerManager);

        let err = ctrl.run(&mut api).unwrap_err();
        assert_eq!(
            err,
            K8sUpgradeError::ComponentUnhealthy(ComponentKind::ControllerManager)
        );
        // Only apiserver got fully upgraded.
        assert_eq!(ctrl.upgraded(), &[ComponentKind::ApiServer]);
        assert_eq!(ctrl.phase(), K8sUpgradePhase::UpgradingControlPlane);
    }

    #[test]
    fn component_versions_tracking() {
        let mut cv = ComponentVersions::uniform(v(1, 28, 0));
        assert!(cv.all_at(&v(1, 28, 0)));
        cv.set(ComponentKind::ApiServer, v(1, 29, 0));
        assert!(!cv.all_at(&v(1, 28, 0)));
        assert_eq!(cv.get(ComponentKind::ApiServer), Some(&v(1, 29, 0)));
    }
}
