//! Pre-push fitness lane kernel.
//!
//! Pure validator (no I/O) for the pre-push gate. Runners feed which gates
//! ran in the local pre-push hook; this kernel emits a ratcheted verdict.

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
    pub ran_check_sh: bool,         // data_class: INTERNAL_ONLY
    pub ran_cargo_test: bool,       // data_class: INTERNAL_ONLY
    pub ran_governance_lanes: bool, // data_class: INTERNAL_ONLY
    pub had_bypass_flag: bool,      // data_class: INTERNAL_ONLY
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
        if input.had_bypass_flag {
            let reason = "pre-push hook bypassed (--no-verify)".to_string();
            return match self.phase {
                LanePhase::Warn => LaneVerdict::Warn(reason),
                LanePhase::Block => LaneVerdict::Block(reason),
            };
        }
        let mut missing: Vec<&'static str> = Vec::new();
        if !input.ran_check_sh {
            missing.push("check.sh");
        }
        if !input.ran_cargo_test {
            missing.push("cargo-test");
        }
        if !input.ran_governance_lanes {
            missing.push("governance-lanes");
        }
        if missing.is_empty() {
            return LaneVerdict::Pass;
        }
        let reason = format!("pre-push gates not run: {}", missing.join(","));
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

    fn input(check: bool, test: bool, lanes: bool, bypass: bool) -> LaneInput {
        LaneInput {
            ran_check_sh: check,
            ran_cargo_test: test,
            ran_governance_lanes: lanes,
            had_bypass_flag: bypass,
        }
    }

    #[test]
    fn passes_when_all_gates_ran() {
        let r = LaneRatchet::new(LanePhase::Block);
        assert_eq!(r.check(&input(true, true, true, false)), LaneVerdict::Pass);
    }

    #[test]
    fn blocks_on_bypass_flag() {
        let r = LaneRatchet::new(LanePhase::Block);
        match r.check(&input(true, true, true, true)) {
            LaneVerdict::Block(reason) => assert!(reason.contains("bypassed")),
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn warns_in_warn_phase_when_gate_missing() {
        let r = LaneRatchet::new(LanePhase::Warn);
        match r.check(&input(false, true, true, false)) {
            LaneVerdict::Warn(reason) => assert!(reason.contains("check.sh")),
            other => panic!("expected Warn, got {other:?}"),
        }
    }
}
