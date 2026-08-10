//! Rendering of kubeconfig files (admin, controller-manager, scheduler, kubelet).
//!
//! Mirrors Talos `pkg/kubernetes` kubeconfig generation and kubeadm's
//! `BuildKubeConfigFromSpec`: a kubeconfig binds a cluster (server URL + CA),
//! an auth-info (client cert / token), and a context tying them together. The
//! admin kubeconfig is what `talosctl kubeconfig` hands the operator; the
//! component kubeconfigs are mounted into the static pods.
//!
//! We render a deterministic, dependency-free YAML approximation rather than
//! depending on a serializer. The structure (clusters/users/contexts) matches
//! the real `kubeconfig` v1 schema.

use crate::config::K8sConfig;
use crate::error::{K8sError, Result};

/// How a kubeconfig user authenticates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthInfo {
    /// Client certificate + key data, already encoded for kubeconfig `*-data` fields.
    ClientCertificate {
        /// The client certificate data value.
        client_certificate_data: String,
        /// The client key data value.
        client_key_data: String,
    },
    /// A bearer token (service-account style).
    Token(String),
}

/// A single kubeconfig document.
///
/// `*-data` fields are stored as the exact kubeconfig YAML values. Callers that
/// start from opaque certificate/key bytes must encode them first (for example
/// with [`crate::encoding::kubeconfig_data`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeConfig {
    /// The cluster name (e.g. the cluster id).
    pub cluster_name: String,
    /// The apiserver URL.
    pub server: String,
    /// The cluster CA certificate data value.
    pub certificate_authority_data: String,
    /// The user/context name.
    pub user_name: String,
    /// How the user authenticates.
    pub auth: AuthInfo,
}

impl KubeConfig {
    /// The context name binding cluster + user, kubeadm-style: `user@cluster`.
    pub fn context_name(&self) -> String {
        format!("{}@{}", self.user_name, self.cluster_name)
    }

    /// Build the admin kubeconfig (`O=system:masters`) for the cluster.
    ///
    /// `ca_data`, `client_cert_data`, and `client_key_data` must already be
    /// standard-base64 kubeconfig `*-data` values.
    pub fn admin(
        cfg: &K8sConfig,
        cluster_name: impl Into<String>,
        ca_data: impl Into<String>,
        client_cert_data: impl Into<String>,
        client_key_data: impl Into<String>,
    ) -> Result<Self> {
        cfg.validate()?;
        let cluster_name = cluster_name.into();
        if cluster_name.trim().is_empty() {
            return Err(K8sError::InvalidConfig("empty cluster name".to_string()));
        }
        Ok(KubeConfig {
            cluster_name,
            server: cfg.endpoint.url(),
            certificate_authority_data: ca_data.into(),
            user_name: "admin".to_string(),
            auth: AuthInfo::ClientCertificate {
                client_certificate_data: client_cert_data.into(),
                client_key_data: client_key_data.into(),
            },
        })
    }

    /// Build a control-plane component kubeconfig (controller-manager /
    /// scheduler), which talks to the apiserver over loopback.
    ///
    /// `ca_data`, `client_cert_data`, and `client_key_data` must already be
    /// standard-base64 kubeconfig `*-data` values.
    pub fn component(
        component_user: impl Into<String>,
        cluster_name: impl Into<String>,
        ca_data: impl Into<String>,
        client_cert_data: impl Into<String>,
        client_key_data: impl Into<String>,
    ) -> Result<Self> {
        let user_name = component_user.into();
        if user_name.trim().is_empty() {
            return Err(K8sError::InvalidConfig("empty component user".to_string()));
        }
        Ok(KubeConfig {
            cluster_name: cluster_name.into(),
            // Components reach the apiserver locally on the control-plane node.
            server: "https://127.0.0.1:6443".to_string(),
            certificate_authority_data: ca_data.into(),
            user_name,
            auth: AuthInfo::ClientCertificate {
                client_certificate_data: client_cert_data.into(),
                client_key_data: client_key_data.into(),
            },
        })
    }

