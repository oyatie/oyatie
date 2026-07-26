//! Cluster membership and discovery affiliates.
//!
//! Mirrors the Talos discovery data model (`pkg/machinery/resources/cluster`
//! `Affiliate`/`Member` and the `cluster` membership controller). The discovery
//! service exchanges **affiliates** — the self-published facts a node knows
//! about itself (identity, hostname, addresses, KubeSpan key/endpoints). The
//! local controller reconciles the affiliates it learns into **members** and,
//! for KubeSpan, into programmable [`WireguardPeer`]s.
//!
//! The flow modeled here:
//! 1. A node builds its own [`Affiliate`] and publishes it.
//! 2. Peers' affiliates arrive and are stored in a [`Membership`] registry,
//!    keyed by node [`Identity`].
//! 3. The registry promotes affiliates to [`Member`]s, tracking a small state
//!    machine (`Discovered -> Confirmed -> Stale`), and projects KubeSpan peers.

use std::collections::BTreeMap;

use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};
use os_kernel::machine_type::MachineType;

use crate::affiliate::AffiliateData;
use crate::endpoint::EndpointList;
use crate::identity::Identity;
use crate::wireguard::{AllowedIp, WireguardKey, WireguardPeer};

/// The KubeSpan-related facts an affiliate advertises: its public key plus the
/// additional endpoints it can be reached on for the encrypted overlay.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KubeSpanInfo {
    public_key: Option<WireguardKey>,
    /// The node's KubeSpan overlay address (the `AllowedIP` peers route to it).
    address: Option<AllowedIp>,
    additional_endpoints: EndpointList,
}

impl KubeSpanInfo {
    /// An empty (KubeSpan-disabled) info block.
    pub fn disabled() -> Self {
        KubeSpanInfo::default()
    }

    /// Whether KubeSpan is enabled for this affiliate (has a public key).
    pub fn is_enabled(&self) -> bool {
        self.public_key.is_some()
    }

    /// The advertised public key, if any.
    pub fn public_key(&self) -> Option<&WireguardKey> {
        self.public_key.as_ref()
    }

    /// The KubeSpan overlay address.
    pub fn address(&self) -> Option<AllowedIp> {
        self.address
    }

    /// The advertised KubeSpan endpoints.
    pub fn endpoints(&self) -> &EndpointList {
        &self.additional_endpoints
    }
}

/// A self-published description of a node, as exchanged through the discovery
/// service. This is the unit of membership: every node publishes exactly one
/// affiliate for itself, and learns one per peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Affiliate {
    identity: Identity,
    hostname: String,
    machine_type: MachineType,
    /// Routable node addresses (the cluster's regular network, not KubeSpan).
    addresses: Vec<NodeAddress>,
    kubespan: KubeSpanInfo,
}

impl Affiliate {
    /// Start building an affiliate for `identity` with the given hostname/type.
    pub fn new(identity: Identity, hostname: impl Into<String>, machine_type: MachineType) -> Self {
        Affiliate {
            identity,
            hostname: hostname.into(),
            machine_type,
            addresses: Vec::new(),
            kubespan: KubeSpanInfo::disabled(),
        }
    }

    /// Advertise a node address (deduplicated, kept sorted for a stable
    /// fingerprint, mirroring Talos's canonical affiliate encoding).
    pub fn add_address(&mut self, addr: NodeAddress) -> &mut Self {
        if let Err(pos) = self
            .addresses
            .binary_search_by(|a| crate::addr_sort_key(a).cmp(&crate::addr_sort_key(&addr)))
        {
            self.addresses.insert(pos, addr);
        }
        self
    }

    /// Enable KubeSpan on this affiliate.
    pub fn enable_kubespan(
        &mut self,
        public_key: WireguardKey,
        address: AllowedIp,
        endpoints: EndpointList,
    ) -> &mut Self {
        self.kubespan = KubeSpanInfo {
            public_key: Some(public_key),
            address: Some(address),
            additional_endpoints: endpoints,
        };
        self
    }

