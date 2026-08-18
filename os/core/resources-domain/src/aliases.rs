//! Alias resolution.
//!
//! `talosctl get <name>` accepts a friendly name (`mc`, `routes`, `members`,
//! ...) and must resolve it to a canonical resource type. Talos builds this map
//! from every registered RD's `Aliases` plus its kind and full type name. This
//! module provides an [`AliasTable`] that indexes a set of
//! [`ResourceDefinition`]s and resolves user input, reporting ambiguities.

use crate::definition::ResourceDefinition;
use os_kernel::error::{Error, Result};
use std::collections::BTreeMap;

/// A case-insensitive index from friendly names to canonical type names.
#[derive(Debug, Clone, Default)]
pub struct AliasTable {
    // alias (lowercased) -> set of canonical type names that claim it.
    index: BTreeMap<String, Vec<String>>,
}

impl AliasTable {
    /// An empty table.
    pub fn new() -> Self {
        AliasTable {
            index: BTreeMap::new(),
        }
    }

    /// Build a table from a slice of definitions, indexing each by its full
    /// type name, kind, and declared aliases.
    pub fn from_definitions(defs: &[ResourceDefinition]) -> Self {
        let mut table = AliasTable::new();
        for d in defs {
            table.register(d);
        }
        table
    }

    /// Index one definition's names. Idempotent per (alias, type) pair.
    pub fn register(&mut self, def: &ResourceDefinition) {
        let canonical = def.type_name().to_string();
        let mut names: Vec<String> = Vec::new();
        names.push(def.type_name().to_ascii_lowercase());
        names.push(def.kind().to_ascii_lowercase());
        for a in def.aliases() {
            names.push(a.clone());
        }
        for name in names {
            let entry = self.index.entry(name).or_default();
            if !entry.contains(&canonical) {
                entry.push(canonical.clone());
            }
        }
    }

    /// Resolve a friendly name to a single canonical type name.
    ///
    /// Returns [`Error::NotFound`] if nothing matches and
    /// [`Error::Invalid`] if the name is ambiguous between several types.
    pub fn resolve(&self, name: &str) -> Result<&str> {
        let key = name.to_ascii_lowercase();
        match self.index.get(&key) {
            None => Err(Error::not_found(format!("no resource matches '{name}'"))),
            Some(types) if types.len() == 1 => Ok(types[0].as_str()),
            Some(types) => Err(Error::invalid(format!(
                "'{name}' is ambiguous between {}",
                types.join(", ")
            ))),
        }
    }

    /// Whether a friendly name resolves to exactly one type.
    pub fn is_unambiguous(&self, name: &str) -> bool {
        matches!(self.index.get(&name.to_ascii_lowercase()), Some(t) if t.len() == 1)
    }

    /// The number of distinct friendly names indexed.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defs() -> Vec<ResourceDefinition> {
        vec![
            ResourceDefinition::builder("MachineConfigs.config.talos.dev", "config")
                .aliases(["machineconfig", "mc"])
                .build()
                .unwrap(),
            ResourceDefinition::builder("RouteStatuses.net.talos.dev", "network")
                .aliases(["route", "routes"])
                .build()
                .unwrap(),
        ]
    }

    #[test]
    fn resolves_by_alias_kind_and_type() {
        let t = AliasTable::from_definitions(&defs());
        assert_eq!(t.resolve("mc").unwrap(), "MachineConfigs.config.talos.dev");
        assert_eq!(
            t.resolve("MachineConfigs").unwrap(),
            "MachineConfigs.config.talos.dev"
        );
        assert_eq!(
            t.resolve("machineconfigs.config.talos.dev").unwrap(),
            "MachineConfigs.config.talos.dev"
        );
        assert_eq!(t.resolve("routes").unwrap(), "RouteStatuses.net.talos.dev");
    }

    #[test]
    fn resolution_is_case_insensitive() {
        let t = AliasTable::from_definitions(&defs());
        assert_eq!(t.resolve("MC").unwrap(), "MachineConfigs.config.talos.dev");
        assert!(t.is_unambiguous("Route"));
    }

    #[test]
    fn unknown_name_is_not_found() {
        let t = AliasTable::from_definitions(&defs());
        let err = t.resolve("nope").unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn ambiguous_alias_is_reported() {
        let conflicting = vec![
            ResourceDefinition::builder("Members.cluster.talos.dev", "cluster")
                .alias("m")
                .build()
                .unwrap(),
            ResourceDefinition::builder("Mounts.runtime.talos.dev", "runtime")
                .alias("m")
                .build()
                .unwrap(),
        ];
        let t = AliasTable::from_definitions(&conflicting);
        assert!(!t.is_unambiguous("m"));
        let err = t.resolve("m").unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }
}
