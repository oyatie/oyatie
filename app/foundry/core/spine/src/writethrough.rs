//! Write-through: fold the log and mirror each outcome into the durable
//! projection store, so the store IS `fold(log)` rather than a second,
//! independently-maintained copy of it.
//!
//! Two laws shape this module.
//!
//! **A store failure is not a poison.** A poison is derived purely from
//! (log bytes, registry snapshot) and is therefore identical on every
//! replay; a store outage is infrastructure and is not. Recording one as
//! a poison would bake a transient failure into the projection forever.
//! So the runner HALTS on a store refusal, naming the ordinal it stopped
//! at, and the log remains the source of truth.
//!
//! **The in-memory state is disposable.** When the runner halts, the
//! fold has already consumed the entry the store refused, so `state` may
//! be one entry ahead of the store. That is safe precisely because
//! `ProjectionState` is rebuildable at any moment: a caller recovers by
//! refolding from the store's `applied_head`. What must never happen —
//! and what the suite pins — is the store holding a PARTIAL entry or a
//! fabricated poison.

use data_ontology_kernel::EntityTypeId;
use foundry_edits::{OntologyEdit, decode_action_record};
use foundry_projection_draft::{
    AppliedEntry, EntryOutcome, KeyDesignations, ProjectedLink, ProjectedObject, ProjectionStore,
    ProjectionStoreError,
};
use foundry_records_draft::{RecordsLog, RecordsLogError, SealedEnvelope};

use crate::emission::poison_label;
use crate::fold::{FoldOutcome, apply_sealed};
use crate::state::ProjectionState;
use crate::writer::{ActionSubmission, ApplyOutcome, WriteError, submit};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteThroughError {
    /// The store refused the mirror of this ordinal. Entries before it
    /// are durably applied; this one and everything after are not.
    Store {
        ordinal: u64,
        error: ProjectionStoreError,
    },
    /// The log would not hand back the entry it had just accepted, so
    /// nothing could be mirrored; the entry is in the log and the fold.
    EntryUnreadable {
        ordinal: u64,
        error: RecordsLogError,
    },
    /// The replay from the accepted ordinal did not contain it. Not
    /// reachable while the caller holds the tenant's write lock, which is
    /// what `submit_through` assumes; kept as a refusal rather than a panic.
    EntryAbsent { ordinal: u64 },
}

/// Fold `entries` into `state`, mirroring each outcome into `store`.
/// Returns how many entries were mirrored, or halts at the first store
/// refusal.
pub fn project_through(
    state: &mut ProjectionState,
    store: &mut dyn ProjectionStore,
    entries: &[SealedEnvelope],
) -> Result<u64, WriteThroughError> {
    let mut mirrored = 0;
    for sealed in entries {
        let fold = apply_sealed(state, sealed);
        mirror_folded(state, store, sealed, &fold)?;
        mirrored += 1;
    }
    Ok(mirrored)
}

/// Mirror one entry the fold has ALREADY applied to `state`. The store row
/// is built from the post-fold state, so the entry is never applied twice.
pub fn mirror_folded(
    state: &ProjectionState,
    store: &mut dyn ProjectionStore,
    sealed: &SealedEnvelope,
    fold: &FoldOutcome,
) -> Result<(), WriteThroughError> {
    let ordinal = sealed.receipt.ordinal;
    let object_ref = sealed.envelope.object_ref.as_str();
    let outcome = match fold {
        FoldOutcome::Applied => EntryOutcome::Applied {
            // One envelope is one object_ref (spine law), so the touched
            // set is at most that single object.
            objects: projected(state, object_ref).into_iter().collect(),
            // Edges are durable projection state too: without them a store
            // rebuilt from the log comes back with objects and no traversal.
            links: registered_links(sealed, object_ref),
        },
        FoldOutcome::Poisoned(reason) => EntryOutcome::Poisoned {
            reason: poison_label(reason).to_owned(),
        },
    };
    let entry = AppliedEntry {
        tenant_id: state.tenant_id.clone(),
        ordinal,
        outcome,
    };
    store
        .apply(entry, &designations(state, object_ref))
        .map_err(|error| WriteThroughError::Store { ordinal, error })?;
    Ok(())
}

