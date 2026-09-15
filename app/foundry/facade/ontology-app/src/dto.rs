//! The wire shapes of the write surface.
//!
//! `occurred_at_epoch_seconds` is the caller's, not this process's, and
//! that is deliberate: the writer derives every payload byte from the
//! request so a retry is byte-identical and deduplicates. If the facade
//! stamped its own clock, the same request sent twice would produce
//! different bytes under one idempotency key — a loud conflict where the
//! caller expected a dedup. The tenant is absent from this shape by
//! design; it comes from the credential and nothing in the body can move
//! it.

pub use foundry_submission_draft::{SubmitRequest, SubmitResponse};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct RefusalBody {
    pub gate: String,  // data_class: INTERNAL_ONLY
    pub cause: String, // data_class: INTERNAL_ONLY
}
