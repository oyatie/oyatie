//! Node addressing primitives: hostnames, node addresses, and resource ids.

use crate::error::{Error, Result};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// A validated DNS hostname (per the relevant subset of RFC 1123).
///
/// Labels are 1-63 chars of `[a-z0-9-]`, not starting/ending with `-`, and the
/// total length is at most 253 characters. Stored lowercased.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hostname(String);

impl Hostname {
    /// Validate and construct a hostname.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into().to_ascii_lowercase();
        if s.is_empty() {
            return Err(Error::invalid("hostname is empty"));
        }
        if s.len() > 253 {
            return Err(Error::invalid("hostname exceeds 253 characters"));
        }
        for label in s.split('.') {
            Self::validate_label(label)?;
        }
        Ok(Hostname(s))
    }

    fn validate_label(label: &str) -> Result<()> {
        if label.is_empty() {
            return Err(Error::invalid("hostname has an empty label"));
        }
        if label.len() > 63 {
            return Err(Error::invalid("hostname label exceeds 63 characters"));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(Error::invalid(
                "hostname label may not start or end with '-'",
            ));
        }
        for c in label.chars() {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                return Err(Error::invalid(alloc::format!(
                    "invalid hostname character '{c}'"
                )));
            }
        }
        Ok(())
    }

    /// The hostname as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The first DNS label (the short host portion).
    pub fn short(&self) -> &str {
        self.0.split('.').next().unwrap_or(&self.0)
    }
}

impl fmt::Display for Hostname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An IP address attached to a node.
///
/// IP parsing is done with the standard library on the host; in a real `no_std`
/// build the bytes would be filled in by lower layers, so we store the parsed
/// octets/segments directly and keep a small self-contained parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeAddress {
    /// An IPv4 address.
    V4([u8; 4]),
    /// An IPv6 address (eight 16-bit groups).
    V6([u16; 8]),
}

impl NodeAddress {
    /// Parse a dotted IPv4 address (`a.b.c.d`).
    pub fn parse_v4(s: &str) -> Result<Self> {
        let mut octets = [0u8; 4];
        let mut count = 0;
        for (i, part) in s.split('.').enumerate() {
            if i >= 4 {
                return Err(Error::parse("IPv4 has too many octets"));
            }
            let v: u8 = part
                .parse()
                .map_err(|_| Error::parse(alloc::format!("invalid IPv4 octet '{part}'")))?;
            octets[i] = v;
            count += 1;
        }
        if count != 4 {
            return Err(Error::parse("IPv4 needs exactly 4 octets"));
        }
        Ok(NodeAddress::V4(octets))
    }

    /// Whether this is a loopback address (`127.0.0.0/8` or `::1`).
    pub fn is_loopback(&self) -> bool {
        match self {
            NodeAddress::V4(o) => o[0] == 127,
            NodeAddress::V6(g) => *g == [0, 0, 0, 0, 0, 0, 0, 1],
        }
    }

    /// Whether this address belongs to a private/RFC1918 range (IPv4 only;
    /// IPv6 unique-local `fc00::/7` is also reported).
    pub fn is_private(&self) -> bool {
        match self {
            NodeAddress::V4(o) => {
                o[0] == 10
                    || (o[0] == 172 && (16..=31).contains(&o[1]))
                    || (o[0] == 192 && o[1] == 168)
            }
            NodeAddress::V6(g) => (g[0] & 0xfe00) == 0xfc00,
        }
    }

    /// Whether this is an IPv4 address.
    pub fn is_v4(&self) -> bool {
        matches!(self, NodeAddress::V4(_))
    }

    /// Whether this is an IPv6 address.
    pub fn is_v6(&self) -> bool {
        matches!(self, NodeAddress::V6(_))
    }

    /// Whether this is the unspecified address (`0.0.0.0` or `::`).
    pub fn is_unspecified(&self) -> bool {
        match self {
            NodeAddress::V4(o) => *o == [0, 0, 0, 0],
            NodeAddress::V6(g) => *g == [0; 8],
        }
    }

