//! Kubelet configuration: the validated inputs Talos fans out into the kubelet's
//! `config.yaml`.
//!
//! Mirrors `internal/app/machined/pkg/controllers/k8s.KubeletConfigController`
//! and the `pkg/kubelet` config rendering: cluster DNS, cluster domain, the
//! cgroup driver, extra args (with the set of protected flags Talos owns),
//! extra mounts, and credential providers. The config is validated and then
//! consumed by [`crate::spec`] to render the kubelet process/service spec.

use std::collections::BTreeMap;
use std::fmt;

use os_kernel::error::{Error, Result};

/// The default cluster DNS domain Talos uses.
pub const DEFAULT_CLUSTER_DOMAIN: &str = "cluster.local";

/// The default cgroup driver Talos configures for the kubelet.
pub const DEFAULT_CGROUP_DRIVER: CgroupDriver = CgroupDriver::Systemd;

/// Default path of the kubelet binary.
pub const KUBELET_BINARY_PATH: &str = "/usr/local/bin/kubelet";

/// Default container-runtime endpoint Talos points the kubelet at.
pub const DEFAULT_RUNTIME_ENDPOINT: &str = "unix:///run/containerd/containerd.sock";

/// Kubelet command-line / config-file keys that Talos manages itself and refuses
/// to let users override via `extraArgs`.
///
/// Mirrors the deny-list enforced by Talos's kubelet config controller.
pub const PROTECTED_ARGS: &[&str] = &[
    "hostname-override",
    "kubeconfig",
    "bootstrap-kubeconfig",
    "config",
    "cert-dir",
    "container-runtime-endpoint",
    "node-ip",
    "cgroup-driver",
    "cluster-dns",
    "cluster-domain",
];

/// The cgroup driver the kubelet (and runtime) coordinate on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CgroupDriver {
    /// `cgroupfs` driver.
    CgroupFs,
    /// `systemd` driver (Talos default).
    Systemd,
}

impl CgroupDriver {
    /// The wire string the kubelet expects.
    pub fn as_str(self) -> &'static str {
        match self {
            CgroupDriver::CgroupFs => "cgroupfs",
            CgroupDriver::Systemd => "systemd",
        }
    }

    /// Parse a cgroup driver string.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "cgroupfs" => Ok(CgroupDriver::CgroupFs),
            "systemd" => Ok(CgroupDriver::Systemd),
            other => Err(Error::parse(format!("unknown cgroup driver '{other}'"))),
        }
    }
}

impl fmt::Display for CgroupDriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A bind mount Talos injects into the kubelet container (`extraMounts`).
///
/// Models the OCI mount the kubelet runs with so host paths (e.g. for CSI or
/// device plugins) are visible inside the kubelet's mount namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraMount {
    /// Mount destination inside the kubelet container.
    pub destination: String,
    /// Mount source on the host.
    pub source: String,
    /// Mount type (`bind` by default).
    pub mount_type: String,
    /// Mount options (e.g. `rbind`, `ro`, `rshared`).
    pub options: Vec<String>,
}

impl ExtraMount {
    /// Construct a validated bind mount.
    pub fn bind(
        destination: impl Into<String>,
        source: impl Into<String>,
        options: Vec<String>,
    ) -> Result<Self> {
        let m = ExtraMount {
            destination: destination.into(),
            source: source.into(),
            mount_type: "bind".to_string(),
            options,
        };
        m.validate()?;
        Ok(m)
    }

    /// Validate the mount: absolute paths, non-empty destination/source.
    pub fn validate(&self) -> Result<()> {
        if self.destination.is_empty() || self.source.is_empty() {
            return Err(Error::invalid(
                "extra mount source and destination required",
            ));
        }
        if !self.destination.starts_with('/') {
            return Err(Error::invalid(format!(
                "extra mount destination '{}' must be absolute",
                self.destination
            )));
        }
        if !self.source.starts_with('/') {
            return Err(Error::invalid(format!(
                "extra mount source '{}' must be absolute",
                self.source
            )));
        }
        Ok(())
    }

    /// Whether the mount is read-only (`ro` option present).
    pub fn is_read_only(&self) -> bool {
        self.options.iter().any(|o| o == "ro")
    }
}

/// A kubelet image credential provider (`credentialProviders`).
///
/// Mirrors Talos's per-registry credential-provider plugin wiring used to
/// authenticate image pulls (e.g. ECR, GCR helpers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialProvider {
    /// Provider binary name (under the credential-provider bin dir).
    pub name: String,
    /// Image-match globs this provider answers for (e.g. `*.dkr.ecr.*.amazonaws.com`).
    pub match_images: Vec<String>,
    /// Plugin API version (e.g. `credentialprovider.kubelet.k8s.io/v1`).
    pub api_version: String,
    /// Default cache duration string (e.g. `12h`).
    pub default_cache_duration: String,
}

