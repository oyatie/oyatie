use super::dependency_candidate::*;
use super::dependency_graph::{
    complete_envelope, continue_dependency_impact, node, qualified_h2_candidate,
};
use super::lifecycle_support::{digest, profile};
use dependency_declarations_reconcile::*;

pub(super) fn qualification_matrix(msrv_minor: u16) -> ToolchainMatrixV1 {
    ToolchainMatrixV1::try_new(
        profile(
            ToolchainRoleV1::DeclaredMsrvCompatibility,
            RustVersionV1::try_new(1, msrv_minor, 0).unwrap(),
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            "msrv-rustc",
            "msrv-cargo",
        ),
        profile(
            ToolchainRoleV1::QualifiedStableExecution,
            RustVersionV1::try_new(1, 98, 0).unwrap(),
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            "stable-rustc",
            "stable-cargo",
        ),
        profile(
            ToolchainRoleV1::BetaShadow,
            RustVersionV1::try_new(1, 99, 0).unwrap(),
            LifecycleChannelV1::Beta,
            SourceMaturityV1::Provisional,
            "beta-rustc",
            "beta-cargo",
        ),
        profile(
            ToolchainRoleV1::NightlyShadow,
            RustVersionV1::try_new(1, 100, 0).unwrap(),
            LifecycleChannelV1::Nightly,
            SourceMaturityV1::Provisional,
            "nightly-rustc",
            "nightly-cargo",
        ),
    )
    .unwrap()
}

pub(super) fn quarantine_policy() -> DependencyQuarantinePolicyV1 {
    DependencyQuarantinePolicyV1::new(180, 120, 1_000, digest("dependency-policy"))
}

pub(super) fn security_decision() -> DependencySecurityDecisionEvidenceV1 {
    DependencySecurityDecisionEvidenceV1::new(
        digest("security-authority"),
        digest("security-decision-schema"),
        digest("security-decision-receipt"),
    )
}

pub(super) fn candidate_with_proposed_msrv(
    proposed_msrv: DependencyMsrvDeclarationV1,
    proposed_advisories: Vec<DigestV1>,
) -> DependencyCandidateV1 {
    let current = try_release_with_facts(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Released,
            "h2-0.4.15",
        ),
        package("h2"),
        "0.4.15",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
        DependencyReleaseFactsV1 {
            changed: false,
            msrv: DependencyMsrvDeclarationV1::Declared {
                version: RustVersionV1::try_new(1, 63, 0).unwrap(),
                evidence_sha256: digest("current-msrv"),
            },
            published_at: 100,
            observed_at: 200,
            advisory_identities: vec![digest("RUSTSEC-2026-0258")],
        },
    )
    .unwrap();
    let proposed = try_release_with_facts(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Released,
            "h2-0.4.16",
        ),
        package("h2"),
        "0.4.16",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
        DependencyReleaseFactsV1 {
            changed: true,
            msrv: proposed_msrv,
            published_at: 100,
            observed_at: 200,
            advisory_identities: proposed_advisories,
        },
    )
    .unwrap();
    DependencyCandidateV1::try_new(current, proposed, digest("qualified-candidate")).unwrap()
}

pub(super) fn candidate_impact(candidate: &DependencyCandidateV1, now: u64) -> DependencyImpactV1 {
    let envelope = complete_envelope(
        vec![FactEvidenceClassV1::Declared, FactEvidenceClassV1::Proven],
        FactCertaintyV1::Exact,
        FactCoverageV1::CompleteForScope {
            scope_sha256: digest("dependency-impact-scope"),
            exclusions_sha256: digest("dependency-impact-exclusions"),
        },
        100,
        1_000,
    );
    let graph = DependencyGraphV1::try_new(
        envelope,
        vec![node(
            "candidate-package",
            DependencyGraphNodeKindV1::CargoPackage,
            Some(candidate.current().identity_sha256()),
        )],
        Vec::new(),
    )
    .unwrap();
    graph
        .try_analyze_candidates(
            std::slice::from_ref(candidate),
            LifecycleTimestampV1::from_unix_seconds(now),
            continue_dependency_impact,
        )
        .unwrap()
        .impacts()[0]
        .clone()
}

