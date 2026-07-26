//! The `machine:` sub-tree aggregator: machine type, identity, install, network,
//! kubelet, registries, plus the many leaf settings Talos hangs off `machine:`
//! (files, env, sysctls, time, disks, seccomp profiles, kernel modules, udev).
//!
//! Mirrors `MachineConfig` in `pkg/machinery/config/types/v1alpha1`.

use crate::install::{InstallConfig, SystemDiskEncryption};
use crate::kubelet::{KubeletConfig, RegistriesConfig};
use crate::network::NetworkConfig;
use crate::validation::{
    ValidationError, ValidationMode, ValidationReport, Validator, is_hostname, is_identifier,
};
use os_kernel::machine_type::MachineType;

/// File-write operation mode (`machine.files[].op`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOp {
    /// Create the file, failing if it exists.
    Create,
    /// Append to the file.
    Append,
    /// Overwrite the file.
    Overwrite,
}

impl FileOp {
    /// The canonical string used in configs.
    pub fn as_str(self) -> &'static str {
        match self {
            FileOp::Create => "create",
            FileOp::Append => "append",
            FileOp::Overwrite => "overwrite",
        }
    }
}

/// A file Talos writes onto the host (`machine.files[]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineFile {
    /// Absolute destination path.
    pub path: String,
    /// File content.
    pub content: String,
    /// Octal permissions (e.g. `0o644`).
    pub permissions: u32,
    /// Write operation.
    pub op: FileOp,
}

impl Validator for MachineFile {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if !self.path.starts_with('/') {
            report.push(ValidationError::invalid(
                "machine.files[].path",
                format!("'{}' must be an absolute path", self.path),
            ));
        }
        if self.permissions > 0o777 {
            report.push(ValidationError::invalid(
                "machine.files[].permissions",
                "out of range (must be <= 0o777)",
            ));
        }
    }
}

/// A sysctl key/value pair (`machine.sysctls`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sysctl {
    pub key: String,
    pub value: String,
}

/// A timeserver / NTP configuration (`machine.time`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TimeConfig {
    /// Whether time sync is disabled.
    pub disabled: bool,
    /// NTP servers.
    pub servers: Vec<String>,
    /// Max acceptable boot-time drift before refusing to start (seconds).
    pub boot_timeout_secs: u32,
}

impl Validator for TimeConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        for s in &self.servers {
            if !is_hostname(s) && !crate::validation::is_ip(s) {
                report.push(ValidationError::invalid(
                    "machine.time.servers",
                    format!("'{s}' is not a hostname or IP"),
                ));
            }
        }
    }
}

/// A user-defined extra disk to partition and mount (`machine.disks[]`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserDisk {
    /// Block device.
    pub device: String,
    /// Partitions: (mountpoint, size-bytes; 0 = rest of disk).
    pub partitions: Vec<(String, u64)>,
}

impl Validator for UserDisk {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if !self.device.starts_with("/dev/") {
            report.push(ValidationError::invalid(
                "machine.disks[].device",
                format!("'{}' must be a device path", self.device),
            ));
        }
        // Only the final partition may have size 0 (the "rest of disk" sentinel).
        for (i, (mount, size)) in self.partitions.iter().enumerate() {
            if !mount.starts_with('/') {
                report.push(ValidationError::invalid(
                    "machine.disks[].partitions[].mountpoint",
                    format!("'{mount}' must be absolute"),
                ));
            }
            if *size == 0 && i + 1 != self.partitions.len() {
                report.push(ValidationError::invalid(
                    "machine.disks[].partitions[].size",
                    "only the last partition may omit a size",
                ));
            }
        }
    }
}

/// A seccomp profile to install (`machine.seccompProfiles[]`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeccompProfile {
    /// Profile name (file dropped under the kubelet seccomp dir).
    pub name: String,
    /// The raw profile JSON value (stored as a string here).
    pub value: String,
}

/// Kernel module to load (`machine.kernel.modules[]`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KernelModule {
    /// Module name.
    pub name: String,
    /// Module parameters.
    pub parameters: Vec<String>,
}

/// The `machine.kernel` sub-tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KernelConfig {
    /// Modules to load.
    pub modules: Vec<KernelModule>,
}

/// The `machine.udev` sub-tree: extra udev rules.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UdevConfig {
    /// Raw udev rule lines.
    pub rules: Vec<String>,
}

/// The `machine:` sub-tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineConfig {
    /// Node role (`machine.type`).
    pub machine_type: MachineType,
    /// Join token (`machine.token`).
    pub token: String,
    /// PEM machine CA certificate (`machine.ca.crt`).
    pub ca_crt: String,
    /// Extra cert SANs (`machine.certSANs`).
    pub cert_sans: Vec<String>,
    /// Install config.
    pub install: InstallConfig,
    /// Network config.
    pub network: NetworkConfig,
    /// Kubelet config.
    pub kubelet: KubeletConfig,
    /// Container registries.
    pub registries: RegistriesConfig,
    /// System disk encryption.
    pub system_disk_encryption: SystemDiskEncryption,
    /// Static files to write.
    pub files: Vec<MachineFile>,
    /// Environment variables for system services (`machine.env`).
    pub env: Vec<(String, String)>,
    /// Sysctls.
    pub sysctls: Vec<Sysctl>,
    /// Time / NTP config.
    pub time: TimeConfig,
    /// Extra user disks.
    pub disks: Vec<UserDisk>,
    /// Seccomp profiles.
    pub seccomp_profiles: Vec<SeccompProfile>,
    /// Kernel modules.
    pub kernel: KernelConfig,
    /// Udev rules.
    pub udev: UdevConfig,
}

