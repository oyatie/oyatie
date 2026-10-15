use crate::native::native_error;
use crate::*;
use data_ontology_kernel::OntologyEngine;
use std::collections::BTreeSet;

pub(crate) fn stage_named_string_types(
    registry: &mut OntologyEngine,
    tenant: &str,
    model: &ModelDocument,
) -> Result<(), ModelError> {
    use data_ontology_kernel::{
        NamedStringTypeDefinition, NamedStringTypeId, NamedStringTypeRef, OntologyEngineError,
    };
    let mut seen = BTreeSet::new();
    for (index, row) in model.named_string_types.iter().enumerate() {
        let path = format!("named_string_types[{index}]");
        if !seen.insert((row.id.clone(), row.revision)) {
            return Err(native_error(
                &path,
                OntologyEngineError::DuplicateNamedStringType,
            ));
        }
        let id =
            NamedStringTypeId::new(&row.id).map_err(|e| native_error(&format!("{path}.id"), e))?;
        let reference = NamedStringTypeRef::new(tenant, id, row.revision)
            .map_err(|e| native_error(&format!("{path}.revision"), e))?;
        if row.revision != 1 {
            return Err(ModelError::NewDefinitionRevision {
                path: format!("{path}.revision"),
            });
        }
        let definition = NamedStringTypeDefinition::new(
            reference.clone(),
            &row.display_name,
            row.members.clone(),
        )
        .map_err(|e| {
            let failure_path = if e == OntologyEngineError::EmptyDisplayName {
                path.clone()
            } else {
                format!("{path}.members")
            };
            native_error(&failure_path, e)
        })?;
        match registry.named_string_type(&reference) {
            Some(old) if old == &definition => {}
            Some(_) => {
                return Err(native_error(
                    &path,
                    OntologyEngineError::NamedStringDefinitionMismatch,
                ));
            }
            None => {
                registry
                    .register_named_string_type(definition)
                    .map_err(|e| native_error(&path, e))?;
            }
        }
    }
    for old in registry.named_string_types(tenant) {
        if !seen.contains(&(
            old.reference().id().as_str().to_string(),
            old.reference().revision(),
        )) {
            return Err(ModelError::UnsupportedRemoval {
                path: format!("named_string_types.{}", old.reference().id().as_str()),
            });
        }
    }
    Ok(())
}
