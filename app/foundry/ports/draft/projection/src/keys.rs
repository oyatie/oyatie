//! Primary-key designations: which property identifies an object of a
//! given entity type.
//!
//! This is REGISTRY knowledge, not projection state. The registry is
//! fold input, so the designation arrives as an apply parameter rather
//! than as a stored field — which keeps the canonical entry bytes, and
//! therefore dedup identity, exactly what they were before keys
//! existed. A store enforces uniqueness without ever owning a
//! definition.

use std::collections::BTreeMap;

/// The declared key property per entity type, stamped by the projector.
/// A type absent from this map declares no key and is unconstrained.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyDesignations {
    by_entity_type: BTreeMap<String, String>,
}

impl KeyDesignations {
    /// Declare `property` as the key of `entity_type`.
    pub fn declaring(
        mut self,
        entity_type: impl Into<String>,
        property: impl Into<String>,
    ) -> Self {
        self.by_entity_type
            .insert(entity_type.into(), property.into());
        self
    }

    /// The key property of `entity_type`, if it declares one.
    pub fn property_for(&self, entity_type: &str) -> Option<&str> {
        self.by_entity_type
            .get(entity_type)
            .map(|property| property.as_str())
    }

    /// Whether any type declares a key — lets a store skip the whole
    /// uniqueness pass when nothing is keyed.
    pub fn is_empty(&self) -> bool {
        self.by_entity_type.is_empty()
    }
}
