//! `LinkConfig` / `VLANConfig` — static network link configuration documents.
//!
//! Mirrors the Talos v1.12 network documents for configuring physical links and
//! VLAN links: `name`, optional administrative `up`, optional `mtu`, static
//! addresses, optional `multicast`, and static routes. `VLANConfig` adds
//! `vlanID`, `parent`, and optional `vlanMode`.

use crate::document::{ConfigDocument, DocId, DocKind};
use os_kernel::error::{Error, Result};

/// A statically assigned address on a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressConfig {
    /// Address prefix in CIDR form, e.g. `192.168.1.100/24` or `fd00::1/64`.
    pub address: String,
    /// Optional route priority for routes created for this address.
    pub route_priority: Option<u32>,
}

impl AddressConfig {
    /// Build an address config.
    pub fn new(address: impl Into<String>) -> Self {
        AddressConfig {
            address: address.into(),
            route_priority: None,
        }
    }

    /// Builder: route priority.
    #[must_use]
    pub fn with_route_priority(mut self, route_priority: u32) -> Self {
        self.route_priority = Some(route_priority);
        self
    }
}

/// A static route attached to a link.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteConfig {
    /// Destination prefix. Empty means default route when a gateway is present.
    pub destination: String,
    /// Next-hop gateway. Empty means a link-scope route.
    pub gateway: String,
    /// Optional source address.
    pub source: String,
    /// Optional route metric.
    pub metric: Option<u32>,
    /// Optional route MTU.
    pub mtu: Option<u32>,
    /// Optional routing table.
    pub table: Option<u32>,
}

impl RouteConfig {
    /// Build a route with a destination and optional gateway.
    pub fn new(destination: impl Into<String>, gateway: impl Into<String>) -> Self {
        RouteConfig {
            destination: destination.into(),
            gateway: gateway.into(),
            ..Default::default()
        }
    }
}

/// VLAN tagging protocol for `VLANConfig`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VlanMode {
    /// 802.1Q VLAN tagging.
    Dot1Q,
    /// 802.1ad QinQ/service VLAN tagging.
    Dot1Ad,
}

impl VlanMode {
    /// Canonical config string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dot1Q => "802.1q",
            Self::Dot1Ad => "802.1ad",
        }
    }

    /// Parse a config string.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "802.1q" => Self::Dot1Q,
            "802.1ad" => Self::Dot1Ad,
            _ => return None,
        })
    }
}

/// Shared link fields embedded by `LinkConfig` and `VLANConfig`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkFields {
    /// Optional administrative state. Talos defaults this to up when omitted.
    pub up: Option<bool>,
    /// Optional link MTU. `None` delegates to the system default.
    pub mtu: Option<u32>,
    /// Optional multicast flag.
    pub multicast: Option<bool>,
    /// Static addresses assigned to the link.
    pub addresses: Vec<AddressConfig>,
    /// Static routes created via the link.
    pub routes: Vec<RouteConfig>,
}

impl LinkFields {
    fn validate(&self, doc: &str) -> Result<()> {
        if let Some(mtu) = self.mtu {
            validate_mtu(mtu, doc)?;
        }
        for address in &self.addresses {
            validate_cidr(&address.address, &format!("{doc}: addresses[].address"))?;
        }
        for route in &self.routes {
            if route.destination.trim().is_empty() && route.gateway.trim().is_empty() {
                return Err(Error::invalid(format!(
                    "{doc}: routes[] must set destination or gateway"
                )));
            }
            if !route.destination.trim().is_empty() {
                validate_cidr(&route.destination, &format!("{doc}: routes[].destination"))?;
            }
        }
        Ok(())
    }
}

/// The `LinkConfig` document for a physical link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkConfig {
    /// Link/interface name (document key).
    pub name: String,
    /// Shared static link settings.
    pub link: LinkFields,
}

impl LinkConfig {
    /// Construct a minimal `LinkConfig` for `name`.
    pub fn new(name: impl Into<String>) -> Self {
        LinkConfig {
            name: name.into(),
            link: LinkFields::default(),
        }
    }

    /// Builder: administrative state.
    #[must_use]
    pub fn with_up(mut self, up: bool) -> Self {
        self.link.up = Some(up);
        self
    }

    /// Builder: MTU.
    #[must_use]
    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.link.mtu = Some(mtu);
        self
    }

    /// Builder: multicast setting.
    #[must_use]
    pub fn with_multicast(mut self, multicast: bool) -> Self {
        self.link.multicast = Some(multicast);
        self
    }

    /// Builder: add a static address.
    #[must_use]
    pub fn with_address(mut self, address: AddressConfig) -> Self {
        self.link.addresses.push(address);
        self
    }

    /// Builder: add a static route.
    #[must_use]
    pub fn with_route(mut self, route: RouteConfig) -> Self {
        self.link.routes.push(route);
        self
    }
}

impl ConfigDocument for LinkConfig {
    fn kind(&self) -> DocKind {
        DocKind::Link
    }

    fn id(&self) -> DocId {
        DocId::keyed(DocKind::Link, self.name.clone())
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("LinkConfig: name is required"));
        }
        self.link.validate("LinkConfig")
    }
}

/// The `VLANConfig` document for a VLAN link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VlanConfig {
    /// VLAN link name (document key).
    pub name: String,
    /// VLAN id (1..=4094).
    pub vlan_id: u16,
    /// Parent link name.
    pub parent: String,
    /// Optional VLAN tagging protocol; Talos defaults to 802.1q.
    pub vlan_mode: Option<VlanMode>,
    /// Shared static link settings.
    pub link: LinkFields,
}

