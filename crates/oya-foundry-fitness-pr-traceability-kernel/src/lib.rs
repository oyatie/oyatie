//! PR-traceability fitness lane kernel.
//!
//! Pure validator (no I/O) that ensures every PR carries phase+plan+ADR
//! citations matching the ralplan-freelance-prevention-controls regex contract.

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
    pub pr_title: String,           // data_class: INTERNAL_ONLY
    pub pr_body: String,            // data_class: INTERNAL_ONLY
    pub cites_phase_id: bool,       // data_class: INTERNAL_ONLY
    pub cites_plan_or_adr: bool,    // data_class: INTERNAL_ONLY
    pub has_decision_log_row: bool, // data_class: INTERNAL_ONLY
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
        let mut missing: Vec<&'static str> = Vec::new();
        if !input.cites_phase_id {
            missing.push("phase-id");
        }
        if !input.cites_plan_or_adr {
            missing.push("plan-or-adr");
        }
        if !input.has_decision_log_row {
            missing.push("decision-log-row");
        }
        if missing.is_empty() {
            return LaneVerdict::Pass;
        }
        let reason = format!("missing pr-traceability fields: {}", missing.join(","));
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

    fn complete_input() -> LaneInput {
        LaneInput {
            pr_title: "M02-P03 scaffold".into(),
            pr_body: "Cites ADR-0054".into(),
            cites_phase_id: true,
            cites_plan_or_adr: true,
            has_decision_log_row: true,
        }
    }

    #[test]
    fn passes_when_all_fields_present() {
        let r = LaneRatchet::new(LanePhase::Block);
        assert_eq!(r.check(&complete_input()), LaneVerdict::Pass);
    }

    #[test]
    fn blocks_when_plan_or_adr_missing() {
        let r = LaneRatchet::new(LanePhase::Block);
        let mut i = complete_input();
        i.cites_plan_or_adr = false;
        match r.check(&i) {
            LaneVerdict::Block(reason) => assert!(reason.contains("plan-or-adr")),
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn warns_when_decision_log_missing() {
        let r = LaneRatchet::new(LanePhase::Warn);
        let mut i = complete_input();
        i.has_decision_log_row = false;
        match r.check(&i) {
            LaneVerdict::Warn(reason) => assert!(reason.contains("decision-log")),
            other => panic!("expected Warn, got {other:?}"),
        }
    }
}
