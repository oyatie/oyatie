//! Catch-up: bring a durable projection to its log's head by refolding
//! from the store's own `applied_head`.
//!
//! [`crate::project_through`] has always DOCUMENTED this recovery — "a
//! caller recovers by refolding from the store's `applied_head`" — but
//! no caller existed. A store that halted on an outage, and a store
//! that was never populated at all, both had no way back, and
//! [`crate::store_sync_status`] could only report the lag it had no
//! means to close. That gap stopped being cosmetic when reads began
//! being served from the durable projection: a store younger than its
//! log answers incompletely, and answers with authority.
//!
//! **Catch-up resumes INCLUSIVE of the store's own head entry.**
//! Re-applying that entry is a deduplicated no-op when the store really
//! does hold this log, and a refusal when it does not. Without the
//! re-apply, catch-up would happily top up a store built from a
//! DIFFERENT log and leave it reporting `applied_head == log head`
//! while holding rows that are not `fold(log)` — a readiness signal
//! that lies.
//!
//! Revalidation alone proves only the resume POINT, and a head entry's
//! bytes can be identical under two logs that disagree earlier — one
//! envelope is one object, so a differing prefix need not reach it. So
//! catch-up also checks that **the poisons the store holds below its
//! head are the poisons this (log, registry) produces there**
//! ([`CatchUpError::DivergentPrefixPoisons`]). The prefix fold is
//! already computed to rebuild the resume state, so that costs one
//! store read and no extra folding.
//!
//! Neither check proves the whole prefix: a divergence that changes an
//! applied outcome without changing any poison still passes, and a
//! store whose file was swapped below its head can be exactly that. The
//! only full audit is a rebuild from empty. Closing it properly needs a
//! digest written at apply time — a port change, and its own lane.
//!
//! **Precondition: `registry` should be the snapshot the store was
//! built under.** The port deliberately holds no registry identity —
//! snapshots "live elsewhere" by its own scope clause — and proving a
//! resume runs against the SAME fold input is the [`crate::Checkpoint`]'s
//! job, which discards and refolds when the registry differs so that
//! "resume must never produce a state a fresh fold would not". This is
//! the durable twin of that operation and cannot make the comparison,
//! so it detects the evolution only where outcomes CHANGED: a
//! [`crate::PoisonReason::UnknownRevision`] that the evolution should
//! have un-poisoned now surfaces as `DivergentPrefixPoisons` rather
//! than silently persisting, and a changed head outcome surfaces as
//! [`CatchUpError::DivergentResumePoint`]. Both refusals have two
//! readings — a different log, or a different registry — and cannot
//! tell them apart. **After a registry evolution, rebuild from empty.**
//! That is not damage control: it IS the refold `UnknownRevision`
//! promises and the migration doctrine requires.

use data_ontology_kernel::OntologyEngine;
use foundry_projection_draft::{ProjectionStore, ProjectionStoreError};
use foundry_records_draft::SealedEnvelope;

use crate::emission::poison_label;
use crate::fold::fold_from_scratch;
use crate::state::ProjectionState;
use crate::writethrough::{WriteThroughError, project_through};

/// Where a catch-up began, and where it left the store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaughtUp {
    /// The store's head before the call — the ordinal it durably held.
    pub resumed_from: u64,
    /// The store's head afterwards, READ BACK from the store rather
    /// than computed, so this is the same number the store will report
    /// to anyone else who asks.
    pub head: u64,
    /// Whether the resume point was re-applied and agreed with the log.
    /// False only for a rebuild from empty, which has no resume point.
    pub revalidated: bool,
}

