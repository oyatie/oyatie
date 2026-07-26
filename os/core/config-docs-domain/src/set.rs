//! The [`DocumentSet`] multi-document container and cross-document validation.
//!
//! Mirrors the Talos multi-doc config container: a single YAML file holds many
//! typed documents, and the loader enforces *cross-document* invariants that a
//! single document cannot check on its own:
//!
//! - singleton documents (`SideroLinkConfig`, `EventSinkConfig`, `VolumeConfig`
//!   per system volume) must not be duplicated;
//! - name-keyed documents must have unique keys;
//! - user volumes must not collide on their derived mount path;
//! - kmsg sinks must not target the same URL twice;
//! - extension-service mounts must not collide on destination across docs.

use crate::dhcpv4::DhcpV4Config;
use crate::dhcpv6::DhcpV6Config;
use crate::document::{ConfigDocument, DocId, DocKind};
use crate::event_sink::EventSinkConfig;
use crate::extension_service::ExtensionServiceConfig;
use crate::kmsg_log::KmsgLogConfig;
use crate::network_rule::NetworkRuleConfig;
use crate::siderolink::SideroLinkConfig;
use crate::trusted_roots::TrustedRootsConfig;
use crate::volume::{UserVolumeConfig, VolumeConfig};
use os_kernel::error::{Error, Result};

/// A boxed config document plus a cached, cheaply-cloneable id.
#[derive(Debug)]
struct Entry {
    id: DocId,
    doc: Box<dyn ConfigDocument>,
}

/// A description of a cross-document conflict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conflict {
    /// A singleton document kind appeared more than once.
    DuplicateSingleton(DocKind),
    /// Two documents shared the same [`DocId`] (kind + key).
    DuplicateId(DocId),
    /// Two user volumes resolve to the same mount path.
    VolumeMountPath { path: String },
    /// Two kmsg sinks target the same URL.
    KmsgUrl { url: String },
    /// Two distinct extension services claim the same read-write host source.
    MountSource { source: String },
}

impl core::fmt::Display for Conflict {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Conflict::DuplicateSingleton(k) => {
                write!(f, "duplicate singleton document '{k}'")
            }
            Conflict::DuplicateId(id) => write!(f, "duplicate document '{id}'"),
            Conflict::VolumeMountPath { path } => {
                write!(f, "conflicting volume mount path '{path}'")
            }
            Conflict::KmsgUrl { url } => write!(f, "conflicting kmsg sink url '{url}'"),
            Conflict::MountSource { source } => {
                write!(f, "conflicting read-write mount source '{source}'")
            }
        }
    }
}

/// A container of typed configuration documents.
#[derive(Debug, Default)]
pub struct DocumentSet {
    entries: Vec<Entry>,
}

impl DocumentSet {
    /// Construct an empty set.
    #[must_use]
    pub fn new() -> Self {
        DocumentSet {
            entries: Vec::new(),
        }
    }

    /// Number of documents held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Push a document without validation (used by typed `add_*` helpers).
    fn push(&mut self, doc: Box<dyn ConfigDocument>) {
        let id = doc.id();
        self.entries.push(Entry { id, doc });
    }

    /// Add a document, validating it in isolation first. Cross-document
    /// validation is deferred to [`DocumentSet::validate`].
    pub fn add(&mut self, doc: Box<dyn ConfigDocument>) -> Result<()> {
        doc.validate()?;
        self.push(doc);
        Ok(())
    }

    // -- typed convenience adders ------------------------------------------

    /// Add a `SideroLinkConfig`.
    pub fn add_siderolink(&mut self, c: SideroLinkConfig) -> Result<()> {
        self.add(Box::new(c))
    }
    /// Add an `ExtensionServiceConfig`.
    pub fn add_extension_service(&mut self, c: ExtensionServiceConfig) -> Result<()> {
        self.add(Box::new(c))
    }
    /// Add a `NetworkRuleConfig`.
    pub fn add_network_rule(&mut self, c: NetworkRuleConfig) -> Result<()> {
        self.add(Box::new(c))
    }
    /// Add a `VolumeConfig`.
    pub fn add_volume(&mut self, c: VolumeConfig) -> Result<()> {
        self.add(Box::new(c))
    }
    /// Add a `UserVolumeConfig`.
    pub fn add_user_volume(&mut self, c: UserVolumeConfig) -> Result<()> {
        self.add(Box::new(c))
    }
    /// Add a `TrustedRootsConfig`.
    pub fn add_trusted_roots(&mut self, c: TrustedRootsConfig) -> Result<()> {
        self.add(Box::new(c))
    }
    /// Add an `EventSinkConfig`.
    pub fn add_event_sink(&mut self, c: EventSinkConfig) -> Result<()> {
        self.add(Box::new(c))
    }
    /// Add a `KmsgLogConfig`.
    pub fn add_kmsg_log(&mut self, c: KmsgLogConfig) -> Result<()> {
        self.add(Box::new(c))
    }
    /// Add a `DHCPv4Config`.
    pub fn add_dhcpv4(&mut self, c: DhcpV4Config) -> Result<()> {
        self.add(Box::new(c))
    }
    /// Add a `DHCPv6Config`.
    pub fn add_dhcpv6(&mut self, c: DhcpV6Config) -> Result<()> {
        self.add(Box::new(c))
    }

