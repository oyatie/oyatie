//! Cluster control-plane endpoints and per-node KubeSpan/SideroLink endpoints.
//!
//! Mirrors `cluster.Endpoint` / endpoint discovery in Talos: a set of addresses
//! (ip:port) a peer can be reached on. Endpoints are sorted and deduplicated so
//! that membership churn produces stable resource fingerprints.

use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// A single `address:port` endpoint a peer can be reached on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClusterEndpoint {
    addr: NodeAddress,
    port: u16,
}

impl Ord for ClusterEndpoint {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        crate::addr_sort_key(&self.addr)
            .cmp(&crate::addr_sort_key(&other.addr))
            .then(self.port.cmp(&other.port))
    }
}

impl PartialOrd for ClusterEndpoint {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ClusterEndpoint {
    /// Construct an endpoint, rejecting port 0.
    pub fn new(addr: NodeAddress, port: u16) -> Result<Self> {
        if port == 0 {
            return Err(Error::invalid("endpoint port must be non-zero"));
        }
        Ok(ClusterEndpoint { addr, port })
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

    /// Whether this endpoint is reachable off the local host (not loopback).
    pub fn is_routable(&self) -> bool {
        !self.addr.is_loopback()
    }
}

impl core::fmt::Display for ClusterEndpoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.addr {
            NodeAddress::V4(_) => write!(f, "{}:{}", self.addr, self.port),
            // bracket IPv6 hosts per RFC 3986.
            NodeAddress::V6(_) => write!(f, "[{}]:{}", self.addr, self.port),
        }
    }
}

/// An ordered, deduplicated collection of endpoints advertised for a peer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EndpointList {
    endpoints: Vec<ClusterEndpoint>,
}

impl EndpointList {
    /// An empty list.
    pub fn new() -> Self {
        EndpointList {
            endpoints: Vec::new(),
        }
    }

    /// Insert an endpoint, keeping the list sorted and unique. Returns whether a
    /// new endpoint was added.
    pub fn insert(&mut self, ep: ClusterEndpoint) -> bool {
        match self.endpoints.binary_search(&ep) {
            Ok(_) => false,
            Err(pos) => {
                self.endpoints.insert(pos, ep);
                true
            }
        }
    }

    /// Number of endpoints.
    pub fn len(&self) -> usize {
        self.endpoints.len()
    }

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }

    /// The endpoints in canonical (sorted) order.
    pub fn as_slice(&self) -> &[ClusterEndpoint] {
        &self.endpoints
    }

    /// Only the routable (non-loopback) endpoints.
    pub fn routable(&self) -> Vec<ClusterEndpoint> {
        self.endpoints
            .iter()
            .copied()
            .filter(ClusterEndpoint::is_routable)
            .collect()
    }
}

impl FromIterator<ClusterEndpoint> for EndpointList {
    /// Build from an iterator, sorting and deduplicating.
    fn from_iter<T: IntoIterator<Item = ClusterEndpoint>>(iter: T) -> Self {
        let mut list = EndpointList::new();
        for ep in iter {
            list.insert(ep);
        }
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_display_v4() {
        let ep = ClusterEndpoint::parse_v4("10.0.0.5:51820").unwrap();
        assert_eq!(ep.port(), 51820);
        assert_eq!(ep.to_string(), "10.0.0.5:51820");
        assert!(ep.is_routable());
        assert!(ClusterEndpoint::parse_v4("10.0.0.5:0").is_err());
        assert!(ClusterEndpoint::parse_v4("10.0.0.5").is_err());
    }

    #[test]
    fn list_sorts_and_dedups() {
        let mut list = EndpointList::new();
        assert!(list.insert(ClusterEndpoint::parse_v4("10.0.0.5:51820").unwrap()));
        assert!(list.insert(ClusterEndpoint::parse_v4("10.0.0.1:51820").unwrap()));
        // duplicate -> no insert
        assert!(!list.insert(ClusterEndpoint::parse_v4("10.0.0.5:51820").unwrap()));
        assert_eq!(list.len(), 2);
        // sorted: 10.0.0.1 before 10.0.0.5
        assert_eq!(
            list.as_slice()[0].addr(),
            NodeAddress::parse_v4("10.0.0.1").unwrap()
        );
    }

    #[test]
    fn routable_filters_loopback() {
        let list = EndpointList::from_iter([
            ClusterEndpoint::parse_v4("127.0.0.1:51820").unwrap(),
            ClusterEndpoint::parse_v4("192.168.1.1:51820").unwrap(),
        ]);
        assert_eq!(list.routable().len(), 1);
    }

    #[test]
    fn ipv6_is_bracketed() {
        let ep =
            ClusterEndpoint::new(NodeAddress::V6([0xfd00, 0, 0, 0, 0, 0, 0, 1]), 51820).unwrap();
        assert!(ep.to_string().starts_with('['));
    }
}
