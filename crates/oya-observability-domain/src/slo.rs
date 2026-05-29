// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! SLO burn-rate alerting vocabulary.
//!
//! Provides:
//! - [`SLOObjective`]: a value object capturing a target reliability ratio and
//!   rolling measurement window.
//! - [`slo_fields`]: stable telemetry field-name constants for SLO/error-budget
//!   labels, aligned with OpenSLO spec fields and OTel semantic conventions.
//! - [`AlertDecision`]: a low-cardinality alert routing enum (Page/Ticket/None).
//! - [`classify_burn_rate`]: a pure multi-window burn-rate classifier per the
//!   Google SRE multi-window method (no allocation on the hot path).
//!
//! All items are annotated `data_class: INTERNAL_ONLY` — they carry operational
//! metrics that must not leak beyond the observability pipeline.
//!
//! # Implementation status
//!
//! STUB — types and signatures are present so tests compile; bodies are
//! intentionally wrong/unimplemented. The GREEN stage will fill them in.

// ---------------------------------------------------------------------------
// SLOObjective value object
// ---------------------------------------------------------------------------

/// Target SLO expressed as a ratio in the half-open interval (0, 1].
///
/// `target_ratio` is the fraction of requests/events that must succeed
/// (e.g. 0.999 for 99.9% availability). `window_secs` is the rolling
/// measurement window in seconds (e.g. 2_592_000 for 30 days).
///
/// Invariant: `target_ratio` is in (0.0, 1.0]. Use [`SLOObjective::new`] to
/// construct; the constructor rejects out-of-range values.
///
/// # data_class: INTERNAL_ONLY
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SLOObjective {
    target_ratio: f64, // data_class: INTERNAL_ONLY
    window_secs: u64,  // data_class: INTERNAL_ONLY
}

/// Error returned when [`SLOObjective::new`] rejects the supplied `target_ratio`.
///
/// # data_class: INTERNAL_ONLY
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSLOObjective {
    /// Human-readable reason for the rejection.
    pub reason: &'static str,
}

impl std::fmt::Display for InvalidSLOObjective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid SLO objective: {}", self.reason)
    }
}

impl std::error::Error for InvalidSLOObjective {}

impl SLOObjective {
    /// Construct a new `SLOObjective`.
    ///
    /// Returns `Err(InvalidSLOObjective)` when `target_ratio` is not in (0.0, 1.0].
    ///
    /// STUB: always returns Ok regardless of target_ratio — RED.
    pub fn new(target_ratio: f64, window_secs: u64) -> Result<Self, InvalidSLOObjective> {
        // STUB: validation not yet implemented
        let _ = target_ratio;
        let _ = window_secs;
        todo!("sbr-1: implement SLOObjective::new validation")
    }

    /// Return the target reliability ratio.
    ///
    /// # data_class: INTERNAL_ONLY
    pub fn target_ratio(&self) -> f64 {
        todo!("sbr-1: implement SLOObjective::target_ratio")
    }

    /// Return the rolling measurement window in seconds.
    ///
    /// # data_class: INTERNAL_ONLY
    pub fn window_secs(&self) -> u64 {
        todo!("sbr-1: implement SLOObjective::window_secs")
    }
}

// ---------------------------------------------------------------------------
// Stable telemetry field-name constants
// ---------------------------------------------------------------------------

/// Stable field-name constants for SLO and error-budget telemetry labels.
///
/// These wire names are consumed by runtime adapter crates that construct OTel
/// spans and metrics. They are aligned with OpenSLO `SLO.spec.objectives` fields
/// and OTel semantic conventions for `system.*` / custom `oyatie.*` namespaces.
///
/// **Do not change these strings without a deprecation cycle** — downstream
/// exporters, dashboards, and `microservices/<ms>/slos/*.openslo.yaml` documents
/// depend on them.
///
/// # data_class: INTERNAL_ONLY (all constants in this module)
///
/// STUB: wrong placeholder values — RED.
pub mod slo_fields {
    /// Name of the SLO (e.g. `"availability-99.9"`).
    ///
    /// # data_class: INTERNAL_ONLY
    pub const SLO_NAME: &str = "STUB_SLO_NAME";

