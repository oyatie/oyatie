//! Schema evolution for entity type definitions: strict revision
//! monotonicity and the additive-only backward-compatibility contract.

use crate::definitions::{EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition};
use crate::error::OntologyEngineError;

use super::{OntologyEngine, ontology_scoped_key};

impl OntologyEngine {
    /// Register or evolve an entity type definition.
    ///
    /// - **First registration** (id unknown for the tenant): behaves identically
    ///   to [`register_entity_type`](Self::register_entity_type) — inserts the
    ///   definition and returns `Ok(id)`. `DuplicateEntityType` is never
    ///   returned by this method.
    /// - **Evolution** (id already registered): requires
    ///   `definition.revision > stored.revision` (strict monotonicity) and that
    ///   every prior property is retained with unchanged `tier`, `data_class`,
    ///   and `required` flag. New properties may be introduced freely. On
    ///   success the stored definition is replaced with `definition`.
    ///
    /// # Errors
    ///
    /// | Error | Condition |
    /// |-------|-----------|
    /// | [`OntologyEngineError::InvalidTenantId`] | Tenant id fails prefix check. |
    /// | [`OntologyEngineError::EmptyDisplayName`] | `display_name` is blank. |
    /// | [`OntologyEngineError::EmptyProperties`] | `properties` is empty. |
    /// | [`OntologyEngineError::EmptyPropertyName`] | A property name is blank. |
    /// | [`OntologyEngineError::NonMonotonicRevision`] | `definition.revision <= stored.revision`. |
    /// | [`OntologyEngineError::IncompatibleSchemaEvolution`] | A prior property was removed or mutated. |
    pub fn evolve_entity_type(
        &mut self,
        definition: EntityTypeDefinition,
    ) -> Result<EntityTypeId, OntologyEngineError> {
        let key = ontology_scoped_key(&definition.tenant_id, &definition.id.value);
        match self.entity_types.get(&key) {
            None => {
                // First registration: identical to register_entity_type.
                let id = definition.id.clone();
                self.entity_types.insert(key, definition);
                Ok(id)
            }
            Some(stored) => {
                // Revision monotonicity check.
                if definition.revision <= stored.revision {
                    return Err(OntologyEngineError::NonMonotonicRevision);
                }
                // Backward-compatibility check.
                check_schema_compatibility(stored, &definition)?;
                let id = definition.id.clone();
                self.entity_types.insert(key, definition);
                Ok(id)
            }
        }
    }
}

///
/// Rules:
/// - Every property in `prior` must exist in `candidate` with identical
///   `tier`, `data_class`, and `required` flag.
/// - New properties in `candidate` that are absent from `prior` are permitted.
/// - Revision monotonicity is **not** checked here; the caller is responsible.
pub(crate) fn check_schema_compatibility(
    prior: &EntityTypeDefinition,
    candidate: &EntityTypeDefinition,
) -> Result<(), OntologyEngineError> {
    // Build a lookup map from the candidate's property list.
    let candidate_map: std::collections::BTreeMap<&str, &EntityTypePropertyDefinition> = candidate
        .properties
        .iter()
        .map(|p| (p.name.as_str(), p))
        .collect();

    for prior_prop in &prior.properties {
        match candidate_map.get(prior_prop.name.as_str()) {
            None => return Err(OntologyEngineError::IncompatibleSchemaEvolution),
            Some(cand_prop) => {
                if cand_prop.tier != prior_prop.tier
                    || cand_prop.data_class != prior_prop.data_class
                    || cand_prop.required != prior_prop.required
                {
                    return Err(OntologyEngineError::IncompatibleSchemaEvolution);
                }
            }
        }
    }
    Ok(())
}
