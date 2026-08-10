//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 5).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, and `receipt`.
//! Slice 5 wires rust-ir syn/quote emit; CLI subcommands and six-axis end-to-end receipts
//! land in Slice 6.
#![forbid(unsafe_code)]

pub mod driver;

/// Fail-closed readiness gate. `true` once Slice 5 adapter/facade wiring is present.
pub const fn w0_ready() -> bool {
    driver::w0_ready()
}
