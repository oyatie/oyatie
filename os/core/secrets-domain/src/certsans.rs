//! Certificate Subject Alternative Names (`certSANs`) accumulation.
//!
//! Talos builds the SAN set for the API server, etcd, and Talos API
//! certificates from several sources: the machine config's
//! `.machine.certSANs`, the cluster control-plane endpoint, the per-node
//! hostname/addresses, and a handful of well-known Kubernetes service DNS
//! names. This mirrors `internal/app/machined/pkg/controllers/secrets`'s
//! `CertSANs` resource and the helper that folds all of those inputs into a
//! deduplicated, deterministically-ordered list.

use os_kernel::NodeAddress;
use os_kernel::error::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};

/// A single SAN entry. Talos distinguishes DNS names from IP addresses on the
/// wire (they land in different ASN.1 fields); we keep that distinction so the
/// downstream certificate model can emit the correct SAN type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum San {
    /// A DNS name SAN.
    Dns(String),
    /// An IP-address SAN.
    Ip(NodeAddress),
}

impl PartialOrd for San {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for San {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // DNS sorts before IP; within a kind, sort by string form. `NodeAddress`
        // is not itself `Ord`, so we compare canonical string representations.
        use core::cmp::Ordering;
        match (self, other) {
            (San::Dns(a), San::Dns(b)) => a.cmp(b),
            (San::Ip(a), San::Ip(b)) => a.to_string().cmp(&b.to_string()),
            (San::Dns(_), San::Ip(_)) => Ordering::Less,
            (San::Ip(_), San::Dns(_)) => Ordering::Greater,
        }
    }
}

impl San {
    /// Render to the canonical string form used in PEM dumps and comparisons.
    pub fn to_string_repr(&self) -> String {
        match self {
            San::Dns(d) => d.clone(),
            San::Ip(ip) => ip.to_string(),
        }
    }

    /// Whether this entry is a DNS name.
    pub fn is_dns(&self) -> bool {
        matches!(self, San::Dns(_))
    }

    /// Whether this entry is an IP address.
    pub fn is_ip(&self) -> bool {
        matches!(self, San::Ip(_))
    }
}

/// An accumulator that folds SAN sources into a deduplicated set.
///
/// DNS names and IPs are stored separately (sorted) so that the resulting
/// certificate lists DNS names then IPs in a stable order, regardless of the
/// order inputs were added — exactly what Talos's `CertSANs` controller does to
/// keep certificate contents from churning across reconciles.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CertSans {
    dns: BTreeSet<String>,
    /// IPs keyed by their canonical string form so the set is deduplicated and
    /// deterministically ordered (`NodeAddress` itself is not `Ord`).
    ips: BTreeMap<String, NodeAddress>,
}

impl CertSans {
    /// An empty accumulator.
    pub fn new() -> Self {
        CertSans {
            dns: BTreeSet::new(),
            ips: BTreeMap::new(),
        }
    }

