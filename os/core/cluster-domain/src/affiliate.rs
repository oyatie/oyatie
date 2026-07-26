//! Discovery-service affiliate data and the affiliate-merge controller.
//!
//! Talos has two distinct affiliate shapes:
//!
//! * [`Affiliate`](crate::membership::Affiliate) — the rich, local cluster
//!   resource a node assembles about a peer, and
//! * [`AffiliateData`] — the flatter payload that travels through the discovery
//!   service (and, in the real system, is encrypted with the cluster secret).
//!
//! This module models `AffiliateData` plus the **affiliate merge controller**
//! (`internal/app/machined/pkg/controllers/cluster.AffiliateMergeController`),
//! which folds the affiliates learned from several registries (Kubernetes +
//! discovery service) into a single per-id view, with later/more-specific
//! sources filling gaps left by earlier ones. The merge is deterministic so the
//! resulting membership has a stable fingerprint.

use std::collections::BTreeMap;

use os_kernel::address::NodeAddress;
use os_kernel::machine_type::MachineType;

use crate::endpoint::ClusterEndpoint;
use crate::wireguard::WireguardKey;

/// The source registry an affiliate observation came from. Ordering matters for
/// merge precedence: the discovery service is considered more authoritative for
/// KubeSpan facts, Kubernetes for node addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegistrySource {
    /// Learned from Kubernetes node objects.
    Kubernetes,
    /// Learned from the external discovery service.
    Service,
}

/// The flat discovery-service representation of a node.
///
/// This is the payload published to / pulled from the discovery service. It is
/// intentionally simpler than the local `Affiliate` resource: a string id, a
/// hostname, a flat list of node addresses, an optional KubeSpan public key, and
/// a flat list of KubeSpan endpoints. Lists are kept sorted+deduplicated so two
/// equal logical affiliates compare byte-for-byte equal (a stand-in for the
/// canonical protobuf encoding Talos relies on).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffiliateData {
    affiliate_id: String,
    hostname: String,
    machine_type: MachineType,
    addresses: Vec<NodeAddress>,
    kubespan_key: Option<WireguardKey>,
    kubespan_endpoints: Vec<ClusterEndpoint>,
}

impl AffiliateData {
    /// A minimal affiliate with just an id and hostname. The machine type
    /// defaults to [`MachineType::Worker`]; use [`with_machine_type`](Self::with_machine_type)
    /// to mark a control-plane node.
    pub fn new(affiliate_id: impl Into<String>, hostname: impl Into<String>) -> Self {
        AffiliateData {
            affiliate_id: affiliate_id.into(),
            hostname: hostname.into(),
            machine_type: MachineType::Worker,
            addresses: Vec::new(),
            kubespan_key: None,
            kubespan_endpoints: Vec::new(),
        }
    }

    /// Set the machine type (builder style).
    pub fn with_machine_type(mut self, machine_type: MachineType) -> Self {
        self.machine_type = machine_type;
        self
    }

    /// The advertised machine type.
    pub fn machine_type(&self) -> MachineType {
        self.machine_type
    }

    /// Add a node address (sorted + deduplicated). Builder style.
    pub fn with_address(mut self, addr: NodeAddress) -> Self {
        self.add_address(addr);
        self
    }

    /// Add a node address (sorted + deduplicated).
    pub fn add_address(&mut self, addr: NodeAddress) -> bool {
        match self
            .addresses
            .binary_search_by(|a| crate::addr_sort_key(a).cmp(&crate::addr_sort_key(&addr)))
        {
            Ok(_) => false,
            Err(pos) => {
                self.addresses.insert(pos, addr);
                true
            }
        }
    }

    /// Attach a KubeSpan public key and endpoints (builder style).
    pub fn with_kubespan(
        mut self,
        key: WireguardKey,
        endpoints: impl IntoIterator<Item = ClusterEndpoint>,
    ) -> Self {
        self.kubespan_key = Some(key);
        for ep in endpoints {
            self.add_kubespan_endpoint(ep);
        }
        self
    }

    /// Add a single KubeSpan endpoint (sorted + deduplicated).
    pub fn add_kubespan_endpoint(&mut self, ep: ClusterEndpoint) -> bool {
        match self.kubespan_endpoints.binary_search(&ep) {
            Ok(_) => false,
            Err(pos) => {
                self.kubespan_endpoints.insert(pos, ep);
                true
            }
        }
    }

    /// The affiliate id (the discovery key).
    pub fn affiliate_id(&self) -> &str {
        &self.affiliate_id
    }

    /// The advertised hostname.
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    /// The advertised node addresses, in canonical order.
    pub fn addresses(&self) -> &[NodeAddress] {
        &self.addresses
    }

