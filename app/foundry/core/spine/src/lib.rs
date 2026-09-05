//! Foundry spine: the behavior plane of the M3 write path.
//!
//! Born as the refusal-taxonomy stub: [`Refused`] is the writer's one
//! error shape, and [`RefusalGate`] names the deny-by-default gates every
//! submission passes in order — authorization, parameter conformance,
//! edit admission. The writer, projector, checkpoints, history, and audit
//! trail land as their own lanes; each declares its dependency edge
//! (records port, ontology kernel, the edits wire plane) in the lane that
//! first uses it.
#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod audit;
mod boundary;
mod catchup;
mod checkpoint;
mod emission;
mod error;
mod fold;
mod history;
mod migrate;
mod revision;
mod state;
mod writer;
mod writethrough;

pub use boundary::BoundaryError;
pub use catchup::{CatchUpError, CaughtUp, catch_up};
pub use checkpoint::{Checkpoint, SyncStatus, store_sync_status};
pub use emission::{
    DENIED_AUDIT_EVENT_TYPE, DerivedEvents, POISONED_AUDIT_EVENT_TYPE, Underivable,
    UnderivableReason, derive_action_events, derive_denial_events, poison_label,
};
pub use error::{RefusalGate, Refused};
pub use fold::{FoldOutcome, PoisonReason, apply_sealed, fold_from_scratch};
pub use history::{AuditDisposition, AuditEntry, HistoryEntry, audit_view, object_history};
pub use migrate::{
    DefaultValue, MigrationAttestation, MigrationAuthority, MigrationPlan, MigrationStatus,
    PendingUpcast, PlanError, UpcastTransform, ValueConversion, migration_attestation,
    pending_objects, run_to_fixpoint, upcast_idempotency_key,
};
pub use revision::{PinnedObject, UpcastState, ViewError, object_at_revision};
pub use state::{ObjectBinding, ProjectionState};
pub use writer::{ActionSubmission, ApplyOutcome, WriteError, submit};
pub use writethrough::{WriteThroughError, project_through};
