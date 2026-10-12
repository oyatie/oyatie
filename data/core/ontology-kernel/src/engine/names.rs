use crate::{ActionTypeDefinition, EntityTypeDefinition, OntologyEngineError};

pub(super) fn check_parameter_names_unique(
    definition: &ActionTypeDefinition,
) -> Result<(), OntologyEngineError> {
    let mut seen = std::collections::BTreeSet::new();
    for parameter in &definition.parameters {
        if !seen.insert(parameter.name.as_str()) {
            return Err(OntologyEngineError::DuplicateParameterName {
                name: parameter.name.clone(),
            });
        }
    }
    Ok(())
}

pub(super) fn check_property_names_unique(
    definition: &EntityTypeDefinition,
) -> Result<(), OntologyEngineError> {
    let mut names = std::collections::BTreeSet::new();
    for property in &definition.properties {
        if property.name.trim().is_empty() {
            return Err(OntologyEngineError::EmptyPropertyName);
        }
        if !names.insert(property.name.as_str()) {
            return Err(OntologyEngineError::DuplicatePropertyName {
                name: property.name.clone(),
            });
        }
    }
    Ok(())
}
