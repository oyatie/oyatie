//! The `machine.kubelet` sub-tree plus container registry mirrors, mirroring
//! `KubeletConfig`, `RegistriesConfig`, and `RegistryMirrorConfig` in
//! `pkg/machinery/config/types/v1alpha1`.

use crate::defaults;
use crate::validation::{
    ValidationError, ValidationMode, ValidationReport, Validator, is_cidr, is_identifier,
    is_image_ref, is_ip,
};

/// A node taint that the kubelet registers itself with
/// (`machine.kubelet.nodeTaints`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeTaint {
    pub key: String,
    pub value: String,
    /// One of `NoSchedule`, `PreferNoSchedule`, `NoExecute`.
    pub effect: String,
}

impl NodeTaint {
    /// Whether the taint effect is one Kubernetes accepts.
    pub fn effect_is_valid(&self) -> bool {
        matches!(
            self.effect.as_str(),
            "NoSchedule" | "PreferNoSchedule" | "NoExecute"
        )
    }
}

/// The `machine.kubelet` sub-tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KubeletConfig {
    /// Kubelet container image.
    pub image: String,
    /// Extra command-line args (`extraArgs`).
    pub extra_args: Vec<(String, String)>,
    /// Cluster DNS server addresses (`clusterDNS`).
    pub cluster_dns: Vec<String>,
    /// Extra mounts the kubelet container gets.
    pub extra_mounts: Vec<String>,
    /// Node IP subnet hints used to pick the node's address (`nodeIP.validSubnets`).
    pub node_ip_valid_subnets: Vec<String>,
    /// Node labels.
    pub node_labels: Vec<(String, String)>,
    /// Node taints.
    pub node_taints: Vec<NodeTaint>,
    /// Whether to skip kubelet node registration checks (`registerWithFQDN`).
    pub register_with_fqdn: bool,
    /// Whether the static-pod / kubelet config validation is skipped.
    pub skip_node_registration: bool,
}

impl KubeletConfig {
    /// Look up an extra arg by key.
    pub fn extra_arg(&self, key: &str) -> Option<&str> {
        self.extra_args
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Apply defaults: image and cluster DNS.
    pub fn apply_defaults(&mut self, service_subnet: Option<&str>) {
        if self.image.is_empty() {
            self.image = defaults::DEFAULT_KUBELET_IMAGE.to_string();
        }
        if self.cluster_dns.is_empty() {
            let dns = service_subnet
                .and_then(defaults::cluster_dns_for_service_subnet)
                .unwrap_or_else(|| defaults::DEFAULT_CLUSTER_DNS.to_string());
            self.cluster_dns = vec![dns];
        }
    }
}

impl Validator for KubeletConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if !self.image.is_empty() && !is_image_ref(&self.image) {
            report.push(ValidationError::invalid(
                "machine.kubelet.image",
                format!("'{}' is not a valid image reference", self.image),
            ));
        }
        for dns in &self.cluster_dns {
            if !is_ip(dns) {
                report.push(ValidationError::invalid(
                    "machine.kubelet.clusterDNS",
                    format!("'{dns}' is not an IP"),
                ));
            }
        }
        for subnet in &self.node_ip_valid_subnets {
            // Talos allows a bare IP or a CIDR, optionally negated with `!`.
            let s = subnet.strip_prefix('!').unwrap_or(subnet);
            if !is_cidr(s) && !is_ip(s) {
                report.push(ValidationError::invalid(
                    "machine.kubelet.nodeIP.validSubnets",
                    format!("'{subnet}' is not an IP or CIDR"),
                ));
            }
        }
        for (k, _) in &self.extra_args {
            if !is_identifier(k) {
                report.push(ValidationError::invalid(
                    "machine.kubelet.extraArgs",
                    format!("'{k}' is not a valid flag name"),
                ));
            }
            // Talos forbids overriding a handful of args it manages itself.
            if matches!(
                k.as_str(),
                "hostname-override" | "config" | "bootstrap-kubeconfig" | "kubeconfig"
            ) {
                report.push(ValidationError::Conflict(format!(
                    "machine.kubelet.extraArgs may not override '{k}'"
                )));
            }
        }
        for taint in &self.node_taints {
            if taint.key.is_empty() {
                report.push(ValidationError::missing("machine.kubelet.nodeTaints[].key"));
            }
            if !taint.effect_is_valid() {
                report.push(ValidationError::invalid(
                    "machine.kubelet.nodeTaints[].effect",
                    format!("'{}' is not a valid taint effect", taint.effect),
                ));
            }
        }
    }
}

