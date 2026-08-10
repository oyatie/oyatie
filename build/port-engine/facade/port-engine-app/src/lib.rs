//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 11).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, and `receipt`.
//! Slice 11 wires plan→RustIr construction apply into the pipeline. Bridge feedback only —
//! never merge authority.
#![forbid(unsafe_code)]

pub mod cli;
pub mod driver;
pub mod receipt_e2e;

/// Fail-closed readiness gate. `true` once Slice 11 transform wiring is present.
pub const fn w0_ready() -> bool {
    driver::w0_ready()
}
