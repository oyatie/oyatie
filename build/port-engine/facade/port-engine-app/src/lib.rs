//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 8).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, and `receipt`.
//! Slice 8 wires bootstrap SourceModel snapshot admission (OOB fixture; never spawns Go).
//! Bridge feedback only — never merge authority.
#![forbid(unsafe_code)]

pub mod cli;
pub mod driver;
pub mod receipt_e2e;

/// Fail-closed readiness gate. `true` once Slice 8 snapshot admission wiring is present.
pub const fn w0_ready() -> bool {
    driver::w0_ready()
}
