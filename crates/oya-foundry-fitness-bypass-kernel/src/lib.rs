//! Bypass fitness lane kernel.
//!
//! Pure validator (no I/O) for the foundation-bypass lane. Runners feed
//! observed bypass entries; this kernel emits a ratcheted verdict comparing
//! observed-vs-allowed bypass counts.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LanePhase {
    Warn,
    Block,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaneInput {
    pub observed_bypass_count: u32, // data_class: INTERNAL_ONLY
    pub allowed_bypass_count: u32,  // data_class: INTERNAL_ONLY
    pub expired_bypass_count: u32,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LaneVerdict {
    Pass,
    Warn(String),
    Block(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaneRatchet {
    pub phase: LanePhase, // data_class: INTERNAL_ONLY
}

impl LaneRatchet {
    pub fn new(phase: LanePhase) -> Self {
        Self { phase }
    }

    pub fn check(&self, input: &LaneInput) -> LaneVerdict {
        if input.expired_bypass_count > 0 {
            let reason = format!("{} expired bypass entries", input.expired_bypass_count);
            return match self.phase {
                LanePhase::Warn => LaneVerdict::Warn(reason),
                LanePhase::Block => LaneVerdict::Block(reason),
            };
        }
        if input.observed_bypass_count <= input.allowed_bypass_count {
            return LaneVerdict::Pass;
        }
        let reason = format!(
            "bypass count {} exceeds allowed {}",
            input.observed_bypass_count, input.allowed_bypass_count
        );
        match self.phase {
            LanePhase::Warn => LaneVerdict::Warn(reason),
            LanePhase::Block => LaneVerdict::Block(reason),
        }
    }
}

pub fn ratchet(phase: LanePhase, input: &LaneInput) -> LaneVerdict {
    LaneRatchet::new(phase).check(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(observed: u32, allowed: u32, expired: u32) -> LaneInput {
        LaneInput {
            observed_bypass_count: observed,
            allowed_bypass_count: allowed,
            expired_bypass_count: expired,
        }
    }

    #[test]
    fn passes_when_under_allowed_with_no_expiry() {
        let r = LaneRatchet::new(LanePhase::Block);
        assert_eq!(r.check(&input(1, 3, 0)), LaneVerdict::Pass);
    }

    #[test]
    fn blocks_on_expired_entries_in_block_phase() {
        let r = LaneRatchet::new(LanePhase::Block);
        match r.check(&input(0, 3, 2)) {
            LaneVerdict::Block(reason) => assert!(reason.contains("expired")),
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn warns_when_over_allowed_in_warn_phase() {
        let r = LaneRatchet::new(LanePhase::Warn);
        match r.check(&input(4, 2, 0)) {
            LaneVerdict::Warn(reason) => assert!(reason.contains("exceeds")),
            other => panic!("expected Warn, got {other:?}"),
        }
    }
}
