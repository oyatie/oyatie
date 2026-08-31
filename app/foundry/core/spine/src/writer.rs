//! The ActionWriter: the ONE way anything becomes a log entry.
//!
//! Deny-by-default gates run in order — authorization, parameter
//! conformance, edit admission with an advisory dry-run of the fold's
//! own apply — then the record is canonically encoded, appended
//! receipt-by-value, and applied through THE SAME fold replay uses.
//! Determinism law: every payload and envelope byte is a pure function
//! of (request, decision, edits) — the writer never reads a clock and
//! never mints an id; `occurred_at` derives from the caller's request,
//! which is what makes the byte-identical retry contract hold.
//!
//! The registry the gates consult IS the projection's seeded snapshot
//! (`registry_input`), so the writer and the fold can never disagree
//! about the law in force.

use data_ontology_kernel::ActionInvocationReceipt;
use foundry_edits::{ActionRecord, OntologyEdit, encode_action_record};
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};

use crate::boundary;
use crate::error::{RefusalGate, Refused};
use crate::fold::{FoldOutcome, PoisonReason, apply_sealed};
use crate::state::ProjectionState;

/// One caller submission: what the Action wants to do, and the policy
/// decision that authorizes the caller to want it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionSubmission {
    pub request: data_ontology_kernel::ActionInvocationRequest, // data_class: INTERNAL_ONLY
    pub decision: data_ontology_kernel::ActionPolicyDecision,   // data_class: INTERNAL_ONLY
    pub parameters: Vec<foundry_edits::WireProperty>,           // data_class: PII_IDENTIFYING
    pub edits: foundry_edits::EditSet,                          // data_class: PII_IDENTIFYING
}

/// What became of an accepted submission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplyOutcome {
    /// Appended (or deduplicated) and applied.
    Applied { receipt: Receipt },
    /// The entry stands in the log but the projection refused it — the
    /// refusal is the projection's, honestly reported, never un-appended.
    Poisoned {
        receipt: Receipt,
        reason: PoisonReason,
    },
}

/// Why a submission produced no applied entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteError {
    /// A gate refused the submission; nothing was appended.
    Refused(Refused),
    /// The log refused the append — a divergent idempotency-key reuse
    /// surfaces loudly here, never as a silent dedup.
    Log(RecordsLogError),
}

/// Submit one Action: gate, encode, append, apply.
pub fn submit(
    submission: ActionSubmission,
    log: &mut dyn RecordsLog,
    projection: &mut ProjectionState,
) -> Result<ApplyOutcome, WriteError> {
    let registry = projection.registry_input.clone();

    // Gate 1: AUTHORIZE. Failure appends nothing, anywhere.
    let receipt = registry
        .authorize_action_invocation(submission.request.clone(), submission.decision.clone())
        .map_err(|_| {
            refuse(
                RefusalGate::Authorization,
                "policy decision does not cover this invocation",
            )
        })?;

    // Gate 2: PARAMETER CONFORMANCE against the declared schema.
    let converted = submission
        .parameters
        .iter()
        .map(boundary::property)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| {
            refuse(
                RefusalGate::Parameters,
                "parameter value unrepresentable in the kernel",
            )
        })?;
    registry
        .check_action_parameter_conformance(
            &submission.request.tenant_id,
            &submission.request.action_id,
            &converted,
        )
        .map_err(|_| {
            refuse(
                RefusalGate::Parameters,
                "parameters fail the declared schema",
            )
        })?;

    // Gate 3: EDIT ADMISSION. The action's declared entity type bounds
    // every CreateObject, and the writer stamps the CURRENT registered
    // revision — the caller never chooses it.
    let action = registry
        .action_type(&submission.request.tenant_id, &submission.request.action_id)
        .ok_or_else(|| {
            refuse(
                RefusalGate::Admission,
                "action type vanished from the registry",
            )
        })?;
    for edit in submission.edits.edits() {
        if let OntologyEdit::CreateObject { entity_type, .. } = edit
            && entity_type != &action.entity_type.value
        {
            return Err(refuse(
                RefusalGate::Admission,
                "edit entity type differs from the action's declared type",
            ));
        }
    }
    let schema_revision = registry
        .entity_type(&submission.request.tenant_id, &action.entity_type)
        .map(|definition| definition.revision)
        .unwrap_or(1);

    // ENCODE: canonical bytes, receipt fields embedded as payload law.
    let occurred_at_epoch_ms = receipt.occurred_at_epoch_seconds.saturating_mul(1000);
    let record = ActionRecord::new(
        receipt.principal_id.clone(),
        receipt.decision_id.clone(),
        receipt.audit_event_type.clone(),
        receipt.idempotency_key.clone(),
        occurred_at_epoch_ms,
        submission.parameters.clone(),
        submission.edits.clone(),
    )
    .map_err(|_| refuse(RefusalGate::Admission, "record identity fields refused"))?;
    let envelope = ActionEnvelope::new(
        receipt.tenant_id.clone(),
        receipt.entity_id.clone(),
        receipt.action_id.clone(),
        receipt.idempotency_key.clone(),
        schema_revision,
        encode_action_record(&record),
        occurred_at_epoch_ms,
    )
    .map_err(|_| refuse(RefusalGate::Admission, "envelope shape refused"))?;

    // ADVISORY DRY-RUN: the projector's own apply on a scratch copy.
    // Authoritative admission is the fold's re-check; a raced entry
    // poisons deterministically instead of corrupting state.
    let mut scratch = projection.clone();
    let probe = SealedEnvelope {
        envelope: envelope.clone(),
        receipt: Receipt {
            ordinal: scratch.applied_ordinal + 1,
            object_sequence: 0,
            deduplicated: false,
        },
    };
    if let FoldOutcome::Poisoned(_) = apply_sealed(&mut scratch, &probe) {
        return Err(refuse(
            RefusalGate::Admission,
            "edits fail the fold's own admission",
        ));
    }

    // APPEND, receipt by value — then never lie about what happened.
    let log_receipt =
        append_with_receipt(receipt, log, envelope.clone()).map_err(WriteError::Log)?;
    if log_receipt.deduplicated {
        return Ok(match projection.poison.get(&log_receipt.ordinal) {
            Some(reason) => ApplyOutcome::Poisoned {
                receipt: log_receipt,
                reason: reason.clone(),
            },
            None => ApplyOutcome::Applied {
                receipt: log_receipt,
            },
        });
    }

    // APPLY = FOLD: the same function replay uses; no second write path.
    let sealed = SealedEnvelope {
        envelope,
        receipt: log_receipt.clone(),
    };
    Ok(match apply_sealed(projection, &sealed) {
        FoldOutcome::Applied => ApplyOutcome::Applied {
            receipt: log_receipt,
        },
        FoldOutcome::Poisoned(reason) => ApplyOutcome::Poisoned {
            receipt: log_receipt,
            reason,
        },
    })
}

fn refuse(gate: RefusalGate, cause: &'static str) -> WriteError {
    WriteError::Refused(Refused { gate, cause })
}

/// The sole call site of [`RecordsLog::append`] in this crate: no
/// [`ActionInvocationReceipt`] BY VALUE, no append — receipt-gating is
/// structural, not disciplinary.
fn append_with_receipt(
    receipt: ActionInvocationReceipt,
    log: &mut dyn RecordsLog,
    envelope: ActionEnvelope,
) -> Result<Receipt, RecordsLogError> {
    debug_assert_eq!(receipt.idempotency_key, envelope.idempotency_key);
    log.append(envelope)
}
