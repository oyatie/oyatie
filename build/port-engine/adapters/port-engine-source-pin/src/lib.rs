//! # port-engine-source-pin — upstream pin and license verification (W0-B Slice 1 skeleton).
//!
//! ADR-0638 D3: binds `specs/k8s-port/upstream-pin.json`, verifies Apache-2.0 licensing, and
//! records canonical pin / snapshot digest binding. The bootstrap Go extractor runs **out of band**
//! only — never from `verify()`. Slice 3 lands the pin loader and extractor admission.
#![forbid(unsafe_code)]

/// Fail-closed readiness gate. Remains `false` until Slice 3 lands pin acquisition.
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
