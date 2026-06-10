//! Node-level Kubernetes configuration shared by the k8s controllers.
//!
//! Mirrors the bits of the Talos `ClusterConfig` / `KubernetesController`
//! inputs that the controllers fan out into kubelet config, static pods, and
//! manifests.

use crate::error::{K8sError, Result};
use std::fmt;

/// A validated Kubernetes node name (a DNS-1123 subdomain, lowercased).
///
/// Mirrors Talos `k8s.NodeName`: the kubelet registers the node under this
/// name, and the apiserver uses it for the `system:node:<name>` identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeName(String);

impl NodeName {
    /// Validate and construct a node name.
    ///
    /// Rules (RFC 1123 subdomain subset): 1-253 chars, dot-separated labels of
    /// `[a-z0-9-]` not starting/ending with `-`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into().to_ascii_lowercase();
        if s.is_empty() {
            return Err(K8sError::InvalidNodeName("empty".to_string()));
        }
        if s.len() > 253 {
            return Err(K8sError::InvalidNodeName(
                "exceeds 253 characters".to_string(),
            ));
        }
        for label in s.split('.') {
            if label.is_empty() {
                return Err(K8sError::InvalidNodeName("empty label".to_string()));
            }
            if label.starts_with('-') || label.ends_with('-') {
                return Err(K8sError::InvalidNodeName(
                    "label starts or ends with '-'".to_string(),
                ));
            }
            for c in label.chars() {
                if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                    return Err(K8sError::InvalidNodeName(format!(
                        "invalid character '{c}'"
                    )));
                }
            }
        }
        Ok(NodeName(s))
    }

    /// The node name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The RBAC subject the kubelet authenticates as.
    pub fn rbac_subject(&self) -> String {
        format!("system:node:{}", self.0)
    }
}

impl fmt::Display for NodeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The cluster's control-plane endpoint (`https://host:port`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterEndpoint {
    /// Host or IP the apiserver is reachable at.
    pub host: String,
    /// Port the apiserver listens on (defaults to 6443).
    pub port: u16,
}

impl ClusterEndpoint {
    /// Construct an endpoint, validating the host is non-empty.
    pub fn new(host: impl Into<String>, port: u16) -> Result<Self> {
        let host = host.into();
        if host.trim().is_empty() {
            return Err(K8sError::InvalidConfig(
                "empty control-plane host".to_string(),
            ));
        }
        if port == 0 {
            return Err(K8sError::InvalidConfig(
                "control-plane port is 0".to_string(),
            ));
        }
        Ok(ClusterEndpoint { host, port })
    }

    /// Render the endpoint as a URL.
    pub fn url(&self) -> String {
        format!("https://{}:{}", self.host, self.port)
    }
}

/// Node-level Kubernetes configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sConfig {
    /// The node's registration name.
    pub node_name: NodeName,
    /// The cluster (DNS) domain, e.g. `cluster.local`.
    pub cluster_domain: String,
    /// The Pod CIDR(s) used by the cluster.
    pub pod_cidrs: Vec<String>,
    /// The Service CIDR(s).
    pub service_cidrs: Vec<String>,
    /// The control-plane endpoint.
    pub endpoint: ClusterEndpoint,
    /// Kubernetes version (semantic, no leading `v`).
    pub version: String,
    /// Whether this node is a control-plane node.
    pub control_plane: bool,
}

impl K8sConfig {
    /// Validate cross-field invariants.
    pub fn validate(&self) -> Result<()> {
        if self.cluster_domain.trim().is_empty() {
            return Err(K8sError::InvalidConfig("empty cluster domain".to_string()));
        }
        if self.pod_cidrs.is_empty() {
            return Err(K8sError::InvalidConfig("no pod CIDRs".to_string()));
        }
        if self.service_cidrs.is_empty() {
            return Err(K8sError::InvalidConfig("no service CIDRs".to_string()));
        }
        if self.version.trim().is_empty() {
            return Err(K8sError::InvalidConfig(
                "empty kubernetes version".to_string(),
            ));
        }
        Ok(())
    }

    /// The well-known cluster DNS service IP: the 10th address of the first
    /// service CIDR's network base. We approximate by replacing the host part
    /// of an `a.b.c.0/n` style CIDR with `.10`.
    pub fn dns_service_ip(&self) -> Result<String> {
        let cidr = self
            .service_cidrs
            .first()
            .ok_or_else(|| K8sError::InvalidConfig("no service CIDRs".to_string()))?;
        let base = cidr.split('/').next().unwrap_or(cidr);
        let mut octets: Vec<&str> = base.split('.').collect();
        if octets.len() != 4 {
            return Err(K8sError::InvalidConfig(format!(
                "service CIDR is not IPv4: {cidr}"
            )));
        }
        octets[3] = "10";
        Ok(octets.join("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> K8sConfig {
        K8sConfig {
            node_name: NodeName::new("worker-1").unwrap(),
            cluster_domain: "cluster.local".into(),
            pod_cidrs: vec!["10.244.0.0/16".into()],
            service_cidrs: vec!["10.96.0.0/12".into()],
            endpoint: ClusterEndpoint::new("api.example.com", 6443).unwrap(),
            version: "1.30.0".into(),
            control_plane: true,
        }
    }

    #[test]
    fn node_name_validation_and_subject() {
        assert!(NodeName::new("").is_err());
        assert!(NodeName::new("-bad").is_err());
        assert!(NodeName::new("UP_PER").is_err());
        let n = NodeName::new("Node-01.example").unwrap();
        assert_eq!(n.as_str(), "node-01.example");
        assert_eq!(n.rbac_subject(), "system:node:node-01.example");
    }

    #[test]
    fn endpoint_url_and_validation() {
        assert!(ClusterEndpoint::new("", 6443).is_err());
        assert!(ClusterEndpoint::new("h", 0).is_err());
        assert_eq!(
            ClusterEndpoint::new("h", 6443).unwrap().url(),
            "https://h:6443"
        );
    }

    #[test]
    fn config_validates_and_computes_dns_ip() {
        let c = cfg();
        assert!(c.validate().is_ok());
        assert_eq!(c.dns_service_ip().unwrap(), "10.96.0.10");
    }

    #[test]
    fn config_rejects_missing_cidrs() {
        let mut c = cfg();
        c.pod_cidrs.clear();
        assert_eq!(c.validate().unwrap_err().kind(), "invalid_config");
    }
}
