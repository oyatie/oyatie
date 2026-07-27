//! `NetworkRuleConfig` — host ingress firewall rules.
//!
//! Mirrors `pkg/machinery/config/types/network`. Each document is keyed by
//! `name:` and declares an ingress rule: a protocol, a set of port ranges, and
//! a list of allowed source CIDR subnets. The Talos network firewall
//! controller compiles these into nftables rules.

use crate::document::{ConfigDocument, DocId, DocKind};
use os_kernel::error::{Error, Result};

/// Transport protocol for an ingress rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// TCP.
    Tcp,
    /// UDP.
    Udp,
    /// ICMP (port ranges ignored).
    Icmp,
    /// `ICMPv6` (port ranges ignored).
    Icmpv6,
}

impl Protocol {
    /// Canonical lowercase string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
            Protocol::Icmp => "icmp",
            Protocol::Icmpv6 => "icmpv6",
        }
    }

    /// Parse from a protocol string.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "tcp" => Protocol::Tcp,
            "udp" => Protocol::Udp,
            "icmp" => Protocol::Icmp,
            "icmpv6" => Protocol::Icmpv6,
            _ => return None,
        })
    }

    /// Whether this protocol uses port ranges.
    #[must_use]
    pub fn uses_ports(self) -> bool {
        matches!(self, Protocol::Tcp | Protocol::Udp)
    }
}

/// An inclusive port range `[lo, hi]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortRange {
    /// Lower bound (inclusive).
    pub lo: u16,
    /// Upper bound (inclusive).
    pub hi: u16,
}

impl PortRange {
    /// A single port.
    #[must_use]
    pub fn single(port: u16) -> Self {
        PortRange { lo: port, hi: port }
    }

    /// A range.
    #[must_use]
    pub fn new(lo: u16, hi: u16) -> Self {
        PortRange { lo, hi }
    }

    /// Whether two ranges overlap.
    #[must_use]
    pub fn overlaps(self, other: PortRange) -> bool {
        self.lo <= other.hi && other.lo <= self.hi
    }

    fn validate(self) -> Result<()> {
        if self.lo == 0 {
            return Err(Error::invalid("NetworkRuleConfig: port 0 is not allowed"));
        }
        if self.lo > self.hi {
            return Err(Error::invalid(format!(
                "NetworkRuleConfig: port range {}-{} is inverted",
                self.lo, self.hi
            )));
        }
        Ok(())
    }
}

/// The ingress portion of a network rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressRule {
    /// Allowed source subnets in CIDR form (e.g. `10.0.0.0/8`).
    pub subnets: Vec<String>,
}

impl IngressRule {
    /// Construct from a list of subnets.
    pub fn new(subnets: impl IntoIterator<Item = String>) -> Self {
        IngressRule {
            subnets: subnets.into_iter().collect(),
        }
    }
}

/// The `NetworkRuleConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkRuleConfig {
    /// Rule name (document key).
    pub name: String,
    /// Transport protocol.
    pub protocol: Protocol,
    /// Destination port ranges (ignored for ICMP).
    pub port_ranges: Vec<PortRange>,
    /// Ingress source restriction.
    pub ingress: IngressRule,
}

impl NetworkRuleConfig {
    /// Construct a TCP rule by default.
    pub fn new(name: impl Into<String>, protocol: Protocol) -> Self {
        NetworkRuleConfig {
            name: name.into(),
            protocol,
            port_ranges: Vec::new(),
            ingress: IngressRule::new([]),
        }
    }

    /// Builder: add a port range.
    #[must_use]
    pub fn with_port_range(mut self, range: PortRange) -> Self {
        self.port_ranges.push(range);
        self
    }

    /// Builder: set ingress subnets.
    pub fn with_subnets(mut self, subnets: impl IntoIterator<Item = String>) -> Self {
        self.ingress = IngressRule::new(subnets);
        self
    }

    /// Validate a CIDR string of the simplistic `a.b.c.d/n` IPv4 form (the
    /// controller delegates full parsing to the netlink layer; here we model
    /// the structural checks Talos performs at config load time).
    fn validate_cidr(cidr: &str) -> Result<()> {
        let (addr, prefix) = cidr.split_once('/').ok_or_else(|| {
            Error::invalid(format!("NetworkRuleConfig: subnet '{cidr}' must be CIDR"))
        })?;
        let prefix: u8 = prefix
            .parse()
            .map_err(|_| Error::invalid(format!("NetworkRuleConfig: bad prefix in '{cidr}'")))?;
        let is_v6 = addr.contains(':');
        let max = if is_v6 { 128 } else { 32 };
        if prefix > max {
            return Err(Error::invalid(format!(
                "NetworkRuleConfig: prefix /{prefix} out of range for '{cidr}'"
            )));
        }
        if !is_v6 {
            let octets: Vec<&str> = addr.split('.').collect();
            if octets.len() != 4 {
                return Err(Error::invalid(format!(
                    "NetworkRuleConfig: malformed IPv4 address in '{cidr}'"
                )));
            }
            for o in octets {
                o.parse::<u8>().map_err(|_| {
                    Error::invalid(format!("NetworkRuleConfig: bad octet in '{cidr}'"))
                })?;
            }
        }
        Ok(())
    }
}

