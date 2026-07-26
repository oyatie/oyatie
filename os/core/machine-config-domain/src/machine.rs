//! The `machine:` sub-tree of the v1alpha1 config: machine type, install
//! configuration, node identity (token / CA), kubelet, and the machine network.
//!
//! Mirrors the Talos `MachineConfig` / `InstallConfig` types in
//! `pkg/machinery/config/types/v1alpha1`.

use crate::validation::{ValidationError, ValidationMode, ValidationReport, Validator};
use crate::volume_config::EncryptionSpec;
use os_kernel::error::{Error, Result};
use os_kernel::machine_type::MachineType;

/// Disk install configuration (`machine.install`).
///
/// Mirrors `InstallConfig`: the target disk, the boot image, whether to wipe on
/// install, and extra kernel args.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstallConfig {
    /// Target block device (e.g. `/dev/sda`).
    pub disk: String,
    /// Installer image reference.
    pub image: String,
    /// Whether to wipe the disk before installing.
    pub wipe: bool,
    /// Additional kernel command-line arguments appended at install.
    pub extra_kernel_args: Vec<String>,
}

impl InstallConfig {
    /// Build an install config targeting `disk` with `image`.
    pub fn new(disk: impl Into<String>, image: impl Into<String>) -> Self {
        InstallConfig {
            disk: disk.into(),
            image: image.into(),
            wipe: false,
            extra_kernel_args: Vec::new(),
        }
    }

    /// Whether a disk has been configured.
    pub fn has_disk(&self) -> bool {
        !self.disk.is_empty()
    }

    /// Validate the disk path is an absolute device node, mirroring the Talos
    /// requirement that `machine.install.disk` be a `/dev/...` path.
    pub fn validate_disk(&self) -> Result<()> {
        if self.disk.is_empty() {
            return Err(Error::invalid("machine.install.disk is empty"));
        }
        if !self.disk.starts_with("/dev/") {
            return Err(Error::invalid(format!(
                "machine.install.disk '{}' must be an absolute device path",
                self.disk
            )));
        }
        Ok(())
    }
}

/// Kubelet configuration (`machine.kubelet`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KubeletConfig {
    /// Kubelet container image.
    pub image: String,
    /// Extra `--node-labels` style labels.
    pub extra_args: Vec<(String, String)>,
    /// Cluster DNS server addresses.
    pub cluster_dns: Vec<String>,
}

impl KubeletConfig {
    /// Look up an extra arg by key.
    pub fn extra_arg(&self, key: &str) -> Option<&str> {
        self.extra_args
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// DHCP behavior attached to a legacy v1alpha1 interface.
///
/// Mirrors Talos `DHCPOptions`: absent `ipv4` means enabled, absent `ipv6`
/// means disabled, and `routeMetric = 0` delegates to the network operator
/// default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DhcpOptions {
    /// Route metric override (`0` means use the operator default).
    pub route_metric: u32,
    /// Optional IPv4 DHCP toggle.
    pub ipv4: Option<bool>,
    /// Optional IPv6 DHCP toggle.
    pub ipv6: Option<bool>,
    /// Raw DHCPv6 DUID encoded as hex (`duidv6`).
    pub duid_v6: String,
}

impl DhcpOptions {
    /// Whether DHCPv4 is enabled after applying Talos defaults.
    pub fn ipv4(&self) -> bool {
        self.ipv4.unwrap_or(true)
    }

    /// Whether DHCPv6 is enabled after applying Talos defaults.
    pub fn ipv6(&self) -> bool {
        self.ipv6.unwrap_or(false)
    }

    /// Effective route metric after applying a caller-provided default.
    pub fn route_metric_or(&self, default_metric: u32) -> u32 {
        if self.route_metric == 0 {
            default_metric
        } else {
            self.route_metric
        }
    }

    fn validate_into(&self, report: &mut ValidationReport, field: &str) {
        if !self.duid_v6.is_empty()
            && (!self.duid_v6.len().is_multiple_of(2)
                || !self.duid_v6.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            report.push(ValidationError::invalid(
                format!("{field}.duidv6"),
                "must be an even-length hexadecimal string",
            ));
        }
    }
}

/// A legacy `machine.network.interfaces[]` entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkInterface {
    /// Interface name (e.g. `eth0`).
    pub interface: String,
    /// Whether DHCP is enabled for this interface.
    pub dhcp: bool,
    /// Whether Talos should ignore this interface for automatic operators.
    pub ignore: bool,
    /// Optional DHCP behavior.
    pub dhcp_options: DhcpOptions,
    /// VLAN subinterfaces configured under this interface.
    pub vlans: Vec<NetworkVlan>,
}

