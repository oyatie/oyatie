use crate::{OntologyEngine, OntologyEngineError};

impl OntologyEngine {
    pub(super) fn check_property_defaults(
        &self,
        definition: &crate::EntityTypeDefinition,
    ) -> Result<(), OntologyEngineError> {
        for property in &definition.properties {
            if let Some(value) = &property.default_value {
                super::value_conformance::admits_declared_value(
                    property.value_type.as_ref(),
                    value,
                )
                .map_err(|cause| OntologyEngineError::InvalidPropertyDefault {
                    name: property.name.clone(),
                    cause,
                })?;
            }
        }
        Ok(())
    }

    /// Requiring the projection to equal the stated tier rejects a `Some` on
    /// any tier the projection never yields (Timeseries/Geo/Ciphertext).
    pub(super) fn check_value_type_integrity<'a>(
        &self,
        tenant: &str,
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
                if let crate::ValueTypeDeclaration::NamedString(definition) = declaration {
                    if definition.reference().tenant_id() != tenant {
                        return Err(OntologyEngineError::TenantMismatch);
                    }
                    if self.named_string_type(definition.reference()) != Some(definition) {
                        return Err(OntologyEngineError::NamedStringDefinitionMismatch);
                    }
                }
                if &declaration.tier() != tier {
                    return Err(OntologyEngineError::ValueTypeTierMismatch {
                        name: name.to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}
