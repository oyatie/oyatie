//! Config controllers: acquisition, application, and the derived machine /
//! cluster config views.
//!
//! Mirrors Talos `internal/app/machined/pkg/controllers/config/*`:
//!
//! - `acquire.go`: obtains the raw machine config (from disk, platform
//!   metadata, or maintenance-mode API) and publishes a `MachineConfig`.
//! - `machine_config.go` / `k8s_address_filter.go` etc.: derive narrower views
//!   (k8s control-plane config, cluster config) from the acquired config.
//!
//! Here the raw config is a small validated struct and the controllers move it
//! through `Acquired -> Applied -> Derived` resources in the COSI store.

use crate::reconcile::{
    Controller, Input, Output, ReconcileContext, ReconcileError, ReconcileResult,
};
use os_cosi_domain::resource::ResourceKind;
use os_cosi_domain::{Metadata, Resource};
use os_kernel::{Error, MachineType, ResourceId, Result};

/// Where a machine config was acquired from. Mirrors Talos config sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    /// Persisted to disk (`/system/state/config.yaml`).
    Disk,
    /// Fetched from the platform metadata service / user-data.
    Platform,
    /// Supplied over the maintenance-mode API.
    Maintenance,
    /// Default config synthesized when none is present.
    Default,
}

impl ConfigSource {
    /// Stable lowercase string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigSource::Disk => "disk",
            ConfigSource::Platform => "platform",
            ConfigSource::Maintenance => "maintenance",
            ConfigSource::Default => "default",
        }
    }
}

/// A minimal but real machine-config spec carried through the controllers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineConfigSpec {
    /// controlplane or worker.
    pub machine_type: MachineType,
    /// The cluster id / name.
    pub cluster_name: String,
    /// Kubernetes control-plane endpoint (e.g. `https://10.0.0.1:6443`).
    pub control_plane_endpoint: String,
    /// Kubernetes version requested by the config.
    pub kubernetes_version: String,
}

impl MachineConfigSpec {
    /// Validate a config spec, mirroring Talos config validation:
    /// the cluster name and endpoint are required and the endpoint must be an
    /// https URL.
    pub fn validate(&self) -> Result<()> {
        if self.cluster_name.trim().is_empty() {
            return Err(Error::invalid("cluster name is required"));
        }
        if self.control_plane_endpoint.trim().is_empty() {
            return Err(Error::invalid("control plane endpoint is required"));
        }
        if !self.control_plane_endpoint.starts_with("https://") {
            return Err(Error::invalid("control plane endpoint must be https"));
        }
        if self.kubernetes_version.trim().is_empty() {
            return Err(Error::invalid("kubernetes version is required"));
        }
        Ok(())
    }

    fn fingerprint(&self) -> String {
        format!(
            "type={};cluster={};endpoint={};k8s={}",
            self.machine_type.as_str(),
            self.cluster_name,
            self.control_plane_endpoint,
            self.kubernetes_version
        )
    }
}

/// The raw acquired config resource, published by the acquire controller.
#[derive(Debug, Clone)]
pub struct AcquiredConfig {
    meta: Metadata,
    /// Where it came from.
    pub source: ConfigSource,
    /// The parsed/validated spec.
    pub spec: MachineConfigSpec,
}

impl AcquiredConfig {
    /// The singleton id.
    pub const ID: &'static str = "v1alpha1";

    /// Build an acquired-config singleton.
    pub fn new(source: ConfigSource, spec: MachineConfigSpec) -> Self {
        AcquiredConfig {
            meta: Metadata::new(
                "config",
                "AcquiredConfig",
                ResourceId::new(Self::ID).unwrap(),
            ),
            source,
            spec,
        }
    }

    /// The resource kind.
    pub fn kind() -> ResourceKind {
        ResourceKind::new("config", "AcquiredConfig")
    }
}