    /// The node identity (the affiliate key).
    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    /// The advertised hostname.
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    /// The node's machine type.
    pub fn machine_type(&self) -> MachineType {
        self.machine_type
    }

    /// The advertised, sorted node addresses.
    pub fn addresses(&self) -> &[NodeAddress] {
        &self.addresses
    }

    /// The KubeSpan info block.
    pub fn kubespan(&self) -> &KubeSpanInfo {
        &self.kubespan
    }

    /// Whether this affiliate carries enough information to be a useful member:
    /// it must advertise at least one routable address.
    pub fn is_publishable(&self) -> bool {
        self.addresses.iter().any(|a| !a.is_loopback())
    }

    /// Flatten this local affiliate into the [`AffiliateData`] wire form the
    /// discovery service exchanges. KubeSpan key and endpoints are carried over;
    /// the overlay address is not part of the flat form (peers recompute it).
    pub fn to_data(&self) -> AffiliateData {
        let mut data = AffiliateData::new(self.identity.as_str(), self.hostname.clone())
            .with_machine_type(self.machine_type);
        for addr in &self.addresses {
            data.add_address(*addr);
        }
        if let Some(key) = self.kubespan.public_key() {
            data = data.with_kubespan(
                key.clone(),
                self.kubespan.endpoints().as_slice().iter().copied(),
            );
        }
        data
    }

    /// Reconstruct a local [`Affiliate`] from the discovery wire form. The
    /// affiliate id must be a valid [`Identity`]. KubeSpan facts are restored,
    /// but because the flat form omits the overlay address, the rebuilt affiliate
    /// carries a KubeSpan key/endpoints only when `overlay` is supplied.
    pub fn from_data(data: &AffiliateData, overlay: Option<AllowedIp>) -> Result<Self> {
        let identity = Identity::new(data.affiliate_id())?;
        let mut aff = Affiliate::new(identity, data.hostname().to_string(), data.machine_type());
        for addr in data.addresses() {
            aff.add_address(*addr);
        }
        if let (Some(key), Some(address)) = (data.kubespan_key(), overlay) {
            let endpoints: EndpointList = data.kubespan_endpoints().iter().copied().collect();
            aff.enable_kubespan(key.clone(), address, endpoints);
        }
        Ok(aff)
    }

    /// Project this affiliate into a KubeSpan [`WireguardPeer`], if it advertises
    /// a key, an overlay address, and a routable endpoint. Returns `None` when
    /// KubeSpan is disabled or the peer would not be programmable.
    pub fn to_wireguard_peer(&self, keepalive_secs: u16) -> Option<WireguardPeer> {
        let key = self.kubespan.public_key.clone()?;
        let address = self.kubespan.address?;
        let endpoint = self
            .kubespan
            .additional_endpoints
            .routable()
            .into_iter()
            .next()?;
        let mut peer = WireguardPeer::new(key)
            .with_endpoint(endpoint)
            .with_keepalive(keepalive_secs);
        peer.add_allowed_ip(address);
        Some(peer)
    }
}

/// The lifecycle state of a learned member, reconciled from discovery updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemberState {
    /// Seen via discovery but not yet corroborated by a second update.
    Discovered,
    /// Seen at least twice; treated as a live cluster member.
    Confirmed,
    /// Previously confirmed but missing from recent discovery snapshots.
    Stale,
}

/// A reconciled cluster member: the affiliate plus controller-tracked state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    affiliate: Affiliate,
    state: MemberState,
    /// How many discovery updates have referenced this member.
    observations: u32,
}

impl Member {
    fn new(affiliate: Affiliate) -> Self {
        Member {
            affiliate,
            state: MemberState::Discovered,
            observations: 1,
        }
    }

    /// The underlying affiliate.
    pub fn affiliate(&self) -> &Affiliate {
        &self.affiliate
    }

    /// The member's identity.
    pub fn identity(&self) -> &Identity {
        self.affiliate.identity()
    }

    /// The current lifecycle state.
    pub fn state(&self) -> MemberState {
        self.state
    }

    /// Number of discovery updates observed for this member.
    pub fn observations(&self) -> u32 {
        self.observations
    }

