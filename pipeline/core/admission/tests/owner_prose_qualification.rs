//! Typed exact-revision owner-prose qualification controls.

#[path = "support/owner_prose.rs"]
mod support;

use pipeline_admission::{
    OwnerProseClassification, OwnerProseNativeConsumer, OwnerProseProjection,
    OwnerProseQualification, OwnerProseRefusalKind, OwnerProseWorkReference, owner_prose_sha256,
};
use support::Fixture;

fn ready(fixture: &Fixture) -> pipeline_admission::QualifiedOwnerProseView {
    match fixture.qualify() {
        OwnerProseQualification::Ready(view) => *view,
        OwnerProseQualification::Unknown(refusals) => {
            panic!("expected Ready, got {refusals:#?}")
        }
    }
}

fn kinds(fixture: &Fixture) -> Vec<OwnerProseRefusalKind> {
    match fixture.qualify() {
        OwnerProseQualification::Ready(view) => panic!("expected Unknown, got {view:#?}"),
        OwnerProseQualification::Unknown(refusals) => {
            refusals.into_iter().map(|refusal| refusal.kind).collect()
        }
    }
}

#[test]
fn exact_complete_input_is_deterministically_ready() {
    let fixture = Fixture::complete();
    let first = ready(&fixture);
    let second = ready(&fixture);
    assert_eq!(first, second);
    assert_eq!(first.schema(), "oyatie.owner-prose-qualified-view.v1");
    assert_eq!(first.repository(), &fixture.observed);
    assert_eq!(first.qualifier().identity, "pipeline-owner-prose-qualifier");
    assert_eq!(first.source_digests().len(), 4);
    assert!(first.candidate_digests().is_empty());
    assert_eq!(first.claims().len(), 4);
    assert_eq!(
        serde_json::to_vec(&first).expect("serialize first"),
        serde_json::to_vec(&second).expect("serialize second")
    );
}

#[test]
fn accepted_current_fact_projects_once_into_native_authority() {
    let mut fixture = Fixture::complete();
    let native_path = "policy/core/evaluate/src/authority.rs";
    let native = b"pub const DEFAULT_DENY: bool = true;\n".to_vec();
    fixture
        .candidate
        .insert(native_path.to_owned(), native.clone());
    let source = fixture
        .manifest
        .sources
        .iter_mut()
        .find(|source| source.path == "policy/ADR.md")
        .expect("ADR source");
    source.claims[0].classification = OwnerProseClassification::AcceptedCurrent;
    source.claims[0].projections = vec![OwnerProseProjection {
        path: native_path.to_owned(),
        start: 0,
        end: native.len(),
        sha256: owner_prose_sha256(&native),
        consumer: OwnerProseNativeConsumer::RustCompiler,
    }];

    let view = ready(&fixture);
    assert_eq!(view.candidate_digests().len(), 1);
    assert_eq!(view.candidate_digests()[0].path, native_path);
}

#[test]
fn proposal_work_is_retained_by_one_bound_external_record() {
    let mut fixture = Fixture::complete();
    let claim = &mut fixture.manifest.sources[0].claims[0];
    claim.classification = OwnerProseClassification::ProposalWork;
    claim.work_reference = Some(OwnerProseWorkReference {
        system: "github-pr".to_owned(),
        locator: "https://github.com/oyatie/oyatie/pull/2400".to_owned(),
    });
    let view = ready(&fixture);
    assert_eq!(
        view.claims()
            .iter()
            .filter(|claim| claim.work_reference.is_some())
            .count(),
        1
    );
}

#[test]
fn exact_revision_and_source_digest_mismatches_refuse_unknown() {
    let mut revision = Fixture::complete();
    revision.manifest.repository.source.tree = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into();
    assert!(kinds(&revision).contains(&OwnerProseRefusalKind::RepositoryBindingMismatch));

    let mut digest = Fixture::complete();
    digest.manifest.sources[0].sha256 = "00".repeat(32);
    assert!(kinds(&digest).contains(&OwnerProseRefusalKind::SourceDigestMismatch));

    let mut producer = Fixture::complete();
    producer.manifest.producer.identity = "untrusted-classifier".to_owned();
    assert!(kinds(&producer).contains(&OwnerProseRefusalKind::ProducerInvalid));
}

#[test]
fn unavailable_source_and_unknown_classification_refuse_unknown() {
    let mut missing = Fixture::complete();
    missing.source.remove("policy/PLAN.md");
    assert!(kinds(&missing).contains(&OwnerProseRefusalKind::SourceUnavailable));

    let mut unknown = Fixture::complete();
    unknown.manifest.sources[0].claims[0].classification = OwnerProseClassification::Unknown;
    assert!(kinds(&unknown).contains(&OwnerProseRefusalKind::UnknownClassification));
}
