//! The multi-document **kind registry**, mirroring the Talos
//! `pkg/machinery/config/configloader` / `registry` machinery that maps a
//! document `kind` string to its decode/validation behavior.
//!
//! In upstream Talos every typed document kind registers itself so the loader
//! can split a multi-document config and dispatch each document to the right
//! decoder. We model the registration table itself: a map from `kind` to a
//! [`KindSpec`] describing the document's cardinality, the runtime modes it is
//! valid in, and whether it is a control-plane-only document. This is enough to
//! drive the container's singleton/uniqueness checks and to reject unknown
//! kinds during load.

use crate::validation::ValidationMode;
use std::collections::BTreeMap;
use os_kernel::error::{Error, Result};

/// Cardinality of a document kind within a single config container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// At most one document of this kind may appear (the common case).
    Singleton,
    /// Any number of documents of this kind may appear.
    Multiple,
}

impl Cardinality {
    /// Whether multiple documents are permitted.
    pub fn allows_multiple(self) -> bool {
        matches!(self, Cardinality::Multiple)
    }
}

/// The registration record for a single document kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindSpec {
    /// The document kind string (e.g. `"SideroLinkConfig"`).
    pub kind: String,
    /// How many of this document may appear.
    pub cardinality: Cardinality,
    /// Whether this document is only meaningful on a control-plane node.
    pub control_plane_only: bool,
    /// Runtime modes in which this document is supported. Empty means "all".
    pub modes: Vec<ValidationMode>,
}

impl KindSpec {
    /// A singleton document supported in all modes.
    pub fn singleton(kind: impl Into<String>) -> Self {
        KindSpec {
            kind: kind.into(),
            cardinality: Cardinality::Singleton,
            control_plane_only: false,
            modes: Vec::new(),
        }
    }

    /// A repeatable document supported in all modes.
    pub fn multiple(kind: impl Into<String>) -> Self {
        KindSpec {
            kind: kind.into(),
            cardinality: Cardinality::Multiple,
            control_plane_only: false,
            modes: Vec::new(),
        }
    }

    /// Restrict this kind to control-plane nodes.
    pub fn control_plane_only(mut self) -> Self {
        self.control_plane_only = true;
        self
    }

    /// Restrict this kind to the given runtime modes.
    pub fn in_modes(mut self, modes: impl IntoIterator<Item = ValidationMode>) -> Self {
        self.modes = modes.into_iter().collect();
        self
    }

    /// Whether the kind is supported in `mode` (an empty mode list means all).
    pub fn supports_mode(&self, mode: ValidationMode) -> bool {
        self.modes.is_empty() || self.modes.contains(&mode)
    }

    /// Whether multiple of this kind may appear.
    pub fn allows_multiple(&self) -> bool {
        self.cardinality.allows_multiple()
    }
}

/// The registry of known document kinds.
///
/// Mirrors the Talos document-kind registry: a loader consults it to learn how
/// to handle each document it parses out of a multi-document config.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Registry {
    kinds: BTreeMap<String, KindSpec>,
}

impl Registry {
    /// An empty registry.
    pub fn new() -> Self {
        Registry {
            kinds: BTreeMap::new(),
        }
    }

    /// A registry pre-populated with the document kinds shipped by Talos that
    /// this port models. Mirrors the default registrations in upstream Talos.
    pub fn with_builtins() -> Self {
        let mut r = Registry::new();
        // The legacy monolithic document.
        r.register(KindSpec::singleton("v1alpha1")).unwrap();
        // SideroLink / KMS / event-sink style singletons.
        r.register(KindSpec::singleton("SideroLinkConfig")).unwrap();
        r.register(KindSpec::singleton("KmsgLogConfig")).unwrap();
        r.register(KindSpec::singleton("EventSinkConfig")).unwrap();
        r.register(KindSpec::singleton("ResolverConfig")).unwrap();
        r.register(KindSpec::singleton("ExtensionServiceConfig"))
            .unwrap();
        // Trusted-roots / volumes are control-plane-flavored singletons.
        r.register(KindSpec::singleton("TrustedRootsConfig"))
            .unwrap();
        r.register(KindSpec::multiple("VolumeConfig")).unwrap();
        // Repeatable documents.
        r.register(KindSpec::multiple("DHCPv4Config")).unwrap();
        r.register(KindSpec::multiple("DHCPv6Config")).unwrap();
        r.register(KindSpec::multiple("LinkConfig")).unwrap();
        r.register(KindSpec::multiple("VLANConfig")).unwrap();
        r.register(KindSpec::multiple("NetworkRuleConfig")).unwrap();
        r.register(KindSpec::multiple("NetworkDefaultActionConfig"))
            .unwrap();
        r.register(KindSpec::multiple("UserVolumeConfig")).unwrap();
        r.register(KindSpec::multiple("RawVolumeConfig")).unwrap();
        r.register(KindSpec::multiple("ExistingVolumeConfig"))
            .unwrap();
        r.register(KindSpec::multiple("ExternalVolumeConfig"))
            .unwrap();
        r.register(KindSpec::multiple("SwapVolumeConfig")).unwrap();
        r
    }

