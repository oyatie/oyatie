use super::dependency_currency::qualification_recommendation;
use super::dependency_graph::qualified_candidate;
use super::dependency_qualification::*;
use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

#[test]
fn absent_or_unknown_msrv_remains_an_explicit_qualification_blocker() {
    let cases = [
        DependencyMsrvDeclarationV1::Absent {
            evidence_sha256: digest("absent-msrv"),
        },
        DependencyMsrvDeclarationV1::Unknown {
            evidence_sha256: digest("unknown-msrv"),
        },
    ];
    let matrix = qualification_matrix(64);
    let policy = quarantine_policy();
    let now = LifecycleTimestampV1::from_unix_seconds(350);

    for proposed_msrv in cases {
        let candidate = candidate_with_proposed_msrv(proposed_msrv, Vec::new());
        let compatibility = DependencyMsrvCompatibilityV1::new(&candidate, &matrix);
        let quarantine =
            DependencyQuarantineV1::try_evaluate(&candidate, &policy, None, now).unwrap();
        let impact = candidate_impact(&candidate, &matrix, 350);
        let recommendation =
            qualification_recommendation(&candidate, &impact, &compatibility, &quarantine, now)
                .unwrap();
        assert_eq!(
            recommendation.blockers(),
            &[DependencyQualificationBlockerV1::MsrvEvidence]
        );
        assert_eq!(recommendation.mode(), None);
    }
}

#[test]
fn security_exception_requires_a_current_advisory_removed_by_the_candidate() {
    let candidate = candidate_with_proposed_msrv(
        DependencyMsrvDeclarationV1::Declared {
            version: RustVersionV1::try_new(1, 64, 0).unwrap(),
            evidence_sha256: digest("proposed-msrv"),
        },
        Vec::new(),
    );
    let policy = quarantine_policy();
    let invalid = DependencyEmergencySecurityExceptionV1::try_new(
        &candidate,
        &policy,
        digest("unrelated-advisory"),
        LifecycleTimestampV1::from_unix_seconds(220),
        LifecycleTimestampV1::from_unix_seconds(300),
        security_decision(),
    )
    .unwrap_err();
    assert_eq!(
        invalid.class(),
        LifecycleFailureClassV1::InvalidSecurityException
    );

    let retained = candidate_with_proposed_msrv(
        DependencyMsrvDeclarationV1::Declared {
            version: RustVersionV1::try_new(1, 64, 0).unwrap(),
            evidence_sha256: digest("proposed-msrv"),
        },
        vec![digest("RUSTSEC-2026-0258")],
    );
    let invalid = DependencyEmergencySecurityExceptionV1::try_new(
        &retained,
        &policy,
        digest("RUSTSEC-2026-0258"),
        LifecycleTimestampV1::from_unix_seconds(220),
        LifecycleTimestampV1::from_unix_seconds(300),
        security_decision(),
    )
    .unwrap_err();
    assert_eq!(
        invalid.class(),
        LifecycleFailureClassV1::InvalidSecurityException
    );

    let invalid = DependencyEmergencySecurityExceptionV1::try_new(
        &candidate,
        &policy,
        digest("RUSTSEC-2026-0258"),
        LifecycleTimestampV1::from_unix_seconds(301),
        LifecycleTimestampV1::from_unix_seconds(300),
        security_decision(),
    )
    .unwrap_err();
    assert_eq!(
        invalid.class(),
        LifecycleFailureClassV1::InvalidSecurityException
    );
}

#[test]
fn security_exception_identity_binds_authority_schema_and_receipt() {
    let candidate = super::dependency_graph::qualified_h2_candidate();
    let policy = quarantine_policy();
    let exception_for = |decision| {
        DependencyEmergencySecurityExceptionV1::try_new(
            &candidate,
            &policy,
            digest("RUSTSEC-2026-0258"),
            LifecycleTimestampV1::from_unix_seconds(220),
            LifecycleTimestampV1::from_unix_seconds(300),
            decision,
        )
        .unwrap()
    };
    let baseline_decision = security_decision();
    let baseline = exception_for(baseline_decision);

    assert_eq!(baseline.security_decision(), baseline_decision);
    for changed in [
        DependencySecurityDecisionEvidenceV1::new(
            digest("other-security-authority"),
            digest("security-decision-schema"),
            digest("security-decision-receipt"),
        ),
        DependencySecurityDecisionEvidenceV1::new(
            digest("security-authority"),
            digest("other-security-decision-schema"),
            digest("security-decision-receipt"),
        ),
        DependencySecurityDecisionEvidenceV1::new(
            digest("security-authority"),
            digest("security-decision-schema"),
            digest("other-security-decision-receipt"),
        ),
    ] {
        assert_ne!(
            baseline.identity_sha256(),
            exception_for(changed).identity_sha256()
        );
    }
}

