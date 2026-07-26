//! The netlink boundary, modelled as a trait with an in-memory implementation.
//!
//! Mirrors `pkg/machinery/nethelpers` and the `rtnetlink`-backed plumbing used
//! by the link/address/route controllers. The real Talos code talks to the
//! kernel over `AF_NETLINK` sockets; here that OS boundary is a [`Netlink`]
//! trait so controllers can be exercised against a deterministic in-memory
//! kernel ([`InMemoryNetlink`]) in tests.
//!
//! The trait surface covers the operations the spec controllers issue when they
//! reconcile desired specs against observed kernel state: list/create/delete
//! links, set admin state and MTU, and add/delete addresses and routes.

use crate::address::AddressSpec;
use crate::link::{LinkSpec, LinkStatus, LinkType, OperState};
use crate::route::RouteSpec;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use os_kernel::error::{Error, Result};

/// Operations the network controllers issue against the kernel routing stack.
///
/// Each method mirrors an `rtnetlink` request. Implementations must be
/// idempotent where the kernel is (e.g. re-adding an existing address is a
/// no-op success), since controllers reconcile repeatedly.
pub trait Netlink {
    /// List all links currently known to the kernel.
    fn list_links(&self) -> Vec<LinkStatus>;

    /// Look up a single link by name.
    fn get_link(&self, name: &str) -> Option<LinkStatus>;

    /// Create a (virtual) link from a spec. Errors if a link with that name
    /// already exists.
    fn create_link(&mut self, spec: &LinkSpec) -> Result<()>;

    /// Delete a link by name. Errors if the link does not exist.
    fn delete_link(&mut self, name: &str) -> Result<()>;

    /// Set the administrative up/down flag on a link.
    fn set_link_up(&mut self, name: &str, up: bool) -> Result<()>;

    /// Set the MTU on a link.
    fn set_link_mtu(&mut self, name: &str, mtu: u32) -> Result<()>;

    /// List all addresses currently configured.
    fn list_addresses(&self) -> Vec<AddressSpec>;

    /// Add an address. Idempotent: re-adding an existing address succeeds.
    /// Errors if the target link does not exist.
    fn add_address(&mut self, spec: &AddressSpec) -> Result<()>;

    /// Delete an address by its logical id (`<link>/<addr>/<prefix>`).
    fn delete_address(&mut self, id: &str) -> Result<()>;

    /// List all routes currently configured.
    fn list_routes(&self) -> Vec<RouteSpec>;

    /// Add a route. Idempotent on the route's logical id.
    fn add_route(&mut self, spec: &RouteSpec) -> Result<()>;

    /// Delete a route by its logical id.
    fn delete_route(&mut self, id: &str) -> Result<()>;
}

/// A deterministic in-memory stand-in for the kernel routing stack.
///
/// State is keyed by the same logical ids the controllers use, so reconcilers
/// can be driven and asserted against without any OS interaction.
#[derive(Debug, Default)]
pub struct InMemoryNetlink {
    links: BTreeMap<String, LinkStatus>,
    addresses: BTreeMap<String, AddressSpec>,
    routes: BTreeMap<String, RouteSpec>,
}

impl InMemoryNetlink {
    /// An empty kernel with no links.
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed an observed link directly (e.g. a discovered physical interface).
    pub fn with_link(mut self, status: LinkStatus) -> Self {
        self.links.insert(status.name.clone(), status);
        self
    }

    fn require_link(&self, name: &str) -> Result<()> {
        if self.links.contains_key(name) {
            Ok(())
        } else {
            Err(Error::not_found(alloc::format!(
                "link '{name}' does not exist"
            )))
        }
    }
}

impl Netlink for InMemoryNetlink {
    fn list_links(&self) -> Vec<LinkStatus> {
        self.links.values().cloned().collect()
    }

    fn get_link(&self, name: &str) -> Option<LinkStatus> {
        self.links.get(name).cloned()
    }

    fn create_link(&mut self, spec: &LinkSpec) -> Result<()> {
        spec.validate()?;
        if self.links.contains_key(&spec.name) {
            return Err(Error::invalid(alloc::format!(
                "link '{}' already exists",
                spec.name
            )));
        }
        let status = LinkStatus {
            name: spec.name.clone(),
            link_type: LinkType::Ether,
            kind: spec.kind.kind_str().to_string(),
            aliases: Vec::new(),
            admin_up: spec.up,
            oper_state: if spec.up {
                OperState::Up
            } else {
                OperState::Down
            },
            // virtual links report carrier once members/parent are present;
            // model them as carrier-up on creation.
            carrier: spec.up,
            hardware_addr: [0; 6],
            mtu: spec.mtu,
        };
        self.links.insert(spec.name.clone(), status);
        Ok(())
    }

