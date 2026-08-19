//! # port-engine-emit — the emit seam: canary selection and allowlisted materialize.
//!
//! Hard stops (ADR-0704 / W0-B):
//! - NEVER writes under the corpus destination root
//! - the canary path emits exactly one file
//! - a materialize root's basename must be allowlisted
#![forbid(unsafe_code)]

/// This crate's own sources, for the engine-identity axis assembled by the facade.
mod sources;
pub use sources::CRATE_SOURCES;

mod canary;
mod error;
mod materialize;

pub use canary::{
    CanaryArtifact, assert_matches_golden, emit_canary_checked, golden_canary_bytes,
    golden_canary_digest, select_canary,
};
pub use error::EmitError;
pub use materialize::{
    materialize_canary, materialize_canary_roundtrip, materialize_tree, validate_canary_out_dir,
    validate_emit_out_dir,
};

/// Required basename for a canary materialize root (envelope fence).
pub const CANARY_OUT_DIRNAME: &str = "port-engine-canary-out";

/// Required basename for a full emit-tree materialize root.
///
/// A SECOND allowlisted basename, not a relaxation of the first. Keeping them separate means
/// "write one canary" and "write a corpus" can never be confused by a caller passing the wrong
/// directory.
pub const EMIT_OUT_DIRNAME: &str = "port-engine-emit-out";

/// Every basename a materialize root may have.
pub const ALLOWED_OUT_DIRNAMES: &[&str] = &[CANARY_OUT_DIRNAME, EMIT_OUT_DIRNAME];

/// Filename written inside the canary-out directory.
pub const CANARY_FILENAME: &str = "canary.rs";

/// Region id suffix produced by the `empty_canary` construction.
pub const CANARY_RULE_SUFFIX: &str = "__canary_empty_unit";

/// Embedded golden canary source (post syn/quote spelling of the mini fixture).
pub(crate) const GOLDEN_CANARY_RS: &str = include_str!("golden-canary-v0.txt");

/// Fail-closed readiness gate. `true` once the canary emit seam is present.
#[must_use]
pub const fn w0_ready() -> bool {
    true
}