/// Why a catch-up refused. None of these is a poison: a poison derives
/// from (log bytes, registry snapshot) and repeats identically on every
/// replay, and not one of these does.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatchUpError {
    /// The store could not be read — its head, or its poison ledger.
    /// Never read as zero or as empty: those readings would rebuild the
    /// whole log over contents we cannot see, turning an infrastructure
    /// blip into a destructive operation.
    Read(ProjectionStoreError),
    /// The store claims more of the log than the log holds. Serving it
    /// would answer from rows the log can no longer justify, so it is a
    /// refusal rather than a no-op.
    StoreAheadOfLog { store_head: u64, log_head: u64 },
    /// The entry the store stopped at is not this log's entry at that
    /// ordinal: the store belongs to a different log.
    DivergentResumePoint { ordinal: u64 },
    /// The poisons the store holds below its head are not the poisons
    /// this (log, registry) produces there. Two readings, and the
    /// refusal cannot tell them apart: the store was built from a
    /// different log, or it was built under a different registry
    /// snapshot. Either way resuming would leave a mixture — part of
    /// the projection folded under one input, part under another, with
    /// `applied_head` equal to the log head and nothing reporting that
    /// `store == fold(log, registry)` is false.
    DivergentPrefixPoisons {
        store_holds: Vec<(u64, String)>,
        log_produces: Vec<(u64, String)>,
    },
    /// The mirror halted on a store refusal, which names its ordinal.
    Mirror(WriteThroughError),
}

/// Fold `log` into `store` from wherever the store already stands.
///
/// `log` is one tenant's entries in ordinal order. A rebuild from
/// scratch is not a separate code path — it is this function against an
/// empty store.
pub fn catch_up(
    tenant_id: &str,
    registry: &OntologyEngine,
    store: &mut dyn ProjectionStore,
    log: &[SealedEnvelope],
) -> Result<CaughtUp, CatchUpError> {
    let resumed_from = store.applied_head(tenant_id).map_err(CatchUpError::Read)?;
    let log_head = log.last().map_or(0, |sealed| sealed.receipt.ordinal);
    if resumed_from > log_head {
        return Err(CatchUpError::StoreAheadOfLog {
            store_head: resumed_from,
            log_head,
        });
    }
    // Resume AT the head entry, not after it. Ordinals are dense from 1,
    // so an empty store starts at the first entry and has nothing to
    // revalidate.
    let resume_at = resumed_from.max(1);
    let split = log
        .iter()
        .position(|sealed| sealed.receipt.ordinal >= resume_at)
        .unwrap_or(log.len());
    // Rebuild the in-memory fold to exactly what the store durably
    // holds, so every entry mirrored from here carries the same bytes it
    // carried the first time — which is what lets the store recognise a
    // re-apply as a duplicate rather than as divergence.
    let mut state = fold_from_scratch(tenant_id, registry, log[..split].iter());
    check_prefix_agrees(tenant_id, store, &state, resume_at)?;
    project_through(&mut state, store, &log[split..]).map_err(|error| match error {
        WriteThroughError::Store {
            ordinal,
            error: ProjectionStoreError::DivergentReplay { .. },
        } if ordinal == resumed_from => CatchUpError::DivergentResumePoint { ordinal },
        other => CatchUpError::Mirror(other),
    })?;
    let head = store.applied_head(tenant_id).map_err(CatchUpError::Read)?;
    Ok(CaughtUp {
        resumed_from,
        head,
        revalidated: resumed_from > 0,
    })
}

/// The poisons the store holds below its head must be the poisons this
/// (log, registry) produces there.
///
/// Revalidating the head entry cannot see a divergence beneath it: one
/// envelope is one object, so a head entry's mirrored bytes can be
/// IDENTICAL under two logs that disagree earlier — it dedups, and the
/// store finishes at the log's head holding a prefix from somewhere
/// else. The prefix fold is already computed to rebuild the resume
/// state, so this costs one additional store read and no additional
/// folding.
///
/// It does not close every case: a divergence that changes an applied
/// outcome without changing any poison still passes. That limit, and
/// its remedy, are the module's.
fn check_prefix_agrees(
    tenant_id: &str,
    store: &dyn ProjectionStore,
    state: &ProjectionState,
    resume_at: u64,
) -> Result<(), CatchUpError> {
    let log_produces: Vec<(u64, String)> = state
        .poison
        .iter()
        .map(|(ordinal, reason)| (*ordinal, poison_label(reason).to_owned()))
        .collect();
    let mut store_holds: Vec<(u64, String)> = store
        .poisoned(tenant_id)
        .map_err(CatchUpError::Read)?
        .into_iter()
        .filter(|(ordinal, _)| *ordinal < resume_at)
        .collect();
    store_holds.sort();
    if store_holds == log_produces {
        return Ok(());
    }
    Err(CatchUpError::DivergentPrefixPoisons {
        store_holds,
        log_produces,
    })
}