#[test]
fn exception_reuse_and_cross_candidate_analysis_are_refused() {
    let h2 = super::dependency_graph::qualified_h2_candidate();
    let tokio = qualified_candidate("tokio", "1.47.0", "1.48.0");
    let policy = quarantine_policy();
    let other_policy = DependencyQuarantinePolicyV1::new(181, 120, 1_000, digest("other-policy"));
    let exception = DependencyEmergencySecurityExceptionV1::try_new(
        &h2,
        &policy,
        digest("RUSTSEC-2026-0258"),
        LifecycleTimestampV1::from_unix_seconds(220),
        LifecycleTimestampV1::from_unix_seconds(300),
        security_decision(),
    )
    .unwrap();
    let now = LifecycleTimestampV1::from_unix_seconds(250);

    for failure in [
        DependencyQuarantineV1::try_evaluate(&tokio, &policy, Some(&exception), now).unwrap_err(),
        DependencyQuarantineV1::try_evaluate(&h2, &other_policy, Some(&exception), now)
            .unwrap_err(),
    ] {
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::DependencyAnalysisMismatch
        );
    }

    let future = DependencyQuarantineV1::try_evaluate(
        &h2,
        &policy,
        None,
        LifecycleTimestampV1::from_unix_seconds(199),
    )
    .unwrap_err();
    assert_eq!(future.class(), LifecycleFailureClassV1::StaleFact);

    let matrix = qualification_matrix(64);
    let compatibility = DependencyMsrvCompatibilityV1::new(&tokio, &matrix);
    let quarantine = DependencyQuarantineV1::try_evaluate(&tokio, &policy, None, now).unwrap();
    let h2_impact = candidate_impact(&h2, &matrix, 250);
    let mismatch =
        qualification_recommendation(&tokio, &h2_impact, &compatibility, &quarantine, now)
            .unwrap_err();
    assert_eq!(
        mismatch.class(),
        LifecycleFailureClassV1::DependencyAnalysisMismatch
    );
}

#[test]
fn inactive_security_exception_never_bypasses_quarantine() {
    let candidate = super::dependency_graph::qualified_h2_candidate();
    let policy = quarantine_policy();
    let exception = DependencyEmergencySecurityExceptionV1::try_new(
        &candidate,
        &policy,
        digest("RUSTSEC-2026-0258"),
        LifecycleTimestampV1::from_unix_seconds(220),
        LifecycleTimestampV1::from_unix_seconds(300),
        security_decision(),
    )
    .unwrap();

    let not_yet = DependencyQuarantineV1::try_evaluate(
        &candidate,
        &policy,
        Some(&exception),
        LifecycleTimestampV1::from_unix_seconds(210),
    )
    .unwrap();
    assert_eq!(
        not_yet.security_exception_state(),
        DependencySecurityExceptionStateV1::NotYetValid
    );
    assert!(matches!(
        not_yet.publication_age(),
        DependencyQuarantineGateV1::Held { .. }
    ));

    let expired = DependencyQuarantineV1::try_evaluate(
        &candidate,
        &policy,
        Some(&exception),
        LifecycleTimestampV1::from_unix_seconds(310),
    )
    .unwrap();
    assert_eq!(
        expired.security_exception_state(),
        DependencySecurityExceptionStateV1::Expired
    );
    assert!(matches!(
        expired.maintainer_change(),
        DependencyQuarantineGateV1::Held { .. }
    ));
}

#[test]
fn recommendation_rechecks_impact_freshness_at_evaluation_time() {
    let candidate = super::dependency_graph::qualified_h2_candidate();
    let policy = quarantine_policy();
    let matrix = qualification_matrix(64);
    let compatibility = DependencyMsrvCompatibilityV1::new(&candidate, &matrix);
    let impact = candidate_impact(&candidate, &matrix, 350);
    let now = LifecycleTimestampV1::from_unix_seconds(1_001);
    let quarantine = DependencyQuarantineV1::try_evaluate(&candidate, &policy, None, now).unwrap();
    let failure =
        qualification_recommendation(&candidate, &impact, &compatibility, &quarantine, now)
            .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::StaleFact);
}

#[test]
fn stale_registry_observation_cannot_age_into_quarantine_eligibility() {
    let candidate = super::dependency_graph::qualified_h2_candidate();
    let failure = DependencyQuarantineV1::try_evaluate(
        &candidate,
        &quarantine_policy(),
        None,
        LifecycleTimestampV1::from_unix_seconds(1_201),
    )
    .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::StaleFact);
}

#[test]
fn quarantine_refuses_timestamp_overflow() {
    let candidate = super::dependency_graph::qualified_h2_candidate();
    let now = LifecycleTimestampV1::from_unix_seconds(250);

    for (minimum_age, maintainer_hold) in [(u64::MAX, 0), (0, u64::MAX)] {
        let policy = DependencyQuarantinePolicyV1::new(
            minimum_age,
            maintainer_hold,
            1_000,
            digest("overflow-policy"),
        );
        let failure =
            DependencyQuarantineV1::try_evaluate(&candidate, &policy, None, now).unwrap_err();
        assert_eq!(failure.class(), LifecycleFailureClassV1::BoundsExceeded);
    }
}