/// A submission's outcome, and separately whether the durable store took
/// its mirror. The outcome stands on its own: the log accepted the write
/// and the fold applied it before the store was asked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mirrored {
    pub outcome: ApplyOutcome,                 // data_class: INTERNAL_ONLY
    pub mirror: Result<(), WriteThroughError>, // data_class: INTERNAL_ONLY
}

/// [`submit`], then mirror the accepted entry into `store`. A deduplicated
/// append mirrors nothing: the store is not consulted for it, so a mirror
/// the store missed earlier is repaired by catch-up, not by a retry.
pub fn submit_through(
    submission: ActionSubmission,
    log: &mut dyn RecordsLog,
    denial_log: &mut dyn RecordsLog,
    projection: &mut ProjectionState,
    store: &mut dyn ProjectionStore,
) -> Result<Mirrored, WriteError> {
    let outcome = submit(submission, log, denial_log, projection)?;
    let (receipt, fold) = match &outcome {
        ApplyOutcome::Applied { receipt } => (receipt, FoldOutcome::Applied),
        ApplyOutcome::Poisoned { receipt, reason } => {
            (receipt, FoldOutcome::Poisoned(reason.clone()))
        }
    };
    if receipt.deduplicated {
        return Ok(Mirrored {
            outcome,
            mirror: Ok(()),
        });
    }
    let ordinal = receipt.ordinal;
    let mirror = log
        .replay(&projection.tenant_id, ordinal)
        .map_err(|error| WriteThroughError::EntryUnreadable { ordinal, error })
        .and_then(|entries| {
            entries
                .iter()
                .find(|sealed| sealed.receipt.ordinal == ordinal)
                .map(|sealed| mirror_folded(projection, store, sealed, &fold))
                .unwrap_or(Err(WriteThroughError::EntryAbsent { ordinal }))
        });
    Ok(Mirrored { outcome, mirror })
}

/// The edges this entry registered. `CreateLink` is owned by the FROM
/// endpoint — the envelope's own object — so the source is the
/// envelope's `object_ref` and never a payload field that could
/// disagree with it. An entry the fold APPLIED necessarily decoded, so
/// a decode failure here is unreachable and yields no edges rather
/// than a panic.
fn registered_links(sealed: &SealedEnvelope, object_ref: &str) -> Vec<ProjectedLink> {
    let Ok(record) = decode_action_record(&sealed.envelope.payload) else {
        return Vec::new();
    };
    record
        .edits
        .edits()
        .iter()
        .filter_map(|edit| match edit {
            OntologyEdit::CreateLink {
                link_type,
                to_entity_id,
            } => Some(ProjectedLink {
                link_type: link_type.clone(),
                from_object_ref: object_ref.to_owned(),
                to_object_ref: to_entity_id.clone(),
                // The record's own timestamp, so an edge's freshness is
                // a fact of the log rather than of when a projector
                // happened to run.
                observed_at_epoch_ms: record.occurred_at_epoch_ms,
            }),
            _ => None,
        })
        .collect()
}

fn projected(state: &ProjectionState, object_ref: &str) -> Option<ProjectedObject> {
    let binding = state.bindings.get(object_ref)?;
    let entity = state.objects.get(&state.tenant_id, object_ref)?;
    Some(ProjectedObject {
        entity: entity.clone(),
        schema_revision: binding.schema_revision,
        last_ordinal: binding.last_ordinal,
        last_actor: binding.last_actor.clone(),
    })
}

/// Stamp the registry's key designation for the type this entry touched.
/// The store owns no definitions, so identity law reaches it only
/// because the projector — which DOES hold the registry — passes it in.
fn designations(state: &ProjectionState, object_ref: &str) -> KeyDesignations {
    let keys = KeyDesignations::default();
    let Some(binding) = state.bindings.get(object_ref) else {
        return keys;
    };
    let Ok(entity_type_id) = EntityTypeId::new(binding.entity_type.clone()) else {
        return keys;
    };
    let Some(definition) = state
        .registry_input
        .entity_type(&state.tenant_id, &entity_type_id)
    else {
        return keys;
    };
    match &definition.primary_key_property {
        Some(property) => keys.declaring(binding.entity_type.clone(), property.clone()),
        None => keys,
    }
}
