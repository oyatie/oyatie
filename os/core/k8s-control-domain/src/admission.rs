//! Admission-control, audit-policy and secrets-encryption configuration.
//!
//! Mirrors the Talos controllers
//! `AdmissionControlConfigController`, `AuditPolicyConfigController` and the
//! API-server `EncryptionConfig` rendering under
//! `internal/app/machined/pkg/controllers/k8s`. Each produces a config file the
//! kube-apiserver static pod references via `--admission-control-config-file`,
//! `--audit-policy-file` and `--encryption-provider-config`.

use crate::error::{ControlError, Result};
use std::fmt::Write as _;

/// The default set of admission plugins Talos enables, in the order the
/// apiserver applies them. Mirrors `constants.DefaultEnabledAdmissionPlugins`.
pub const DEFAULT_ADMISSION_PLUGINS: &[&str] = &[
    "NodeRestriction",
    "LimitRanger",
    "ServiceAccount",
    "DefaultStorageClass",
    "MutatingAdmissionWebhook",
    "ValidatingAdmissionWebhook",
    "ResourceQuota",
    "PodSecurity",
];

/// A single admission plugin configuration block, rendered into the
/// `AdmissionConfiguration` document the apiserver loads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionPluginConfig {
    /// Plugin name (e.g. `PodSecurity`).
    pub name: String,
    /// Inline configuration body (already-serialized YAML/JSON fragment).
    pub configuration: String,
}

/// The pod-security enforcement level, mirroring the upstream PSA levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PodSecurityLevel {
    /// Unrestricted, the default for components that opt out.
    Privileged,
    /// Minimally restrictive, prevents known privilege escalations.
    Baseline,
    /// Heavily restricted, hardened best-practice policy.
    Restricted,
}

impl PodSecurityLevel {
    /// Canonical lowercase identifier used in the PSA config.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            PodSecurityLevel::Privileged => "privileged",
            PodSecurityLevel::Baseline => "baseline",
            PodSecurityLevel::Restricted => "restricted",
        }
    }
}

/// The Talos default Pod Security admission configuration: `baseline` enforced,
/// `restricted` warned/audited, with `kube-system` exempted (mirrors
/// `constants` PSA defaults).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodSecurityConfig {
    /// Level enforced on admission.
    pub enforce: PodSecurityLevel,
    /// Level surfaced as a client warning.
    pub warn: PodSecurityLevel,
    /// Level recorded in the audit log.
    pub audit: PodSecurityLevel,
    /// Namespaces exempt from enforcement.
    pub exempt_namespaces: Vec<String>,
}

impl Default for PodSecurityConfig {
    fn default() -> Self {
        PodSecurityConfig {
            enforce: PodSecurityLevel::Baseline,
            warn: PodSecurityLevel::Restricted,
            audit: PodSecurityLevel::Restricted,
            exempt_namespaces: vec!["kube-system".to_string()],
        }
    }
}

impl PodSecurityConfig {
    /// Render the inline `PodSecurity` plugin configuration body.
    #[must_use]
    pub fn render(&self) -> String {
        let exempt = self
            .exempt_namespaces
            .iter()
            .map(|n| format!("    - {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "apiVersion: pod-security.admission.config.k8s.io/v1\n\
             kind: PodSecurityConfiguration\n\
             defaults:\n\
             \x20 enforce: {enforce}\n\
             \x20 enforce-version: latest\n\
             \x20 warn: {warn}\n\
             \x20 audit: {audit}\n\
             exemptions:\n\
             \x20 usernames: []\n\
             \x20 runtimeClasses: []\n\
             \x20 namespaces:\n{exempt}\n",
            enforce = self.enforce.as_str(),
            warn = self.warn.as_str(),
            audit = self.audit.as_str(),
        )
    }
}

/// The admission control configuration controller output.
///
/// Mirrors `AdmissionControlConfigController`: it assembles the enabled plugin
/// set plus any inline plugin configuration into a single
/// `AdmissionConfiguration` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionControlConfig {
    plugins: Vec<AdmissionPluginConfig>,
}

