//! The resource-definition and namespace registry.
//!
//! Mirrors COSI's `meta`-namespace registry: at boot Talos registers every
//! [`ResourceDefinition`] and every namespace, then COSI and `talosctl get`
//! consume the registry to validate requests, resolve aliases, choose default
//! namespaces, and render print columns. This [`Registry`] is the in-memory
//! analogue used by tests and higher crates.

use crate::aliases::AliasTable;
use crate::definition::ResourceDefinition;
use crate::namespaces::{self, Namespace};
use std::collections::BTreeMap;
use os_kernel::error::{Error, Result};

/// A registry of resource definitions keyed by canonical type name, plus the
/// set of registered namespaces and a derived alias index.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    definitions: BTreeMap<String, ResourceDefinition>,
    namespaces: BTreeMap<String, Namespace>,
    aliases: AliasTable,
}

impl Registry {
    /// An empty registry.
    pub fn new() -> Self {
        Registry {
            definitions: BTreeMap::new(),
            namespaces: BTreeMap::new(),
            aliases: AliasTable::new(),
        }
    }

    /// Build a registry pre-populated with every built-in namespace and the
    /// full Talos resource-type catalog. This is what Talos's `meta` namespace
    /// effectively holds after `RegisterDefaultResources`.
    pub fn with_defaults() -> Self {
        let mut reg = Registry::new();
        for ns in namespaces::ALL {
            reg.register_namespace(*ns);
        }
        for def in crate::kinds::all_definitions() {
            reg.register(def)
                .expect("built-in catalog must register cleanly");
        }
        reg
    }

    /// Register a namespace (idempotent on name).
    pub fn register_namespace(&mut self, ns: Namespace) {
        self.namespaces.insert(ns.name().to_string(), ns);
    }

    /// Register a resource definition.
    ///
    /// Fails with [`Error::InvalidState`] if a definition with the same type
    /// name is already registered, or [`Error::Invalid`] if the definition's
    /// default namespace has not been registered.
    pub fn register(&mut self, def: ResourceDefinition) -> Result<()> {
        if self.definitions.contains_key(def.type_name()) {
            return Err(Error::invalid_state(format!(
                "resource definition '{}' already registered",
                def.type_name()
            )));
        }
        if !self.namespaces.contains_key(def.default_namespace()) {
            return Err(Error::invalid(format!(
                "namespace '{}' for '{}' is not registered",
                def.default_namespace(),
                def.type_name()
            )));
        }
        self.aliases.register(&def);
        self.definitions.insert(def.type_name().to_string(), def);
        Ok(())
    }

    /// Look up a definition by its exact canonical type name.
    pub fn get(&self, type_name: &str) -> Option<&ResourceDefinition> {
        self.definitions.get(type_name)
    }

    /// Resolve a friendly name (alias/kind/type) to its definition, the way
    /// `talosctl get <name>` does. Propagates not-found and ambiguity errors.
    pub fn resolve(&self, name: &str) -> Result<&ResourceDefinition> {
        let canonical = self.aliases.resolve(name)?;
        self.definitions
            .get(canonical)
            .ok_or_else(|| Error::not_found(format!("definition '{canonical}' missing")))
    }

    /// The effective namespace for a `get` request: the user-supplied namespace
    /// if given and known, otherwise the resource's default namespace.
    ///
    /// Returns an owned `String` because the chosen namespace may come from the
    /// caller's request rather than the definition. A requested namespace that
    /// is not registered yields [`Error::NotFound`].
    pub fn effective_namespace(
        &self,
        def: &ResourceDefinition,
        requested: Option<&str>,
    ) -> Result<String> {
        match requested {
            None => Ok(def.default_namespace().to_string()),
            Some(ns) if self.namespaces.contains_key(ns) => Ok(ns.to_string()),
            Some(ns) => Err(Error::not_found(format!(
                "namespace '{ns}' is not registered"
            ))),
        }
    }

    /// Whether a namespace name is registered.
    pub fn has_namespace(&self, name: &str) -> bool {
        self.namespaces.contains_key(name)
    }

    /// Number of registered definitions.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Whether the registry has no definitions.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Iterate over all registered definitions in canonical-type order.
    pub fn definitions(&self) -> impl Iterator<Item = &ResourceDefinition> {
        self.definitions.values()
    }

    /// Iterate over all registered namespaces in name order.
    pub fn namespaces(&self) -> impl Iterator<Item = &Namespace> {
        self.namespaces.values()
    }

    /// All definitions whose default namespace is `ns`.
    pub fn in_namespace<'a>(&'a self, ns: &'a str) -> impl Iterator<Item = &'a ResourceDefinition> {
        self.definitions
            .values()
            .filter(move |d| d.default_namespace() == ns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_registry_is_populated() {
        let reg = Registry::with_defaults();
        assert!(reg.len() >= 30);
        assert!(reg.has_namespace("config"));
        assert!(reg.has_namespace("secrets"));
        assert!(!reg.is_empty());
    }

    #[test]
    fn resolve_through_registry() {
        let reg = Registry::with_defaults();
        let mc = reg.resolve("mc").unwrap();
        assert_eq!(mc.kind(), "MachineConfigs");
        assert!(mc.sensitivity().is_sensitive());

        let routes = reg.resolve("route").unwrap();
        assert_eq!(routes.kind(), "RouteStatuses");

        assert_eq!(
            reg.resolve("totally-unknown").unwrap_err().kind(),
            "not_found"
        );
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut reg = Registry::new();
        reg.register_namespace(namespaces::CONFIG);
        let d1 = ResourceDefinition::builder("MachineConfigs.config.talos.dev", "config")
            .build()
            .unwrap();
        let d2 = d1.clone();
        assert!(reg.register(d1).is_ok());
        assert_eq!(reg.register(d2).unwrap_err().kind(), "invalid_state");
    }

    #[test]
    fn registration_requires_known_namespace() {
        let mut reg = Registry::new();
        let d = ResourceDefinition::builder("Foos.x.talos.dev", "nope")
            .build()
            .unwrap();
        assert_eq!(reg.register(d).unwrap_err().kind(), "invalid");
    }

    #[test]
    fn effective_namespace_defaults_and_validates() {
        let reg = Registry::with_defaults();
        let mc = reg.resolve("mc").unwrap();
        assert_eq!(reg.effective_namespace(mc, None).unwrap(), "config");
        assert!(reg.effective_namespace(mc, Some("config")).is_ok());
        assert_eq!(
            reg.effective_namespace(mc, Some("ghost"))
                .unwrap_err()
                .kind(),
            "not_found"
        );
    }

    #[test]
    fn in_namespace_filters() {
        let reg = Registry::with_defaults();
        let secrets: Vec<_> = reg.in_namespace("secrets").collect();
        assert!(!secrets.is_empty());
        assert!(secrets.iter().all(|d| d.sensitivity().is_sensitive()));
    }

    #[test]
    fn get_returns_exact_type() {
        let reg = Registry::with_defaults();
        assert!(reg.get("MachineConfigs.config.talos.dev").is_some());
        assert!(reg.get("Nonexistent.x.talos.dev").is_none());
    }
}
