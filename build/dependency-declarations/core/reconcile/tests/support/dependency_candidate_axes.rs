use super::dependency_candidate::*;
use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

fn release_for_axis(
    version: &str,
    changed_axis: Option<DependencyChangeAxisV1>,
) -> CargoDependencyReleaseV1 {
    let changed = |axis| changed_axis == Some(axis);
    let source = dependency_source(
        LifecycleComponentV1::DependencyRegistry,
        SourceMaturityV1::Released,
        if changed(DependencyChangeAxisV1::Source) {
            "changed-source"
        } else {
            "baseline-source"
        },
    );
    let publication = DependencyPublicationV1::try_new(
        LifecycleTimestampV1::from_unix_seconds(100),
        LifecycleTimestampV1::from_unix_seconds(if changed(DependencyChangeAxisV1::Publication) {
            201
        } else {
            200
        }),
        DependencyPublicationStateV1::Available,
        digest(if changed(DependencyChangeAxisV1::Publication) {
            "changed-publication"
        } else {
            "baseline-publication"
        }),
    )
    .unwrap();
    let metadata = DependencyMetadataV1::new(
        named(if changed(DependencyChangeAxisV1::Maintainers) {
            &["maintainer:alice", "maintainer:bob"]
        } else {
            &["maintainer:alice"]
        }),
        DependencyLicenseV1::try_new(
            "MIT",
            digest(if changed(DependencyChangeAxisV1::License) {
                "changed-license-evidence"
            } else {
                "baseline-license-evidence"
            }),
        )
        .unwrap(),
        named(if changed(DependencyChangeAxisV1::Features) {
            &["feature:stream", "feature:unstable"]
        } else {
            &["feature:stream"]
        }),
        DependencyMsrvDeclarationV1::Declared {
            version: RustVersionV1::try_new(1, 63, 0).unwrap(),
            evidence_sha256: digest(if changed(DependencyChangeAxisV1::Msrv) {
                "changed-msrv-evidence"
            } else {
                "baseline-msrv-evidence"
            }),
        },
    );
    let build_surface = DependencyBuildSurfaceV1::new(
        changed(DependencyChangeAxisV1::BuildScript).then(|| digest("build-script")),
        changed(DependencyChangeAxisV1::ProcMacro),
        named(if changed(DependencyChangeAxisV1::NativeInputs) {
            &["native:cc"]
        } else {
            &[]
        }),
    );
    let evidence = DependencyReleaseEvidenceV1::new(
        digest(if changed(DependencyChangeAxisV1::DependencyManifest) {
            "changed-manifest"
        } else {
            "baseline-manifest"
        }),
        DependencyAdvisorySetV1::try_new(
            changed(DependencyChangeAxisV1::Advisories)
                .then(|| digest("RUSTSEC-2026-0258"))
                .into_iter()
                .collect(),
        )
        .unwrap(),
        digest(if changed(DependencyChangeAxisV1::Audit) {
            "changed-audit"
        } else {
            "baseline-audit"
        }),
        digest(if changed(DependencyChangeAxisV1::Provenance) {
            "changed-provenance"
        } else {
            "baseline-provenance"
        }),
        digest(if changed(DependencyChangeAxisV1::Sbom) {
            "changed-sbom"
        } else {
            "baseline-sbom"
        }),
    );
    CargoDependencyReleaseV1::try_new(
        DependencyReleaseCoordinatesV1::new(
            source,
            package("h2"),
            CargoVersionV1::try_new(version).unwrap(),
            digest(if changed(DependencyChangeAxisV1::Checksum) {
                "changed-checksum"
            } else {
                "baseline-checksum"
            }),
            publication,
        ),
        metadata,
        build_surface,
        evidence,
        qualified_dependency(),
    )
    .unwrap()
}

#[test]
fn every_release_axis_is_isolated_and_canonical() {
    let current = release_for_axis("0.4.15", None);
    let version_only = DependencyCandidateV1::try_new(
        current.clone(),
        release_for_axis("0.4.16", None),
        digest("version-only"),
    )
    .unwrap();
    assert!(version_only.delta().axes().is_empty());
    assert_ne!(
        version_only.current().identity_sha256(),
        version_only.proposed().identity_sha256()
    );

    for axis in DependencyChangeAxisV1::ALL {
        let candidate = DependencyCandidateV1::try_new(
            current.clone(),
            release_for_axis("0.4.16", Some(axis)),
            digest(&format!("{axis:?}-projection")),
        )
        .unwrap();
        assert_eq!(candidate.delta().axes(), &[axis]);
        for observed_axis in DependencyChangeAxisV1::ALL {
            assert_eq!(
                candidate.current().axes().identity_sha256(observed_axis)
                    != candidate.proposed().axes().identity_sha256(observed_axis),
                observed_axis == axis,
                "{axis:?} moved the {observed_axis:?} projection"
            );
        }
    }
}

#[test]
fn maintained_license_evidence_refresh_is_review_visible() {
    let candidate = DependencyCandidateV1::try_new(
        release_for_axis("0.4.15", None),
        release_for_axis("0.4.16", Some(DependencyChangeAxisV1::License)),
        digest("license-evidence"),
    )
    .unwrap();

    assert_eq!(candidate.delta().axes(), &[DependencyChangeAxisV1::License]);
}
