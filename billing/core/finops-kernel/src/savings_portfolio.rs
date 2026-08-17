//! Savings portfolio rollup (cloud-finops-savings-portfolio-rollup).
//!
//! Aggregates a slice of `Recommendation`s against their baseline `CostReport`(s)
//! into a deterministic `SavingsPortfolio` projection.
//! Pure std-only; no I/O, no async, no new external dependencies.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::HashMap;

use crate::{CostReport, Recommendation, RecommendationKind, RecommendationState};

/// A deterministic projection of projected savings across a set of recommendations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavingsPortfolio {
    /// Sum of `estimated_savings_micros` over Active and Applied recommendations.
    // data_class: INTERNAL_ONLY
    pub estimated_savings_micros: u128,
    /// Count of contributing (Active + Applied) recommendations per kind.
    // data_class: INTERNAL_ONLY
    pub counts_by_kind: HashMap<RecommendationKind, u32>,
    /// Projected savings as a fraction of total baseline spend, in basis points
    /// (1 bps = 0.01 %). Saturating; capped at 10_000 (= 100 %).
    // data_class: INTERNAL_ONLY
    pub coverage_bps: u16,
}

/// Errors produced by [`roll_up_savings`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RollupError {
    /// A contributing recommendation references a `baseline_report_id` that is
    /// absent from the supplied `reports` slice.
    MissingBaselineReport {
        /// The report id that could not be resolved.
        baseline_report_id: String,
    },
}

impl RollupError {
    /// Human-readable description of the error.
    pub fn message(&self) -> String {
        match self {
            Self::MissingBaselineReport { baseline_report_id } => {
                format!(
                    "baseline report '{baseline_report_id}' referenced by recommendation is not present in the supplied reports"
                )
            }
        }
    }
}

