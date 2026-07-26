//! The `machine.install` sub-tree plus disk encryption, mirroring
//! `InstallConfig`, `InstallDiskSelector`, `InstallExtensionConfig`, and the
//! `SystemDiskEncryptionConfig` / `EncryptionConfig` types in
//! `pkg/machinery/config/types/v1alpha1`.

use crate::defaults;
use crate::validation::{
    ValidationError, ValidationMode, ValidationReport, Validator, is_image_ref,
};

/// Disk-selector matcher (`machine.install.diskSelector`). Talos can pick an
/// install disk by matching one or more attributes instead of a fixed path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstallDiskSelector {
    /// Match by disk size expression, e.g. `>= 120GB`.
    pub size: Option<String>,
    /// Match by model string.
    pub model: Option<String>,
    /// Match by serial number.
    pub serial: Option<String>,
    /// Match by bus type (e.g. `nvme`, `sata`).
    pub bus_type: Option<String>,
    /// Match by device name glob (e.g. `/dev/nvme*`).
    pub name: Option<String>,
}

impl InstallDiskSelector {
    /// Whether any matcher field is populated.
    pub fn is_empty(&self) -> bool {
        self.size.is_none()
            && self.model.is_none()
            && self.serial.is_none()
            && self.bus_type.is_none()
            && self.name.is_none()
    }
}

/// One encryption key provider. Talos supports a static passphrase, a
/// node-identity-derived key, a KMS server, or a TPM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionKey {
    /// Static passphrase (`key.static.passphrase`).
    Static { passphrase: String, slot: u8 },
    /// Key derived from the node identity (`key.nodeID`).
    NodeId { slot: u8 },
    /// Remote KMS endpoint (`key.kms.endpoint`).
    Kms { endpoint: String, slot: u8 },
    /// TPM-sealed key (`key.tpm`).
    Tpm { slot: u8 },
}

impl EncryptionKey {
    /// The LUKS key slot this provider occupies.
    pub fn slot(&self) -> u8 {
        match self {
            EncryptionKey::Static { slot, .. }
            | EncryptionKey::NodeId { slot }
            | EncryptionKey::Kms { slot, .. }
            | EncryptionKey::Tpm { slot } => *slot,
        }
    }
}

/// Encryption cipher for a partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EncryptionCipher {
    /// `aes-xts-plain64` (Talos default).
    #[default]
    AesXtsPlain64,
    /// `xchacha12,aes-adiantum-plain64`.
    XChaCha12,
    /// `xchacha20,aes-adiantum-plain64`.
    XChaCha20,
}

impl EncryptionCipher {
    /// The cryptsetup cipher string.
    pub fn as_str(self) -> &'static str {
        match self {
            EncryptionCipher::AesXtsPlain64 => "aes-xts-plain64",
            EncryptionCipher::XChaCha12 => "xchacha12,aes-adiantum-plain64",
            EncryptionCipher::XChaCha20 => "xchacha20,aes-adiantum-plain64",
        }
    }
}

/// Per-partition encryption config (`machine.systemDiskEncryption.{state,ephemeral}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionConfig {
    /// Provider: must be `luks2` for Talos.
    pub provider: String,
    /// Key providers; at least one is required.
    pub keys: Vec<EncryptionKey>,
    /// Cipher.
    pub cipher: EncryptionCipher,
    /// Optional explicit key size in bits.
    pub key_size: Option<u32>,
    /// Optional PBKDF2 iteration / memory tuning expressed as a free-form string.
    pub block_size: Option<u64>,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        EncryptionConfig {
            provider: "luks2".to_string(),
            keys: Vec::new(),
            cipher: EncryptionCipher::default(),
            key_size: None,
            block_size: None,
        }
    }
}

impl Validator for EncryptionConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if self.provider != "luks2" {
            report.push(ValidationError::invalid(
                "machine.systemDiskEncryption.provider",
                format!(
                    "unsupported provider '{}', only luks2 is supported",
                    self.provider
                ),
            ));
        }
        if self.keys.is_empty() {
            report.push(ValidationError::missing(
                "machine.systemDiskEncryption.keys",
            ));
        }
        // Each slot must be unique.
        let mut slots: Vec<u8> = self.keys.iter().map(EncryptionKey::slot).collect();
        slots.sort_unstable();
        for w in slots.windows(2) {
            if w[0] == w[1] {
                report.push(ValidationError::Conflict(format!(
                    "duplicate encryption key slot {}",
                    w[0]
                )));
            }
        }
        for key in &self.keys {
            if let EncryptionKey::Static { passphrase, .. } = key
                && passphrase.is_empty() {
                    report.push(ValidationError::missing(
                        "machine.systemDiskEncryption.keys[].static.passphrase",
                    ));
                }
            if let EncryptionKey::Kms { endpoint, .. } = key
                && endpoint.is_empty() {
                    report.push(ValidationError::missing(
                        "machine.systemDiskEncryption.keys[].kms.endpoint",
                    ));
                }
        }
    }
}

/// System-disk encryption (`machine.systemDiskEncryption`): which partitions
/// are encrypted and how.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemDiskEncryption {
    /// Encryption for the STATE partition (holds the machine config).
    pub state: Option<EncryptionConfig>,
    /// Encryption for the EPHEMERAL partition (var/, container storage).
    pub ephemeral: Option<EncryptionConfig>,
}

