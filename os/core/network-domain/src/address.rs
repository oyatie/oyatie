//! Address specs and the node-address aggregation.
//!
//! Mirrors `AddressSpec`, `AddressConfigController`, `AddressMergeController`
//! and `NodeAddressController`. An [`AddressSpec`] describes a single IP/prefix
//! to assign to a link; the merge logic folds specs from multiple layers by
//! priority; [`NodeAddressSpec`] collects the resulting node-wide address set.

use crate::config_layer::ConfigLayer;
use crate::link::AddressFamily;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// Scope of an address, mirroring the kernel `rt_scope` values used by Talos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Global scope (routable).
    Global,
    /// Link-local scope.
    Link,
    /// Host scope (loopback).
    Host,
}

impl Scope {
    /// The kernel numeric value.
    pub fn rt_scope(self) -> u8 {
        match self {
            Scope::Global => 0,
            Scope::Link => 253,
            Scope::Host => 254,
        }
    }
}

/// Bitflags for address attributes, mirroring `IFA_F_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AddressFlags {
    /// Permanent (statically configured) address.
    pub permanent: bool,
    /// Temporary (privacy) address.
    pub temporary: bool,
    /// Address is still going through duplicate-address-detection.
    pub tentative: bool,
}

impl AddressFlags {
    /// A permanent address with no other flags.
    pub fn permanent() -> Self {
        AddressFlags {
            permanent: true,
            ..Default::default()
        }
    }
}

/// A single network-address assignment for a link.
///
/// Equivalent to Talos `network.AddressSpecSpec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressSpec {
    /// The IP address.
    pub address: NodeAddress,
    /// CIDR prefix length (e.g. 24 for IPv4 /24).
    pub prefix_len: u8,
    /// Name of the link this address is bound to (e.g. `eth0`).
    pub link_name: String,
    /// Address family.
    pub family: AddressFamily,
    /// Routing scope.
    pub scope: Scope,
    /// Attribute flags.
    pub flags: AddressFlags,
    /// Route priority for routes derived from this address.
    pub priority: u32,
    /// Provenance / priority of this spec.
    pub layer: ConfigLayer,
}

impl AddressSpec {
    /// Build a permanent global address for a link, inferring the family from
    /// the address and validating the prefix length.
    pub fn new(
        address: NodeAddress,
        prefix_len: u8,
        link_name: impl Into<String>,
        layer: ConfigLayer,
    ) -> Result<Self> {
        let family = if address.is_v4() {
            AddressFamily::Inet4
        } else {
            AddressFamily::Inet6
        };
        let spec = AddressSpec {
            address,
            prefix_len,
            link_name: link_name.into(),
            family,
            scope: if address.is_loopback() {
                Scope::Host
            } else {
                Scope::Global
            },
            flags: AddressFlags::permanent(),
            priority: 0,
            layer,
        };
        spec.validate()?;
        Ok(spec)
    }

    /// Validate prefix length against the family and require a link name.
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
        if self.link_name.is_empty() {
            return Err(Error::invalid("address spec has empty link name"));
        }
        match (self.family, self.address) {
            (AddressFamily::Inet4, NodeAddress::V6(_))
            | (AddressFamily::Inet6, NodeAddress::V4(_)) => {
                Err(Error::invalid("address family does not match address"))
            }
            _ => Ok(()),
        }
    }

    /// A stable COSI id: `<link>/<addr>/<prefix>`.
    pub fn id(&self) -> String {
        alloc::format!("{}/{}/{}", self.link_name, self.address, self.prefix_len)
    }
}

/// The set of addresses present on the node, aggregated across links and
/// filtered for use as the node's advertised addresses.
///
/// Mirrors `network.NodeAddressSpec`. The filtered/ordered output is what
/// kubelet and the API server advertise.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeAddressSpec {
    addresses: Vec<NodeAddress>,
}

impl NodeAddressSpec {
    /// An empty node-address set.
    pub fn new() -> Self {
        NodeAddressSpec {
            addresses: Vec::new(),
        }
    }

    /// Add an address, de-duplicating.
    pub fn add(&mut self, addr: NodeAddress) {
        if !self.addresses.contains(&addr) {
            self.addresses.push(addr);
        }
    }

    /// All known node addresses.
    pub fn all(&self) -> &[NodeAddress] {
        &self.addresses
    }

    /// Addresses excluding loopback — the "current" advertised set.
    pub fn routed(&self) -> Vec<NodeAddress> {
        self.addresses
            .iter()
            .copied()
            .filter(|a| !a.is_loopback())
            .collect()
    }

