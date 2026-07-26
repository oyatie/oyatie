//! # talos-config-v1alpha1
//!
//! Models the `v1alpha1` Talos machine configuration API version: the top-level
//! [`V1Alpha1Config`] document (`machine:` + `cluster:`), the machine type
//! (controlplane/worker), and every nested sub-section Talos hangs off those
//! two trees — install, disk encryption, network (devices / vlans / bonds /
//! routes), kubelet, registries, files, env, sysctls, time, disks, seccomp,
//! kernel modules, udev, etcd, the static control-plane components
//! (apiServer / controllerManager / scheduler / proxy), discovery, and logging.
//!
//! It mirrors `pkg/machinery/config/types/v1alpha1` including the defaulting and
//! runtime-mode aware validation Talos applies before a config is accepted.
//!
//! The crate builds on [`os_machine_config_domain`] for the shared [`Validator`] /
//! [`ValidationMode`] / [`ValidationReport`] machinery and on `talos-core` for
//! the cross-cutting [`os_kernel::Error`] type, pulling in no external crates so
//! the workspace build stays fully offline.
//!
//! ## Module map
//!
//! - [`machine`] — the `machine:` sub-tree and its leaf settings.
//! - [`cluster`] — the `cluster:` sub-tree, etcd, control-plane components,
//!   discovery, and logging.
//! - [`install`] — `machine.install` plus system disk encryption.
//! - [`network`] — `machine.network` devices, vlans, bonds, and routes.
//! - [`kubelet`] — `machine.kubelet` and `machine.registries`.
//! - [`etcd`] — `cluster.etcd` and the static control-plane component configs.
//! - [`validation`] — re-exported validation primitives plus v1alpha1 field
//!   predicates.
//! - [`defaults`] — the default constants Talos applies.

pub mod cluster;
pub mod defaults;
pub mod etcd;
pub mod install;
pub mod kubelet;
pub mod machine;
pub mod network;
pub mod validation;

pub use cluster::{
    ClusterConfig, ControlPlaneEndpoint, DiscoveryConfig, LogProtocol, LoggingDestination,
};
pub use etcd::{
    ApiServerConfig, ControlPlaneComponent, ControllerManagerConfig, EtcdConfig, ProxyConfig,
    SchedulerConfig,
};
pub use install::{
    EncryptionCipher, EncryptionConfig, EncryptionKey, InstallConfig, InstallDiskSelector,
    InstallExtension, SystemDiskEncryption,
};
pub use kubelet::{KubeletConfig, NodeTaint, RegistriesConfig, RegistryAuth, RegistryMirror};
pub use machine::{
    FileOp, KernelConfig, KernelModule, MachineConfig, MachineFile, SeccompProfile, Sysctl,
    TimeConfig, UdevConfig, UserDisk,
};
pub use network::{Bond, BondMode, Device, DhcpOptions, NetworkConfig, Route, Vlan};
pub use validation::{ValidationError, ValidationMode, ValidationReport, Validator};

pub use os_kernel::machine_type::MachineType;
pub use os_kernel::{Error, Result};

/// The `apiVersion` value identifying this document kind.
pub const API_VERSION: &str = "v1alpha1";

/// The top-level v1alpha1 config document: `machine:` + `cluster:`.
///
/// Mirrors `v1alpha1.Config` (`pkg/machinery/config/types/v1alpha1`). Every
/// node's machine config is one of these documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1Alpha1Config {
    /// Whether persistence of the config to disk is enabled (`persist`).
    pub persist: bool,
    /// The `machine:` sub-tree.
    pub machine: machine::MachineConfig,
    /// The `cluster:` sub-tree.
    pub cluster: cluster::ClusterConfig,
}

impl Default for V1Alpha1Config {
    fn default() -> Self {
        V1Alpha1Config {
            persist: true,
            machine: machine::MachineConfig::default(),
            cluster: cluster::ClusterConfig::default(),
        }
    }
}

impl V1Alpha1Config {
    /// Build a config from a machine and cluster sub-tree.
    pub fn new(machine: machine::MachineConfig, cluster: cluster::ClusterConfig) -> Self {
        V1Alpha1Config {
            persist: true,
            machine,
            cluster,
        }
    }

    /// The `apiVersion` of this document.
    pub fn api_version(&self) -> &'static str {
        API_VERSION
    }

    /// The node's machine type.
    pub fn machine_type(&self) -> MachineType {
        self.machine.machine_type
    }

    /// Whether this node is a control-plane node.
    pub fn is_control_plane(&self) -> bool {
        self.machine.is_control_plane()
    }

    /// Apply Talos defaults across both sub-trees. The kubelet's cluster DNS is
    /// derived from the cluster's first service subnet.
    pub fn apply_defaults(&mut self) {
        let service_subnet = self
            .cluster
            .primary_service_subnet()
            .map(ToString::to_string);
        self.cluster.apply_defaults();
        self.machine.apply_defaults(service_subnet.as_deref());
    }
}

