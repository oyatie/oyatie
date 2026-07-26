//! The KubeSpan manager controller.
//!
//! Mirrors the `ManagerController` in
//! `internal/app/machined/pkg/controllers/kubespan`: it owns the local
//! [`KubeSpanIdentity`], reconciles the set of [`PeerSpec`]s derived from
//! cluster affiliates into a [`WireguardDeviceSpec`], drives per-peer
//! [`PeerStatus`] from handshake reports, and pushes the device spec to the
//! kernel through the [`WireguardDevice`] OS boundary.
//!
//! The OS boundary is a trait so tests use an in-memory device; a real build
//! would back it with netlink/`wgctrl`.

use crate::endpoints::{Endpoint, EndpointController, EndpointObservation};
use crate::identity::KubeSpanIdentity;
use crate::peers::{HandshakeReport, PeerSpec, PeerStatus, PeerStatusController};
use crate::wireguard_spec::{AllowedIp, WireguardDeviceSpec, WireguardPeerSpec};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use os_kernel::error::{Error, Result};
use os_kernel::id::Fingerprint;

/// OS boundary for applying a WireGuard device configuration.
///
/// Real implementations program the kernel interface (via netlink/wgctrl); the
/// in-memory [`InMemoryWireguardDevice`] records the last applied spec so the
/// reconcile loop can be tested deterministically.
pub trait WireguardDevice {
    /// Apply a device spec to the interface. Returns whether the spec changed.
    fn apply(&mut self, spec: &WireguardDeviceSpec) -> Result<bool>;

    /// The fingerprint of the currently-applied spec, if any.
    fn current_fingerprint(&self) -> Option<Fingerprint>;
}

/// In-memory [`WireguardDevice`] used by tests and for modeling.
#[derive(Debug, Default)]
pub struct InMemoryWireguardDevice {
    applied: Option<WireguardDeviceSpec>,
    apply_count: u32,
}

impl InMemoryWireguardDevice {
    /// A device with nothing applied yet.
    pub fn new() -> Self {
        InMemoryWireguardDevice {
            applied: None,
            apply_count: 0,
        }
    }

    /// The last spec applied, if any.
    pub fn applied(&self) -> Option<&WireguardDeviceSpec> {
        self.applied.as_ref()
    }

    /// How many times a *changed* spec was applied.
    pub fn apply_count(&self) -> u32 {
        self.apply_count
    }
}

impl WireguardDevice for InMemoryWireguardDevice {
    fn apply(&mut self, spec: &WireguardDeviceSpec) -> Result<bool> {
        spec.validate()?;
        let changed = self
            .applied
            .as_ref()
            .is_none_or(|cur| cur.fingerprint() != spec.fingerprint());
        if changed {
            self.applied = Some(spec.clone());
            self.apply_count += 1;
        }
        Ok(changed)
    }

    fn current_fingerprint(&self) -> Option<Fingerprint> {
        self.applied.as_ref().map(|s| s.fingerprint())
    }
}

/// Whether KubeSpan is enabled and how it advertises endpoints.
///
/// Mirrors the relevant `KubeSpanConfig` fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerConfig {
    /// Master switch; when false the manager applies an empty peer set.
    pub enabled: bool,
    /// Advertise the node's own non-KubeSpan addresses as endpoints.
    pub advertise_kubernetes_networks: bool,
    /// Force the routing of all KubeSpan traffic even for the same subnet.
    pub force_routing: bool,
    /// Persistent-keepalive applied to every peer (seconds, 0 = disabled).
    pub keepalive_secs: u32,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        ManagerConfig {
            enabled: true,
            advertise_kubernetes_networks: false,
            force_routing: true,
            keepalive_secs: 25,
        }
    }
}

/// The KubeSpan manager: owns identity, peers, statuses, and the device spec.
pub struct KubeSpanManager {
    identity: KubeSpanIdentity,
    config: ManagerConfig,
    peers: BTreeMap<u64, PeerSpec>,
    statuses: BTreeMap<u64, PeerStatus>,
    status_controller: PeerStatusController,
    endpoint_controller: EndpointController,
}

impl KubeSpanManager {
    /// Construct a manager for a node identity and config.
    pub fn new(identity: KubeSpanIdentity, config: ManagerConfig) -> Self {
        KubeSpanManager {
            identity,
            config,
            peers: BTreeMap::new(),
            statuses: BTreeMap::new(),
            status_controller: PeerStatusController::new(),
            endpoint_controller: EndpointController::new(),
        }
    }

