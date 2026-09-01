use super::dependency_currency::{
    currency_decision, currency_policy, qualification_recommendation,
};
use super::dependency_graph::{qualified_candidate, qualified_h2_candidate};
use super::dependency_qualification::{candidate_impact, qualification_matrix, quarantine_policy};
use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

#[test]
fn recommendation_refuses_msrv_evidence_from_an_alternate_toolchain_matrix() {
    let candidate = qualified_h2_candidate();
    let impact_matrix = qualification_matrix(64);
    let alternate_matrix = qualification_matrix(65);
    let alternate_compatibility = DependencyMsrvCompatibilityV1::new(&candidate, &alternate_matrix);
    let now = LifecycleTimestampV1::from_unix_seconds(350);
    let quarantine =
        DependencyQuarantineV1::try_evaluate(&candidate, &quarantine_policy(), None, now).unwrap();
    let impact = candidate_impact(&candidate, &impact_matrix, 350);

    assert!(matches!(
        alternate_compatibility.proposed(),
        DependencyMsrvRelationV1::WithinDeclaredFloor { .. }
    ));
    assert_ne!(
        alternate_compatibility.toolchain_matrix_identity_sha256(),
        impact.fact_envelope().temporal().scope().toolchain_sha256()
    );
    let failure = qualification_recommendation(
        &candidate,
        &impact,
        &alternate_compatibility,
        &quarantine,
        now,
    )
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::DependencyAnalysisMismatch
    );
}

#[test]
fn currency_policy_identity_binds_every_control() {
    let baseline = currency_policy();
    for changed in [
        DependencyCurrencyPolicyV1::new(121, 1_000, 600, digest("dependency-currency-policy")),
        DependencyCurrencyPolicyV1::new(120, 1_001, 600, digest("dependency-currency-policy")),
        DependencyCurrencyPolicyV1::new(120, 1_000, 601, digest("dependency-currency-policy")),
        DependencyCurrencyPolicyV1::new(120, 1_000, 600, digest("other-policy-receipt")),
    ] {
        assert_ne!(baseline.identity_sha256(), changed.identity_sha256());
    }
}

#[test]
fn currency_exception_refuses_an_inverted_or_overlong_window() {
    let candidate = qualified_h2_candidate();
    let bounded_policy =
        DependencyCurrencyPolicyV1::new(120, 1_000, 30, digest("bounded-exception-policy"));
    for failure in [
        DependencyCurrencyExceptionV1::try_new(
            &candidate,
            &bounded_policy,
            LifecycleTimestampV1::from_unix_seconds(301),
            LifecycleTimestampV1::from_unix_seconds(300),
            currency_decision(),
        )
        .unwrap_err(),
        DependencyCurrencyExceptionV1::try_new(
            &candidate,
            &bounded_policy,
            LifecycleTimestampV1::from_unix_seconds(220),
            LifecycleTimestampV1::from_unix_seconds(251),
            currency_decision(),
        )
        .unwrap_err(),
    ] {
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::InvalidCurrencyException
        );
    }
}

#[test]
fn currency_exception_is_candidate_and_policy_scoped() {
    let h2 = qualified_h2_candidate();
    let tokio = qualified_candidate("tokio", "1.47.0", "1.48.0");
    let policy = currency_policy();
    let other_policy =
        DependencyCurrencyPolicyV1::new(121, 1_000, 600, digest("other-currency-policy"));
    let exception = DependencyCurrencyExceptionV1::try_new(
        &h2,
        &policy,
        LifecycleTimestampV1::from_unix_seconds(220),
        LifecycleTimestampV1::from_unix_seconds(300),
        currency_decision(),
    )
    .unwrap();
    let now = LifecycleTimestampV1::from_unix_seconds(250);

    for failure in [
        DependencyCurrencyAssessmentV1::try_evaluate(&tokio, &policy, Some(&exception), now)
            .unwrap_err(),
        DependencyCurrencyAssessmentV1::try_evaluate(&h2, &other_policy, Some(&exception), now)
            .unwrap_err(),
    ] {
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::DependencyAnalysisMismatch
        );
    }
}

#[test]
fn currency_exception_identity_binds_authority_schema_and_receipt() {
    let candidate = qualified_h2_candidate();
    let policy = currency_policy();
    let exception_for = |decision| {
        DependencyCurrencyExceptionV1::try_new(
            &candidate,
            &policy,
            LifecycleTimestampV1::from_unix_seconds(220),
            LifecycleTimestampV1::from_unix_seconds(300),
            decision,
        )
        .unwrap()
    };
    let baseline_decision = currency_decision();
    let baseline = exception_for(baseline_decision);

    assert_eq!(baseline.currency_decision(), baseline_decision);
    for changed in [
        DependencyCurrencyDecisionEvidenceV1::new(
            digest("other-currency-authority"),
            digest("currency-decision-schema"),
            digest("currency-decision-receipt"),
        ),
        DependencyCurrencyDecisionEvidenceV1::new(
            digest("currency-authority"),
            digest("other-currency-decision-schema"),
            digest("currency-decision-receipt"),
        ),
        DependencyCurrencyDecisionEvidenceV1::new(
            digest("currency-authority"),
            digest("currency-decision-schema"),
            digest("other-currency-decision-receipt"),
        ),
    ] {
        assert_ne!(
            baseline.identity_sha256(),
            exception_for(changed).identity_sha256()
        );
    }
}

#[test]
fn currency_refuses_stale_observations_and_timestamp_overflow() {
    let candidate = qualified_h2_candidate();
    let stale = DependencyCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &currency_policy(),
        None,
        LifecycleTimestampV1::from_unix_seconds(1_201),
    )
    .unwrap_err();
    assert_eq!(stale.class(), LifecycleFailureClassV1::StaleFact);

    for policy in [
        DependencyCurrencyPolicyV1::new(u64::MAX, 1_000, 600, digest("lag-overflow-policy")),
        DependencyCurrencyPolicyV1::new(120, u64::MAX, 600, digest("freshness-overflow-policy")),
    ] {
        let failure = DependencyCurrencyAssessmentV1::try_evaluate(
            &candidate,
            &policy,
            None,
            LifecycleTimestampV1::from_unix_seconds(250),
        )
        .unwrap_err();
        assert_eq!(failure.class(), LifecycleFailureClassV1::BoundsExceeded);
    }
}

#[test]
fn recommendation_refuses_mismatched_currency_evidence() {
    let h2 = qualified_h2_candidate();
    let tokio = qualified_candidate("tokio", "1.47.0", "1.48.0");
    let now = LifecycleTimestampV1::from_unix_seconds(250);
    let matrix = qualification_matrix(64);
    let compatibility = DependencyMsrvCompatibilityV1::new(&tokio, &matrix);
    let quarantine =
        DependencyQuarantineV1::try_evaluate(&tokio, &quarantine_policy(), None, now).unwrap();
    let impact = candidate_impact(&tokio, &matrix, 250);
    let h2_currency =
        DependencyCurrencyAssessmentV1::try_evaluate(&h2, &currency_policy(), None, now).unwrap();
    let wrong_time_currency = DependencyCurrencyAssessmentV1::try_evaluate(
        &tokio,
        &currency_policy(),
        None,
        LifecycleTimestampV1::from_unix_seconds(249),
    )
    .unwrap();

    for currency in [&h2_currency, &wrong_time_currency] {
        let failure = DependencyQualificationRecommendationV1::try_new(
            &tokio,
            &impact,
            &compatibility,
            &quarantine,
            currency,
            now,
        )
        .unwrap_err();
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::DependencyAnalysisMismatch
        );
    }
}
