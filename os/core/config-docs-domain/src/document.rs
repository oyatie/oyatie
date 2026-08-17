//! The [`ConfigDocument`] trait and shared document metadata.
//!
//! Mirrors the Talos `config.Document` interface: every machinery config
//! document carries an `apiVersion` + `kind` header, validates itself in
//! isolation, and declares whether it is a singleton.

use os_kernel::error::Result;
use std::fmt;

/// The `apiVersion` used by every modern typed machinery config document.
pub const API_VERSION: &str = "v1alpha1";

/// The enumerated set of document kinds modeled by this crate.
///
/// Mirrors the `kind:` discriminator of each Talos config document type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocKind {
    /// `SideroLinkConfig` — `SideroLink` (KMS / wireguard) join URL.
    SideroLink,
    /// `ExtensionServiceConfig` — per-extension-service environment & files.
    ExtensionService,
    /// `NetworkRuleConfig` — host ingress firewall rule.
    NetworkRule,
    /// `VolumeConfig` — system volume provisioning override.
    Volume,
    /// `UserVolumeConfig` — user-defined partition + mount.
    UserVolume,
    /// `TrustedRootsConfig` — extra trusted CA roots.
    TrustedRoots,
    /// `EventSinkConfig` — gRPC machine event sink endpoint.
    EventSink,
    /// `KmsgLogConfig` — kernel log delivery destination.
    KmsgLog,
    /// `DHCPv4Config` — DHCPv4 client configuration for one link.
    DhcpV4,
    /// `DHCPv6Config` — DHCPv6 client configuration for one link.
    DhcpV6,
    /// `LinkConfig` — physical link configuration.
    Link,
    /// `VLANConfig` — VLAN link configuration.
    Vlan,
    /// `ResolverConfig` — DNS resolver and hostDNS configuration.
    Resolver,
}

impl DocKind {
    /// The canonical `kind:` string for this document.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            DocKind::SideroLink => "SideroLinkConfig",
            DocKind::ExtensionService => "ExtensionServiceConfig",
            DocKind::NetworkRule => "NetworkRuleConfig",
            DocKind::Volume => "VolumeConfig",
            DocKind::UserVolume => "UserVolumeConfig",
            DocKind::TrustedRoots => "TrustedRootsConfig",
            DocKind::EventSink => "EventSinkConfig",
            DocKind::KmsgLog => "KmsgLogConfig",
            DocKind::DhcpV4 => "DHCPv4Config",
            DocKind::DhcpV6 => "DHCPv6Config",
            DocKind::Link => "LinkConfig",
            DocKind::Vlan => "VLANConfig",
            DocKind::Resolver => "ResolverConfig",
        }
    }

    /// Parse a `kind:` string back to a [`DocKind`].
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "SideroLinkConfig" => DocKind::SideroLink,
            "ExtensionServiceConfig" => DocKind::ExtensionService,
            "NetworkRuleConfig" => DocKind::NetworkRule,
            "VolumeConfig" => DocKind::Volume,
            "UserVolumeConfig" => DocKind::UserVolume,
            "TrustedRootsConfig" => DocKind::TrustedRoots,
            "EventSinkConfig" => DocKind::EventSink,
            "KmsgLogConfig" => DocKind::KmsgLog,
            "DHCPv4Config" => DocKind::DhcpV4,
            "DHCPv6Config" => DocKind::DhcpV6,
            "LinkConfig" => DocKind::Link,
            "VLANConfig" => DocKind::Vlan,
            "ResolverConfig" => DocKind::Resolver,
            _ => return None,
        })
    }

    /// Whether more than one document of this kind may appear in a container.
    ///
    /// In Talos, most documents are singletons (one `SideroLinkConfig`, one
    /// `EventSinkConfig`, etc.), while name-keyed documents
    /// (`ExtensionServiceConfig`, `UserVolumeConfig`, `NetworkRuleConfig`,
    /// `TrustedRootsConfig`, `KmsgLogConfig`, `DHCPv4Config`, `DHCPv6Config`,
    /// `LinkConfig`, `VLANConfig`) may be repeated.
    #[must_use]
    pub fn allows_multiple(self) -> bool {
        matches!(
            self,
            DocKind::ExtensionService
                | DocKind::NetworkRule
                | DocKind::UserVolume
                | DocKind::TrustedRoots
                | DocKind::KmsgLog
                | DocKind::DhcpV4
                | DocKind::DhcpV6
                | DocKind::Link
                | DocKind::Vlan
        )
    }
}

