//! Ontology kernel: data-classed entities, property-tier semantics, and pillar isolation.
//!
//! # Registration invariants
//!
//! ## Endpoint-reference validation
//!
//! [`OntologyEngine::register_link_type`] and
//! [`OntologyEngine::register_action_type`] validate that every
//! [`EntityTypeId`] referenced by the definition was previously registered for
//! the same tenant via [`OntologyEngine::register_entity_type`]. A dangling
//! reference (an `EntityTypeId` that has not been registered) is rejected with
//! [`OntologyEngineError::UnknownEntityTypeEndpoint`].
//!
//! ## Pillar-consistency enforcement (Bominal-ADR-0132)
//!
//! [`OntologyEngine::register_link_type`] enforces the org/person isolation
//! boundary. If both the `from_entity_type` and `to_entity_type` carry an
//! [`OntologyPillar`] annotation (via [`EntityTypeDefinition::with_pillar`])
//! and those pillars differ, the registration is rejected with
//! [`OntologyEngineError::CrossPillarLink`]. Entity types with no pillar
//! annotation (`pillar: None`) are pillar-agnostic and do not trigger this
//! check.
//!
//! ## New error variants
//!
//! | Variant | Trigger |
//! |---------|---------|
//! | [`OntologyEngineError::UnknownEntityTypeEndpoint`] | A `LinkTypeDefinition` or `ActionTypeDefinition` references an `EntityTypeId` not registered for the same tenant. |
//! | [`OntologyEngineError::CrossPillarLink`] | A `LinkTypeDefinition` binds an org-pillar endpoint to a person-pillar endpoint (or vice versa). |
//! | [`OntologyEngineError::PillarChangedOnEvolution`] | An `evolve_entity_type` candidate changes the stored pillar annotation. |

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod pillar;
pub use pillar::{OntologyPillar, UnknownPillarLabel};

mod action_parameters;
mod definitions;
mod engine;
mod error;
mod object_graph;
mod property;
mod revisioning;
mod value;
mod value_type;

#[cfg(test)]
mod tests;

pub use action_parameters::ActionParameterDefinition;
pub use definitions::{
    ActionInvocationReceipt, ActionInvocationRequest, ActionPolicyDecision, ActionTypeDefinition,
    ActionTypeId, AutonomyTier, EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition,
    LinkCardinality, LinkTypeDefinition, LinkTypeId,
};
pub use engine::{LinkInstanceOutcome, OntologyEngine};
pub use error::OntologyEngineError;
pub use object_graph::{
    ObjectEntity, ObjectEntityUpsertOutcome, ObjectGraph, ObjectGraphError,
    ObjectPropertyUpsertOutcome,
};
pub use property::{ObjectProperty, PropertyTier};
pub use value::{CalendarDate, FiniteDouble, PropertyValue, StorageClass, ValueTypeError};
pub use value_type::{
    MAX_VALUE_TYPE_DEPTH, ScalarType, StructFieldDeclaration, StructSchema, ValueTypeDeclaration,
    ValueTypeViolation,
};
