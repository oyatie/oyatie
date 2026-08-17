//! The `cluster.etcd` sub-tree plus the static control-plane component configs
//! (apiServer, controllerManager, scheduler, proxy), mirroring `EtcdConfig`,
//! `APIServerConfig`, `ControllerManagerConfig`, `SchedulerConfig`, and
//! `ProxyConfig` in `pkg/machinery/config/types/v1alpha1`.

use crate::defaults;
use crate::validation::{
    ValidationError, ValidationMode, ValidationReport, Validator, is_cidr, is_identifier,
    is_image_ref,
};

/// The `cluster.etcd` sub-tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EtcdConfig {
    /// etcd container image.
    pub image: String,
    /// Subnets etcd advertises/listens on (`advertisedSubnets` / `listenSubnets`).
    pub advertised_subnets: Vec<String>,
    /// Listen subnets.
    pub listen_subnets: Vec<String>,
    /// Extra etcd args.
    pub extra_args: Vec<(String, String)>,
    /// Optional explicit data directory quota in bytes.
    pub quota_backend_bytes: Option<i64>,
}

impl EtcdConfig {
    /// Apply defaults to the image.
    pub fn apply_defaults(&mut self) {
        if self.image.is_empty() {
            self.image = "gcr.io/etcd-development/etcd".to_string();
        }
    }
}

impl Validator for EtcdConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if !self.image.is_empty() && !is_image_ref(&self.image) {
            report.push(ValidationError::invalid(
                "cluster.etcd.image",
                format!("'{}' is not a valid image reference", self.image),
            ));
        }
        for subnet in self
            .advertised_subnets
            .iter()
            .chain(self.listen_subnets.iter())
        {
            let s = subnet.strip_prefix('!').unwrap_or(subnet);
            if !is_cidr(s) {
                report.push(ValidationError::invalid(
                    "cluster.etcd.advertisedSubnets",
                    format!("'{subnet}' is not a CIDR"),
                ));
            }
        }
        for (k, _) in &self.extra_args {
            // Talos manages these etcd flags itself and forbids overriding them.
            if matches!(
                k.as_str(),
                "name"
                    | "data-dir"
                    | "listen-peer-urls"
                    | "listen-client-urls"
                    | "cert-file"
                    | "key-file"
                    | "trusted-ca-file"
                    | "peer-cert-file"
                    | "peer-key-file"
                    | "peer-trusted-ca-file"
            ) {
                report.push(ValidationError::Conflict(format!(
                    "cluster.etcd.extraArgs may not override '{k}'"
                )));
            }
        }
        if let Some(q) = self.quota_backend_bytes
            && q <= 0
        {
            report.push(ValidationError::invalid(
                "cluster.etcd.quotaBackendBytes",
                "must be positive",
            ));
        }
    }
}

/// Shared shape of the static control-plane components: image plus extra args
/// plus extra volume mounts. apiServer/controllerManager/scheduler all share
/// this, with apiServer adding cert SANs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ControlPlaneComponent {
    /// Component container image (empty = use the Kubernetes default).
    pub image: String,
    /// Extra command-line args.
    pub extra_args: Vec<(String, String)>,
    /// Whether to disable the component (only meaningful for scheduler / CM
    /// in some Talos profiles).
    pub disabled: bool,
}

impl ControlPlaneComponent {
    fn validate_common(&self, field: &str, report: &mut ValidationReport) {
        if !self.image.is_empty() && !is_image_ref(&self.image) {
            report.push(ValidationError::invalid(
                format!("{field}.image"),
                format!("'{}' is not a valid image reference", self.image),
            ));
        }
        for (k, _) in &self.extra_args {
            if !is_identifier(k) {
                report.push(ValidationError::invalid(
                    format!("{field}.extraArgs"),
                    format!("'{k}' is not a valid flag name"),
                ));
            }
        }
    }
}

/// `cluster.apiServer`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApiServerConfig {
    /// Shared component config.
    pub component: ControlPlaneComponent,
    /// Additional cert SANs the API server certificate must cover.
    pub cert_sans: Vec<String>,
    /// Admission plugins to enable/disable, etc., kept as raw flags in
    /// `component.extra_args`; this list captures explicit disabled plugins.
    pub disable_pod_security_policy: bool,
}