    fn delete_link(&mut self, name: &str) -> Result<()> {
        if self.links.remove(name).is_none() {
            return Err(Error::not_found(alloc::format!(
                "link '{name}' does not exist"
            )));
        }
        // Cascade: drop addresses and routes bound to the link, as the kernel
        // does when an interface goes away.
        self.addresses.retain(|_, a| a.link_name != name);
        self.routes.retain(|_, r| r.out_link != name);
        Ok(())
    }

    fn set_link_up(&mut self, name: &str, up: bool) -> Result<()> {
        let link = self
            .links
            .get_mut(name)
            .ok_or_else(|| Error::not_found(alloc::format!("link '{name}' does not exist")))?;
        link.admin_up = up;
        link.oper_state = if up { OperState::Up } else { OperState::Down };
        link.carrier = up;
        Ok(())
    }

    fn set_link_mtu(&mut self, name: &str, mtu: u32) -> Result<()> {
        if !(68..=65535).contains(&mtu) {
            return Err(Error::invalid(alloc::format!(
                "MTU {mtu} out of range 68..=65535"
            )));
        }
        let link = self
            .links
            .get_mut(name)
            .ok_or_else(|| Error::not_found(alloc::format!("link '{name}' does not exist")))?;
        link.mtu = mtu;
        Ok(())
    }

    fn list_addresses(&self) -> Vec<AddressSpec> {
        self.addresses.values().cloned().collect()
    }

    fn add_address(&mut self, spec: &AddressSpec) -> Result<()> {
        spec.validate()?;
        self.require_link(&spec.link_name)?;
        self.addresses.insert(spec.id(), spec.clone());
        Ok(())
    }

    fn delete_address(&mut self, id: &str) -> Result<()> {
        if self.addresses.remove(id).is_none() {
            return Err(Error::not_found(alloc::format!(
                "address '{id}' not configured"
            )));
        }
        Ok(())
    }

    fn list_routes(&self) -> Vec<RouteSpec> {
        self.routes.values().cloned().collect()
    }

    fn add_route(&mut self, spec: &RouteSpec) -> Result<()> {
        spec.validate()?;
        self.require_link(&spec.out_link)?;
        self.routes.insert(spec.id(), spec.clone());
        Ok(())
    }

    fn delete_route(&mut self, id: &str) -> Result<()> {
        if self.routes.remove(id).is_none() {
            return Err(Error::not_found(alloc::format!(
                "route '{id}' not configured"
            )));
        }
        Ok(())
    }
}

/// Reconcile the desired set of addresses for a link against the kernel: add
/// any missing, delete any extra. Mirrors `AddressSpecController.apply`.
///
/// Returns the number of (added, deleted) operations performed.
pub fn reconcile_addresses<N: Netlink>(
    nl: &mut N,
    desired: &[AddressSpec],
) -> Result<(usize, usize)> {
    use alloc::collections::BTreeSet;
    let desired_ids: BTreeSet<String> = desired.iter().map(AddressSpec::id).collect();
    let current_ids: BTreeSet<String> = nl.list_addresses().iter().map(AddressSpec::id).collect();

    let mut added = 0;
    for spec in desired {
        if !current_ids.contains(&spec.id()) {
            nl.add_address(spec)?;
            added += 1;
        }
    }
    let mut deleted = 0;
    for id in current_ids.difference(&desired_ids) {
        nl.delete_address(id)?;
        deleted += 1;
    }
    Ok((added, deleted))
}