impl Resource for AcquiredConfig {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }
    fn spec_fingerprint(&self) -> String {
        format!(
            "source={};{}",
            self.source.as_str(),
            self.spec.fingerprint()
        )
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// The applied machine config resource (validated, ready for consumers).
#[derive(Debug, Clone)]
pub struct MachineConfig {
    meta: Metadata,
    /// The validated spec.
    pub spec: MachineConfigSpec,
}

impl MachineConfig {
    /// The singleton id.
    pub const ID: &'static str = "v1alpha1";

    /// The resource kind.
    pub fn kind() -> ResourceKind {
        ResourceKind::new("config", "MachineConfig")
    }
}

impl Resource for MachineConfig {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }
    fn spec_fingerprint(&self) -> String {
        self.spec.fingerprint()
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Source-shaped active Talos machine config document.
///
/// Upstream Talos stores the active applied config as
/// `config/MachineConfigs.config.talos.dev/v1alpha1` and exposes the YAML bytes
/// to config-derived controllers. The compact `MachineConfig` above is the
/// legacy controller smoke-test view; this resource preserves the raw document
/// boundary needed by source-guided network config controllers.
#[derive(Debug, Clone)]
pub struct MachineConfigDocument {
    meta: Metadata,
    /// Raw YAML/multi-doc machine config contents.
    pub contents: String,
}

impl MachineConfigDocument {
    /// The active applied-config singleton id.
    pub const ACTIVE_ID: &'static str = "v1alpha1";
    /// Source-shaped upstream kind.
    pub const KIND: &'static str = "MachineConfigs.config.talos.dev";

    /// Build an active machine-config document.
    pub fn new(contents: impl Into<String>) -> Self {
        MachineConfigDocument {
            meta: Metadata::new(
                "config",
                Self::KIND,
                ResourceId::new(Self::ACTIVE_ID).unwrap(),
            ),
            contents: contents.into(),
        }
    }

    /// The resource kind.
    pub fn kind() -> ResourceKind {
        ResourceKind::new("config", Self::KIND)
    }

    /// Stable key for the active machine config.
    pub fn active_key() -> String {
        format!("config/{}/{}", Self::KIND, Self::ACTIVE_ID)
    }
}

impl Resource for MachineConfigDocument {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }
    fn spec_fingerprint(&self) -> String {
        format!("contents={}", hex_bytes(self.contents.as_bytes()))
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// The narrow cluster-config view derived for Kubernetes consumers.
#[derive(Debug, Clone)]
pub struct ClusterConfig {
    meta: Metadata,
    /// Cluster name.
    pub cluster_name: String,
    /// Control-plane endpoint.
    pub endpoint: String,
    /// Kubernetes version.
    pub kubernetes_version: String,
    /// Whether this machine participates in the control plane.
    pub is_control_plane: bool,
}

impl ClusterConfig {
    /// The singleton id.
    pub const ID: &'static str = "cluster";

    /// The resource kind.
    pub fn kind() -> ResourceKind {
        ResourceKind::new("config", "ClusterConfig")
    }
}

impl Resource for ClusterConfig {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }
    fn spec_fingerprint(&self) -> String {
        format!(
            "cluster={};endpoint={};k8s={};cp={}",
            self.cluster_name, self.endpoint, self.kubernetes_version, self.is_control_plane
        )
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// The config-acquire controller: validates the [`AcquiredConfig`] and, when
/// valid, publishes a [`MachineConfig`]. Invalid configs are rejected and the
/// previously applied config (if any) is left untouched.
#[derive(Debug, Default)]
pub struct ConfigAcquireController;

impl ConfigAcquireController {
    /// Construct the controller.
    pub fn new() -> Self {
        ConfigAcquireController
    }
}

impl Controller for ConfigAcquireController {
    fn name(&self) -> &str {
        "config.ConfigAcquireController"
    }
    fn inputs(&self) -> Vec<Input> {
        vec![Input::weak(AcquiredConfig::kind())]
    }
    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(MachineConfig::kind())]
    }
    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let Some(acquired) = ctx.get(&format!("config/AcquiredConfig/{}", AcquiredConfig::ID))
        else {
            return Ok(()); // nothing acquired yet
        };
        let spec = parse_spec(&acquired.spec_fingerprint());
        spec.validate()
            .map_err(|e| ReconcileError::Invalid(e.to_string()))?;

        let cfg = MachineConfig {
            meta: Metadata::new(
                "config",
                "MachineConfig",
                ResourceId::new(MachineConfig::ID).unwrap(),
            ),
            spec,
        };
        ctx.write(Box::new(cfg))?;
        Ok(())
    }
}

/// The cluster-config controller: derives the narrow [`ClusterConfig`] view
/// from the applied [`MachineConfig`].
#[derive(Debug, Default)]
pub struct ClusterConfigController;

impl ClusterConfigController {
    /// Construct the controller.
    pub fn new() -> Self {
        ClusterConfigController
    }
}

impl Controller for ClusterConfigController {
    fn name(&self) -> &str {
        "config.ClusterConfigController"
    }
    fn inputs(&self) -> Vec<Input> {
        vec![Input::weak(MachineConfig::kind())]
    }
    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(ClusterConfig::kind())]
    }
    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let Some(mc) = ctx.get(&format!("config/MachineConfig/{}", MachineConfig::ID)) else {
            // No machine config: ensure the derived view is removed.
            ctx.destroy(&format!("config/ClusterConfig/{}", ClusterConfig::ID))?;
            return Ok(());
        };
        let spec = parse_spec(&mc.spec_fingerprint());
        let cc = ClusterConfig {
            meta: Metadata::new(
                "config",
                "ClusterConfig",
                ResourceId::new(ClusterConfig::ID).unwrap(),
            ),
            cluster_name: spec.cluster_name.clone(),
            endpoint: spec.control_plane_endpoint.clone(),
            kubernetes_version: spec.kubernetes_version.clone(),
            is_control_plane: spec.machine_type == MachineType::ControlPlane,
        };
        ctx.write(Box::new(cc))?;
        Ok(())
    }
}

