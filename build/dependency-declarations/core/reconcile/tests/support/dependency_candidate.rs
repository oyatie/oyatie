use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

pub(super) fn dependency_source(
    component: LifecycleComponentV1,
    maturity: SourceMaturityV1,
    revision: &str,
) -> LifecycleSourceV1 {
    LifecycleSourceV1::try_new(
        LifecycleSourceDescriptorV1::try_new(
            "crates.io",
            component,
            LifecycleChannelV1::Dependency,
            revision,
            "registry-release",
            LifecycleSourceScopeV1::Global,
            maturity,
        )
        .unwrap(),
        8192,
        digest(&format!("{revision}-registry-object")),
        digest("dependency-release-schema-v1"),
    )
    .unwrap()
}

pub(super) fn qualified_dependency() -> DependencyFactQualificationV1 {
    DependencyFactQualificationV1::Qualified {
        qualification_receipt_sha256: digest("qualified-dependency-provider"),
    }
}

pub(super) fn candidate_dependency() -> DependencyFactQualificationV1 {
    DependencyFactQualificationV1::Candidate {
        observation_receipt_sha256: digest("candidate-dependency-provider"),
    }
}

pub(super) fn named(values: &[&str]) -> DependencyNamedFactSetV1 {
    DependencyNamedFactSetV1::try_new(values.iter().map(|value| (*value).to_owned()).collect())
        .unwrap()
}

pub(super) fn package(name: &str) -> CargoPackageIdentityV1 {
    CargoPackageIdentityV1::try_new("https://github.com/rust-lang/crates.io-index", name).unwrap()
}

pub(super) fn release(
    source: LifecycleSourceV1,
    package: CargoPackageIdentityV1,
    version: &str,
    state: DependencyPublicationStateV1,
    qualification: DependencyFactQualificationV1,
    changed: bool,
) -> CargoDependencyReleaseV1 {
    try_release(source, package, version, state, qualification, changed).unwrap()
}

pub(super) fn try_release(
    source: LifecycleSourceV1,
    package: CargoPackageIdentityV1,
    version: &str,
    state: DependencyPublicationStateV1,
    qualification: DependencyFactQualificationV1,
    changed: bool,
) -> Result<CargoDependencyReleaseV1, LifecycleFailureV1> {
    let msrv = DependencyMsrvDeclarationV1::Declared {
        version: RustVersionV1::try_new(1, if changed { 64 } else { 63 }, 0).unwrap(),
        evidence_sha256: digest("declared-msrv"),
    };
    let advisories = if changed {
        Vec::new()
    } else {
        vec![digest("RUSTSEC-2026-0258")]
    };
    try_release_with_facts(
        source,
        package,
        version,
        state,
        qualification,
        DependencyReleaseFactsV1 {
            changed,
            msrv,
            published_at: 100,
            observed_at: 200,
            advisory_identities: advisories,
        },
    )
}

pub(super) struct DependencyReleaseFactsV1 {
    pub changed: bool,
    pub msrv: DependencyMsrvDeclarationV1,
    pub published_at: u64,
    pub observed_at: u64,
    pub advisory_identities: Vec<DigestV1>,
}

pub(super) fn try_release_with_facts(
    source: LifecycleSourceV1,
    package: CargoPackageIdentityV1,
    version: &str,
    state: DependencyPublicationStateV1,
    qualification: DependencyFactQualificationV1,
    facts: DependencyReleaseFactsV1,
) -> Result<CargoDependencyReleaseV1, LifecycleFailureV1> {
    let publication = DependencyPublicationV1::try_new(
        LifecycleTimestampV1::from_unix_seconds(facts.published_at),
        LifecycleTimestampV1::from_unix_seconds(facts.observed_at),
        state,
        digest(if facts.changed {
            "proposed-publication"
        } else {
            "current-publication"
        }),
    )
    .unwrap();
    let metadata = DependencyMetadataV1::new(
        named(if facts.changed {
            &["maintainer:alice", "maintainer:bob"]
        } else {
            &["maintainer:alice"]
        }),
        DependencyLicenseV1::try_new(
            if facts.changed {
                "MIT OR Apache-2.0"
            } else {
                "MIT"
            },
            digest(if facts.changed {
                "proposed-license"
            } else {
                "current-license"
            }),
        )
        .unwrap(),
        named(if facts.changed {
            &["feature:stream", "feature:unstable"]
        } else {
            &["feature:stream"]
        }),
        facts.msrv,
    );
    let build_surface = DependencyBuildSurfaceV1::new(
        facts.changed.then(|| digest("build-script")),
        facts.changed,
        named(if facts.changed { &["native:cc"] } else { &[] }),
    );
    let evidence = DependencyReleaseEvidenceV1::new(
        digest(if facts.changed {
            "proposed-dependency-manifest"
        } else {
            "current-dependency-manifest"
        }),
        DependencyAdvisorySetV1::try_new(facts.advisory_identities).unwrap(),
        digest(if facts.changed {
            "proposed-audit"
        } else {
            "current-audit"
        }),
        digest(if facts.changed {
            "proposed-provenance"
        } else {
            "current-provenance"
        }),
        digest(if facts.changed {
            "proposed-sbom"
        } else {
            "current-sbom"
        }),
    );
    CargoDependencyReleaseV1::try_new(
        DependencyReleaseCoordinatesV1::new(
            source,
            package,
            CargoVersionV1::try_new(version).unwrap(),
            digest(&format!("{version}-checksum")),
            publication,
        ),
        metadata,
        build_surface,
        evidence,
        qualification,
    )
}

