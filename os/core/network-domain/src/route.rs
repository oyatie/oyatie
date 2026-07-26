//! Route specs and route merging.
//!
//! Mirrors `network.RouteSpec`, `RouteConfigController` and
//! `RouteMergeController`. A [`RouteSpec`] is a single kernel routing-table
//! entry (destination/source prefix, gateway, output link, metric, table, MTU).
//! The merge logic folds specs from multiple layers by priority, keyed by the
//! route's logical identity, so the highest-precedence layer wins per route.

use crate::config_layer::ConfigLayer;
use crate::link::AddressFamily;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// The routing table a route belongs to, mirroring the kernel `rt_table`
/// values Talos uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteTable {
    /// The main routing table (`254`).
    Main,
    /// The local table (`255`), populated automatically by the kernel.
    Local,
    /// A custom numbered table (e.g. for policy routing).
    Custom(u32),
}

impl RouteTable {
    /// The kernel numeric table id.
    pub fn table_id(self) -> u32 {
        match self {
            RouteTable::Main => 254,
            RouteTable::Local => 255,
            RouteTable::Custom(id) => id,
        }
    }
}

/// The protocol that installed a route, mirroring the kernel `rt_protocol`
/// values (`RTPROT_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteProtocol {
    /// Statically configured by the operator/admin (`RTPROT_STATIC`).
    Static,
    /// Installed by a routing daemon / boot process (`RTPROT_BOOT`).
    Boot,
    /// Installed by the kernel itself (`RTPROT_KERNEL`), e.g. on-link routes.
    Kernel,
    /// Installed by a DHCP client (`RTPROT_DHCP`).
    Dhcp,
}

impl RouteProtocol {
    /// The kernel numeric protocol value.
    pub fn protocol_id(self) -> u8 {
        match self {
            RouteProtocol::Boot => 3,
            RouteProtocol::Static => 4,
            RouteProtocol::Kernel => 2,
            RouteProtocol::Dhcp => 16,
        }
    }
}

/// A single routing-table entry.
///
/// Equivalent to `network.RouteSpecSpec`. A destination with `prefix_len == 0`
/// and no explicit destination address is the default route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteSpec {
    /// Destination prefix; `None` together with `prefix_len == 0` means the
    /// default route (`0.0.0.0/0` or `::/0`).
    pub destination: Option<NodeAddress>,
    /// Destination prefix length.
    pub prefix_len: u8,
    /// Preferred source address; `None` means the kernel may select it.
    pub source: Option<NodeAddress>,
    /// Next-hop gateway; `None` for an on-link (directly connected) route.
    pub gateway: Option<NodeAddress>,
    /// Output link name (e.g. `eth0`).
    pub out_link: String,
    /// Address family.
    pub family: AddressFamily,
    /// Route metric / priority (lower wins in the kernel).
    pub metric: u32,
    /// Per-route MTU; `0` means unspecified.
    pub mtu: u32,
    /// Routing table.
    pub table: RouteTable,
    /// Installing protocol.
    pub protocol: RouteProtocol,
    /// Provenance / priority of this spec.
    pub layer: ConfigLayer,
}

impl RouteSpec {
    /// Build a default route via a gateway out of a link.
    pub fn default_via(
        gateway: NodeAddress,
        out_link: impl Into<String>,
        layer: ConfigLayer,
    ) -> Result<Self> {
        let family = if gateway.is_v4() {
            AddressFamily::Inet4
        } else {
            AddressFamily::Inet6
        };
        let spec = RouteSpec {
            destination: None,
            prefix_len: 0,
            source: None,
            gateway: Some(gateway),
            out_link: out_link.into(),
            family,
            metric: 1024,
            mtu: 0,
            table: RouteTable::Main,
            protocol: RouteProtocol::Static,
            layer,
        };
        spec.validate()?;
        Ok(spec)
    }

    /// Whether this is the default route (`0.0.0.0/0` / `::/0`).
    pub fn is_default(&self) -> bool {
        self.prefix_len == 0 && self.destination.is_none()
    }

    /// Validate the route's invariants.
    pub fn validate(&self) -> Result<()> {
        let max = self.family.max_prefix_len();
        if self.prefix_len > max {
            return Err(Error::invalid(alloc::format!(
                "prefix length {} exceeds maximum {} for {:?}",
                self.prefix_len,
                max,
                self.family
            )));
        }
        if self.out_link.is_empty() {
            return Err(Error::invalid("route spec has empty out link"));
        }
        // family/address consistency for the destination and gateway
        if let Some(dest) = self.destination {
            self.check_family(dest, "destination")?;
        }
        if let Some(source) = self.source {
            self.check_family(source, "source")?;
        }
        if let Some(gw) = self.gateway {
            self.check_family(gw, "gateway")?;
        }
        // a non-default route must have an explicit destination
        if !self.is_default() && self.destination.is_none() && self.prefix_len != 0 {
            return Err(Error::invalid("non-default route has no destination"));
        }
        Ok(())
    }