fn parse_spec(fp: &str) -> MachineConfigSpec {
    let mut machine_type = MachineType::Worker;
    let mut cluster_name = String::new();
    let mut control_plane_endpoint = String::new();
    let mut kubernetes_version = String::new();
    for part in fp.split(';') {
        if let Some(v) = part.strip_prefix("type=") {
            machine_type = match v {
                "controlplane" => MachineType::ControlPlane,
                "init" => MachineType::Init,
                _ => MachineType::Worker,
            };
        } else if let Some(v) = part.strip_prefix("cluster=") {
            cluster_name = v.to_string();
        } else if let Some(v) = part.strip_prefix("endpoint=") {
            control_plane_endpoint = v.to_string();
        } else if let Some(v) = part.strip_prefix("k8s=") {
            kubernetes_version = v.to_string();
        }
    }
    MachineConfigSpec {
        machine_type,
        cluster_name,
        control_plane_endpoint,
        kubernetes_version,
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_cosi_domain::State;

    fn good_spec() -> MachineConfigSpec {
        MachineConfigSpec {
            machine_type: MachineType::ControlPlane,
            cluster_name: "talos-default".into(),
            control_plane_endpoint: "https://10.0.0.1:6443".into(),
            kubernetes_version: "1.30.0".into(),
        }
    }

    #[test]
    fn spec_validation() {
        assert!(good_spec().validate().is_ok());

        let mut s = good_spec();
        s.cluster_name = String::new();
        assert!(s.validate().is_err());

        let mut s = good_spec();
        s.control_plane_endpoint = "http://x".into();
        assert!(s.validate().is_err());

        let mut s = good_spec();
        s.kubernetes_version = String::new();
        assert!(s.validate().is_err());
    }

    #[test]
    fn acquire_publishes_machine_config() {
        let mut state = State::new();
        state
            .create(Box::new(AcquiredConfig::new(
                ConfigSource::Disk,
                good_spec(),
            )))
            .unwrap();
        let mut c = ConfigAcquireController::new();
        {
            let mut ctx = ReconcileContext::new(
                &mut state,
                "config.ConfigAcquireController",
                vec![MachineConfig::kind()],
            );
            c.reconcile(&mut ctx).unwrap();
        }
        let mc = state.get("config/MachineConfig/v1alpha1").unwrap();
        assert_eq!(
            mc.spec_fingerprint(),
            "type=controlplane;cluster=talos-default;endpoint=https://10.0.0.1:6443;k8s=1.30.0"
        );
    }

    #[test]
    fn acquire_rejects_invalid_config() {
        let mut state = State::new();
        let mut bad = good_spec();
        bad.control_plane_endpoint = "ftp://x".into();
        state
            .create(Box::new(AcquiredConfig::new(
                ConfigSource::Maintenance,
                bad,
            )))
            .unwrap();
        let mut c = ConfigAcquireController::new();
        let mut ctx = ReconcileContext::new(
            &mut state,
            "config.ConfigAcquireController",
            vec![MachineConfig::kind()],
        );
        let err = c.reconcile(&mut ctx).unwrap_err();
        assert!(matches!(err, ReconcileError::Invalid(_)));
    }

    #[test]
    fn acquire_noop_without_input() {
        let mut state = State::new();
        let mut c = ConfigAcquireController::new();
        let mut ctx = ReconcileContext::new(
            &mut state,
            "config.ConfigAcquireController",
            vec![MachineConfig::kind()],
        );
        assert!(c.reconcile(&mut ctx).is_ok());
        assert!(!ctx.state().contains("config/MachineConfig/v1alpha1"));
    }

    #[test]
    fn cluster_config_derives_control_plane_flag() {
        let mut state = State::new();
        state
            .create(Box::new(MachineConfig {
                meta: Metadata::new(
                    "config",
                    "MachineConfig",
                    ResourceId::new("v1alpha1").unwrap(),
                ),
                spec: good_spec(),
            }))
            .unwrap();
        let mut c = ClusterConfigController::new();
        {
            let mut ctx = ReconcileContext::new(
                &mut state,
                "config.ClusterConfigController",
                vec![ClusterConfig::kind()],
            );
            c.reconcile(&mut ctx).unwrap();
        }
        let cc = state.get("config/ClusterConfig/cluster").unwrap();
        assert_eq!(
            cc.spec_fingerprint(),
            "cluster=talos-default;endpoint=https://10.0.0.1:6443;k8s=1.30.0;cp=true"
        );
    }

    #[test]
    fn cluster_config_removed_when_machine_config_absent() {
        let mut state = State::new();
        // Pre-seed a stale derived view owned by the controller.
        let mut existing = ClusterConfig {
            meta: Metadata::new(
                "config",
                "ClusterConfig",
                ResourceId::new("cluster").unwrap(),
            ),
            cluster_name: "old".into(),
            endpoint: "https://x".into(),
            kubernetes_version: "1.0.0".into(),
            is_control_plane: true,
        };
        existing.meta.set_owner("config.ClusterConfigController");
        state.create(Box::new(existing)).unwrap();

        let mut c = ClusterConfigController::new();
        {
            let mut ctx = ReconcileContext::new(
                &mut state,
                "config.ClusterConfigController",
                vec![ClusterConfig::kind()],
            );
            c.reconcile(&mut ctx).unwrap();
        }
        assert!(!state.contains("config/ClusterConfig/cluster"));
    }
}
