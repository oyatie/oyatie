//! Node IP, labels, and taints.
//!
//! Mirrors `internal/app/machined/pkg/controllers/k8s.NodeIPController` plus the
//! node labels/taints config surface. The node-IP controller selects which of
//! the machine's addresses the kubelet advertises (`--node-ip`) by filtering the
//! candidate addresses through the configured `validSubnets` (include/exclude
//! CIDRs), preferring one address per IP family and skipping loopback.

use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// A CIDR filter entry: either an inclusion or a negated exclusion.
///
/// Talos `validSubnets` accepts entries like `10.0.0.0/8` (include) and
/// `!10.0.0.0/16` (exclude). Exclusions take precedence over inclusions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubnetFilter {
    network: [u8; 4],
    prefix: u8,
    exclude: bool,
}

impl SubnetFilter {
    /// Parse a single subnet filter (IPv4). A leading `!` marks an exclusion.
    pub fn parse(s: &str) -> Result<Self> {
        let (exclude, rest) = match s.strip_prefix('!') {
            Some(r) => (true, r),
            None => (false, s),
        };
        let (addr_str, prefix_str) = rest
            .split_once('/')
            .ok_or_else(|| Error::parse(format!("subnet '{rest}' missing prefix")))?;
        let prefix: u8 = prefix_str
            .parse()
            .map_err(|_| Error::parse(format!("invalid prefix in '{rest}'")))?;
        if prefix > 32 {
            return Err(Error::parse(format!("prefix /{prefix} out of range")));
        }
        let mut octets = [0u8; 4];
        let mut count = 0;
        for (i, part) in addr_str.split('.').enumerate() {
            if i >= 4 {
                return Err(Error::parse(format!(
                    "address '{addr_str}' has too many octets"
                )));
            }
            octets[i] = part
                .parse()
                .map_err(|_| Error::parse(format!("invalid octet '{part}'")))?;
            count += 1;
        }
        if count != 4 {
            return Err(Error::parse(format!("address '{addr_str}' needs 4 octets")));
        }
        Ok(SubnetFilter {
            network: octets,
            prefix,
            exclude,
        })
    }

    /// Whether the given address falls inside this subnet (ignoring polarity).
    pub fn contains(&self, addr: &NodeAddress) -> bool {
        let octets = match addr {
            NodeAddress::V4(o) => *o,
            NodeAddress::V6(_) => return false,
        };
        let a = u32::from_be_bytes(octets);
        let n = u32::from_be_bytes(self.network);
        if self.prefix == 0 {
            return true;
        }
        let mask: u32 = u32::MAX << (32 - self.prefix);
        (a & mask) == (n & mask)
    }

    /// Whether this is an exclusion filter.
    pub fn is_exclude(&self) -> bool {
        self.exclude
    }
}

/// The node-IP selection spec the controller reconciles.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeIpSpec {
    /// Candidate addresses (the machine's current addresses), in priority order.
    pub candidates: Vec<NodeAddress>,
    /// Subnet filters; empty means "accept any non-loopback address".
    pub valid_subnets: Vec<SubnetFilter>,
}

impl NodeIpSpec {
    /// Whether a candidate address is acceptable under the filters.
    ///
    /// An address is rejected if it is loopback, if any exclusion matches, or if
    /// inclusions exist and none match it.
    pub fn accepts(&self, addr: &NodeAddress) -> bool {
        if addr.is_loopback() {
            return false;
        }
        let includes: Vec<&SubnetFilter> =
            self.valid_subnets.iter().filter(|f| !f.exclude).collect();
        for f in self.valid_subnets.iter().filter(|f| f.exclude) {
            if f.contains(addr) {
                return false;
            }
        }
        if includes.is_empty() {
            return true;
        }
        includes.iter().any(|f| f.contains(addr))
    }

    /// Reconcile the node IPs: at most one address per IP family, in candidate
    /// priority order, that passes the filters. Talos advertises up to one IPv4
    /// and one IPv6 address.
    pub fn reconcile(&self) -> Result<Vec<NodeAddress>> {
        let mut chosen: Vec<NodeAddress> = Vec::new();
        let mut have_v4 = false;
        let mut have_v6 = false;
        for addr in &self.candidates {
            if !self.accepts(addr) {
                continue;
            }
            match addr {
                NodeAddress::V4(_) if !have_v4 => {
                    have_v4 = true;
                    chosen.push(*addr);
                }
                NodeAddress::V6(_) if !have_v6 => {
                    have_v6 = true;
                    chosen.push(*addr);
                }
                _ => {}
            }
        }
        if chosen.is_empty() {
            return Err(Error::not_found(
                "no node IP matches the configured subnets",
            ));
        }
        Ok(chosen)
    }
}

/// A node label (`key=value`) applied at kubelet registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeLabel {
    /// Label key (may contain an optional `prefix/` part).
    pub key: String,
    /// Label value.
    pub value: String,
}

impl NodeLabel {
    /// Validate and construct a node label. Reserved `kubernetes.io` and
    /// `k8s.io` prefixes are rejected for user labels, matching Talos's guard
    /// against clients setting restricted labels via the kubelet.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let value = value.into();
        if key.is_empty() {
            return Err(Error::invalid("node label key is empty"));
        }
        if let Some((prefix, name)) = key.split_once('/') {
            if prefix.is_empty() || name.is_empty() {
                return Err(Error::invalid(format!("malformed node label key '{key}'")));
            }
            if is_restricted_label_prefix(prefix) {
                return Err(Error::permission_denied(format!(
                    "node label prefix '{prefix}' is restricted"
                )));
            }
        }
        if value.len() > 63 {
            return Err(Error::invalid("node label value exceeds 63 characters"));
        }
        Ok(NodeLabel { key, value })
    }
}

