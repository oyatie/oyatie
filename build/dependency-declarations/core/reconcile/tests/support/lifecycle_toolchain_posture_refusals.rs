use super::toolchain_currency_support::*;
use dependency_declarations_reconcile::*;

#[test]
fn posture_refuses_stale_or_future_observation() {
    let current = matrix(MatrixFixtureV1::default());
    let observed = matrix(MatrixFixtureV1 {
        nightly_commit: "bff8e12ff",
        nightly_cargo_commit: "e8cb624d5",
        ..MatrixFixtureV1::default()
    });
    let snapshot = snapshot(&observed, [100, 800, 900], 1_000);
    for evaluated_at in [999, 2_001] {
        let failure = ToolchainCurrencyPostureV1::try_evaluate(
            &current,
            &snapshot,
            &policy(),
            LifecycleTimestampV1::from_unix_seconds(evaluated_at),
        )
        .unwrap_err();
        assert_eq!(failure.class(), LifecycleFailureClassV1::StaleFact);
    }
}

#[test]
fn posture_refuses_a_snapshot_older_than_the_current_matrix() {
    let current = matrix(MatrixFixtureV1 {
        stable_minor: 99,
        beta_minor: 100,
        nightly_minor: 101,
        stable_commit: "stable-99",
        beta_commit: "beta-100",
        nightly_commit: "nightly-101",
        nightly_cargo_commit: "nightly-cargo-101",
        ..MatrixFixtureV1::default()
    });
    let older = matrix(MatrixFixtureV1::default());
    let failure = ToolchainCurrencyPostureV1::try_evaluate(
        &current,
        &snapshot(&older, [100, 200, 300], 400),
        &policy(),
        LifecycleTimestampV1::from_unix_seconds(400),
    )
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::UnsupportedVersionRelation
    );
}

#[test]
fn posture_refuses_timestamp_overflow() {
    let current = matrix(MatrixFixtureV1::default());
    let snapshot = snapshot(
        &current,
        [u64::MAX - 10, u64::MAX - 10, u64::MAX - 10],
        u64::MAX - 1,
    );
    let failure = ToolchainCurrencyPostureV1::try_evaluate(
        &current,
        &snapshot,
        &policy(),
        LifecycleTimestampV1::from_unix_seconds(u64::MAX - 1),
    )
    .unwrap_err();

    assert_eq!(failure.class(), LifecycleFailureClassV1::BoundsExceeded);
}
