use super::lifecycle_support::digest;
use super::toolchain_safety_support::*;
use dependency_declarations_reconcile::*;

fn recovery_policy() -> ToolchainStableRecoveryPolicyV1 {
    ToolchainStableRecoveryPolicyV1::new(600, digest("stable-recovery-policy"))
}

#[test]
fn blocked_stable_can_recover_to_an_exact_retained_profile() {
    let (current, proposed) = recovery_matrices();
    let evaluated_at = LifecycleTimestampV1::from_unix_seconds(500);
    let current_safety = safety_posture(
        current.stable(),
        vec![blocking_defect(current.stable(), "rust-lang/rust#161441")],
        400,
    );
    let proposed_safety =
        safety_posture(proposed.stable(), Vec::new(), evaluated_at.unix_seconds());
    let retained = RetainedStableProfileV1::try_new(
        proposed.stable().clone(),
        LifecycleTimestampV1::from_unix_seconds(100),
        LifecycleTimestampV1::from_unix_seconds(300),
        digest("stable-history"),
        digest("retained-stable-artifact"),
    )
    .unwrap();
    let policy = recovery_policy();
    let incident = ToolchainRecoveryIncidentV1::try_new(
        &current_safety,
        &policy,
        LifecycleTimestampV1::from_unix_seconds(400),
        LifecycleTimestampV1::from_unix_seconds(600),
        ToolchainRecoveryDecisionEvidenceV1::new(
            digest("incident-authority"),
            digest("incident-schema"),
            digest("incident-receipt"),
        ),
    )
    .unwrap();
    let candidate = ToolchainStableRecoveryCandidateV1::try_new(
        current,
        proposed,
        ToolchainStableRecoveryEvidenceV1::new(current_safety, proposed_safety, retained, incident),
        evaluated_at,
        digest("recovery-discovery"),
    )
    .unwrap();

    assert_eq!(
        candidate.delta().changed_roles(),
        &[ToolchainRoleV1::QualifiedStableExecution]
    );
    assert_eq!(
        candidate.current().msrv().identity_sha256(),
        candidate.proposed().msrv().identity_sha256()
    );
    assert_eq!(
        candidate.current().beta().identity_sha256(),
        candidate.proposed().beta().identity_sha256()
    );
    assert_eq!(
        candidate.current().nightly().identity_sha256(),
        candidate.proposed().nightly().identity_sha256()
    );
    assert!(candidate.proposed().stable().version() < candidate.current().stable().version());
    assert_eq!(candidate.evaluated_at(), evaluated_at);
}

#[test]
fn recovery_identity_binds_incident_history_and_safety_evidence() {
    let (current, proposed) = recovery_matrices();
    let now = LifecycleTimestampV1::from_unix_seconds(500);
    let current_safety = safety_posture(
        current.stable(),
        vec![blocking_defect(current.stable(), "rust-lang/rust#161441")],
        400,
    );
    let proposed_safety = safety_posture(proposed.stable(), Vec::new(), now.unix_seconds());
    let policy = recovery_policy();
    let make = |history_receipt, incident_receipt| {
        let retained = RetainedStableProfileV1::try_new(
            proposed.stable().clone(),
            LifecycleTimestampV1::from_unix_seconds(100),
            LifecycleTimestampV1::from_unix_seconds(300),
            history_receipt,
            digest("retained-stable-artifact"),
        )
        .unwrap();
        let incident = ToolchainRecoveryIncidentV1::try_new(
            &current_safety,
            &policy,
            LifecycleTimestampV1::from_unix_seconds(400),
            LifecycleTimestampV1::from_unix_seconds(600),
            ToolchainRecoveryDecisionEvidenceV1::new(
                digest("incident-authority"),
                digest("incident-schema"),
                incident_receipt,
            ),
        )
        .unwrap();
        ToolchainStableRecoveryCandidateV1::try_new(
            current.clone(),
            proposed.clone(),
            ToolchainStableRecoveryEvidenceV1::new(
                current_safety.clone(),
                proposed_safety.clone(),
                retained,
                incident,
            ),
            now,
            digest("recovery-discovery"),
        )
        .unwrap()
    };
    let baseline = make(digest("stable-history"), digest("incident-receipt"));

    assert_eq!(
        baseline,
        make(digest("stable-history"), digest("incident-receipt"))
    );
    assert_ne!(
        baseline.identity_sha256(),
        make(digest("other-stable-history"), digest("incident-receipt")).identity_sha256()
    );
    assert_ne!(
        baseline.identity_sha256(),
        make(digest("stable-history"), digest("other-incident-receipt")).identity_sha256()
    );
}

#[test]
fn recovery_rechecks_safety_freshness_at_candidate_time() {
    let (current, proposed) = recovery_matrices();
    let current_safety = ToolchainSafetyPostureV1::try_evaluate(
        current.stable(),
        vec![blocking_defect(current.stable(), "rust-lang/rust#161441")],
        safety_envelope(
            current.stable(),
            vec![FactEvidenceClassV1::Proven, FactEvidenceClassV1::Observed],
            FactCertaintyV1::Exact,
            FactCoverageV1::CompleteForScope {
                scope_sha256: digest("supported-toolchain-defect-scope"),
                exclusions_sha256: digest("declared-toolchain-defect-exclusions"),
            },
            100,
            450,
        ),
        LifecycleTimestampV1::from_unix_seconds(400),
    )
    .unwrap();
    let proposed_safety = safety_posture(proposed.stable(), Vec::new(), 500);
    let policy = recovery_policy();
    let incident = ToolchainRecoveryIncidentV1::try_new(
        &current_safety,
        &policy,
        LifecycleTimestampV1::from_unix_seconds(400),
        LifecycleTimestampV1::from_unix_seconds(600),
        ToolchainRecoveryDecisionEvidenceV1::new(
            digest("incident-authority"),
            digest("incident-schema"),
            digest("incident-receipt"),
        ),
    )
    .unwrap();
    let retained = RetainedStableProfileV1::try_new(
        proposed.stable().clone(),
        LifecycleTimestampV1::from_unix_seconds(100),
        LifecycleTimestampV1::from_unix_seconds(300),
        digest("stable-history"),
        digest("retained-stable-artifact"),
    )
    .unwrap();
    let failure = ToolchainStableRecoveryCandidateV1::try_new(
        current,
        proposed,
        ToolchainStableRecoveryEvidenceV1::new(current_safety, proposed_safety, retained, incident),
        LifecycleTimestampV1::from_unix_seconds(500),
        digest("recovery-discovery"),
    )
    .unwrap_err();

    assert_eq!(failure.class(), LifecycleFailureClassV1::StaleFact);
}
