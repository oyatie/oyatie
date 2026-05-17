//! Cloud FinOps kernel (M03-P03-IP-004 minimum viable kernel;
//! M05-P01-IP-003 public-GA surface).
//!
//! Pure I/O-free model for cost-report periods, savings recommendations,
//! and the admission rule that a recommendation cannot be promoted to
//! "active" without a baseline-period reference + estimated savings.
//! The `finops_public` module adds the tenant-visible summary projection.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod finops_public;
pub use finops_public::{FINOPS_PUBLIC_SCHEMA_VERSION, PublicCostSummary, report_public};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ReportPeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annual,
}

impl ReportPeriod {
    pub fn name(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Quarterly => "quarterly",
            Self::Annual => "annual",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecommendationKind {
    RightsizeInstance,
    ReservedCapacityPurchase,
    SpotCommit,
    StorageTier,
    UnusedResourceCleanup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecommendationState {
    Draft,
    Active,
    Applied,
    Dismissed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostReport {
    // data_class: INTERNAL_ONLY
    pub report_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub period: ReportPeriod, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub period_start_unix_ms: u64, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub period_end_unix_ms: u64, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub total_spend_micros: u128, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recommendation {
    // data_class: INTERNAL_ONLY
    pub recommendation_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub kind: RecommendationKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub baseline_report_id: Option<String>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub estimated_savings_micros: u128, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub state: RecommendationState, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FinopsError {
    EmptyReportId,
    InvalidPeriod,
    NegativeSpend,
    EmptyRecommendationId,
    NoBaselineForActivation,
    ZeroSavingsEstimate,
    InvalidStateTransition {
        from: RecommendationState,
        to: RecommendationState,
    },
}

impl FinopsError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyReportId => "report id is empty".to_owned(),
            Self::InvalidPeriod => "period_end must be after period_start".to_owned(),
            Self::NegativeSpend => "spend cannot be negative".to_owned(),
            Self::EmptyRecommendationId => "recommendation id is empty".to_owned(),
            Self::NoBaselineForActivation => {
                "recommendation cannot activate without baseline_report_id".to_owned()
            }
            Self::ZeroSavingsEstimate => {
                "recommendation cannot activate with zero estimated savings".to_owned()
            }
            Self::InvalidStateTransition { from, to } => {
                format!(
                    "invalid recommendation state transition: {:?} -> {:?}",
                    from, to
                )
            }
        }
    }
}

pub fn validate_report(r: &CostReport) -> Result<(), FinopsError> {
    if r.report_id.is_empty() {
        return Err(FinopsError::EmptyReportId);
    }
    if r.period_end_unix_ms <= r.period_start_unix_ms {
        return Err(FinopsError::InvalidPeriod);
    }
    Ok(())
}

pub fn activate(rec: &mut Recommendation) -> Result<(), FinopsError> {
    if rec.recommendation_id.is_empty() {
        return Err(FinopsError::EmptyRecommendationId);
    }
    if rec.state != RecommendationState::Draft {
        return Err(FinopsError::InvalidStateTransition {
            from: rec.state,
            to: RecommendationState::Active,
        });
    }
    if rec.baseline_report_id.is_none() {
        return Err(FinopsError::NoBaselineForActivation);
    }
    if rec.estimated_savings_micros == 0 {
        return Err(FinopsError::ZeroSavingsEstimate);
    }
    rec.state = RecommendationState::Active;
    Ok(())
}

pub fn apply(rec: &mut Recommendation) -> Result<(), FinopsError> {
    if rec.state != RecommendationState::Active {
        return Err(FinopsError::InvalidStateTransition {
            from: rec.state,
            to: RecommendationState::Applied,
        });
    }
    rec.state = RecommendationState::Applied;
    Ok(())
}

pub fn dismiss(rec: &mut Recommendation) -> Result<(), FinopsError> {
    if matches!(rec.state, RecommendationState::Applied) {
        return Err(FinopsError::InvalidStateTransition {
            from: rec.state,
            to: RecommendationState::Dismissed,
        });
    }
    rec.state = RecommendationState::Dismissed;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(id: &str, baseline: Option<&str>, savings: u128) -> Recommendation {
        Recommendation {
            recommendation_id: id.into(),
            kind: RecommendationKind::RightsizeInstance,
            baseline_report_id: baseline.map(String::from),
            estimated_savings_micros: savings,
            state: RecommendationState::Draft,
        }
    }

    #[test]
    fn period_names_distinct() {
        use std::collections::HashSet;
        let s: HashSet<_> = [
            ReportPeriod::Daily,
            ReportPeriod::Weekly,
            ReportPeriod::Monthly,
            ReportPeriod::Quarterly,
            ReportPeriod::Annual,
        ]
        .iter()
        .map(|p| p.name())
        .collect();
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn validate_report_passes_for_valid() {
        let r = CostReport {
            report_id: "r1".into(),
            period: ReportPeriod::Monthly,
            period_start_unix_ms: 1000,
            period_end_unix_ms: 2000,
            total_spend_micros: 500,
        };
        assert!(validate_report(&r).is_ok());
    }

    #[test]
    fn report_with_empty_id_rejected() {
        let r = CostReport {
            report_id: String::new(),
            period: ReportPeriod::Monthly,
            period_start_unix_ms: 1000,
            period_end_unix_ms: 2000,
            total_spend_micros: 500,
        };
        assert!(matches!(
            validate_report(&r),
            Err(FinopsError::EmptyReportId)
        ));
    }

    #[test]
    fn report_with_inverted_period_rejected() {
        let r = CostReport {
            report_id: "r1".into(),
            period: ReportPeriod::Monthly,
            period_start_unix_ms: 2000,
            period_end_unix_ms: 1000,
            total_spend_micros: 500,
        };
        assert!(matches!(
            validate_report(&r),
            Err(FinopsError::InvalidPeriod)
        ));
    }

    #[test]
    fn activate_valid_recommendation() {
        let mut r = rec("R1", Some("baseline-1"), 1000);
        assert!(activate(&mut r).is_ok());
        assert_eq!(r.state, RecommendationState::Active);
    }

    #[test]
    fn activate_without_baseline_rejected() {
        let mut r = rec("R1", None, 1000);
        assert!(matches!(
            activate(&mut r),
            Err(FinopsError::NoBaselineForActivation)
        ));
    }

    #[test]
    fn activate_with_zero_savings_rejected() {
        let mut r = rec("R1", Some("b"), 0);
        assert!(matches!(
            activate(&mut r),
            Err(FinopsError::ZeroSavingsEstimate)
        ));
    }

    #[test]
    fn activate_already_active_rejected() {
        let mut r = rec("R1", Some("b"), 100);
        r.state = RecommendationState::Active;
        assert!(matches!(
            activate(&mut r),
            Err(FinopsError::InvalidStateTransition { .. })
        ));
    }

    #[test]
    fn apply_requires_active() {
        let mut r = rec("R1", Some("b"), 100);
        assert!(matches!(
            apply(&mut r),
            Err(FinopsError::InvalidStateTransition { .. })
        ));
        activate(&mut r).unwrap();
        assert!(apply(&mut r).is_ok());
        assert_eq!(r.state, RecommendationState::Applied);
    }

    #[test]
    fn dismiss_after_applied_rejected() {
        let mut r = rec("R1", Some("b"), 100);
        activate(&mut r).unwrap();
        apply(&mut r).unwrap();
        assert!(matches!(
            dismiss(&mut r),
            Err(FinopsError::InvalidStateTransition { .. })
        ));
    }

    #[test]
    fn dismiss_draft_succeeds() {
        let mut r = rec("R1", None, 0);
        assert!(dismiss(&mut r).is_ok());
        assert_eq!(r.state, RecommendationState::Dismissed);
    }
}
