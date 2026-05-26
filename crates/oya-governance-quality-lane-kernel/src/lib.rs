//! Quality-lane fitness kernel.
//!
//! Pure validator (no I/O) that asserts the standard quality lanes ran and
//! returned green. Per CI lanes contract: clippy, fmt, doc, deny.
//!
//! Also exports lean architecture check vocabulary (`lean-a1..lean-a4`) from
//! [`lean_check`] — the typed contracts consumed by
//! `oya-shared-architecture-check-cli` (M02-substrate/P01-foundry-engine-consolidation).

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod lean_check;
pub use lean_check::{LeanCheckId, LeanViolation};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LanePhase {
    Warn,
    Block,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaneInput {
    pub clippy_green: bool, // data_class: INTERNAL_ONLY
    pub fmt_green: bool,    // data_class: INTERNAL_ONLY
    pub doc_green: bool,    // data_class: INTERNAL_ONLY
    pub deny_green: bool,   // data_class: INTERNAL_ONLY
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
        let mut red: Vec<&'static str> = Vec::new();
        if !input.clippy_green {
            red.push("clippy");
        }
        if !input.fmt_green {
            red.push("fmt");
        }
        if !input.doc_green {
            red.push("doc");
        }
        if !input.deny_green {
            red.push("deny");
        }
        if red.is_empty() {
            return LaneVerdict::Pass;
        }
        let reason = format!("quality lanes red: {}", red.join(","));
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

    fn input(c: bool, f: bool, d: bool, dn: bool) -> LaneInput {
        LaneInput {
            clippy_green: c,
            fmt_green: f,
            doc_green: d,
            deny_green: dn,
        }
    }

    #[test]
    fn passes_when_all_lanes_green() {
        let r = LaneRatchet::new(LanePhase::Block);
        assert_eq!(r.check(&input(true, true, true, true)), LaneVerdict::Pass);
    }

    #[test]
    fn blocks_when_clippy_red() {
        let r = LaneRatchet::new(LanePhase::Block);
        match r.check(&input(false, true, true, true)) {
            LaneVerdict::Block(reason) => assert!(reason.contains("clippy")),
            other => panic!("expected Block, got {other:?}"),
        }
    }

    #[test]
    fn warns_in_warn_phase_when_multiple_red() {
        let r = LaneRatchet::new(LanePhase::Warn);
        match r.check(&input(false, false, true, true)) {
            LaneVerdict::Warn(reason) => {
                assert!(reason.contains("clippy"));
                assert!(reason.contains("fmt"));
            }
            other => panic!("expected Warn, got {other:?}"),
        }
    }
}
