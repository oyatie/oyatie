//! `VolumeConfig` and `UserVolumeConfig` — block volume provisioning.
//!
//! Mirrors `pkg/machinery/config/types/block`. `VolumeConfig` overrides the
//! provisioning of a *system* volume (e.g. `EPHEMERAL`), while
//! `UserVolumeConfig` declares a brand new user volume that Talos partitions,
//! formats, and mounts under `/var/mnt/<name>`.

use crate::document::{ConfigDocument, DocId, DocKind};
use os_kernel::error::{Error, Result};

/// Filesystem type for a provisioned volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filesystem {
    /// ext4.
    Ext4,
    /// XFS (Talos default for user volumes).
    Xfs,
    /// No filesystem (raw block).
    None,
}

impl Filesystem {
    /// Canonical lowercase string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Filesystem::Ext4 => "ext4",
            Filesystem::Xfs => "xfs",
            Filesystem::None => "none",
        }
    }

    /// Parse from a string.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "ext4" => Filesystem::Ext4,
            "xfs" => Filesystem::Xfs,
            "none" | "" => Filesystem::None,
            _ => return None,
        })
    }
}

/// Provisioning parameters: how much disk to allocate, and growth behaviour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provisioning {
    /// Minimum partition size in bytes.
    pub min_size: u64,
    /// Maximum partition size in bytes (`0` = unbounded).
    pub max_size: u64,
    /// Whether the partition should grow to fill available space.
    pub grow: bool,
    /// A disk selector expression (CEL-like in Talos; modeled as opaque here).
    pub disk_selector: Option<String>,
}

impl Provisioning {
    /// Construct a fixed-size, non-growing provisioning request.
    #[must_use]
    pub fn fixed(size: u64) -> Self {
        Provisioning {
            min_size: size,
            max_size: size,
            grow: false,
            disk_selector: None,
        }
    }

    /// Construct a growable provisioning request between min and max bytes.
    #[must_use]
    pub fn growable(min_size: u64, max_size: u64) -> Self {
        Provisioning {
            min_size,
            max_size,
            grow: true,
            disk_selector: None,
        }
    }

    /// Builder: set the disk selector.
    pub fn with_disk_selector(mut self, sel: impl Into<String>) -> Self {
        self.disk_selector = Some(sel.into());
        self
    }

    fn validate(&self) -> Result<()> {
        if self.min_size == 0 {
            return Err(Error::invalid(
                "VolumeConfig: minSize must be greater than zero",
            ));
        }
        if self.max_size != 0 && self.max_size < self.min_size {
            return Err(Error::invalid(format!(
                "VolumeConfig: maxSize {} is smaller than minSize {}",
                self.max_size, self.min_size
            )));
        }
        if let Some(sel) = &self.disk_selector
            && sel.trim().is_empty() {
                return Err(Error::invalid(
                    "VolumeConfig: diskSelector, if set, must be non-empty",
                ));
            }
        Ok(())
    }
}

/// The `VolumeConfig` document (system volume override).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeConfig {
    /// The system volume name (e.g. `EPHEMERAL`, `STATE`).
    pub name: String,
    /// Provisioning override.
    pub provisioning: Provisioning,
}

/// Known reserved system volume names.
const SYSTEM_VOLUMES: &[&str] = &["EPHEMERAL", "STATE", "META", "BOOT"];

impl VolumeConfig {
    /// Construct a system volume override.
    pub fn new(name: impl Into<String>, provisioning: Provisioning) -> Self {
        VolumeConfig {
            name: name.into(),
            provisioning,
        }
    }

    /// Whether `name` is a recognized system volume.
    #[must_use]
    pub fn is_system_volume(name: &str) -> bool {
        SYSTEM_VOLUMES.contains(&name)
    }
}

impl ConfigDocument for VolumeConfig {
    fn kind(&self) -> DocKind {
        DocKind::Volume
    }

    fn id(&self) -> DocId {
        DocId::keyed(DocKind::Volume, self.name.clone())
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("VolumeConfig: name is required"));
        }
        if !Self::is_system_volume(&self.name) {
            return Err(Error::invalid(format!(
                "VolumeConfig: '{}' is not a known system volume (use UserVolumeConfig)",
                self.name
            )));
        }
        self.provisioning.validate()
    }
}

/// The `UserVolumeConfig` document (new user volume).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserVolumeConfig {
    /// The volume name; mounted at `/var/mnt/<name>`.
    pub name: String,
    /// Provisioning request.
    pub provisioning: Provisioning,
    /// Filesystem to format the partition with.
    pub filesystem: Filesystem,
}

impl UserVolumeConfig {
    /// Construct a user volume, defaulting to XFS.
    pub fn new(name: impl Into<String>, provisioning: Provisioning) -> Self {
        UserVolumeConfig {
            name: name.into(),
            provisioning,
            filesystem: Filesystem::Xfs,
        }
    }