impl fmt::Display for DocKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The header common to every document: `apiVersion` + `kind`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocMeta {
    /// The `apiVersion` value (always [`API_VERSION`] for now).
    pub api_version: String,
    /// The document kind.
    pub kind: DocKind,
}

impl DocMeta {
    /// Build a metadata header for the given kind at the current api version.
    #[must_use]
    pub fn new(kind: DocKind) -> Self {
        DocMeta {
            api_version: API_VERSION.to_string(),
            kind,
        }
    }
}

/// A stable identity for a document within a [`crate::set::DocumentSet`].
///
/// Singletons identify only by [`DocKind`]; name-keyed documents also carry
/// their `name:` (or other natural key) so duplicates can be detected.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocId {
    /// The document kind.
    pub kind: DocKind,
    /// The natural key (name) for multi-instance documents; empty for
    /// singletons.
    pub key: String,
}

impl DocId {
    /// Build a singleton id (no name key).
    #[must_use]
    pub fn singleton(kind: DocKind) -> Self {
        DocId {
            kind,
            key: String::new(),
        }
    }

    /// Build a keyed id for a multi-instance document.
    pub fn keyed(kind: DocKind, key: impl Into<String>) -> Self {
        DocId {
            kind,
            key: key.into(),
        }
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.key.is_empty() {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "{}/{}", self.kind, self.key)
        }
    }
}

/// A single typed machinery configuration document.
///
/// Mirrors the Talos `config.Document` interface. Implementors report their
/// metadata, expose a stable [`DocId`], and validate themselves in isolation.
/// Cross-document validation is performed by [`crate::set::DocumentSet`].
pub trait ConfigDocument: fmt::Debug {
    /// The document kind discriminator.
    fn kind(&self) -> DocKind;

    /// The `apiVersion`/`kind` header.
    fn meta(&self) -> DocMeta {
        DocMeta::new(self.kind())
    }

    /// The stable identity of this document within a set.
    fn id(&self) -> DocId;

    /// Validate the document in isolation, returning the first error.
    fn validate(&self) -> Result<()>;

    /// Whether multiple documents of this kind may coexist.
    fn allows_multiple(&self) -> bool {
        self.kind().allows_multiple()
    }

    /// Downcast helper: the document as a `UserVolumeConfig`, if it is one.
    ///
    /// `Box<dyn ConfigDocument>` is not `Any` (we keep the crate
    /// dependency-free and avoid the `'static` bound games), so cross-document
    /// validation recovers concrete types through these cheap, opt-in hooks.
    /// Default returns `None`; only the matching concrete type overrides it.
    fn as_user_volume(&self) -> Option<&crate::volume::UserVolumeConfig> {
        None
    }

    /// Downcast helper: the document as a `KmsgLogConfig`, if it is one.
    fn as_kmsg_log(&self) -> Option<&crate::kmsg_log::KmsgLogConfig> {
        None
    }

    /// Downcast helper: the document as an `ExtensionServiceConfig`, if it is
    /// one.
    fn as_extension_service(&self) -> Option<&crate::extension_service::ExtensionServiceConfig> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_roundtrips() {
        for k in [
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
        ] {
            assert_eq!(DocKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(DocKind::parse("Nonsense"), None);
        assert_eq!(DocKind::parse("  VolumeConfig "), Some(DocKind::Volume));
    }

    #[test]
    fn singleton_vs_multiple() {
        assert!(!DocKind::SideroLink.allows_multiple());
        assert!(!DocKind::EventSink.allows_multiple());
        assert!(!DocKind::Volume.allows_multiple());
        assert!(DocKind::ExtensionService.allows_multiple());
        assert!(DocKind::UserVolume.allows_multiple());
        assert!(DocKind::KmsgLog.allows_multiple());
    }

    #[test]
    fn docid_display() {
        assert_eq!(
            DocId::singleton(DocKind::EventSink).to_string(),
            "EventSinkConfig"
        );
        assert_eq!(
            DocId::keyed(DocKind::UserVolume, "data").to_string(),
            "UserVolumeConfig/data"
        );
    }

    #[test]
    fn meta_carries_api_version() {
        let m = DocMeta::new(DocKind::SideroLink);
        assert_eq!(m.api_version, API_VERSION);
        assert_eq!(m.kind, DocKind::SideroLink);
    }
}
