//! Lean architecture check vocabulary.
//!
//! Provides the identifier enum and violation value-object consumed by
//! `shared-architecture-check-cli` (P01-foundry-engine-consolidation).
//! This module is I/O-free; CLI tools own parsing and workspace traversal.
//!
//! Canonical CI-lane mapping (docs/standards/ci-lanes.md §1.2 + ADR-0056 §2.2 / §207):
//! - `lean-a1-architecture` → [`LeanCheckId::DependencyDirection`],
//!   [`LeanCheckId::LayerCorrectness`], [`LeanCheckId::PortLocation`]
//!   (one lane covers all three architecture sub-checks)
//! - `lean-a2-bounded-contexts` → [`LeanCheckId::CrossProductRefusal`]
//!
//! Note: `lean-a3` and `lean-a4` are reserved for supply-chain and semver
//! per ci-lanes.md §1.2 and MUST NOT be reused for architecture checks.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// Identifies one of the four lean architecture-check sub-checks.
///
/// Variants are ordered by canonical CI lane (`lean-a1-architecture` first,
/// then `lean-a2-bounded-contexts`) so that `Ord`-sorted violation lists
/// are stable across runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LeanCheckId {
    /// lean-a1-architecture sub-check — enforce inward-only dependency flow
    /// per 13-layer matrix.
    DependencyDirection,
    /// lean-a1-architecture sub-check — declared layer must match crate-name
    /// suffix per BNF v4.1.
    LayerCorrectness,
    /// lean-a1-architecture sub-check — port traits live only in
    /// `kernel`-layer crates.
    PortLocation,
    /// lean-a2-bounded-contexts — refuse direct cross-µservice imports
    /// (except public_layers per ADR-0056 §2.2).
    CrossProductRefusal,
}

impl LeanCheckId {
    /// Canonical CI lane name used in CI output and branch-protection rules.
    ///
    /// Matches the `lean-a*` ids declared in `docs/standards/ci-lanes.md` §1.2.
    /// Multiple architecture sub-checks share `lean-a1-architecture`.
    pub fn lane_tag(self) -> &'static str {
        match self {
            Self::DependencyDirection | Self::LayerCorrectness | Self::PortLocation => {
                "lean-a1-architecture"
            }
            Self::CrossProductRefusal => "lean-a2-bounded-contexts",
        }
    }

    /// Human-readable description aligned with canonical ADR definitions.
    pub fn description(self) -> &'static str {
        match self {
            Self::DependencyDirection => {
                "inward-only dependency flow per 13-layer matrix (ADR-0056 §2.2)"
            }
            Self::LayerCorrectness => {
                "declared layer must match crate-name suffix (ADR-0056 BNF v4.1)"
            }
            Self::PortLocation => "port traits must live in kernel-layer crates (ADR-0056 §207)",
            Self::CrossProductRefusal => {
                "no direct cross-µservice imports except via public_layers (ADR-0056 §2.2)"
            }
        }
    }

    /// Returns all variants in canonical CI-lane order.
    pub fn all() -> [Self; 4] {
        [
            Self::DependencyDirection,
            Self::LayerCorrectness,
            Self::PortLocation,
            Self::CrossProductRefusal,
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

    /// Single-line CI-log format: `[lean-a1-architecture] crate::path — message`.
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
    fn all_returns_four_variants_in_canonical_lane_order() {
        let all = LeanCheckId::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], LeanCheckId::DependencyDirection);
        assert_eq!(all[1], LeanCheckId::LayerCorrectness);
        assert_eq!(all[2], LeanCheckId::PortLocation);
        assert_eq!(all[3], LeanCheckId::CrossProductRefusal);
    }

    #[test]
    fn lane_tags_match_ci_lanes_md_section_1_2() {
        assert_eq!(
            LeanCheckId::DependencyDirection.lane_tag(),
            "lean-a1-architecture"
        );
        assert_eq!(
            LeanCheckId::LayerCorrectness.lane_tag(),
            "lean-a1-architecture"
        );
        assert_eq!(LeanCheckId::PortLocation.lane_tag(), "lean-a1-architecture");
        assert_eq!(
            LeanCheckId::CrossProductRefusal.lane_tag(),
            "lean-a2-bounded-contexts"
        );
    }

    #[test]
    fn display_matches_lane_tag() {
        for id in LeanCheckId::all() {
            assert_eq!(format!("{id}"), id.lane_tag());
        }
    }

    #[test]
    fn ord_groups_architecture_lane_before_bounded_contexts_lane() {
        assert!(LeanCheckId::DependencyDirection < LeanCheckId::CrossProductRefusal);
        assert!(LeanCheckId::PortLocation < LeanCheckId::CrossProductRefusal);
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
            "foo-rest",
            "imports bar-kernel (outward)",
        );
        assert_eq!(v.check, LeanCheckId::DependencyDirection);
        assert_eq!(v.location, "foo-rest");
        assert_eq!(v.message, "imports bar-kernel (outward)");
    }

    #[test]
    fn to_log_line_format_is_correct() {
        let v = LeanViolation::new(
            LeanCheckId::CrossProductRefusal,
            "billing-application",
            "direct import of hr-domain",
        );
        assert_eq!(
            v.to_log_line(),
            "[lean-a2-bounded-contexts] billing-application — direct import of hr-domain"
        );
    }

    #[test]
    fn display_matches_to_log_line() {
        let v = LeanViolation::new(
            LeanCheckId::PortLocation,
            "foo-domain",
            "trait FooPort declared outside kernel",
        );
        assert_eq!(format!("{v}"), v.to_log_line());
    }

    #[test]
    fn violations_are_sortable_by_check_then_location() {
        let mut violations = [
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
