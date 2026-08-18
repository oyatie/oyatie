//! # oya-ci-materializer-kernel
//!
//! Pure planner + predicate kernel for the universal generated-artifact lifecycle engine
//! (ADR-0597 — E1). This crate is the analysis phase of the Bazel/Buck genrule
//! "analysis → action" pattern, reimplemented Rust-native.
//!
//! ## Purity guarantee (MF-2 / ADR-0597)
//!
//! This crate MUST contain ZERO uses of:
//!   - `std::process`   (no subprocess spawn)
//!   - `std::time`      (no clock or SystemTime)
//!   - `std::net`       (no network)
//!   - `std::fs`        (no filesystem I/O)
//!   - `std::env`       (no environment reads)
//!   - `rand`           (no randomness)
//!
//! All impurity is confined to `oya-ci-materializer-app` (the thin executor, ADR-0523
//! irreducible-glue items: buck2 bootstrap + scm-facts git boundary).
//!
//! The banned-symbol contract is enforced by a source-grep property test in
//! `tests/conformance.rs` (CP-MF2) and by the absence of those imports in this source.
//!
//! ## MF-3 / no-leak guarantee
//!
//! This crate contains ZERO hardcoded oyatie paths, targets, or names. Specifically:
//!   - No `//cloud/` literals
//!   - No `oya-cloud-ci-` literals
//!   - No `cloud/cloud-ci` literals
//!
//! Everything oyatie-specific lives in the policy data (the `ControlPlane` manifest).
//! This is enforced by a source-grep property test in `tests/conformance.rs` (CP-6/MF-3).
//!
//! ## MF-1 / merge-base anchoring note
//!
//! The de-commit exemption set (which artifact paths are `not-tracked-in-git`) is
//! evaluated from the manifest provided to `plan()` and `evaluate()`. In E1 the caller
//! supplies the candidate manifest. In E3 (the keystone repoint) the caller MUST supply
//! the merge-base manifest for the exemption-set computation, exactly as ADR-0551 anchors
//! the ratchet baseline. The v2 contract is shaped to accept this: `evaluate()` takes
//! `manifest: &ControlPlane` as a separate parameter (not baked in) so E3 can pass the
//! merge-base-materialised manifest without a schema break.
//!
//! ADR-0597: Rust-native materializer kernel (E1 — pure planner).
//! Depends on ADR-0596 (the de-commit firewall guard this kernel materializes around).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod evaluate;
pub mod model;
pub mod plan;

pub use evaluate::{Finding, FindingCode, Findings, evaluate};
pub use model::{
    ArtifactClass, ArtifactId, Bytes, ControlPlane, GeneratedArtifact, Generator,
    MaterializationMode, OutputMode, Runner, RunnerRegistryEntry,
};
pub use plan::{
    MaterializePlan, MaterializeScope, MaterializeStep, OutputSink, PlanError, materialize_closure,
    plan,
};
