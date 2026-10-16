//! The payload roots: what an appended envelope's bytes describe.

use crate::edit::{EditSet, OntologyEdit};
use crate::property::WireProperty;

pub const WIRE_FORMAT_VERSION: u16 = 1;
pub const DEFAULTED_ACTION_FORMAT_VERSION: u16 = 2;

/// Why a payload root was refused at construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordError {
    Empty { field: &'static str },
    NotTrimmed { field: &'static str },
    UnchangedSubmittedEdits,
    InvalidDefaultEdits,
}

/// One applied Action, as its payload bytes describe it.
///
/// Version 2 retains submitted edits alongside materialized creation defaults,
/// so retries compare original input while replay applies explicit values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionRecord {
    pub wire_format_version: u16,         // data_class: INTERNAL_ONLY
    pub principal_id: String,             // data_class: INTERNAL_ONLY
    pub decision_id: String,              // data_class: INTERNAL_ONLY
    pub audit_event_type: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_ms: u64,        // data_class: INTERNAL_ONLY
    pub parameters: Vec<WireProperty>,    // data_class: PII_IDENTIFYING
    pub edits: EditSet,                   // data_class: PII_IDENTIFYING
    pub submitted_edits: Option<EditSet>, // data_class: PII_IDENTIFYING
}

impl ActionRecord {
    pub fn new(
        principal_id: impl Into<String>,
        decision_id: impl Into<String>,
        audit_event_type: impl Into<String>,
        idempotency_key: impl Into<String>,
        occurred_at_epoch_ms: u64,
        parameters: Vec<WireProperty>,
        edits: EditSet,
    ) -> Result<Self, RecordError> {
        let record = Self {
            wire_format_version: WIRE_FORMAT_VERSION,
            principal_id: principal_id.into(),
            decision_id: decision_id.into(),
            audit_event_type: audit_event_type.into(),
            idempotency_key: idempotency_key.into(),
            occurred_at_epoch_ms,
            parameters,
            edits,
            submitted_edits: None,
        };
        validate_identity_fields([
            ("principal_id", &record.principal_id),
            ("decision_id", &record.decision_id),
            ("audit_event_type", &record.audit_event_type),
            ("idempotency_key", &record.idempotency_key),
        ])?;
        Ok(record)
    }

    pub fn with_submitted_edits(mut self, edits: EditSet) -> Result<Self, RecordError> {
        if edits == self.edits {
            return Err(RecordError::UnchangedSubmittedEdits);
        }
        if !default_extension(&edits, &self.edits) {
            return Err(RecordError::InvalidDefaultEdits);
        }
        self.wire_format_version = DEFAULTED_ACTION_FORMAT_VERSION;
        self.submitted_edits = Some(edits);
        Ok(self)
    }
}

fn default_extension(submitted: &EditSet, effective: &EditSet) -> bool {
    submitted.edits().len() == effective.edits().len()
        && submitted
            .edits()
            .iter()
            .zip(effective.edits())
            .all(|(before, after)| match (before, after) {
                (
                    OntologyEdit::CreateObject {
                        entity_type: original_type,
                        properties: original,
                    },
                    OntologyEdit::CreateObject {
                        entity_type,
                        properties,
                    },
                ) => {
                    if original_type != entity_type || !properties.starts_with(original) {
                        return false;
                    }
                    let mut names: std::collections::BTreeSet<_> = original
                        .iter()
                        .map(|property| property.name.as_str())
                        .collect();
                    properties[original.len()..]
                        .iter()
                        .all(|property| names.insert(property.name.as_str()))
                }
                _ => before == after,
            })
}

/// One refused Action, as the SEPARATE audit log's payload bytes describe
/// it. A denial has no receipt and never consumes a tenant object
/// ordinal; the cause is a static label, never a classified value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DenialRecord {
    pub wire_format_version: u16,  // data_class: INTERNAL_ONLY
    pub gate: String,              // data_class: INTERNAL_ONLY
    pub cause: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,      // data_class: INTERNAL_ONLY
    pub decision_id: String,       // data_class: INTERNAL_ONLY
    pub action_id: String,         // data_class: INTERNAL_ONLY
    pub object_ref: String,        // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_ms: u64, // data_class: INTERNAL_ONLY
}

impl DenialRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gate: impl Into<String>,
        cause: impl Into<String>,
        principal_id: impl Into<String>,
        decision_id: impl Into<String>,
        action_id: impl Into<String>,
        object_ref: impl Into<String>,
        occurred_at_epoch_ms: u64,
    ) -> Result<Self, RecordError> {
        let record = Self {
            wire_format_version: WIRE_FORMAT_VERSION,
            gate: gate.into(),
            cause: cause.into(),
            principal_id: principal_id.into(),
            decision_id: decision_id.into(),
            action_id: action_id.into(),
            object_ref: object_ref.into(),
            occurred_at_epoch_ms,
        };
        validate_identity_fields([
            ("gate", &record.gate),
            ("cause", &record.cause),
            ("principal_id", &record.principal_id),
            ("decision_id", &record.decision_id),
            ("action_id", &record.action_id),
            ("object_ref", &record.object_ref),
        ])?;
        Ok(record)
    }
}

fn validate_identity_fields<const N: usize>(
    fields: [(&'static str, &String); N],
) -> Result<(), RecordError> {
    for (field, value) in fields {
        if value.trim().is_empty() {
            return Err(RecordError::Empty { field });
        }
        if value.trim() != value {
            return Err(RecordError::NotTrimmed { field });
        }
    }
    Ok(())
}