impl SystemDiskEncryption {
    /// Whether any partition is configured for encryption.
    pub fn is_enabled(&self) -> bool {
        self.state.is_some() || self.ephemeral.is_some()
    }
}

impl Validator for SystemDiskEncryption {
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
        if let Some(s) = &self.state {
            s.validate_into(mode, report);
        }
        if let Some(e) = &self.ephemeral {
            e.validate_into(mode, report);
        }
    }
}

/// A system extension to install at image build / install time
/// (`machine.install.extensions`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstallExtension {
    /// The extension image reference.
    pub image: String,
}

/// The `machine.install` sub-tree.
///
/// Mirrors `InstallConfig`: target disk (or selector), boot image, wipe flag,
/// extra kernel args, and bundled extensions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstallConfig {
    /// Fixed target block device (e.g. `/dev/sda`).
    pub disk: String,
    /// Disk selector (used when `disk` is empty).
    pub disk_selector: InstallDiskSelector,
    /// Installer image reference.
    pub image: String,
    /// Whether to wipe the disk before installing.
    pub wipe: bool,
    /// Whether to install in a way that supports legacy BIOS booting.
    pub legacy_bios: bool,
    /// Additional kernel command-line arguments appended at install.
    pub extra_kernel_args: Vec<String>,
    /// System extensions to bundle.
    pub extensions: Vec<InstallExtension>,
}

impl InstallConfig {
    /// Build an install config targeting `disk` with `image`.
    pub fn new(disk: impl Into<String>, image: impl Into<String>) -> Self {
        InstallConfig {
            disk: disk.into(),
            image: image.into(),
            ..Default::default()
        }
    }

    /// Whether an install target (fixed disk or selector) is configured.
    pub fn has_target(&self) -> bool {
        !self.disk.is_empty() || !self.disk_selector.is_empty()
    }

    /// Apply Talos defaults: fill the image when empty.
    pub fn apply_defaults(&mut self) {
        if self.image.is_empty() {
            self.image = defaults::DEFAULT_INSTALL_IMAGE.to_string();
        }
    }
}

impl Validator for InstallConfig {
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
        if mode.requires_install_disk() && !self.has_target() {
            report.push(ValidationError::missing("machine.install.disk"));
        }
        if !self.disk.is_empty() && !self.disk.starts_with("/dev/") {
            report.push(ValidationError::invalid(
                "machine.install.disk",
                format!("'{}' must be an absolute device path", self.disk),
            ));
        }
        if !self.image.is_empty() && !is_image_ref(&self.image) {
            report.push(ValidationError::invalid(
                "machine.install.image",
                format!("'{}' is not a valid image reference", self.image),
            ));
        }
        for ext in &self.extensions {
            if !is_image_ref(&ext.image) {
                report.push(ValidationError::invalid(
                    "machine.install.extensions[].image",
                    format!("'{}' is not a valid image reference", ext.image),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_requires_target_on_metal() {
        let mut i = InstallConfig::default();
        i.apply_defaults();
        assert!(i.validate(ValidationMode::Metal).is_err());
        assert!(i.validate(ValidationMode::Container).is_ok());
        i.disk = "/dev/sda".to_string();
        assert!(i.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn selector_satisfies_target() {
        let mut i = InstallConfig::default();
        i.disk_selector.size = Some(">= 100GB".to_string());
        i.apply_defaults();
        assert!(i.has_target());
        assert!(i.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn bad_disk_path_rejected() {
        let mut i = InstallConfig::new("sda", defaults::DEFAULT_INSTALL_IMAGE);
        assert!(i.validate(ValidationMode::Container).is_err());
        i.disk = "/dev/nvme0n1".to_string();
        assert!(i.validate(ValidationMode::Container).is_ok());
    }

    #[test]
    fn default_image_applied() {
        let mut i = InstallConfig::default();
        assert!(i.image.is_empty());
        i.apply_defaults();
        assert_eq!(i.image, defaults::DEFAULT_INSTALL_IMAGE);
    }

    #[test]
    fn encryption_requires_keys_and_luks2() {
        let mut e = EncryptionConfig::default();
        assert!(e.validate(ValidationMode::Metal).is_err()); // no keys
        e.keys.push(EncryptionKey::NodeId { slot: 0 });
        assert!(e.validate(ValidationMode::Metal).is_ok());
        e.provider = "luks1".to_string();
        assert!(e.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn encryption_duplicate_slot_conflict() {
        let mut e = EncryptionConfig::default();
        e.keys.push(EncryptionKey::NodeId { slot: 0 });
        e.keys.push(EncryptionKey::Tpm { slot: 0 });
        let err = e.validate(ValidationMode::Metal).unwrap_err();
        assert!(err.to_string().contains("slot"));
    }

    #[test]
    fn empty_static_passphrase_rejected() {
        let mut e = EncryptionConfig::default();
        e.keys.push(EncryptionKey::Static {
            passphrase: String::new(),
            slot: 1,
        });
        assert!(e.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn system_disk_encryption_enabled_flag() {
        let mut sde = SystemDiskEncryption::default();
        assert!(!sde.is_enabled());
        sde.ephemeral = Some(EncryptionConfig::default());
        assert!(sde.is_enabled());
    }

    #[test]
    fn cipher_strings() {
        assert_eq!(EncryptionCipher::AesXtsPlain64.as_str(), "aes-xts-plain64");
        assert!(EncryptionCipher::XChaCha12.as_str().contains("xchacha12"));
    }
}
