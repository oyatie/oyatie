//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 4).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, and `receipt`.
//! Slice 4 wires frontend-go snapshot decode; CLI subcommands and six-axis end-to-end receipts
//! land in Slice 6.
#![forbid(unsafe_code)]

pub mod driver;

/// Fail-closed readiness gate. `true` once Slice 4 adapter/facade wiring is present.
pub const fn w0_ready() -> bool {
    driver::w0_ready()
}
