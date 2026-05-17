//! Lean architecture check vocabulary (lean-a1..lean-a4).
//!
//! Provides the identifier enum and violation value-object consumed by
//! `oya-shared-architecture-check-cli` (P01-foundry-engine-consolidation).
//! This module is I/O-free; CLI tools own parsing and workspace traversal.
//!
//! Lean lane mapping (ADR-0056 §CI matrix / ADR-0057):
//! - `lean-a1` → [`LeanCheckId::DependencyDirection`]
//! - `lean-a2` → [`LeanCheckId::CrossProductRefusal`]
//! - `lean-a3` → [`LeanCheckId::PortLocation`]
//! - `lean-a4` → [`LeanCheckId::LayerCorrectness`]

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// Identifies one of the four lean architecture-check CI lanes.
///
/// Variants are ordered by their `lean-a*` lane number so that
/// `Ord`-sorted violation lists are stable across runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LeanCheckId {
    /// lean-a1 — enforce inward-only dependency flow per 13-layer matrix.
    DependencyDirection,
    /// lean-a2 — refuse direct cross-µservice imports (except public_layers).
    CrossProductRefusal,
    /// lean-a3 — assert port traits live only in `kernel`-layer crates.
    PortLocation,
    /// lean-a4 — assert declared layer matches crate-name suffix.
    LayerCorrectness,
}

impl LeanCheckId {
    /// Short kebab-case lane tag used in CI output and ADR references.
    pub fn lane_tag(self) -> &'static str {
        match self {
            Self::DependencyDirection => "lean-a1",
            Self::CrossProductRefusal => "lean-a2",
            Self::PortLocation => "lean-a3",
            Self::LayerCorrectness => "lean-a4",
        }
    }

    /// Human-readable description aligned with ADR-0057 lane definitions.
    pub fn description(self) -> &'static str {
        match self {
            Self::DependencyDirection => {
                "inward-only dependency flow per 13-layer matrix (ADR-0056)"
            }
            Self::CrossProductRefusal => {
                "no direct cross-µservice imports except via public_layers (ADR-0056)"
            }
            Self::PortLocation => "port traits must live in kernel-layer crates (ADR-0057)",
            Self::LayerCorrectness => {
                "declared layer must match crate-name suffix (ADR-0056 BNF v4)"
            }
        }
    }

    /// Returns all variants in `lean-a*` order.
    pub fn all() -> [Self; 4] {
        [
            Self::DependencyDirection,
            Self::CrossProductRefusal,
            Self::PortLocation,
            Self::LayerCorrectness,
        ]
    }
}

impl fmt::Display for LeanCheckId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.lane_tag())
    }
}

/// A single architecture violation emitted by a lean-a* check tool.
///
/// `crate_name` and `message` carry `INTERNAL_ONLY` content — repo
/// structure metadata; no user data or secrets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeanViolation {
    /// The lean-a* lane that detected this violation.  // data_class: INTERNAL_ONLY
    pub check: LeanCheckId,
    /// The crate or file where the violation was found. // data_class: INTERNAL_ONLY
    pub location: String,
    /// Human-readable explanation of the specific breach. // data_class: INTERNAL_ONLY
    pub message: String,
}

impl LeanViolation {
    /// Construct a new violation record.
    pub fn new(
        check: LeanCheckId,
        location: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            check,
            location: location.into(),
            message: message.into(),
        }
    }

    /// Single-line CI-log format: `[lean-a1] crate::path — message`.
    pub fn to_log_line(&self) -> String {
        format!(
            "[{}] {} — {}",
            self.check.lane_tag(),
            self.location,
            self.message
        )
    }
}

impl fmt::Display for LeanViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_log_line())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── LeanCheckId ──────────────────────────────────────────────────────────

    #[test]
    fn all_returns_four_variants_in_lane_order() {
        let all = LeanCheckId::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], LeanCheckId::DependencyDirection);
        assert_eq!(all[1], LeanCheckId::CrossProductRefusal);
        assert_eq!(all[2], LeanCheckId::PortLocation);
        assert_eq!(all[3], LeanCheckId::LayerCorrectness);
    }

    #[test]
    fn lane_tags_match_lean_a_numbering() {
        assert_eq!(LeanCheckId::DependencyDirection.lane_tag(), "lean-a1");
        assert_eq!(LeanCheckId::CrossProductRefusal.lane_tag(), "lean-a2");
        assert_eq!(LeanCheckId::PortLocation.lane_tag(), "lean-a3");
        assert_eq!(LeanCheckId::LayerCorrectness.lane_tag(), "lean-a4");
    }

    #[test]
    fn display_matches_lane_tag() {
        for id in LeanCheckId::all() {
            assert_eq!(format!("{id}"), id.lane_tag());
        }
    }

    #[test]
    fn ord_is_stable_lean_a1_before_lean_a4() {
        assert!(LeanCheckId::DependencyDirection < LeanCheckId::LayerCorrectness);
        assert!(LeanCheckId::CrossProductRefusal < LeanCheckId::PortLocation);
    }

    #[test]
    fn descriptions_are_non_empty_and_reference_adrs() {
        for id in LeanCheckId::all() {
            let desc = id.description();
            assert!(!desc.is_empty());
            assert!(
                desc.contains("ADR"),
                "expected ADR ref in description for {id}"
            );
        }
    }

    // ── LeanViolation ────────────────────────────────────────────────────────

    #[test]
    fn new_round_trips_fields() {
        let v = LeanViolation::new(
            LeanCheckId::DependencyDirection,
            "oya-foo-rest",
            "imports oya-bar-kernel (outward)",
        );
        assert_eq!(v.check, LeanCheckId::DependencyDirection);
        assert_eq!(v.location, "oya-foo-rest");
        assert_eq!(v.message, "imports oya-bar-kernel (outward)");
    }

    #[test]
    fn to_log_line_format_is_correct() {
        let v = LeanViolation::new(
            LeanCheckId::CrossProductRefusal,
            "oya-billing-application",
            "direct import of oya-hr-domain",
        );
        assert_eq!(
            v.to_log_line(),
            "[lean-a2] oya-billing-application — direct import of oya-hr-domain"
        );
    }

    #[test]
    fn display_matches_to_log_line() {
        let v = LeanViolation::new(
            LeanCheckId::PortLocation,
            "oya-foo-domain",
            "trait FooPort declared outside kernel",
        );
        assert_eq!(format!("{v}"), v.to_log_line());
    }

    #[test]
    fn violations_are_sortable_by_check_then_location() {
        let mut violations = vec![
            LeanViolation::new(LeanCheckId::LayerCorrectness, "z-crate", "layer mismatch"),
            LeanViolation::new(LeanCheckId::DependencyDirection, "a-crate", "outward dep"),
            LeanViolation::new(LeanCheckId::DependencyDirection, "b-crate", "outward dep"),
        ];
        violations.sort_by(|a, b| a.check.cmp(&b.check).then(a.location.cmp(&b.location)));
        assert_eq!(violations[0].location, "a-crate");
        assert_eq!(violations[1].location, "b-crate");
        assert_eq!(violations[2].check, LeanCheckId::LayerCorrectness);
    }

    #[test]
    fn empty_violation_list_indicates_clean_workspace() {
        let violations: Vec<LeanViolation> = Vec::new();
        assert!(
            violations.is_empty(),
            "clean workspace must have zero violations"
        );
    }
}
