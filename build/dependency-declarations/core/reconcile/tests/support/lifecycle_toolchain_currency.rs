use super::lifecycle_support::{digest, profile};
use super::toolchain_currency_support::*;
use dependency_declarations_reconcile::*;

#[test]
fn channel_heads_ignore_local_qualification_but_bind_exact_tool_material() {
    let fixture = MatrixFixtureV1::default();
    let first = matrix_with_stable_qualification(
        fixture,
        ToolchainQualificationV1::Production {
            qualification_receipt_sha256: digest("qualification-one"),
        },
    );
    let material = first.stable().material().clone();
    let second = ToolchainProfileV1::try_from_material(
        material.clone(),
        ToolchainQualificationV1::Production {
            qualification_receipt_sha256: digest("qualification-two"),
        },
    )
    .unwrap();

    assert_ne!(first.stable().identity_sha256(), second.identity_sha256());
    assert_eq!(
        first.stable().material_identity_sha256(),
        second.material_identity_sha256()
    );
    let head = ToolchainChannelHeadV1::new(material, LifecycleTimestampV1::from_unix_seconds(100));
    assert_eq!(
        head.material_identity_sha256(),
        second.material_identity_sha256()
    );
    let snapshot = snapshot(&first, [100, 200, 300], 400);
    assert_eq!(
        snapshot.stable().material_identity_sha256(),
        second.material_identity_sha256()
    );
    let cargo_drift = profile(
        ToolchainRoleV1::QualifiedStableExecution,
        version(98),
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "88d9e12ae",
        "different-cargo",
    );
    assert_ne!(
        first.stable().material_identity_sha256(),
        cargo_drift.material_identity_sha256()
    );
}

#[test]
fn same_version_nightly_refresh_matches_the_exact_observed_head() {
    let current = matrix(MatrixFixtureV1::default());
    let proposed = matrix(MatrixFixtureV1 {
        nightly_commit: "bff8e12ff",
        nightly_cargo_commit: "e8cb624d5",
        ..MatrixFixtureV1::default()
    });
    let candidate = candidate(current, proposed.clone());
    let snapshot = snapshot(&proposed, [100, 800, 900], 910);
    let assessment = ToolchainCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &snapshot,
        &policy(),
        None,
        LifecycleTimestampV1::from_unix_seconds(920),
    )
    .unwrap();

    assert_eq!(candidate.changed_roles(), &[ToolchainRoleV1::NightlyShadow]);
    assert_eq!(candidate.msrv_effect(), &ToolchainMsrvEffectV1::Unchanged);
    let roles = assessment.changed_execution_roles();
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].role(), ToolchainRoleV1::NightlyShadow);
    assert_eq!(roles[0].lag_seconds(), 20);
    assert_eq!(roles[0].due_at().unix_seconds(), 930);
    assert_eq!(
        roles[0].status(),
        ToolchainCurrencyRoleStatusV1::CandidateMatchesObservedHeadWithinTarget
    );
}

#[test]
fn stable_adoption_lag_never_moves_the_declared_msrv() {
    let current = matrix(MatrixFixtureV1 {
        beta_minor: 100,
        nightly_minor: 101,
        beta_commit: "beta-1.100",
        nightly_commit: "nightly-1.101",
        ..MatrixFixtureV1::default()
    });
    let proposed = matrix(MatrixFixtureV1 {
        stable_minor: 99,
        beta_minor: 100,
        nightly_minor: 101,
        stable_commit: "stable-1.99",
        beta_commit: "beta-1.100",
        nightly_commit: "nightly-1.101",
        ..MatrixFixtureV1::default()
    });
    let candidate = candidate(current, proposed.clone());
    let assessment = ToolchainCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &snapshot(&proposed, [100, 800, 900], 1_000),
        &policy(),
        None,
        LifecycleTimestampV1::from_unix_seconds(1_000),
    )
    .unwrap();

    assert_eq!(
        candidate.changed_roles(),
        &[ToolchainRoleV1::QualifiedStableExecution]
    );
    assert_eq!(candidate.msrv_effect(), &ToolchainMsrvEffectV1::Unchanged);
    assert_eq!(assessment.changed_execution_roles().len(), 1);
    assert_eq!(
        assessment.changed_execution_roles()[0].status(),
        ToolchainCurrencyRoleStatusV1::CandidateMatchesObservedHeadOverdue
    );
}

#[test]
fn qualification_only_refresh_is_already_on_the_observed_head() {
    let fixture = MatrixFixtureV1::default();
    let current = matrix_with_stable_qualification(
        fixture,
        ToolchainQualificationV1::Production {
            qualification_receipt_sha256: digest("old-qualification"),
        },
    );
    let proposed = matrix_with_stable_qualification(
        fixture,
        ToolchainQualificationV1::Production {
            qualification_receipt_sha256: digest("new-qualification"),
        },
    );
    let candidate = candidate(current, proposed.clone());
    let assessment = ToolchainCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &snapshot(&proposed, [100, 200, 300], 1_000),
        &policy(),
        None,
        LifecycleTimestampV1::from_unix_seconds(1_000),
    )
    .unwrap();

    assert_eq!(
        assessment.changed_execution_roles()[0].status(),
        ToolchainCurrencyRoleStatusV1::AlreadyOnObservedHead
    );
}