    /// All ids in insertion order.
    #[must_use]
    pub fn ids(&self) -> Vec<DocId> {
        self.entries.iter().map(|e| e.id.clone()).collect()
    }

    /// Count documents of a given kind.
    #[must_use]
    pub fn count_kind(&self, kind: DocKind) -> usize {
        self.entries.iter().filter(|e| e.id.kind == kind).count()
    }

    /// Find the first cross-document conflict, if any. This does not validate
    /// individual documents — that is the caller's responsibility (the `add_*`
    /// helpers already do it).
    #[must_use]
    pub fn find_conflict(&self) -> Option<Conflict> {
        // 1. Duplicate ids (covers duplicate singletons and duplicate keys).
        for (i, a) in self.entries.iter().enumerate() {
            for b in &self.entries[i + 1..] {
                if a.id == b.id {
                    if !a.doc.allows_multiple() {
                        return Some(Conflict::DuplicateSingleton(a.id.kind));
                    }
                    return Some(Conflict::DuplicateId(a.id.clone()));
                }
            }
        }

        // 2. User-volume mount-path collisions (derived, not the doc key).
        let user_volumes: Vec<&UserVolumeConfig> = self.downcast_user_volumes();
        for (i, a) in user_volumes.iter().enumerate() {
            for b in &user_volumes[i + 1..] {
                if a.mount_path() == b.mount_path() {
                    return Some(Conflict::VolumeMountPath {
                        path: a.mount_path(),
                    });
                }
            }
        }

        // 3. Kmsg sink url collisions.
        let kmsg: Vec<&KmsgLogConfig> = self.downcast_kmsg();
        for (i, a) in kmsg.iter().enumerate() {
            for b in &kmsg[i + 1..] {
                if a.url == b.url {
                    return Some(Conflict::KmsgUrl { url: a.url.clone() });
                }
            }
        }

        // 4. Extension-service host-source collisions: two *distinct* service
        //    documents must not bind-mount the same exclusive host source via a
        //    read-write mount (a read-write source may only be claimed once).
        let exts: Vec<&ExtensionServiceConfig> = self.downcast_extensions();
        for (i, a) in exts.iter().enumerate() {
            for b in &exts[i + 1..] {
                if a.name == b.name {
                    // same service name is already a DuplicateId above.
                    continue;
                }
                for ma in a.mounts.iter().filter(|m| !m.read_only) {
                    for mb in b.mounts.iter().filter(|m| !m.read_only) {
                        if ma.source == mb.source {
                            return Some(Conflict::MountSource {
                                source: ma.source.clone(),
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Validate every document in isolation and then check for cross-document
    /// conflicts. Returns the first error encountered.
    pub fn validate(&self) -> Result<()> {
        for e in &self.entries {
            e.doc.validate()?;
        }
        if let Some(conflict) = self.find_conflict() {
            return Err(Error::invalid(format!(
                "multi-document conflict: {conflict}"
            )));
        }
        Ok(())
    }

    // -- downcasting helpers ------------------------------------------------
    //
    // The `ConfigDocument` trait objects are stored erased; to perform the
    // type-specific cross-document checks we recover the concrete type through
    // the opt-in `as_*` hooks on the trait (which default to `None` and are
    // overridden only by the matching concrete type). This keeps the crate
    // dependency-free without resorting to `Any`/`'static` downcasting.

    fn downcast_user_volumes(&self) -> Vec<&UserVolumeConfig> {
        self.entries
            .iter()
            .filter_map(|e| e.doc.as_user_volume())
            .collect()
    }
    fn downcast_kmsg(&self) -> Vec<&KmsgLogConfig> {
        self.entries
            .iter()
            .filter_map(|e| e.doc.as_kmsg_log())
            .collect()
    }
    fn downcast_extensions(&self) -> Vec<&ExtensionServiceConfig> {
        self.entries
            .iter()
            .filter_map(|e| e.doc.as_extension_service())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network_rule::{PortRange, Protocol};
    use crate::volume::Provisioning;

    #[test]
    fn empty_set_is_valid() {
        let set = DocumentSet::new();
        assert!(set.is_empty());
        assert!(set.validate().is_ok());
        assert!(set.find_conflict().is_none());
    }

    #[test]
    fn heterogeneous_set_validates() {
        let mut set = DocumentSet::new();
        set.add_siderolink(SideroLinkConfig::new("https://host/?jointoken=t"))
            .unwrap();
        set.add_event_sink(EventSinkConfig::new("10.0.0.1:4242"))
            .unwrap();
        set.add_user_volume(UserVolumeConfig::new("data", Provisioning::fixed(1 << 30)))
            .unwrap();
        set.add_network_rule(
            NetworkRuleConfig::new("k", Protocol::Tcp)
                .with_port_range(PortRange::single(10250))
                .with_subnets(["10.0.0.0/8".into()]),
        )
        .unwrap();
        assert_eq!(set.len(), 4);
        assert!(set.validate().is_ok());
    }

    #[test]
    fn duplicate_singleton_detected() {
        let mut set = DocumentSet::new();
        set.add_siderolink(SideroLinkConfig::new("https://a/"))
            .unwrap();
        set.add_siderolink(SideroLinkConfig::new("https://b/"))
            .unwrap();
        assert_eq!(
            set.find_conflict(),
            Some(Conflict::DuplicateSingleton(DocKind::SideroLink))
        );
        assert!(set.validate().is_err());
    }

    #[test]
    fn duplicate_keyed_id_detected() {
        let mut set = DocumentSet::new();
        set.add_user_volume(UserVolumeConfig::new("data", Provisioning::fixed(1 << 30)))
            .unwrap();
        set.add_user_volume(UserVolumeConfig::new("data", Provisioning::fixed(2 << 30)))
            .unwrap();
        // Same id -> duplicate id (user volumes allow multiple, but not same key)
        assert!(matches!(
            set.find_conflict(),
            Some(Conflict::DuplicateId(_) | Conflict::VolumeMountPath { .. })
        ));
        assert!(set.validate().is_err());
    }

    #[test]
    fn distinct_user_volumes_ok() {
        let mut set = DocumentSet::new();
        set.add_user_volume(UserVolumeConfig::new("data", Provisioning::fixed(1 << 30)))
            .unwrap();
        set.add_user_volume(UserVolumeConfig::new("logs", Provisioning::fixed(1 << 30)))
            .unwrap();
        assert!(set.find_conflict().is_none());
        assert_eq!(set.count_kind(DocKind::UserVolume), 2);
    }

    #[test]
    fn kmsg_url_conflict_detected() {
        let mut set = DocumentSet::new();
        set.add_kmsg_log(KmsgLogConfig::new("a", "tcp://10.0.0.1:514"))
            .unwrap();
        set.add_kmsg_log(KmsgLogConfig::new("b", "tcp://10.0.0.1:514"))
            .unwrap();
        assert_eq!(
            set.find_conflict(),
            Some(Conflict::KmsgUrl {
                url: "tcp://10.0.0.1:514".into()
            })
        );
    }

    #[test]
    fn kmsg_distinct_urls_ok() {
        let mut set = DocumentSet::new();
        set.add_kmsg_log(KmsgLogConfig::new("a", "tcp://10.0.0.1:514"))
            .unwrap();
        set.add_kmsg_log(KmsgLogConfig::new("b", "udp://10.0.0.2:514"))
            .unwrap();
        assert!(set.find_conflict().is_none());
    }

    #[test]
    fn extension_rw_source_conflict_detected() {
        use crate::extension_service::Mount;
        let mut set = DocumentSet::new();
        set.add_extension_service(
            ExtensionServiceConfig::new("svc-a").with_mount(Mount::new("/shared", "/dst", false)),
        )
        .unwrap();
        set.add_extension_service(
            ExtensionServiceConfig::new("svc-b").with_mount(Mount::new("/shared", "/other", false)),
        )
        .unwrap();
        assert_eq!(
            set.find_conflict(),
            Some(Conflict::MountSource {
                source: "/shared".into()
            })
        );
    }

    #[test]
    fn extension_readonly_shared_source_ok() {
        use crate::extension_service::Mount;
        let mut set = DocumentSet::new();
        set.add_extension_service(
            ExtensionServiceConfig::new("svc-a").with_mount(Mount::new("/shared", "/dst", true)),
        )
        .unwrap();
        set.add_extension_service(
            ExtensionServiceConfig::new("svc-b").with_mount(Mount::new("/shared", "/other", true)),
        )
        .unwrap();
        assert!(set.find_conflict().is_none());
    }

    #[test]
    fn add_rejects_invalid_document() {
        let mut set = DocumentSet::new();
        assert!(set.add_siderolink(SideroLinkConfig::new("")).is_err());
        assert!(set.is_empty());
    }

    #[test]
    fn conflict_display() {
        let c = Conflict::DuplicateSingleton(DocKind::EventSink);
        assert!(c.to_string().contains("EventSinkConfig"));
    }
}
