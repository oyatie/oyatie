use super::{lifecycle_producer_artifact_fixture::*, lifecycle_support::digest};
use dependency_declarations_reconcile::*;

#[test]
fn artifact_binds_every_plane_and_replays_with_one_identity() {
    let first = ArtifactFixture::new().try_build().unwrap();
    let second = ArtifactFixture::new().try_build().unwrap();
    let mut changed_fixture = ArtifactFixture::new();
    changed_fixture.envelope = artifact_envelope(
        changed_fixture.toolchains.identity_sha256(),
        "repository-revision",
        "replacement-lifecycle-artifact-producer",
        100,
        300,
    );
    let changed = changed_fixture.try_build().unwrap();

    assert_eq!(first.identity_sha256(), second.identity_sha256());
    assert_ne!(first.identity_sha256(), changed.identity_sha256());
    assert_eq!(
        first.released_releases().completeness(),
        ReleaseLedgerCompletenessV1::ReleasedComplete
    );
    assert_eq!(
        first.preview_releases().completeness(),
        ReleaseLedgerCompletenessV1::Provisional
    );
    assert_eq!(first.advisories().facts().len(), 1);
    assert_eq!(first.dependencies().nodes().len(), 1);
    assert_eq!(
        first.toolchains().msrv().version(),
        first.toolchains().stable().version()
    );
    assert_ne!(
        first.toolchains().nightly().material_identity_sha256(),
        first.channels().nightly().material_identity_sha256()
    );
    first
        .require_safe_at(LifecycleTimestampV1::from_unix_seconds(250))
        .unwrap();
}

#[test]
fn artifact_refuses_a_mixed_repository_view() {
    let mut fixture = ArtifactFixture::new();
    fixture.envelope = artifact_envelope(
        fixture.toolchains.identity_sha256(),
        "different-repository-revision",
        "lifecycle-artifact-producer",
        100,
        300,
    );

    assert_eq!(
        fixture.try_build().unwrap_err().class(),
        LifecycleFailureClassV1::LifecycleArtifactMismatch
    );
}

#[test]
fn artifact_refuses_a_toolchain_identity_not_bound_by_its_envelope() {
    let mut fixture = ArtifactFixture::new();
    fixture.envelope = artifact_envelope(
        digest("different-toolchain-matrix"),
        "repository-revision",
        "lifecycle-artifact-producer",
        100,
        300,
    );

    assert_eq!(
        fixture.try_build().unwrap_err().class(),
        LifecycleFailureClassV1::LifecycleArtifactMismatch
    );
}

#[test]
fn artifact_refuses_released_mixed_or_unqualified_preview_evidence() {
    let mut released_preview = ArtifactFixture::new();
    released_preview.preview = released_ledger();
    assert_eq!(
        released_preview.try_build().unwrap_err().class(),
        LifecycleFailureClassV1::LifecycleArtifactMismatch
    );

    let mut mixed_preview = ArtifactFixture::new();
    mixed_preview.preview = mixed_preview_ledger();
    assert_eq!(
        mixed_preview.try_build().unwrap_err().class(),
        LifecycleFailureClassV1::LifecycleArtifactMismatch
    );

    let mut unqualified_preview = ArtifactFixture::new();
    unqualified_preview.preview = preview_ledger(false);
    assert_eq!(
        unqualified_preview.try_build().unwrap_err().class(),
        LifecycleFailureClassV1::UnqualifiedExtraction
    );
}

#[test]
fn artifact_refuses_incomplete_advisory_coverage() {
    let mut fixture = ArtifactFixture::new();
    fixture.advisories = advisory_ledger(false);

    assert_eq!(
        fixture.try_build().unwrap_err().class(),
        LifecycleFailureClassV1::IncompleteFactCoverage
    );
}

#[test]
fn artifact_rechecks_freshness_for_every_consumer() {
    let artifact = ArtifactFixture::new().try_build().unwrap();
    assert_eq!(
        artifact
            .require_safe_at(LifecycleTimestampV1::from_unix_seconds(199))
            .unwrap_err()
            .class(),
        LifecycleFailureClassV1::StaleFact
    );
    assert_eq!(
        artifact
            .require_safe_at(LifecycleTimestampV1::from_unix_seconds(301))
            .unwrap_err()
            .class(),
        LifecycleFailureClassV1::StaleFact
    );
}

#[test]
fn artifact_refuses_an_envelope_stale_at_assembly_time() {
    let mut fixture = ArtifactFixture::new();
    fixture.envelope = artifact_envelope(
        fixture.toolchains.identity_sha256(),
        "repository-revision",
        "lifecycle-artifact-producer",
        100,
        199,
    );

    assert_eq!(
        fixture.try_build().unwrap_err().class(),
        LifecycleFailureClassV1::StaleFact
    );
}
