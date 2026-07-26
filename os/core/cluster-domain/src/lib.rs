//! # talos-cluster
//!
//! Port of the Talos cluster discovery, membership, and KubeSpan/SideroLink
//! subsystem (`pkg/machinery/resources/cluster`, the discovery client, and the
//! `internal/app/machined/pkg/controllers/cluster` + `kubespan` controllers).
//!
//! The crate models the data plane of cluster peering without touching the
//! kernel or the network: nodes advertise an [`identity::Identity`], a set of
//! [`endpoint::ClusterEndpoint`]s, and a [`wireguard::WireguardKey`]. The
//! discovery service exchanges these as **affiliates**; the local membership
//! controller reconciles affiliates into [`membership::Member`]s and decides
//! which become programmable KubeSpan peers.
//!
//! Uses only the standard library plus the internal `talos-core` crate, so the
//! build stays fully offline.

// These pedantic lints fire pervasively on this crate's data-modeling API
// surface (accessor methods, builder constructors) and the documentation
// suggestions they push (`doc_markdown` on KubeSpan/SideroLink/IPv4 and the
// like) add noise without improving clarity, so they are allowed crate-wide.
#![allow(
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::return_self_not_must_use,
    clippy::doc_markdown
)]

pub mod affiliate;
pub mod discovery;
pub mod endpoint;
pub mod identity;
pub mod membership;
pub mod wireguard;

use os_kernel::address::NodeAddress;

/// A total-ordering key for a [`NodeAddress`].
///
/// `os_kernel::address::NodeAddress` deliberately does not implement `Ord`, but
/// this crate needs a canonical, stable order so that endpoint and affiliate
/// lists produce deterministic fingerprints. IPv4 sorts before IPv6, then by the
/// raw octets/segments. Returned as a tuple so callers can derive `Ord` cheaply.
pub(crate) fn addr_sort_key(addr: &NodeAddress) -> (u8, [u16; 8]) {
    match addr {
        NodeAddress::V4(o) => (
            0,
            [
                u16::from(o[0]),
                u16::from(o[1]),
                u16::from(o[2]),
                u16::from(o[3]),
                0,
                0,
                0,
                0,
            ],
        ),
        NodeAddress::V6(g) => (1, *g),
    }
}

pub use affiliate::{AffiliateData, AffiliateMerger, RegistrySource};
pub use discovery::{DEFAULT_AFFILIATE_TTL, DiscoveryClient, DiscoveryService, RegistryConfig};
pub use endpoint::{ClusterEndpoint, EndpointList};
pub use identity::{ClusterIdentity, Identity};
pub use membership::{Affiliate, Member, MemberState, Membership};
pub use wireguard::{AllowedIp, WireguardKey, WireguardPeer};
