use super::dependency_graph::qualified_h2_candidate;
use super::dependency_qualification::{candidate_impact, qualification_matrix, quarantine_policy};
use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

pub(super) fn currency_policy() -> DependencyCurrencyPolicyV1 {
    DependencyCurrencyPolicyV1::new(120, 1_000, 600, digest("dependency-currency-policy"))
}

pub(super) fn currency_decision() -> DependencyCurrencyDecisionEvidenceV1 {
    DependencyCurrencyDecisionEvidenceV1::new(
        digest("currency-authority"),
        digest("currency-decision-schema"),
        digest("currency-decision-receipt"),
    )
}

pub(super) fn currency_assessment(
    candidate: &DependencyCandidateV1,
    now: u64,
) -> DependencyCurrencyAssessmentV1 {
    DependencyCurrencyAssessmentV1::try_evaluate(
        candidate,
        &currency_policy(),
        None,
        LifecycleTimestampV1::from_unix_seconds(now),
    )
    .unwrap()
}

#[test]
fn dependency_currency_reports_exact_lag_and_target_state() {
    let candidate = qualified_h2_candidate();
    let policy = currency_policy();
    let within = DependencyCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &policy,
        None,
        LifecycleTimestampV1::from_unix_seconds(210),
    )
    .unwrap();
    let overdue = DependencyCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &policy,
        None,
        LifecycleTimestampV1::from_unix_seconds(250),
    )
    .unwrap();

    assert_eq!(within.lag_seconds(), 110);
    assert_eq!(
        within.status(),
        DependencyCurrencyStatusV1::WithinTarget {
            due_at: LifecycleTimestampV1::from_unix_seconds(220),
        }
    );
    assert_eq!(overdue.lag_seconds(), 150);
    assert_eq!(
        overdue.status(),
        DependencyCurrencyStatusV1::Overdue {
            due_at: LifecycleTimestampV1::from_unix_seconds(220),
        }
    );
    assert_eq!(
        overdue.exception_state(),
        DependencyCurrencyExceptionStateV1::Absent
    );
    assert_ne!(within.identity_sha256(), overdue.identity_sha256());
}

#[test]
fn currency_exception_is_temporal_and_never_bypasses_qualification_safety() {
    let candidate = qualified_h2_candidate();
    let policy = currency_policy();
    let exception = DependencyCurrencyExceptionV1::try_new(
        &candidate,
        &policy,
        LifecycleTimestampV1::from_unix_seconds(260),
        LifecycleTimestampV1::from_unix_seconds(300),
        currency_decision(),
    )
    .unwrap();

    let not_yet = DependencyCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &policy,
        Some(&exception),
        LifecycleTimestampV1::from_unix_seconds(250),
    )
    .unwrap();
    assert_eq!(
        not_yet.exception_state(),
        DependencyCurrencyExceptionStateV1::NotYetValid
    );
    assert!(matches!(
        not_yet.status(),
        DependencyCurrencyStatusV1::Overdue { .. }
    ));

    let active = DependencyCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &policy,
        Some(&exception),
        LifecycleTimestampV1::from_unix_seconds(270),
    )
    .unwrap();
    assert_eq!(
        active.exception_state(),
        DependencyCurrencyExceptionStateV1::Active
    );
    assert_eq!(
        active.status(),
        DependencyCurrencyStatusV1::OverdueExcepted {
            due_at: LifecycleTimestampV1::from_unix_seconds(220),
            exception_identity_sha256: exception.identity_sha256(),
        }
    );

    let expired = DependencyCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &policy,
        Some(&exception),
        LifecycleTimestampV1::from_unix_seconds(310),
    )
    .unwrap();
    assert_eq!(
        expired.exception_state(),
        DependencyCurrencyExceptionStateV1::Expired
    );
    assert!(matches!(
        expired.status(),
        DependencyCurrencyStatusV1::Overdue { .. }
    ));

    let matrix = qualification_matrix(64);
    let compatibility = DependencyMsrvCompatibilityV1::new(&candidate, &matrix);
    let quarantine = DependencyQuarantineV1::try_evaluate(
        &candidate,
        &quarantine_policy(),
        None,
        LifecycleTimestampV1::from_unix_seconds(270),
    )
    .unwrap();
    let impact = candidate_impact(&candidate, 270);
    let recommendation = DependencyQualificationRecommendationV1::try_new(
        &candidate,
        &impact,
        &compatibility,
        &quarantine,
        &active,
        LifecycleTimestampV1::from_unix_seconds(270),
    )
    .unwrap();

    assert_eq!(
        recommendation.blockers(),
        &[
            DependencyQualificationBlockerV1::PublicationAge,
            DependencyQualificationBlockerV1::MaintainerChangeHold,
        ]
    );
    assert_eq!(recommendation.mode(), None);
    assert_eq!(recommendation.currency(), &active);
}

#[test]
fn currency_and_security_exceptions_keep_independent_authority() {
    let candidate = qualified_h2_candidate();
    let now = LifecycleTimestampV1::from_unix_seconds(250);
    let currency = currency_assessment(&candidate, 250);
    let quarantine_policy = quarantine_policy();
    let security_exception = DependencyEmergencySecurityExceptionV1::try_new(
        &candidate,
        &quarantine_policy,
        digest("RUSTSEC-2026-0258"),
        LifecycleTimestampV1::from_unix_seconds(220),
        LifecycleTimestampV1::from_unix_seconds(300),
        super::dependency_qualification::security_decision(),
    )
    .unwrap();
    let quarantine = DependencyQuarantineV1::try_evaluate(
        &candidate,
        &quarantine_policy,
        Some(&security_exception),
        now,
    )
    .unwrap();
    let compatibility = DependencyMsrvCompatibilityV1::new(&candidate, &qualification_matrix(64));
    let impact = candidate_impact(&candidate, 250);
    let recommendation = DependencyQualificationRecommendationV1::try_new(
        &candidate,
        &impact,
        &compatibility,
        &quarantine,
        &currency,
        now,
    )
    .unwrap();

    assert!(matches!(
        recommendation.currency().status(),
        DependencyCurrencyStatusV1::Overdue { .. }
    ));
    assert_eq!(
        recommendation.mode(),
        Some(DependencyQualificationModeV1::ExpeditedSecurity {
            exception_identity_sha256: security_exception.identity_sha256(),
        })
    );
}
