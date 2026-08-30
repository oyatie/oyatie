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
    ///   and `required` flag. New properties must be optional
    ///   (`required: false`), and the pillar annotation is immutable. On
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
    /// | [`OntologyEngineError::IncompatibleSchemaEvolution`] | A prior property was removed or mutated, or a new property is `required`. |
    /// | [`OntologyEngineError::PillarChangedOnEvolution`] | The pillar annotation differs from the stored definition's. |
    pub fn evolve_entity_type(
        &mut self,
        definition: EntityTypeDefinition,
    ) -> Result<EntityTypeId, OntologyEngineError> {
        super::check_designation_integrity(&definition)?;
        super::check_value_type_integrity(
            definition
                .properties
                .iter()
                .map(|p| (p.name.as_str(), &p.tier, p.value_type.as_ref())),
        )?;
        let key = ontology_scoped_key(&definition.tenant_id, &definition.id.value);
        match self.entity_types.get(&key) {
            None => {
                // First registration: identical to register_entity_type.
                let id = definition.id.clone();
                self.retain_entity_type_revision(&definition);
                self.entity_types.insert(key, definition);
                Ok(id)
            }
            Some(stored) => {
                // Revision monotonicity check.
                if definition.revision <= stored.revision {
                    return Err(OntologyEngineError::NonMonotonicRevision);
                }
                // Pillar immutability: link types were endpoint-validated
                // against the stored pillar; changing it would void the
                // CrossPillarLink guarantee for existing link types.
                if definition.pillar != stored.pillar {
                    return Err(OntologyEngineError::PillarChangedOnEvolution);
                }
                // Primary-key immutability: adopting a key (None -> Some) is
                // allowed; changing or removing a set key re-keys the
                // population, a breaking change.
                if stored.primary_key_property.is_some()
                    && definition.primary_key_property != stored.primary_key_property
                {
                    return Err(OntologyEngineError::PrimaryKeyChangedOnEvolution);
                }
                // Backward-compatibility check.
                check_schema_compatibility(stored, &definition)?;
                let id = definition.id.clone();
                self.retain_entity_type_revision(&definition);
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
/// - New properties in `candidate` that are absent from `prior` must be
///   optional (`required: false`): every object projected under `prior`
///   lacks them, so a required new property would invalidate the existing
///   population.
/// - The `value_type` declaration is part of the frozen quadruple: `Some`
///   is immutable, and `None -> Some` in-place typing is rejected — the
///   blessed idiom is a NEW optional typed property.
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

    let prior_names: std::collections::BTreeSet<&str> =
        prior.properties.iter().map(|p| p.name.as_str()).collect();
    for cand_prop in &candidate.properties {
        if cand_prop.required && !prior_names.contains(cand_prop.name.as_str()) {
            return Err(OntologyEngineError::IncompatibleSchemaEvolution);
        }
    }

    for prior_prop in &prior.properties {
        match candidate_map.get(prior_prop.name.as_str()) {
            None => return Err(OntologyEngineError::IncompatibleSchemaEvolution),
            Some(cand_prop) => {
                if cand_prop.tier != prior_prop.tier
                    || cand_prop.data_class != prior_prop.data_class
                    || cand_prop.required != prior_prop.required
                    || cand_prop.value_type != prior_prop.value_type
                {
                    return Err(OntologyEngineError::IncompatibleSchemaEvolution);
                }
            }
        }
    }
    Ok(())
}