    /// Target reliability ratio expressed as a decimal fraction (e.g. `0.999`).
    ///
    /// Maps to OpenSLO `SLO.spec.objectives[].target`.
    ///
    /// # data_class: INTERNAL_ONLY
    pub const SLO_OBJECTIVE_RATIO: &str = "STUB_SLO_OBJECTIVE_RATIO";

    /// Remaining error-budget fraction (0.0 = exhausted, 1.0 = fully remaining).
    ///
    /// # data_class: INTERNAL_ONLY
    pub const ERROR_BUDGET_REMAINING: &str = "STUB_ERROR_BUDGET_REMAINING";

    /// Current burn rate (dimensionless multiplier relative to the target ratio).
    ///
    /// # data_class: INTERNAL_ONLY
    pub const BURN_RATE: &str = "STUB_BURN_RATE";
}

// ---------------------------------------------------------------------------
// AlertDecision enum
// ---------------------------------------------------------------------------

/// Low-cardinality alert routing decision produced by the multi-window
/// burn-rate classifier.
///
/// - `Page`: triggers immediate on-call escalation.
/// - `Ticket`: opens a next-business-day work item.
/// - `None`: quiescent; no action required.
///
/// # data_class: INTERNAL_ONLY
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AlertDecision {
    /// Immediate on-call page.
    Page,
    /// Create a next-business-day ticket.
    Ticket,
    /// No alert action required.
    None,
}

// ---------------------------------------------------------------------------
// Threshold constants (Google SRE multi-window method)
// ---------------------------------------------------------------------------

/// Page-tier burn-rate threshold.
///
/// STUB: wrong value — RED.
pub const PAGE_BURN_RATE_THRESHOLD: f64 = 0.0;

/// Minimum error-budget fraction consumed before firing a page.
///
/// STUB: wrong value — RED.
pub const PAGE_BUDGET_CONSUMED_MIN: f64 = 0.0;

/// Ticket-tier burn-rate threshold.
///
/// STUB: wrong value — RED.
pub const TICKET_BURN_RATE_THRESHOLD: f64 = 0.0;

/// Minimum error-budget fraction consumed before opening a ticket.
///
/// STUB: wrong value — RED.
pub const TICKET_BUDGET_CONSUMED_MIN: f64 = 0.0;

// ---------------------------------------------------------------------------
// classify_burn_rate — pure multi-window classifier (STUB)
// ---------------------------------------------------------------------------

/// Pure multi-window multi-burn-rate alert classifier.
///
/// STUB: always returns `AlertDecision::None` — RED.
///
/// # data_class: INTERNAL_ONLY (inputs are operational metrics)
pub fn classify_burn_rate(
    _error_budget_consumed: f64,
    _fast_burn_rate: f64,
    _slow_burn_rate: f64,
) -> AlertDecision {
    // STUB: classifier not yet implemented
    AlertDecision::None
}