    /// The local node's identity.
    pub fn identity(&self) -> &KubeSpanIdentity {
        &self.identity
    }

    fn peer_index(spec: &PeerSpec) -> u64 {
        EndpointController::peer_index(&spec.public_key)
    }

    /// Add or replace a peer derived from a cluster affiliate. Returns the
    /// peer's compact index.
    pub fn upsert_peer(&mut self, spec: PeerSpec) -> u64 {
        let idx = Self::peer_index(&spec);
        self.statuses
            .entry(idx)
            .or_insert_with(|| PeerStatus::new(spec.public_key.clone()));
        self.peers.insert(idx, spec);
        idx
    }

    /// Remove a peer (affiliate left the cluster). Returns whether it existed.
    pub fn remove_peer(&mut self, idx: u64) -> bool {
        let existed = self.peers.remove(&idx).is_some();
        self.statuses.remove(&idx);
        self.endpoint_controller.forget(idx);
        existed
    }

    /// Number of known peers.
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Look up a peer's live status.
    pub fn status(&self, idx: u64) -> Option<&PeerStatus> {
        self.statuses.get(&idx)
    }

    /// Feed a per-peer handshake report into the status state machine and the
    /// endpoint controller. Returns whether the peer's status changed.
    pub fn observe_handshake(
        &mut self,
        idx: u64,
        report: HandshakeReport,
        now: u64,
    ) -> Result<bool> {
        let status = self
            .statuses
            .get_mut(&idx)
            .ok_or_else(|| Error::not_found("unknown peer index"))?;
        let changed = self.status_controller.reconcile(status, &report, now);

        // Pin the last-known-good endpoint when a handshake actually landed.
        if let (Some(ep), Some(t)) = (report.endpoint, report.last_handshake_tick) {
            self.endpoint_controller.observe(EndpointObservation {
                peer_key: idx,
                endpoint: ep,
                observed_at_tick: t,
            });
        }
        Ok(changed)
    }

    /// Choose the endpoint for a peer: its last-known-good endpoint, else the
    /// preferred routable candidate from its advertised set.
    pub fn select_endpoint(&self, idx: u64) -> Option<Endpoint> {
        let spec = self.peers.get(&idx)?;
        let candidates = self
            .endpoint_controller
            .reconcile_candidates(idx, &spec.endpoints);
        candidates.into_iter().next()
    }

    /// Build the WireGuard device spec from the current identity and peers.
    ///
    /// When KubeSpan is disabled the device keeps only the local key material
    /// and no peers. Each enabled peer becomes a [`WireguardPeerSpec`] with the
    /// peer's `/128` overlay address (and any advertised subnets) as allowed-ips
    /// and the selected endpoint.
    pub fn build_device_spec(&self) -> WireguardDeviceSpec {
        let mut device = WireguardDeviceSpec::new(self.identity.private_key().clone());

        if !self.config.enabled {
            return device;
        }

        let mut peers: Vec<WireguardPeerSpec> = Vec::with_capacity(self.peers.len());
        for (idx, spec) in &self.peers {
            let mut wg = WireguardPeerSpec::new(spec.public_key.clone())
                .with_keepalive(self.config.keepalive_secs);

            // Route the peer's overlay /128.
            wg.add_allowed_ip(AllowedIp::host_route(spec.address));
            // Plus any advertised additional subnets.
            for extra in spec.additional_addresses.iter() {
                wg.add_allowed_ip(AllowedIp::host_route(extra.addr()));
            }

            if let Some(ep) = self.select_endpoint(*idx) {
                wg = wg.with_endpoint(ep.addr(), ep.port());
            }
            peers.push(wg);
        }

        device.set_peers(peers);
        device
    }

