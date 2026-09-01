//! Fail-closed injections for every load-bearing owner-prose input class.

#[path = "support/owner_prose.rs"]
mod support;

use pipeline_admission::{
    OwnerProseClaim, OwnerProseClassification, OwnerProseNativeConsumer, OwnerProseProjection,
    OwnerProseQualification, OwnerProseRefusalKind, OwnerProseRevision, owner_prose_sha256,
    qualify_owner_prose,
};
use support::Fixture;

fn kinds(fixture: &Fixture) -> Vec<OwnerProseRefusalKind> {
    match fixture.qualify() {
        OwnerProseQualification::Ready(view) => panic!("expected Unknown, got {view:#?}"),
        OwnerProseQualification::Unknown(refusals) => {
            refusals.into_iter().map(|refusal| refusal.kind).collect()
        }
    }
}

fn set_claims(fixture: &mut Fixture, path: &str, claims: Vec<OwnerProseClaim>) {
    fixture
        .manifest
        .sources
        .iter_mut()
        .find(|source| source.path == path)
        .expect("source")
        .claims = claims;
}

fn claim(
    id: &str,
    bytes: &[u8],
    start: usize,
    end: usize,
    classification: OwnerProseClassification,
) -> OwnerProseClaim {
    OwnerProseClaim {
        id: id.to_owned(),
        start,
        end,
        sha256: owner_prose_sha256(&bytes[start..end]),
        classification,
        work_reference: None,
        projections: Vec::new(),
    }
}

#[test]
fn gaps_overlaps_and_duplicate_classifications_are_unknown() {
    let mut gap = Fixture::complete();
    let bytes = gap.source["policy/ADR.md"].clone();
    set_claims(
        &mut gap,
        "policy/ADR.md",
        vec![
            claim(
                "before-gap",
                &bytes,
                0,
                1,
                OwnerProseClassification::ProposalWork,
            ),
            claim(
                "after-gap",
                &bytes,
                2,
                bytes.len(),
                OwnerProseClassification::HistoricalRejected,
            ),
        ],
    );
    assert!(kinds(&gap).contains(&OwnerProseRefusalKind::ClaimCoverageMismatch));

    let mut overlap = Fixture::complete();
    let bytes = overlap.source["policy/ADR.md"].clone();
    set_claims(
        &mut overlap,
        "policy/ADR.md",
        vec![
            claim("left", &bytes, 0, 2, OwnerProseClassification::ProposalWork),
            claim(
                "right",
                &bytes,
                1,
                bytes.len(),
                OwnerProseClassification::HistoricalRejected,
            ),
        ],
    );
    assert!(kinds(&overlap).contains(&OwnerProseRefusalKind::ClaimCoverageMismatch));

    let mut duplicate = Fixture::complete();
    duplicate.manifest.sources[1].claims[0].id = duplicate.manifest.sources[0].claims[0].id.clone();
    assert!(kinds(&duplicate).contains(&OwnerProseRefusalKind::DuplicateClassification));
}

#[test]
fn missing_mismatched_and_duplicate_projections_are_unknown() {
    let mut missing = Fixture::complete();
    missing.manifest.sources[0].claims[0].classification =
        OwnerProseClassification::AcceptedCurrent;
    assert!(kinds(&missing).contains(&OwnerProseRefusalKind::ProjectionCountMismatch));

    let mut unavailable = Fixture::complete();
    let selected = &mut unavailable.manifest.sources[0].claims[0];
    selected.classification = OwnerProseClassification::AcceptedCurrent;
    selected.projections.push(OwnerProseProjection {
        path: "policy/core/evaluate/src/missing.rs".into(),
        start: 0,
        end: 1,
        sha256: "00".repeat(32),
        consumer: OwnerProseNativeConsumer::RustCompiler,
    });
    assert!(kinds(&unavailable).contains(&OwnerProseRefusalKind::ProjectionUnavailable));

    let mut mismatch = Fixture::complete();
    let target = "policy/core/evaluate/src/authority.rs";
    mismatch.candidate.insert(target.into(), b"native".to_vec());
    let selected = &mut mismatch.manifest.sources[0].claims[0];
    selected.classification = OwnerProseClassification::AcceptedCurrent;
    selected.projections.push(OwnerProseProjection {
        path: target.into(),
        start: 0,
        end: 6,
        sha256: "00".repeat(32),
        consumer: OwnerProseNativeConsumer::Admission,
    });
    assert!(kinds(&mismatch).contains(&OwnerProseRefusalKind::ProjectionDigestMismatch));

    let mut duplicate = Fixture::complete();
    let bytes = duplicate.source["policy/ADR.md"].clone();
    let split = bytes.len() / 2;
    let target = "policy/core/evaluate/src/authority.rs";
    let native = b"native".to_vec();
    duplicate.candidate.insert(target.into(), native.clone());
    let projection = OwnerProseProjection {
        path: target.into(),
        start: 0,
        end: native.len(),
        sha256: owner_prose_sha256(&native),
        consumer: OwnerProseNativeConsumer::Admission,
    };
    let mut left = claim(
        "left-current",
        &bytes,
        0,
        split,
        OwnerProseClassification::AcceptedCurrent,
    );
    left.projections.push(projection.clone());
    let mut right = claim(
        "right-current",
        &bytes,
        split,
        bytes.len(),
        OwnerProseClassification::AcceptedCurrent,
    );
    right.projections.push(projection);
    set_claims(&mut duplicate, "policy/ADR.md", vec![left, right]);
    assert!(kinds(&duplicate).contains(&OwnerProseRefusalKind::DuplicateProjection));

    let raw = duplicate.manifest_bytes();
    let result = qualify_owner_prose(&raw, &duplicate.observed, |revision, path| {
        if revision == OwnerProseRevision::Candidate && path == target {
            return Err("injected native consumer failure".to_owned());
        }
        Ok(match revision {
            OwnerProseRevision::Source => duplicate.source.get(path).cloned(),
            OwnerProseRevision::Candidate => duplicate.candidate.get(path).cloned(),
        })
    });
    assert!(matches!(
        result,
        OwnerProseQualification::Unknown(ref refusals)
            if refusals.iter().any(|refusal| refusal.kind == OwnerProseRefusalKind::RepositoryReadFailed)
    ));
}

