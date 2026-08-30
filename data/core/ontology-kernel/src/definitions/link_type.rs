//! Link-type definitions: typed, cardinality-bearing edges.

use crate::error::OntologyEngineError;

use super::identifiers::{EntityTypeId, LinkCardinality, LinkTypeId, validate_ontology_tenant};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkTypeDefinition {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub id: LinkTypeId,                 // data_class: INTERNAL_ONLY
    pub from_entity_type: EntityTypeId, // data_class: INTERNAL_ONLY
    pub to_entity_type: EntityTypeId,   // data_class: INTERNAL_ONLY
    pub cardinality: LinkCardinality,   // data_class: INTERNAL_ONLY
    pub allow_cross_tenant: bool,       // data_class: INTERNAL_ONLY
    pub revision: u32,                  // data_class: INTERNAL_ONLY
}

impl LinkTypeDefinition {
    pub fn new(
        tenant_id: impl Into<String>,
        id: LinkTypeId,
        from_entity_type: EntityTypeId,
        to_entity_type: EntityTypeId,
        cardinality: LinkCardinality,
        allow_cross_tenant: bool,
    ) -> Result<Self, OntologyEngineError> {
        let tenant_id = tenant_id.into();
        validate_ontology_tenant(&tenant_id)?;
        Ok(Self {
            tenant_id,
            id,
            from_entity_type,
            to_entity_type,
            cardinality,
            allow_cross_tenant,
            revision: 1,
        })
    }
}