    /// Reconcile: build the device spec and push it through the OS boundary.
    /// Returns whether the applied spec changed.
    pub fn reconcile<D: WireguardDevice>(&self, device: &mut D) -> Result<bool> {
        let spec = self.build_device_spec();
        device.apply(&spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoints::Endpoint;
    use crate::peers::PeerState;
    use crate::wireguard_spec::WireguardKey;
    use os_kernel::address::NodeAddress;

    fn manager() -> KubeSpanManager {
        let id = KubeSpanIdentity::generate("cluster-1", "self-node").unwrap();
        KubeSpanManager::new(id, ManagerConfig::default())
    }

    fn peer(seed: &str, label: &str) -> PeerSpec {
        let id = KubeSpanIdentity::generate("cluster-1", seed).unwrap();
        let mut p = PeerSpec::new(id.public_key().clone(), label, id.address()).unwrap();
        p.add_endpoint(Endpoint::parse_v4("203.0.113.10:51820").unwrap());
        p
    }

    #[test]
    fn upsert_and_remove_peer() {
        let mut m = manager();
        let idx = m.upsert_peer(peer("node-b", "node-b"));
        assert_eq!(m.peer_count(), 1);
        assert!(m.status(idx).is_some());
        assert_eq!(m.status(idx).unwrap().state, PeerState::Unknown);
        assert!(m.remove_peer(idx));
        assert_eq!(m.peer_count(), 0);
        assert!(!m.remove_peer(idx));
    }

    #[test]
    fn device_spec_includes_peer_allowed_ips_and_endpoint() {
        let mut m = manager();
        m.upsert_peer(peer("node-b", "node-b"));
        let spec = m.build_device_spec();
        assert_eq!(spec.peers.len(), 1);
        let p = &spec.peers[0];
        assert!(!p.allowed_ips.is_empty());
        assert_eq!(p.endpoint.unwrap().1, 51820);
        assert_eq!(p.persistent_keepalive_secs, 25);
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn disabled_kubespan_emits_no_peers() {
        let id = KubeSpanIdentity::generate("cluster-1", "self").unwrap();
        let cfg = ManagerConfig {
            enabled: false,
            ..Default::default()
        };
        let mut m = KubeSpanManager::new(id, cfg);
        m.upsert_peer(peer("node-b", "node-b"));
        let spec = m.build_device_spec();
        assert!(spec.peers.is_empty());
    }

    #[test]
    fn reconcile_is_idempotent_until_change() {
        let mut m = manager();
        m.upsert_peer(peer("node-b", "node-b"));
        let mut dev = InMemoryWireguardDevice::new();
        assert!(m.reconcile(&mut dev).unwrap()); // first apply -> changed
        assert!(!m.reconcile(&mut dev).unwrap()); // identical -> no change
        assert_eq!(dev.apply_count(), 1);
        // adding a peer changes the spec.
        m.upsert_peer(peer("node-c", "node-c"));
        assert!(m.reconcile(&mut dev).unwrap());
        assert_eq!(dev.apply_count(), 2);
    }

    #[test]
    fn handshake_drives_status_and_endpoint_pinning() {
        let mut m = manager();
        let idx = m.upsert_peer(peer("node-b", "node-b"));
        let ep = Endpoint::parse_v4("198.51.100.50:51820").unwrap();
        let changed = m
            .observe_handshake(
                idx,
                HandshakeReport {
                    last_handshake_tick: Some(100),
                    endpoint: Some(ep),
                    rx_bytes: 64,
                    tx_bytes: 128,
                },
                100,
            )
            .unwrap();
        assert!(changed);
        assert_eq!(m.status(idx).unwrap().state, PeerState::Up);
        // last-known-good endpoint should now be selected over the advertised one.
        assert_eq!(m.select_endpoint(idx).unwrap(), ep);
    }

    #[test]
    fn observe_unknown_peer_errors() {
        let mut m = manager();
        let err = m
            .observe_handshake(
                999,
                HandshakeReport {
                    last_handshake_tick: None,
                    endpoint: None,
                    rx_bytes: 0,
                    tx_bytes: 0,
                },
                1,
            )
            .unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn additional_addresses_become_allowed_ips() {
        let mut m = manager();
        let kid = KubeSpanIdentity::generate("cluster-1", "node-d").unwrap();
        let mut p = PeerSpec::new(kid.public_key().clone(), "node-d", kid.address()).unwrap();
        // advertise a pod address as an extra allowed-ip (modeled via endpoint addr).
        p.additional_addresses
            .insert(Endpoint::new(NodeAddress::parse_v4("10.244.3.0").unwrap(), 1).unwrap());
        let idx = m.upsert_peer(p);
        let spec = m.build_device_spec();
        let wg = spec
            .peer(&WireguardKey::derive_from_seed("node-d:public"))
            .unwrap();
        // overlay /128 + extra host route.
        assert!(wg.allowed_ips.len() >= 2);
        let _ = idx;
    }
}
