//! Typed accessors for `LinkConfig` and `VLANConfig` multi-document configs.
//!
//! Talos v1.12 split static link configuration out of legacy
//! `machine.network.interfaces[]` into source-shaped documents. This module
//! decodes the bounded fields used by physical links and VLAN links: name,
//! up/MTU, multicast, static addresses, static routes, and VLAN-specific
//! parent/id/mode.

use crate::container::Config;
use crate::yaml::{self, Yaml};
use std::collections::BTreeSet;
use os_kernel::error::{Error, Result};

/// Canonical Talos document kind for physical links.
pub const LINK_CONFIG_KIND: &str = "LinkConfig";
/// Canonical Talos document kind for VLAN links.
pub const VLAN_CONFIG_KIND: &str = "VLANConfig";

/// A statically assigned address on a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressConfig {
    /// Address prefix in CIDR form.
    pub address: String,
    /// Optional route priority for routes created for this address.
    pub route_priority: u32,
}

impl AddressConfig {
    /// Build an address config.
    pub fn new(address: impl Into<String>) -> Self {
        AddressConfig {
            address: address.into(),
            route_priority: 0,
        }
    }
}

/// A static route attached to a link.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteConfig {
    /// Destination prefix. Empty means default route when a gateway is present.
    pub destination: String,
    /// Next-hop gateway. Empty means link-scope route.
    pub gateway: String,
    /// Optional source address.
    pub source: String,
    /// Optional route metric.
    pub metric: u32,
    /// Optional route MTU.
    pub mtu: u32,
    /// Optional routing table.
    pub table: u32,
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
    /// Parse a YAML string value.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "802.1q" => Ok(Self::Dot1Q),
            "802.1ad" => Ok(Self::Dot1Ad),
            other => Err(Error::parse(format!(
                "VLANConfig: unknown vlanMode '{other}'"
            ))),
        }
    }

    /// Canonical string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dot1Q => "802.1q",
            Self::Dot1Ad => "802.1ad",
        }
    }
}

/// Shared link fields embedded by `LinkConfig` and `VLANConfig`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkFields {
    /// Administrative state. `None` means Talos default (up).
    pub up: Option<bool>,
    /// Link MTU. `0` means Talos/system default.
    pub mtu: u32,
    /// Optional multicast flag.
    pub multicast: Option<bool>,
    /// Static addresses assigned to the link.
    pub addresses: Vec<AddressConfig>,
    /// Static routes created via the link.
    pub routes: Vec<RouteConfig>,
}

impl LinkFields {
    /// Effective administrative state after applying the Talos default.
    pub fn up_or_default(&self) -> bool {
        self.up.unwrap_or(true)
    }

    fn validate(&self, doc: &str) -> Result<()> {
        if self.mtu != 0 {
            validate_mtu(self.mtu, doc)?;
        }
        for address in &self.addresses {
            validate_cidr(&address.address, &format!("{doc}.addresses[].address"))?;
        }
        for route in &self.routes {
            if route.destination.trim().is_empty() && route.gateway.trim().is_empty() {
                return Err(Error::invalid(format!(
                    "{doc}.routes[] must set destination or gateway"
                )));
            }
            if !route.destination.trim().is_empty() {
                validate_cidr(&route.destination, &format!("{doc}.routes[].destination"))?;
            }
        }
        Ok(())
    }
}

/// Parsed `LinkConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkConfig {
    /// Link/interface name.
    pub name: String,
    /// Static link settings.
    pub link: LinkFields,
}

impl LinkConfig {
    /// Build a minimal link config.
    pub fn new(name: impl Into<String>) -> Self {
        LinkConfig {
            name: name.into(),
            link: LinkFields::default(),
        }
    }

    /// Validate the document in isolation.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("LinkConfig: name is required"));
        }
        self.link.validate("LinkConfig")
    }
}

