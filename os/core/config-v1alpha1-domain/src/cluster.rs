//! The `cluster:` sub-tree aggregator: cluster identity, control-plane endpoint,
//! network CIDRs, etcd, the static control-plane components, discovery, and
//! logging. Mirrors `ClusterConfig` in
//! `pkg/machinery/config/types/v1alpha1`.

use crate::defaults;
use crate::etcd::{
    ApiServerConfig, ControllerManagerConfig, EtcdConfig, ProxyConfig, SchedulerConfig,
};
use crate::validation::{
    ValidationError, ValidationMode, ValidationReport, Validator, is_cidr, is_hostname, is_ip,
};
use os_kernel::error::{Error, Result};

/// The Kubernetes API control-plane endpoint (`cluster.controlPlane.endpoint`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPlaneEndpoint {
    /// URL scheme; must be `https`.
    pub scheme: String,
    /// Host (DNS name or IP literal).
    pub host: String,
    /// TCP port.
    pub port: u16,
}

impl Default for ControlPlaneEndpoint {
    fn default() -> Self {
        ControlPlaneEndpoint {
            scheme: defaults::DEFAULT_ENDPOINT_SCHEME.to_string(),
            host: String::new(),
            port: defaults::DEFAULT_APISERVER_PORT,
        }
    }
}

impl ControlPlaneEndpoint {
    /// Parse from a URL like `https://10.0.0.1:6443`.
    pub fn parse(url: &str) -> Result<Self> {
        let (scheme, rest) = url.split_once("://").ok_or_else(|| {
            Error::parse("endpoint must include a scheme, e.g. https://host:port")
        })?;
        if scheme != "https" {
            return Err(Error::invalid(format!(
                "control plane endpoint scheme must be https, got '{scheme}'"
            )));
        }
        let authority = rest.split('/').next().unwrap_or(rest);
        if authority.is_empty() {
            return Err(Error::parse("endpoint host is empty"));
        }
        let (host, port) = match authority.rsplit_once(':') {
            // Guard against IPv6 literals where the last colon is part of the address.
            Some((h, p)) if !h.contains(':') || authority.starts_with('[') => {
                let port: u16 = p
                    .trim_end_matches(']')
                    .parse()
                    .map_err(|_| Error::parse(format!("invalid port '{p}'")))?;
                (h.trim_start_matches('[').to_string(), port)
            }
            _ => (authority.to_string(), defaults::DEFAULT_APISERVER_PORT),
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

/// The `cluster.discovery` sub-tree: node discovery via the registry and the
/// Talos discovery service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryConfig {
    /// Whether discovery is enabled.
    pub enabled: bool,
    /// Whether the kubernetes registry is used.
    pub registry_kubernetes: bool,
    /// Whether the external Talos discovery service is used.
    pub registry_service: bool,
    /// Discovery service endpoint.
    pub service_endpoint: String,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        DiscoveryConfig {
            enabled: true,
            registry_kubernetes: true,
            registry_service: true,
            service_endpoint: defaults::DEFAULT_DISCOVERY_ENDPOINT.to_string(),
        }
    }
}

impl Validator for DiscoveryConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if self.enabled && self.registry_service && self.service_endpoint.is_empty() {
            report.push(ValidationError::missing(
                "cluster.discovery.registries.service.endpoint",
            ));
        }
        if self.enabled && !self.registry_kubernetes && !self.registry_service {
            report.push(ValidationError::Conflict(
                "cluster.discovery enabled but every registry is disabled".to_string(),
            ));
        }
    }
}

/// Logging destination protocol (`cluster.*`/`machine.logging`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogProtocol {
    Tcp,
    Udp,
}

impl LogProtocol {
    pub fn as_str(self) -> &'static str {
        match self {
            LogProtocol::Tcp => "tcp",
            LogProtocol::Udp => "udp",
        }
    }
}