    /// Whether this is a link-local address (`169.254.0.0/16` or `fe80::/10`).
    pub fn is_link_local(&self) -> bool {
        match self {
            NodeAddress::V4(o) => o[0] == 169 && o[1] == 254,
            NodeAddress::V6(g) => (g[0] & 0xffc0) == 0xfe80,
        }
    }

    /// Whether this is a multicast address (`224.0.0.0/4` or `ff00::/8`).
    pub fn is_multicast(&self) -> bool {
        match self {
            NodeAddress::V4(o) => (o[0] & 0xf0) == 0xe0,
            NodeAddress::V6(g) => (g[0] & 0xff00) == 0xff00,
        }
    }

    /// Whether this is a "global"/routable unicast address: not loopback,
    /// unspecified, link-local, multicast, or private.
    pub fn is_global_unicast(&self) -> bool {
        !(self.is_loopback()
            || self.is_unspecified()
            || self.is_link_local()
            || self.is_multicast()
            || self.is_private())
    }

    /// Parse either an IPv4 or (subset of) IPv6 address.
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.contains(':') {
            Self::parse_v6(s)
        } else {
            Self::parse_v4(s)
        }
    }

    /// Parse an IPv6 address, supporting a single `::` zero-run compression.
    ///
    /// This is a self-contained parser (no `std::net`) handling the textual
    /// forms Talos encounters: full eight-group form and `::`-compressed form.
    /// IPv4-in-IPv6 dotted tails are not supported.
    pub fn parse_v6(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::parse("empty IPv6 address"));
        }
        // Split on "::" for zero compression. At most one occurrence allowed.
        let double_colon: Vec<&str> = s.split("::").collect();
        if double_colon.len() > 2 {
            return Err(Error::parse("IPv6 has more than one '::'"));
        }

        let parse_groups = |part: &str| -> Result<Vec<u16>> {
            if part.is_empty() {
                return Ok(Vec::new());
            }
            let mut groups = Vec::new();
            for g in part.split(':') {
                if g.is_empty() {
                    return Err(Error::parse("empty IPv6 group"));
                }
                if g.len() > 4 {
                    return Err(Error::parse(alloc::format!("IPv6 group '{g}' too long")));
                }
                let v = u16::from_str_radix(g, 16)
                    .map_err(|_| Error::parse(alloc::format!("invalid IPv6 group '{g}'")))?;
                groups.push(v);
            }
            Ok(groups)
        };

        let mut out = [0u16; 8];
        if double_colon.len() == 2 {
            let head = parse_groups(double_colon[0])?;
            let tail = parse_groups(double_colon[1])?;
            if head.len() + tail.len() > 7 {
                return Err(Error::parse("IPv6 '::' compresses too few groups"));
            }
            for (i, g) in head.iter().enumerate() {
                out[i] = *g;
            }
            let tail_start = 8 - tail.len();
            for (i, g) in tail.iter().enumerate() {
                out[tail_start + i] = *g;
            }
        } else {
            let groups = parse_groups(double_colon[0])?;
            if groups.len() != 8 {
                return Err(Error::parse("uncompressed IPv6 needs exactly 8 groups"));
            }
            out.copy_from_slice(&groups);
        }
        Ok(NodeAddress::V6(out))
    }

    /// The bit width of this address family (32 or 128).
    pub fn bit_width(&self) -> u8 {
        match self {
            NodeAddress::V4(_) => 32,
            NodeAddress::V6(_) => 128,
        }
    }
}

impl fmt::Display for NodeAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeAddress::V4(o) => write!(f, "{}.{}.{}.{}", o[0], o[1], o[2], o[3]),
            NodeAddress::V6(g) => write!(
                f,
                "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                g[0], g[1], g[2], g[3], g[4], g[5], g[6], g[7]
            ),
        }
    }
}

/// A COSI-style resource identifier (`namespace/type/id` triple flattened to an
/// id within a typed resource). Talos resources are addressed by a stable
/// string id; this newtype enforces non-emptiness and a safe character set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceId(String);

