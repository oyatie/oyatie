//! Hostname specs, status, and the hostname merge layer.
//!
//! Mirrors `network.HostnameSpec`, `network.HostnameStatus`,
//! `HostnameConfigController`, `HostnameMergeController` and
//! `HostnameSpecController`. A [`HostnameSpec`] carries the desired hostname
//! and domain for the node, tagged with a [`ConfigLayer`]; the merge logic
//! folds candidate specs from every layer by priority so the highest-priority
//! source wins. The resulting [`HostnameStatus`] is what the rest of the system
//! (etcd, kubelet, certificates) observes.

use crate::config_layer::ConfigLayer;
use alloc::string::{String, ToString};
use os_kernel::address::{Hostname, NodeAddress};
use os_kernel::error::{Error, Result};

/// A candidate hostname configuration tagged with its provenance.
///
/// Equivalent to `network.HostnameSpecSpec`. `hostname` is the short host
/// portion and `domainname` is the optional DNS domain; together they form the
/// FQDN. Talos derives a default hostname from the node's primary address when
/// nothing else provides one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostnameSpec {
    /// The short hostname (first DNS label), already validated.
    pub hostname: Hostname,
    /// The DNS domain, if any (e.g. `cluster.local`).
    pub domainname: Option<String>,
    /// Provenance / priority of this spec.
    pub layer: ConfigLayer,
}

impl HostnameSpec {
    /// Build a hostname spec from a short hostname string, validating it.
    pub fn new(hostname: impl Into<String>, layer: ConfigLayer) -> Result<Self> {
        let hostname = Hostname::new(hostname)?;
        // The configured "hostname" must be a single label, not an FQDN; the
        // domain is configured separately. Reject anything with a dot.
        if hostname.as_str().contains('.') {
            return Err(Error::invalid(
                "hostname spec must be a single label; use domainname for the domain",
            ));
        }
        Ok(HostnameSpec {
            hostname,
            domainname: None,
            layer,
        })
    }

    /// Build a hostname spec with an explicit domain.
    pub fn with_domain(
        hostname: impl Into<String>,
        domainname: impl Into<String>,
        layer: ConfigLayer,
    ) -> Result<Self> {
        let mut spec = Self::new(hostname, layer)?;
        let domain: String = domainname.into();
        // Validate the domain as a (possibly multi-label) hostname.
        Hostname::new(domain.clone())?;
        spec.domainname = Some(domain);
        Ok(spec)
    }

    /// Derive a default hostname from a node address, mirroring Talos'
    /// `talos-<hyphenated-ip>` scheme used when no other source applies.
    pub fn from_address(addr: NodeAddress, layer: ConfigLayer) -> Result<Self> {
        let host = match addr {
            NodeAddress::V4(o) => {
                alloc::format!("talos-{}-{}-{}-{}", o[0], o[1], o[2], o[3])
            }
            NodeAddress::V6(_) => {
                // For v6 we fall back to a stable hash-free label of the
                // rendered address with separators replaced.
                let rendered = addr.to_string().replace(':', "-");
                alloc::format!("talos-{rendered}")
            }
        };
        Self::new(host, layer)
    }

    /// The fully-qualified domain name (`hostname[.domain]`).
    pub fn fqdn(&self) -> String {
        match &self.domainname {
            Some(d) => alloc::format!("{}.{}", self.hostname.as_str(), d),
            None => self.hostname.as_str().to_string(),
        }
    }
}

/// The merged, observed hostname state for the node.
///
/// Equivalent to `network.HostnameStatusSpec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostnameStatus {
    /// Short hostname.
    pub hostname: String,
    /// DNS domain, if any.
    pub domainname: Option<String>,
}

impl HostnameStatus {
    /// The FQDN derived from the status.
    pub fn fqdn(&self) -> String {
        match &self.domainname {
            Some(d) => alloc::format!("{}.{}", self.hostname, d),
            None => self.hostname.clone(),
        }
    }
}