impl AdmissionControlConfig {
    /// Build with the Talos-default `PodSecurity` plugin configured.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut cfg = AdmissionControlConfig {
            plugins: Vec::new(),
        };
        cfg.plugins.push(AdmissionPluginConfig {
            name: "PodSecurity".to_string(),
            configuration: PodSecurityConfig::default().render(),
        });
        cfg
    }

    /// An empty config (no inline plugin configuration).
    #[must_use]
    pub fn empty() -> Self {
        AdmissionControlConfig {
            plugins: Vec::new(),
        }
    }

    /// Add an inline plugin configuration block, rejecting duplicates.
    pub fn add_plugin(&mut self, plugin: AdmissionPluginConfig) -> Result<()> {
        if plugin.name.trim().is_empty() {
            return Err(ControlError::Policy(
                "admission plugin name is empty".into(),
            ));
        }
        if self.plugins.iter().any(|p| p.name == plugin.name) {
            return Err(ControlError::Policy(format!(
                "duplicate admission plugin: {}",
                plugin.name
            )));
        }
        self.plugins.push(plugin);
        Ok(())
    }

    /// Configured plugins.
    #[must_use]
    pub fn plugins(&self) -> &[AdmissionPluginConfig] {
        &self.plugins
    }

    /// Render the full `AdmissionConfiguration` document.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::from(
            "apiVersion: apiserver.config.k8s.io/v1\nkind: AdmissionConfiguration\nplugins:\n",
        );
        for p in &self.plugins {
            let _ = write!(out, "  - name: {}\n    configuration:\n", p.name);
            for line in p.configuration.lines() {
                let _ = writeln!(out, "      {line}");
            }
        }
        out
    }
}

/// Audit policy verbosity level, mirroring `audit.Level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditLevel {
    /// Don't log events matching this rule.
    None,
    /// Log request metadata only.
    Metadata,
    /// Log metadata and the request body.
    Request,
    /// Log metadata, request and response bodies.
    RequestResponse,
}

impl AuditLevel {
    /// Canonical name as it appears in the policy document.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            AuditLevel::None => "None",
            AuditLevel::Metadata => "Metadata",
            AuditLevel::Request => "Request",
            AuditLevel::RequestResponse => "RequestResponse",
        }
    }
}

/// The audit-policy configuration controller output.
///
/// Mirrors `AuditPolicyConfigController`, which renders the default `Metadata`
/// policy unless overridden via `cluster.apiServer.auditPolicy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditPolicyConfig {
    /// Default level applied to all requests.
    pub default_level: AuditLevel,
}

impl Default for AuditPolicyConfig {
    fn default() -> Self {
        AuditPolicyConfig {
            default_level: AuditLevel::Metadata,
        }
    }
}

impl AuditPolicyConfig {
    /// Render the `Policy` document for `--audit-policy-file`.
    #[must_use]
    pub fn render(&self) -> String {
        format!(
            "apiVersion: audit.k8s.io/v1\nkind: Policy\nrules:\n  - level: {}\n",
            self.default_level.as_str()
        )
    }
}

/// Supported secret-encryption providers for `EncryptionConfiguration`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionProvider {
    /// AES-CBC with PKCS#7 padding.
    AesCbc,
    /// AES-GCM (recommended).
    AesGcm,
    /// `XSalsa20` + Poly1305 (secretbox).
    SecretBox,
}

impl EncryptionProvider {
    /// The provider key used in the encryption config document.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            EncryptionProvider::AesCbc => "aescbc",
            EncryptionProvider::AesGcm => "aesgcm",
            EncryptionProvider::SecretBox => "secretbox",
        }
    }

    /// Required key length in bytes for this provider's secret.
    #[must_use]
    pub fn key_len(self) -> usize {
        // AES-CBC requires 32 bytes; AES-GCM accepts 16/24/32 (Talos uses 32);
        // secretbox uses a 32-byte key. All three settle on 32.
        match self {
            EncryptionProvider::AesCbc
            | EncryptionProvider::AesGcm
            | EncryptionProvider::SecretBox => 32,
        }
    }
}

/// Encryption-at-rest configuration for etcd `secrets` (and optionally other
/// resources). Mirrors the apiserver `EncryptionConfiguration` Talos writes when
/// `cluster.apiServer.resources` / secret encryption is enabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionConfig {
    /// Resources to encrypt (e.g. `secrets`).
    pub resources: Vec<String>,
    /// Active provider used to encrypt new writes.
    pub provider: EncryptionProvider,
    /// Base64-ish secret key material (opaque bytes here).
    pub key: Vec<u8>,
}