#[test]
fn exact_role_exception_acknowledges_but_does_not_erase_head_mismatch() {
    let current = matrix(MatrixFixtureV1::default());
    let proposed = matrix(MatrixFixtureV1 {
        nightly_commit: "intermediate-nightly",
        nightly_cargo_commit: "intermediate-cargo",
        ..MatrixFixtureV1::default()
    });
    let observed = matrix(MatrixFixtureV1 {
        nightly_commit: "newest-nightly",
        nightly_cargo_commit: "newest-cargo",
        ..MatrixFixtureV1::default()
    });
    let candidate = candidate(current, proposed);
    let policy = policy();
    let exception = ToolchainCurrencyExceptionV1::try_new(
        &candidate,
        &policy,
        vec![ToolchainRoleV1::NightlyShadow],
        LifecycleTimestampV1::from_unix_seconds(1_000),
        LifecycleTimestampV1::from_unix_seconds(1_500),
        decision(),
    )
    .unwrap();
    let assessment = ToolchainCurrencyAssessmentV1::try_evaluate(
        &candidate,
        &snapshot(&observed, [100, 800, 900], 1_000),
        &policy,
        Some(&exception),
        LifecycleTimestampV1::from_unix_seconds(1_100),
    )
    .unwrap();

    assert_eq!(
        assessment.changed_execution_roles()[0].status(),
        ToolchainCurrencyRoleStatusV1::CandidateDoesNotMatchObservedHeadExcepted {
            exception_identity_sha256: exception.identity_sha256(),
        }
    );
    assert_eq!(
        assessment.exception_state(),
        ToolchainCurrencyExceptionStateV1::Active
    );
}

#[test]
fn inactive_exception_never_changes_the_head_relation() {
    let current = matrix(MatrixFixtureV1::default());
    let proposed = matrix(MatrixFixtureV1 {
        nightly_commit: "intermediate-nightly",
        nightly_cargo_commit: "intermediate-cargo",
        ..MatrixFixtureV1::default()
    });
    let observed = matrix(MatrixFixtureV1 {
        nightly_commit: "newest-nightly",
        nightly_cargo_commit: "newest-cargo",
        ..MatrixFixtureV1::default()
    });
    let candidate = candidate(current, proposed);
    let policy = policy();
    let exception = ToolchainCurrencyExceptionV1::try_new(
        &candidate,
        &policy,
        vec![ToolchainRoleV1::NightlyShadow],
        LifecycleTimestampV1::from_unix_seconds(1_000),
        LifecycleTimestampV1::from_unix_seconds(1_500),
        decision(),
    )
    .unwrap();
    let snapshot = snapshot(&observed, [100, 800, 900], 900);
    for (evaluated_at, expected_state) in [
        (999, ToolchainCurrencyExceptionStateV1::NotYetValid),
        (1_501, ToolchainCurrencyExceptionStateV1::Expired),
    ] {
        let assessment = ToolchainCurrencyAssessmentV1::try_evaluate(
            &candidate,
            &snapshot,
            &policy,
            Some(&exception),
            LifecycleTimestampV1::from_unix_seconds(evaluated_at),
        )
        .unwrap();
        assert_eq!(assessment.exception_state(), expected_state);
        assert_eq!(
            assessment.changed_execution_roles()[0].status(),
            ToolchainCurrencyRoleStatusV1::CandidateDoesNotMatchObservedHead
        );
    }
}

#[test]
fn snapshot_and_policy_identities_bind_every_currency_input() {
    let baseline = matrix(MatrixFixtureV1::default());
    let first = snapshot(&baseline, [100, 200, 300], 400);
    let second = ToolchainChannelSnapshotV1::try_new(
        first.stable().clone(),
        first.beta().clone(),
        first.nightly().clone(),
        first.observed_at(),
        ToolchainChannelSnapshotEvidenceV1::new(
            digest("different-provider"),
            first.schema_identity_sha256(),
            first.source_snapshot_sha256(),
            first.completeness_receipt_sha256(),
        ),
    )
    .unwrap();
    assert_ne!(first.identity_sha256(), second.identity_sha256());

    let policy = policy();
    let changed = ToolchainCurrencyPolicyV1::new(
        policy.stable_adoption_target_seconds() + 1,
        policy.beta_refresh_target_seconds(),
        policy.nightly_refresh_target_seconds(),
        policy.observation_freshness_seconds(),
        policy.maximum_exception_duration_seconds(),
        policy.policy_receipt_sha256(),
    );
    assert_ne!(policy.identity_sha256(), changed.identity_sha256());
}