// ---------------------------------------------------------------------------
// Inline unit tests (sbr-1 acceptance + classifier hot-path)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- SLOObjective constructor ---

    #[test]
    fn slo_objective_accepts_typical_three_nines_ratio() {
        let obj = SLOObjective::new(0.999, 2_592_000).unwrap();
        assert_eq!(obj.target_ratio(), 0.999);
        assert_eq!(obj.window_secs(), 2_592_000);
    }

    #[test]
    fn slo_objective_accepts_boundary_ratio_one_point_zero() {
        let obj = SLOObjective::new(1.0, 86_400).unwrap();
        assert_eq!(obj.target_ratio(), 1.0);
    }

    #[test]
    fn slo_objective_accepts_very_small_but_positive_ratio() {
        let obj = SLOObjective::new(0.001, 86_400).unwrap();
        assert_eq!(obj.target_ratio(), 0.001);
    }

    #[test]
    fn slo_objective_rejects_zero_ratio() {
        let err = SLOObjective::new(0.0, 86_400).unwrap_err();
        assert_eq!(err.reason, "target_ratio must be in (0.0, 1.0]");
    }

    #[test]
    fn slo_objective_rejects_ratio_above_one() {
        let err = SLOObjective::new(1.001, 86_400).unwrap_err();
        assert_eq!(err.reason, "target_ratio must be in (0.0, 1.0]");
    }

    #[test]
    fn slo_objective_rejects_negative_ratio() {
        let err = SLOObjective::new(-0.5, 86_400).unwrap_err();
        assert_eq!(err.reason, "target_ratio must be in (0.0, 1.0]");
    }

    #[test]
    fn slo_objective_accessors_return_stored_values_without_mutation() {
        let obj = SLOObjective::new(0.95, 604_800).unwrap();
        assert_eq!(obj.target_ratio(), 0.95);
        assert_eq!(obj.window_secs(), 604_800);
        // Second call must return the same value (not mutating).
        assert_eq!(obj.target_ratio(), 0.95);
    }

    #[test]
    fn invalid_slo_objective_display_includes_reason() {
        let err = InvalidSLOObjective {
            reason: "target_ratio must be in (0.0, 1.0]",
        };
        assert!(err.to_string().contains("target_ratio must be in (0.0, 1.0]"));
    }

    // --- slo_fields constants ---

    #[test]
    fn slo_fields_wire_values_are_stable() {
        assert_eq!(slo_fields::SLO_NAME, "oyatie.slo.name");
        assert_eq!(slo_fields::SLO_OBJECTIVE_RATIO, "oyatie.slo.objective_ratio");
        assert_eq!(
            slo_fields::ERROR_BUDGET_REMAINING,
            "oyatie.slo.error_budget_remaining"
        );
        assert_eq!(slo_fields::BURN_RATE, "oyatie.slo.burn_rate");
    }

    // --- classify_burn_rate inline unit tests ---

    #[test]
    fn classify_both_windows_above_page_threshold_returns_page() {
        assert_eq!(
            classify_burn_rate(0.03, 15.0, 15.0),
            AlertDecision::Page
        );
    }

    #[test]
    fn classify_fast_below_page_slow_above_returns_none_or_lower_tier() {
        // fast=10 is below PAGE threshold (14.4); does not qualify for page
        let decision = classify_burn_rate(0.03, 10.0, 15.0);
        assert_ne!(decision, AlertDecision::Page);
    }

    #[test]
    fn classify_fast_above_page_slow_below_returns_none_or_lower_tier() {
        // slow=10 is below PAGE threshold (14.4); does not qualify for page
        let decision = classify_burn_rate(0.03, 15.0, 10.0);
        assert_ne!(decision, AlertDecision::Page);
    }

    #[test]
    fn classify_page_budget_not_consumed_enough_returns_none_or_lower_tier() {
        // consumed=0.01 is below PAGE_BUDGET_CONSUMED_MIN (0.02)
        let decision = classify_burn_rate(0.01, 15.0, 15.0);
        assert_ne!(decision, AlertDecision::Page);
    }

    #[test]
    fn classify_both_windows_above_ticket_threshold_returns_ticket() {
        assert_eq!(
            classify_burn_rate(0.06, 7.0, 7.0),
            AlertDecision::Ticket
        );
    }

    #[test]
    fn classify_ticket_budget_not_consumed_enough_returns_none() {
        // consumed=0.04 < TICKET_BUDGET_CONSUMED_MIN (0.05)
        assert_eq!(
            classify_burn_rate(0.04, 7.0, 7.0),
            AlertDecision::None
        );
    }

    #[test]
    fn classify_below_all_thresholds_returns_none() {
        assert_eq!(
            classify_burn_rate(0.50, 1.0, 1.0),
            AlertDecision::None
        );
    }

    #[test]
    fn classify_page_wins_over_ticket_when_both_conditions_met() {
        // exceeds both page and ticket budget minimums; page check fires first
        assert_eq!(
            classify_burn_rate(0.10, 15.0, 15.0),
            AlertDecision::Page
        );
    }

    #[test]
    fn classify_exact_page_boundary_returns_page() {
        assert_eq!(
            classify_burn_rate(0.02, 14.4, 14.4),
            AlertDecision::Page
        );
    }
}
