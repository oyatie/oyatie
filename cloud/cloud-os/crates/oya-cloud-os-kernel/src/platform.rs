//! Supported install platforms, mirroring Talos `pkg/machinery` platform names.

use crate::error::{Error, Result};
use core::fmt;

/// A platform Talos can run on. Determines how config and network metadata are
/// discovered at boot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    /// Bare-metal install.
    Metal,
    /// Amazon Web Services.
    Aws,
    /// Google Cloud Platform.
    Gcp,
    /// Microsoft Azure.
    Azure,
    /// QEMU/KVM virtual machine.
    Qemu,
    /// `VMware` vSphere.
    VMware,
    /// A container (Docker/sidero) runtime.
    Container,
    /// Unknown / unrecognized platform.
    Unknown,
}

impl Platform {
    /// Canonical lowercase platform name used by Talos.
    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Metal => "metal",
            Platform::Aws => "aws",
            Platform::Gcp => "gcp",
            Platform::Azure => "azure",
            Platform::Qemu => "qemu",
            Platform::VMware => "vmware",
            Platform::Container => "container",
            Platform::Unknown => "unknown",
        }
    }

    /// Parse a platform from its canonical name.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "metal" | "bare-metal" => Ok(Platform::Metal),
            "aws" => Ok(Platform::Aws),
            "gcp" | "gce" => Ok(Platform::Gcp),
            "azure" => Ok(Platform::Azure),
            "qemu" | "kvm" => Ok(Platform::Qemu),
            "vmware" => Ok(Platform::VMware),
            "container" | "docker" => Ok(Platform::Container),
            "unknown" | "" => Ok(Platform::Unknown),
            other => Err(Error::parse(alloc::format!("unknown platform '{other}'"))),
        }
    }

    /// Whether this platform is a cloud provider (has an instance metadata
    /// service used for config discovery).
    pub fn is_cloud(self) -> bool {
        matches!(self, Platform::Aws | Platform::Gcp | Platform::Azure)
    }

    /// Whether config is normally delivered via a virtual block device / ISO
    /// rather than a metadata endpoint.
    pub fn uses_local_config(self) -> bool {
        matches!(
            self,
            Platform::Metal | Platform::Qemu | Platform::VMware | Platform::Container
        )
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::str::FromStr for Platform {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Platform::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_and_aliases() {
        assert_eq!(Platform::parse("aws").unwrap(), Platform::Aws);
        assert_eq!(Platform::parse("GCE").unwrap(), Platform::Gcp);
        assert_eq!(Platform::parse("bare-metal").unwrap(), Platform::Metal);
        assert_eq!(Platform::parse("docker").unwrap(), Platform::Container);
        assert!(Platform::parse("heroku").is_err());
    }

    #[test]
    fn cloud_classification() {
        assert!(Platform::Aws.is_cloud());
        assert!(Platform::Azure.is_cloud());
        assert!(!Platform::Metal.is_cloud());
        assert!(!Platform::Qemu.is_cloud());
    }

    #[test]
    fn local_config_classification() {
        assert!(Platform::Metal.uses_local_config());
        assert!(!Platform::Aws.uses_local_config());
        assert_eq!(Platform::VMware.to_string(), "vmware");
    }
}
