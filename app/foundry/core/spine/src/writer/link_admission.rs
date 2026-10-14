use data_ontology_kernel::LinkTypeId;
use foundry_edits::{EditSet, OntologyEdit};
use foundry_records_draft::ActionEnvelope;

use super::{WriteError, refuse};
use crate::error::RefusalGate;
use crate::state::ProjectionState;

pub(super) fn admit_created_link_endpoints(
    staged: &ProjectionState,
    envelope: &ActionEnvelope,
    edits: &EditSet,
) -> Result<(), WriteError> {
    for edit in edits.edits() {
        let OntologyEdit::CreateLink {
            link_type,
            to_entity_id,
        } = edit
        else {
            continue;
        };
        let definition = LinkTypeId::new(link_type)
            .ok()
            .and_then(|id| staged.engine.link_type(&envelope.tenant_id, &id));
        let Some(definition) = definition else {
            return Err(endpoint_refusal());
        };
        for (reference, expected_type) in [
            (&envelope.object_ref, &definition.from_entity_type.value),
            (to_entity_id, &definition.to_entity_type.value),
        ] {
            if staged
                .objects
                .get(&envelope.tenant_id, reference)
                .is_none_or(|object| &object.entity_type.value != expected_type)
            {
                return Err(endpoint_refusal());
            }
        }
    }
    Ok(())
}

fn endpoint_refusal() -> WriteError {
    refuse(
        RefusalGate::Admission,
        "link endpoints do not match the declared relationship",
    )
}