    /// Whether the member is a control-plane node.
    pub fn is_control_plane(&self) -> bool {
        self.affiliate.machine_type().is_control_plane()
    }
}

/// The local node's view of cluster membership: its own affiliate plus the
/// affiliates learned from the discovery service, reconciled into members.
///
/// This is the in-memory state the membership controller maintains. It is
/// deliberately decoupled from any transport: callers feed it discovery
/// snapshots via [`Membership::observe`] and [`Membership::mark_snapshot`].
#[derive(Debug, Clone)]
pub struct Membership {
    local: Identity,
    members: BTreeMap<String, Member>,
}

impl Membership {
    /// Create a membership view owned by `local`.
    pub fn new(local: Identity) -> Self {
        Membership {
            local,
            members: BTreeMap::new(),
        }
    }

    /// The local node's identity.
    pub fn local(&self) -> &Identity {
        &self.local
    }

    /// Observe an affiliate from a discovery update.
    ///
    /// - The local node's own affiliate is ignored (a node is not its own peer).
    /// - Unpublishable affiliates (no routable address) are rejected.
    /// - A new identity is added in `Discovered`; a repeat promotes the member
    ///   to `Confirmed` and refreshes its affiliate data.
    ///
    /// Returns the resulting state of the member, or an error if the affiliate
    /// was rejected.
    pub fn observe(&mut self, affiliate: Affiliate) -> Result<MemberState> {
        if affiliate.identity() == &self.local {
            return Err(Error::invalid_state("cannot observe self as a peer"));
        }
        if !affiliate.is_publishable() {
            return Err(Error::invalid("affiliate advertises no routable address"));
        }
        let key = affiliate.identity().as_str().to_string();
        let state = if let Some(existing) = self.members.get_mut(&key) {
            existing.affiliate = affiliate;
            existing.observations = existing.observations.saturating_add(1);
            if existing.state != MemberState::Confirmed {
                existing.state = MemberState::Confirmed;
            }
            existing.state
        } else {
            let member = Member::new(affiliate);
            let state = member.state;
            self.members.insert(key, member);
            state
        };
        Ok(state)
    }

    /// Observe an affiliate received in the discovery wire form. Convenience that
    /// rebuilds a local [`Affiliate`] (optionally restoring KubeSpan with the
    /// supplied overlay address) and feeds it to [`observe`](Self::observe).
    pub fn observe_data(
        &mut self,
        data: &AffiliateData,
        overlay: Option<AllowedIp>,
    ) -> Result<MemberState> {
        let aff = Affiliate::from_data(data, overlay)?;
        self.observe(aff)
    }

    /// Mark members not present in `present` (a discovery snapshot's identities)
    /// as [`MemberState::Stale`]. Returns the number of members newly marked
    /// stale. Mirrors the controller pruning members absent from a fresh
    /// discovery snapshot.
    pub fn mark_snapshot(&mut self, present: &[&Identity]) -> usize {
        let mut stale = 0;
        for member in self.members.values_mut() {
            let seen = present.iter().any(|id| *id == member.identity());
            if !seen && member.state != MemberState::Stale {
                member.state = MemberState::Stale;
                stale += 1;
            }
        }
        stale
    }

    /// Remove a member by identity, returning it if present.
    pub fn forget(&mut self, identity: &Identity) -> Option<Member> {
        self.members.remove(identity.as_str())
    }

    /// Look up a member by identity.
    pub fn member(&self, identity: &Identity) -> Option<&Member> {
        self.members.get(identity.as_str())
    }

    /// Total number of known members (any state).
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Whether no members are known.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// All members in identity order.
    pub fn members(&self) -> impl Iterator<Item = &Member> {
        self.members.values()
    }

    /// The confirmed members only.
    pub fn confirmed(&self) -> impl Iterator<Item = &Member> {
        self.members
            .values()
            .filter(|m| m.state == MemberState::Confirmed)
    }

    /// The number of confirmed control-plane members. Used to decide whether the
    /// control plane has quorum.
    pub fn control_plane_count(&self) -> usize {
        self.confirmed().filter(|m| m.is_control_plane()).count()
    }

