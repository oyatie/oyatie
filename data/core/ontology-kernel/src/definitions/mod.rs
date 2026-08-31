//! Ontology type-plane definitions, split by definition kind. The module
//! path is the crate's stable internal vocabulary: everything re-exports
//! here, so `crate::definitions::X` never changes as files split.

mod action_type;
mod entity_type;
mod identifiers;
mod link_type;

pub use action_type::{
    ActionInvocationReceipt, ActionInvocationRequest, ActionPolicyDecision, ActionTypeDefinition,
};
pub use entity_type::{EntityTypeDefinition, EntityTypePropertyDefinition};
pub use identifiers::{ActionTypeId, AutonomyTier, EntityTypeId, LinkCardinality, LinkTypeId};
pub use link_type::LinkTypeDefinition;

pub(crate) use identifiers::validate_ontology_tenant;