impl VlanConfig {
    /// Construct a minimal `VLANConfig`.
    pub fn new(name: impl Into<String>, vlan_id: u16, parent: impl Into<String>) -> Self {
        VlanConfig {
            name: name.into(),
            vlan_id,
            parent: parent.into(),
            vlan_mode: None,
            link: LinkFields::default(),
        }
    }

    /// Builder: VLAN mode.
    #[must_use]
    pub fn with_vlan_mode(mut self, vlan_mode: VlanMode) -> Self {
        self.vlan_mode = Some(vlan_mode);
        self
    }

    /// Builder: administrative state.
    #[must_use]
    pub fn with_up(mut self, up: bool) -> Self {
        self.link.up = Some(up);
        self
    }

    /// Builder: MTU.
    #[must_use]
    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.link.mtu = Some(mtu);
        self
    }

    /// Builder: multicast setting.
    #[must_use]
    pub fn with_multicast(mut self, multicast: bool) -> Self {
        self.link.multicast = Some(multicast);
        self
    }

    /// Builder: add a static address.
    #[must_use]
    pub fn with_address(mut self, address: AddressConfig) -> Self {
        self.link.addresses.push(address);
        self
    }

    /// Builder: add a static route.
    #[must_use]
    pub fn with_route(mut self, route: RouteConfig) -> Self {
        self.link.routes.push(route);
        self
    }
}

impl ConfigDocument for VlanConfig {
    fn kind(&self) -> DocKind {
        DocKind::Vlan
    }

    fn id(&self) -> DocId {
        DocId::keyed(DocKind::Vlan, self.name.clone())
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("VLANConfig: name is required"));
        }
        if self.parent.trim().is_empty() {
            return Err(Error::invalid("VLANConfig: parent is required"));
        }
        if self.vlan_id == 0 || self.vlan_id > 4094 {
            return Err(Error::invalid(format!(
                "VLANConfig: vlanID {} out of range 1..=4094",
                self.vlan_id
            )));
        }
        self.link.validate("VLANConfig")
    }
}

fn validate_mtu(mtu: u32, doc: &str) -> Result<()> {
    if !(576..=65535).contains(&mtu) {
        return Err(Error::invalid(format!(
            "{doc}: mtu {mtu} out of range 576..=65535"
        )));
    }
    Ok(())
}

fn validate_cidr(cidr: &str, field: &str) -> Result<()> {
    let (addr, prefix) = cidr
        .trim()
        .split_once('/')
        .ok_or_else(|| Error::invalid(format!("{field} must be CIDR")))?;
    if addr.trim().is_empty() {
        return Err(Error::invalid(format!("{field} has empty address")));
    }
    let prefix = prefix
        .parse::<u8>()
        .map_err(|_| Error::invalid(format!("{field} has bad prefix")))?;
    let max = if addr.contains(':') { 128 } else { 32 };
    if prefix > max {
        return Err(Error::invalid(format!(
            "{field} prefix /{prefix} out of range for address family"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_config_validates_source_shaped_fields() {
        let doc = LinkConfig::new("enp0s2")
            .with_up(true)
            .with_mtu(9000)
            .with_multicast(true)
            .with_address(AddressConfig::new("192.168.1.100/24").with_route_priority(2048))
            .with_address(AddressConfig::new("fd00::1/64"))
            .with_route(RouteConfig::new("10.0.0.0/8", "10.0.0.1"));
        assert!(doc.validate().is_ok());
        assert_eq!(doc.link.multicast, Some(true));
        assert_eq!(doc.kind(), DocKind::Link);
        assert_eq!(doc.id(), DocId::keyed(DocKind::Link, "enp0s2"));
        assert!(doc.allows_multiple());
    }

    #[test]
    fn vlan_config_validates_vlan_specific_fields() {
        let doc = VlanConfig::new("enp0s3.34", 34, "enp0s3")
            .with_vlan_mode(VlanMode::Dot1Q)
            .with_up(true)
            .with_mtu(1500)
            .with_multicast(true)
            .with_address(AddressConfig::new("192.168.1.100/24"));
        assert!(doc.validate().is_ok());
        assert_eq!(doc.link.multicast, Some(true));
        assert_eq!(doc.kind(), DocKind::Vlan);
        assert_eq!(doc.id(), DocId::keyed(DocKind::Vlan, "enp0s3.34"));
        assert_eq!(VlanMode::parse("802.1ad"), Some(VlanMode::Dot1Ad));
    }

    #[test]
    fn link_config_rejects_empty_name_bad_mtu_and_bad_cidr() {
        assert!(LinkConfig::new("").validate().is_err());
        assert!(LinkConfig::new("eth0").with_mtu(1).validate().is_err());
        assert!(
            LinkConfig::new("eth0")
                .with_address(AddressConfig::new("192.168.1.1"))
                .validate()
                .is_err()
        );
    }

    #[test]
    fn vlan_config_rejects_bad_vlan_id_and_parent() {
        assert!(VlanConfig::new("eth0.0", 0, "eth0").validate().is_err());
        assert!(VlanConfig::new("eth0.10", 10, "").validate().is_err());
        assert_eq!(VlanMode::parse("qinq"), None);
    }
}