impl CredentialProvider {
    /// Validate the provider: a name, an api version, and at least one match.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::invalid("credential provider name is empty"));
        }
        if self.api_version.is_empty() {
            return Err(Error::invalid("credential provider apiVersion is empty"));
        }
        if self.match_images.is_empty() {
            return Err(Error::invalid(format!(
                "credential provider '{}' matches no images",
                self.name
            )));
        }
        Ok(())
    }
}

/// Validated kubelet configuration derived from the machine config.
///
/// This is the data behind the kubelet's `config.yaml` plus the few things that
/// end up as process flags. It is produced by validation/merging of the machine
/// config and consumed by the spec renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeletConfig {
    /// The cluster DNS service IP(s) handed to pods (as strings; the IP type
    /// lives in `talos-core` but DNS IPs are kept textual for rendering).
    pub cluster_dns: Vec<String>,
    /// The cluster DNS domain (`cluster.local`).
    pub cluster_domain: String,
    /// The cgroup driver.
    pub cgroup_driver: CgroupDriver,
    /// Whether the kubelet registers the node with the apiserver.
    pub register_node: bool,
    /// Whether to skip node registration's default OS labels.
    pub skip_node_registration: bool,
    /// Extra config args appended verbatim after validation, keyed for
    /// determinism and de-duplication.
    pub extra_args: BTreeMap<String, String>,
    /// Extra mounts injected into the kubelet container.
    pub extra_mounts: Vec<ExtraMount>,
    /// Image credential providers.
    pub credential_providers: Vec<CredentialProvider>,
    /// Whether the kubelet should run with seccomp default enabled.
    pub default_runtime_seccomp_enabled: bool,
}

impl Default for KubeletConfig {
    fn default() -> Self {
        KubeletConfig {
            cluster_dns: Vec::new(),
            cluster_domain: DEFAULT_CLUSTER_DOMAIN.to_string(),
            cgroup_driver: DEFAULT_CGROUP_DRIVER,
            register_node: true,
            skip_node_registration: false,
            extra_args: BTreeMap::new(),
            extra_mounts: Vec::new(),
            credential_providers: Vec::new(),
            default_runtime_seccomp_enabled: false,
        }
    }
}

impl KubeletConfig {
    /// Build a config with a single cluster-DNS IP, deriving it from the service
    /// CIDR per Talos convention (the `.10` address of the service subnet).
    ///
    /// `service_cidr` must be in `a.b.c.d/prefix` form; the returned DNS IP is
    /// the network address with its final octet replaced by `10`.
    pub fn with_dns_from_service_cidr(service_cidr: &str) -> Result<Self> {
        let dns_ip = derive_cluster_dns_ip(service_cidr)?;
        Ok(KubeletConfig {
            cluster_dns: vec![dns_ip],
            ..KubeletConfig::default()
        })
    }