/// Aggregate `recommendations` into a [`SavingsPortfolio`] projection.
///
/// Only `Active` and `Applied` recommendations contribute; `Draft` and `Dismissed`
/// are silently skipped.
///
/// Returns `Err(RollupError::MissingBaselineReport)` if a contributing
/// recommendation has a `baseline_report_id` that is not present in `reports`.
pub fn roll_up_savings(
    recommendations: &[Recommendation],
    reports: &[CostReport],
) -> Result<SavingsPortfolio, RollupError> {
    // Build a quick lookup from report_id -> total_spend_micros.
    let report_index: HashMap<&str, u128> = reports
        .iter()
        .map(|r| (r.report_id.as_str(), r.total_spend_micros))
        .collect();

    let mut estimated_savings_micros: u128 = 0;
    let mut counts_by_kind: HashMap<RecommendationKind, u32> = HashMap::new();
    // Collect the set of unique baseline report ids referenced by contributors.
    let mut referenced_report_ids: HashMap<&str, u128> = HashMap::new();

    for rec in recommendations {
        // Only Active and Applied contribute.
        match rec.state {
            RecommendationState::Active | RecommendationState::Applied => {}
            RecommendationState::Draft | RecommendationState::Dismissed => continue,
        }

        // Validate baseline_report_id if present.
        if let Some(ref baseline_id) = rec.baseline_report_id {
            match report_index.get(baseline_id.as_str()) {
                Some(&spend) => {
                    referenced_report_ids
                        .entry(baseline_id.as_str())
                        .or_insert(spend);
                }
                None => {
                    return Err(RollupError::MissingBaselineReport {
                        baseline_report_id: baseline_id.clone(),
                    });
                }
            }
        }

        estimated_savings_micros =
            estimated_savings_micros.saturating_add(rec.estimated_savings_micros);
        *counts_by_kind.entry(rec.kind).or_insert(0) += 1;
    }

    // Sum baseline spend from all uniquely referenced reports.
    let total_baseline_spend_micros: u128 = referenced_report_ids.values().copied().sum();

    let coverage_bps = if total_baseline_spend_micros == 0 {
        0u16
    } else {
        // (savings * 10_000) / spend, saturating at u16::MAX then capped at 10_000.
        let bps = estimated_savings_micros.saturating_mul(10_000) / total_baseline_spend_micros;
        if bps >= 10_000 {
            10_000u16
        } else {
            // Safe: bps < 10_000 < u16::MAX
            bps as u16
        }
    };

    Ok(SavingsPortfolio {
        estimated_savings_micros,
        counts_by_kind,
        coverage_bps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CostReport, Recommendation, RecommendationKind, RecommendationState, ReportPeriod,
    };

    fn report(id: &str, spend: u128) -> CostReport {
        CostReport {
            report_id: id.into(),
            period: ReportPeriod::Monthly,
            period_start_unix_ms: 1_000,
            period_end_unix_ms: 2_000,
            total_spend_micros: spend,
        }
    }

    fn rec(
        id: &str,
        kind: RecommendationKind,
        baseline: Option<&str>,
        savings: u128,
        state: RecommendationState,
    ) -> Recommendation {
        Recommendation {
            recommendation_id: id.into(),
            kind,
            baseline_report_id: baseline.map(String::from),
            estimated_savings_micros: savings,
            state,
        }
    }

    // (a) empty set -> zero portfolio
    #[test]
    fn empty_recommendations_returns_zero_portfolio() {
        let result = roll_up_savings(&[], &[]).unwrap();
        assert_eq!(result.estimated_savings_micros, 0);
        assert!(result.counts_by_kind.is_empty());
        assert_eq!(result.coverage_bps, 0);
    }

    // (b) mixed states: only Active + Applied count
    #[test]
    fn mixed_states_only_counts_active_and_applied() {
        let reps = vec![report("r1", 1_000_000)];
        let recs = vec![
            rec(
                "d1",
                RecommendationKind::RightsizeInstance,
                Some("r1"),
                100_000,
                RecommendationState::Draft,
            ),
            rec(
                "d2",
                RecommendationKind::RightsizeInstance,
                Some("r1"),
                200_000,
                RecommendationState::Dismissed,
            ),
            rec(
                "a1",
                RecommendationKind::RightsizeInstance,
                Some("r1"),
                50_000,
                RecommendationState::Active,
            ),
            rec(
                "a2",
                RecommendationKind::StorageTier,
                Some("r1"),
                30_000,
                RecommendationState::Applied,
            ),
        ];
        let result = roll_up_savings(&recs, &reps).unwrap();
        // Only a1 + a2 contribute
        assert_eq!(result.estimated_savings_micros, 80_000);
        assert_eq!(
            *result
                .counts_by_kind
                .get(&RecommendationKind::RightsizeInstance)
                .unwrap(),
            1
        );
        assert_eq!(
            *result
                .counts_by_kind
                .get(&RecommendationKind::StorageTier)
                .unwrap(),
            1
        );
        assert!(
            !result
                .counts_by_kind
                .contains_key(&RecommendationKind::SpotCommit)
        );
    }

    // (c) missing baseline -> error
    #[test]
    fn missing_baseline_report_returns_error() {
        let recs = vec![rec(
            "a1",
            RecommendationKind::RightsizeInstance,
            Some("missing-id"),
            50_000,
            RecommendationState::Active,
        )];
        let result = roll_up_savings(&recs, &[]);
        assert!(matches!(
            result,
            Err(RollupError::MissingBaselineReport { ref baseline_report_id }) if baseline_report_id == "missing-id"
        ));
    }

    // (d) coverage_bps saturates at 10_000 when savings >= spend
    #[test]
    fn coverage_bps_saturates_at_ten_thousand() {
        let reps = vec![report("r1", 1_000)];
        let recs = vec![rec(
            "a1",
            RecommendationKind::RightsizeInstance,
            Some("r1"),
            5_000,
            RecommendationState::Active,
        )];
        let result = roll_up_savings(&recs, &reps).unwrap();
        assert_eq!(result.coverage_bps, 10_000);
    }

    // (e) per-kind counts correct
    #[test]
    fn per_kind_counts_are_correct() {
        let reps = vec![report("r1", 10_000_000)];
        let recs = vec![
            rec(
                "r1",
                RecommendationKind::RightsizeInstance,
                Some("r1"),
                100_000,
                RecommendationState::Active,
            ),
            rec(
                "r2",
                RecommendationKind::RightsizeInstance,
                Some("r1"),
                100_000,
                RecommendationState::Applied,
            ),
            rec(
                "r3",
                RecommendationKind::SpotCommit,
                Some("r1"),
                50_000,
                RecommendationState::Active,
            ),
            rec(
                "r4",
                RecommendationKind::ReservedCapacityPurchase,
                Some("r1"),
                75_000,
                RecommendationState::Active,
            ),
            rec(
                "r5",
                RecommendationKind::UnusedResourceCleanup,
                None,
                20_000,
                RecommendationState::Draft,
            ),
        ];
        let result = roll_up_savings(&recs, &reps).unwrap();
        assert_eq!(
            *result
                .counts_by_kind
                .get(&RecommendationKind::RightsizeInstance)
                .unwrap(),
            2
        );
        assert_eq!(
            *result
                .counts_by_kind
                .get(&RecommendationKind::SpotCommit)
                .unwrap(),
            1
        );
        assert_eq!(
            *result
                .counts_by_kind
                .get(&RecommendationKind::ReservedCapacityPurchase)
                .unwrap(),
            1
        );
        // Draft UnusedResourceCleanup should not appear
        assert!(
            !result
                .counts_by_kind
                .contains_key(&RecommendationKind::UnusedResourceCleanup)
        );
        // Total savings: 100_000 + 100_000 + 50_000 + 75_000 = 325_000
        assert_eq!(result.estimated_savings_micros, 325_000);
    }

    // Extra: zero baseline spend yields zero coverage_bps
    #[test]
    fn zero_baseline_spend_yields_zero_bps() {
        let reps = vec![report("r1", 0)];
        let recs = vec![rec(
            "a1",
            RecommendationKind::RightsizeInstance,
            Some("r1"),
            1_000,
            RecommendationState::Active,
        )];
        let result = roll_up_savings(&recs, &reps).unwrap();
        assert_eq!(result.coverage_bps, 0);
    }

    // Extra: recommendation without baseline_report_id contributes savings but not spend
    #[test]
    fn active_rec_without_baseline_contributes_savings_not_spend() {
        let recs = vec![rec(
            "a1",
            RecommendationKind::StorageTier,
            None,
            999_999,
            RecommendationState::Active,
        )];
        let result = roll_up_savings(&recs, &[]).unwrap();
        assert_eq!(result.estimated_savings_micros, 999_999);
        // No baseline spend => coverage_bps = 0
        assert_eq!(result.coverage_bps, 0);
        assert_eq!(
            *result
                .counts_by_kind
                .get(&RecommendationKind::StorageTier)
                .unwrap(),
            1
        );
    }

    // Extra: error message is non-empty
    #[test]
    fn rollup_error_message_non_empty() {
        let e = RollupError::MissingBaselineReport {
            baseline_report_id: "rpt-99".to_owned(),
        };
        assert!(!e.message().is_empty());
        assert!(e.message().contains("rpt-99"));
    }
}
