use super::lifecycle_support::{digest, profile};
use super::toolchain_safety_support::*;
use dependency_declarations_reconcile::*;

fn policy() -> ToolchainStableRecoveryPolicyV1 {
    ToolchainStableRecoveryPolicyV1::new(600, digest("stable-recovery-policy"))
}

fn retained(profile: &ToolchainProfileV1) -> RetainedStableProfileV1 {
    RetainedStableProfileV1::try_new(
        profile.clone(),
        LifecycleTimestampV1::from_unix_seconds(100),
        LifecycleTimestampV1::from_unix_seconds(300),
        digest("stable-history"),
        digest("retained-stable-artifact"),
    )
    .unwrap()
}

fn incident(
    current_safety: &ToolchainSafetyPostureV1,
    opened_at: u64,
    expires_at: u64,
) -> Result<ToolchainRecoveryIncidentV1, LifecycleFailureV1> {
    ToolchainRecoveryIncidentV1::try_new(
        current_safety,
        &policy(),
        LifecycleTimestampV1::from_unix_seconds(opened_at),
        LifecycleTimestampV1::from_unix_seconds(expires_at),
        ToolchainRecoveryDecisionEvidenceV1::new(
            digest("incident-authority"),
            digest("incident-schema"),
            digest("incident-receipt"),
        ),
    )
}

#[test]
fn recovery_incident_requires_a_blocked_posture_and_bounded_window() {
    let (current, _) = recovery_matrices();
    let clear = safety_posture(current.stable(), Vec::new(), 500);
    let failure = incident(&clear, 500, 600).unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidToolchainRecovery
    );

    let blocked = safety_posture(
        current.stable(),
        vec![blocking_defect(current.stable(), "rust-lang/rust#161441")],
        500,
    );
    for (opened_at, expires_at) in [(700, 600), (0, 601), (499, 600)] {
        let failure = incident(&blocked, opened_at, expires_at).unwrap_err();
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::InvalidToolchainRecovery
        );
    }
}

#[test]
fn retained_profile_requires_production_stable_history() {
    let (current, proposed) = recovery_matrices();
    let timestamps = |profile, qualified_at, superseded_at| {
        RetainedStableProfileV1::try_new(
            profile,
            LifecycleTimestampV1::from_unix_seconds(qualified_at),
            LifecycleTimestampV1::from_unix_seconds(superseded_at),
            digest("stable-history"),
            digest("retained-stable-artifact"),
        )
    };

    for failure in [
        timestamps(current.nightly().clone(), 100, 300).unwrap_err(),
        timestamps(proposed.stable().clone(), 300, 300).unwrap_err(),
        timestamps(proposed.stable().clone(), 301, 300).unwrap_err(),
    ] {
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::InvalidToolchainRecovery
        );
    }
}

#[test]
fn recovery_refuses_a_blocked_target_or_inactive_incident() {
    let (current, proposed) = recovery_matrices();
    let now = LifecycleTimestampV1::from_unix_seconds(500);
    let current_safety = safety_posture(
        current.stable(),
        vec![blocking_defect(current.stable(), "rust-lang/rust#161441")],
        300,
    );
    let active_incident = incident(&current_safety, 400, 600).unwrap();
    for proposed_safety in [
        safety_posture(
            proposed.stable(),
            vec![blocking_defect(proposed.stable(), "rust-lang/rust#150000")],
            now.unix_seconds(),
        ),
        safety_posture(proposed.stable(), Vec::new(), 399),
        safety_posture(proposed.stable(), Vec::new(), 501),
    ] {
        let failure = ToolchainStableRecoveryCandidateV1::try_new(
            current.clone(),
            proposed.clone(),
            ToolchainStableRecoveryEvidenceV1::new(
                current_safety.clone(),
                proposed_safety,
                retained(proposed.stable()),
                active_incident.clone(),
            ),
            now,
            digest("recovery-discovery"),
        )
        .unwrap_err();
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::InvalidToolchainRecovery
        );
    }

    let proposed_safety = safety_posture(proposed.stable(), Vec::new(), now.unix_seconds());
    let expired_incident = incident(&current_safety, 300, 400).unwrap();
    let failure = ToolchainStableRecoveryCandidateV1::try_new(
        current,
        proposed.clone(),
        ToolchainStableRecoveryEvidenceV1::new(
            current_safety,
            proposed_safety,
            retained(proposed.stable()),
            expired_incident,
        ),
        now,
        digest("recovery-discovery"),
    )
    .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::StaleFact);
}

