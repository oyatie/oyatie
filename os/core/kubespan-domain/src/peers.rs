//! KubeSpan peers and the peer-status state machine.
//!
//! Mirrors `pkg/machinery/resources/kubespan.PeerSpec` / `PeerStatus` and the
//! `PeerSpecController` + `PeerStatusController` in
//! `internal/app/machined/pkg/controllers/kubespan`.
//!
//! A `PeerSpec` is derived from a cluster affiliate: its WireGuard public key,
//! KubeSpan address, advertised endpoints and label. The `PeerStatusController`
//! folds WireGuard handshake observations into a connection state machine
//! (`Unknown -> Up -> Down`) and decides when to rotate endpoints.

use crate::endpoints::{Endpoint, EndpointSet};
use crate::wireguard_spec::WireguardKey;
use alloc::string::String;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// How long without a handshake before a peer is considered down (seconds).
///
/// Matches Talos's `PeerDownInterval` default.
pub const PEER_DOWN_INTERVAL_SECS: u64 = 5 * 60;

/// The connection state of a KubeSpan peer.
///
/// Mirrors `kubespan.PeerState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    /// No handshake has ever been observed.
    Unknown,
    /// A recent handshake confirms the tunnel is up.
    Up,
    /// A handshake was seen before but has since gone stale.
    Down,
}

impl PeerState {
    /// The lowercase label Talos uses for this state.
    pub fn as_str(&self) -> &'static str {
        match self {
            PeerState::Unknown => "unknown",
            PeerState::Up => "up",
            PeerState::Down => "down",
        }
    }
}

/// A KubeSpan peer specification derived from a cluster affiliate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerSpec {
    /// The peer's WireGuard public key (the peer's KubeSpan identity).
    pub public_key: WireguardKey,
    /// A human-readable label (the affiliate's nodename), for diagnostics.
    pub label: String,
    /// The peer's KubeSpan overlay address, routed as a `/128` allowed-ip.
    pub address: NodeAddress,
    /// Additional allowed subnets advertised by the peer (e.g. pod CIDRs).
    pub additional_addresses: EndpointSet,
    /// Candidate endpoints the peer can be reached on.
    pub endpoints: EndpointSet,
}

impl PeerSpec {
    /// Construct a peer spec.
    pub fn new(
        public_key: WireguardKey,
        label: impl Into<String>,
        address: NodeAddress,
    ) -> Result<Self> {
        let label = label.into();
        if label.is_empty() {
            return Err(Error::invalid("peer label is empty"));
        }
        Ok(PeerSpec {
            public_key,
            label,
            address,
            additional_addresses: EndpointSet::new(),
            endpoints: EndpointSet::new(),
        })
    }

    /// Add a candidate endpoint.
    pub fn add_endpoint(&mut self, ep: Endpoint) -> bool {
        self.endpoints.insert(ep)
    }

    /// Whether this peer advertises any routable endpoint.
    pub fn is_reachable(&self) -> bool {
        self.endpoints.preferred().is_some()
    }
}

/// Live status of a KubeSpan peer: its state machine and connection metrics.
///
/// Mirrors `kubespan.PeerStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerStatus {
    /// The peer's public key.
    pub public_key: WireguardKey,
    /// Current connection state.
    pub state: PeerState,
    /// Tick of the last observed successful handshake (0 = never).
    pub last_handshake_tick: u64,
    /// The endpoint the peer is currently using, if known.
    pub endpoint: Option<Endpoint>,
    /// Total bytes received from the peer.
    pub rx_bytes: u64,
    /// Total bytes sent to the peer.
    pub tx_bytes: u64,
    /// How many times the manager has rotated this peer's endpoint.
    pub endpoint_rotations: u32,
}

impl PeerStatus {
    /// A fresh status in the `Unknown` state.
    pub fn new(public_key: WireguardKey) -> Self {
        PeerStatus {
            public_key,
            state: PeerState::Unknown,
            last_handshake_tick: 0,
            endpoint: None,
            rx_bytes: 0,
            tx_bytes: 0,
            endpoint_rotations: 0,
        }
    }
}

/// Folds WireGuard handshake observations into peer status transitions.
///
/// This is the in-memory model of the `PeerStatusController` reconcile loop. It
/// is driven by `tick` (a logical clock) and per-peer handshake reports.
#[derive(Debug, Default)]
pub struct PeerStatusController {
    down_interval: u64,
}

/// A WireGuard handshake report for a single peer at the current tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandshakeReport {
    /// Tick of the most recent handshake, or `None` if there has been none.
    pub last_handshake_tick: Option<u64>,
    /// The endpoint the peer is currently using, if any.
    pub endpoint: Option<Endpoint>,
    /// Cumulative receive/transmit byte counters from the kernel.
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

impl PeerStatusController {
    /// New controller using the default down interval.
    pub fn new() -> Self {
        PeerStatusController {
            down_interval: PEER_DOWN_INTERVAL_SECS,
        }
    }

    /// New controller with a custom down interval (ticks).
    pub fn with_down_interval(down_interval: u64) -> Self {
        PeerStatusController { down_interval }
    }

