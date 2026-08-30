//! The tenant-scoped ontology registry engine.
//!
//! Registration methods live here; schema evolution, link-instance
//! registration, and action-invocation authorization live in the sibling
//! modules of this directory, each as its own `impl OntologyEngine` block.

mod conformance;
mod evolution;
mod invocation;
mod links;

pub use links::LinkInstanceOutcome;

#[cfg(test)]
pub(crate) use evolution::check_schema_compatibility;

use std::collections::BTreeMap;

use crate::definitions::{
    ActionTypeDefinition, ActionTypeId, EntityTypeDefinition, EntityTypeId, LinkTypeDefinition,
    LinkTypeId,
};
use crate::error::OntologyEngineError;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OntologyEngine {
    entity_types: BTreeMap<OntologyScopedKey, EntityTypeDefinition>,
    link_types: BTreeMap<OntologyScopedKey, LinkTypeDefinition>,
    action_types: BTreeMap<OntologyScopedKey, ActionTypeDefinition>,
    /// Full 4-tuple registry for idempotency checks.
    /// Key: (tenant_id, link_type_id, from_entity_id, to_entity_id)
    /// data_class: INTERNAL_ONLY
    link_instances: BTreeMap<(String, String, String, String), ()>,
    /// Outbound index: at most one outbound edge per (tenant, link_type, from) for OneToOne.
    /// Key: (tenant_id, link_type_id, from_entity_id)
    /// data_class: INTERNAL_ONLY
    link_outbound: BTreeMap<(String, String, String), ()>,
    /// Inbound index: at most one inbound edge per (tenant, link_type, to) for OneToOne/OneToMany.
    /// Key: (tenant_id, link_type_id, to_entity_id)
    /// data_class: INTERNAL_ONLY
    link_inbound: BTreeMap<(String, String, String), ()>,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct OntologyScopedKey {
    tenant_id: String,
    id: String,
}

impl OntologyEngine {
    pub fn register_entity_type(
        &mut self,
        definition: EntityTypeDefinition,
    ) -> Result<EntityTypeId, OntologyEngineError> {
        let key = ontology_scoped_key(&definition.tenant_id, &definition.id.value);
        if self.entity_types.contains_key(&key) {
            return Err(OntologyEngineError::DuplicateEntityType);
        }
        let id = definition.id.clone();
        self.entity_types.insert(key, definition);
        Ok(id)
    }

    pub fn register_link_type(
        &mut self,
        definition: LinkTypeDefinition,
    ) -> Result<LinkTypeId, OntologyEngineError> {
        // st1: endpoint-reference validation
        let from_def = self
            .entity_types
            .get(&ontology_scoped_key(
                &definition.tenant_id,
                &definition.from_entity_type.value,
            ))
            .ok_or(OntologyEngineError::UnknownEntityTypeEndpoint)?;
        let to_def = self
            .entity_types
            .get(&ontology_scoped_key(
                &definition.tenant_id,
                &definition.to_entity_type.value,
            ))
            .ok_or(OntologyEngineError::UnknownEntityTypeEndpoint)?;
        // st2: pillar-consistency enforcement (Bominal-ADR-0132)
        if let (Some(from_pillar), Some(to_pillar)) = (from_def.pillar, to_def.pillar)
            && from_pillar != to_pillar
        {
            return Err(OntologyEngineError::CrossPillarLink);
        }
        let key = ontology_scoped_key(&definition.tenant_id, &definition.id.value);
        if self.link_types.contains_key(&key) {
            return Err(OntologyEngineError::DuplicateLinkType);
        }
        let id = definition.id.clone();
        self.link_types.insert(key, definition);
        Ok(id)
    }
    pub fn register_action_type(
        &mut self,
        definition: ActionTypeDefinition,
    ) -> Result<ActionTypeId, OntologyEngineError> {
        // st1: endpoint-reference validation
        if !self.has_entity_type(&definition.tenant_id, &definition.entity_type) {
            return Err(OntologyEngineError::UnknownEntityTypeEndpoint);
        }
        let key = ontology_scoped_key(&definition.tenant_id, &definition.id.value);
        if self.action_types.contains_key(&key) {
            return Err(OntologyEngineError::DuplicateActionType);
        }
        let id = definition.id.clone();
        self.action_types.insert(key, definition);
        Ok(id)
    }
    pub fn entity_type(&self, tenant_id: &str, id: &EntityTypeId) -> Option<&EntityTypeDefinition> {
        self.entity_types
            .get(&ontology_scoped_key(tenant_id, &id.value))
    }
    /// Return the [`LinkTypeDefinition`] registered for `tenant_id` and `id`,
    /// or `None` if no such link type has been registered.
    pub fn link_type(&self, tenant_id: &str, id: &LinkTypeId) -> Option<&LinkTypeDefinition> {
        self.link_types
            .get(&ontology_scoped_key(tenant_id, &id.value))
    }
    /// Return the [`ActionTypeDefinition`] registered for `tenant_id` and `id`,
    /// or `None` if no such action type has been registered.
    pub fn action_type(&self, tenant_id: &str, id: &ActionTypeId) -> Option<&ActionTypeDefinition> {
        self.action_types
            .get(&ontology_scoped_key(tenant_id, &id.value))
    }

    fn has_entity_type(&self, tenant_id: &str, id: &EntityTypeId) -> bool {
        self.entity_types
            .contains_key(&ontology_scoped_key(tenant_id, &id.value))
    }
}

fn ontology_scoped_key(tenant_id: &str, id: &str) -> OntologyScopedKey {
    OntologyScopedKey {
        tenant_id: tenant_id.to_string(),
        id: id.to_string(),
    }
}