impl ConfigDocument for NetworkRuleConfig {
    fn kind(&self) -> DocKind {
        DocKind::NetworkRule
    }

    fn id(&self) -> DocId {
        DocId::keyed(DocKind::NetworkRule, self.name.clone())
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("NetworkRuleConfig: name is required"));
        }
        if self.protocol.uses_ports() {
            if self.port_ranges.is_empty() {
                return Err(Error::invalid(format!(
                    "NetworkRuleConfig '{}': {} rule requires at least one port range",
                    self.name,
                    self.protocol.as_str()
                )));
            }
            for r in &self.port_ranges {
                r.validate()?;
            }
            // Overlapping port ranges within one rule are a config error.
            for (i, a) in self.port_ranges.iter().enumerate() {
                for b in &self.port_ranges[i + 1..] {
                    if a.overlaps(*b) {
                        return Err(Error::invalid(format!(
                            "NetworkRuleConfig '{}': overlapping port ranges {}-{} and {}-{}",
                            self.name, a.lo, a.hi, b.lo, b.hi
                        )));
                    }
                }
            }
        }
        if self.ingress.subnets.is_empty() {
            return Err(Error::invalid(format!(
                "NetworkRuleConfig '{}': ingress must specify at least one subnet",
                self.name
            )));
        }
        for s in &self.ingress.subnets {
            Self::validate_cidr(s)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule() -> NetworkRuleConfig {
        NetworkRuleConfig::new("kubelet", Protocol::Tcp)
            .with_port_range(PortRange::single(10250))
            .with_subnets(["10.0.0.0/8".to_string()])
    }

    #[test]
    fn valid_rule() {
        assert!(rule().validate().is_ok());
        assert!(rule().allows_multiple());
    }

    #[test]
    fn protocol_parse() {
        assert_eq!(Protocol::parse("TCP"), Some(Protocol::Tcp));
        assert_eq!(Protocol::parse("icmpv6"), Some(Protocol::Icmpv6));
        assert_eq!(Protocol::parse("sctp"), None);
        assert!(!Protocol::Icmp.uses_ports());
    }

    #[test]
    fn tcp_without_ports_rejected() {
        let r = NetworkRuleConfig::new("x", Protocol::Tcp).with_subnets(["10.0.0.0/8".into()]);
        assert!(r.validate().is_err());
    }

    #[test]
    fn icmp_needs_no_ports() {
        let r = NetworkRuleConfig::new("ping", Protocol::Icmp).with_subnets(["0.0.0.0/0".into()]);
        assert!(r.validate().is_ok());
    }

    #[test]
    fn inverted_range_rejected() {
        let r = rule().with_port_range(PortRange::new(200, 100));
        assert!(r.validate().is_err());
    }

    #[test]
    fn port_zero_rejected() {
        let r = NetworkRuleConfig::new("x", Protocol::Udp)
            .with_port_range(PortRange::single(0))
            .with_subnets(["10.0.0.0/8".into()]);
        assert!(r.validate().is_err());
    }

    #[test]
    fn overlapping_ranges_rejected() {
        let r = rule().with_port_range(PortRange::new(10240, 10260));
        // 10250 single overlaps 10240-10260
        assert!(r.validate().is_err());
    }

    #[test]
    fn missing_subnet_rejected() {
        let r = NetworkRuleConfig::new("x", Protocol::Tcp).with_port_range(PortRange::single(80));
        assert!(r.validate().is_err());
    }

    #[test]
    fn bad_cidr_rejected() {
        let r = NetworkRuleConfig::new("x", Protocol::Tcp)
            .with_port_range(PortRange::single(80))
            .with_subnets(["10.0.0.0".into()]);
        assert!(r.validate().is_err());
        let r = NetworkRuleConfig::new("x", Protocol::Tcp)
            .with_port_range(PortRange::single(80))
            .with_subnets(["10.0.0.0/40".into()]);
        assert!(r.validate().is_err());
        let r = NetworkRuleConfig::new("x", Protocol::Tcp)
            .with_port_range(PortRange::single(80))
            .with_subnets(["999.0.0.0/8".into()]);
        assert!(r.validate().is_err());
    }

    #[test]
    fn ipv6_cidr_ok() {
        let r = NetworkRuleConfig::new("x", Protocol::Tcp)
            .with_port_range(PortRange::single(80))
            .with_subnets(["fd00::/8".into()]);
        assert!(r.validate().is_ok());
    }
}