impl Validator for ApiServerConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        self.component.validate_common("cluster.apiServer", report);
        // apiServer extra args that Talos forbids overriding.
        for (k, _) in &self.component.extra_args {
            if matches!(
                k.as_str(),
                "etcd-servers" | "client-ca-file" | "service-account-key-file" | "tls-cert-file"
            ) {
                report.push(ValidationError::Conflict(format!(
                    "cluster.apiServer.extraArgs may not override '{k}'"
                )));
            }
        }
    }
}

/// `cluster.controllerManager`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ControllerManagerConfig {
    pub component: ControlPlaneComponent,
}

impl Validator for ControllerManagerConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        self.component
            .validate_common("cluster.controllerManager", report);
    }
}

/// `cluster.scheduler`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchedulerConfig {
    pub component: ControlPlaneComponent,
}

impl Validator for SchedulerConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        self.component.validate_common("cluster.scheduler", report);
    }
}

/// `cluster.proxy` (kube-proxy).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyConfig {
    /// kube-proxy image.
    pub image: String,
    /// Proxy mode (`iptables` / `ipvs`).
    pub mode: String,
    /// Whether kube-proxy is disabled entirely (CNI-managed service routing).
    pub disabled: bool,
    /// Extra args.
    pub extra_args: Vec<(String, String)>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        ProxyConfig {
            image: String::new(),
            mode: defaults::DEFAULT_PROXY_MODE.to_string(),
            disabled: false,
            extra_args: Vec::new(),
        }
    }
}

impl Validator for ProxyConfig {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if !self.disabled && !matches!(self.mode.as_str(), "iptables" | "ipvs" | "nftables") {
            report.push(ValidationError::invalid(
                "cluster.proxy.mode",
                format!("'{}' is not a supported proxy mode", self.mode),
            ));
        }
        if !self.image.is_empty() && !is_image_ref(&self.image) {
            report.push(ValidationError::invalid(
                "cluster.proxy.image",
                format!("'{}' is not a valid image reference", self.image),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etcd_forbids_managed_args() {
        let mut e = EtcdConfig::default();
        e.extra_args
            .push(("data-dir".to_string(), "/x".to_string()));
        assert!(e.validate(ValidationMode::Metal).is_err());
        e.extra_args = vec![("election-timeout".to_string(), "1000".to_string())];
        assert!(e.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn etcd_subnet_must_be_cidr() {
        let mut e = EtcdConfig {
            advertised_subnets: vec!["10.0.0.0/8".to_string()],
            ..EtcdConfig::default()
        };
        assert!(e.validate(ValidationMode::Metal).is_ok());
        e.advertised_subnets = vec!["10.0.0.1".to_string()];
        assert!(e.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn etcd_negated_subnet_ok() {
        let e = EtcdConfig {
            listen_subnets: vec!["!10.0.0.0/8".to_string()],
            ..EtcdConfig::default()
        };
        assert!(e.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn etcd_quota_positive() {
        let mut e = EtcdConfig {
            quota_backend_bytes: Some(-1),
            ..EtcdConfig::default()
        };
        assert!(e.validate(ValidationMode::Metal).is_err());
        e.quota_backend_bytes = Some(8 * 1024 * 1024 * 1024);
        assert!(e.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn apiserver_forbids_managed_args() {
        let mut a = ApiServerConfig::default();
        a.component
            .extra_args
            .push(("etcd-servers".to_string(), "x".to_string()));
        assert!(a.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn proxy_mode_validation() {
        let mut p = ProxyConfig::default();
        assert!(p.validate(ValidationMode::Metal).is_ok());
        p.mode = "bpf".to_string();
        assert!(p.validate(ValidationMode::Metal).is_err());
        // A disabled proxy skips the mode check.
        p.disabled = true;
        assert!(p.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn scheduler_and_cm_image_validated() {
        let mut s = SchedulerConfig::default();
        s.component.image = "bad image".to_string();
        assert!(s.validate(ValidationMode::Metal).is_err());
        let mut c = ControllerManagerConfig::default();
        c.component.image = "registry.k8s.io/kube-controller-manager:v1.30.0".to_string();
        assert!(c.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn etcd_default_image() {
        let mut e = EtcdConfig::default();
        e.apply_defaults();
        assert!(!e.image.is_empty());
    }
}
