//! The read-only [`Provider`] accessor trait, mirroring the Talos
//! `config.Provider` interface that runtime subsystems use to read the active
//! machine configuration without depending on its concrete schema.

use crate::cluster::ClusterConfig;
use crate::machine::MachineConfig;
use os_kernel::machine_type::MachineType;

/// A read-only view over a fully decoded machine configuration.
///
/// Mirrors the Talos `config.Provider`: runtime controllers depend on this
/// trait rather than on `v1alpha1.Config` directly, so the schema can evolve
/// behind a stable accessor surface.
pub trait Provider {
    /// The node role.
    fn machine_type(&self) -> MachineType;

    /// The machine sub-tree.
    fn machine(&self) -> &MachineConfig;

    /// The cluster sub-tree.
    fn cluster(&self) -> &ClusterConfig;

    /// Whether this node is part of the control plane.
    fn is_control_plane(&self) -> bool {
        self.machine_type().is_control_plane()
    }

    /// Whether this node is a worker.
    fn is_worker(&self) -> bool {
        self.machine_type() == MachineType::Worker
    }

    /// The Kubernetes control-plane endpoint URL, if configured.
    fn control_plane_endpoint(&self) -> Option<String> {
        let cluster = self.cluster();
        if cluster.has_endpoint() {
            Some(cluster.endpoint.to_url())
        } else {
            None
        }
    }

    /// The join token, if present.
    fn join_token(&self) -> Option<&str> {
        let t = &self.machine().token;
        if t.is_empty() { None } else { Some(t.as_str()) }
    }
}

impl Provider for crate::v1alpha1::V1Alpha1Config {
    fn machine_type(&self) -> MachineType {
        self.machine.machine_type
    }

    fn machine(&self) -> &MachineConfig {
        &self.machine
    }

    fn cluster(&self) -> &ClusterConfig {
        &self.cluster
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::ControlPlaneEndpoint;
    use crate::v1alpha1::V1Alpha1Config;

    fn cp_config() -> V1Alpha1Config {
        let mut machine = MachineConfig::new(MachineType::ControlPlane);
        machine.token = "abcdef.0123456789abcdef".to_string();
        let cluster = ClusterConfig::new(
            "prod",
            ControlPlaneEndpoint::parse("https://10.0.0.1:6443").unwrap(),
        );
        V1Alpha1Config::new(machine, cluster)
    }

    #[test]
    fn provider_reports_role() {
        let c = cp_config();
        assert!(c.is_control_plane());
        assert!(!c.is_worker());
        assert_eq!(c.machine_type(), MachineType::ControlPlane);
    }

    #[test]
    fn endpoint_accessor() {
        let c = cp_config();
        assert_eq!(
            c.control_plane_endpoint().as_deref(),
            Some("https://10.0.0.1:6443")
        );
        let empty = V1Alpha1Config::default();
        assert_eq!(empty.control_plane_endpoint(), None);
    }

    #[test]
    fn join_token_accessor() {
        let c = cp_config();
        assert_eq!(c.join_token(), Some("abcdef.0123456789abcdef"));
        assert_eq!(V1Alpha1Config::default().join_token(), None);
    }

    #[test]
    fn worker_classification() {
        let mut c = cp_config();
        c.machine.machine_type = MachineType::Worker;
        assert!(c.is_worker());
        assert!(!c.is_control_plane());
    }
}