    /// The first non-loopback address, preferring private/RFC1918 ranges, used
    /// as the primary node IP. Returns `None` when only loopback is present.
    pub fn primary(&self) -> Option<NodeAddress> {
        let mut routed = self.routed();
        if routed.is_empty() {
            return None;
        }
        // stable preference: private addresses first, then the rest
        routed.sort_by_key(|a| u8::from(!a.is_private()));
        routed.first().copied()
    }
}

/// In-memory merge of address specs by COSI id, applying layer priority.
///
/// Equivalent to `AddressMergeController`: for each logical address id the
/// highest-priority layer wins. Returns the merged specs sorted by id.
pub fn merge_addresses(specs: &[AddressSpec]) -> Vec<AddressSpec> {
    let mut by_id: BTreeMap<String, AddressSpec> = BTreeMap::new();
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

impl core::str::FromStr for Scope {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "global" => Ok(Scope::Global),
            "link" => Ok(Scope::Link),
            "host" => Ok(Scope::Host),
            other => Err(Error::parse(alloc::format!("unknown scope '{other}'"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    fn v4(s: &str) -> NodeAddress {
        NodeAddress::parse_v4(s).unwrap()
    }

    #[test]
    fn address_spec_validation() {
        let ok = AddressSpec::new(v4("10.0.0.5"), 24, "eth0", ConfigLayer::Platform).unwrap();
        assert_eq!(ok.family, AddressFamily::Inet4);
        assert_eq!(ok.scope, Scope::Global);
        assert_eq!(ok.priority, 0);
        assert_eq!(ok.id(), "eth0/10.0.0.5/24");

        // prefix too large for v4
        assert!(AddressSpec::new(v4("10.0.0.5"), 40, "eth0", ConfigLayer::Platform).is_err());

        // empty link name
        let mut bad = ok.clone();
        bad.link_name = String::new();
        assert!(bad.validate().is_err());

        // mismatched family
        let mut mism = ok.clone();
        mism.family = AddressFamily::Inet6;
        assert!(mism.validate().is_err());
    }

    #[test]
    fn loopback_gets_host_scope() {
        let lo = AddressSpec::new(v4("127.0.0.1"), 8, "lo", ConfigLayer::Default).unwrap();
        assert_eq!(lo.scope, Scope::Host);
        assert_eq!(lo.scope.rt_scope(), 254);
    }

    #[test]
    fn merge_prefers_higher_layer() {
        let low = AddressSpec::new(v4("10.0.0.5"), 24, "eth0", ConfigLayer::Cmdline).unwrap();
        let high =
            AddressSpec::new(v4("10.0.0.5"), 24, "eth0", ConfigLayer::Configuration).unwrap();
        let other = AddressSpec::new(v4("10.0.0.6"), 24, "eth0", ConfigLayer::Platform).unwrap();

        let merged = merge_addresses(&[low, high.clone(), other.clone()]);
        assert_eq!(merged.len(), 2);
        let same_id: Vec<_> = merged.iter().filter(|s| s.id() == high.id()).collect();
        assert_eq!(same_id.len(), 1);
        assert_eq!(same_id[0].layer, ConfigLayer::Configuration);
    }

    #[test]
    fn priority_is_modeled_but_not_part_of_address_id() {
        let mut default =
            AddressSpec::new(v4("10.0.0.5"), 24, "eth0", ConfigLayer::Platform).unwrap();
        assert_eq!(default.priority, 0);

        let id = default.id();
        default.priority = 2048;

        assert_eq!(default.priority, 2048);
        assert_eq!(default.id(), id);
        assert!(default.validate().is_ok());
    }

    #[test]
    fn merge_carries_priority_from_winning_layer() {
        let mut low = AddressSpec::new(v4("10.0.0.5"), 24, "eth0", ConfigLayer::Cmdline).unwrap();
        low.priority = 1024;
        let mut high =
            AddressSpec::new(v4("10.0.0.5"), 24, "eth0", ConfigLayer::Configuration).unwrap();
        high.priority = 2048;

        let merged = merge_addresses(&[low, high.clone()]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].id(), high.id());
        assert_eq!(merged[0].layer, ConfigLayer::Configuration);
        assert_eq!(merged[0].priority, 2048);
    }

    #[test]
    fn node_address_primary_prefers_private() {
        let mut na = NodeAddressSpec::new();
        na.add(v4("127.0.0.1"));
        na.add(v4("8.8.8.8"));
        na.add(v4("10.0.0.5"));
        assert_eq!(na.routed().len(), 2);
        assert_eq!(na.primary(), Some(v4("10.0.0.5")));

        let mut only_lo = NodeAddressSpec::new();
        only_lo.add(v4("127.0.0.1"));
        assert_eq!(only_lo.primary(), None);
    }

    #[test]
    fn scope_from_str() {
        assert_eq!(Scope::from_str("global").unwrap(), Scope::Global);
        assert!(Scope::from_str("bogus").is_err());
    }
}
