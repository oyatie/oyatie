//! The legacy monolithic `v1alpha1` config document: a single document carrying
//! both the `machine:` and `cluster:` sub-trees.
//!
//! Mirrors the Talos `v1alpha1.Config` type — the original (and still default)
//! machine configuration document.

use crate::cluster::ClusterConfig;
use crate::document::{ConfigVersion, Document, DocumentMeta};
use crate::machine::MachineConfig;
use crate::validation::{ValidationMode, ValidationReport, Validator};
use os_kernel::error::Result;
use os_kernel::machine_type::MachineType;

/// The v1alpha1 machine configuration document.
///
/// Holds the persisted version marker, the machine sub-tree, and the cluster
/// sub-tree. Implements both [`Document`] (so it can live in the multi-document
/// [`crate::container::Config`]) and [`Validator`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1Alpha1Config {
    /// Whether the persisted config should be re-applied on the next boot
    /// (`persist`).
    pub persist: bool,
    /// The machine sub-tree.
    pub machine: MachineConfig,
    /// The cluster sub-tree.
    pub cluster: ClusterConfig,
}

impl Default for V1Alpha1Config {
    fn default() -> Self {
        V1Alpha1Config {
            persist: true,
            machine: MachineConfig::default(),
            cluster: ClusterConfig::default(),
        }
    }
}

impl V1Alpha1Config {
    /// Build a config from a machine + cluster sub-tree.
    pub fn new(machine: MachineConfig, cluster: ClusterConfig) -> Self {
        V1Alpha1Config {
            persist: true,
            machine,
            cluster,
        }
    }

    /// The node role declared by the machine sub-tree.
    pub fn machine_type(&self) -> MachineType {
        self.machine.machine_type
    }

    /// Whether this document configures a control-plane node.
    pub fn is_control_plane(&self) -> bool {
        self.machine.is_control_plane()
    }
}

impl Document for V1Alpha1Config {
    fn meta(&self) -> DocumentMeta {
        DocumentMeta::new(ConfigVersion::V1Alpha1, "v1alpha1")
    }

    fn validate_document(&self) -> Result<()> {
        // Document-level validation uses the generate (least strict) mode; the
        // container drives mode-specific validation explicitly.
        self.validate(ValidationMode::Generate).map(|_| ())
    }
}

impl Validator for V1Alpha1Config {
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
        self.machine.validate_into(mode, report);
        self.cluster.validate_into(mode, report);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::ControlPlaneEndpoint;
    use crate::machine::InstallConfig;

    fn good_config() -> V1Alpha1Config {
        let mut machine = MachineConfig::new(MachineType::ControlPlane);
        machine.token = "tok".to_string();
        machine.ca_crt = "ca".to_string();
        machine.install = InstallConfig::new("/dev/sda", "img");
        let cluster = ClusterConfig::new(
            "prod",
            ControlPlaneEndpoint::parse("https://10.0.0.1:6443").unwrap(),
        );
        V1Alpha1Config::new(machine, cluster)
    }

    #[test]
    fn document_meta_is_v1alpha1() {
        let c = V1Alpha1Config::default();
        assert_eq!(c.meta().version, ConfigVersion::V1Alpha1);
        assert_eq!(c.kind(), "v1alpha1");
    }

    #[test]
    fn full_config_validates_on_metal() {
        let c = good_config();
        assert!(c.validate(ValidationMode::Metal).is_ok());
        assert!(c.validate_document().is_ok());
    }

    #[test]
    fn missing_machine_fields_fail_metal() {
        let mut c = good_config();
        c.machine.install = InstallConfig::default();
        assert!(c.validate(ValidationMode::Metal).is_err());
        // Container mode tolerates the missing disk.
        assert!(c.validate(ValidationMode::Container).is_ok());
    }

    #[test]
    fn default_persists() {
        assert!(V1Alpha1Config::default().persist);
    }

    #[test]
    fn role_accessors() {
        let c = good_config();
        assert_eq!(c.machine_type(), MachineType::ControlPlane);
        assert!(c.is_control_plane());
    }
}