impl EncryptionConfig {
    /// Build and validate an encryption config.
    pub fn new(
        provider: EncryptionProvider,
        key: impl Into<Vec<u8>>,
        resources: Vec<String>,
    ) -> Result<Self> {
        let key = key.into();
        if key.len() != provider.key_len() {
            return Err(ControlError::Policy(format!(
                "{} key must be {} bytes, got {}",
                provider.as_str(),
                provider.key_len(),
                key.len()
            )));
        }
        if resources.is_empty() {
            return Err(ControlError::Policy(
                "encryption config must list at least one resource".into(),
            ));
        }
        Ok(EncryptionConfig {
            resources,
            provider,
            key,
        })
    }

    /// A default config encrypting `secrets` with AES-GCM.
    pub fn default_secrets(key: impl Into<Vec<u8>>) -> Result<Self> {
        Self::new(EncryptionProvider::AesGcm, key, vec!["secrets".to_string()])
    }

    /// Render the `EncryptionConfiguration` document. The key is emitted as a
    /// simple hex digest so the output is deterministic and dependency-free; a
    /// real implementation base64-encodes the raw key.
    #[must_use]
    pub fn render(&self) -> String {
        let key_hex = self
            .key
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let resources = self
            .resources
            .iter()
            .map(|r| format!("        - {r}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "apiVersion: apiserver.config.k8s.io/v1\n\
             kind: EncryptionConfiguration\n\
             resources:\n\
             \x20 - resources:\n{resources}\n\
             \x20\x20\x20 providers:\n\
             \x20\x20\x20\x20\x20 - {provider}:\n\
             \x20\x20\x20\x20\x20\x20\x20\x20 keys:\n\
             \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 - name: key1\n\
             \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 secret: {key_hex}\n\
             \x20\x20\x20\x20\x20 - identity: {{}}\n",
            provider = self.provider.as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_admission_includes_pod_security() {
        let cfg = AdmissionControlConfig::with_defaults();
        assert_eq!(cfg.plugins().len(), 1);
        let doc = cfg.render();
        assert!(doc.contains("AdmissionConfiguration"));
        assert!(doc.contains("name: PodSecurity"));
        assert!(doc.contains("enforce: baseline"));
        assert!(doc.contains("- kube-system"));
    }

    #[test]
    fn admission_rejects_duplicate_and_empty_plugins() {
        let mut cfg = AdmissionControlConfig::empty();
        cfg.add_plugin(AdmissionPluginConfig {
            name: "EventRateLimit".into(),
            configuration: "x".into(),
        })
        .unwrap();
        let dup = cfg.add_plugin(AdmissionPluginConfig {
            name: "EventRateLimit".into(),
            configuration: "y".into(),
        });
        assert_eq!(dup.unwrap_err().kind(), "policy");
        let empty = cfg.add_plugin(AdmissionPluginConfig {
            name: "  ".into(),
            configuration: "y".into(),
        });
        assert!(empty.is_err());
    }

    #[test]
    fn default_plugins_have_pod_security_last() {
        assert_eq!(*DEFAULT_ADMISSION_PLUGINS.last().unwrap(), "PodSecurity");
        assert!(DEFAULT_ADMISSION_PLUGINS.contains(&"NodeRestriction"));
    }

    #[test]
    fn audit_policy_defaults_to_metadata() {
        let cfg = AuditPolicyConfig::default();
        assert_eq!(cfg.default_level, AuditLevel::Metadata);
        let doc = cfg.render();
        assert!(doc.contains("kind: Policy"));
        assert!(doc.contains("level: Metadata"));
    }

    #[test]
    fn encryption_validates_key_length() {
        let short = EncryptionConfig::new(
            EncryptionProvider::AesGcm,
            vec![0u8; 8],
            vec!["secrets".into()],
        );
        assert_eq!(short.unwrap_err().kind(), "policy");
        let ok = EncryptionConfig::default_secrets(vec![7u8; 32]).unwrap();
        assert_eq!(ok.provider, EncryptionProvider::AesGcm);
        let doc = ok.render();
        assert!(doc.contains("EncryptionConfiguration"));
        assert!(doc.contains("aesgcm"));
        assert!(doc.contains("- identity: {}"));
    }

    #[test]
    fn encryption_requires_resources() {
        let err = EncryptionConfig::new(EncryptionProvider::AesCbc, vec![0u8; 32], vec![]);
        assert!(err.is_err());
    }
}