impl NetworkInterface {
    /// A DHCP interface by name.
    pub fn dhcp(interface: impl Into<String>) -> Self {
        NetworkInterface {
            interface: interface.into(),
            dhcp: true,
            ignore: false,
            dhcp_options: DhcpOptions::default(),
            vlans: Vec::new(),
        }
    }
}

/// A legacy `machine.network.interfaces[].vlans[]` entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkVlan {
    /// VLAN identifier (`vlanId`).
    pub vlan_id: u16,
    /// Whether DHCP is enabled for this VLAN.
    pub dhcp: bool,
    /// Optional DHCP behavior for this VLAN.
    pub dhcp_options: DhcpOptions,
}

/// The legacy `machine.network` sub-tree subset decoded by this crate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkConfig {
    /// Interface configuration entries.
    pub interfaces: Vec<NetworkInterface>,
}

/// The legacy `machine.features` sub-tree subset decoded by this crate.
///
/// Mirrors Talos `FeaturesConfig` fields that downstream controllers consume.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MachineFeatures {
    /// Source `.machine.features.diskQuotaSupport`.
    pub disk_quota_support: bool,
}

impl MachineFeatures {
    /// Source `FeaturesConfig.DiskQuotaSupportEnabled()`.
    pub fn disk_quota_support_enabled(&self) -> bool {
        self.disk_quota_support
    }
}

/// Legacy v1alpha1 system-disk encryption (`machine.systemDiskEncryption`).
///
/// Source: `pkg/machinery/config/types/v1alpha1.SystemDiskEncryptionConfig`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemDiskEncryption {
    /// STATE partition encryption fallback.
    pub state: Option<EncryptionSpec>,
    /// EPHEMERAL partition encryption fallback.
    pub ephemeral: Option<EncryptionSpec>,
}

impl SystemDiskEncryption {
    /// Whether either system partition carries legacy encryption config.
    pub fn is_enabled(&self) -> bool {
        self.state.is_some() || self.ephemeral.is_some()
    }

    /// Look up a legacy encryption config by source partition label.
    pub fn get(&self, partition_label: &str) -> Option<&EncryptionSpec> {
        match partition_label {
            "STATE" => self.state.as_ref(),
            "EPHEMERAL" => self.ephemeral.as_ref(),
            _ => None,
        }
    }
}

/// The `machine:` sub-tree.
///
/// Mirrors `MachineConfig`: the node role (`machine.type`), the join token, the
/// machine CA, the install block, and the kubelet block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineConfig {
    /// Node role.
    pub machine_type: MachineType,
    /// The join token (`machine.token`).
    pub token: String,
    /// The PEM machine CA certificate (`machine.ca.crt`).
    pub ca_crt: String,
    /// The machine certificate SANs (extra hostnames/IPs the API cert covers).
    pub cert_sans: Vec<String>,
    /// Install configuration.
    pub install: InstallConfig,
    /// Kubelet configuration.
    pub kubelet: KubeletConfig,
    /// Machine network configuration.
    pub network: NetworkConfig,
    /// Machine feature toggles.
    pub features: MachineFeatures,
    /// Legacy system-disk encryption fallback.
    pub system_disk_encryption: SystemDiskEncryption,
}

impl Default for MachineConfig {
    fn default() -> Self {
        MachineConfig {
            machine_type: MachineType::Unknown,
            token: String::new(),
            ca_crt: String::new(),
            cert_sans: Vec::new(),
            install: InstallConfig::default(),
            kubelet: KubeletConfig::default(),
            network: NetworkConfig::default(),
            features: MachineFeatures::default(),
            system_disk_encryption: SystemDiskEncryption::default(),
        }
    }
}

impl MachineConfig {
    /// A new machine config with the given role.
    pub fn new(machine_type: MachineType) -> Self {
        MachineConfig {
            machine_type,
            ..Default::default()
        }
    }

    /// Whether this node participates in the control plane.
    pub fn is_control_plane(&self) -> bool {
        self.machine_type.is_control_plane()
    }
}

