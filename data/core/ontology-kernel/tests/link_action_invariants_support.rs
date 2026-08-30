//! Shared fixtures for the split halves of this suite. Included via
//! `mod`, so this file is also discovered as an (empty) test binary.
#![allow(dead_code, unused_imports)]

pub use data_boundary_kernel::{DataClass, PrivacyDataClass};
pub use data_ontology_kernel::{
    ActionTypeDefinition, ActionTypeId, AutonomyTier, EntityTypeDefinition, EntityTypeId,
    EntityTypePropertyDefinition, LinkCardinality, LinkTypeDefinition, LinkTypeId, OntologyEngine,
    OntologyEngineError, OntologyPillar, PropertyTier,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

pub fn prop(name: &str) -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new(
        name,
        PropertyTier::Scalar,
        PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
        true,
    )
    .unwrap()
}

pub fn entity(tenant: &str, id: &str, display: &str) -> EntityTypeDefinition {
    EntityTypeDefinition::new(
        tenant,
        EntityTypeId::new(id).unwrap(),
        display,
        vec![prop("name")],
        1,
    )
    .unwrap()
}

pub fn entity_with_pillar(
    tenant: &str,
    id: &str,
    display: &str,
    pillar: OntologyPillar,
) -> EntityTypeDefinition {
    entity(tenant, id, display).with_pillar(pillar)
}

// ---------------------------------------------------------------------------
// st1 – cross-tenant endpoint isolation
// ---------------------------------------------------------------------------
