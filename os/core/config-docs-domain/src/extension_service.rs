//! `ExtensionServiceConfig` — per extension-service configuration.
//!
//! Mirrors `pkg/machinery/config/types/runtime/extensions`. Each document is
//! keyed by the extension service `name:` and supplies environment variables,
//! inline configuration files (written into the service's rootfs), and extra
//! bind mounts.

use crate::document::{ConfigDocument, DocId, DocKind};
use os_kernel::error::{Error, Result};

/// An inline configuration file injected into the extension service rootfs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionServiceConfigFile {
    /// File contents.
    pub content: String,
    /// Absolute mount path inside the service container.
    pub mount_path: String,
}

impl ExtensionServiceConfigFile {
    /// Construct a config file entry.
    pub fn new(content: impl Into<String>, mount_path: impl Into<String>) -> Self {
        ExtensionServiceConfigFile {
            content: content.into(),
            mount_path: mount_path.into(),
        }
    }

    fn validate(&self) -> Result<()> {
        if !self.mount_path.starts_with('/') {
            return Err(Error::invalid(format!(
                "ExtensionServiceConfig: config file mountPath '{}' must be absolute",
                self.mount_path
            )));
        }
        Ok(())
    }
}

/// An extra bind mount for an extension service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    /// Source path on the host.
    pub source: String,
    /// Destination path inside the service.
    pub destination: String,
    /// Whether the mount is read-only.
    pub read_only: bool,
}

impl Mount {
    /// Construct a mount.
    pub fn new(source: impl Into<String>, destination: impl Into<String>, read_only: bool) -> Self {
        Mount {
            source: source.into(),
            destination: destination.into(),
            read_only,
        }
    }

    fn validate(&self) -> Result<()> {
        if !self.source.starts_with('/') {
            return Err(Error::invalid(format!(
                "ExtensionServiceConfig: mount source '{}' must be absolute",
                self.source
            )));
        }
        if !self.destination.starts_with('/') {
            return Err(Error::invalid(format!(
                "ExtensionServiceConfig: mount destination '{}' must be absolute",
                self.destination
            )));
        }
        Ok(())
    }
}

/// The `ExtensionServiceConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionServiceConfig {
    /// The extension service name this config applies to. Acts as the document
    /// key for multi-doc validation.
    pub name: String,
    /// Environment variables in `KEY=VALUE` form.
    pub environment: Vec<String>,
    /// Inline configuration files.
    pub config_files: Vec<ExtensionServiceConfigFile>,
    /// Extra bind mounts.
    pub mounts: Vec<Mount>,
}

impl ExtensionServiceConfig {
    /// Construct an empty config for a named extension service.
    pub fn new(name: impl Into<String>) -> Self {
        ExtensionServiceConfig {
            name: name.into(),
            environment: Vec::new(),
            config_files: Vec::new(),
            mounts: Vec::new(),
        }
    }

    /// Builder: add an environment variable.
    pub fn with_env(mut self, kv: impl Into<String>) -> Self {
        self.environment.push(kv.into());
        self
    }

    /// Builder: add a config file.
    #[must_use]
    pub fn with_config_file(mut self, file: ExtensionServiceConfigFile) -> Self {
        self.config_files.push(file);
        self
    }

    /// Builder: add a mount.
    #[must_use]
    pub fn with_mount(mut self, mount: Mount) -> Self {
        self.mounts.push(mount);
        self
    }
}

impl ConfigDocument for ExtensionServiceConfig {
    fn kind(&self) -> DocKind {
        DocKind::ExtensionService
    }

    fn id(&self) -> DocId {
        DocId::keyed(DocKind::ExtensionService, self.name.clone())
    }

    fn as_extension_service(&self) -> Option<&ExtensionServiceConfig> {
        Some(self)
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("ExtensionServiceConfig: name is required"));
        }
        for env in &self.environment {
            if !env.contains('=') || env.starts_with('=') {
                return Err(Error::invalid(format!(
                    "ExtensionServiceConfig: environment entry '{env}' must be KEY=VALUE"
                )));
            }
        }
        for file in &self.config_files {
            file.validate()?;
        }
        // Reject duplicate config-file mount paths.
        for (i, a) in self.config_files.iter().enumerate() {
            for b in &self.config_files[i + 1..] {
                if a.mount_path == b.mount_path {
                    return Err(Error::invalid(format!(
                        "ExtensionServiceConfig: duplicate config file mountPath '{}'",
                        a.mount_path
                    )));
                }
            }
        }
        for mount in &self.mounts {
            mount.validate()?;
        }
        // Reject duplicate mount destinations within a single service doc.
        for (i, a) in self.mounts.iter().enumerate() {
            for b in &self.mounts[i + 1..] {
                if a.destination == b.destination {
                    return Err(Error::invalid(format!(
                        "ExtensionServiceConfig: duplicate mount destination '{}'",
                        a.destination
                    )));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_full_config() {
        let c = ExtensionServiceConfig::new("nut-client")
            .with_env("UPS=ups@host")
            .with_config_file(ExtensionServiceConfigFile::new(
                "body",
                "/etc/nut/upsmon.conf",
            ))
            .with_mount(Mount::new("/var/run", "/run", false));
        assert!(c.validate().is_ok());
        assert_eq!(
            c.id(),
            DocId::keyed(DocKind::ExtensionService, "nut-client")
        );
        assert!(c.allows_multiple());
    }

    #[test]
    fn empty_name_rejected() {
        assert!(ExtensionServiceConfig::new("  ").validate().is_err());
    }

    #[test]
    fn bad_env_rejected() {
        let c = ExtensionServiceConfig::new("svc").with_env("NOEQUALS");
        assert!(c.validate().is_err());
        let c = ExtensionServiceConfig::new("svc").with_env("=value");
        assert!(c.validate().is_err());
    }

    #[test]
    fn relative_config_path_rejected() {
        let c = ExtensionServiceConfig::new("svc")
            .with_config_file(ExtensionServiceConfigFile::new("x", "etc/rel.conf"));
        assert!(c.validate().is_err());
    }

    #[test]
    fn duplicate_config_path_rejected() {
        let c = ExtensionServiceConfig::new("svc")
            .with_config_file(ExtensionServiceConfigFile::new("a", "/etc/x"))
            .with_config_file(ExtensionServiceConfigFile::new("b", "/etc/x"));
        assert!(c.validate().is_err());
    }

    #[test]
    fn duplicate_mount_destination_rejected() {
        let c = ExtensionServiceConfig::new("svc")
            .with_mount(Mount::new("/a", "/dst", false))
            .with_mount(Mount::new("/b", "/dst", true));
        assert!(c.validate().is_err());
    }

    #[test]
    fn relative_mount_rejected() {
        let c = ExtensionServiceConfig::new("svc").with_mount(Mount::new("rel", "/dst", false));
        assert!(c.validate().is_err());
        let c = ExtensionServiceConfig::new("svc").with_mount(Mount::new("/src", "rel", false));
        assert!(c.validate().is_err());
    }
}