impl Validator for MachineConfig {
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
        if self.machine_type == MachineType::Unknown {
            report.push(ValidationError::missing("machine.type"));
        }
        if self.token.is_empty() {
            report.push(ValidationError::missing("machine.token"));
        }
        // Install disk requirements depend on the runtime mode.
        if mode.requires_install_disk() {
            if !self.install.has_disk() {
                report.push(ValidationError::missing("machine.install.disk"));
            } else if let Err(e) = self.install.validate_disk() {
                report.push(ValidationError::invalid(
                    "machine.install.disk",
                    e.to_string(),
                ));
            }
        } else if self.install.has_disk() {
            // Even when not required, a present disk must be well-formed.
            if let Err(e) = self.install.validate_disk() {
                report.push(ValidationError::invalid(
                    "machine.install.disk",
                    e.to_string(),
                ));
            }
        }
        if self.ca_crt.is_empty() {
            report.push(ValidationError::Warning(
                "machine.ca is empty; node cannot serve the Talos API".to_string(),
            ));
        }
        for interface in &self.network.interfaces {
            interface
                .dhcp_options
                .validate_into(report, "machine.network.interfaces[].dhcpOptions");
            for vlan in &interface.vlans {
                if vlan.vlan_id == 0 || vlan.vlan_id > 4094 {
                    report.push(ValidationError::invalid(
                        "machine.network.interfaces[].vlans[].vlanId",
                        format!("VLAN id {} out of range 1..=4094", vlan.vlan_id),
                    ));
                }
                vlan.dhcp_options
                    .validate_into(report, "machine.network.interfaces[].vlans[].dhcpOptions");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_disk_validation() {
        let mut i = InstallConfig::new("/dev/sda", "img");
        assert!(i.validate_disk().is_ok());
        i.disk = "sda".to_string();
        assert!(i.validate_disk().is_err());
        i.disk = String::new();
        assert!(i.validate_disk().is_err());
        assert!(!i.has_disk());
    }

    #[test]
    fn metal_requires_disk() {
        let mut m = MachineConfig::new(MachineType::ControlPlane);
        m.token = "tok".to_string();
        m.ca_crt = "ca".to_string();
        // No disk -> fails on metal.
        assert!(m.validate(ValidationMode::Metal).is_err());
        // Container mode tolerates missing disk.
        assert!(m.validate(ValidationMode::Container).is_ok());
        m.install = InstallConfig::new("/dev/sda", "img");
        assert!(m.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn malformed_disk_fails_even_in_container() {
        let mut m = MachineConfig::new(MachineType::Worker);
        m.token = "tok".to_string();
        m.ca_crt = "ca".to_string();
        m.install = InstallConfig::new("relative", "img");
        assert!(m.validate(ValidationMode::Container).is_err());
    }

    #[test]
    fn unknown_type_and_missing_token_reported() {
        let m = MachineConfig::default();
        let err = m.validate(ValidationMode::Generate).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn control_plane_flag() {
        assert!(MachineConfig::new(MachineType::ControlPlane).is_control_plane());
        assert!(MachineConfig::new(MachineType::Init).is_control_plane());
        assert!(!MachineConfig::new(MachineType::Worker).is_control_plane());
    }

    #[test]
    fn kubelet_extra_arg_lookup() {
        let mut k = KubeletConfig::default();
        k.extra_args
            .push(("max-pods".to_string(), "110".to_string()));
        assert_eq!(k.extra_arg("max-pods"), Some("110"));
        assert_eq!(k.extra_arg("missing"), None);
    }

    #[test]
    fn dhcp_options_follow_legacy_defaults() {
        let opts = DhcpOptions::default();
        assert!(opts.ipv4());
        assert!(!opts.ipv6());
        assert_eq!(opts.route_metric_or(1024), 1024);
    }

    #[test]
    fn dhcp_options_reject_bad_duid_hex() {
        let mut m = MachineConfig::new(MachineType::Worker);
        m.token = "tok".to_string();
        m.ca_crt = "ca".to_string();
        m.network.interfaces.push(NetworkInterface {
            interface: "eth0".to_string(),
            dhcp: true,
            ignore: false,
            dhcp_options: DhcpOptions {
                duid_v6: "bad-hex".to_string(),
                ..DhcpOptions::default()
            },
            vlans: Vec::new(),
        });

        assert!(m.validate(ValidationMode::Container).is_err());
    }

    #[test]
    fn vlan_id_range_is_validated() {
        let mut m = MachineConfig::new(MachineType::Worker);
        m.token = "tok".to_string();
        m.ca_crt = "ca".to_string();
        m.network.interfaces.push(NetworkInterface {
            interface: "eth0".to_string(),
            dhcp: false,
            ignore: false,
            dhcp_options: DhcpOptions::default(),
            vlans: vec![NetworkVlan {
                vlan_id: 4095,
                dhcp: true,
                dhcp_options: DhcpOptions::default(),
            }],
        });

        assert!(m.validate(ValidationMode::Container).is_err());
    }

    #[test]
    fn vlan_dhcp_options_reject_bad_duid_hex() {
        let mut m = MachineConfig::new(MachineType::Worker);
        m.token = "tok".to_string();
        m.ca_crt = "ca".to_string();
        m.network.interfaces.push(NetworkInterface {
            interface: "eth0".to_string(),
            dhcp: false,
            ignore: false,
            dhcp_options: DhcpOptions::default(),
            vlans: vec![NetworkVlan {
                vlan_id: 100,
                dhcp: true,
                dhcp_options: DhcpOptions {
                    duid_v6: "abc".to_string(),
                    ..DhcpOptions::default()
                },
            }],
        });

        assert!(m.validate(ValidationMode::Container).is_err());
    }
}
