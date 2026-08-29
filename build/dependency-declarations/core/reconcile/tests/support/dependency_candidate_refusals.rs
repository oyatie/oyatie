use super::dependency_candidate::*;
use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

fn h2_release(
    version: &str,
    state: DependencyPublicationStateV1,
    qualification: DependencyFactQualificationV1,
) -> CargoDependencyReleaseV1 {
    release(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Released,
            &format!("h2-{version}"),
        ),
        package("h2"),
        version,
        state,
        qualification,
        version == "0.4.16",
    )
}

#[test]
fn dependency_release_requires_the_registry_source_contract() {
    let wrong_component = try_release(
        dependency_source(
            LifecycleComponentV1::Cargo,
            SourceMaturityV1::Released,
            "cargo-source",
        ),
        package("h2"),
        "0.4.15",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
        false,
    )
    .unwrap_err();
    assert_eq!(
        wrong_component.class(),
        LifecycleFailureClassV1::DependencySourceMismatch
    );

    for (channel, scope) in [
        (LifecycleChannelV1::Stable, LifecycleSourceScopeV1::Global),
        (
            LifecycleChannelV1::Dependency,
            LifecycleSourceScopeV1::Target(
                LifecycleTargetTripleV1::try_new("aarch64-apple-darwin").unwrap(),
            ),
        ),
    ] {
        let source = LifecycleSourceV1::try_new(
            LifecycleSourceDescriptorV1::try_new(
                "crates.io",
                LifecycleComponentV1::DependencyRegistry,
                channel,
                "wrong-source-shape",
                "registry-release",
                scope,
                SourceMaturityV1::Released,
            )
            .unwrap(),
            8192,
            digest("wrong-source-object"),
            digest("dependency-release-schema-v1"),
        )
        .unwrap();
        let failure = try_release(
            source,
            package("h2"),
            "0.4.15",
            DependencyPublicationStateV1::Available,
            qualified_dependency(),
            false,
        )
        .unwrap_err();
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::DependencySourceMismatch
        );
    }

    let provisional_qualified = try_release(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Provisional,
            "provisional-source",
        ),
        package("h2"),
        "0.4.16",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
        true,
    )
    .unwrap_err();
    assert_eq!(
        provisional_qualified.class(),
        LifecycleFailureClassV1::ProvisionalSource
    );
}

#[test]
fn dependency_candidate_refuses_non_upgrade_or_cross_package_pairs() {
    let current = h2_release(
        "0.4.15",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
    );
    let same = h2_release(
        "0.4.15",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
    );
    let failure =
        DependencyCandidateV1::try_new(current.clone(), same, digest("same-version")).unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidDependencyCandidate
    );

    let other_package = release(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Released,
            "http-1.0.0",
        ),
        package("http"),
        "1.0.0",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
        true,
    );
    let failure = DependencyCandidateV1::try_new(current, other_package, digest("cross-package"))
        .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidDependencyCandidate
    );
}

#[test]
fn dependency_candidate_refuses_unavailable_or_unqualified_proposals() {
    let current = h2_release(
        "0.4.15",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
    );
    let yanked = h2_release(
        "0.4.16",
        DependencyPublicationStateV1::Yanked,
        qualified_dependency(),
    );
    let failure =
        DependencyCandidateV1::try_new(current.clone(), yanked, digest("yanked-proposal"))
            .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::UnavailableDependencyRelease
    );

    let unqualified = h2_release(
        "0.4.16",
        DependencyPublicationStateV1::Available,
        candidate_dependency(),
    );
    let failure =
        DependencyCandidateV1::try_new(current, unqualified, digest("unqualified-proposal"))
            .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::UnqualifiedExtraction
    );
}

#[test]
fn dependency_sets_and_publication_times_refuse_ambiguity() {
    let duplicate = DependencyNamedFactSetV1::try_new(vec![
        "feature:stream".to_owned(),
        "feature:stream".to_owned(),
    ])
    .unwrap_err();
    assert_eq!(
        duplicate.class(),
        LifecycleFailureClassV1::DuplicateIdentity
    );

    let duplicate_advisory =
        DependencyAdvisorySetV1::try_new(vec![digest("same"), digest("same")]).unwrap_err();
    assert_eq!(
        duplicate_advisory.class(),
        LifecycleFailureClassV1::DuplicateIdentity
    );

    let invalid_time = DependencyPublicationV1::try_new(
        LifecycleTimestampV1::from_unix_seconds(200),
        LifecycleTimestampV1::from_unix_seconds(100),
        DependencyPublicationStateV1::Available,
        digest("invalid-time"),
    )
    .unwrap_err();
    assert_eq!(invalid_time.class(), LifecycleFailureClassV1::InvalidFact);

    let excessive = DependencyNamedFactSetV1::try_new(vec![
        "fact".to_owned();
        LifecycleBoundsV1::MAX_DEPENDENCY_NAMED_FACTS
            + 1
    ])
    .unwrap_err();
    assert_eq!(excessive.class(), LifecycleFailureClassV1::BoundsExceeded);
}