/// Parsed `VLANConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VlanConfig {
    /// VLAN link name.
    pub name: String,
    /// VLAN ID.
    pub vlan_id: u16,
    /// Parent link name.
    pub parent: String,
    /// VLAN tagging protocol. `None` means Talos default (802.1q).
    pub vlan_mode: Option<VlanMode>,
    /// Static link settings.
    pub link: LinkFields,
}

impl VlanConfig {
    /// Effective VLAN mode after applying the Talos default.
    pub fn vlan_mode_or_default(&self) -> VlanMode {
        self.vlan_mode.unwrap_or(VlanMode::Dot1Q)
    }

    /// Validate the document in isolation.
    pub fn validate(&self) -> Result<()> {
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

/// Decode and validate one `LinkConfig` document body.
pub fn decode_link_config_body(body: &str) -> Result<LinkConfig> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != LINK_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "LinkConfig: unexpected kind '{kind}'"
        )));
    }

    let config = LinkConfig {
        name: required_string(&root, "name", "LinkConfig.name")?,
        link: decode_link_fields(&root, "LinkConfig")?,
    };
    config.validate()?;
    Ok(config)
}

/// Decode and validate one `VLANConfig` document body.
pub fn decode_vlan_config_body(body: &str) -> Result<VlanConfig> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != VLAN_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "VLANConfig: unexpected kind '{kind}'"
        )));
    }

    let vlan_mode = match root.get_str("vlanMode") {
        Some(value) if !value.trim().is_empty() => Some(VlanMode::parse(value)?),
        _ => None,
    };
    let config = VlanConfig {
        name: required_string(&root, "name", "VLANConfig.name")?,
        vlan_id: required_u16(root.get("vlanID"), "VLANConfig.vlanID")?,
        parent: required_string(&root, "parent", "VLANConfig.parent")?,
        vlan_mode,
        link: decode_link_fields(&root, "VLANConfig")?,
    };
    config.validate()?;
    Ok(config)
}

/// Extract all `LinkConfig` docs from a loaded config, rejecting duplicate names.
pub fn link_configs(config: &Config) -> Result<Vec<LinkConfig>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for doc in config
        .documents()
        .iter()
        .filter(|doc| doc.meta.kind == LINK_CONFIG_KIND)
    {
        let parsed = decode_link_config_body(&doc.body)?;
        if !seen.insert(parsed.name.clone()) {
            return Err(Error::invalid(format!(
                "duplicate LinkConfig document for link '{}'",
                parsed.name
            )));
        }
        out.push(parsed);
    }
    Ok(out)
}

/// Extract all `VLANConfig` docs from a loaded config, rejecting duplicate names.
pub fn vlan_configs(config: &Config) -> Result<Vec<VlanConfig>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for doc in config
        .documents()
        .iter()
        .filter(|doc| doc.meta.kind == VLAN_CONFIG_KIND)
    {
        let parsed = decode_vlan_config_body(&doc.body)?;
        if !seen.insert(parsed.name.clone()) {
            return Err(Error::invalid(format!(
                "duplicate VLANConfig document for link '{}'",
                parsed.name
            )));
        }
        out.push(parsed);
    }
    Ok(out)
}

fn decode_link_fields(root: &Yaml, doc: &str) -> Result<LinkFields> {
    let addresses = match root.get("addresses") {
        Some(value) => decode_addresses(value, doc)?,
        None => Vec::new(),
    };
    let routes = match root.get("routes") {
        Some(value) => decode_routes(value, doc)?,
        None => Vec::new(),
    };
    Ok(LinkFields {
        up: optional_bool(root.get("up"), &format!("{doc}.up"))?,
        mtu: optional_u32(root.get("mtu"), &format!("{doc}.mtu"))?.unwrap_or(0),
        multicast: optional_bool(root.get("multicast"), &format!("{doc}.multicast"))?,
        addresses,
        routes,
    })
}