    /// Project all confirmed, KubeSpan-enabled members into programmable
    /// [`WireguardPeer`]s, skipping any that are not fully programmable. The
    /// result is the peer set the KubeSpan controller would install.
    pub fn kubespan_peers(&self, keepalive_secs: u16) -> Vec<WireguardPeer> {
        self.confirmed()
            .filter_map(|m| m.affiliate().to_wireguard_peer(keepalive_secs))
            .filter(WireguardPeer::is_programmable)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::affiliate::AffiliateData;
    use crate::endpoint::ClusterEndpoint;

    fn affiliate(seed: &str, addr: &str, control_plane: bool) -> Affiliate {
        let identity = Identity::derive_from_seed(seed);
        let mt = if control_plane {
            MachineType::ControlPlane
        } else {
            MachineType::Worker
        };
        let mut a = Affiliate::new(identity, format!("{seed}.local"), mt);
        a.add_address(NodeAddress::parse_v4(addr).unwrap());
        a
    }

    fn with_kubespan(mut a: Affiliate, key_seed: &str, overlay: &str, endpoint: &str) -> Affiliate {
        let key = WireguardKey::derive_from_seed(key_seed);
        let address = AllowedIp::parse_v4(overlay).unwrap();
        let endpoints = EndpointList::from_iter([ClusterEndpoint::parse_v4(endpoint).unwrap()]);
        a.enable_kubespan(key, address, endpoints);
        a
    }

    #[test]
    fn observe_promotes_to_confirmed_on_repeat() {
        let local = Identity::derive_from_seed("local");
        let mut m = Membership::new(local);
        let peer = affiliate("peer-a", "10.0.0.2", false);

        assert_eq!(m.observe(peer.clone()).unwrap(), MemberState::Discovered);
        assert_eq!(m.observe(peer).unwrap(), MemberState::Confirmed);
        assert_eq!(m.len(), 1);
        assert_eq!(m.confirmed().count(), 1);
    }

    #[test]
    fn cannot_observe_self_or_unpublishable() {
        let local = Identity::derive_from_seed("local");
        let mut m = Membership::new(local.clone());

        // self
        let mut self_aff = Affiliate::new(local, "me.local", MachineType::Worker);
        self_aff.add_address(NodeAddress::parse_v4("10.0.0.1").unwrap());
        assert!(m.observe(self_aff).is_err());

        // loopback-only -> not publishable
        let mut lo = affiliate("loopback-peer", "127.0.0.1", false);
        // overwrite: the helper added a loopback only
        assert!(!lo.is_publishable());
        lo.add_address(NodeAddress::parse_v4("127.0.0.1").unwrap());
        assert!(m.observe(lo).is_err());
        assert!(m.is_empty());
    }

    #[test]
    fn snapshot_marks_absent_members_stale() {
        let mut m = Membership::new(Identity::derive_from_seed("local"));
        let a = affiliate("peer-a", "10.0.0.2", false);
        let b = affiliate("peer-b", "10.0.0.3", true);
        m.observe(a.clone()).unwrap();
        m.observe(b.clone()).unwrap();

        // snapshot only contains peer-a -> peer-b goes stale.
        let present = [a.identity()];
        assert_eq!(m.mark_snapshot(&present), 1);
        assert_eq!(m.member(b.identity()).unwrap().state(), MemberState::Stale);
        assert_eq!(
            m.member(a.identity()).unwrap().state(),
            MemberState::Discovered
        );
        // marking again is idempotent (already stale).
        assert_eq!(m.mark_snapshot(&present), 0);
    }

    #[test]
    fn control_plane_quorum_counts_confirmed_only() {
        let mut m = Membership::new(Identity::derive_from_seed("local"));
        let cp = affiliate("cp-1", "10.0.0.2", true);
        let worker = affiliate("w-1", "10.0.0.3", false);

        m.observe(cp.clone()).unwrap();
        // not yet confirmed -> not counted
        assert_eq!(m.control_plane_count(), 0);
        m.observe(cp).unwrap();
        m.observe(worker.clone()).unwrap();
        m.observe(worker).unwrap();
        assert_eq!(m.control_plane_count(), 1);
    }

    #[test]
    fn kubespan_peers_projected_from_confirmed_members() {
        let mut m = Membership::new(Identity::derive_from_seed("local"));

        // peer with KubeSpan + routable endpoint -> programmable
        let good = with_kubespan(
            affiliate("ks-good", "10.0.0.2", false),
            "ks-good-key",
            "10.244.0.1/32",
            "192.168.1.2:51820",
        );
        // peer with KubeSpan but only a loopback endpoint -> not programmable
        let bad = with_kubespan(
            affiliate("ks-bad", "10.0.0.3", false),
            "ks-bad-key",
            "10.244.0.2/32",
            "127.0.0.1:51820",
        );
        // peer without KubeSpan -> skipped
        let plain = affiliate("plain", "10.0.0.4", false);

        for a in [&good, &bad, &plain] {
            m.observe(a.clone()).unwrap();
            m.observe(a.clone()).unwrap();
        }

        let peers = m.kubespan_peers(25);
        assert_eq!(peers.len(), 1);
        assert!(peers[0].is_programmable());
        assert_eq!(peers[0].keepalive_secs(), 25);
        assert_eq!(
            peers[0].public_key(),
            &WireguardKey::derive_from_seed("ks-good-key")
        );
    }

    #[test]
    fn affiliate_to_data_and_back_round_trips() {
        let overlay = AllowedIp::parse_v4("10.244.0.7/32").unwrap();
        let original = with_kubespan(
            affiliate("rt", "10.0.0.9", true),
            "rt-key",
            "10.244.0.7/32",
            "192.168.1.9:51820",
        );
        let data = original.to_data();
        assert_eq!(data.affiliate_id(), original.identity().as_str());
        assert_eq!(data.hostname(), original.hostname());
        assert!(data.machine_type().is_control_plane());
        assert!(data.has_kubespan());
        assert_eq!(data.kubespan_endpoints().len(), 1);

        let rebuilt = Affiliate::from_data(&data, Some(overlay)).unwrap();
        assert_eq!(rebuilt.identity(), original.identity());
        assert_eq!(rebuilt.machine_type(), original.machine_type());
        assert_eq!(rebuilt.addresses(), original.addresses());
        assert!(rebuilt.kubespan().is_enabled());
        assert_eq!(rebuilt.kubespan().address(), Some(overlay));
    }

    #[test]
    fn from_data_without_overlay_drops_kubespan() {
        let original = with_kubespan(
            affiliate("rt2", "10.0.0.9", false),
            "rt2-key",
            "10.244.0.8/32",
            "192.168.1.9:51820",
        );
        let data = original.to_data();
        let rebuilt = Affiliate::from_data(&data, None).unwrap();
        assert!(!rebuilt.kubespan().is_enabled());
    }

    #[test]
    fn observe_data_feeds_membership() {
        let mut m = Membership::new(Identity::derive_from_seed("local"));
        let data = affiliate("peer-d", "10.0.0.5", false).to_data();
        assert_eq!(
            m.observe_data(&data, None).unwrap(),
            MemberState::Discovered
        );
        assert_eq!(m.observe_data(&data, None).unwrap(), MemberState::Confirmed);
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn from_data_rejects_invalid_identity() {
        let data = AffiliateData::new("has space", "h")
            .with_address(NodeAddress::parse_v4("10.0.0.2").unwrap());
        assert!(Affiliate::from_data(&data, None).is_err());
    }

    #[test]
    fn forget_removes_member() {
        let mut m = Membership::new(Identity::derive_from_seed("local"));
        let a = affiliate("peer-a", "10.0.0.2", false);
        m.observe(a.clone()).unwrap();
        assert!(m.member(a.identity()).is_some());
        let removed = m.forget(a.identity()).unwrap();
        assert_eq!(removed.identity(), a.identity());
        assert!(m.member(a.identity()).is_none());
    }
}