    /// Register a new document kind. Errors on duplicate registration.
    pub fn register(&mut self, spec: KindSpec) -> Result<()> {
        if self.kinds.contains_key(&spec.kind) {
            return Err(Error::invalid(format!(
                "document kind '{}' already registered",
                spec.kind
            )));
        }
        self.kinds.insert(spec.kind.clone(), spec);
        Ok(())
    }

    /// Register, replacing any existing entry of the same kind.
    pub fn register_or_replace(&mut self, spec: KindSpec) {
        self.kinds.insert(spec.kind.clone(), spec);
    }

    /// Look up a kind's spec.
    pub fn get(&self, kind: &str) -> Option<&KindSpec> {
        self.kinds.get(kind)
    }

    /// Whether a kind is registered.
    pub fn contains(&self, kind: &str) -> bool {
        self.kinds.contains_key(kind)
    }

    /// The number of registered kinds.
    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    /// Whether the registry has no registrations.
    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    /// All registered kind names, in sorted order.
    pub fn kinds(&self) -> impl Iterator<Item = &str> {
        self.kinds.keys().map(String::as_str)
    }

    /// Resolve a kind, returning an error if it is unknown — the loader's
    /// "unknown document kind" rejection path.
    pub fn resolve(&self, kind: &str) -> Result<&KindSpec> {
        self.get(kind)
            .ok_or_else(|| Error::not_found(format!("unknown document kind '{kind}'")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_contain_core_kinds() {
        let r = Registry::with_builtins();
        assert!(r.contains("v1alpha1"));
        assert!(r.contains("SideroLinkConfig"));
        assert!(r.contains("DHCPv4Config"));
        assert!(r.contains("ResolverConfig"));
        assert!(r.contains("DHCPv6Config"));
        assert!(r.contains("LinkConfig"));
        assert!(r.contains("VLANConfig"));
        assert!(r.contains("NetworkRuleConfig"));
        assert!(!r.is_empty());
        assert!(r.len() >= 9);
    }

    #[test]
    fn singleton_vs_multiple_cardinality() {
        let r = Registry::with_builtins();
        assert!(!r.get("SideroLinkConfig").unwrap().allows_multiple());
        assert!(!r.get("ResolverConfig").unwrap().allows_multiple());
        assert!(r.get("VolumeConfig").unwrap().allows_multiple());
        assert!(r.get("DHCPv4Config").unwrap().allows_multiple());
        assert!(r.get("DHCPv6Config").unwrap().allows_multiple());
        assert!(r.get("LinkConfig").unwrap().allows_multiple());
        assert!(r.get("VLANConfig").unwrap().allows_multiple());
        assert!(r.get("NetworkRuleConfig").unwrap().allows_multiple());
    }

    #[test]
    fn duplicate_registration_rejected() {
        let mut r = Registry::new();
        r.register(KindSpec::singleton("Foo")).unwrap();
        let err = r.register(KindSpec::singleton("Foo")).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn register_or_replace_overwrites() {
        let mut r = Registry::new();
        r.register(KindSpec::singleton("Foo")).unwrap();
        r.register_or_replace(KindSpec::multiple("Foo"));
        assert!(r.get("Foo").unwrap().allows_multiple());
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn resolve_unknown_kind_errors() {
        let r = Registry::with_builtins();
        assert!(r.resolve("v1alpha1").is_ok());
        let err = r.resolve("NoSuchKind").unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn mode_restriction() {
        let spec = KindSpec::singleton("OnlyMetal").in_modes([ValidationMode::Metal]);
        assert!(spec.supports_mode(ValidationMode::Metal));
        assert!(!spec.supports_mode(ValidationMode::Container));
        // An unrestricted spec supports every mode.
        let any = KindSpec::singleton("Any");
        assert!(any.supports_mode(ValidationMode::Generate));
    }

    #[test]
    fn control_plane_only_flag() {
        let spec = KindSpec::singleton("CPOnly").control_plane_only();
        assert!(spec.control_plane_only);
        assert!(!KindSpec::singleton("X").control_plane_only);
    }

    #[test]
    fn kinds_iter_is_sorted() {
        let mut r = Registry::new();
        r.register(KindSpec::singleton("Zeta")).unwrap();
        r.register(KindSpec::singleton("Alpha")).unwrap();
        let names: Vec<&str> = r.kinds().collect();
        assert_eq!(names, vec!["Alpha", "Zeta"]);
    }
}
