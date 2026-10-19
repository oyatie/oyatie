use data_ontology_kernel::EntityTypeId;
use foundry_edits::{EditSet, OntologyEdit, WireProperty};

use super::{ActionSubmission, WriteError, refuse};
use crate::{boundary, error::RefusalGate, state::ProjectionState};

pub(super) fn materialize(
    state: &ProjectionState,
    submission: &ActionSubmission,
) -> Result<EditSet, WriteError> {
    let tenant = &submission.request.tenant_id;
    let mut exists = state
        .objects
        .get(tenant, &submission.request.entity_id)
        .is_some();
    let mut edits = submission.edits.edits().to_vec();
    for edit in &mut edits {
        let OntologyEdit::CreateObject {
            entity_type,
            properties,
        } = edit
        else {
            continue;
        };
        if !exists {
            let id = EntityTypeId::new(entity_type.clone()).map_err(|_| refused())?;
            let definition = state
                .registry_input
                .entity_type(tenant, &id)
                .ok_or_else(refused)?;
            for declared in &definition.properties {
                if properties
                    .iter()
                    .any(|property| property.name == declared.name)
                {
                    continue;
                }
                let Some(value) = &declared.default_value else {
                    continue;
                };
                properties.push(
                    WireProperty::new(
                        declared.name.clone(),
                        boundary::wire_tier(declared.tier),
                        boundary::wire_label(&declared.data_class).ok_or_else(refused)?,
                        boundary::wire_value(value).map_err(|_| refused())?,
                    )
                    .map_err(|_| refused())?,
                );
            }
        }
        exists = true;
    }
    EditSet::new(edits).map_err(|_| refused())
}

fn refused() -> WriteError {
    refuse(
        RefusalGate::Admission,
        "creation defaults are not representable",
    )
}
