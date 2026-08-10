//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 7).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, and `receipt`.
//! Slice 7 wires hashing + embedded neutral rulepack v0. Bridge feedback only — never merge
//! authority. Forever `specs/port-rules/**` remains integ/specs; this tip embeds a hermetic mirror.
#![forbid(unsafe_code)]

pub mod cli;
pub mod driver;
pub mod receipt_e2e;

/// Fail-closed readiness gate. `true` once Slice 7 hash + rulepack wiring is present.
pub const fn w0_ready() -> bool {
    driver::w0_ready()
}