    fn check_family(&self, addr: NodeAddress, what: &str) -> Result<()> {
        let ok = match self.family {
            AddressFamily::Inet4 => addr.is_v4(),
            AddressFamily::Inet6 => !addr.is_v4(),
        };
        if ok {
            Ok(())
        } else {
            Err(Error::invalid(alloc::format!(
                "{} address family does not match route family {:?}",
                what,
                self.family
            )))
        }
    }

    fn family_id(&self) -> &'static str {
        match self.family {
            AddressFamily::Inet4 => "inet4",
            AddressFamily::Inet6 => "inet6",
        }
    }

    /// A stable COSI id encoding the route's logical identity.
    ///
    /// This mirrors Talos' `network.RouteID`: non-main tables prefix the ID,
    /// IPv6 routes additionally prefix the output link, then the route is keyed
    /// as `family/gateway/destination/priority`. The destination is rendered as
    /// an empty component for the normalized default route. Talos RouteID
    /// deliberately excludes source, MTU, protocol, layer, and IPv4 output
    /// link, so changing those fields does not produce a distinct route.
    pub fn id(&self) -> String {
        let mut id = String::new();
        if let Some(table) = route_table_id(self.table) {
            id.push_str(&table);
            id.push('/');
        }
        if self.family == AddressFamily::Inet6 {
            id.push_str(&self.out_link);
            id.push('/');
        }
        id.push_str(self.family_id());
        id.push('/');
        id.push_str(&route_gateway_id(self.gateway));
        id.push('/');
        id.push_str(&route_destination_id(self.destination, self.prefix_len));
        id.push('/');
        id.push_str(&self.metric.to_string());
        id
    }
}

fn route_table_id(table: RouteTable) -> Option<String> {
    match table {
        RouteTable::Main => None,
        RouteTable::Local => Some("local".to_string()),
        RouteTable::Custom(253) => Some("default".to_string()),
        RouteTable::Custom(254) => None,
        RouteTable::Custom(255) => Some("local".to_string()),
        RouteTable::Custom(id) => Some(id.to_string()),
    }
}

fn route_destination_id(destination: Option<NodeAddress>, prefix_len: u8) -> String {
    match destination {
        Some(address) => alloc::format!("{}/{}", route_address_id(address), prefix_len),
        None => String::new(),
    }
}

fn route_gateway_id(gateway: Option<NodeAddress>) -> String {
    gateway.map(route_address_id).unwrap_or_default()
}

fn route_address_id(address: NodeAddress) -> String {
    match address {
        NodeAddress::V4(octets) => {
            alloc::format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
        }
        NodeAddress::V6(groups) => route_ipv6_id(groups),
    }
}

fn route_ipv6_id(groups: [u16; 8]) -> String {
    let mut best_start = 8usize;
    let mut best_len = 0usize;
    let mut i = 0usize;
    while i < groups.len() {
        if groups[i] != 0 {
            i += 1;
            continue;
        }
        let start = i;
        while i < groups.len() && groups[i] == 0 {
            i += 1;
        }
        let len = i - start;
        if len >= 2 && len > best_len {
            best_start = start;
            best_len = len;
        }
    }

    let mut out = String::new();
    let mut i = 0usize;
    while i < groups.len() {
        if i == best_start {
            out.push_str("::");
            i += best_len;
            if i >= groups.len() {
                break;
            }
            continue;
        }
        if !out.is_empty() && !out.ends_with(':') {
            out.push(':');
        }
        out.push_str(&alloc::format!("{:x}", groups[i]));
        i += 1;
    }

    if out.is_empty() {
        "::".to_string()
    } else {
        out
    }
}

