//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 14).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, `receipt`,
//! `emit-canary`, `materialize-canary`, and `canary-defect` (single-fixture only — never bulk
//! `k8s/`). Bridge feedback only — never merge authority.
#![forbid(unsafe_code)]

pub mod cli;
pub mod driver;
pub mod receipt_codec;
pub mod receipt_e2e;

/// Fail-closed readiness gate. `true` once Slice 14 canary materialize/defect seams are wired.
pub const fn w0_ready() -> bool {
    driver::w0_ready()
}