#[test]
fn each_retained_owner_law_file_makes_atomic_deletion_unknown() {
    for path in [
        "policy/ADR.md",
        "policy/PLAN.md",
        "policy/PRD.md",
        "policy/SPEC.md",
    ] {
        let mut fixture = Fixture::complete();
        fixture
            .candidate
            .insert(path.to_owned(), fixture.source[path].clone());
        let result = fixture.qualify();
        let OwnerProseQualification::Unknown(refusals) = result else {
            panic!("retained {path} unexpectedly qualified");
        };
        assert!(refusals.iter().any(|refusal| {
            refusal.kind == OwnerProseRefusalKind::AtomicDeletionIncomplete
                && refusal.subject == path
        }));
    }
}

#[test]
fn strict_schema_missing_source_and_invalid_projection_target_are_unknown() {
    let fixture = Fixture::complete();
    let mut value = serde_json::to_value(&fixture.manifest).expect("manifest value");
    value
        .as_object_mut()
        .expect("manifest object")
        .insert("unclassified".into(), serde_json::Value::Bool(true));
    let raw = serde_json::to_vec(&value).expect("manifest bytes");
    let result = qualify_owner_prose(&raw, &fixture.observed, |revision, path| {
        Ok(match revision {
            OwnerProseRevision::Source => fixture.source.get(path).cloned(),
            OwnerProseRevision::Candidate => fixture.candidate.get(path).cloned(),
        })
    });
    assert!(matches!(
        result,
        OwnerProseQualification::Unknown(ref refusals)
            if refusals[0].kind == OwnerProseRefusalKind::ManifestInvalid
    ));

    let mut missing = Fixture::complete();
    missing.manifest.sources.pop();
    assert!(kinds(&missing).contains(&OwnerProseRefusalKind::SourceSetMismatch));

    let mut invalid = Fixture::complete();
    invalid.manifest.sources[0].claims[0].classification =
        OwnerProseClassification::AcceptedCurrent;
    invalid.manifest.sources[0].claims[0]
        .projections
        .push(OwnerProseProjection {
            path: "docs/current.md".into(),
            start: 0,
            end: 1,
            sha256: "00".repeat(32),
            consumer: OwnerProseNativeConsumer::RustCompiler,
        });
    assert!(kinds(&invalid).contains(&OwnerProseRefusalKind::ProjectionTargetInvalid));
}

#[test]
fn empty_classification_invalid_owner_and_consumer_path_are_unknown() {
    let mut empty = Fixture::complete();
    empty.manifest.sources[0].claims.clear();
    assert!(kinds(&empty).contains(&OwnerProseRefusalKind::ClaimCoverageMismatch));

    let mut invalid_id = Fixture::complete();
    invalid_id.manifest.sources[0].claims[0].id = "ADR-1".to_owned();
    assert!(kinds(&invalid_id).contains(&OwnerProseRefusalKind::ClaimIdentityInvalid));

    let mut owner = Fixture::complete();
    owner.manifest.owner = "app".to_owned();
    assert!(kinds(&owner).contains(&OwnerProseRefusalKind::OwnerInvalid));

    let mut proposal = Fixture::complete();
    proposal.manifest.sources[0].claims[0].classification = OwnerProseClassification::ProposalWork;
    assert!(kinds(&proposal).contains(&OwnerProseRefusalKind::WorkReferenceInvalid));

    let mut consumer = Fixture::complete();
    let target = "policy/core/evaluate/src/NotCargo.toml";
    let native = b"native".to_vec();
    consumer.candidate.insert(target.to_owned(), native.clone());
    let selected = &mut consumer.manifest.sources[0].claims[0];
    selected.classification = OwnerProseClassification::AcceptedCurrent;
    selected.projections.push(OwnerProseProjection {
        path: target.to_owned(),
        start: 0,
        end: native.len(),
        sha256: owner_prose_sha256(&native),
        consumer: OwnerProseNativeConsumer::Cargo,
    });
    assert!(kinds(&consumer).contains(&OwnerProseRefusalKind::ProjectionTargetInvalid));
}