/// Fold candidate hostname specs by layer priority, returning the winning
/// status. Mirrors `HostnameMergeController`: the highest-precedence layer
/// wins, and ties favour the later spec in input order (matching Talos' stable
/// iteration over deterministically ordered resources).
///
/// Returns `None` when no candidate is supplied.
pub fn merge_hostname(specs: &[HostnameSpec]) -> Option<HostnameStatus> {
    let mut best: Option<&HostnameSpec> = None;
    for spec in specs {
        match best {
            Some(b) if b.layer.precedence() > spec.layer.precedence() => {}
            _ => best = Some(spec),
        }
    }
    best.map(|s| HostnameStatus {
        hostname: s.hostname.as_str().to_string(),
        domainname: s.domainname.clone(),
    })
}

/// Parse a `talos.hostname=` / `ip=...:<hostname>` style cmdline value into a
/// [`HostnameSpec`] at the [`ConfigLayer::Cmdline`] layer.
pub fn parse_cmdline_hostname(value: &str) -> Result<HostnameSpec> {
    let value = value.trim();
    if value.is_empty() {
        return Err(Error::parse("empty cmdline hostname"));
    }
    // The cmdline may carry an FQDN; split the first label as the host and the
    // remainder as the domain.
    match value.split_once('.') {
        Some((host, domain)) => HostnameSpec::with_domain(host, domain, ConfigLayer::Cmdline),
        None => HostnameSpec::new(value, ConfigLayer::Cmdline),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(s: &str) -> NodeAddress {
        NodeAddress::parse_v4(s).unwrap()
    }

    #[test]
    fn new_rejects_fqdn() {
        assert!(HostnameSpec::new("node1", ConfigLayer::Configuration).is_ok());
        assert!(HostnameSpec::new("node1.example.com", ConfigLayer::Configuration).is_err());
    }

    #[test]
    fn with_domain_builds_fqdn() {
        let s = HostnameSpec::with_domain("node1", "cluster.local", ConfigLayer::Configuration)
            .unwrap();
        assert_eq!(s.fqdn(), "node1.cluster.local");
        assert_eq!(s.domainname.as_deref(), Some("cluster.local"));
    }

    #[test]
    fn from_address_v4_scheme() {
        let s = HostnameSpec::from_address(v4("10.0.0.5"), ConfigLayer::Default).unwrap();
        assert_eq!(s.hostname.as_str(), "talos-10-0-0-5");
        assert_eq!(s.fqdn(), "talos-10-0-0-5");
    }

    #[test]
    fn merge_prefers_higher_layer() {
        let dflt = HostnameSpec::from_address(v4("10.0.0.5"), ConfigLayer::Default).unwrap();
        let cfg = HostnameSpec::new("master1", ConfigLayer::Configuration).unwrap();
        let plat = HostnameSpec::new("platformhost", ConfigLayer::Platform).unwrap();

        let status = merge_hostname(&[dflt, plat, cfg]).unwrap();
        assert_eq!(status.hostname, "master1");
    }

    #[test]
    fn merge_empty_is_none() {
        assert!(merge_hostname(&[]).is_none());
    }

    #[test]
    fn cmdline_parsing() {
        let s = parse_cmdline_hostname("worker-7.dc1.example").unwrap();
        assert_eq!(s.hostname.as_str(), "worker-7");
        assert_eq!(s.domainname.as_deref(), Some("dc1.example"));
        assert_eq!(s.layer, ConfigLayer::Cmdline);

        let bare = parse_cmdline_hostname("plainhost").unwrap();
        assert_eq!(bare.domainname, None);

        assert!(parse_cmdline_hostname("   ").is_err());
    }

    #[test]
    fn status_fqdn() {
        let st = HostnameStatus {
            hostname: "h".to_string(),
            domainname: Some("d.com".to_string()),
        };
        assert_eq!(st.fqdn(), "h.d.com");
        let st2 = HostnameStatus {
            hostname: "h".to_string(),
            domainname: None,
        };
        assert_eq!(st2.fqdn(), "h");
    }
}