fn decode_addresses(value: &Yaml, doc: &str) -> Result<Vec<AddressConfig>> {
    let Some(items) = value.as_sequence() else {
        return Err(Error::parse(format!("{doc}.addresses must be a sequence")));
    };
    let mut out = Vec::new();
    for item in items {
        if item.as_mapping().is_none() {
            return Err(Error::parse(format!("{doc}.addresses[] must be a mapping")));
        }
        out.push(AddressConfig {
            address: required_string(item, "address", &format!("{doc}.addresses[].address"))?,
            route_priority: optional_u32(
                item.get("routePriority"),
                &format!("{doc}.addresses[].routePriority"),
            )?
            .unwrap_or(0),
        });
    }
    Ok(out)
}

fn decode_routes(value: &Yaml, doc: &str) -> Result<Vec<RouteConfig>> {
    let Some(items) = value.as_sequence() else {
        return Err(Error::parse(format!("{doc}.routes must be a sequence")));
    };
    let mut out = Vec::new();
    for item in items {
        if item.as_mapping().is_none() {
            return Err(Error::parse(format!("{doc}.routes[] must be a mapping")));
        }
        out.push(RouteConfig {
            destination: item.get_str("destination").unwrap_or("").trim().to_string(),
            gateway: item.get_str("gateway").unwrap_or("").trim().to_string(),
            source: item.get_str("source").unwrap_or("").trim().to_string(),
            metric: optional_u32(item.get("metric"), &format!("{doc}.routes[].metric"))?
                .unwrap_or(0),
            mtu: optional_u32(item.get("mtu"), &format!("{doc}.routes[].mtu"))?.unwrap_or(0),
            table: optional_u32(item.get("table"), &format!("{doc}.routes[].table"))?.unwrap_or(0),
        });
    }
    Ok(out)
}

fn required_string(root: &Yaml, key: &str, field: &str) -> Result<String> {
    root.get_str(key)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| Error::invalid(format!("{field} is required")))
}

fn required_u16(value: Option<&Yaml>, field: &str) -> Result<u16> {
    optional_u16(value, field)?.ok_or_else(|| Error::invalid(format!("{field} is required")))
}

fn optional_bool(value: Option<&Yaml>, field: &str) -> Result<Option<bool>> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_bool()
        .map(Some)
        .ok_or_else(|| Error::parse(format!("{field} must be a boolean")))
}

fn optional_u16(value: Option<&Yaml>, field: &str) -> Result<Option<u16>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(raw) = value.as_str() else {
        return Err(Error::parse(format!("{field} must be an integer")));
    };
    raw.parse::<u16>()
        .map(Some)
        .map_err(|_| Error::parse(format!("{field} must be an unsigned 16-bit integer")))
}

fn optional_u32(value: Option<&Yaml>, field: &str) -> Result<Option<u32>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(raw) = value.as_str() else {
        return Err(Error::parse(format!("{field} must be an integer")));
    };
    raw.parse::<u32>()
        .map(Some)
        .map_err(|_| Error::parse(format!("{field} must be an unsigned 32-bit integer")))
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
    use crate::load::load_from_bytes;

    const BASE: &str = "\
version: v1alpha1
machine:
  type: worker
