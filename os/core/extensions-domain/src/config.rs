//! `ExtensionServiceConfig` machine-configuration document.
//!
//! Talos lets a user configure an installed extension service from the machine
//! config via the `ExtensionServiceConfig` document (kind
//! `ExtensionServiceConfig`, apiVersion `v1alpha1`). It carries environment
//! variables and the contents of config files to drop into the service's
//! working directory. This module models that document and its validation,
//! mirroring `pkg/machinery/config/types/runtime/extension_service_config.go`.

use os_kernel::error::{Error, Result};

/// A file the user wants materialized for an extension service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFile {
    /// Absolute mount path inside the service rootfs.
    pub mount_path: String,
    /// Literal file content.
    pub content: String,
    /// File mode (octal, e.g. `0o600`). 0 means "use default".
    pub permissions: u32,
}

impl ConfigFile {
    /// Construct a config file with default permissions.
    pub fn new(mount_path: impl Into<String>, content: impl Into<String>) -> Self {
        ConfigFile {
            mount_path: mount_path.into(),
            content: content.into(),
            permissions: 0o644,
        }
    }

    /// Validate the mount path is absolute.
    pub fn validate(&self) -> Result<()> {
        if !self.mount_path.starts_with('/') {
            return Err(Error::invalid(format!(
                "config file mount path '{}' must be absolute",
                self.mount_path
            )));
        }
        if self.permissions > 0o777 {
            return Err(Error::invalid("config file permissions out of range"));
        }
        Ok(())
    }
}

/// A single `KEY=VALUE` environment entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVar {
    /// Variable name.
    pub key: String,
    /// Variable value.
    pub value: String,
}

impl EnvVar {
    /// Parse a `KEY=VALUE` string.
    pub fn parse(s: &str) -> Result<Self> {
        let (k, v) = s
            .split_once('=')
            .ok_or_else(|| Error::parse(format!("env var '{s}' is not KEY=VALUE")))?;
        let k = k.trim();
        if k.is_empty() {
            return Err(Error::parse("env var has empty key"));
        }
        if !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(Error::parse(format!(
                "env var key '{k}' has invalid characters"
            )));
        }
        Ok(EnvVar {
            key: k.to_string(),
            value: v.to_string(),
        })
    }
}

/// The `ExtensionServiceConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionServiceConfig {
    /// The name of the extension service this configures (the document `name`).
    pub name: String,
    /// Environment overrides for the service.
    pub environment: Vec<EnvVar>,
    /// Config files to materialize.
    pub config_files: Vec<ConfigFile>,
}

impl ExtensionServiceConfig {
    /// The document kind, matching Talos.
    pub const KIND: &'static str = "ExtensionServiceConfig";
    /// The document apiVersion.
    pub const API_VERSION: &'static str = "v1alpha1";

    /// A new empty config for service `name`.
    pub fn new(name: impl Into<String>) -> Self {
        ExtensionServiceConfig {
            name: name.into(),
            environment: Vec::new(),
            config_files: Vec::new(),
        }
    }

    /// Builder: add an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.push(EnvVar {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Builder: add a config file.
    pub fn with_file(mut self, file: ConfigFile) -> Self {
        self.config_files.push(file);
        self
    }

    /// Render the environment as a sorted `KEY=VALUE` list (the form passed to
    /// the service process), deduplicating by key with last-wins semantics.
    pub fn rendered_env(&self) -> Vec<String> {
        let mut map: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
        for e in &self.environment {
            map.insert(e.key.as_str(), e.value.as_str());
        }
        map.into_iter().map(|(k, v)| format!("{k}={v}")).collect()
    }

    /// Validate the document: a non-empty name, valid env keys, valid files, and
    /// no two config files mounting at the same path.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::invalid(
                "ExtensionServiceConfig name must not be empty",
            ));
        }
        for e in &self.environment {
            if e.key.is_empty() || !e.key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Err(Error::invalid(format!("invalid env var key '{}'", e.key)));
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for f in &self.config_files {
            f.validate()?;
            if !seen.insert(f.mount_path.as_str()) {
                return Err(Error::invalid(format!(
                    "duplicate config file mount path '{}'",
                    f.mount_path
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_var_parsing() {
        let e = EnvVar::parse("FOO=bar").unwrap();
        assert_eq!(e.key, "FOO");
        assert_eq!(e.value, "bar");
        assert_eq!(EnvVar::parse("HAS=eq=signs").unwrap().value, "eq=signs");
        assert!(EnvVar::parse("noequals").is_err());
        assert!(EnvVar::parse("=novalue").is_err());
        assert!(EnvVar::parse("bad-key=v").is_err());
    }

    #[test]
    fn rendered_env_sorts_and_dedups() {
        let cfg = ExtensionServiceConfig::new("svc")
            .with_env("B", "2")
            .with_env("A", "1")
            .with_env("A", "override");
        assert_eq!(
            cfg.rendered_env(),
            vec!["A=override".to_string(), "B=2".to_string()]
        );
    }

    #[test]
    fn validate_rejects_bad_inputs() {
        let mut cfg = ExtensionServiceConfig::new("svc");
        assert!(cfg.validate().is_ok());

        cfg.name = String::new();
        assert!(cfg.validate().is_err());

        let cfg = ExtensionServiceConfig::new("svc")
            .with_file(ConfigFile::new("/etc/a.conf", "x"))
            .with_file(ConfigFile::new("/etc/a.conf", "y"));
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn config_file_validation() {
        assert!(ConfigFile::new("/etc/x", "c").validate().is_ok());
        assert!(ConfigFile::new("relative", "c").validate().is_err());
        let mut f = ConfigFile::new("/etc/x", "c");
        f.permissions = 0o7777;
        assert!(f.validate().is_err());
    }

    #[test]
    fn document_constants() {
        assert_eq!(ExtensionServiceConfig::KIND, "ExtensionServiceConfig");
        assert_eq!(ExtensionServiceConfig::API_VERSION, "v1alpha1");
    }
}