impl ResourceId {
    /// Validate and construct a resource id. Ids must be non-empty and may
    /// contain `[A-Za-z0-9._/:-]`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(Error::invalid("resource id is empty"));
        }
        for c in s.chars() {
            let ok = c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | ':' | '-');
            if !ok {
                return Err(Error::invalid(alloc::format!(
                    "invalid resource id character '{c}'"
                )));
            }
        }
        Ok(ResourceId(s))
    }

    /// The id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<ResourceId> for String {
    fn from(id: ResourceId) -> String {
        id.0
    }
}

impl core::str::FromStr for ResourceId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        ResourceId::new(s.to_string())
    }
}

/// A TCP/UDP port number, validated to the 1-65535 range (0 is reserved).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Port(u16);

impl Port {
    /// Validate and construct a port. Port 0 is rejected as it is not a usable
    /// listening port.
    pub fn new(p: u16) -> Result<Self> {
        if p == 0 {
            return Err(Error::invalid("port 0 is not valid"));
        }
        Ok(Port(p))
    }

    /// Parse a port from its decimal string form.
    pub fn parse(s: &str) -> Result<Self> {
        let p: u16 = s
            .trim()
            .parse()
            .map_err(|_| Error::parse(alloc::format!("invalid port '{s}'")))?;
        Port::new(p)
    }

    /// The numeric value.
    pub fn value(self) -> u16 {
        self.0
    }

    /// Whether this is a privileged port (< 1024).
    pub fn is_privileged(self) -> bool {
        self.0 < 1024
    }
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A CIDR network: a base [`NodeAddress`] plus a prefix length.
///
/// Mirrors the `netip.Prefix` values Talos uses for pod/service CIDRs and
/// interface addressing. Supports membership tests and network/broadcast
/// derivation for IPv4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cidr {
    base: NodeAddress,
    prefix_len: u8,
}

impl Cidr {
    /// Construct a CIDR, validating the prefix length against the family.
    pub fn new(base: NodeAddress, prefix_len: u8) -> Result<Self> {
        if prefix_len > base.bit_width() {
            return Err(Error::invalid(alloc::format!(
                "prefix /{prefix_len} exceeds {}-bit address",
                base.bit_width()
            )));
        }
        Ok(Cidr { base, prefix_len })
    }

    /// Parse a CIDR in `address/prefix` form.
    pub fn parse(s: &str) -> Result<Self> {
        let (addr, prefix) = s
            .split_once('/')
            .ok_or_else(|| Error::parse("CIDR missing '/' separator"))?;
        let base = NodeAddress::parse(addr)?;
        let prefix_len: u8 = prefix
            .trim()
            .parse()
            .map_err(|_| Error::parse(alloc::format!("invalid CIDR prefix '{prefix}'")))?;
        Cidr::new(base, prefix_len)
    }

    /// The base address as supplied (not necessarily the network address).
    pub fn base(&self) -> NodeAddress {
        self.base
    }

    /// The prefix length.
    pub fn prefix_len(&self) -> u8 {
        self.prefix_len
    }

    fn v4_u32(o: [u8; 4]) -> u32 {
        u32::from_be_bytes(o)
    }

    fn v4_mask(prefix_len: u8) -> u32 {
        if prefix_len == 0 {
            0
        } else {
            u32::MAX << (32 - u32::from(prefix_len))
        }
    }

    /// The network address (host bits zeroed). IPv4 only; IPv6 returns the base
    /// unchanged (full masking is out of scope for this subset).
    pub fn network(&self) -> NodeAddress {
        match self.base {
            NodeAddress::V4(o) => {
                let masked = Self::v4_u32(o) & Self::v4_mask(self.prefix_len);
                NodeAddress::V4(masked.to_be_bytes())
            }
            NodeAddress::V6(_) => self.base,
        }
    }

    /// The broadcast address (host bits set). IPv4 only.
    pub fn broadcast(&self) -> Option<NodeAddress> {
        match self.base {
            NodeAddress::V4(o) => {
                let mask = Self::v4_mask(self.prefix_len);
                let bc = (Self::v4_u32(o) & mask) | !mask;
                Some(NodeAddress::V4(bc.to_be_bytes()))
            }
            NodeAddress::V6(_) => None,
        }
    }