    /// Reconcile one peer's status given a handshake report at `now`.
    ///
    /// Returns `true` if the peer's state changed. Implements the Talos
    /// transition rules:
    /// - never handshaked -> `Unknown`
    /// - handshaked within `down_interval` -> `Up`
    /// - last handshake older than `down_interval` -> `Down`
    ///
    /// A change of endpoint while Up bumps the rotation counter.
    pub fn reconcile(&self, status: &mut PeerStatus, report: &HandshakeReport, now: u64) -> bool {
        let prev_state = status.state;
        let prev_endpoint = status.endpoint;

        status.rx_bytes = report.rx_bytes;
        status.tx_bytes = report.tx_bytes;

        let new_state = match report.last_handshake_tick {
            None => PeerState::Unknown,
            Some(t) => {
                status.last_handshake_tick = t;
                if now.saturating_sub(t) <= self.down_interval {
                    PeerState::Up
                } else {
                    PeerState::Down
                }
            }
        };
        status.state = new_state;

        if let Some(ep) = report.endpoint {
            // Count an endpoint rotation only between two known endpoints.
            if let Some(old) = prev_endpoint
                && old != ep
            {
                status.endpoint_rotations += 1;
            }
            status.endpoint = Some(ep);
        }

        prev_state != new_state || prev_endpoint != status.endpoint
    }

    /// Whether a peer in this state should have its endpoint rotated by the
    /// manager (the manager cycles candidates while a peer stays Down).
    pub fn should_rotate_endpoint(&self, status: &PeerStatus) -> bool {
        matches!(status.state, PeerState::Down | PeerState::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoints::Endpoint;

    fn key(seed: &str) -> WireguardKey {
        WireguardKey::derive_from_seed(seed)
    }

    fn addr() -> NodeAddress {
        NodeAddress::V6([0xfd00, 0, 0, 0, 0, 0, 0, 1])
    }

    fn ep(s: &str) -> Endpoint {
        Endpoint::parse_v4(s).unwrap()
    }

    #[test]
    fn peer_spec_requires_label() {
        assert!(PeerSpec::new(key("a"), "node-a", addr()).is_ok());
        assert!(PeerSpec::new(key("a"), "", addr()).is_err());
    }

    #[test]
    fn peer_reachability_tracks_endpoints() {
        let mut p = PeerSpec::new(key("a"), "node-a", addr()).unwrap();
        assert!(!p.is_reachable());
        p.add_endpoint(ep("127.0.0.1:51820"));
        assert!(!p.is_reachable()); // only loopback
        p.add_endpoint(ep("203.0.113.1:51820"));
        assert!(p.is_reachable());
    }

    #[test]
    fn state_strings() {
        assert_eq!(PeerState::Unknown.as_str(), "unknown");
        assert_eq!(PeerState::Up.as_str(), "up");
        assert_eq!(PeerState::Down.as_str(), "down");
    }

    #[test]
    fn reconcile_unknown_to_up_to_down() {
        let c = PeerStatusController::with_down_interval(100);
        let mut st = PeerStatus::new(key("a"));
        assert_eq!(st.state, PeerState::Unknown);

        // no handshake -> stays Unknown.
        let changed = c.reconcile(
            &mut st,
            &HandshakeReport {
                last_handshake_tick: None,
                endpoint: None,
                rx_bytes: 0,
                tx_bytes: 0,
            },
            50,
        );
        assert!(!changed);
        assert_eq!(st.state, PeerState::Unknown);

        // fresh handshake -> Up.
        let changed = c.reconcile(
            &mut st,
            &HandshakeReport {
                last_handshake_tick: Some(40),
                endpoint: Some(ep("203.0.113.1:51820")),
                rx_bytes: 10,
                tx_bytes: 20,
            },
            50,
        );
        assert!(changed);
        assert_eq!(st.state, PeerState::Up);
        assert_eq!(st.rx_bytes, 10);

        // stale handshake -> Down.
        let changed = c.reconcile(
            &mut st,
            &HandshakeReport {
                last_handshake_tick: Some(40),
                endpoint: Some(ep("203.0.113.1:51820")),
                rx_bytes: 10,
                tx_bytes: 20,
            },
            200,
        );
        assert!(changed);
        assert_eq!(st.state, PeerState::Down);
    }

    #[test]
    fn endpoint_rotation_counted() {
        let c = PeerStatusController::with_down_interval(100);
        let mut st = PeerStatus::new(key("a"));
        c.reconcile(
            &mut st,
            &HandshakeReport {
                last_handshake_tick: Some(10),
                endpoint: Some(ep("203.0.113.1:51820")),
                rx_bytes: 0,
                tx_bytes: 0,
            },
            10,
        );
        assert_eq!(st.endpoint_rotations, 0);
        // endpoint changes -> rotation.
        c.reconcile(
            &mut st,
            &HandshakeReport {
                last_handshake_tick: Some(20),
                endpoint: Some(ep("198.51.100.9:51820")),
                rx_bytes: 0,
                tx_bytes: 0,
            },
            20,
        );
        assert_eq!(st.endpoint_rotations, 1);
        // same endpoint -> no rotation.
        c.reconcile(
            &mut st,
            &HandshakeReport {
                last_handshake_tick: Some(30),
                endpoint: Some(ep("198.51.100.9:51820")),
                rx_bytes: 0,
                tx_bytes: 0,
            },
            30,
        );
        assert_eq!(st.endpoint_rotations, 1);
    }

    #[test]
    fn rotation_decision_by_state() {
        let c = PeerStatusController::new();
        let mut st = PeerStatus::new(key("a"));
        st.state = PeerState::Up;
        assert!(!c.should_rotate_endpoint(&st));
        st.state = PeerState::Down;
        assert!(c.should_rotate_endpoint(&st));
        st.state = PeerState::Unknown;
        assert!(c.should_rotate_endpoint(&st));
    }
}