    /// The KubeSpan public key, if any.
    pub fn kubespan_key(&self) -> Option<&WireguardKey> {
        self.kubespan_key.as_ref()
    }

    /// The KubeSpan endpoints, in canonical order.
    pub fn kubespan_endpoints(&self) -> &[ClusterEndpoint] {
        &self.kubespan_endpoints
    }

    /// Whether this affiliate carries KubeSpan facts.
    pub fn has_kubespan(&self) -> bool {
        self.kubespan_key.is_some()
    }

    /// Whether the affiliate advertises any routable (non-loopback) address.
    pub fn is_routable(&self) -> bool {
        self.addresses.iter().any(|a| !a.is_loopback())
    }

    /// Fold another observation of the *same* affiliate into this one. Addresses
    /// and KubeSpan endpoints are unioned; the KubeSpan key and hostname are
    /// taken from `other` only when this one lacks them. Returns whether `self`
    /// changed.
    ///
    /// Panics in debug builds if the ids differ — merging two different nodes is
    /// a programming error.
    pub fn merge_from(&mut self, other: &AffiliateData) -> bool {
        debug_assert_eq!(self.affiliate_id, other.affiliate_id);
        let mut changed = false;
        if self.hostname.is_empty() && !other.hostname.is_empty() {
            self.hostname.clone_from(&other.hostname);
            changed = true;
        }
        // control-plane is more specific than the worker default: let either
        // registry upgrade the role, but never downgrade it.
        if other.machine_type.is_control_plane() && !self.machine_type.is_control_plane() {
            self.machine_type = other.machine_type;
            changed = true;
        }
        for addr in &other.addresses {
            if self.add_address(*addr) {
                changed = true;
            }
        }
        if self.kubespan_key.is_none()
            && let Some(k) = &other.kubespan_key {
                self.kubespan_key = Some(k.clone());
                changed = true;
            }
        for ep in &other.kubespan_endpoints {
            if self.add_kubespan_endpoint(*ep) {
                changed = true;
            }
        }
        changed
    }
}

/// The affiliate-merge controller: accumulates affiliate observations from
/// multiple registries and produces one merged affiliate per id.
///
/// Mirrors `AffiliateMergeController`: discovery emits per-registry affiliates
/// (e.g. `service/<id>` and `k8s/<id>`); this controller merges them into the
/// canonical `cluster/<id>` affiliate the membership controller consumes.
#[derive(Debug, Default, Clone)]
pub struct AffiliateMerger {
    /// id -> merged affiliate.
    merged: BTreeMap<String, AffiliateData>,
}

impl AffiliateMerger {
    /// An empty merger.
    pub fn new() -> Self {
        AffiliateMerger {
            merged: BTreeMap::new(),
        }
    }

    /// Ingest an affiliate observation from `source`. If an affiliate with the
    /// same id already exists it is merged; otherwise it is inserted. Returns
    /// whether the merged set changed as a result.
    pub fn ingest(&mut self, _source: RegistrySource, data: AffiliateData) -> bool {
        if let Some(existing) = self.merged.get_mut(data.affiliate_id()) {
            existing.merge_from(&data)
        } else {
            self.merged.insert(data.affiliate_id().to_string(), data);
            true
        }
    }

    /// Remove an affiliate entirely (e.g. it disappeared from every registry).
    pub fn remove(&mut self, affiliate_id: &str) -> Option<AffiliateData> {
        self.merged.remove(affiliate_id)
    }

    /// The number of distinct merged affiliates.
    pub fn len(&self) -> usize {
        self.merged.len()
    }

    /// Whether the merger holds nothing.
    pub fn is_empty(&self) -> bool {
        self.merged.is_empty()
    }

    /// Look up a merged affiliate by id.
    pub fn get(&self, affiliate_id: &str) -> Option<&AffiliateData> {
        self.merged.get(affiliate_id)
    }

    /// All merged affiliates, in id order (the canonical snapshot the membership
    /// controller reconciles).
    pub fn affiliates(&self) -> impl Iterator<Item = &AffiliateData> {
        self.merged.values()
    }

