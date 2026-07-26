#![cfg_attr(not(test), no_std)]
//! # talos-kubespan
//!
//! Implements KubeSpan, Talos's WireGuard-based encrypted cluster mesh. Mirrors
//! `pkg/kubespan` and `internal/app/machined/pkg/controllers/kubespan` from
//! `siderolabs/talos`:
//!
//! - [`wireguard_spec`]: WireGuard keys, allowed-ips, peer config and the
//!   device spec the manager programs into the kernel.
//! - [`identity`]: the node's KubeSpan identity (key pair + derived ULA IPv6
//!   overlay address), as produced by the `IdentityController`.
//! - [`endpoints`]: endpoint primitives and the `EndpointController`'s NAT-aware
//!   last-known-good endpoint reconciliation.
//! - [`peers`]: affiliate-derived `PeerSpec`s and the `PeerStatusController`
//!   connection state machine (`Unknown -> Up -> Down`).
//! - [`manager`]: the `ManagerController` that owns identity, peers, statuses
//!   and pushes the assembled device spec through the
//!   [`manager::WireguardDevice`] OS boundary.
//!
//! OS boundaries (the WireGuard interface) are modeled as traits with in-memory
//! implementations used by the tests. The crate is `no_std` for real builds and
//! only uses the `alloc` crate plus an internal path dependency on `talos-core`;
//! under `cargo test` it links against `std` on the host.

// Pedantic lints that add churn without improving the API for this crate:
// `must_use` on every getter/builder, `# Errors` doc sections on `Result`
// returns, and backticking every type name in prose. The project as a whole
// opts out of these (see sibling crates).
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]

extern crate alloc;

pub mod endpoints;
pub mod identity;
pub mod manager;
pub mod peers;
pub mod wireguard_spec;

pub use endpoints::{Endpoint, EndpointController, EndpointObservation, EndpointSet};
pub use identity::{KubeSpanIdentity, overlay_cidr};
pub use manager::{InMemoryWireguardDevice, KubeSpanManager, ManagerConfig, WireguardDevice};
pub use peers::{
    HandshakeReport, PEER_DOWN_INTERVAL_SECS, PeerSpec, PeerState, PeerStatus, PeerStatusController,
};
pub use wireguard_spec::{
    AllowedIp, DEFAULT_LISTEN_PORT, FIREWALL_MARK, KEY_B64_LEN, KEY_BYTES, WireguardDeviceSpec,
    WireguardKey, WireguardPeerSpec,
};

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::address::NodeAddress;

    /// End-to-end: two nodes form a mesh, exchange identities, and the manager
    /// programs a device spec whose peer matches the other node's identity.
    #[test]
    fn two_node_mesh_end_to_end() {
        let a = KubeSpanIdentity::generate("cluster-x", "node-a").unwrap();
        let b = KubeSpanIdentity::generate("cluster-x", "node-b").unwrap();

        let mut mgr = KubeSpanManager::new(a.clone(), ManagerConfig::default());

        let mut peer_b = PeerSpec::new(b.public_key().clone(), "node-b", b.address()).unwrap();
        peer_b.add_endpoint(Endpoint::parse_v4("203.0.113.20:51820").unwrap());
        let idx = mgr.upsert_peer(peer_b);

        let mut device = InMemoryWireguardDevice::new();
        assert!(mgr.reconcile(&mut device).unwrap());

        let spec = device.applied().unwrap();
        assert_eq!(spec.peers.len(), 1);
        // the lone peer is node-b.
        assert_eq!(spec.peers[0].public_key, *b.public_key());
        // node-a's own private key drives the device.
        assert_eq!(spec.private_key, *a.private_key());

        // node-b confirms a handshake; status goes Up.
        mgr.observe_handshake(
            idx,
            HandshakeReport {
                last_handshake_tick: Some(10),
                endpoint: Some(Endpoint::parse_v4("203.0.113.20:51820").unwrap()),
                rx_bytes: 1,
                tx_bytes: 1,
            },
            10,
        )
        .unwrap();
        assert_eq!(mgr.status(idx).unwrap().state, PeerState::Up);
    }

    #[test]
    fn reexports_are_usable() {
        // Touch a representative slice of the public surface.
        let k = WireguardKey::derive_from_seed("x");
        let _ = AllowedIp::host_route(NodeAddress::parse_v4("10.0.0.1").unwrap());
        let _ = WireguardDeviceSpec::new(k);
        assert_eq!(DEFAULT_LISTEN_PORT, 51820);
        assert_eq!(KEY_BYTES, 32);
        assert_eq!(KEY_B64_LEN, 44);
        assert_eq!(FIREWALL_MARK, 0x51820);
        assert_eq!(PEER_DOWN_INTERVAL_SECS, 300);
    }
}
