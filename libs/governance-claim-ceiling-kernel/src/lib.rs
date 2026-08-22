//! Claim-ceiling fitness lane kernel.
//!
//! Pure validator (no I/O) for the claim-ceiling ratchet (M02-P03-IP-002).
//! Runners feed observed claim depth + per-agent ceiling configuration and
//! receive a [`LaneVerdict`] suitable for WARN→BLOCK ratchet promotion.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// WARN→BLOCK ratchet phase for any fitness lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LanePhase {
    /// Surface but do not block.
    Warn,
    /// Hard-fail the merge.
    Block,
}

/// Observed input snapshot used by [`LaneRatchet::check`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaneInput {
    /// Observed depth of claim chain for the agent under test.
    pub observed_claim_depth: u32, // data_class: INTERNAL_ONLY
    /// Configured ceiling per ADR-0054 + autonomy-ceiling.md.
    pub configured_ceiling: u32, // data_class: INTERNAL_ONLY
    /// Optional rationale for documented exception.
    pub rationale: Option<String>, // data_class: INTERNAL_ONLY
}

/// Verdict for a lane check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LaneVerdict {
    Pass,
    Warn(String),
    Block(String),
}

/// Lane ratchet for claim-ceiling fitness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaneRatchet {
    pub phase: LanePhase, // data_class: INTERNAL_ONLY
}

impl LaneRatchet {
    pub fn new(phase: LanePhase) -> Self {
        Self { phase }
    }

    /// Check a single lane sample.
    pub fn check(&self, input: &LaneInput) -> LaneVerdict {
        if input.observed_claim_depth <= input.configured_ceiling {
            return LaneVerdict::Pass;
        }
        let reason = format!(
            "claim depth {} exceeds ceiling {}",
            input.observed_claim_depth, input.configured_ceiling
        );
        match self.phase {
            LanePhase::Warn => LaneVerdict::Warn(reason),
            LanePhase::Block => LaneVerdict::Block(reason),
        }
    }
}

/// Convenience entry point alias matching IP-002 symbol contract.
pub fn ratchet(phase: LanePhase, input: &LaneInput) -> LaneVerdict {
    LaneRatchet::new(phase).check(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(depth: u32, ceiling: u32) -> LaneInput {
        LaneInput {
            observed_claim_depth: depth,
            configured_ceiling: ceiling,
            rationale: None,
        }
    }

    #[test]
    fn passes_when_under_ceiling() {
        let ratchet = LaneRatchet::new(LanePhase::Block);
        assert_eq!(ratchet.check(&input(2, 3)), LaneVerdict::Pass);
    }

    #[test]
    fn warns_in_warn_phase_when_over_ceiling() {
        let ratchet = LaneRatchet::new(LanePhase::Warn);
        match ratchet.check(&input(5, 3)) {
            LaneVerdict::Warn(reason) => assert!(reason.contains("exceeds")),
            other => panic!("expected Warn, got {other:?}"),
        }
    }

    #[test]
    fn blocks_in_block_phase_when_over_ceiling() {
        let ratchet = LaneRatchet::new(LanePhase::Block);
        match ratchet.check(&input(5, 3)) {
            LaneVerdict::Block(reason) => assert!(reason.contains("exceeds")),
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn ratchet_alias_matches_struct_path() {
        let i = input(5, 3);
        assert_eq!(
            ratchet(LanePhase::Block, &i),
            LaneRatchet::new(LanePhase::Block).check(&i)
        );
    }
}
