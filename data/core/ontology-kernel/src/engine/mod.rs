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
    /// Every ACCEPTED entity-type definition, keyed by
    /// (tenant_id, type id, revision) — the revision history behind reader
    /// pinning and instance-to-revision binding. Rejected candidates are
    /// never retained.
    /// data_class: INTERNAL_ONLY
    entity_type_revisions: BTreeMap<(String, String, u32), EntityTypeDefinition>,
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
        check_designation_integrity(&definition)?;
        crate::display::check_display_integrity(definition.display.as_ref())?;
        for property in &definition.properties {
            crate::display::check_display_integrity(property.display.as_ref())?;
        }
        check_value_type_integrity(
            definition
                .properties
                .iter()
                .map(|p| (p.name.as_str(), &p.tier, p.value_type.as_ref())),
        )?;
        let key = ontology_scoped_key(&definition.tenant_id, &definition.id.value);
        if self.entity_types.contains_key(&key) {
            return Err(OntologyEngineError::DuplicateEntityType);
        }
        let id = definition.id.clone();
        self.retain_entity_type_revision(&definition);
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
        crate::display::check_display_integrity(definition.display.as_ref())?;
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
        crate::display::check_display_integrity(definition.display.as_ref())?;
        // Parameter-schema integrity: names are unique.
        let mut seen = std::collections::BTreeSet::new();
        for parameter in &definition.parameters {
            if !seen.insert(parameter.name.as_str()) {
                return Err(OntologyEngineError::DuplicateParameterName {
                    name: parameter.name.clone(),
                });
            }
        }
        check_value_type_integrity(
            definition
                .parameters
                .iter()
                .map(|p| (p.name.as_str(), &p.tier, p.value_type.as_ref())),
        )?;
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
    /// The definition as it was ACCEPTED at `revision`, or `None` if no
    /// evolution ever landed that exact revision for the tenant. History is
    /// retained per accepted evolution — rejected candidates never appear.
    pub fn entity_type_at_revision(
        &self,
        tenant_id: &str,
        id: &EntityTypeId,
        revision: u32,
    ) -> Option<&EntityTypeDefinition> {
        self.entity_type_revisions
            .get(&(tenant_id.to_string(), id.value.clone(), revision))
    }
    pub(crate) fn retain_entity_type_revision(&mut self, definition: &EntityTypeDefinition) {
        self.entity_type_revisions.insert(
            (
                definition.tenant_id.clone(),
                definition.id.value.clone(),
                definition.revision,
            ),
            definition.clone(),
        );
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

/// Value-type integrity: every `Some` declaration must validate and its
/// tier projection must equal the stated tier — a `Some` on a tier the
/// projection never yields (Timeseries/Geo/Ciphertext) is thereby rejected.
pub(crate) fn check_value_type_integrity<'a>(
    declarations: impl Iterator<
        Item = (
            &'a str,
            &'a crate::PropertyTier,
            Option<&'a crate::ValueTypeDeclaration>,
        ),
    >,
) -> Result<(), OntologyEngineError> {
    for (name, tier, value_type) in declarations {
        if let Some(declaration) = value_type {
            declaration
                .validate()
                .map_err(|cause| OntologyEngineError::InvalidValueType {
                    name: name.to_string(),
                    cause,
                })?;
            if &declaration.tier() != tier {
                return Err(OntologyEngineError::ValueTypeTierMismatch {
                    name: name.to_string(),
                });
            }
        }
    }
    Ok(())
}

/// Designation integrity: a primary-key or title designation must name a
/// declared property, and the key property must be `required` — a key
/// absent from a conformant instance is a contradiction.
pub(crate) fn check_designation_integrity(
    definition: &EntityTypeDefinition,
) -> Result<(), OntologyEngineError> {
    for name in [
        definition.primary_key_property.as_deref(),
        definition.title_property.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if !definition.properties.iter().any(|p| p.name == name) {
            return Err(OntologyEngineError::DesignatedPropertyNotDeclared {
                name: name.to_string(),
            });
        }
    }
    if let Some(key_name) = definition.primary_key_property.as_deref()
        && definition
            .properties
            .iter()
            .any(|p| p.name == key_name && !p.required)
    {
        return Err(OntologyEngineError::PrimaryKeyPropertyNotRequired {
            name: key_name.to_string(),
        });
    }
    Ok(())
}

fn ontology_scoped_key(tenant_id: &str, id: &str) -> OntologyScopedKey {
    OntologyScopedKey {
        tenant_id: tenant_id.to_string(),
        id: id.to_string(),
    }
}