impl Validator for V1Alpha1Config {
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
        self.machine.validate_into(mode, report);
        self.cluster.validate_into(mode, report);

        // Cross-cutting rules that need both sub-trees.

        // etcd is only meaningful (and only validated as "must have an image
        // referencing a control-plane component) on control-plane nodes; a
        // worker carrying an etcd block is suspicious.
        if !self.machine.is_control_plane()
            && (!self.cluster.etcd.advertised_subnets.is_empty()
                || !self.cluster.etcd.extra_args.is_empty())
        {
            report.push(ValidationError::Warning(
                "cluster.etcd is configured on a worker node and will be ignored".to_string(),
            ));
        }

        // The pod and service subnets must not overlap on their network prefix
        // (a coarse check: identical CIDR strings are a definite conflict).
        for pod in &self.cluster.pod_subnets {
            if self.cluster.service_subnets.iter().any(|s| s == pod) {
                report.push(ValidationError::Conflict(format!(
                    "pod subnet {pod} also used as a service subnet"
                )));
            }
        }

        // A control-plane node with scheduling disabled and no workers in the
        // cluster would be unschedulable, but that's a cluster-wide concern; we
        // only warn when a worker config carries control-plane-only blocks.
        if !self.machine.is_control_plane()
            && !self.cluster.apiserver.component.extra_args.is_empty()
        {
            report.push(ValidationError::Warning(
                "cluster.apiServer is configured on a worker node and will be ignored".to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_control_plane() -> V1Alpha1Config {
        let mut m = machine::MachineConfig::new(MachineType::ControlPlane);
        m.token = "tok".to_string();
        m.ca_crt = "ca-pem".to_string();
        m.install.disk = "/dev/sda".to_string();

        let cluster = cluster::ClusterConfig::new(
            "prod",
            cluster::ControlPlaneEndpoint::parse("https://10.0.0.1:6443").unwrap(),
        );

        let mut cfg = V1Alpha1Config::new(m, cluster);
        cfg.apply_defaults();
        cfg
    }

    #[test]
    fn full_control_plane_validates_clean() {
        let cfg = full_control_plane();
        let warnings = cfg.validate(ValidationMode::Metal).unwrap();
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
    }

    #[test]
    fn api_version_and_type() {
        let cfg = full_control_plane();
        assert_eq!(cfg.api_version(), "v1alpha1");
        assert_eq!(cfg.machine_type(), MachineType::ControlPlane);
        assert!(cfg.is_control_plane());
    }

    #[test]
    fn defaulting_derives_kubelet_dns_from_service_subnet() {
        let cfg = full_control_plane();
        assert_eq!(
            cfg.machine.kubelet.cluster_dns,
            vec!["10.96.0.10".to_string()]
        );
        assert_eq!(cfg.machine.install.image, defaults::DEFAULT_INSTALL_IMAGE);
    }

    #[test]
    fn empty_config_fails_on_metal() {
        let cfg = V1Alpha1Config::default();
        let err = cfg.validate(ValidationMode::Metal).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn pod_service_subnet_overlap_conflict() {
        let mut cfg = full_control_plane();
        cfg.cluster.pod_subnets = vec!["10.96.0.0/12".to_string()];
        let err = cfg.validate(ValidationMode::Metal).unwrap_err();
        assert!(err.to_string().contains("subnet"));
    }

    #[test]
    fn worker_with_etcd_block_warns_not_fatal() {
        let mut m = machine::MachineConfig::new(MachineType::Worker);
        m.token = "tok".to_string();
        m.ca_crt = "ca".to_string();
        m.install.disk = "/dev/sda".to_string();
        let cluster = cluster::ClusterConfig::new(
            "prod",
            cluster::ControlPlaneEndpoint::parse("https://10.0.0.1:6443").unwrap(),
        );
        let mut cfg = V1Alpha1Config::new(m, cluster);
        cfg.apply_defaults();
        cfg.cluster.etcd.advertised_subnets = vec!["10.0.0.0/8".to_string()];
        let warnings = cfg.validate(ValidationMode::Metal).unwrap();
        assert!(warnings.iter().any(|w| w.contains("etcd")));
    }

    #[test]
    fn container_mode_tolerates_missing_disk() {
        let mut m = machine::MachineConfig::new(MachineType::ControlPlane);
        m.token = "tok".to_string();
        m.ca_crt = "ca".to_string();
        let cluster = cluster::ClusterConfig::new(
            "prod",
            cluster::ControlPlaneEndpoint::parse("https://10.0.0.1:6443").unwrap(),
        );
        let mut cfg = V1Alpha1Config::new(m, cluster);
        cfg.apply_defaults();
        // No install disk.
        assert!(cfg.validate(ValidationMode::Container).is_ok());
        assert!(cfg.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn persist_defaults_true() {
        assert!(V1Alpha1Config::default().persist);
    }
}