#[test]
fn recovery_refuses_non_stable_changes_and_unretained_targets() {
    let (current, proposed) = recovery_matrices();
    let changed_msrv = profile(
        ToolchainRoleV1::DeclaredMsrvCompatibility,
        RustVersionV1::try_new(1, 95, 0).unwrap(),
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "msrv-1.95.0",
        "cargo-msrv-1.95.0",
    );
    let proposed_with_msrv_change = ToolchainMatrixV1::try_new(
        changed_msrv,
        proposed.stable().clone(),
        proposed.beta().clone(),
        proposed.nightly().clone(),
    )
    .unwrap();
    let now = LifecycleTimestampV1::from_unix_seconds(500);
    let current_safety = safety_posture(
        current.stable(),
        vec![blocking_defect(current.stable(), "rust-lang/rust#161441")],
        400,
    );
    let proposed_safety = safety_posture(
        proposed_with_msrv_change.stable(),
        Vec::new(),
        now.unix_seconds(),
    );
    let active_incident = incident(&current_safety, 400, 600).unwrap();
    let failure = ToolchainStableRecoveryCandidateV1::try_new(
        current.clone(),
        proposed_with_msrv_change,
        ToolchainStableRecoveryEvidenceV1::new(
            current_safety.clone(),
            proposed_safety,
            retained(proposed.stable()),
            active_incident.clone(),
        ),
        now,
        digest("recovery-discovery"),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidToolchainRecovery
    );

    let proposed_safety = safety_posture(proposed.stable(), Vec::new(), now.unix_seconds());
    let failure = ToolchainStableRecoveryCandidateV1::try_new(
        current.clone(),
        proposed.clone(),
        ToolchainStableRecoveryEvidenceV1::new(
            current_safety.clone(),
            proposed_safety,
            retained(current.stable()),
            active_incident,
        ),
        now,
        digest("recovery-discovery"),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ToolchainAnalysisMismatch
    );

    let forward_current_safety = safety_posture(
        proposed.stable(),
        vec![blocking_defect(proposed.stable(), "rust-lang/rust#150000")],
        400,
    );
    let forward_target_safety = safety_posture(current.stable(), Vec::new(), now.unix_seconds());
    let failure = ToolchainStableRecoveryCandidateV1::try_new(
        proposed.clone(),
        current.clone(),
        ToolchainStableRecoveryEvidenceV1::new(
            forward_current_safety.clone(),
            forward_target_safety,
            retained(current.stable()),
            incident(&forward_current_safety, 400, 600).unwrap(),
        ),
        now,
        digest("recovery-discovery"),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidToolchainRecovery
    );
}

#[test]
fn recovery_incident_cannot_be_replayed_across_safety_evidence() {
    let (current, proposed) = recovery_matrices();
    let now = LifecycleTimestampV1::from_unix_seconds(500);
    let original_safety = safety_posture(
        current.stable(),
        vec![blocking_defect(current.stable(), "rust-lang/rust#161441")],
        400,
    );
    let advisory = normalized_upstream_advisory("rust-lang/rust#161441");
    let changed_defect = ToolchainBlockingDefectV1::try_new(
        current.stable(),
        &advisory,
        digest("rust-lang/rust#161441-applicability"),
        ToolchainDefectDecisionEvidenceV1::new(
            digest("toolchain-safety-authority"),
            digest("toolchain-safety-decision-schema"),
            digest("changed-decision"),
        ),
    )
    .unwrap();
    let changed_safety = safety_posture(current.stable(), vec![changed_defect], 400);
    let failure = ToolchainStableRecoveryCandidateV1::try_new(
        current,
        proposed.clone(),
        ToolchainStableRecoveryEvidenceV1::new(
            changed_safety,
            safety_posture(proposed.stable(), Vec::new(), now.unix_seconds()),
            retained(proposed.stable()),
            incident(&original_safety, 400, 600).unwrap(),
        ),
        now,
        digest("recovery-discovery"),
    )
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ToolchainAnalysisMismatch
    );
}

#[test]
fn an_execution_rollback_cannot_cross_the_declared_msrv() {
    let (current, proposed) = recovery_matrices();
    let msrv_1_98 = profile(
        ToolchainRoleV1::DeclaredMsrvCompatibility,
        RustVersionV1::try_new(1, 98, 0).unwrap(),
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "msrv-1.98.0",
        "cargo-msrv-1.98.0",
    );
    let failure = ToolchainMatrixV1::try_new(
        msrv_1_98,
        proposed.stable().clone(),
        current.beta().clone(),
        current.nightly().clone(),
    )
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::UnsupportedVersionRelation
    );
}
