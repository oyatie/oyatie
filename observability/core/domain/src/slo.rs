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
    pub fn new(target_ratio: f64, window_secs: u64) -> Result<Self, InvalidSLOObjective> {
        if target_ratio <= 0.0 || target_ratio > 1.0 {
            return Err(InvalidSLOObjective {
                reason: "target_ratio must be in (0.0, 1.0]",
            });
        }
        Ok(Self {
            target_ratio,
            window_secs,
        })
    }

    /// Return the target reliability ratio.
    ///
    /// # data_class: INTERNAL_ONLY
    pub fn target_ratio(&self) -> f64 {
        self.target_ratio
    }

    /// Return the rolling measurement window in seconds.
    ///
    /// # data_class: INTERNAL_ONLY
    pub fn window_secs(&self) -> u64 {
        self.window_secs
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
pub mod slo_fields {
    /// Name of the SLO (e.g. `"availability-99.9"`).
    ///
    /// # data_class: INTERNAL_ONLY
    pub const SLO_NAME: &str = "oyatie.slo.name";

    /// Target reliability ratio expressed as a decimal fraction (e.g. `0.999`).
    ///
    /// Maps to OpenSLO `SLO.spec.objectives[].target`.
    ///
    /// # data_class: INTERNAL_ONLY
    pub const SLO_OBJECTIVE_RATIO: &str = "oyatie.slo.objective_ratio";

    /// Remaining error-budget fraction (0.0 = exhausted, 1.0 = fully remaining).
    ///
    /// # data_class: INTERNAL_ONLY
    pub const ERROR_BUDGET_REMAINING: &str = "oyatie.slo.error_budget_remaining";

    /// Current burn rate (dimensionless multiplier relative to the target ratio).
    ///
    /// # data_class: INTERNAL_ONLY
    pub const BURN_RATE: &str = "oyatie.slo.burn_rate";
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

/// Page-tier burn-rate threshold (14.4×).
///
/// Per the Google SRE multi-window method: both fast and slow windows must
/// exceed this multiplier and the budget consumed must exceed
/// [`PAGE_BUDGET_CONSUMED_MIN`] to fire a page.
///
/// # data_class: INTERNAL_ONLY
pub const PAGE_BURN_RATE_THRESHOLD: f64 = 14.4;

/// Minimum error-budget fraction consumed before firing a page (2%).
///
/// # data_class: INTERNAL_ONLY
pub const PAGE_BUDGET_CONSUMED_MIN: f64 = 0.02;

/// Ticket-tier burn-rate threshold (6.0×).
///
/// Both fast and slow windows must exceed this multiplier and the budget
/// consumed must exceed [`TICKET_BUDGET_CONSUMED_MIN`] to open a ticket.
///
/// # data_class: INTERNAL_ONLY
pub const TICKET_BURN_RATE_THRESHOLD: f64 = 6.0;

/// Minimum error-budget fraction consumed before opening a ticket (5%).
///
/// # data_class: INTERNAL_ONLY
pub const TICKET_BUDGET_CONSUMED_MIN: f64 = 0.05;

// ---------------------------------------------------------------------------
// classify_burn_rate — pure multi-window classifier (STUB)
// ---------------------------------------------------------------------------

/// Pure multi-window multi-burn-rate alert classifier.
///
/// Implements the Google SRE Workbook "Multiwindow, Multi-Burn-Rate Alerts"
/// method (Chapter 5). Both the fast window and the slow window burn rates
/// must exceed a tier's threshold **and** the error-budget consumed must meet
/// the tier's minimum before that tier fires.  Page is checked before Ticket
/// so the higher-severity decision always wins.
///
/// No allocation occurs on the hot path.
///
/// # data_class: INTERNAL_ONLY (inputs are operational metrics)
pub fn classify_burn_rate(
    error_budget_consumed: f64,
    fast_burn_rate: f64,
    slow_burn_rate: f64,
) -> AlertDecision {
    if fast_burn_rate >= PAGE_BURN_RATE_THRESHOLD
        && slow_burn_rate >= PAGE_BURN_RATE_THRESHOLD
        && error_budget_consumed >= PAGE_BUDGET_CONSUMED_MIN
    {
        return AlertDecision::Page;
    }
    if fast_burn_rate >= TICKET_BURN_RATE_THRESHOLD
        && slow_burn_rate >= TICKET_BURN_RATE_THRESHOLD
        && error_budget_consumed >= TICKET_BUDGET_CONSUMED_MIN
    {
        return AlertDecision::Ticket;
    }
    AlertDecision::None
}

// ---------------------------------------------------------------------------
// slo::budget — error-budget / burn-rate computation kernel
// ---------------------------------------------------------------------------

/// Pure error-budget / burn-rate computation kernel.
///
/// Turns raw good/bad event counts plus an [`SLOObjective`] into the
/// burn-rate and error-budget-remaining inputs that [`classify_burn_rate`]
/// already consumes, and provides a one-call helper that wires both
/// together.
///
/// No allocation occurs on the hot path; all functions are pure and
/// deterministic.
///
/// # data_class: INTERNAL_ONLY
pub mod budget {
    use super::{AlertDecision, SLOObjective, classify_burn_rate};

    // -----------------------------------------------------------------------
    // BudgetWindow value object
    // -----------------------------------------------------------------------

    /// A pair of raw good/bad event counts representing one observation window.
    ///
    /// `bad_events` is clamped to `total` (i.e. `bad_events > total` saturates
    /// to `total`) so downstream callers never receive a ratio above 1.0.
    ///
    /// # data_class: INTERNAL_ONLY
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct BudgetWindow {
        /// Events that met the SLO (good requests).
        pub good_events: u64, // data_class: INTERNAL_ONLY
        /// Events that did not meet the SLO (bad requests).
        pub bad_events: u64, // data_class: INTERNAL_ONLY
    }

    impl BudgetWindow {
        /// Construct a new `BudgetWindow`.
        ///
        /// # data_class: INTERNAL_ONLY
        pub const fn new(good_events: u64, bad_events: u64) -> Self {
            Self {
                good_events,
                bad_events,
            }
        }

        /// Total event count (`good + bad`), saturating at `u64::MAX`.
        ///
        /// # data_class: INTERNAL_ONLY
        #[inline]
        pub fn total(&self) -> u64 {
            self.good_events.saturating_add(self.bad_events)
        }

        /// Effective bad-event count, clamped to total to guard against
        /// `bad_events > total` saturation inputs.
        ///
        /// # data_class: INTERNAL_ONLY
        #[inline]
        fn effective_bad(&self) -> u64 {
            let total = self.total();
            self.bad_events.min(total)
        }

        /// Observed bad-event ratio in [0.0, 1.0].
        ///
        /// Returns `0.0` when `total == 0` (fail-open on zero traffic:
        /// no evidence of failure → full budget).
        ///
        /// # data_class: INTERNAL_ONLY
        #[inline]
        fn observed_bad_ratio(&self) -> f64 {
            let total = self.total();
            if total == 0 {
                return 0.0;
            }
            (self.effective_bad() as f64) / (total as f64)
        }
    }

    // -----------------------------------------------------------------------
    // error_budget_remaining_ratio
    // -----------------------------------------------------------------------

    /// Compute the fraction of the error budget that remains, in [0.0, 1.0].
    ///
    /// `1.0` means the full budget is intact; `0.0` means it is fully
    /// exhausted (or the `target_ratio` is 1.0 so the allowed budget is
    /// zero — in that case any bad event exhausts the budget immediately,
    /// and the function returns `0.0`).
    ///
    /// When `total == 0` the function returns `1.0` (no failures observed
    /// → full budget remaining).
    ///
    /// The result is clamped to [0.0, 1.0].
    ///
    /// # data_class: INTERNAL_ONLY
    pub fn error_budget_remaining_ratio(objective: SLOObjective, window: BudgetWindow) -> f64 {
        let allowed_bad_ratio = 1.0 - objective.target_ratio();
        if allowed_bad_ratio <= 0.0 {
            // target_ratio == 1.0 → zero tolerance; any bad event exhausts the budget.
            if window.bad_events == 0 {
                return 1.0;
            }
            return 0.0;
        }

        let observed = window.observed_bad_ratio();
        // budget_consumed = observed / allowed; remaining = 1 - consumed
        let remaining = 1.0 - (observed / allowed_bad_ratio);
        remaining.clamp(0.0, 1.0)
    }

    // -----------------------------------------------------------------------
    // burn_rate
    // -----------------------------------------------------------------------

    /// Compute the burn rate: observed bad ratio divided by allowed bad ratio.
    ///
    /// A burn rate of `1.0` means the SLO is being consumed exactly on pace;
    /// `14.4` means the window is burning through the error budget 14.4×
    /// faster than allowed (page-tier threshold).
    ///
    /// Returns `0.0` when `total == 0` (zero burn on no traffic).
    ///
    /// Returns `f64::INFINITY` when `target_ratio == 1.0` and there are bad
    /// events (infinite burn rate on a 100%-target SLO with any failures).
    ///
    /// # data_class: INTERNAL_ONLY
    pub fn burn_rate(objective: SLOObjective, window: BudgetWindow) -> f64 {
        let allowed_bad_ratio = 1.0 - objective.target_ratio();
        if allowed_bad_ratio <= 0.0 {
            // Zero tolerance: any bad event → infinite burn; no bad events → 0.
            if window.bad_events == 0 {
                return 0.0;
            }
            return f64::INFINITY;
        }

        let observed = window.observed_bad_ratio();
        if observed == 0.0 {
            return 0.0;
        }
        observed / allowed_bad_ratio
    }

    // -----------------------------------------------------------------------
    // classify_budget_windows — one-call helper
    // -----------------------------------------------------------------------

    /// Classify two observation windows against an SLO objective and return
    /// the appropriate [`AlertDecision`].
    ///
    /// This is a thin adapter that:
    /// 1. Computes `error_budget_remaining_ratio` from the fast window.
    /// 2. Computes `burn_rate` for both windows.
    /// 3. Feeds the results into [`classify_burn_rate`].
    ///
    /// The `fast_window` is used to derive `error_budget_consumed`; both
    /// windows contribute their individual burn rates.
    ///
    /// # data_class: INTERNAL_ONLY
    pub fn classify_budget_windows(
        objective: SLOObjective,
        fast_window: BudgetWindow,
        slow_window: BudgetWindow,
    ) -> AlertDecision {
        let budget_remaining = error_budget_remaining_ratio(objective, fast_window);
        let error_budget_consumed = 1.0 - budget_remaining;
        let fast_burn = burn_rate(objective, fast_window);
        let slow_burn = burn_rate(objective, slow_window);
        classify_burn_rate(error_budget_consumed, fast_burn, slow_burn)
    }

    // -----------------------------------------------------------------------
    // Inline unit tests
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::slo::{AlertDecision, SLOObjective};

        fn obj(target: f64) -> SLOObjective {
            SLOObjective::new(target, 3_600).unwrap()
        }

        // 1. Clean budget: no bad events → remaining = 1.0, burn_rate = 0.0
        #[test]
        fn clean_budget_no_bad_events_returns_full_remaining_and_zero_burn() {
            let w = BudgetWindow::new(1_000, 0);
            assert_eq!(error_budget_remaining_ratio(obj(0.999), w), 1.0);
            assert_eq!(burn_rate(obj(0.999), w), 0.0);
        }

        // 2. Exhausted budget: bad events == allowed → remaining = 0.0
        #[test]
        fn exhausted_budget_returns_zero_remaining() {
            // 99.9% SLO → allowed bad ratio = 0.001
            // 1 bad out of 1000 total → observed = 0.001 = allowed → consumed = 1.0
            let w = BudgetWindow::new(999, 1);
            let remaining = error_budget_remaining_ratio(obj(0.999), w);
            assert!((remaining - 0.0).abs() < 1e-9, "remaining={remaining}");
        }

        // 3. Partially consumed budget
        #[test]
        fn partial_budget_returns_proportional_remaining() {
            // 99% SLO → allowed bad ratio = 0.01
            // 5 bad out of 1000 → observed = 0.005 → consumed = 0.5 → remaining = 0.5
            let w = BudgetWindow::new(995, 5);
            let remaining = error_budget_remaining_ratio(obj(0.99), w);
            assert!((remaining - 0.5).abs() < 1e-9, "remaining={remaining}");
        }

        // 4. Zero total events → remaining = 1.0, burn = 0.0
        #[test]
        fn zero_total_events_returns_full_remaining_and_zero_burn() {
            let w = BudgetWindow::new(0, 0);
            assert_eq!(error_budget_remaining_ratio(obj(0.999), w), 1.0);
            assert_eq!(burn_rate(obj(0.999), w), 0.0);
        }

        // 5. bad_events > total saturation is handled (bad_events saturates at total)
        #[test]
        fn bad_events_exceeding_total_saturates_at_total() {
            // good=0, bad=u64::MAX → effective bad = total = u64::MAX, ratio = 1.0
            let w = BudgetWindow::new(0, u64::MAX);
            // remaining should be 0.0 (fully exhausted / over-budget)
            let remaining = error_budget_remaining_ratio(obj(0.999), w);
            assert_eq!(remaining, 0.0, "remaining={remaining}");
        }

        // 6. target_ratio == 1.0 with no bad events → remaining = 1.0
        #[test]
        fn target_ratio_one_no_bad_events_returns_full_remaining() {
            let w = BudgetWindow::new(1_000, 0);
            assert_eq!(error_budget_remaining_ratio(obj(1.0), w), 1.0);
            assert_eq!(burn_rate(obj(1.0), w), 0.0);
        }

        // 7. target_ratio == 1.0 with any bad events → remaining = 0.0, burn = INFINITY
        #[test]
        fn target_ratio_one_with_bad_events_returns_zero_remaining_and_infinite_burn() {
            let w = BudgetWindow::new(999, 1);
            assert_eq!(error_budget_remaining_ratio(obj(1.0), w), 0.0);
            assert_eq!(burn_rate(obj(1.0), w), f64::INFINITY);
        }

        // 8. PAGE alert fires when both windows exceed page threshold
        #[test]
        fn classify_budget_windows_page_fires_on_high_burn_both_windows() {
            // 99.9% SLO → allowed bad = 0.001
            // burn rate of 14.4 → observed bad ratio = 14.4 * 0.001 = 0.0144
            // ~14 bad events out of 972 total → 0.0144 ratio
            let fast = BudgetWindow::new(958, 14); // ~1.44% bad → burn ≈ 14.4
            let slow = BudgetWindow::new(958, 14);
            let decision = classify_budget_windows(obj(0.999), fast, slow);
            assert_eq!(decision, AlertDecision::Page);
        }

        // 9. TICKET alert fires when both windows exceed ticket threshold but not page
        #[test]
        fn classify_budget_windows_ticket_fires_on_moderate_burn_both_windows() {
            // 99% SLO → allowed bad = 0.01
            // burn rate of 7.0 → observed bad ratio = 0.07 → 7 bad out of 100
            let fast = BudgetWindow::new(93, 7); // 7% bad → burn = 7.0
            let slow = BudgetWindow::new(93, 7);
            let decision = classify_budget_windows(obj(0.99), fast, slow);
            assert_eq!(decision, AlertDecision::Ticket);
        }

        // 10. No alert when burn rate is low
        #[test]
        fn classify_budget_windows_none_on_low_burn() {
            // 99% SLO → allowed bad = 0.01
            // 1 bad out of 1000 → burn = 0.1 → no alert
            let fast = BudgetWindow::new(999, 1);
            let slow = BudgetWindow::new(999, 1);
            let decision = classify_budget_windows(obj(0.99), fast, slow);
            assert_eq!(decision, AlertDecision::None);
        }

        // 11. Burn rate well above PAGE boundary fires PAGE
        #[test]
        fn burn_rate_at_exactly_page_boundary_returns_page() {
            // 99.9% SLO → allowed bad = 0.001; 15 bad out of 1000 → burn = 15.0
            // (15.0 > 14.4 PAGE_BURN_RATE_THRESHOLD) and consumed > 2%
            let fast = BudgetWindow::new(985, 15);
            let slow = BudgetWindow::new(985, 15);
            let decision = classify_budget_windows(obj(0.999), fast, slow);
            assert_eq!(decision, AlertDecision::Page);
        }

        // 12. Over-budget remaining is clamped to 0.0 (not negative)
        #[test]
        fn over_budget_remaining_is_clamped_to_zero_not_negative() {
            // 500 bad out of 1000 total, SLO = 99.9% → massively over budget
            let w = BudgetWindow::new(500, 500);
            let remaining = error_budget_remaining_ratio(obj(0.999), w);
            assert_eq!(remaining, 0.0);
        }

        // 13. BudgetWindow total() uses saturating add
        #[test]
        fn budget_window_total_saturates_on_overflow() {
            let w = BudgetWindow::new(u64::MAX, u64::MAX);
            assert_eq!(w.total(), u64::MAX); // saturating_add
        }
    }
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
        assert!(
            err.to_string()
                .contains("target_ratio must be in (0.0, 1.0]")
        );
    }

    // --- slo_fields constants ---

    #[test]
    fn slo_fields_wire_values_are_stable() {
        assert_eq!(slo_fields::SLO_NAME, "oyatie.slo.name");
        assert_eq!(
            slo_fields::SLO_OBJECTIVE_RATIO,
            "oyatie.slo.objective_ratio"
        );
        assert_eq!(
            slo_fields::ERROR_BUDGET_REMAINING,
            "oyatie.slo.error_budget_remaining"
        );
        assert_eq!(slo_fields::BURN_RATE, "oyatie.slo.burn_rate");
    }

    // --- classify_burn_rate inline unit tests ---

    #[test]
    fn classify_both_windows_above_page_threshold_returns_page() {
        assert_eq!(classify_burn_rate(0.03, 15.0, 15.0), AlertDecision::Page);
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
        assert_eq!(classify_burn_rate(0.06, 7.0, 7.0), AlertDecision::Ticket);
    }

    #[test]
    fn classify_ticket_budget_not_consumed_enough_returns_none() {
        // consumed=0.04 < TICKET_BUDGET_CONSUMED_MIN (0.05)
        assert_eq!(classify_burn_rate(0.04, 7.0, 7.0), AlertDecision::None);
    }

    #[test]
    fn classify_below_all_thresholds_returns_none() {
        assert_eq!(classify_burn_rate(0.50, 1.0, 1.0), AlertDecision::None);
    }

    #[test]
    fn classify_page_wins_over_ticket_when_both_conditions_met() {
        // exceeds both page and ticket budget minimums; page check fires first
        assert_eq!(classify_burn_rate(0.10, 15.0, 15.0), AlertDecision::Page);
    }

    #[test]
    fn classify_exact_page_boundary_returns_page() {
        assert_eq!(classify_burn_rate(0.02, 14.4, 14.4), AlertDecision::Page);
    }
}
