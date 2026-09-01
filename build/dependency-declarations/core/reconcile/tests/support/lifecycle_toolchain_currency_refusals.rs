use super::lifecycle_support::{digest, nightly_profile_on_host};
use super::toolchain_currency_support::*;
use dependency_declarations_reconcile::*;

fn nightly_candidate() -> (ToolchainCandidateV1, ToolchainMatrixV1) {
    let current = matrix(MatrixFixtureV1::default());
    let proposed = matrix(MatrixFixtureV1 {
        nightly_commit: "bff8e12ff",
        nightly_cargo_commit: "e8cb624d5",
        ..MatrixFixtureV1::default()
    });
    (candidate(current, proposed.clone()), proposed)
}

#[test]
fn stale_or_future_channel_observation_refuses() {
    let (candidate, proposed) = nightly_candidate();
    let snapshot = snapshot(&proposed, [100, 800, 900], 1_000);
    for evaluated_at in [999, 2_001] {
        let failure = ToolchainCurrencyAssessmentV1::try_evaluate(
            &candidate,
            &snapshot,
            &policy(),
            None,
            LifecycleTimestampV1::from_unix_seconds(evaluated_at),
        )
        .unwrap_err();
        assert_eq!(failure.class(), LifecycleFailureClassV1::StaleFact);
    }
}

#[test]
fn channel_snapshot_refuses_future_release_or_mixed_host_cell() {
    let baseline = matrix(MatrixFixtureV1::default());
    let future_head = ToolchainChannelHeadV1::new(
        baseline.stable().material().clone(),
        LifecycleTimestampV1::from_unix_seconds(1_001),
    );
    let failure = ToolchainChannelSnapshotV1::try_new(
        future_head,
        ToolchainChannelHeadV1::new(
            baseline.beta().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(800),
        ),
        ToolchainChannelHeadV1::new(
            baseline.nightly().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(900),
        ),
        LifecycleTimestampV1::from_unix_seconds(1_000),
        ToolchainChannelSnapshotEvidenceV1::new(
            digest("provider"),
            digest("schema"),
            digest("snapshot"),
            digest("receipt"),
        ),
    )
    .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::InvalidFact);

    let mixed_host_profile = nightly_profile_on_host("x86_64-unknown-linux-gnu");
    let mixed_host = ToolchainChannelHeadV1::new(
        mixed_host_profile.material().clone(),
        LifecycleTimestampV1::from_unix_seconds(900),
    );
    let failure = ToolchainChannelSnapshotV1::try_new(
        ToolchainChannelHeadV1::new(
            baseline.stable().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(100),
        ),
        ToolchainChannelHeadV1::new(
            baseline.beta().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(800),
        ),
        mixed_host,
        LifecycleTimestampV1::from_unix_seconds(1_000),
        ToolchainChannelSnapshotEvidenceV1::new(
            digest("provider"),
            digest("schema"),
            digest("snapshot"),
            digest("receipt"),
        ),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ToolchainTargetMismatch
    );
}

#[test]
fn channel_snapshot_refuses_wrong_role_positions() {
    let baseline = matrix(MatrixFixtureV1::default());
    let failure = ToolchainChannelSnapshotV1::try_new(
        ToolchainChannelHeadV1::new(
            baseline.beta().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(200),
        ),
        ToolchainChannelHeadV1::new(
            baseline.stable().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(100),
        ),
        ToolchainChannelHeadV1::new(
            baseline.nightly().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(300),
        ),
        LifecycleTimestampV1::from_unix_seconds(400),
        ToolchainChannelSnapshotEvidenceV1::new(
            digest("provider"),
            digest("schema"),
            digest("snapshot"),
            digest("receipt"),
        ),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ToolchainRoleMismatch
    );
}

#[test]
fn currency_exception_refuses_msrv_unmodified_role_and_excess_duration() {
    let (candidate, _) = nightly_candidate();
    let policy = policy();
    for roles in [
        vec![ToolchainRoleV1::DeclaredMsrvCompatibility],
        vec![ToolchainRoleV1::QualifiedStableExecution],
    ] {
        let failure = ToolchainCurrencyExceptionV1::try_new(
            &candidate,
            &policy,
            roles,
            LifecycleTimestampV1::from_unix_seconds(1_000),
            LifecycleTimestampV1::from_unix_seconds(1_100),
            decision(),
        )
        .unwrap_err();
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::InvalidToolchainCurrencyException
        );
    }

    let failure = ToolchainCurrencyExceptionV1::try_new(
        &candidate,
        &policy,
        vec![ToolchainRoleV1::NightlyShadow],
        LifecycleTimestampV1::from_unix_seconds(1_000),
        LifecycleTimestampV1::from_unix_seconds(1_601),
        decision(),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidToolchainCurrencyException
    );

    for (roles, authorized_at, expires_at) in [
        (
            vec![
                ToolchainRoleV1::NightlyShadow,
                ToolchainRoleV1::NightlyShadow,
            ],
            1_000,
            1_100,
        ),
        (vec![ToolchainRoleV1::NightlyShadow], 1_100, 1_000),
    ] {
        let failure = ToolchainCurrencyExceptionV1::try_new(
            &candidate,
            &policy,
            roles,
            LifecycleTimestampV1::from_unix_seconds(authorized_at),
            LifecycleTimestampV1::from_unix_seconds(expires_at),
            decision(),
        )
        .unwrap_err();
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::InvalidToolchainCurrencyException
        );
    }
}

#[test]
fn currency_assessment_refuses_cross_candidate_or_policy_exception() {
    let (candidate_value, proposed) = nightly_candidate();
    let other_candidate = candidate(
        matrix(MatrixFixtureV1::default()),
        matrix(MatrixFixtureV1 {
            nightly_commit: "other-nightly",
            nightly_cargo_commit: "other-cargo",
            ..MatrixFixtureV1::default()
        }),
    );
    let policy = policy();
    let other_policy =
        ToolchainCurrencyPolicyV1::new(121, 60, 30, 1_000, 600, digest("other-policy"));
    for exception in [
        ToolchainCurrencyExceptionV1::try_new(
            &other_candidate,
            &policy,
            vec![ToolchainRoleV1::NightlyShadow],
            LifecycleTimestampV1::from_unix_seconds(1_000),
            LifecycleTimestampV1::from_unix_seconds(1_500),
            decision(),
        )
        .unwrap(),
        ToolchainCurrencyExceptionV1::try_new(
            &candidate_value,
            &other_policy,
            vec![ToolchainRoleV1::NightlyShadow],
            LifecycleTimestampV1::from_unix_seconds(1_000),
            LifecycleTimestampV1::from_unix_seconds(1_500),
            decision(),
        )
        .unwrap(),
    ] {
        let failure = ToolchainCurrencyAssessmentV1::try_evaluate(
            &candidate_value,
            &snapshot(&proposed, [100, 800, 900], 1_000),
            &policy,
            Some(&exception),
            LifecycleTimestampV1::from_unix_seconds(1_100),
        )
        .unwrap_err();
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::ToolchainAnalysisMismatch
        );
    }
}

#[test]
fn timestamp_overflow_refuses_currency_evaluation() {
    let (candidate, proposed) = nightly_candidate();
    let snapshot = snapshot(
        &proposed,
        [u64::MAX - 10, u64::MAX - 10, u64::MAX - 10],
        u64::MAX - 1,
    );
    let failure = ToolchainCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &snapshot,
        &policy(),
        None,
        LifecycleTimestampV1::from_unix_seconds(u64::MAX - 1),
    )
    .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::BoundsExceeded);
}
