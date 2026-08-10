//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 12).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, and `receipt`.
//! Slice 12 hardens receipts (golden + byte-identical re-run) and wires `render`/`verify`/`delta`.
//! No bulk `k8s/` emission (W0-B hard stop). Bridge feedback only — never merge authority.
#![forbid(unsafe_code)]

pub mod cli;
pub mod driver;
pub mod receipt_codec;
pub mod receipt_e2e;

/// Fail-closed readiness gate. `true` once Slice 12 receipt hardening is present.
pub const fn w0_ready() -> bool {
    driver::w0_ready()
}
