//! The discovery service client and registries.
//!
//! Mirrors `pkg/cluster/discovery` and the discovery service client in Talos
//! (`internal/app/machined/pkg/controllers/cluster.DiscoveryServiceController`).
//!
//! Talos nodes optionally register with an external discovery service. Each node
//! pushes an **encrypted affiliate** keyed by `(cluster_id, affiliate_id)` and
//! periodically pulls the full set of affiliates for its cluster back down. The
//! service holds each affiliate for a TTL; an affiliate that is not refreshed
//! expires and disappears from the snapshot. Locally, two registries feed the
//! membership controller:
//!
//! * the **Kubernetes registry**, populated from node objects, and
//! * the **service registry**, populated from the discovery service.
//!
//! This module models the *service* side: a [`DiscoveryService`] (an in-memory
//! stand-in for the remote service, used by tests), a [`DiscoveryClient`] that a
//! node uses to publish/refresh/pull, and a [`RegistryConfig`] describing which
//! registries are enabled. No real networking or crypto is performed; the
//! "encrypted" affiliate is modeled as an opaque blob carrying an
//! [`AffiliateData`].

use std::collections::BTreeMap;

use os_kernel::error::{Error, Result};

use crate::affiliate::AffiliateData;

/// The default TTL, in logical ticks, a discovery service holds an affiliate
/// before expiring it. Talos uses 30 minutes; here a "tick" is an abstract unit
/// advanced by [`DiscoveryService::tick`].
pub const DEFAULT_AFFILIATE_TTL: u64 = 30;

/// Which discovery registries a node has enabled. Mirrors the
/// `cluster.discovery.registries` machine-config block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryConfig {
    /// Whether discovery is enabled at all.
    pub enabled: bool,
    /// Whether the Kubernetes registry (node objects) is used.
    pub kubernetes: bool,
    /// Whether the external discovery service registry is used.
    pub service: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        // Talos default: discovery on, Kubernetes registry on, service registry
        // on (when a discovery service endpoint is configured).
        RegistryConfig {
            enabled: true,
            kubernetes: true,
            service: true,
        }
    }
}

impl RegistryConfig {
    /// Discovery completely disabled.
    pub fn disabled() -> Self {
        RegistryConfig {
            enabled: false,
            kubernetes: false,
            service: false,
        }
    }

    /// Whether the external service registry is effectively active.
    pub fn uses_service(&self) -> bool {
        self.enabled && self.service
    }

    /// Whether the Kubernetes registry is effectively active.
    pub fn uses_kubernetes(&self) -> bool {
        self.enabled && self.kubernetes
    }
}

/// A single affiliate as held by the discovery service: the opaque encrypted
/// blob plus its remaining TTL. The blob is modeled as an [`AffiliateData`]
/// (encryption is out of scope), tagged with the publishing affiliate id.
#[derive(Debug, Clone, PartialEq, Eq)]
struct StoredAffiliate {
    data: AffiliateData,
    /// Remaining ticks before expiry.
    ttl: u64,
}

/// An in-memory model of the remote discovery service for one cluster namespace.
///
/// The service is keyed by `cluster_id`; within a cluster, affiliates are keyed
/// by their affiliate id. Tests drive it directly; production code would talk to
/// the gRPC service. The model captures the behaviors the controller depends on:
/// per-affiliate TTL, refresh-on-push, expiry on [`tick`](Self::tick), and a
/// snapshot pull that returns every live affiliate except (optionally) the
/// caller's own.
#[derive(Debug, Default, Clone)]
pub struct DiscoveryService {
    clusters: BTreeMap<String, BTreeMap<String, StoredAffiliate>>,
}

impl DiscoveryService {
    /// A fresh, empty service.
    pub fn new() -> Self {
        DiscoveryService {
            clusters: BTreeMap::new(),
        }
    }

    /// Publish (or refresh) an affiliate into `cluster_id` with the default TTL.
    /// Returns whether this created a new affiliate (vs. refreshed an existing).
    pub fn push(&mut self, cluster_id: &str, data: AffiliateData) -> bool {
        self.push_with_ttl(cluster_id, data, DEFAULT_AFFILIATE_TTL)
    }

    /// Publish (or refresh) an affiliate with an explicit TTL.
    pub fn push_with_ttl(&mut self, cluster_id: &str, data: AffiliateData, ttl: u64) -> bool {
        let cluster = self.clusters.entry(cluster_id.to_string()).or_default();
        let key = data.affiliate_id().to_string();
        let stored = StoredAffiliate { data, ttl };
        cluster.insert(key, stored).is_none()
    }

