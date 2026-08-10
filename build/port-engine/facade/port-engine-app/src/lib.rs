//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 13).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, `receipt`, and
//! `emit-canary` (single-fixture canary only — never bulk `k8s/`).
//! Bridge feedback only — never merge authority.
#![forbid(unsafe_code)]

pub mod cli;
pub mod driver;
pub mod receipt_codec;
pub mod receipt_e2e;

/// Fail-closed readiness gate. `true` once Slice 13 canary emit is wired.
pub const fn w0_ready() -> bool {
    driver::w0_ready()
}