/// A registry mirror entry (`machine.registries.mirrors`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryMirror {
    /// Mirror endpoints, tried in order.
    pub endpoints: Vec<String>,
    /// Whether to skip the upstream registry fallback (`overridePath`).
    pub override_path: bool,
}

/// Per-registry TLS / auth config (`machine.registries.config`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryAuth {
    /// Basic-auth username.
    pub username: String,
    /// Basic-auth password.
    pub password: String,
    /// Whether to skip TLS verification (`tls.insecureSkipVerify`).
    pub insecure_skip_verify: bool,
}

/// The `machine.registries` sub-tree: mirrors keyed by registry host plus
/// per-host auth/TLS config.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistriesConfig {
    /// Host -> mirror config. The special host `*` is the catch-all mirror.
    pub mirrors: Vec<(String, RegistryMirror)>,
    /// Host -> auth/TLS config.
    pub config: Vec<(String, RegistryAuth)>,
}

impl RegistriesConfig {
    /// Look up the mirror config for a host, honoring the `*` catch-all.
    pub fn mirror_for(&self, host: &str) -> Option<&RegistryMirror> {
        self.mirrors
            .iter()
            .find(|(h, _)| h == host)
            .or_else(|| self.mirrors.iter().find(|(h, _)| h == "*"))
            .map(|(_, m)| m)
    }
}

impl Validator for RegistriesConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        for (host, mirror) in &self.mirrors {
            if host != "*" && !is_identifier(host) {
                report.push(ValidationError::invalid(
                    "machine.registries.mirrors",
                    format!("'{host}' is not a valid registry host"),
                ));
            }
            if mirror.endpoints.is_empty() {
                report.push(ValidationError::missing(
                    "machine.registries.mirrors[].endpoints",
                ));
            }
        }
        for (host, _auth) in &self.config {
            if !is_identifier(host) {
                report.push(ValidationError::invalid(
                    "machine.registries.config",
                    format!("'{host}' is not a valid registry host"),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kubelet_defaults_derive_cluster_dns() {
        let mut k = KubeletConfig::default();
        k.apply_defaults(Some("10.96.0.0/12"));
        assert_eq!(k.image, defaults::DEFAULT_KUBELET_IMAGE);
        assert_eq!(k.cluster_dns, vec!["10.96.0.10".to_string()]);
    }

    #[test]
    fn kubelet_rejects_managed_extra_args() {
        let mut k = KubeletConfig::default();
        k.extra_args
            .push(("hostname-override".to_string(), "x".to_string()));
        assert!(k.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn kubelet_extra_arg_lookup() {
        let mut k = KubeletConfig::default();
        k.extra_args
            .push(("max-pods".to_string(), "110".to_string()));
        assert_eq!(k.extra_arg("max-pods"), Some("110"));
        assert_eq!(k.extra_arg("nope"), None);
    }

    #[test]
    fn kubelet_node_ip_subnet_accepts_negation() {
        let mut k = KubeletConfig {
            node_ip_valid_subnets: vec!["!10.0.0.0/8".to_string(), "192.168.0.0/16".to_string()],
            ..KubeletConfig::default()
        };
        assert!(k.validate(ValidationMode::Metal).is_ok());
        k.node_ip_valid_subnets = vec!["garbage".to_string()];
        assert!(k.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn taint_effect_validation() {
        let mut k = KubeletConfig::default();
        k.node_taints.push(NodeTaint {
            key: "dedicated".to_string(),
            value: "gpu".to_string(),
            effect: "NoSuchEffect".to_string(),
        });
        assert!(k.validate(ValidationMode::Metal).is_err());
        k.node_taints[0].effect = "NoSchedule".to_string();
        assert!(k.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn registry_catch_all_mirror() {
        let mut r = RegistriesConfig::default();
        r.mirrors.push((
            "*".to_string(),
            RegistryMirror {
                endpoints: vec!["https://mirror.local".to_string()],
                override_path: false,
            },
        ));
        assert!(r.mirror_for("docker.io").is_some());
        assert!(r.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn registry_mirror_requires_endpoints() {
        let mut r = RegistriesConfig::default();
        r.mirrors
            .push(("docker.io".to_string(), RegistryMirror::default()));
        assert!(r.validate(ValidationMode::Metal).is_err());
    }
}