";

    fn multidoc(doc: &str) -> String {
        format!("{BASE}---\n{doc}")
    }

    #[test]
    fn link_config_decodes_static_fields() {
        let doc = "\
apiVersion: v1alpha1
kind: LinkConfig
name: enp0s2
up: true
mtu: 9000
multicast: true
addresses:
  - address: 192.168.1.100/24
    routePriority: 2048
  - address: fd00::1/64
routes:
  - destination: 10.0.0.0/8
    gateway: 10.0.0.1
    source: 192.168.1.100
    metric: 100
    mtu: 1400
    table: 254
  - gateway: fe80::1
";
        let parsed = decode_link_config_body(doc).unwrap();
        assert_eq!(parsed.name, "enp0s2");
        assert_eq!(parsed.link.up, Some(true));
        assert!(parsed.link.up_or_default());
        assert_eq!(parsed.link.mtu, 9000);
        assert_eq!(parsed.link.multicast, Some(true));
        assert_eq!(parsed.link.addresses.len(), 2);
        assert_eq!(parsed.link.addresses[0].route_priority, 2048);
        assert_eq!(parsed.link.routes.len(), 2);
        assert_eq!(parsed.link.routes[0].destination, "10.0.0.0/8");
        assert_eq!(parsed.link.routes[0].metric, 100);
        assert_eq!(parsed.link.routes[1].gateway, "fe80::1");
    }

    #[test]
    fn link_config_defaults_up_and_mtu() {
        let parsed =
            decode_link_config_body("apiVersion: v1alpha1\nkind: LinkConfig\nname: eth0\n")
                .unwrap();
        assert_eq!(parsed.link.up, None);
        assert!(parsed.link.up_or_default());
        assert_eq!(parsed.link.mtu, 0);
    }

    #[test]
    fn vlan_config_decodes_vlan_fields_and_embedded_link_fields() {
        let doc = "\
apiVersion: v1alpha1
kind: VLANConfig
name: enp0s3.34
vlanID: 34
vlanMode: 802.1ad
parent: enp0s3
up: false
mtu: 1500
multicast: true
addresses:
  - address: 192.168.1.100/24
routes:
  - destination: 192.168.0.0/16
    gateway: 192.168.1.1
";
        let parsed = decode_vlan_config_body(doc).unwrap();
        assert_eq!(parsed.name, "enp0s3.34");
        assert_eq!(parsed.vlan_id, 34);
        assert_eq!(parsed.parent, "enp0s3");
        assert_eq!(parsed.vlan_mode_or_default(), VlanMode::Dot1Ad);
        assert_eq!(parsed.link.up, Some(false));
        assert_eq!(parsed.link.multicast, Some(true));
        assert_eq!(parsed.link.addresses[0].address, "192.168.1.100/24");
    }

    #[test]
    fn link_and_vlan_configs_extract_from_loaded_config() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: LinkConfig
name: eth0
mtu: 9000
---
apiVersion: v1alpha1
kind: VLANConfig
name: eth0.100
vlanID: 100
parent: eth0
",
        );
        let container = load_from_bytes(&cfg).unwrap();
        let links = link_configs(&container).unwrap();
        let vlans = vlan_configs(&container).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].name, "eth0");
        assert_eq!(vlans.len(), 1);
        assert_eq!(vlans[0].vlan_id, 100);
    }

    #[test]
    fn load_rejects_duplicate_link_or_vlan_names() {
        let dup_link = multidoc(
            "\
apiVersion: v1alpha1
kind: LinkConfig
name: eth0
---
apiVersion: v1alpha1
kind: LinkConfig
name: eth0
",
        );
        assert_eq!(load_from_bytes(&dup_link).unwrap_err().kind(), "invalid");

        let dup_vlan = multidoc(
            "\
apiVersion: v1alpha1
kind: VLANConfig
name: eth0.100
vlanID: 100
parent: eth0
---
apiVersion: v1alpha1
kind: VLANConfig
name: eth0.100
vlanID: 100
parent: eth0
",
        );
        assert_eq!(load_from_bytes(&dup_vlan).unwrap_err().kind(), "invalid");
    }

    #[test]
    fn rejects_bad_shapes() {
        assert!(
            decode_link_config_body("apiVersion: v1alpha1\nkind: LinkConfig\nname: \"\"\n")
                .is_err()
        );
        assert!(
            decode_link_config_body("apiVersion: v1alpha1\nkind: LinkConfig\nname: eth0\nmtu: 1\n")
                .is_err()
        );
        assert!(decode_link_config_body("apiVersion: v1alpha1\nkind: LinkConfig\nname: eth0\naddresses:\n  - address: 192.168.1.1\n").is_err());
        assert!(
            decode_vlan_config_body(
                "apiVersion: v1alpha1\nkind: VLANConfig\nname: eth0.0\nvlanID: 0\nparent: eth0\n"
            )
            .is_err()
        );
        assert!(decode_vlan_config_body("apiVersion: v1alpha1\nkind: VLANConfig\nname: eth0.10\nvlanID: 10\nparent: eth0\nvlanMode: qinq\n").is_err());
    }
}