    /// Explicitly delete an affiliate (a node leaving the cluster cleanly).
    /// Returns whether something was removed.
    pub fn delete(&mut self, cluster_id: &str, affiliate_id: &str) -> bool {
        match self.clusters.get_mut(cluster_id) {
            Some(cluster) => cluster.remove(affiliate_id).is_some(),
            None => false,
        }
    }

    /// Pull the live affiliates for `cluster_id`, excluding `exclude_id` (the
    /// caller's own affiliate). Returns them in affiliate-id order for a stable
    /// snapshot.
    pub fn pull(&self, cluster_id: &str, exclude_id: &str) -> Vec<AffiliateData> {
        match self.clusters.get(cluster_id) {
            None => Vec::new(),
            Some(cluster) => cluster
                .iter()
                .filter(|(id, _)| id.as_str() != exclude_id)
                .map(|(_, s)| s.data.clone())
                .collect(),
        }
    }

    /// The number of live affiliates in a cluster (including all nodes).
    pub fn count(&self, cluster_id: &str) -> usize {
        self.clusters.get(cluster_id).map_or(0, BTreeMap::len)
    }

    /// Advance time by `ticks`, decrementing every affiliate's TTL and removing
    /// any that reach zero. Returns the total number of affiliates expired.
    pub fn tick(&mut self, ticks: u64) -> usize {
        let mut expired = 0;
        for cluster in self.clusters.values_mut() {
            let before = cluster.len();
            cluster.retain(|_, s| {
                s.ttl = s.ttl.saturating_sub(ticks);
                s.ttl > 0
            });
            expired += before - cluster.len();
        }
        // drop now-empty clusters to keep the map tidy.
        self.clusters.retain(|_, c| !c.is_empty());
        expired
    }
}

/// A node's discovery client: it knows its cluster id, its own affiliate id, and
/// which registries are enabled, and drives publish/refresh/pull cycles against
/// a [`DiscoveryService`].
#[derive(Debug, Clone)]
pub struct DiscoveryClient {
    cluster_id: String,
    affiliate_id: String,
    config: RegistryConfig,
    /// How many push cycles have run (diagnostic / metrics-ish).
    push_count: u64,
}

impl DiscoveryClient {
    /// Construct a client for `cluster_id`/`affiliate_id`.
    pub fn new(
        cluster_id: impl Into<String>,
        affiliate_id: impl Into<String>,
        config: RegistryConfig,
    ) -> Result<Self> {
        let cluster_id = cluster_id.into();
        let affiliate_id = affiliate_id.into();
        if cluster_id.is_empty() {
            return Err(Error::invalid("discovery cluster id is empty"));
        }
        if affiliate_id.is_empty() {
            return Err(Error::invalid("discovery affiliate id is empty"));
        }
        Ok(DiscoveryClient {
            cluster_id,
            affiliate_id,
            config,
            push_count: 0,
        })
    }

    /// The cluster id this client publishes to.
    pub fn cluster_id(&self) -> &str {
        &self.cluster_id
    }

    /// The local affiliate id.
    pub fn affiliate_id(&self) -> &str {
        &self.affiliate_id
    }

    /// The registry configuration.
    pub fn config(&self) -> RegistryConfig {
        self.config
    }

    /// Number of completed push cycles.
    pub fn push_count(&self) -> u64 {
        self.push_count
    }

    /// Publish/refresh the local affiliate to the service registry. Does nothing
    /// (returns `Ok(false)`) if the service registry is disabled or the supplied
    /// data does not belong to this client. Returns `Ok(true)` on a successful
    /// push.
    pub fn publish(&mut self, service: &mut DiscoveryService, data: AffiliateData) -> Result<bool> {
        if !self.config.uses_service() {
            return Ok(false);
        }
        if data.affiliate_id() != self.affiliate_id {
            return Err(Error::invalid_state(
                "published affiliate id does not match client identity",
            ));
        }
        service.push(&self.cluster_id, data);
        self.push_count += 1;
        Ok(true)
    }

    /// Pull the peer affiliates from the service registry (excluding self).
    /// Returns an empty vec if the service registry is disabled.
    pub fn pull(&self, service: &DiscoveryService) -> Vec<AffiliateData> {
        if !self.config.uses_service() {
            return Vec::new();
        }
        service.pull(&self.cluster_id, &self.affiliate_id)
    }

