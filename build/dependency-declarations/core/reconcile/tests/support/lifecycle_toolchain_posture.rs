use super::lifecycle_support::digest;
use super::toolchain_currency_support::*;
use dependency_declarations_reconcile::*;

fn evaluate(
    current: &ToolchainMatrixV1,
    observed: &ToolchainMatrixV1,
    published_at: [u64; 3],
    observed_at: u64,
    evaluated_at: u64,
) -> ToolchainCurrencyPostureV1 {
    ToolchainCurrencyPostureV1::try_evaluate(
        current,
        &snapshot(observed, published_at, observed_at),
        &policy(),
        LifecycleTimestampV1::from_unix_seconds(evaluated_at),
    )
    .unwrap()
}

#[test]
fn posture_assesses_every_execution_role_and_excludes_msrv() {
    let current = matrix(MatrixFixtureV1::default());
    let observed = matrix(MatrixFixtureV1 {
        nightly_commit: "bff8e12ff",
        nightly_cargo_commit: "e8cb624d5",
        ..MatrixFixtureV1::default()
    });
    let posture = evaluate(&current, &observed, [100, 800, 900], 900, 920);

    let roles = posture.execution_roles();
    assert_eq!(roles.len(), 3);
    assert_eq!(roles[0].role(), ToolchainRoleV1::QualifiedStableExecution);
    assert_eq!(roles[1].role(), ToolchainRoleV1::BetaShadow);
    assert_eq!(roles[2].role(), ToolchainRoleV1::NightlyShadow);
    assert!(
        roles
            .iter()
            .all(|role| role.role() != ToolchainRoleV1::DeclaredMsrvCompatibility)
    );
    assert_eq!(
        roles[0].status(),
        ToolchainCurrencyPostureStatusV1::OnObservedHead
    );
    assert_eq!(
        roles[1].status(),
        ToolchainCurrencyPostureStatusV1::OnObservedHead
    );
    assert_eq!(
        roles[2].status(),
        ToolchainCurrencyPostureStatusV1::DiffersFromObservedHeadWithinTarget
    );
}

#[test]
fn same_version_nightly_commit_change_is_currency_drift() {
    let current = matrix(MatrixFixtureV1::default());
    let observed = matrix(MatrixFixtureV1 {
        nightly_commit: "bff8e12ff",
        nightly_cargo_commit: "e8cb624d5",
        ..MatrixFixtureV1::default()
    });
    assert_eq!(current.nightly().version(), observed.nightly().version());

    let posture = evaluate(&current, &observed, [100, 800, 900], 900, 931);
    let nightly = &posture.execution_roles()[2];
    assert_ne!(
        nightly.current_material_identity_sha256(),
        nightly.observed_material_identity_sha256()
    );
    assert_eq!(nightly.head_age_seconds(), 31);
    assert_eq!(nightly.due_at().unix_seconds(), 930);
    assert_eq!(
        nightly.status(),
        ToolchainCurrencyPostureStatusV1::DiffersFromObservedHeadOverdue
    );
}

#[test]
fn qualification_refresh_does_not_create_material_currency_drift() {
    let fixture = MatrixFixtureV1::default();
    let current = matrix(fixture);
    let differently_qualified = matrix_with_stable_qualification(
        fixture,
        ToolchainQualificationV1::Production {
            qualification_receipt_sha256: digest("refreshed-stable-qualification"),
        },
    );
    assert_ne!(
        current.stable().identity_sha256(),
        differently_qualified.stable().identity_sha256()
    );
    assert_eq!(
        current.stable().material_identity_sha256(),
        differently_qualified.stable().material_identity_sha256()
    );

    let posture = evaluate(&current, &differently_qualified, [100, 800, 900], 900, 920);
    assert!(
        posture
            .execution_roles()
            .iter()
            .all(|role| { role.status() == ToolchainCurrencyPostureStatusV1::OnObservedHead })
    );
}

#[test]
fn posture_identity_binds_current_snapshot_policy_and_time() {
    let current = matrix(MatrixFixtureV1::default());
    let observed = matrix(MatrixFixtureV1 {
        nightly_commit: "bff8e12ff",
        nightly_cargo_commit: "e8cb624d5",
        ..MatrixFixtureV1::default()
    });
    let snapshot = snapshot(&observed, [100, 800, 900], 900);
    let base = ToolchainCurrencyPostureV1::try_evaluate(
        &current,
        &snapshot,
        &policy(),
        LifecycleTimestampV1::from_unix_seconds(920),
    )
    .unwrap();
    let later = ToolchainCurrencyPostureV1::try_evaluate(
        &current,
        &snapshot,
        &policy(),
        LifecycleTimestampV1::from_unix_seconds(921),
    )
    .unwrap();
    let changed_policy =
        ToolchainCurrencyPolicyV1::new(121, 60, 30, 1_000, 600, digest("changed-policy"));
    let changed = ToolchainCurrencyPostureV1::try_evaluate(
        &current,
        &snapshot,
        &changed_policy,
        LifecycleTimestampV1::from_unix_seconds(920),
    )
    .unwrap();
    let changed_current = matrix_with_stable_qualification(
        MatrixFixtureV1::default(),
        ToolchainQualificationV1::Production {
            qualification_receipt_sha256: digest("other-current-qualification"),
        },
    );
    let changed_current = ToolchainCurrencyPostureV1::try_evaluate(
        &changed_current,
        &snapshot,
        &policy(),
        LifecycleTimestampV1::from_unix_seconds(920),
    )
    .unwrap();

    assert_ne!(base.identity_sha256(), later.identity_sha256());
    assert_ne!(base.identity_sha256(), changed.identity_sha256());
    assert_ne!(base.identity_sha256(), changed_current.identity_sha256());
    assert_eq!(base.current().identity_sha256(), current.identity_sha256());
    assert_eq!(
        base.snapshot().identity_sha256(),
        snapshot.identity_sha256()
    );
}
