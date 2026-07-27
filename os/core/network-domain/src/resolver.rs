//! DNS resolver specs and the resolver merge layer.
//!
//! Mirrors `network.ResolverSpec`, `network.ResolverStatus`,
//! `ResolverConfigController` and `ResolverMergeController`. A
//! [`ResolverSpec`] carries an ordered list of DNS server addresses and an
//! optional set of search domains, tagged with a [`ConfigLayer`]. The merge
//! logic selects the highest-priority non-empty resolver set, mirroring how
//! Talos prefers config-supplied nameservers over DHCP- or platform-supplied
//! ones.

use crate::config_layer::ConfigLayer;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// Maximum number of nameservers honoured by glibc's resolver (`MAXNS`).
pub const MAX_NAMESERVERS: usize = 3;

/// A candidate DNS resolver configuration tagged with its provenance.
///
/// Equivalent to `network.ResolverSpecSpec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverSpec {
    /// Ordered DNS server addresses.
    pub servers: Vec<NodeAddress>,
    /// Optional DNS search domains.
    pub search_domains: Vec<String>,
    /// Provenance / priority of this spec.
    pub layer: ConfigLayer,
}

impl ResolverSpec {
    /// Build a resolver spec from server addresses.
    pub fn new(servers: Vec<NodeAddress>, layer: ConfigLayer) -> Result<Self> {
        let spec = ResolverSpec {
            servers,
            search_domains: Vec::new(),
            layer,
        };
        spec.validate()?;
        Ok(spec)
    }

    /// Build a resolver spec from DNS servers and/or search domains.
    pub fn new_with_search(
        servers: Vec<NodeAddress>,
        search_domains: Vec<String>,
        layer: ConfigLayer,
    ) -> Result<Self> {
        let spec = ResolverSpec {
            servers,
            search_domains,
            layer,
        };
        spec.validate()?;
        Ok(spec)
    }

    /// Attach search domains, validating each as a DNS name fragment.
    pub fn with_search(mut self, domains: Vec<String>) -> Result<Self> {
        self.search_domains = domains;
        self.validate()?;
        Ok(self)
    }

    /// Validate the resolver spec invariants.
    pub fn validate(&self) -> Result<()> {
        if self.servers.is_empty() && self.search_domains.is_empty() {
            return Err(Error::invalid(
                "resolver spec has no servers or search domains",
            ));
        }
        for d in &self.search_domains {
            validate_search_domain(d)?;
        }
        Ok(())
    }

    /// The effective server list, truncated to [`MAX_NAMESERVERS`] and
    /// de-duplicated while preserving order (matching how `resolv.conf` is
    /// written).
    pub fn effective_servers(&self) -> Vec<NodeAddress> {
        let mut out: Vec<NodeAddress> = Vec::new();
        for &s in &self.servers {
            if !out.contains(&s) {
                out.push(s);
            }
            if out.len() == MAX_NAMESERVERS {
                break;
            }
        }
        out
    }

    /// Render a `resolv.conf` body for this resolver set.
    pub fn render_resolv_conf(&self) -> String {
        let mut s = String::new();
        if !self.search_domains.is_empty() {
            s.push_str("search ");
            s.push_str(&self.search_domains.join(" "));
            s.push('\n');
        }
        for srv in self.effective_servers() {
            s.push_str("nameserver ");
            s.push_str(&srv.to_string());
            s.push('\n');
        }
        s
    }
}

fn validate_search_domain(d: &str) -> Result<()> {
    if d.is_empty() {
        return Err(Error::invalid("empty search domain"));
    }
    if d.len() > 253 {
        return Err(Error::invalid("search domain exceeds 253 characters"));
    }
    for label in d.split('.') {
        if label.is_empty() {
            return Err(Error::invalid("search domain has an empty label"));
        }
        for c in label.chars() {
            if !(c.is_ascii_alphanumeric() || c == '-') {
                return Err(Error::invalid(alloc::format!(
                    "invalid search-domain character '{c}'"
                )));
            }
        }
    }
    Ok(())
}

/// The merged, observed resolver state for the node.
///
/// Equivalent to `network.ResolverStatusSpec`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolverStatus {
    /// Effective DNS servers.
    pub servers: Vec<NodeAddress>,
    /// Effective search domains.
    pub search_domains: Vec<String>,
}

