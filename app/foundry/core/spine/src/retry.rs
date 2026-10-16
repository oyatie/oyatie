//! A retry is the same request again, whatever decision authorized it.
//!
//! The policy decision point mints a decision per request, and the record
//! carries the decision that first authorized the write. A retry of that
//! request arrives with a new decision id and otherwise identical content;
//! the writer appends nothing and answers the original receipt as
//! deduplicated, so the log keeps the first decision. Any other difference
//! is divergent content under a spent key, which the log refuses.

use foundry_edits::decode_action_record;
use foundry_records_draft::ActionEnvelope;

/// True when `fresh` is `stored` re-sent: every envelope field equal and every
/// record field equal but the decision id. Undecodable stored bytes are not
/// the same request.
pub fn same_request(stored: &ActionEnvelope, fresh: &ActionEnvelope) -> bool {
    let envelope_fields_equal = stored.tenant_id == fresh.tenant_id
        && stored.object_ref == fresh.object_ref
        && stored.action_type == fresh.action_type
        && stored.idempotency_key == fresh.idempotency_key
        && stored.schema_revision == fresh.schema_revision
        && stored.observed_at_epoch_ms == fresh.observed_at_epoch_ms;
    if !envelope_fields_equal {
        return false;
    }
    let (Ok(theirs), Ok(ours)) = (
        decode_action_record(&stored.payload),
        decode_action_record(&fresh.payload),
    ) else {
        return false;
    };
    theirs.wire_format_version == ours.wire_format_version
        && theirs.principal_id == ours.principal_id
        && theirs.audit_event_type == ours.audit_event_type
        && theirs.idempotency_key == ours.idempotency_key
        && theirs.occurred_at_epoch_ms == ours.occurred_at_epoch_ms
        && theirs.parameters == ours.parameters
        && theirs.edits == ours.edits
        && theirs.submitted_edits == ours.submitted_edits
}

pub(crate) fn same_submission(
    registry: &data_ontology_kernel::OntologyEngine,
    stored: &ActionEnvelope,
    submission: &crate::writer::ActionSubmission,
    receipt: &data_ontology_kernel::ActionInvocationReceipt,
) -> bool {
    let request = &submission.request;
    let retained = registry
        .action_type(&request.tenant_id, &request.action_id)
        .and_then(|action| {
            registry.entity_type_at_revision(
                &request.tenant_id,
                &action.entity_type,
                stored.schema_revision,
            )
        })
        .is_some();
    if !retained
        || stored.tenant_id != request.tenant_id
        || stored.object_ref != request.entity_id
        || stored.action_type != request.action_id.value
        || stored.idempotency_key != request.idempotency_key
        || stored.observed_at_epoch_ms != receipt.occurred_at_epoch_seconds.saturating_mul(1000)
    {
        return false;
    }
    let Ok(record) = decode_action_record(&stored.payload) else {
        return false;
    };
    record.principal_id == receipt.principal_id
        && record.audit_event_type == receipt.audit_event_type
        && record.idempotency_key == request.idempotency_key
        && record.occurred_at_epoch_ms == stored.observed_at_epoch_ms
        && record.parameters == submission.parameters
        && record.submitted_edits.as_ref().unwrap_or(&record.edits) == &submission.edits
}
