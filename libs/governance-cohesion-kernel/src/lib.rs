//! Cohesion-fitness lane kernel.
//!
//! Pure validator (no I/O) for the cohesion lane. Runners feed observed
//! cross-module fan-out + per-module ceilings and receive a ratcheted verdict.

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
    pub module: String,              // data_class: INTERNAL_ONLY
    pub fan_out_observed: u32,       // data_class: INTERNAL_ONLY
    pub fan_out_ceiling: u32,        // data_class: INTERNAL_ONLY
    pub cross_layer_violations: u32, // data_class: INTERNAL_ONLY
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
        if input.cross_layer_violations > 0 {
            let reason = format!(
                "{}: {} clean-arch layer violations",
                input.module, input.cross_layer_violations
            );
            return match self.phase {
                LanePhase::Warn => LaneVerdict::Warn(reason),
                LanePhase::Block => LaneVerdict::Block(reason),
            };
        }
        if input.fan_out_observed <= input.fan_out_ceiling {
            return LaneVerdict::Pass;
        }
        let reason = format!(
            "{}: fan-out {} exceeds ceiling {}",
            input.module, input.fan_out_observed, input.fan_out_ceiling
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

    fn input(observed: u32, ceiling: u32, violations: u32) -> LaneInput {
        LaneInput {
            module: "test-domain".into(),
            fan_out_observed: observed,
            fan_out_ceiling: ceiling,
            cross_layer_violations: violations,
        }
    }

    #[test]
    fn passes_when_under_ceiling_and_no_violations() {
        let r = LaneRatchet::new(LanePhase::Block);
        assert_eq!(r.check(&input(3, 5, 0)), LaneVerdict::Pass);
    }

    #[test]
    fn blocks_on_layer_violations_in_block_phase() {
        let r = LaneRatchet::new(LanePhase::Block);
        match r.check(&input(1, 5, 2)) {
            LaneVerdict::Block(reason) => assert!(reason.contains("violations")),
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn warns_when_over_fan_out_in_warn_phase() {
        let r = LaneRatchet::new(LanePhase::Warn);
        match r.check(&input(7, 5, 0)) {
            LaneVerdict::Warn(reason) => assert!(reason.contains("fan-out")),
            other => panic!("expected Warn, got {other:?}"),
        }
    }
}