    /// Add an extra kubelet argument, rejecting protected flags Talos owns.
    pub fn with_extra_arg(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self> {
        let key = key.into();
        let bare = key.trim_start_matches('-').to_string();
        if PROTECTED_ARGS.contains(&bare.as_str()) {
            return Err(Error::invalid(format!(
                "kubelet arg '{bare}' is managed by Talos and cannot be overridden"
            )));
        }
        if bare.is_empty() {
            return Err(Error::invalid("kubelet extra arg key is empty"));
        }
        self.extra_args.insert(bare, value.into());
        Ok(self)
    }

    /// Add an extra mount.
    pub fn with_extra_mount(mut self, mount: ExtraMount) -> Result<Self> {
        mount.validate()?;
        self.extra_mounts.push(mount);
        Ok(self)
    }

    /// Add a credential provider.
    pub fn with_credential_provider(mut self, provider: CredentialProvider) -> Result<Self> {
        provider.validate()?;
        self.credential_providers.push(provider);
        Ok(self)
    }

    /// Validate the config is internally consistent.
    pub fn validate(&self) -> Result<()> {
        if self.cluster_dns.is_empty() {
            return Err(Error::invalid("kubelet has no cluster DNS"));
        }
        if self.cluster_domain.is_empty() {
            return Err(Error::invalid("kubelet cluster domain is empty"));
        }
        for m in &self.extra_mounts {
            m.validate()?;
        }
        for p in &self.credential_providers {
            p.validate()?;
        }
        for k in self.extra_args.keys() {
            if PROTECTED_ARGS.contains(&k.as_str()) {
                return Err(Error::invalid(format!(
                    "kubelet arg '{k}' is managed by Talos and cannot be overridden"
                )));
            }
        }
        Ok(())
    }
}

/// Derive the cluster DNS IP from a service CIDR, taking the `.10` host address
/// of the network (Talos / kubeadm convention). IPv4 only.
pub fn derive_cluster_dns_ip(service_cidr: &str) -> Result<String> {
    let (addr, _prefix) = parse_cidr_v4(service_cidr)?;
    let dns = [addr[0], addr[1], addr[2], 10];
    Ok(format!("{}.{}.{}.{}", dns[0], dns[1], dns[2], dns[3]))
}

/// Parse an IPv4 CIDR into (network-address-octets, prefix-length).
fn parse_cidr_v4(cidr: &str) -> Result<([u8; 4], u8)> {
    let (addr_str, prefix_str) = cidr
        .split_once('/')
        .ok_or_else(|| Error::parse(format!("CIDR '{cidr}' missing prefix")))?;
    let prefix: u8 = prefix_str
        .parse()
        .map_err(|_| Error::parse(format!("invalid CIDR prefix in '{cidr}'")))?;
    if prefix > 32 {
        return Err(Error::parse(format!("CIDR prefix /{prefix} out of range")));
    }
    let mut octets = [0u8; 4];
    let mut count = 0;
    for (i, part) in addr_str.split('.').enumerate() {
        if i >= 4 {
            return Err(Error::parse(format!(
                "CIDR address '{addr_str}' has too many octets"
            )));
        }
        octets[i] = part
            .parse()
            .map_err(|_| Error::parse(format!("invalid octet '{part}' in CIDR")))?;
        count += 1;
    }
    if count != 4 {
        return Err(Error::parse(format!(
            "CIDR address '{addr_str}' needs 4 octets"
        )));
    }
    Ok((octets, prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgroup_driver_roundtrip() {
        assert_eq!(
            CgroupDriver::parse("systemd").unwrap(),
            CgroupDriver::Systemd
        );
        assert_eq!(
            CgroupDriver::parse("cgroupfs").unwrap(),
            CgroupDriver::CgroupFs
        );
        assert_eq!(CgroupDriver::Systemd.as_str(), "systemd");
        assert!(CgroupDriver::parse("nope").is_err());
    }

    #[test]
    fn derive_dns_ip_takes_tenth_address() {
        assert_eq!(derive_cluster_dns_ip("10.96.0.0/12").unwrap(), "10.96.0.10");
        assert_eq!(
            derive_cluster_dns_ip("172.20.0.0/16").unwrap(),
            "172.20.0.10"
        );
        assert!(derive_cluster_dns_ip("10.96.0.0").is_err());
        assert!(derive_cluster_dns_ip("10.96.0/12").is_err());
        assert!(derive_cluster_dns_ip("10.96.0.0/64").is_err());
    }

    #[test]
    fn config_from_service_cidr_validates() {
        let cfg = KubeletConfig::with_dns_from_service_cidr("10.96.0.0/12").unwrap();
        assert_eq!(cfg.cluster_dns, vec!["10.96.0.10".to_string()]);
        assert_eq!(cfg.cluster_domain, DEFAULT_CLUSTER_DOMAIN);
        assert_eq!(cfg.cgroup_driver, CgroupDriver::Systemd);
        cfg.validate().unwrap();
    }

    #[test]
    fn protected_args_rejected_on_insert_and_validate() {
        let cfg = KubeletConfig::with_dns_from_service_cidr("10.96.0.0/12").unwrap();
        let err = cfg
            .clone()
            .with_extra_arg("--node-ip", "1.2.3.4")
            .unwrap_err();
        assert_eq!(err.kind(), "invalid");

        // Inserting directly then validating must also catch it.
        let mut bad = cfg;
        bad.extra_args
            .insert("cluster-dns".to_string(), "9.9.9.9".to_string());
        assert!(bad.validate().is_err());
    }

    #[test]
    fn extra_args_dedup_by_key() {
        let cfg = KubeletConfig::with_dns_from_service_cidr("10.96.0.0/12")
            .unwrap()
            .with_extra_arg("max-pods", "100")
            .unwrap()
            .with_extra_arg("max-pods", "200")
            .unwrap();
        assert_eq!(cfg.extra_args.get("max-pods").unwrap(), "200");
        assert_eq!(cfg.extra_args.len(), 1);
    }

    #[test]
    fn extra_mount_validation() {
        let m = ExtraMount::bind(
            "/var/lib/longhorn",
            "/var/lib/longhorn",
            vec!["rbind".into(), "ro".into()],
        )
        .unwrap();
        assert!(m.is_read_only());
        assert!(ExtraMount::bind("relative", "/abs", vec![]).is_err());
        assert!(ExtraMount::bind("/abs", "relative", vec![]).is_err());
    }

    #[test]
    fn credential_provider_validation() {
        let p = CredentialProvider {
            name: "ecr-credential-provider".into(),
            match_images: vec!["*.dkr.ecr.*.amazonaws.com".into()],
            api_version: "credentialprovider.kubelet.k8s.io/v1".into(),
            default_cache_duration: "12h".into(),
        };
        p.validate().unwrap();

        let mut bad = p.clone();
        bad.match_images.clear();
        assert!(bad.validate().is_err());
    }
}