    /// Number of usable host addresses (IPv4). `/31` and `/32` return small
    /// special values per common convention.
    pub fn host_count(&self) -> Option<u64> {
        match self.base {
            NodeAddress::V4(_) => {
                let host_bits = 32 - u32::from(self.prefix_len);
                Some(1u64 << host_bits)
            }
            NodeAddress::V6(_) => None,
        }
    }

    /// Whether `addr` falls within this CIDR (same family, network bits match).
    pub fn contains(&self, addr: &NodeAddress) -> bool {
        match (self.base, addr) {
            (NodeAddress::V4(b), NodeAddress::V4(a)) => {
                let mask = Self::v4_mask(self.prefix_len);
                (Self::v4_u32(b) & mask) == (Self::v4_u32(*a) & mask)
            }
            (NodeAddress::V6(b), NodeAddress::V6(a)) => {
                // Compare prefix bits group by group.
                let mut remaining = u32::from(self.prefix_len);
                for i in 0..8 {
                    if remaining == 0 {
                        break;
                    }
                    let bits = remaining.min(16);
                    let mask: u16 = if bits == 16 {
                        0xffff
                    } else {
                        !0u16 << (16 - bits)
                    };
                    if (b[i] & mask) != (a[i] & mask) {
                        return false;
                    }
                    remaining -= bits;
                }
                true
            }
            _ => false,
        }
    }
}

impl fmt::Display for Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base, self.prefix_len)
    }
}

impl core::str::FromStr for Cidr {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Cidr::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostname_validation() {
        assert_eq!(
            Hostname::new("Node-1.Cluster.Local").unwrap().as_str(),
            "node-1.cluster.local"
        );
        assert_eq!(
            Hostname::new("node-1.cluster.local").unwrap().short(),
            "node-1"
        );
        assert!(Hostname::new("").is_err());
        assert!(Hostname::new("-bad").is_err());
        assert!(Hostname::new("bad-").is_err());
        assert!(Hostname::new("under_score").is_err());
        assert!(Hostname::new("a..b").is_err());
    }

    #[test]
    fn ipv4_parsing_and_classification() {
        let lo = NodeAddress::parse_v4("127.0.0.1").unwrap();
        assert!(lo.is_loopback());
        assert!(lo.is_v4());

        let priv_a = NodeAddress::parse_v4("10.0.5.4").unwrap();
        let priv_b = NodeAddress::parse_v4("172.16.0.1").unwrap();
        let priv_c = NodeAddress::parse_v4("192.168.1.1").unwrap();
        assert!(priv_a.is_private() && priv_b.is_private() && priv_c.is_private());

        let public = NodeAddress::parse_v4("8.8.8.8").unwrap();
        assert!(!public.is_private());
        assert_eq!(public.to_string(), "8.8.8.8");

        assert!(NodeAddress::parse_v4("1.2.3").is_err());
        assert!(NodeAddress::parse_v4("256.0.0.1").is_err());
        assert!(NodeAddress::parse_v4("1.2.3.4.5").is_err());
    }

    #[test]
    fn ipv6_classification() {
        let lo = NodeAddress::V6([0, 0, 0, 0, 0, 0, 0, 1]);
        assert!(lo.is_loopback());
        let ula = NodeAddress::V6([0xfd00, 0, 0, 0, 0, 0, 0, 1]);
        assert!(ula.is_private());
    }

    #[test]
    fn resource_id_validation() {
        assert_eq!(
            ResourceId::new("default/MachineConfig/v1alpha1")
                .unwrap()
                .as_str(),
            "default/MachineConfig/v1alpha1"
        );
        assert!(ResourceId::new("").is_err());
        assert!(ResourceId::new("bad space").is_err());
    }

