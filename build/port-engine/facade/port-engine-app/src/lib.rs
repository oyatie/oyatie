//! # port-engine-app — driver facade for the owned deterministic port engine (W0-B Slice 1).
//!
//! ADR-0637 D1 facade face: entrypoints `plan`, `render`, `verify`, `delta`, and `receipt`.
//! Slice 6 lands end-to-end six-axis receipt wiring. This crate is an empty, fail-closed shell.
#![forbid(unsafe_code)]

/// Fail-closed readiness gate. Remains `false` until Slice 6 lands the driver pipeline.
pub const fn w0_ready() -> bool {
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn slice1_does_not_claim_readiness() {
        assert!(!super::w0_ready());
    }
}
