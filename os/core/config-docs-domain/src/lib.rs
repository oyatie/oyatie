//! # talos-config-docs
//!
//! Additional machinery configuration *document kinds* for the operating-system Talos
//! port — everything beyond the legacy monolithic `v1alpha1` machine config.
//!
//! This crate mirrors the Talos `pkg/machinery/config/types/{siderolink,
//! extensions,runtime,network,block,security}` packages. Talos splits modern
//! configuration into many small, independently typed YAML documents joined in
//! a single multi-document file:
//!
//! ```yaml
//! apiVersion: v1alpha1
//! kind: SideroLinkConfig
//! apiUrl: https://siderolink.example/?jointoken=abc
//! ---
//! apiVersion: v1alpha1
//! kind: KmsgLogConfig
//! name: remote
//! url: tcp://10.0.0.1:514
//! ```
//!
//! Each document kind is modeled as a real Rust type implementing the
//! [`ConfigDocument`] trait: it knows its `apiVersion`/`kind`, validates itself
//! in isolation, and declares whether multiple instances may coexist. The
//! [`DocumentSet`] container then performs *cross-document* (multi-doc)
//! validation: detecting duplicate singletons, conflicting volume mounts,
//! conflicting kmsg/event sinks, and name collisions.
//!
//! The crate depends only on `talos-core` and the standard library — no
//! external crates — so the workspace keeps building fully offline.
//!
//! ## Module map
//!
//! - [`document`] — the [`ConfigDocument`] trait, [`DocId`], and shared meta.
//! - [`siderolink`] — `SideroLinkConfig` (KMS / wireguard join URL).
//! - [`extension_service`] — `ExtensionServiceConfig` (extension service env,
//!   config files, mounts).
//! - [`network_rule`] — `NetworkRuleConfig` (ingress firewall rules).
//! - [`volume`] — `VolumeConfig` / `UserVolumeConfig` (provisioning + mounts).
//! - [`trusted_roots`] — `TrustedRootsConfig` (extra CA certificates).
//! - [`event_sink`] — `EventSinkConfig` (gRPC event sink endpoint).
//! - [`kmsg_log`] — `KmsgLogConfig` (kernel log delivery sink).
//! - [`dhcpv4`] — `DHCPv4Config` (DHCPv4 client config per link).
//! - [`dhcpv6`] — `DHCPv6Config` (DHCPv6 client config per link).
//! - [`link_config`] — `LinkConfig` / `VLANConfig` (static link config).
//! - [`resolver`] — `ResolverConfig` (DNS resolver and hostDNS config).
//! - [`set`] — the [`DocumentSet`] multi-document container + conflict checks.

pub mod dhcpv4;
pub mod dhcpv6;
pub mod document;
pub mod event_sink;
pub mod extension_service;
pub mod kmsg_log;
pub mod link_config;
pub mod network_rule;
pub mod resolver;
pub mod set;
pub mod siderolink;
pub mod trusted_roots;
pub mod volume;

pub use dhcpv4::{DhcpV4ClientIdentifier, DhcpV4Config};
pub use dhcpv6::{DhcpClientIdentifier, DhcpV6Config};
pub use document::{API_VERSION, ConfigDocument, DocId, DocKind, DocMeta};
pub use event_sink::EventSinkConfig;
pub use extension_service::{ExtensionServiceConfig, ExtensionServiceConfigFile, Mount};
pub use kmsg_log::KmsgLogConfig;
pub use link_config::{AddressConfig, LinkConfig, LinkFields, RouteConfig, VlanConfig, VlanMode};
pub use network_rule::{IngressRule, NetworkRuleConfig, PortRange, Protocol};
pub use resolver::{
    DnsProtocol, HostDnsConfig, NameserverConfig, ResolverConfig, SearchDomainsConfig,
};
pub use set::{Conflict, DocumentSet};
pub use siderolink::SideroLinkConfig;
pub use trusted_roots::TrustedRootsConfig;
pub use volume::{Filesystem, Provisioning, UserVolumeConfig, VolumeConfig};

/// Re-export of the shared workspace error type.
pub use os_kernel::{Error, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_kinds_distinct() {
        let kinds = [
            DocKind::SideroLink,
            DocKind::ExtensionService,
            DocKind::NetworkRule,
            DocKind::Volume,
            DocKind::UserVolume,
            DocKind::TrustedRoots,
            DocKind::EventSink,
            DocKind::KmsgLog,
            DocKind::DhcpV4,
            DocKind::DhcpV6,
            DocKind::Link,
            DocKind::Vlan,
            DocKind::Resolver,
        ];
        for (i, a) in kinds.iter().enumerate() {
            for (j, b) in kinds.iter().enumerate() {
                assert_eq!(i == j, a.as_str() == b.as_str());
            }
        }
    }

    #[test]
    fn api_version_is_v1alpha1() {
        assert_eq!(API_VERSION, "v1alpha1");
    }
}