impl Default for MachineConfig {
    fn default() -> Self {
        MachineConfig {
            machine_type: MachineType::Unknown,
            token: String::new(),
            ca_crt: String::new(),
            cert_sans: Vec::new(),
            install: InstallConfig::default(),
            network: NetworkConfig::default(),
            kubelet: KubeletConfig::default(),
            registries: RegistriesConfig::default(),
            system_disk_encryption: SystemDiskEncryption::default(),
            files: Vec::new(),
            env: Vec::new(),
            sysctls: Vec::new(),
            time: TimeConfig::default(),
            disks: Vec::new(),
            seccomp_profiles: Vec::new(),
            kernel: KernelConfig::default(),
            udev: UdevConfig::default(),
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

    /// Apply Talos defaults to the machine sub-tree given the cluster service
    /// subnet (used for kubelet `clusterDNS`).
    pub fn apply_defaults(&mut self, service_subnet: Option<&str>) {
        self.install.apply_defaults();
        self.network.apply_defaults();
        self.kubelet.apply_defaults(service_subnet);
    }

    /// Look up an env var.
    pub fn env_var(&self, key: &str) -> Option<&str> {
        self.env
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
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
        if self.ca_crt.is_empty() {
            report.push(ValidationError::Warning(
                "machine.ca is empty; node cannot serve the Talos API".to_string(),
            ));
        }
        for san in &self.cert_sans {
            if !is_hostname(san) && !crate::validation::is_ip(san) {
                report.push(ValidationError::invalid(
                    "machine.certSANs",
                    format!("'{san}' is not a hostname or IP"),
                ));
            }
        }
        self.install.validate_into(mode, report);
        self.network.validate_into(mode, report);
        self.kubelet.validate_into(mode, report);
        self.registries.validate_into(mode, report);
        if self.system_disk_encryption.is_enabled() {
            self.system_disk_encryption.validate_into(mode, report);
        }
        for f in &self.files {
            f.validate_into(mode, report);
        }
        for (k, _) in &self.env {
            if !is_identifier(k) {
                report.push(ValidationError::invalid(
                    "machine.env",
                    format!("'{k}' is not a valid env var name"),
                ));
            }
        }
        for s in &self.sysctls {
            if !is_identifier(&s.key) {
                report.push(ValidationError::invalid(
                    "machine.sysctls",
                    format!("'{}' is not a valid sysctl key", s.key),
                ));
            }
        }
        self.time.validate_into(mode, report);
        for d in &self.disks {
            d.validate_into(mode, report);
        }
        // A worker may not have a system-disk-encryption STATE provider that
        // depends on KMS without an endpoint; covered by encryption validation.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_cp() -> MachineConfig {
        let mut m = MachineConfig::new(MachineType::ControlPlane);
        m.token = "tok".to_string();
        m.ca_crt = "ca-pem".to_string();
        m.install.disk = "/dev/sda".to_string();
        m.apply_defaults(Some("10.96.0.0/12"));
        m
    }

    #[test]
    fn minimal_control_plane_validates() {
        let m = minimal_cp();
        assert!(m.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn missing_type_and_token_fail() {
        let m = MachineConfig::default();
        let err = m.validate(ValidationMode::Generate).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn file_path_must_be_absolute() {
        let mut m = minimal_cp();
        m.files.push(MachineFile {
            path: "relative".to_string(),
            content: "x".to_string(),
            permissions: 0o644,
            op: FileOp::Create,
        });
        assert!(m.validate(ValidationMode::Metal).is_err());
        m.files[0].path = "/etc/foo".to_string();
        assert!(m.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn file_permissions_range() {
        let f = MachineFile {
            path: "/etc/x".to_string(),
            content: String::new(),
            permissions: 0o7777,
            op: FileOp::Overwrite,
        };
        assert!(f.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn env_lookup_and_validation() {
        let mut m = minimal_cp();
        m.env
            .push(("HTTP_PROXY".to_string(), "http://p:3128".to_string()));
        assert_eq!(m.env_var("HTTP_PROXY"), Some("http://p:3128"));
        assert!(m.validate(ValidationMode::Metal).is_ok());
        m.env.push(("bad name".to_string(), "v".to_string()));
        assert!(m.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn sysctl_key_validation() {
        let mut m = minimal_cp();
        m.sysctls.push(Sysctl {
            key: "net.ipv4.ip_forward".to_string(),
            value: "1".to_string(),
        });
        assert!(m.validate(ValidationMode::Metal).is_ok());
        m.sysctls.push(Sysctl {
            key: "bad key".to_string(),
            value: "1".to_string(),
        });
        assert!(m.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn user_disk_last_partition_sizeless() {
        let mut d = UserDisk {
            device: "/dev/sdb".to_string(),
            partitions: vec![],
        };
        d.partitions.push(("/var/mnt/a".to_string(), 1024));
        d.partitions.push(("/var/mnt/b".to_string(), 0)); // last -> ok
        assert!(d.validate(ValidationMode::Metal).is_ok());
        d.partitions.insert(0, ("/var/mnt/c".to_string(), 0)); // non-last sizeless -> err
        assert!(d.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn time_servers_validated() {
        let mut t = TimeConfig {
            servers: vec!["pool.ntp.org".to_string(), "10.0.0.1".to_string()],
            ..TimeConfig::default()
        };
        assert!(t.validate(ValidationMode::Metal).is_ok());
        t.servers.push("bad host".to_string());
        assert!(t.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn cert_sans_validated() {
        let mut m = minimal_cp();
        m.cert_sans = vec!["api.example.com".to_string(), "10.0.0.1".to_string()];
        assert!(m.validate(ValidationMode::Metal).is_ok());
        m.cert_sans.push("not valid!".to_string());
        assert!(m.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn file_op_strings() {
        assert_eq!(FileOp::Append.as_str(), "append");
        assert_eq!(FileOp::Create.as_str(), "create");
    }
}