/// A logging destination (`machine.logging.destinations[]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggingDestination {
    /// Endpoint URL (`tcp://host:port`).
    pub endpoint: String,
    /// Wire format (`json_lines`).
    pub format: String,
    /// Transport protocol.
    pub protocol: LogProtocol,
}

impl Validator for LoggingDestination {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if self.endpoint.is_empty() {
            report.push(ValidationError::missing("logging.destinations[].endpoint"));
        }
        if self.format != "json_lines" {
            report.push(ValidationError::invalid(
                "logging.destinations[].format",
                format!("'{}' is not a supported log format", self.format),
            ));
        }
    }
}

/// The `cluster:` sub-tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterConfig {
    /// Cluster name (`cluster.clusterName`).
    pub name: String,
    /// Cluster id (`cluster.id`).
    pub id: String,
    /// Shared bootstrap secret used to authenticate the cluster (`cluster.secret`).
    pub secret: String,
    /// Control-plane endpoint.
    pub endpoint: ControlPlaneEndpoint,
    /// Pod subnet CIDRs.
    pub pod_subnets: Vec<String>,
    /// Service subnet CIDRs.
    pub service_subnets: Vec<String>,
    /// DNS domain.
    pub dns_domain: String,
    /// CNI plugin name (`flannel`, `custom`, `none`).
    pub cni: String,
    /// etcd config.
    pub etcd: EtcdConfig,
    /// API server config.
    pub apiserver: ApiServerConfig,
    /// Controller-manager config.
    pub controller_manager: ControllerManagerConfig,
    /// Scheduler config.
    pub scheduler: SchedulerConfig,
    /// kube-proxy config.
    pub proxy: ProxyConfig,
    /// Discovery config.
    pub discovery: DiscoveryConfig,
    /// Cluster-wide logging destinations.
    pub logging: Vec<LoggingDestination>,
    /// Whether this control plane should allow scheduling workloads on it
    /// (`cluster.allowSchedulingOnControlPlanes`).
    pub allow_scheduling_on_control_planes: bool,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        ClusterConfig {
            name: String::new(),
            id: String::new(),
            secret: String::new(),
            endpoint: ControlPlaneEndpoint::default(),
            pod_subnets: vec![defaults::DEFAULT_POD_SUBNET.to_string()],
            service_subnets: vec![defaults::DEFAULT_SERVICE_SUBNET.to_string()],
            dns_domain: defaults::DEFAULT_DNS_DOMAIN.to_string(),
            cni: "flannel".to_string(),
            etcd: EtcdConfig::default(),
            apiserver: ApiServerConfig::default(),
            controller_manager: ControllerManagerConfig::default(),
            scheduler: SchedulerConfig::default(),
            proxy: ProxyConfig::default(),
            discovery: DiscoveryConfig::default(),
            logging: Vec::new(),
            allow_scheduling_on_control_planes: false,
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

    /// The first service subnet (used to derive cluster DNS), if any.
    pub fn primary_service_subnet(&self) -> Option<&str> {
        self.service_subnets.first().map(String::as_str)
    }

    /// Apply Talos defaults to the cluster sub-tree.
    pub fn apply_defaults(&mut self) {
        self.etcd.apply_defaults();
    }
}

