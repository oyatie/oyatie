mod support;

use dependency_declarations_reconcile::*;

use support::{
    FixedProjection, RecordingPublisher, ScriptedGenerator, graph, rendered,
    valid_generation_request,
};

#[test]
fn exact_observed_bytes_are_current_and_replay_identically() {
    let generation = validated_generation();
    let observation = GeneratedArtifactObservationV1::try_present(
        generation.request_id(),
        CanonicalPathV1::try_new("third-party/BUCK").unwrap(),
        generation.bytes().to_vec(),
    )
    .unwrap();

    let first = assess_generated_artifact_freshness(&generation, &observation).unwrap();
    let second = assess_generated_artifact_freshness(&generation, &observation).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.state(), GeneratedArtifactFreshnessStateV1::Current);
    assert_eq!(first.request_id(), generation.request_id());
    assert_eq!(first.generation_id(), generation.generation_id());
    assert_eq!(first.path().as_str(), "third-party/BUCK");
    assert_eq!(
        first.expected_length_bytes(),
        generation.output_length_bytes()
    );
    assert_eq!(first.expected_sha256(), generation.output_sha256());
    assert_eq!(first.observed_sha256(), Some(generation.output_sha256()));
    assert_eq!(
        first.observed_length_bytes(),
        Some(generation.output_length_bytes())
    );

    let other_path = GeneratedArtifactObservationV1::try_present(
        generation.request_id(),
        CanonicalPathV1::try_new("qualification/BUCK").unwrap(),
        generation.bytes().to_vec(),
    )
    .unwrap();
    let other_path = assess_generated_artifact_freshness(&generation, &other_path).unwrap();
    assert_eq!(
        other_path.state(),
        GeneratedArtifactFreshnessStateV1::Current
    );
    assert_ne!(first.receipt_sha256(), other_path.receipt_sha256());
}

#[test]
fn changed_and_missing_artifacts_are_distinct_stale_states() {
    let generation = validated_generation();
    let path = CanonicalPathV1::try_new("third-party/BUCK").unwrap();
    let changed = GeneratedArtifactObservationV1::try_present(
        generation.request_id(),
        path.clone(),
        b"changed\n".to_vec(),
    )
    .unwrap();
    let missing = GeneratedArtifactObservationV1::absent(generation.request_id(), path);

    let changed = assess_generated_artifact_freshness(&generation, &changed).unwrap();
    let missing = assess_generated_artifact_freshness(&generation, &missing).unwrap();

    assert_eq!(changed.state(), GeneratedArtifactFreshnessStateV1::Drifted);
    assert_eq!(missing.state(), GeneratedArtifactFreshnessStateV1::Missing);
    assert!(changed.observed_sha256().is_some());
    assert_eq!(missing.observed_sha256(), None);
    assert_ne!(changed.receipt_sha256(), missing.receipt_sha256());
}

#[test]
fn observation_from_another_request_refuses_as_changed_input() {
    let generation = validated_generation();
    let observation = GeneratedArtifactObservationV1::absent(
        DigestV1::of(b"another request"),
        CanonicalPathV1::try_new("third-party/BUCK").unwrap(),
    );

    let failure = assess_generated_artifact_freshness(&generation, &observation).unwrap_err();

    assert_eq!(failure.class(), FailureClassV1::InputChanged);
}

#[test]
fn observation_is_bounded_before_it_can_be_assessed() {
    let generation = validated_generation();
    let failure = GeneratedArtifactObservationV1::try_present(
        generation.request_id(),
        CanonicalPathV1::try_new("third-party/BUCK").unwrap(),
        vec![0; ValidationBoundsV1::MAX_OUTPUT_BYTES + 1],
    )
    .unwrap_err();

    assert_eq!(failure.class(), FailureClassV1::InvalidRequest);
}

fn validated_generation() -> ValidatedGenerationV1 {
    let request = valid_generation_request(false);
    let generator = ScriptedGenerator::new(vec![
        Ok((graph("demo"), rendered("demo"))),
        Ok((graph("demo"), rendered("demo"))),
    ]);
    let projection = FixedProjection::new(graph("demo"), request.parser_identity());
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);
    let result = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &generator,
        &projection,
        &publisher,
    );
    let ReconciliationResultV1::Generated { generation } = result else {
        panic!("expected validated generation");
    };
    generation
}
