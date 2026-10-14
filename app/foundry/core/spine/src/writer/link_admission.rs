use data_ontology_kernel::LinkTypeId;
use foundry_edits::{EditSet, OntologyEdit};
use foundry_records_draft::ActionEnvelope;

use super::{WriteError, refuse};
use crate::error::RefusalGate;
use crate::state::ProjectionState;

pub(super) fn admit_link_endpoints(
    original: &ProjectionState,
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
        check_endpoints(
            staged,
            &envelope.tenant_id,
            link_type,
            &envelope.object_ref,
            to_entity_id,
        )?;
    }
    let prior = original
        .objects
        .get(&envelope.tenant_id, &envelope.object_ref);
    let current = staged
        .objects
        .get(&envelope.tenant_id, &envelope.object_ref);
    if prior
        .zip(current)
        .is_some_and(|(before, after)| before.entity_type.value != after.entity_type.value)
    {
        // No executable link deletion exists, so original tuples are the retained final tuples.
        for (link_type, from, to) in original.engine.link_instances(&envelope.tenant_id) {
            if from == envelope.object_ref || to == envelope.object_ref {
                check_endpoints(staged, &envelope.tenant_id, link_type, from, to)?;
            }
        }
    }
    Ok(())
}

fn check_endpoints(
    staged: &ProjectionState,
    tenant_id: &str,
    link_type: &str,
    from: &str,
    to: &str,
) -> Result<(), WriteError> {
    let definition = LinkTypeId::new(link_type)
        .ok()
        .and_then(|id| staged.engine.link_type(tenant_id, &id));
    let Some(definition) = definition else {
        return Err(endpoint_refusal());
    };
    for (reference, expected_type) in [
        (from, &definition.from_entity_type.value),
        (to, &definition.to_entity_type.value),
    ] {
        if staged
            .objects
            .get(tenant_id, reference)
            .is_none_or(|object| &object.entity_type.value != expected_type)
        {
            return Err(endpoint_refusal());
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