#[test]
fn msrv_precheck_keeps_declared_floor_and_stable_execution_separate() {
    let candidate = qualified_h2_candidate();
    let matrix = qualification_matrix(64);
    let compatibility = DependencyMsrvCompatibilityV1::new(&candidate, &matrix);

    assert_ne!(
        compatibility.declared_msrv_profile_identity_sha256(),
        compatibility.stable_profile_identity_sha256()
    );
    assert_eq!(
        compatibility.declared_msrv_version(),
        RustVersionV1::try_new(1, 64, 0).unwrap()
    );
    assert_eq!(
        compatibility.stable_version(),
        RustVersionV1::try_new(1, 98, 0).unwrap()
    );
    assert!(matches!(
        compatibility.current(),
        DependencyMsrvRelationV1::WithinDeclaredFloor { .. }
    ));
    assert!(matches!(
        compatibility.proposed(),
        DependencyMsrvRelationV1::WithinDeclaredFloor { .. }
    ));
}

#[test]
fn quarantine_holds_both_windows_and_security_bypasses_only_time_gates() {
    let candidate = qualified_h2_candidate();
    let matrix = qualification_matrix(64);
    let compatibility = DependencyMsrvCompatibilityV1::new(&candidate, &matrix);
    let policy = quarantine_policy();
    let now = LifecycleTimestampV1::from_unix_seconds(250);
    let held = DependencyQuarantineV1::try_evaluate(&candidate, &policy, None, now).unwrap();

    assert_eq!(
        held.publication_age(),
        DependencyQuarantineGateV1::Held {
            eligible_at: LifecycleTimestampV1::from_unix_seconds(280),
        }
    );
    assert_eq!(
        held.maintainer_change(),
        DependencyQuarantineGateV1::Held {
            eligible_at: LifecycleTimestampV1::from_unix_seconds(320),
        }
    );
    let impact = candidate_impact(&candidate, 250);
    let recommendation = DependencyQualificationRecommendationV1::try_new(
        &candidate,
        &impact,
        &compatibility,
        &held,
        now,
    )
    .unwrap();
    assert_eq!(
        recommendation.blockers(),
        &[
            DependencyQualificationBlockerV1::PublicationAge,
            DependencyQualificationBlockerV1::MaintainerChangeHold,
        ]
    );
    assert_eq!(recommendation.mode(), None);

    let exception = DependencyEmergencySecurityExceptionV1::try_new(
        &candidate,
        &policy,
        digest("RUSTSEC-2026-0258"),
        LifecycleTimestampV1::from_unix_seconds(220),
        LifecycleTimestampV1::from_unix_seconds(300),
        security_decision(),
    )
    .unwrap();
    let expedited =
        DependencyQuarantineV1::try_evaluate(&candidate, &policy, Some(&exception), now).unwrap();
    assert!(matches!(
        expedited.publication_age(),
        DependencyQuarantineGateV1::Bypassed { .. }
    ));
    assert!(matches!(
        expedited.maintainer_change(),
        DependencyQuarantineGateV1::Bypassed { .. }
    ));
    let recommendation = DependencyQualificationRecommendationV1::try_new(
        &candidate,
        &impact,
        &compatibility,
        &expedited,
        now,
    )
    .unwrap();
    assert_eq!(recommendation.blockers(), &[]);
    assert_eq!(
        recommendation.mode(),
        Some(DependencyQualificationModeV1::ExpeditedSecurity {
            exception_identity_sha256: exception.identity_sha256(),
        })
    );

    let higher_floor = DependencyMsrvCompatibilityV1::new(&candidate, &qualification_matrix(63));
    let blocked = DependencyQualificationRecommendationV1::try_new(
        &candidate,
        &impact,
        &higher_floor,
        &expedited,
        now,
    )
    .unwrap();
    assert_eq!(
        blocked.blockers(),
        &[DependencyQualificationBlockerV1::MsrvFloorDecision]
    );
    assert_eq!(blocked.mode(), None);
}

#[test]
fn mature_candidate_is_ready_for_qualification_not_accepted() {
    let candidate = qualified_h2_candidate();
    let matrix = qualification_matrix(64);
    let compatibility = DependencyMsrvCompatibilityV1::new(&candidate, &matrix);
    let policy = quarantine_policy();
    let now = LifecycleTimestampV1::from_unix_seconds(350);
    let quarantine = DependencyQuarantineV1::try_evaluate(&candidate, &policy, None, now).unwrap();
    let impact = candidate_impact(&candidate, 350);
    let recommendation = DependencyQualificationRecommendationV1::try_new(
        &candidate,
        &impact,
        &compatibility,
        &quarantine,
        now,
    )
    .unwrap();

    assert!(recommendation.is_ready_for_qualification());
    assert_eq!(recommendation.blockers(), &[]);
    assert_eq!(
        recommendation.mode(),
        Some(DependencyQualificationModeV1::Standard)
    );
    assert_eq!(
        recommendation.impact_identity_sha256(),
        impact.identity_sha256()
    );
}