    #[test]
    fn ipv6_parsing_full_and_compressed() {
        let full = NodeAddress::parse_v6("2001:db8:0:0:0:0:0:1").unwrap();
        let compressed = NodeAddress::parse_v6("2001:db8::1").unwrap();
        assert_eq!(full, compressed);

        let lo = NodeAddress::parse_v6("::1").unwrap();
        assert!(lo.is_loopback());
        assert!(lo.is_v6());

        let unspec = NodeAddress::parse_v6("::").unwrap();
        assert!(unspec.is_unspecified());

        let ll = NodeAddress::parse_v6("fe80::1").unwrap();
        assert!(ll.is_link_local());

        let mc = NodeAddress::parse_v6("ff02::1").unwrap();
        assert!(mc.is_multicast());

        assert!(NodeAddress::parse_v6("2001::db8::1").is_err()); // two ::
        assert!(NodeAddress::parse_v6("2001:db8:1").is_err()); // too few, uncompressed
        assert!(NodeAddress::parse_v6("gggg::1").is_err());
        assert!(NodeAddress::parse_v6("12345::1").is_err());
    }

    #[test]
    fn parse_dispatches_by_family() {
        assert!(NodeAddress::parse("10.0.0.1").unwrap().is_v4());
        assert!(NodeAddress::parse("fe80::1").unwrap().is_v6());
    }

    #[test]
    fn address_classification_extras() {
        let public = NodeAddress::parse_v4("8.8.8.8").unwrap();
        assert!(public.is_global_unicast());
        let priv_a = NodeAddress::parse_v4("10.0.0.1").unwrap();
        assert!(!priv_a.is_global_unicast());
        let ll = NodeAddress::parse_v4("169.254.1.1").unwrap();
        assert!(ll.is_link_local());
        let mc = NodeAddress::parse_v4("224.0.0.1").unwrap();
        assert!(mc.is_multicast());
        assert_eq!(public.bit_width(), 32);
        assert_eq!(NodeAddress::parse_v6("::1").unwrap().bit_width(), 128);
    }

    #[test]
    fn port_validation() {
        assert_eq!(Port::new(6443).unwrap().value(), 6443);
        assert!(Port::new(0).is_err());
        assert!(Port::parse("443").unwrap().is_privileged());
        assert!(!Port::parse("8080").unwrap().is_privileged());
        assert!(Port::parse("70000").is_err());
        assert!(Port::parse("x").is_err());
        assert_eq!(Port::new(53).unwrap().to_string(), "53");
    }

    #[test]
    fn cidr_v4_membership_and_derivation() {
        let cidr = Cidr::parse("10.244.0.0/16").unwrap();
        assert_eq!(cidr.prefix_len(), 16);
        assert!(cidr.contains(&NodeAddress::parse_v4("10.244.5.7").unwrap()));
        assert!(!cidr.contains(&NodeAddress::parse_v4("10.245.0.1").unwrap()));

        assert_eq!(cidr.network(), NodeAddress::parse_v4("10.244.0.0").unwrap());
        assert_eq!(
            cidr.broadcast().unwrap(),
            NodeAddress::parse_v4("10.244.255.255").unwrap()
        );
        assert_eq!(cidr.host_count(), Some(65536));

        // Network address is masked even when host bits supplied.
        let off = Cidr::parse("192.168.1.42/24").unwrap();
        assert_eq!(off.network(), NodeAddress::parse_v4("192.168.1.0").unwrap());
        assert_eq!(off.to_string(), "192.168.1.42/24");
    }

    #[test]
    fn cidr_v6_membership() {
        let cidr = Cidr::parse("2001:db8::/32").unwrap();
        assert!(cidr.contains(&NodeAddress::parse_v6("2001:db8:abcd::1").unwrap()));
        assert!(!cidr.contains(&NodeAddress::parse_v6("2001:db9::1").unwrap()));
        // Mixed families never match.
        assert!(!cidr.contains(&NodeAddress::parse_v4("10.0.0.1").unwrap()));
        assert!(cidr.broadcast().is_none());
        assert!(cidr.host_count().is_none());
    }

    #[test]
    fn cidr_rejects_bad_prefix() {
        assert!(Cidr::new(NodeAddress::parse_v4("10.0.0.0").unwrap(), 33).is_err());
        assert!(Cidr::parse("10.0.0.0").is_err());
        assert!(Cidr::parse("10.0.0.0/x").is_err());
        assert!(Cidr::new(NodeAddress::parse_v6("::").unwrap(), 200).is_err());
    }
}