    /// Builder: set the filesystem.
    #[must_use]
    pub fn with_filesystem(mut self, fs: Filesystem) -> Self {
        self.filesystem = fs;
        self
    }

    /// The mount path Talos derives for this volume.
    #[must_use]
    pub fn mount_path(&self) -> String {
        format!("/var/mnt/{}", self.name)
    }

    /// Validate the user-supplied volume name. Talos requires a short
    /// DNS-label-ish name: lowercase alphanumerics and dashes, 1..=34 chars.
    fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() || name.len() > 34 {
            return Err(Error::invalid(format!(
                "UserVolumeConfig: name '{name}' must be 1..=34 characters"
            )));
        }
        let ok = name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !ok {
            return Err(Error::invalid(format!(
                "UserVolumeConfig: name '{name}' may only contain [a-z0-9-]"
            )));
        }
        if name.starts_with('-') || name.ends_with('-') {
            return Err(Error::invalid(format!(
                "UserVolumeConfig: name '{name}' must not start or end with '-'"
            )));
        }
        if VolumeConfig::is_system_volume(&name.to_ascii_uppercase()) {
            return Err(Error::invalid(format!(
                "UserVolumeConfig: name '{name}' collides with a reserved system volume"
            )));
        }
        Ok(())
    }
}

impl ConfigDocument for UserVolumeConfig {
    fn kind(&self) -> DocKind {
        DocKind::UserVolume
    }

    fn id(&self) -> DocId {
        DocId::keyed(DocKind::UserVolume, self.name.clone())
    }

    fn as_user_volume(&self) -> Option<&UserVolumeConfig> {
        Some(self)
    }

    fn validate(&self) -> Result<()> {
        Self::validate_name(&self.name)?;
        self.provisioning.validate()?;
        if self.filesystem == Filesystem::None {
            return Err(Error::invalid(
                "UserVolumeConfig: a user volume must specify a filesystem",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_volume_ok() {
        let v = VolumeConfig::new("EPHEMERAL", Provisioning::growable(1 << 30, 0));
        assert!(v.validate().is_ok());
        assert_eq!(v.id(), DocId::keyed(DocKind::Volume, "EPHEMERAL"));
    }

    #[test]
    fn unknown_system_volume_rejected() {
        let v = VolumeConfig::new("DATA", Provisioning::fixed(1 << 30));
        assert!(v.validate().is_err());
    }

    #[test]
    fn provisioning_min_zero_rejected() {
        let v = VolumeConfig::new("STATE", Provisioning::fixed(0));
        assert!(v.validate().is_err());
    }

    #[test]
    fn provisioning_max_lt_min_rejected() {
        let p = Provisioning {
            min_size: 100,
            max_size: 50,
            grow: true,
            disk_selector: None,
        };
        let v = VolumeConfig::new("STATE", p);
        assert!(v.validate().is_err());
    }

    #[test]
    fn user_volume_valid() {
        let v = UserVolumeConfig::new("data", Provisioning::fixed(1 << 30));
        assert!(v.validate().is_ok());
        assert_eq!(v.mount_path(), "/var/mnt/data");
        assert!(v.allows_multiple());
    }

    #[test]
    fn user_volume_bad_name_rejected() {
        assert!(
            UserVolumeConfig::new("Data", Provisioning::fixed(1))
                .validate()
                .is_err()
        );
        assert!(
            UserVolumeConfig::new("-data", Provisioning::fixed(1))
                .validate()
                .is_err()
        );
        assert!(
            UserVolumeConfig::new("data-", Provisioning::fixed(1))
                .validate()
                .is_err()
        );
        assert!(
            UserVolumeConfig::new("", Provisioning::fixed(1))
                .validate()
                .is_err()
        );
        let long = "a".repeat(40);
        assert!(
            UserVolumeConfig::new(long, Provisioning::fixed(1))
                .validate()
                .is_err()
        );
    }

    #[test]
    fn user_volume_reserved_name_rejected() {
        assert!(
            UserVolumeConfig::new("ephemeral", Provisioning::fixed(1 << 20))
                .validate()
                .is_err()
        );
    }

    #[test]
    fn user_volume_needs_filesystem() {
        let v = UserVolumeConfig::new("data", Provisioning::fixed(1 << 30))
            .with_filesystem(Filesystem::None);
        assert!(v.validate().is_err());
    }

    #[test]
    fn filesystem_parse() {
        assert_eq!(Filesystem::parse("XFS"), Some(Filesystem::Xfs));
        assert_eq!(Filesystem::parse(""), Some(Filesystem::None));
        assert_eq!(Filesystem::parse("btrfs"), None);
    }
}