    /// Cleanly remove the local affiliate from the service registry (node
    /// shutdown / leaving the cluster). Returns whether anything was removed.
    pub fn deregister(&self, service: &mut DiscoveryService) -> bool {
        if !self.config.uses_service() {
            return false;
        }
        service.delete(&self.cluster_id, &self.affiliate_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::affiliate::AffiliateData;

    fn data(id: &str) -> AffiliateData {
        AffiliateData::new(id, format!("{id}.local"))
    }

    #[test]
    fn registry_config_defaults_and_disabled() {
        let d = RegistryConfig::default();
        assert!(d.uses_service());
        assert!(d.uses_kubernetes());
        let off = RegistryConfig::disabled();
        assert!(!off.uses_service());
        assert!(!off.uses_kubernetes());
    }

    #[test]
    fn push_pull_excludes_self() {
        let mut svc = DiscoveryService::new();
        assert!(svc.push("c1", data("a")));
        assert!(svc.push("c1", data("b")));
        // refresh of existing is not "new"
        assert!(!svc.push("c1", data("a")));
        assert_eq!(svc.count("c1"), 2);

        let peers = svc.pull("c1", "a");
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].affiliate_id(), "b");
    }

    #[test]
    fn ttl_expiry_drops_affiliate() {
        let mut svc = DiscoveryService::new();
        svc.push_with_ttl("c1", data("a"), 3);
        svc.push_with_ttl("c1", data("b"), 10);
        assert_eq!(svc.count("c1"), 2);
        // advance past a's ttl but not b's
        assert_eq!(svc.tick(3), 1);
        assert_eq!(svc.count("c1"), 1);
        assert_eq!(svc.pull("c1", "")[0].affiliate_id(), "b");
        // b still alive
        assert_eq!(svc.tick(5), 0);
        assert_eq!(svc.count("c1"), 1);
    }

    #[test]
    fn refresh_resets_ttl() {
        let mut svc = DiscoveryService::new();
        svc.push_with_ttl("c1", data("a"), 5);
        svc.tick(4); // ttl now 1
        // refresh resets ttl back to default
        svc.push("c1", data("a"));
        // a should survive a 4-tick advance now
        assert_eq!(svc.tick(4), 0);
        assert_eq!(svc.count("c1"), 1);
    }

    #[test]
    fn delete_removes_and_cleans_empty_cluster() {
        let mut svc = DiscoveryService::new();
        svc.push("c1", data("a"));
        assert!(svc.delete("c1", "a"));
        assert!(!svc.delete("c1", "a"));
        assert_eq!(svc.count("c1"), 0);
    }

    #[test]
    fn client_publish_and_pull_round_trip() {
        let mut svc = DiscoveryService::new();
        let mut client = DiscoveryClient::new("c1", "a", RegistryConfig::default()).unwrap();
        // also seed a peer directly
        svc.push("c1", data("b"));

        assert!(client.publish(&mut svc, data("a")).unwrap());
        assert_eq!(client.push_count(), 1);
        assert_eq!(svc.count("c1"), 2);

        let peers = client.pull(&svc);
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].affiliate_id(), "b");
    }

    #[test]
    fn client_rejects_foreign_affiliate() {
        let mut svc = DiscoveryService::new();
        let mut client = DiscoveryClient::new("c1", "a", RegistryConfig::default()).unwrap();
        assert!(client.publish(&mut svc, data("other")).is_err());
    }

    #[test]
    fn disabled_service_registry_is_noop() {
        let mut svc = DiscoveryService::new();
        let mut client = DiscoveryClient::new("c1", "a", RegistryConfig::disabled()).unwrap();
        assert!(!client.publish(&mut svc, data("a")).unwrap());
        assert_eq!(svc.count("c1"), 0);
        assert!(client.pull(&svc).is_empty());
        assert!(!client.deregister(&mut svc));
    }

    #[test]
    fn deregister_removes_local_affiliate() {
        let mut svc = DiscoveryService::new();
        let mut client = DiscoveryClient::new("c1", "a", RegistryConfig::default()).unwrap();
        client.publish(&mut svc, data("a")).unwrap();
        assert!(client.deregister(&mut svc));
        assert_eq!(svc.count("c1"), 0);
    }

    #[test]
    fn empty_ids_rejected() {
        assert!(DiscoveryClient::new("", "a", RegistryConfig::default()).is_err());
        assert!(DiscoveryClient::new("c", "", RegistryConfig::default()).is_err());
    }
}
