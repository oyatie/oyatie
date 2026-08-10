//! # port-engine-rust-ir — Rust IR and deterministic renderer (W0-B Slice 1 skeleton).
//!
//! ADR-0637 D1 core face: holds `TargetIr` rendering with stable ordering and normalized
//! formatting. Slice 5 lands the syn/quote path and forbids clock/RNG/env/path leakage in render
//! inputs. This crate is an empty, fail-closed shell at Slice 1.
#![forbid(unsafe_code)]

/// Fail-closed readiness gate. Remains `false` until Slice 5 lands the renderer skeleton.
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