    /// Render the kubeconfig as deterministic YAML (v1 Config schema).
    pub fn render(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        out.push_str("apiVersion: v1\n");
        out.push_str("kind: Config\n");
        out.push_str("clusters:\n");
        let _ = writeln!(out, "- name: {}", self.cluster_name);
        out.push_str("  cluster:\n");
        let _ = writeln!(out, "    server: {}", self.server);
        let _ = writeln!(
            out,
            "    certificate-authority-data: {}",
            self.certificate_authority_data
        );
        out.push_str("users:\n");
        let _ = writeln!(out, "- name: {}", self.user_name);
        out.push_str("  user:\n");
        match &self.auth {
            AuthInfo::ClientCertificate {
                client_certificate_data,
                client_key_data,
            } => {
                let _ = writeln!(
                    out,
                    "    client-certificate-data: {client_certificate_data}"
                );
                let _ = writeln!(out, "    client-key-data: {client_key_data}");
            }
            AuthInfo::Token(t) => {
                let _ = writeln!(out, "    token: {t}");
            }
        }
        out.push_str("contexts:\n");
        let _ = writeln!(out, "- name: {}", self.context_name());
        out.push_str("  context:\n");
        let _ = writeln!(out, "    cluster: {}", self.cluster_name);
        let _ = writeln!(out, "    user: {}", self.user_name);
        let _ = writeln!(out, "current-context: {}", self.context_name());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ClusterEndpoint, NodeName};

    fn cfg() -> K8sConfig {
        K8sConfig {
            node_name: NodeName::new("cp-1").unwrap(),
            cluster_domain: "cluster.local".into(),
            pod_cidrs: vec!["10.244.0.0/16".into()],
            service_cidrs: vec!["10.96.0.0/12".into()],
            endpoint: ClusterEndpoint::new("api.example.com", 6443).unwrap(),
            version: "1.30.0".into(),
            control_plane: true,
        }
    }

    #[test]
    fn admin_kubeconfig_points_at_endpoint() {
        let kc = KubeConfig::admin(&cfg(), "talos-cluster", "CA", "CERT", "KEY").unwrap();
        assert_eq!(kc.server, "https://api.example.com:6443");
        assert_eq!(kc.user_name, "admin");
        assert_eq!(kc.context_name(), "admin@talos-cluster");
    }

    #[test]
    fn admin_rejects_empty_cluster_name() {
        assert!(KubeConfig::admin(&cfg(), "  ", "CA", "C", "K").is_err());
    }

    #[test]
    fn component_uses_loopback_server() {
        let kc = KubeConfig::component("system:kube-controller-manager", "talos", "CA", "C", "K")
            .unwrap();
        assert_eq!(kc.server, "https://127.0.0.1:6443");
    }

    #[test]
    fn render_contains_all_sections() {
        let kc = KubeConfig::admin(&cfg(), "talos", "CADATA", "CERTDATA", "KEYDATA").unwrap();
        let y = kc.render();
        assert!(y.contains("kind: Config"));
        assert!(y.contains("server: https://api.example.com:6443"));
        assert!(y.contains("certificate-authority-data: CADATA"));
        assert!(y.contains("client-certificate-data: CERTDATA"));
        assert!(y.contains("client-key-data: KEYDATA"));
        assert!(y.contains("current-context: admin@talos"));
    }

    #[test]
    fn token_auth_renders_token() {
        let kc = KubeConfig {
            cluster_name: "c".into(),
            server: "https://127.0.0.1:6443".into(),
            certificate_authority_data: "CA".into(),
            user_name: "kubelet".into(),
            auth: AuthInfo::Token("abc.def".into()),
        };
        let y = kc.render();
        assert!(y.contains("token: abc.def"));
        assert!(!y.contains("client-certificate-data"));
    }

    #[test]
    fn render_is_deterministic() {
        let kc = KubeConfig::admin(&cfg(), "talos", "CA", "C", "K").unwrap();
        assert_eq!(kc.render(), kc.render());
    }
}