/// Reconcile the desired set of routes against the kernel. Mirrors
/// `RouteSpecController.apply`.
pub fn reconcile_routes<N: Netlink>(nl: &mut N, desired: &[RouteSpec]) -> Result<(usize, usize)> {
    use alloc::collections::BTreeSet;
    let desired_ids: BTreeSet<String> = desired.iter().map(RouteSpec::id).collect();
    let current_ids: BTreeSet<String> = nl.list_routes().iter().map(RouteSpec::id).collect();

    let mut added = 0;
    for spec in desired {
        if !current_ids.contains(&spec.id()) {
            nl.add_route(spec)?;
            added += 1;
        }
    }
    let mut deleted = 0;
    for id in current_ids.difference(&desired_ids) {
        nl.delete_route(id)?;
        deleted += 1;
    }
    Ok((added, deleted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_layer::ConfigLayer;
    use crate::link::LinkKind;
    use os_kernel::address::NodeAddress;

    fn v4(s: &str) -> NodeAddress {
        NodeAddress::parse_v4(s).unwrap()
    }

    fn seeded() -> InMemoryNetlink {
        InMemoryNetlink::new().with_link(LinkStatus {
            name: String::from("eth0"),
            link_type: LinkType::Ether,
            kind: String::new(),
            aliases: Vec::new(),
            admin_up: true,
            oper_state: OperState::Up,
            carrier: true,
            hardware_addr: [0x02, 0, 0, 0, 0, 1],
            mtu: 1500,
        })
    }

    #[test]
    fn create_and_delete_virtual_link() {
        let mut nl = seeded();
        let bond = LinkSpec {
            name: String::from("bond0"),
            up: true,
            mtu: 1500,
            multicast: None,
            kind: LinkKind::Bond {
                members: alloc::vec![String::from("eth0")],
                mode: crate::link::BondMode::Lacp,
            },
            layer: ConfigLayer::Configuration,
        };
        nl.create_link(&bond).unwrap();
        assert!(nl.get_link("bond0").is_some());
        // duplicate create fails
        assert!(nl.create_link(&bond).is_err());

        nl.delete_link("bond0").unwrap();
        assert!(nl.get_link("bond0").is_none());
        assert!(nl.delete_link("bond0").is_err());
    }

    #[test]
    fn set_up_and_mtu() {
        let mut nl = seeded();
        nl.set_link_up("eth0", false).unwrap();
        assert!(!nl.get_link("eth0").unwrap().admin_up);
        nl.set_link_mtu("eth0", 9000).unwrap();
        assert_eq!(nl.get_link("eth0").unwrap().mtu, 9000);
        assert!(nl.set_link_mtu("eth0", 1).is_err());
        assert!(nl.set_link_up("missing", true).is_err());
    }

    #[test]
    fn address_requires_link_and_is_idempotent() {
        let mut nl = seeded();
        let a = AddressSpec::new(v4("10.0.0.5"), 24, "eth0", ConfigLayer::Configuration).unwrap();
        nl.add_address(&a).unwrap();
        nl.add_address(&a).unwrap(); // idempotent
        assert_eq!(nl.list_addresses().len(), 1);

        let orphan =
            AddressSpec::new(v4("10.0.0.6"), 24, "eth9", ConfigLayer::Configuration).unwrap();
        assert!(nl.add_address(&orphan).is_err());

        nl.delete_address(&a.id()).unwrap();
        assert!(nl.delete_address(&a.id()).is_err());
    }

    #[test]
    fn delete_link_cascades() {
        let mut nl = seeded();
        let a = AddressSpec::new(v4("10.0.0.5"), 24, "eth0", ConfigLayer::Configuration).unwrap();
        let r = RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Configuration).unwrap();
        nl.add_address(&a).unwrap();
        nl.add_route(&r).unwrap();
        nl.delete_link("eth0").unwrap();
        assert!(nl.list_addresses().is_empty());
        assert!(nl.list_routes().is_empty());
    }

    #[test]
    fn reconcile_addresses_adds_and_deletes() {
        let mut nl = seeded();
        let a = AddressSpec::new(v4("10.0.0.5"), 24, "eth0", ConfigLayer::Configuration).unwrap();
        let b = AddressSpec::new(v4("10.0.0.6"), 24, "eth0", ConfigLayer::Configuration).unwrap();

        let (added, deleted) = reconcile_addresses(&mut nl, &[a.clone(), b.clone()]).unwrap();
        assert_eq!((added, deleted), (2, 0));

        // now only `a` is desired: `b` should be deleted
        let (added, deleted) = reconcile_addresses(&mut nl, core::slice::from_ref(&a)).unwrap();
        assert_eq!((added, deleted), (0, 1));
        assert_eq!(nl.list_addresses().len(), 1);
        assert_eq!(nl.list_addresses()[0].id(), a.id());
    }

    #[test]
    fn reconcile_routes_converges() {
        let mut nl = seeded();
        let r = RouteSpec::default_via(v4("10.0.0.1"), "eth0", ConfigLayer::Configuration).unwrap();
        let (added, _) = reconcile_routes(&mut nl, core::slice::from_ref(&r)).unwrap();
        assert_eq!(added, 1);
        // reconciling the same set is a no-op
        let (added, deleted) = reconcile_routes(&mut nl, core::slice::from_ref(&r)).unwrap();
        assert_eq!((added, deleted), (0, 0));
        // emptying it removes the route
        let (_, deleted) = reconcile_routes(&mut nl, &[]).unwrap();
        assert_eq!(deleted, 1);
    }
}
