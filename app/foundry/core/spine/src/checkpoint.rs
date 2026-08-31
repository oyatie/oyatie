//! Checkpoints and sync status: resume without refolding the world, and
//! say honestly how far behind the log the projection is.
//!
//! A V1 checkpoint is the in-memory projection itself, bound to the
//! registry snapshot it was folded against — held verbatim and compared
//! by `Eq`, which mechanizes the invalidation rule (a different registry
//! input discards the checkpoint and refolds from ordinal 1, so
//! revision-ahead poisons un-poison consistently after an evolution)
//! with no invented digest byte-law. A content digest becomes
//! load-bearing only with the durable-checkpoint follow-up, and arrives
//! with it.

use data_ontology_kernel::OntologyEngine;
use foundry_records_draft::SealedEnvelope;

use crate::fold::apply_sealed;
use crate::state::ProjectionState;

/// A resumable snapshot of one tenant's projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    state: ProjectionState,
}

impl Checkpoint {
    /// Capture the projection as it stands.
    pub fn capture(state: &ProjectionState) -> Self {
        Self {
            state: state.clone(),
        }
    }

    /// The ordinal this checkpoint has applied through.
    pub fn applied_ordinal(&self) -> u64 {
        self.state.applied_ordinal
    }

    /// Resume from this checkpoint against `registry`, folding `entries`
    /// (a full replay from ordinal 1 — entries at or below the
    /// checkpoint are skipped without refolding).
    ///
    /// If `registry` is not the exact snapshot the checkpoint was folded
    /// against, the checkpoint is DISCARDED and the whole replay refolds
    /// from scratch — resume must never produce a state a fresh fold
    /// would not.
    pub fn resume<'a>(
        &self,
        registry: &OntologyEngine,
        entries: impl IntoIterator<Item = &'a SealedEnvelope>,
    ) -> ProjectionState {
        if self.state.registry_input != *registry {
            return crate::fold::fold_from_scratch(&self.state.tenant_id, registry, entries);
        }
        let mut state = self.state.clone();
        for sealed in entries {
            if sealed.receipt.ordinal <= state.applied_ordinal {
                continue;
            }
            let _ = apply_sealed(&mut state, sealed);
        }
        state
    }
}

/// How far behind its log one projection is, and how much of the log it
/// could not apply.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyncStatus {
    /// The ordinal the fold has consumed through.
    pub applied_ordinal: u64, // data_class: INTERNAL_ONLY
    /// The log's head ordinal, as reported by the log.
    pub head: u64, // data_class: INTERNAL_ONLY
    /// Entries the fold has not yet consumed.
    pub lag: u64, // data_class: INTERNAL_ONLY
    /// Entries consumed but refused, with typed reasons in the ledger.
    pub poisoned_count: u64, // data_class: INTERNAL_ONLY
    /// The earliest poisoned ordinal, if any — where an operator starts.
    pub first_poisoned_ordinal: Option<u64>, // data_class: INTERNAL_ONLY
}

impl ProjectionState {
    /// The projection's honest position against a log whose head is
    /// `head`.
    pub fn sync_status(&self, head: u64) -> SyncStatus {
        SyncStatus {
            applied_ordinal: self.applied_ordinal,
            head,
            lag: head.saturating_sub(self.applied_ordinal),
            poisoned_count: self.poison.len() as u64,
            first_poisoned_ordinal: self.poison.keys().next().copied(),
        }
    }
}