impl Validator for ClusterConfig {
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
        if self.name.is_empty() {
            report.push(ValidationError::missing("cluster.clusterName"));
        }
        if !self.has_endpoint() {
            report.push(ValidationError::missing("cluster.controlPlane.endpoint"));
        } else if !is_hostname(&self.endpoint.host) && !is_ip(&self.endpoint.host) {
            report.push(ValidationError::invalid(
                "cluster.controlPlane.endpoint",
                format!("'{}' is not a hostname or IP", self.endpoint.host),
            ));
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
        for c in &self.pod_subnets {
            if !is_cidr(c) {
                report.push(ValidationError::invalid(
                    "cluster.network.podSubnets",
                    format!("'{c}' is not a CIDR"),
                ));
            }
        }
        if self.service_subnets.is_empty() {
            report.push(ValidationError::missing("cluster.network.serviceSubnets"));
        }
        for c in &self.service_subnets {
            if !is_cidr(c) {
                report.push(ValidationError::invalid(
                    "cluster.network.serviceSubnets",
                    format!("'{c}' is not a CIDR"),
                ));
            }
        }
        if self.dns_domain.is_empty() {
            report.push(ValidationError::Warning(
                "cluster.network.dnsDomain empty; defaulting to cluster.local".to_string(),
            ));
        } else if !is_hostname(&self.dns_domain) {
            report.push(ValidationError::invalid(
                "cluster.network.dnsDomain",
                format!("'{}' is not a valid DNS domain", self.dns_domain),
            ));
        }
        if !matches!(self.cni.as_str(), "flannel" | "custom" | "none") {
            report.push(ValidationError::invalid(
                "cluster.network.cni.name",
                format!("'{}' is not a supported CNI", self.cni),
            ));
        }
        self.etcd.validate_into(mode, report);
        self.apiserver.validate_into(mode, report);
        self.controller_manager.validate_into(mode, report);
        self.scheduler.validate_into(mode, report);
        self.proxy.validate_into(mode, report);
        self.discovery.validate_into(mode, report);
        for dest in &self.logging {
            dest.validate_into(mode, report);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_cluster() -> ClusterConfig {
        ClusterConfig::new(
            "prod",
            ControlPlaneEndpoint::parse("https://10.0.0.1:6443").unwrap(),
        )
    }

    #[test]
    fn endpoint_parse_variants() {
        let e = ControlPlaneEndpoint::parse("https://api.example.com").unwrap();
        assert_eq!(e.host, "api.example.com");
        assert_eq!(e.port, 6443);
        let e2 = ControlPlaneEndpoint::parse("https://10.0.0.1:8443").unwrap();
        assert_eq!(e2.port, 8443);
        assert!(ControlPlaneEndpoint::parse("http://x").is_err());
        assert!(ControlPlaneEndpoint::parse("noscheme").is_err());
    }

    #[test]
    fn valid_cluster_passes() {
        let c = valid_cluster();
        let warnings = c.validate(ValidationMode::Metal).unwrap();
        assert!(warnings.is_empty());
    }

    #[test]
    fn missing_fields_fail() {
        let c = ClusterConfig::default();
        assert!(c.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn bad_pod_subnet_fails() {
        let mut c = valid_cluster();
        c.pod_subnets = vec!["10.244.0.0".to_string()];
        assert!(c.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn unsupported_cni_fails() {
        let mut c = valid_cluster();
        c.cni = "calico-but-typo".to_string();
        assert!(c.validate(ValidationMode::Metal).is_err());
        c.cni = "none".to_string();
        assert!(c.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn discovery_all_disabled_conflict() {
        let d = DiscoveryConfig {
            registry_kubernetes: false,
            registry_service: false,
            ..DiscoveryConfig::default()
        };
        assert!(d.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn logging_format_validation() {
        let mut c = valid_cluster();
        c.logging.push(LoggingDestination {
            endpoint: "tcp://10.0.0.5:5044".to_string(),
            format: "json_lines".to_string(),
            protocol: LogProtocol::Tcp,
        });
        assert!(c.validate(ValidationMode::Metal).is_ok());
        c.logging[0].format = "syslog".to_string();
        assert!(c.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn primary_service_subnet_used_for_dns() {
        let c = valid_cluster();
        assert_eq!(c.primary_service_subnet(), Some("10.96.0.0/12"));
    }

    #[test]
    fn defaults_have_standard_values() {
        let c = ClusterConfig::default();
        assert_eq!(c.dns_domain, "cluster.local");
        assert_eq!(c.cni, "flannel");
        assert!(c.proxy.mode == "iptables");
    }
}
