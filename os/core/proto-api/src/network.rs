//! The `NetworkService` API surface (interfaces, routes).
//!
//! Mirrors `pkg/machinery/api/network/network.proto`: `Interfaces` and `Routes`
//! introspection. Modeled over an in-memory network view exposed through the
//! [`NetworkBackend`] trait.

use crate::common::{ApiError, RequestContext};
use os_kernel::role::Role;

/// The administrative/operational state of a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// The link is up and operational.
    Up,
    /// The link is administratively or physically down.
    Down,
    /// Operational state is unknown.
    Unknown,
}

impl LinkState {
    /// Lowercase string form.
    pub fn as_str(self) -> &'static str {
        match self {
            LinkState::Up => "up",
            LinkState::Down => "down",
            LinkState::Unknown => "unknown",
        }
    }
}

/// A network interface, mirroring `network.Interface`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    /// The kernel interface index.
    pub index: u32,
    /// The interface name (e.g. `eth0`, `lo`).
    pub name: String,
    /// The hardware (MAC) address, if any.
    pub mac: Option<String>,
    /// The MTU.
    pub mtu: u32,
    /// The operational state.
    pub state: LinkState,
    /// Assigned CIDR addresses (e.g. `10.0.0.5/24`).
    pub addresses: Vec<String>,
}

impl Interface {
    /// Whether this is the loopback interface.
    pub fn is_loopback(&self) -> bool {
        self.name == "lo"
    }
}

/// A routing-table family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFamily {
    /// IPv4.
    V4,
    /// IPv6.
    V6,
}

/// A route, mirroring `network.Route`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    /// The destination prefix (`0.0.0.0/0` for default).
    pub destination: String,
    /// The gateway, if any.
    pub gateway: Option<String>,
    /// The outgoing interface name.
    pub interface: String,
    /// The route metric/priority (lower wins).
    pub metric: u32,
    /// The address family.
    pub family: AddressFamily,
}

impl Route {
    /// Whether this is a default route.
    pub fn is_default(&self) -> bool {
        self.destination == "0.0.0.0/0" || self.destination == "::/0"
    }
}

/// The network view consulted by the service, behind a trait.
pub trait NetworkBackend {
    /// All interfaces.
    fn interfaces(&self) -> Vec<Interface>;

    /// All routes.
    fn routes(&self) -> Vec<Route>;
}

/// The `NetworkService`.
pub struct NetworkService<B: NetworkBackend> {
    backend: B,
}

impl<B: NetworkBackend> NetworkService<B> {
    /// Wrap a backend.
    pub fn new(backend: B) -> Self {
        NetworkService { backend }
    }

    /// `Interfaces`: list interfaces sorted by kernel index.
    pub fn interfaces(&self, ctx: &RequestContext) -> Result<Vec<Interface>, ApiError> {
        ctx.authorize(Role::Reader)?;
        let mut ifaces = self.backend.interfaces();
        ifaces.sort_by_key(|i| i.index);
        Ok(ifaces)
    }

    /// `Routes`: list routes sorted by (metric, destination).
    pub fn routes(&self, ctx: &RequestContext) -> Result<Vec<Route>, ApiError> {
        ctx.authorize(Role::Reader)?;
        let mut routes = self.backend.routes();
        routes.sort_by(|a, b| {
            a.metric
                .cmp(&b.metric)
                .then(a.destination.cmp(&b.destination))
        });
        Ok(routes)
    }

    /// The selected default route for a family: the default route with the
    /// lowest metric. Mirrors how Talos picks the node's primary gateway.
    pub fn default_route(
        &self,
        ctx: &RequestContext,
        family: AddressFamily,
    ) -> Result<Option<Route>, ApiError> {
        Ok(self
            .routes(ctx)?
            .into_iter()
            .filter(|r| r.is_default() && r.family == family)
            .min_by_key(|r| r.metric))
    }
}

/// An in-memory network view for tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryNetwork {
    /// The interfaces.
    pub interfaces: Vec<Interface>,
    /// The routes.
    pub routes: Vec<Route>,
}

impl InMemoryNetwork {
    /// A simple node with loopback + eth0 and a default route.
    pub fn single_nic() -> Self {
        InMemoryNetwork {
            interfaces: vec![
                Interface {
                    index: 1,
                    name: "lo".into(),
                    mac: None,
                    mtu: 65536,
                    state: LinkState::Up,
                    addresses: vec!["127.0.0.1/8".into()],
                },
                Interface {
                    index: 2,
                    name: "eth0".into(),
                    mac: Some("aa:bb:cc:dd:ee:ff".into()),
                    mtu: 1500,
                    state: LinkState::Up,
                    addresses: vec!["10.0.0.5/24".into()],
                },
            ],
            routes: vec![
                Route {
                    destination: "0.0.0.0/0".into(),
                    gateway: Some("10.0.0.1".into()),
                    interface: "eth0".into(),
                    metric: 100,
                    family: AddressFamily::V4,
                },
                Route {
                    destination: "10.0.0.0/24".into(),
                    gateway: None,
                    interface: "eth0".into(),
                    metric: 0,
                    family: AddressFamily::V4,
                },
            ],
        }
    }
}

impl NetworkBackend for InMemoryNetwork {
    fn interfaces(&self) -> Vec<Interface> {
        self.interfaces.clone()
    }
    fn routes(&self) -> Vec<Route> {
        self.routes.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Code;

    #[test]
    fn interfaces_sorted_by_index() {
        let svc = NetworkService::new(InMemoryNetwork::single_nic());
        let ifaces = svc.interfaces(&RequestContext::admin_local()).unwrap();
        assert_eq!(ifaces[0].index, 1);
        assert!(ifaces[0].is_loopback());
        assert_eq!(ifaces[1].name, "eth0");
        assert_eq!(ifaces[1].state.as_str(), "up");
    }

    #[test]
    fn routes_sorted_by_metric() {
        let svc = NetworkService::new(InMemoryNetwork::single_nic());
        let routes = svc.routes(&RequestContext::admin_local()).unwrap();
        // metric 0 (link-local) before metric 100 (default).
        assert_eq!(routes[0].metric, 0);
        assert!(!routes[0].is_default());
        assert!(routes[1].is_default());
    }

    #[test]
    fn default_route_selection() {
        let mut net = InMemoryNetwork::single_nic();
        // Add a second default route with a higher metric.
        net.routes.push(Route {
            destination: "0.0.0.0/0".into(),
            gateway: Some("10.0.0.254".into()),
            interface: "eth0".into(),
            metric: 200,
            family: AddressFamily::V4,
        });
        let svc = NetworkService::new(net);
        let def = svc
            .default_route(&RequestContext::admin_local(), AddressFamily::V4)
            .unwrap()
            .unwrap();
        assert_eq!(def.metric, 100);
        assert_eq!(def.gateway.as_deref(), Some("10.0.0.1"));

        // No IPv6 default present.
        assert!(
            svc.default_route(&RequestContext::admin_local(), AddressFamily::V6)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn read_gated() {
        let svc = NetworkService::new(InMemoryNetwork::single_nic());
        let nobody = RequestContext::with_roles(os_kernel::role::RoleSet::new());
        assert_eq!(
            svc.interfaces(&nobody).unwrap_err().code,
            Code::PermissionDenied
        );
    }
}