/// Fold candidate resolver specs by layer priority. The highest-precedence
/// layer with a non-empty server list wins outright; this mirrors how Talos
/// treats config-supplied nameservers as fully replacing DHCP/platform ones
/// rather than appending.
///
/// Returns `None` when no candidate supplies any server.
pub fn merge_resolvers(specs: &[ResolverSpec]) -> Option<ResolverStatus> {
    let mut best: Option<&ResolverSpec> = None;
    for spec in specs {
        if spec.servers.is_empty() && spec.search_domains.is_empty() {
            continue;
        }
        match best {
            Some(b) if b.layer.precedence() > spec.layer.precedence() => {}
            _ => best = Some(spec),
        }
    }
    best.map(|s| ResolverStatus {
        servers: s.effective_servers(),
        search_domains: s.search_domains.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(s: &str) -> NodeAddress {
        NodeAddress::parse_v4(s).unwrap()
    }

    #[test]
    fn empty_resolver_invalid() {
        assert!(ResolverSpec::new(Vec::new(), ConfigLayer::Configuration).is_err());
    }

    #[test]
    fn search_only_resolver_is_valid() {
        let spec = ResolverSpec::new_with_search(
            Vec::new(),
            alloc::vec!["example.com".to_string()],
            ConfigLayer::Operator,
        )
        .unwrap();
        assert_eq!(
            spec.render_resolv_conf(),
            "search example.com
"
        );
    }

    #[test]
    fn effective_servers_truncate_and_dedup() {
        let spec = ResolverSpec::new(
            alloc::vec![
                v4("1.1.1.1"),
                v4("1.1.1.1"),
                v4("8.8.8.8"),
                v4("9.9.9.9"),
                v4("8.8.4.4"),
            ],
            ConfigLayer::Platform,
        )
        .unwrap();
        let eff = spec.effective_servers();
        assert_eq!(eff.len(), MAX_NAMESERVERS);
        assert_eq!(eff[0], v4("1.1.1.1"));
        assert_eq!(eff[1], v4("8.8.8.8"));
        assert_eq!(eff[2], v4("9.9.9.9"));
    }

    #[test]
    fn search_domain_validation() {
        let spec = ResolverSpec::new(alloc::vec![v4("1.1.1.1")], ConfigLayer::Configuration)
            .unwrap()
            .with_search(alloc::vec!["cluster.local".to_string(), "svc".to_string()]);
        assert!(spec.is_ok());

        let bad = ResolverSpec::new(alloc::vec![v4("1.1.1.1")], ConfigLayer::Configuration)
            .unwrap()
            .with_search(alloc::vec!["bad_domain!".to_string()]);
        assert!(bad.is_err());
    }

    #[test]
    fn render_resolv_conf_output() {
        let spec = ResolverSpec::new(
            alloc::vec![v4("1.1.1.1"), v4("8.8.8.8")],
            ConfigLayer::Configuration,
        )
        .unwrap()
        .with_search(alloc::vec!["cluster.local".to_string()])
        .unwrap();
        let out = spec.render_resolv_conf();
        assert_eq!(
            out,
            "search cluster.local\nnameserver 1.1.1.1\nnameserver 8.8.8.8\n"
        );
    }

    #[test]
    fn merge_prefers_higher_layer_nonempty() {
        let plat = ResolverSpec::new(alloc::vec![v4("8.8.8.8")], ConfigLayer::Platform).unwrap();
        let cfg =
            ResolverSpec::new(alloc::vec![v4("1.1.1.1")], ConfigLayer::Configuration).unwrap();
        let status = merge_resolvers(&[plat, cfg]).unwrap();
        assert_eq!(status.servers, alloc::vec![v4("1.1.1.1")]);
    }

    #[test]
    fn merge_skips_empty_servers() {
        // a spec built directly (bypassing the constructor) with empty servers
        let empty = ResolverSpec {
            servers: Vec::new(),
            search_domains: Vec::new(),
            layer: ConfigLayer::Configuration,
        };
        let plat = ResolverSpec::new(alloc::vec![v4("8.8.8.8")], ConfigLayer::Platform).unwrap();
        let status = merge_resolvers(&[empty, plat]).unwrap();
        assert_eq!(status.servers, alloc::vec![v4("8.8.8.8")]);
    }

    #[test]
    fn merge_empty_is_none() {
        assert!(merge_resolvers(&[]).is_none());
    }
}
