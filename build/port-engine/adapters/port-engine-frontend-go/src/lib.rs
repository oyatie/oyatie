//! # port-engine-frontend-go — Go SourceModel snapshot consumer (W0-B Slice 1 skeleton).
//!
//! ADR-0638 D3 snapshot firewall: this adapter consumes **SourceModel snapshot bytes only** and
//! must never invoke Go in-process or from the `verify()` path. Slice 4 lands decode and the
//! architecture test forbidding `Command::new("go")` in library code used by verify.
#![forbid(unsafe_code)]

/// Fail-closed readiness gate. Remains `false` until Slice 4 lands snapshot decode.
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
