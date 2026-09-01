use super::lifecycle_support::digest;
use super::toolchain_safety_support::*;
use dependency_declarations_reconcile::*;

#[test]
fn safety_posture_refuses_unsafe_completeness_claims() {
    let (current, _) = recovery_matrices();
    let profile = current.stable();
    let cases = [
        safety_envelope(
            profile,
            vec![FactEvidenceClassV1::Inferred],
            FactCertaintyV1::Exact,
            FactCoverageV1::CompleteForScope {
                scope_sha256: digest("scope"),
                exclusions_sha256: digest("exclusions"),
            },
            100,
            1_000,
        ),
        safety_envelope(
            profile,
            vec![FactEvidenceClassV1::Observed],
            FactCertaintyV1::Speculative,
            FactCoverageV1::CompleteForScope {
                scope_sha256: digest("scope"),
                exclusions_sha256: digest("exclusions"),
            },
            100,
            1_000,
        ),
    ];

    for envelope in cases {
        let failure = ToolchainSafetyPostureV1::try_evaluate(
            profile,
            Vec::new(),
            envelope,
            LifecycleTimestampV1::from_unix_seconds(500),
        )
        .unwrap_err();
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::UnsupportedFactEvidence
        );
    }

    for coverage in [
        FactCoverageV1::Partial {
            scope_sha256: digest("scope"),
            evidence_sha256: digest("partial"),
        },
        FactCoverageV1::Unknown {
            scope_sha256: digest("scope"),
            reason_sha256: digest("unknown"),
        },
        FactCoverageV1::Excluded {
            scope_sha256: digest("scope"),
            exclusion_sha256: digest("excluded"),
        },
    ] {
        let failure = ToolchainSafetyPostureV1::try_evaluate(
            profile,
            Vec::new(),
            safety_envelope(
                profile,
                vec![FactEvidenceClassV1::Observed],
                FactCertaintyV1::Exact,
                coverage,
                100,
                1_000,
            ),
            LifecycleTimestampV1::from_unix_seconds(500),
        )
        .unwrap_err();
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::IncompleteFactCoverage
        );
    }
}

#[test]
fn safety_posture_refuses_stale_or_future_evidence() {
    let (current, _) = recovery_matrices();
    for evaluated_at in [99, 1_001] {
        let failure = ToolchainSafetyPostureV1::try_evaluate(
            current.stable(),
            Vec::new(),
            complete_safety_envelope(current.stable()),
            LifecycleTimestampV1::from_unix_seconds(evaluated_at),
        )
        .unwrap_err();
        assert_eq!(failure.class(), LifecycleFailureClassV1::StaleFact);
    }
}

#[test]
fn safety_posture_refuses_cross_profile_evidence_and_findings() {
    let (current, proposed) = recovery_matrices();
    let envelope_mismatch = ToolchainSafetyPostureV1::try_evaluate(
        current.stable(),
        Vec::new(),
        complete_safety_envelope(proposed.stable()),
        LifecycleTimestampV1::from_unix_seconds(500),
    )
    .unwrap_err();
    assert_eq!(
        envelope_mismatch.class(),
        LifecycleFailureClassV1::ToolchainAnalysisMismatch
    );

    let finding_mismatch = ToolchainSafetyPostureV1::try_evaluate(
        current.stable(),
        vec![blocking_defect(proposed.stable(), "rust-lang/rust#161441")],
        complete_safety_envelope(current.stable()),
        LifecycleTimestampV1::from_unix_seconds(500),
    )
    .unwrap_err();
    assert_eq!(
        finding_mismatch.class(),
        LifecycleFailureClassV1::ToolchainAnalysisMismatch
    );
}

#[test]
fn one_advisory_cannot_be_counted_twice_for_one_profile() {
    let (current, _) = recovery_matrices();
    let defect = blocking_defect(current.stable(), "rust-lang/rust#161441");
    let failure = ToolchainSafetyPostureV1::try_evaluate(
        current.stable(),
        vec![defect.clone(), defect],
        complete_safety_envelope(current.stable()),
        LifecycleTimestampV1::from_unix_seconds(500),
    )
    .unwrap_err();

    assert_eq!(failure.class(), LifecycleFailureClassV1::DuplicateIdentity);
}

#[test]
fn withdrawn_normalized_advisory_cannot_remain_a_blocker() {
    let (current, _) = recovery_matrices();
    let advisory = withdrawn_upstream_advisory("rust-lang/rust#161441");
    let failure = ToolchainBlockingDefectV1::try_new(
        current.stable(),
        &advisory,
        digest("applicability"),
        ToolchainDefectDecisionEvidenceV1::new(
            digest("authority"),
            digest("schema"),
            digest("decision"),
        ),
    )
    .unwrap_err();

    assert_eq!(failure.class(), LifecycleFailureClassV1::InvalidFact);
}

#[test]
fn safety_posture_refuses_an_excessive_blocker_set() {
    let (current, _) = recovery_matrices();
    let blockers = (0..=LifecycleBoundsV1::MAX_TOOLCHAIN_BLOCKING_DEFECTS)
        .map(|index| blocking_defect(current.stable(), &format!("rust-lang/rust#{index}")))
        .collect();
    let failure = ToolchainSafetyPostureV1::try_evaluate(
        current.stable(),
        blockers,
        complete_safety_envelope(current.stable()),
        LifecycleTimestampV1::from_unix_seconds(500),
    )
    .unwrap_err();

    assert_eq!(failure.class(), LifecycleFailureClassV1::BoundsExceeded);
}
