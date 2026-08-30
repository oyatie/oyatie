//! Instance-vs-type conformance: validate an [`ObjectEntity`](crate::ObjectEntity)
//! against the [`EntityTypeDefinition`](crate::EntityTypeDefinition) registered
//! for its tenant.
//!
//! # The identity join
//!
//! A conformant instance carries the registered [`EntityTypeId`](crate::EntityTypeId)
//! **value** (`ety_`-prefixed) as its `entity_type`, and a `ten_`-prefixed
//! tenant id — the same `(tenant_id, type id)` key the registry uses. An
//! entity written with any other vocabulary resolves to no definition and is
//! rejected with [`OntologyEngineError::UnknownEntityType`](crate::OntologyEngineError::UnknownEntityType).

use data_boundary_kernel::DataClassification;

use crate::error::OntologyEngineError;
use crate::object_graph::ObjectEntity;

use super::{OntologyEngine, ontology_scoped_key};

impl OntologyEngine {
    /// Check that `entity` conforms to the entity type definition registered
    /// for `(entity.tenant_id, entity.entity_type)`.
    ///
    /// Fail-closed contract, in check order:
    ///
    /// | Error | Condition |
    /// |-------|-----------|
    /// | [`OntologyEngineError::UnknownEntityType`](crate::OntologyEngineError::UnknownEntityType) | No definition registered under the entity's `(tenant_id, entity_type)`. |
    /// | [`OntologyEngineError::MissingRequiredProperty`](crate::OntologyEngineError::MissingRequiredProperty) | A definition property with `required: true` is absent from the instance. |
    /// | [`OntologyEngineError::UndeclaredProperty`](crate::OntologyEngineError::UndeclaredProperty) | The instance carries a property the definition does not declare. |
    /// | [`OntologyEngineError::PropertyTierMismatch`](crate::OntologyEngineError::PropertyTierMismatch) | An instance property's tier differs from the declared tier. |
    /// | [`OntologyEngineError::PropertyDataClassMismatch`](crate::OntologyEngineError::PropertyDataClassMismatch) | An instance property's data class differs from the declared class. |
    pub fn check_instance_conformance(
        &self,
        entity: &ObjectEntity,
    ) -> Result<(), OntologyEngineError> {
        let definition = self
            .entity_types
            .get(&ontology_scoped_key(
                &entity.tenant_id,
                &entity.entity_type.value,
            ))
            .ok_or(OntologyEngineError::UnknownEntityType)?;

        for declared in &definition.properties {
            if declared.required && !entity.properties.contains_key(&declared.name) {
                return Err(OntologyEngineError::MissingRequiredProperty {
                    name: declared.name.clone(),
                });
            }
        }

        for (name, property) in &entity.properties {
            let Some(declared) = definition.properties.iter().find(|p| &p.name == name) else {
                return Err(OntologyEngineError::UndeclaredProperty { name: name.clone() });
            };
            if property.tier != declared.tier {
                return Err(OntologyEngineError::PropertyTierMismatch { name: name.clone() });
            }
            if property.value.data_class != DataClassification::from(declared.data_class) {
                return Err(OntologyEngineError::PropertyDataClassMismatch { name: name.clone() });
            }
        }
        Ok(())
    }
}
