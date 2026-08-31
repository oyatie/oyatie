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

mod boundary;
mod checkpoint;
mod error;
mod fold;
mod state;

pub use boundary::BoundaryError;
pub use checkpoint::{Checkpoint, SyncStatus};
pub use error::{RefusalGate, Refused};
pub use fold::{FoldOutcome, PoisonReason, apply_sealed, fold_from_scratch};
pub use state::{ObjectBinding, ProjectionState};
