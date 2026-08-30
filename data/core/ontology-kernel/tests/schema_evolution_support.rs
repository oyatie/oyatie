//! Shared fixtures for the split halves of this suite. Included via
//! `mod`, so this file is also discovered as an (empty) test binary.
#![allow(dead_code, unused_imports)]

pub use data_boundary_kernel::{DataClass, PrivacyDataClass};
pub use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, OntologyEngine,
    OntologyEngineError, PropertyTier,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

pub fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

pub fn pii() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::PiiIdentifying).unwrap()
}

pub fn quasi() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::PiiQuasiIdentifier).unwrap()
}

/// Build a named property definition with explicit tier/data_class/required.
pub fn prop(
    name: &str,
    tier: PropertyTier,
    data_class: PrivacyDataClass,
    required: bool,
) -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new(name, tier, data_class, required).unwrap()
}

/// Build an `EntityTypeDefinition` for tenant `ten_test` / id `ety_thing` at
/// `revision` with `name:Scalar/InternalOnly/required=true` as the base
/// property, plus any `extra_props`.
pub fn base_def(
    revision: u32,
    extra_props: Vec<EntityTypePropertyDefinition>,
) -> EntityTypeDefinition {
    let mut props = vec![prop("name", PropertyTier::Scalar, internal(), true)];
    props.extend(extra_props);
    EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_thing").unwrap(),
        "Thing",
        props,
        revision,
    )
    .unwrap()
}

/// Build a definition for a different tenant so cross-tenant isolation tests
/// have a clean second registration.
pub fn other_tenant_def(revision: u32) -> EntityTypeDefinition {
    EntityTypeDefinition::new(
        "ten_other",
        EntityTypeId::new("ety_thing").unwrap(),
        "Thing",
        vec![prop("name", PropertyTier::Scalar, internal(), true)],
        revision,
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// ST1 – additive-only backward-compatibility checker
// ---------------------------------------------------------------------------
