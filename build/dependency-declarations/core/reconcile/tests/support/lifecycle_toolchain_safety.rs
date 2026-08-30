use super::lifecycle_support::digest;
use super::toolchain_safety_support::*;
use dependency_declarations_reconcile::*;

#[test]
fn exact_rust_1_98_defect_blocks_the_affected_material() {
    let (current, _) = recovery_matrices();
    let defect = blocking_defect(current.stable(), "rust-lang/rust#161441");
    let posture = safety_posture(current.stable(), vec![defect.clone()], 500);

    assert_eq!(posture.status(), ToolchainSafetyPostureStatusV1::Blocked);
    assert_eq!(posture.blocking_defects(), &[defect]);
    assert_eq!(
        posture.profile_material_identity_sha256(),
        current.stable().material_identity_sha256()
    );
}

#[test]
fn complete_empty_scope_reports_no_known_blocker() {
    let (current, _) = recovery_matrices();
    let posture = safety_posture(current.stable(), Vec::new(), 500);

    assert_eq!(
        posture.status(),
        ToolchainSafetyPostureStatusV1::NoKnownBlockingDefect
    );
    assert!(posture.blocking_defects().is_empty());
}

#[test]
fn defect_order_does_not_change_the_posture_identity() {
    let (current, _) = recovery_matrices();
    let first = blocking_defect(current.stable(), "rust-lang/rust#161441");
    let second = blocking_defect(current.stable(), "rust-lang/rust#161442");
    let forward = safety_posture(current.stable(), vec![first.clone(), second.clone()], 500);
    let reverse = safety_posture(current.stable(), vec![second, first], 500);

    assert_eq!(forward, reverse);
    assert_eq!(forward.identity_sha256(), reverse.identity_sha256());
    assert!(forward.blocking_defects().windows(2).all(|pair| {
        pair[0].normalized_advisory_identity_sha256()
            < pair[1].normalized_advisory_identity_sha256()
    }));
}

#[test]
fn decision_and_applicability_evidence_change_defect_identity() {
    let (current, _) = recovery_matrices();
    let advisory = normalized_upstream_advisory("rust-lang/rust#161441");
    let decision = ToolchainDefectDecisionEvidenceV1::new(
        digest("authority"),
        digest("schema"),
        digest("decision"),
    );
    let baseline = ToolchainBlockingDefectV1::try_new(
        current.stable(),
        &advisory,
        digest("applicability"),
        decision,
    )
    .unwrap();

    assert_ne!(
        baseline.identity_sha256(),
        ToolchainBlockingDefectV1::try_new(
            current.stable(),
            &advisory,
            digest("other-applicability"),
            decision,
        )
        .unwrap()
        .identity_sha256()
    );
    assert_ne!(
        baseline.identity_sha256(),
        ToolchainBlockingDefectV1::try_new(
            current.stable(),
            &advisory,
            digest("applicability"),
            ToolchainDefectDecisionEvidenceV1::new(
                digest("authority"),
                digest("schema"),
                digest("other-decision"),
            ),
        )
        .unwrap()
        .identity_sha256()
    );
}
