//! KubeSpan endpoints and NAT/endpoint reconciliation.
//!
//! Mirrors `pkg/machinery/resources/kubespan.Endpoint` and the
//! `EndpointController` in `internal/app/machined/pkg/controllers/kubespan`.
//!
//! A KubeSpan peer can be reachable on several endpoints (advertised local
//! addresses plus observed NAT-traversed addresses). The controller learns the
//! "last known good" endpoint for each peer from peer-status observations and
//! feeds it back so the affiliate endpoint set converges on the address that
//! actually carried traffic.

use crate::wireguard_spec::WireguardKey;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::fmt;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// A single `address:port` endpoint a peer can be reached on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Endpoint {
    addr: NodeAddress,
    port: u16,
}

impl Ord for Endpoint {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        crate::wireguard_spec::addr_sort_key(&self.addr)
            .cmp(&crate::wireguard_spec::addr_sort_key(&other.addr))
            .then(self.port.cmp(&other.port))
    }
}

impl PartialOrd for Endpoint {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Endpoint {
    /// Construct an endpoint, rejecting port 0.
    pub fn new(addr: NodeAddress, port: u16) -> Result<Self> {
        if port == 0 {
            return Err(Error::invalid("endpoint port must be non-zero"));
        }
        Ok(Endpoint { addr, port })
    }

    /// Parse an `a.b.c.d:port` IPv4 endpoint.
    pub fn parse_v4(s: &str) -> Result<Self> {
        let (host, port) = s
            .rsplit_once(':')
            .ok_or_else(|| Error::parse("endpoint missing ':port'"))?;
        let addr = NodeAddress::parse_v4(host)?;
        let port: u16 = port
            .parse()
            .map_err(|_| Error::parse("invalid endpoint port"))?;
        Self::new(addr, port)
    }

    /// The address component.
    pub fn addr(&self) -> NodeAddress {
        self.addr
    }

    /// The port component.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Whether this endpoint is usable for an off-host WireGuard tunnel.
    pub fn is_routable(&self) -> bool {
        !self.addr.is_loopback()
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.addr {
            NodeAddress::V4(_) => write!(f, "{}:{}", self.addr, self.port),
            NodeAddress::V6(_) => write!(f, "[{}]:{}", self.addr, self.port),
        }
    }
}

/// An ordered, deduplicated set of candidate endpoints for a peer.
///
/// Routable (non-loopback) endpoints sort ahead of loopback ones so the
/// manager tries the most useful candidate first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EndpointSet {
    endpoints: Vec<Endpoint>,
}

impl EndpointSet {
    /// An empty set.
    pub fn new() -> Self {
        EndpointSet {
            endpoints: Vec::new(),
        }
    }

    /// Insert an endpoint, keeping the set sorted (routable first) and unique.
    pub fn insert(&mut self, ep: Endpoint) -> bool {
        if self.endpoints.contains(&ep) {
            return false;
        }
        self.endpoints.push(ep);
        self.endpoints
            .sort_by(|a, b| b.is_routable().cmp(&a.is_routable()).then(a.cmp(b)));
        true
    }

    /// Number of endpoints.
    pub fn len(&self) -> usize {
        self.endpoints.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }

    /// The preferred (first routable) endpoint, if any.
    pub fn preferred(&self) -> Option<Endpoint> {
        self.endpoints.iter().copied().find(|e| e.is_routable())
    }

    /// Iterate endpoints in priority order.
    pub fn iter(&self) -> impl Iterator<Item = &Endpoint> {
        self.endpoints.iter()
    }

    /// The endpoints as a slice.
    pub fn as_slice(&self) -> &[Endpoint] {
        &self.endpoints
    }
}

/// An observation that a peer was reachable on a specific endpoint at a tick.
///
/// Mirrors the data the manager records when a handshake succeeds, used to pin
/// the "last known good" endpoint for a peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndpointObservation {
    pub peer_key: u64,
    pub endpoint: Endpoint,
    pub observed_at_tick: u64,
}

/// Tracks last-known-good endpoints per peer and reconciles them against the
/// advertised candidate sets.
///
/// This is the in-memory model of the `EndpointController`'s state: given a
/// stream of successful-handshake observations, it produces, for each peer, the
/// single endpoint that should be pinned for the affiliate.
#[derive(Debug, Default)]
pub struct EndpointController {
    last_known_good: BTreeMap<u64, EndpointObservation>,
}

impl EndpointController {
    /// New, empty controller.
    pub fn new() -> Self {
        EndpointController {
            last_known_good: BTreeMap::new(),
        }
    }

    /// Key a public key into the controller's compact peer index.
    pub fn peer_index(key: &WireguardKey) -> u64 {
        os_kernel::id::Fingerprint::of_str(key.as_str()).value()
    }

    /// Record a successful-handshake observation. A newer observation for the
    /// same peer overrides an older one; an older one is ignored.
    pub fn observe(&mut self, obs: EndpointObservation) -> bool {
        match self.last_known_good.get(&obs.peer_key) {
            Some(existing) if existing.observed_at_tick >= obs.observed_at_tick => false,
            _ => {
                self.last_known_good.insert(obs.peer_key, obs);
                true
            }
        }
    }