#[test]
fn candidate_binds_exact_release_facts_and_mechanical_deltas() {
    let current = release(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Released,
            "h2-0.4.15",
        ),
        package("h2"),
        "0.4.15",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
        false,
    );
    let proposed = release(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Released,
            "h2-0.4.16",
        ),
        package("h2"),
        "0.4.16",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
        true,
    );
    let candidate =
        DependencyCandidateV1::try_new(current, proposed, digest("dependency-candidate-discovery"))
            .unwrap();

    assert_eq!(candidate.current().version().as_str(), "0.4.15");
    assert_eq!(candidate.proposed().version().as_str(), "0.4.16");
    for axis in [
        DependencyChangeAxisV1::Source,
        DependencyChangeAxisV1::Checksum,
        DependencyChangeAxisV1::Maintainers,
        DependencyChangeAxisV1::License,
        DependencyChangeAxisV1::Features,
        DependencyChangeAxisV1::Msrv,
        DependencyChangeAxisV1::BuildScript,
        DependencyChangeAxisV1::ProcMacro,
        DependencyChangeAxisV1::NativeInputs,
        DependencyChangeAxisV1::DependencyManifest,
        DependencyChangeAxisV1::Advisories,
        DependencyChangeAxisV1::Audit,
        DependencyChangeAxisV1::Provenance,
        DependencyChangeAxisV1::Sbom,
    ] {
        assert!(candidate.delta().changed(axis));
    }
    assert!(
        !candidate
            .delta()
            .changed(DependencyChangeAxisV1::PublicationState)
    );

    let refreshed = DependencyCandidateV1::try_new(
        candidate.current().clone(),
        candidate.proposed().clone(),
        digest("different-discovery-receipt"),
    )
    .unwrap();
    assert_ne!(candidate.identity_sha256(), refreshed.identity_sha256());
}

#[test]
fn moving_away_from_a_yanked_current_release_records_publication_state() {
    let current = release(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Released,
            "h2-0.4.15-yanked",
        ),
        package("h2"),
        "0.4.15",
        DependencyPublicationStateV1::Yanked,
        qualified_dependency(),
        false,
    );
    let proposed = release(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Released,
            "h2-0.4.16",
        ),
        package("h2"),
        "0.4.16",
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
        true,
    );
    let candidate =
        DependencyCandidateV1::try_new(current, proposed, digest("leave-yanked")).unwrap();
    assert!(
        candidate
            .delta()
            .changed(DependencyChangeAxisV1::PublicationState)
    );
}

#[test]
fn named_and_advisory_sets_are_order_independent_and_materialized() {
    let names = named(&["feature:z", "feature:a"]);
    let reversed = named(&["feature:a", "feature:z"]);
    assert_eq!(names, reversed);
    assert_eq!(
        names.values().collect::<Vec<_>>(),
        ["feature:a", "feature:z"]
    );

    let first = digest("advisory-a");
    let second = digest("advisory-b");
    let advisories = DependencyAdvisorySetV1::try_new(vec![second, first]).unwrap();
    let reversed = DependencyAdvisorySetV1::try_new(vec![first, second]).unwrap();
    assert_eq!(advisories, reversed);
    assert_eq!(advisories.identities(), &[first, second]);
}