    /// Add a raw SAN string, auto-classifying it as an IP or a DNS name. An
    /// empty string is rejected. This mirrors Talos's `AppendStdSANs` which
    /// parses each `certSANs` entry and routes it to the right list.
    pub fn append(&mut self, raw: &str) -> Result<()> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(Error::invalid("empty SAN"));
        }
        if let Ok(ip) = NodeAddress::parse(raw) {
            self.ips.insert(ip.to_string(), ip);
        } else {
            validate_dns(raw)?;
            self.dns.insert(raw.to_string());
        }
        Ok(())
    }

    /// Add an explicit DNS name.
    pub fn append_dns(&mut self, name: &str) -> Result<()> {
        validate_dns(name)?;
        self.dns.insert(name.to_string());
        Ok(())
    }

    /// Add an explicit IP address.
    pub fn append_ip(&mut self, ip: NodeAddress) {
        self.ips.insert(ip.to_string(), ip);
    }

    /// Fold many raw SAN strings at once, short-circuiting on the first error.
    pub fn extend_raw<'a, I: IntoIterator<Item = &'a str>>(&mut self, it: I) -> Result<()> {
        for s in it {
            self.append(s)?;
        }
        Ok(())
    }

    /// Append the standard Kubernetes API-server service SAN names for a given
    /// cluster domain (e.g. `cluster.local`). These are always present on the
    /// apiserver certificate regardless of user `certSANs`.
    pub fn append_kubernetes_service_sans(&mut self, cluster_domain: &str) -> Result<()> {
        let domain = cluster_domain.trim_matches('.');
        for name in [
            "kubernetes".to_string(),
            "kubernetes.default".to_string(),
            "kubernetes.default.svc".to_string(),
            format!("kubernetes.default.svc.{domain}"),
        ] {
            self.append_dns(&name)?;
        }
        Ok(())
    }

    /// Whether a DNS name is present.
    pub fn contains_dns(&self, name: &str) -> bool {
        self.dns.contains(name)
    }

    /// Whether an IP is present.
    pub fn contains_ip(&self, ip: &NodeAddress) -> bool {
        self.ips.contains_key(&ip.to_string())
    }

    /// The sorted DNS names.
    pub fn dns_names(&self) -> Vec<String> {
        self.dns.iter().cloned().collect()
    }

    /// The sorted IP addresses.
    pub fn ip_addresses(&self) -> Vec<NodeAddress> {
        self.ips.values().cloned().collect()
    }

    /// All SANs as a single ordered list: DNS names first, then IPs.
    pub fn all(&self) -> Vec<San> {
        let mut out: Vec<San> = self.dns.iter().cloned().map(San::Dns).collect();
        out.extend(self.ips.values().cloned().map(San::Ip));
        out
    }

    /// Total number of SAN entries.
    pub fn len(&self) -> usize {
        self.dns.len() + self.ips.len()
    }

    /// Whether there are no SANs at all.
    pub fn is_empty(&self) -> bool {
        self.dns.is_empty() && self.ips.is_empty()
    }

    /// A stable fingerprint over the SAN set, used by controllers to detect
    /// when a certificate must be regenerated because its SANs changed.
    pub fn fingerprint(&self) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for san in self.all() {
            for b in san.to_string_repr().bytes() {
                hash ^= b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            hash ^= 0x2c; // ',' separator
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

/// Validate a DNS name for use as a SAN. Talos allows the wildcard `*` as the
/// left-most label; otherwise labels must be DNS-1123-ish.
pub fn validate_dns(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::invalid("empty DNS SAN"));
    }
    if name.len() > 253 {
        return Err(Error::invalid("DNS SAN exceeds 253 characters"));
    }
    for (i, label) in name.split('.').enumerate() {
        if label.is_empty() {
            return Err(Error::invalid("DNS SAN has an empty label"));
        }
        if label == "*" {
            if i != 0 {
                return Err(Error::invalid("wildcard only allowed in left-most label"));
            }
            continue;
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(Error::invalid("DNS label may not start or end with '-'"));
        }
        for c in label.chars() {
            if !(c.is_ascii_alphanumeric() || c == '-') {
                return Err(Error::invalid(format!("invalid DNS character '{c}'")));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_classifies_ip_vs_dns() {
        let mut s = CertSans::new();
        s.append("10.0.0.1").unwrap();
        s.append("api.cluster.local").unwrap();
        s.append("  ").err().unwrap();
        assert_eq!(s.dns_names(), vec!["api.cluster.local".to_string()]);
        assert_eq!(s.ip_addresses().len(), 1);
        assert!(s.contains_ip(&NodeAddress::parse("10.0.0.1").unwrap()));
    }

    #[test]
    fn dedup_and_ordering_is_stable() {
        let mut a = CertSans::new();
        a.extend_raw([
            "b.example",
            "a.example",
            "10.0.0.2",
            "10.0.0.1",
            "a.example",
        ])
        .unwrap();
        assert_eq!(a.len(), 4);
        let all = a.all();
        // DNS names first (sorted), then IPs (sorted).
        assert_eq!(all[0], San::Dns("a.example".to_string()));
        assert_eq!(all[1], San::Dns("b.example".to_string()));
        assert!(all[2].is_ip());
    }

    #[test]
    fn kubernetes_service_sans_added() {
        let mut s = CertSans::new();
        s.append_kubernetes_service_sans("cluster.local").unwrap();
        assert!(s.contains_dns("kubernetes"));
        assert!(s.contains_dns("kubernetes.default.svc.cluster.local"));
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn fingerprint_changes_with_content() {
        let mut a = CertSans::new();
        a.append("a.example").unwrap();
        let f1 = a.fingerprint();
        a.append("b.example").unwrap();
        assert_ne!(f1, a.fingerprint());

        // Order of insertion does not matter.
        let mut b = CertSans::new();
        b.append("b.example").unwrap();
        b.append("a.example").unwrap();
        assert_eq!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn dns_validation_rules() {
        assert!(validate_dns("foo.bar").is_ok());
        assert!(validate_dns("*.apps.example").is_ok());
        assert!(validate_dns("a.*.b").is_err());
        assert!(validate_dns("-bad.example").is_err());
        assert!(validate_dns("").is_err());
        assert!(validate_dns("foo..bar").is_err());
    }
}