/// In-memory merge of route specs by COSI id, applying layer priority.
///
/// Equivalent to `RouteMergeController`: for each logical route id the
/// highest-priority layer wins. Returns the merged specs sorted by id.
pub fn merge_routes(specs: &[RouteSpec]) -> Vec<RouteSpec> {
    let mut by_id: BTreeMap<String, RouteSpec> = BTreeMap::new();
    for spec in specs {
        let id = spec.id();
        match by_id.get(&id) {
            Some(existing) if existing.layer.precedence() >= spec.layer.precedence() => {}
            _ => {
                by_id.insert(id, spec.clone());
            }
        }
    }
    by_id.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(s: &str) -> NodeAddress {
        NodeAddress::parse_v4(s).unwrap()
    }

    fn v6(s: &str) -> NodeAddress {
        NodeAddress::parse_v6(s).unwrap()
    }

    #[test]
    fn default_route_construction() {
        let r = RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Platform).unwrap();
        assert!(r.is_default());
        assert_eq!(r.family, AddressFamily::Inet4);
        assert_eq!(r.source, None);
        assert_eq!(r.mtu, 0);
        assert_eq!(r.table, RouteTable::Main);
        assert_eq!(r.table.table_id(), 254);
        assert_eq!(r.id(), "inet4/10.0.0.1//1024");
        assert_eq!(r.protocol.protocol_id(), 4);
    }

    #[test]
    fn route_validation_rejects_bad_prefix_and_link() {
        let mut r = RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Default).unwrap();
        r.destination = Some(v4("192.168.0.0"));
        r.prefix_len = 40;
        assert!(r.validate().is_err());

        let mut empty =
            RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Default).unwrap();
        empty.out_link = String::new();
        assert!(empty.validate().is_err());
    }

    #[test]
    fn route_family_mismatch_is_rejected() {
        let mut r = RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Default).unwrap();
        r.family = AddressFamily::Inet6;
        // gateway is v4 but family says v6
        assert!(r.validate().is_err());
    }

    #[test]
    fn route_source_family_mismatch_is_rejected() {
        let mut r = RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Default).unwrap();
        r.source = Some(v6("2001:db8::10"));

        assert!(r.validate().is_err());

        r.source = Some(v4("10.0.0.10"));
        assert!(r.validate().is_ok());
    }

    #[test]
    fn specific_route_has_source_compatible_destination_in_id() {
        let r = RouteSpec {
            destination: Some(v4("172.16.0.0")),
            prefix_len: 16,
            source: Some(v4("172.16.0.10")),
            gateway: Some(v4("10.0.0.1")),
            out_link: String::from("eth0"),
            family: AddressFamily::Inet4,
            metric: 100,
            mtu: 1400,
            table: RouteTable::Custom(42),
            protocol: RouteProtocol::Dhcp,
            layer: ConfigLayer::Operator,
        };
        assert!(r.validate().is_ok());
        assert!(!r.is_default());
        assert_eq!(r.id(), "42/inet4/10.0.0.1/172.16.0.0/16/100");
    }

    #[test]
    fn route_id_matches_talos_route_id_vectors() {
        let default = RouteSpec::default_via(v4("10.5.0.3"), "eth0", ConfigLayer::Cmdline).unwrap();
        assert_eq!(default.id(), "inet4/10.5.0.3//1024");

        let direct = RouteSpec {
            destination: Some(v4("169.254.0.1")),
            prefix_len: 32,
            source: None,
            gateway: None,
            out_link: String::from("eth0"),
            family: AddressFamily::Inet4,
            metric: 1024,
            mtu: 0,
            table: RouteTable::Main,
            protocol: RouteProtocol::Boot,
            layer: ConfigLayer::Operator,
        };
        assert_eq!(direct.id(), "inet4//169.254.0.1/32/1024");

        let v6_default =
            RouteSpec::default_via(v6("2001:db8::1"), "eth0", ConfigLayer::Operator).unwrap();
        assert_eq!(v6_default.id(), "eth0/inet6/2001:db8::1//1024");

        let local = RouteSpec {
            table: RouteTable::Local,
            ..direct.clone()
        };
        assert_eq!(local.id(), "local/inet4//169.254.0.1/32/1024");
    }

    #[test]
    fn route_id_excludes_source_and_mtu() {
        let mut base =
            RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Cmdline).unwrap();
        let mut changed = base.clone();
        changed.source = Some(v4("10.0.0.10"));
        changed.mtu = 1400;
        changed.protocol = RouteProtocol::Dhcp;
        changed.layer = ConfigLayer::Operator;
        changed.out_link = "eth1".to_string();

        assert_eq!(base.id(), changed.id());

        base.metric = 100;
        assert_ne!(base.id(), changed.id());
    }

    #[test]
    fn merge_prefers_higher_layer() {
        let low = RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Cmdline).unwrap();
        let mut high =
            RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Configuration).unwrap();
        high.source = Some(v4("10.0.0.10"));
        high.mtu = 1400;
        let other = RouteSpec::default_via(v4("10.0.0.1"), "eth1", ConfigLayer::Platform).unwrap();

        let merged = merge_routes(&[low, high.clone(), other]);
        // Talos RouteID excludes IPv4 out-link names, so all three specs share
        // the same route identity (default via 10.0.0.1 at priority 1024).
        assert_eq!(merged.len(), 1);
        let winner: Vec<_> = merged.iter().filter(|r| r.id() == high.id()).collect();
        assert_eq!(winner.len(), 1);
        assert_eq!(winner[0].layer, ConfigLayer::Configuration);
        assert_eq!(winner[0].source, Some(v4("10.0.0.10")));
        assert_eq!(winner[0].mtu, 1400);
    }

    #[test]
    fn merge_preserves_distinct_gateways() {
        let left = RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Cmdline).unwrap();
        let right =
            RouteSpec::default_via(v4("10.0.0.254"), "eth0", ConfigLayer::Configuration).unwrap();

        let merged = merge_routes(&[left, right]);

        assert_eq!(merged.len(), 2);
        assert!(
            merged
                .iter()
                .any(|r| r.gateway.map(|g| g.to_string()) == Some("10.0.0.1".to_string()))
        );
        assert!(
            merged
                .iter()
                .any(|r| r.gateway.map(|g| g.to_string()) == Some("10.0.0.254".to_string()))
        );
    }
}
