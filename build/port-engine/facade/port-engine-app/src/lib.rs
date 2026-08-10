//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 6).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, and `receipt`.
//! Slice 6 lands the hand-rolled CLI and six-axis receipt end-to-end harness. Bridge feedback
//! only — never merge authority.
#![forbid(unsafe_code)]

pub mod cli;
pub mod driver;
pub mod receipt_e2e;

/// Fail-closed readiness gate. `true` once Slice 6 CLI + receipt e2e wiring is present.
pub const fn w0_ready() -> bool {
    driver::w0_ready()
}
