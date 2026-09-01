//! Ontology query execution usecase foundation.
//!
//! This crate is the source-level orchestration seam between future REST/gRPC
//! adapters and the ontology query domain. It accepts a precomputed policy
//! decision, enforces idempotent execution semantics, emits metadata-only audit
//! events, and never carries raw property values or provider/runtime credentials.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod execution;
mod types;

pub use execution::OntologyQueryExecutionUsecase;
pub use types::*;

#[cfg(test)]
mod tests;