    /// Retain only the affiliate ids present in `present`, dropping the rest.
    /// Returns the number removed. Used when a fresh discovery snapshot replaces
    /// the previous one.
    pub fn retain_ids(&mut self, present: &[&str]) -> usize {
        let before = self.merged.len();
        self.merged.retain(|id, _| present.contains(&id.as_str()));
        before - self.merged.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wireguard::WireguardKey;

    fn ep(s: &str) -> ClusterEndpoint {
        ClusterEndpoint::parse_v4(s).unwrap()
    }

    fn addr(s: &str) -> NodeAddress {
        NodeAddress::parse_v4(s).unwrap()
    }

    #[test]
    fn addresses_sorted_and_deduped() {
        let mut d = AffiliateData::new("a", "a.local");
        assert!(d.add_address(addr("10.0.0.5")));
        assert!(d.add_address(addr("10.0.0.1")));
        assert!(!d.add_address(addr("10.0.0.5")));
        assert_eq!(d.addresses()[0], addr("10.0.0.1"));
        assert_eq!(d.addresses().len(), 2);
    }

    #[test]
    fn routable_detection() {
        let lo = AffiliateData::new("a", "a").with_address(addr("127.0.0.1"));
        assert!(!lo.is_routable());
        let r = lo.with_address(addr("10.0.0.2"));
        assert!(r.is_routable());
    }

    #[test]
    fn kubespan_builder() {
        let key = WireguardKey::derive_from_seed("k");
        let d = AffiliateData::new("a", "a").with_kubespan(key.clone(), [ep("10.0.0.1:51820")]);
        assert!(d.has_kubespan());
        assert_eq!(d.kubespan_key(), Some(&key));
        assert_eq!(d.kubespan_endpoints().len(), 1);
    }

    #[test]
    fn merge_unions_and_fills_gaps() {
        // service registry knows kubespan + an endpoint, but no hostname/address.
        let key = WireguardKey::derive_from_seed("k");
        let mut svc = AffiliateData::new("a", "");
        svc.kubespan_key = Some(key.clone());
        svc.add_kubespan_endpoint(ep("1.2.3.4:51820"));

        // k8s registry knows hostname + address.
        let k8s = AffiliateData::new("a", "a.local").with_address(addr("10.0.0.2"));

        let mut merged = svc.clone();
        assert!(merged.merge_from(&k8s));
        assert_eq!(merged.hostname(), "a.local");
        assert_eq!(merged.addresses(), &[addr("10.0.0.2")]);
        assert_eq!(merged.kubespan_key(), Some(&key));
        assert_eq!(merged.kubespan_endpoints().len(), 1);

        // merging again is a no-op (idempotent).
        assert!(!merged.merge_from(&k8s));
    }

    #[test]
    fn merge_does_not_overwrite_existing_key() {
        let k1 = WireguardKey::derive_from_seed("k1");
        let k2 = WireguardKey::derive_from_seed("k2");
        let mut a = AffiliateData::new("a", "a");
        a.kubespan_key = Some(k1.clone());
        let mut b = AffiliateData::new("a", "a");
        b.kubespan_key = Some(k2);
        a.merge_from(&b);
        // existing key wins.
        assert_eq!(a.kubespan_key(), Some(&k1));
    }

    #[test]
    fn merger_combines_two_registries() {
        let key = WireguardKey::derive_from_seed("k");
        let svc = AffiliateData::new("a", "").with_kubespan(key.clone(), [ep("1.2.3.4:51820")]);
        let k8s = AffiliateData::new("a", "a.local").with_address(addr("10.0.0.2"));

        let mut merger = AffiliateMerger::new();
        assert!(merger.ingest(RegistrySource::Service, svc));
        // second registry for same id merges (set changed).
        assert!(merger.ingest(RegistrySource::Kubernetes, k8s));
        assert_eq!(merger.len(), 1);

        let m = merger.get("a").unwrap();
        assert_eq!(m.hostname(), "a.local");
        assert!(m.has_kubespan());
        assert!(m.is_routable());
    }

    #[test]
    fn merger_distinct_ids_kept_separate() {
        let mut merger = AffiliateMerger::new();
        merger.ingest(RegistrySource::Service, AffiliateData::new("a", "a"));
        merger.ingest(RegistrySource::Service, AffiliateData::new("b", "b"));
        assert_eq!(merger.len(), 2);
        assert!(merger.get("a").is_some());
        assert!(merger.get("b").is_some());
    }

    #[test]
    fn retain_ids_prunes_absent() {
        let mut merger = AffiliateMerger::new();
        merger.ingest(RegistrySource::Service, AffiliateData::new("a", "a"));
        merger.ingest(RegistrySource::Service, AffiliateData::new("b", "b"));
        merger.ingest(RegistrySource::Service, AffiliateData::new("c", "c"));
        assert_eq!(merger.retain_ids(&["a", "c"]), 1);
        assert_eq!(merger.len(), 2);
        assert!(merger.get("b").is_none());
    }

    #[test]
    fn remove_returns_affiliate() {
        let mut merger = AffiliateMerger::new();
        merger.ingest(RegistrySource::Service, AffiliateData::new("a", "a"));
        assert!(merger.remove("a").is_some());
        assert!(merger.is_empty());
    }
}
