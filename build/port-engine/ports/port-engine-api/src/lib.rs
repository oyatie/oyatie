//! # port-engine-api — neutral seam types for the owned deterministic port engine (W0-B Slice 1).
//!
//! ADR-0637 D1 assigns the ports face: `SourceModel`, `RulePack`, `TransformPlan`, `TargetIr`,
//! `Renderer`, and six-axis `Receipt`. Those types still live on `port-engine-kernel` until Slice 2
//! moves them here. This crate is an empty, fail-closed shell so the six-crate inventory and
//! workspace membership exist before seam extraction.
#![forbid(unsafe_code)]

/// Fail-closed readiness gate. Remains `false` until Slice 2 lands the neutral seam types.
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
