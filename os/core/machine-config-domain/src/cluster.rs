//! The `cluster:` sub-tree of the v1alpha1 config: the control-plane endpoint,
//! cluster identity, network CIDRs, and the cluster-wide secrets reference.
//!
//! Mirrors the Talos `ClusterConfig` type in
//! `pkg/machinery/config/types/v1alpha1`.

use crate::secrets::Secrets;
use crate::validation::{ValidationError, ValidationMode, ValidationReport, Validator};
use os_kernel::error::{Error, Result};

/// The Kubernetes API control-plane endpoint (`cluster.controlPlane.endpoint`).
///
/// Stored as scheme/host/port; Talos requires an `https` URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPlaneEndpoint {
    /// URL scheme; must be `https`.
    pub scheme: String,
    /// Host (DNS name or IP literal).
    pub host: String,
    /// TCP port (defaults to 6443).
    pub port: u16,
}

impl Default for ControlPlaneEndpoint {
    fn default() -> Self {
        ControlPlaneEndpoint {
            scheme: "https".to_string(),
            host: String::new(),
            port: 6443,
        }
    }
}

impl ControlPlaneEndpoint {
    /// Parse from a URL string like `https://10.0.0.1:6443`.
    pub fn parse(url: &str) -> Result<Self> {
        let (scheme, rest) = url.split_once("://").ok_or_else(|| {
            Error::parse("endpoint must include a scheme, e.g. https://host:port")
        })?;
        if scheme != "https" {
            return Err(Error::invalid(format!(
                "control plane endpoint scheme must be https, got '{scheme}'"
            )));
        }
        // Strip any trailing path.
        let authority = rest.split('/').next().unwrap_or(rest);
        if authority.is_empty() {
            return Err(Error::parse("endpoint host is empty"));
        }
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) => {
                let port: u16 = p
                    .parse()
                    .map_err(|_| Error::parse(format!("invalid port '{p}'")))?;
                (h.to_string(), port)
            }
            None => (authority.to_string(), 6443u16),
        };
        if host.is_empty() {
            return Err(Error::parse("endpoint host is empty"));
        }
        Ok(ControlPlaneEndpoint {
            scheme: scheme.to_string(),
            host,
            port,
        })
    }

    /// Render back to URL form.
    pub fn to_url(&self) -> String {
        format!("{}://{}:{}", self.scheme, self.host, self.port)
    }
}

/// The `cluster:` sub-tree.
///
/// Mirrors `ClusterConfig`: cluster id/name, the control-plane endpoint, the pod
/// and service CIDRs, the DNS domain, and the cluster secret bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterConfig {
    /// Cluster name (`cluster.clusterName`).
    pub name: String,
    /// Cluster id (`cluster.id`).
    pub id: String,
    /// Control-plane endpoint.
    pub endpoint: ControlPlaneEndpoint,
    /// Pod subnet CIDRs (`cluster.network.podSubnets`).
    pub pod_subnets: Vec<String>,
    /// Service subnet CIDRs (`cluster.network.serviceSubnets`).
    pub service_subnets: Vec<String>,
    /// Cluster DNS domain (`cluster.network.dnsDomain`).
    pub dns_domain: String,
    /// Shared cluster secrets.
    pub secrets: Secrets,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        ClusterConfig {
            name: String::new(),
            id: String::new(),
            endpoint: ControlPlaneEndpoint::default(),
            pod_subnets: vec!["10.244.0.0/16".to_string()],
            service_subnets: vec!["10.96.0.0/12".to_string()],
            dns_domain: "cluster.local".to_string(),
            secrets: Secrets::default(),
        }
    }
}

impl ClusterConfig {
    /// A new named cluster reachable at `endpoint`.
    pub fn new(name: impl Into<String>, endpoint: ControlPlaneEndpoint) -> Self {
        ClusterConfig {
            name: name.into(),
            endpoint,
            ..Default::default()
        }
    }

    /// Whether the endpoint host is set.
    pub fn has_endpoint(&self) -> bool {
        !self.endpoint.host.is_empty()
    }
}

impl Validator for ClusterConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if self.name.is_empty() {
            report.push(ValidationError::missing("cluster.clusterName"));
        }
        if !self.has_endpoint() {
            report.push(ValidationError::missing("cluster.controlPlane.endpoint"));
        }
        if self.endpoint.scheme != "https" {
            report.push(ValidationError::invalid(
                "cluster.controlPlane.endpoint",
                "scheme must be https",
            ));
        }
        if self.pod_subnets.is_empty() {
            report.push(ValidationError::missing("cluster.network.podSubnets"));
        }
        if self.service_subnets.is_empty() {
            report.push(ValidationError::missing("cluster.network.serviceSubnets"));
        }
        if self.dns_domain.is_empty() {
            report.push(ValidationError::Warning(
                "cluster.network.dnsDomain empty; defaulting to cluster.local".to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_parse_with_explicit_port() {
        let e = ControlPlaneEndpoint::parse("https://10.0.0.1:6443").unwrap();
        assert_eq!(e.host, "10.0.0.1");
        assert_eq!(e.port, 6443);
        assert_eq!(e.to_url(), "https://10.0.0.1:6443");
    }

    #[test]
    fn endpoint_parse_defaults_port() {
        let e = ControlPlaneEndpoint::parse("https://api.example.com").unwrap();
        assert_eq!(e.host, "api.example.com");
        assert_eq!(e.port, 6443);
    }

    #[test]
    fn endpoint_rejects_http_and_missing_scheme() {
        assert!(ControlPlaneEndpoint::parse("http://host:6443").is_err());
        assert!(ControlPlaneEndpoint::parse("host:6443").is_err());
        assert!(ControlPlaneEndpoint::parse("https://host:notaport").is_err());
    }

    #[test]
    fn cluster_validation_flags_missing_fields() {
        let c = ClusterConfig::default();
        let err = c.validate(ValidationMode::Metal).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn valid_cluster_passes() {
        let c = ClusterConfig::new(
            "prod",
            ControlPlaneEndpoint::parse("https://10.0.0.1:6443").unwrap(),
        );
        let warnings = c.validate(ValidationMode::Metal).unwrap();
        assert!(warnings.is_empty());
    }

    #[test]
    fn defaults_have_standard_cidrs() {
        let c = ClusterConfig::default();
        assert_eq!(c.dns_domain, "cluster.local");
        assert!(c.pod_subnets.contains(&"10.244.0.0/16".to_string()));
    }
}