    /// The pinned last-known-good endpoint for a peer, if any.
    pub fn last_known_good(&self, peer_key: u64) -> Option<Endpoint> {
        self.last_known_good.get(&peer_key).map(|o| o.endpoint)
    }

    /// Reconcile a peer's advertised candidate set with the last-known-good
    /// endpoint: if a good endpoint is known, it is moved to the front of the
    /// returned candidate list (NAT-aware ordering). Otherwise the candidate
    /// set's own priority order is preserved.
    pub fn reconcile_candidates(&self, peer_key: u64, candidates: &EndpointSet) -> Vec<Endpoint> {
        let mut out: Vec<Endpoint> = candidates.iter().copied().collect();
        if let Some(good) = self.last_known_good(peer_key) {
            if let Some(pos) = out.iter().position(|e| *e == good) {
                let e = out.remove(pos);
                out.insert(0, e);
            } else {
                out.insert(0, good);
            }
        }
        out
    }

    /// Drop the pinned endpoint for a peer (e.g. peer removed from cluster).
    pub fn forget(&mut self, peer_key: u64) -> bool {
        self.last_known_good.remove(&peer_key).is_some()
    }

    /// Number of peers with a pinned endpoint.
    pub fn tracked_peers(&self) -> usize {
        self.last_known_good.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep(s: &str) -> Endpoint {
        Endpoint::parse_v4(s).unwrap()
    }

    #[test]
    fn endpoint_parse_and_format() {
        let e = ep("203.0.113.7:51820");
        assert_eq!(e.port(), 51820);
        assert_eq!(e.to_string(), "203.0.113.7:51820");
        assert!(Endpoint::parse_v4("203.0.113.7:0").is_err());
        assert!(Endpoint::parse_v4("nope").is_err());
    }

    #[test]
    fn endpoint_set_dedups_and_orders_routable_first() {
        let mut set = EndpointSet::new();
        assert!(set.insert(ep("127.0.0.1:51820")));
        assert!(set.insert(ep("203.0.113.7:51820")));
        assert!(!set.insert(ep("203.0.113.7:51820")));
        assert_eq!(set.len(), 2);
        // routable endpoint preferred over loopback.
        assert_eq!(set.preferred().unwrap(), ep("203.0.113.7:51820"));
        assert_eq!(set.as_slice()[0], ep("203.0.113.7:51820"));
    }

    #[test]
    fn observation_keeps_newest() {
        let mut c = EndpointController::new();
        let k = 42u64;
        assert!(c.observe(EndpointObservation {
            peer_key: k,
            endpoint: ep("203.0.113.7:51820"),
            observed_at_tick: 10,
        }));
        // older observation ignored.
        assert!(!c.observe(EndpointObservation {
            peer_key: k,
            endpoint: ep("198.51.100.1:51820"),
            observed_at_tick: 5,
        }));
        assert_eq!(c.last_known_good(k).unwrap(), ep("203.0.113.7:51820"));
        // newer observation wins.
        assert!(c.observe(EndpointObservation {
            peer_key: k,
            endpoint: ep("198.51.100.1:51820"),
            observed_at_tick: 20,
        }));
        assert_eq!(c.last_known_good(k).unwrap(), ep("198.51.100.1:51820"));
    }

    #[test]
    fn reconcile_promotes_last_known_good() {
        let mut c = EndpointController::new();
        let k = 7u64;
        let mut set = EndpointSet::new();
        set.insert(ep("203.0.113.7:51820"));
        set.insert(ep("198.51.100.1:51820"));
        c.observe(EndpointObservation {
            peer_key: k,
            endpoint: ep("198.51.100.1:51820"),
            observed_at_tick: 1,
        });
        let ordered = c.reconcile_candidates(k, &set);
        assert_eq!(ordered[0], ep("198.51.100.1:51820"));
    }

    #[test]
    fn reconcile_injects_unknown_good_endpoint() {
        let mut c = EndpointController::new();
        let k = 7u64;
        let set = EndpointSet::new();
        c.observe(EndpointObservation {
            peer_key: k,
            endpoint: ep("198.51.100.1:51820"),
            observed_at_tick: 1,
        });
        // candidate set empty, but a known-good NAT endpoint exists.
        let ordered = c.reconcile_candidates(k, &set);
        assert_eq!(ordered, alloc::vec![ep("198.51.100.1:51820")]);
    }

    #[test]
    fn forget_removes_tracking() {
        let mut c = EndpointController::new();
        c.observe(EndpointObservation {
            peer_key: 1,
            endpoint: ep("203.0.113.7:51820"),
            observed_at_tick: 1,
        });
        assert_eq!(c.tracked_peers(), 1);
        assert!(c.forget(1));
        assert!(!c.forget(1));
        assert_eq!(c.tracked_peers(), 0);
    }
}
