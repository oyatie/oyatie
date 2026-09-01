//! Foundry projection port: the durable, indexed read plane for the
//! ontology projection — the store the projector mirrors each fold
//! outcome into, and the surface every object read is served from.
//!
//! The contract in one breath: applies are a dense per-tenant mirror of
//! consumed log entries (byte-identical re-apply dedups, divergence is
//! loud, a poison spends its ordinal visibly); reads are tenant-isolated
//! and deterministic (`object_ref` order) with typed cursors whose pages
//! partition the full result; predicates are `Equals` (exact typed
//! equality) and inclusive kind-scoped `Range` over the scalar value
//! kinds — one kind per range, arrays and structs unrankable, and a
//! kind-mismatched stored value refuses loudly instead of matching
//! false ([`data_ontology_kernel::StorageClass`] stays the adapters'
//! index affinity, but a class groups several kinds, so the kind is
//! the comparison unit). The port
//! speaks the kernel's own types — this is Foundry's read plane for
//! Foundry's domain, not a foreign wire seam. A store failure is
//! infrastructure, never a poison: the projector halts, the log stays
//! the source of truth. Scope holds: link instances, registry and
//! checkpoint snapshots, and text search live elsewhere.
#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod conformance;
mod keys;
mod memory;
mod predicate;
mod store;

pub use keys::KeyDesignations;
pub use memory::MemoryProjectionStore;
pub use predicate::{PredicateError, PropertyPredicate};
pub use store::{
    AppliedEntry, ApplyReceipt, EntryOutcome, Page, PageRequest, ProjectedObject, ProjectionCursor,
    ProjectionStore, ProjectionStoreError,
};