fn is_restricted_label_prefix(prefix: &str) -> bool {
    prefix == "kubernetes.io"
        || prefix == "k8s.io"
        || prefix.ends_with(".kubernetes.io")
        || prefix.ends_with(".k8s.io")
}

/// The scheduling effect of a node taint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaintEffect {
    /// Pods that don't tolerate are not scheduled.
    NoSchedule,
    /// Like NoSchedule, but only a soft preference.
    PreferNoSchedule,
    /// Pods that don't tolerate are evicted.
    NoExecute,
}

impl TaintEffect {
    /// The wire string for the effect.
    pub fn as_str(self) -> &'static str {
        match self {
            TaintEffect::NoSchedule => "NoSchedule",
            TaintEffect::PreferNoSchedule => "PreferNoSchedule",
            TaintEffect::NoExecute => "NoExecute",
        }
    }

    /// Parse an effect string.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "NoSchedule" => Ok(TaintEffect::NoSchedule),
            "PreferNoSchedule" => Ok(TaintEffect::PreferNoSchedule),
            "NoExecute" => Ok(TaintEffect::NoExecute),
            other => Err(Error::parse(format!("unknown taint effect '{other}'"))),
        }
    }
}

/// A node taint applied at registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeTaint {
    /// Taint key.
    pub key: String,
    /// Optional taint value.
    pub value: Option<String>,
    /// Scheduling effect.
    pub effect: TaintEffect,
}

impl NodeTaint {
    /// The control-plane taint Talos applies to control-plane nodes.
    pub fn control_plane() -> Self {
        NodeTaint {
            key: "node-role.kubernetes.io/control-plane".to_string(),
            value: None,
            effect: TaintEffect::NoSchedule,
        }
    }

    /// Render the taint as `key[=value]:effect`.
    pub fn render(&self) -> String {
        match &self.value {
            Some(v) => format!("{}={}:{}", self.key, v, self.effect.as_str()),
            None => format!("{}:{}", self.key, self.effect.as_str()),
        }
    }

    /// Parse a taint from `key[=value]:effect`.
    pub fn parse(s: &str) -> Result<Self> {
        let (kv, effect_str) = s
            .rsplit_once(':')
            .ok_or_else(|| Error::parse(format!("taint '{s}' missing effect")))?;
        let effect = TaintEffect::parse(effect_str)?;
        let (key, value) = match kv.split_once('=') {
            Some((k, v)) => (k.to_string(), Some(v.to_string())),
            None => (kv.to_string(), None),
        };
        if key.is_empty() {
            return Err(Error::parse(format!("taint '{s}' has empty key")));
        }
        Ok(NodeTaint { key, value, effect })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(s: &str) -> NodeAddress {
        NodeAddress::parse_v4(s).unwrap()
    }

    #[test]
    fn subnet_filter_contains() {
        let f = SubnetFilter::parse("10.0.0.0/8").unwrap();
        assert!(f.contains(&v4("10.5.6.7")));
        assert!(!f.contains(&v4("11.0.0.1")));
        assert!(!f.is_exclude());

        let ex = SubnetFilter::parse("!192.168.0.0/16").unwrap();
        assert!(ex.is_exclude());
        assert!(ex.contains(&v4("192.168.5.5")));
    }

    #[test]
    fn node_ip_picks_first_acceptable_per_family() {
        let spec = NodeIpSpec {
            candidates: vec![
                v4("127.0.0.1"),
                v4("10.0.0.5"),
                v4("10.0.0.6"),
                NodeAddress::V6([0xfd00, 0, 0, 0, 0, 0, 0, 1]),
            ],
            valid_subnets: vec![],
        };
        let ips = spec.reconcile().unwrap();
        assert_eq!(ips.len(), 2);
        assert_eq!(ips[0], v4("10.0.0.5"));
        assert!(matches!(ips[1], NodeAddress::V6(_)));
    }

    #[test]
    fn exclusion_overrides_inclusion() {
        let spec = NodeIpSpec {
            candidates: vec![v4("10.0.1.5"), v4("10.5.0.9")],
            valid_subnets: vec![
                SubnetFilter::parse("10.0.0.0/8").unwrap(),
                SubnetFilter::parse("!10.0.1.0/24").unwrap(),
            ],
        };
        let ips = spec.reconcile().unwrap();
        assert_eq!(ips, vec![v4("10.5.0.9")]);
    }

    #[test]
    fn no_matching_address_errors() {
        let spec = NodeIpSpec {
            candidates: vec![v4("127.0.0.1")],
            valid_subnets: vec![],
        };
        assert_eq!(spec.reconcile().unwrap_err().kind(), "not_found");
    }

    #[test]
    fn node_label_restricted_prefix() {
        assert!(NodeLabel::new("topology.kubernetes.io/zone", "us-east-1a").is_err());
        assert_eq!(
            NodeLabel::new("topology.kubernetes.io/zone", "us-east-1a")
                .unwrap_err()
                .kind(),
            "permission_denied"
        );
        let ok = NodeLabel::new("example.com/pool", "gpu").unwrap();
        assert_eq!(ok.key, "example.com/pool");
    }

    #[test]
    fn taint_render_and_parse_roundtrip() {
        let t = NodeTaint::control_plane();
        assert_eq!(
            t.render(),
            "node-role.kubernetes.io/control-plane:NoSchedule"
        );
        let parsed = NodeTaint::parse("dedicated=gpu:NoExecute").unwrap();
        assert_eq!(parsed.key, "dedicated");
        assert_eq!(parsed.value, Some("gpu".to_string()));
        assert_eq!(parsed.effect, TaintEffect::NoExecute);
        assert!(NodeTaint::parse("bad-no-effect").is_err());
    }
}
