//! # port-engine-frontend-go — Go SourceModel snapshot consumer.
//!
//! ADR-0638 D3 snapshot firewall: this adapter consumes **SourceModel snapshot bytes only** and
//! must never invoke a Go toolchain in-process or from the `verify()` path, nor read the Go corpus
//! from disk. Both halves of that fence are enforced by `tests/firewall.rs`.
//!
//! The crate is modular, so the fence is no longer "scan lib.rs". A `mod` declaration compiles a
//! file the old single-file scan never read; the fence now enumerates every production source and
//! PROVES the enumeration is the whole of `src/`.
#![forbid(unsafe_code)]

mod convert;
mod error;
mod model;
mod vocabulary;
mod wire;

pub use error::SnapshotError;
pub use model::GoSourceModel;
pub use vocabulary::{
    ATTR_GO_NODE, ATTR_LIT_KIND, ATTR_OP, ATTR_REF, ATTR_VALUE, KNOWN_ATTR_KEYS,
    KNOWN_DECLARATION_KINDS, KNOWN_FLAGS, KNOWN_MEMBER_KINDS, KNOWN_TYPE_KINDS,
    PRODUCER_BOOTSTRAP_GO, PRODUCER_OWNED_RUST, SCHEMA_VERSION_DECLARATIONS,
    SCHEMA_VERSION_FLAT_TYPES, SCHEMA_VERSION_IDENTITY_ONLY,
};

/// Fail-closed readiness gate. `true` once snapshot decode is present.
#[must_use]
pub const fn w0_ready() -> bool {
    true
}
