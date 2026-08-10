//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 3).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, and `receipt`.
//! Slice 3 wires kernel + adapter stubs; CLI subcommands and six-axis end-to-end receipts land
//! in Slice 6.
#![forbid(unsafe_code)]

pub mod driver;

/// Fail-closed readiness gate. `true` once Slice 3 adapter/facade wiring is present.
pub const fn w0_ready() -> bool {
    driver::w0_ready()
}
