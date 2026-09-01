mod support;

use dependency_declarations_reconcile::*;

use support::{
    FixedBuckConsumer, FixedProjection, RecordingPublisher, ScriptedGenerator,
    generation_request_with_manifest, graph, graph_with_fragment, rendered, rendered_fragment,
    valid_generation_request,
};

#[test]
fn two_independent_invocations_produce_one_validated_generation() {
    let request = valid_generation_request(false);
    let projection_profile = request.projection_profile_sha256();
    let graph_value = graph("demo");
    let generator = ScriptedGenerator::with_stderr(
        vec![
            Ok((graph_value.clone(), rendered("demo"))),
            Ok((graph_value.clone(), rendered("demo"))),
        ],
        b"first bounded diagnostic".to_vec(),
    );
    let projection = FixedProjection::new(graph_value, projection_profile);
    let consumer = FixedBuckConsumer::new();
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

    let result = reconcile(
        &ReconciliationRequestV1::new(request.clone(), None),
        &generator,
        &projection,
        &consumer,
        &publisher,
    );
    let ReconciliationResultV1::Generated { generation } = result else {
        panic!("expected a validated generation");
    };
    let invocations = generator.invocations();
    assert_eq!(invocations.len(), 2);
    assert_ne!(invocations[0], invocations[1]);
    assert_ne!(generation.attempts()[0], generation.attempts()[1]);
    assert_eq!(projection.calls(), 1);
    assert_eq!(consumer.calls(), 1);
    assert_eq!(publisher.calls(), 0);

    let replay_generator = ScriptedGenerator::with_stderr(
        vec![
            Ok((graph("demo"), rendered("demo"))),
            Ok((graph("demo"), rendered("demo"))),
        ],
        b"different bounded diagnostic".to_vec(),
    );
    let replay_projection = FixedProjection::new(graph("demo"), projection_profile);
    let replay = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &replay_generator,
        &replay_projection,
        &consumer,
        &publisher,
    );
    let ReconciliationResultV1::Generated {
        generation: replay_generation,
    } = replay
    else {
        panic!("expected a replayed generation");
    };
    assert_eq!(
        generation.generation_id(),
        replay_generation.generation_id()
    );
    assert_eq!(generation.request_id(), replay_generation.request_id());
    assert_eq!(generation.bytes(), replay_generation.bytes());
    assert_eq!(generation.graph(), replay_generation.graph());
    assert_eq!(generation.attempts(), replay_generation.attempts());
    assert_eq!(
        generation.execution_fingerprint_sha256(),
        replay_generation.execution_fingerprint_sha256()
    );
    assert_eq!(
        generation.projection_receipt(),
        replay_generation.projection_receipt()
    );
    assert_eq!(
        generation.consumer_qualification_fingerprint(),
        replay_generation.consumer_qualification_fingerprint()
    );
    assert_ne!(
        generation.consumer_qualification_receipt(),
        replay_generation.consumer_qualification_receipt()
    );
}

#[test]
fn byte_or_full_graph_disagreement_refuses_before_projection() {
    let request = valid_generation_request(false);
    let parser = FixedProjection::new(graph("demo"), request.projection_profile_sha256());
    let consumer = FixedBuckConsumer::new();
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);
    let byte_mismatch = ScriptedGenerator::new(vec![
        Ok((
            graph_with_fragment("demo", b"first"),
            rendered_fragment(b"first"),
        )),
        Ok((
            graph_with_fragment("demo", b"second"),
            rendered_fragment(b"second"),
        )),
    ]);
    let result = reconcile(
        &ReconciliationRequestV1::new(request.clone(), None),
        &byte_mismatch,
        &parser,
        &consumer,
        &publisher,
    );
    assert_refusal(result, FailureClassV1::NondeterministicOutput);
    assert_eq!(parser.calls(), 0);

    let graph_mismatch = ScriptedGenerator::new(vec![
        Ok((
            graph_with_fragment("first", b"same"),
            rendered_fragment(b"same"),
        )),
        Ok((
            graph_with_fragment("second", b"same"),
            rendered_fragment(b"same"),
        )),
    ]);
    let result = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &graph_mismatch,
        &parser,
        &consumer,
        &publisher,
    );
    assert_refusal(result, FailureClassV1::NondeterministicOutput);
    assert_eq!(parser.calls(), 0);
    assert_eq!(consumer.calls(), 0);
}

#[test]
fn observed_access_disagreement_refuses_before_projection() {
    let request = valid_generation_request(false);
    let parser = FixedProjection::new(graph("demo"), request.projection_profile_sha256());
    let consumer = FixedBuckConsumer::new();
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);
    let generator = ScriptedGenerator::with_observed_reads(
        vec![
            Ok((graph("demo"), rendered("demo"))),
            Ok((graph("demo"), rendered("demo"))),
        ],
        vec![
            DigestV1::of(b"first observed read set"),
            DigestV1::of(b"second observed read set"),
        ],
    );

    let result = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &generator,
        &parser,
        &consumer,
        &publisher,
    );

    assert_refusal(result, FailureClassV1::NondeterministicExecution);
    assert_eq!(parser.calls(), 0);
    assert_eq!(consumer.calls(), 0);
    assert_eq!(publisher.calls(), 0);
}

#[test]
fn consumer_evidence_cannot_be_replayed_across_generation_requests() {
    let first_request = valid_generation_request(false);
    let second_request = generation_request_with_manifest(b"[workspace]\nmembers = []\n");
    assert_eq!(
        first_request.projection_profile_sha256(),
        second_request.projection_profile_sha256()
    );
    let generator = ScriptedGenerator::new(
        (0..4)
            .map(|_| Ok((graph("demo"), rendered("demo"))))
            .collect(),
    );
    let parser = FixedProjection::new(graph("demo"), first_request.projection_profile_sha256());
    let consumer = FixedBuckConsumer::replaying();
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

    let first = reconcile(
        &ReconciliationRequestV1::new(first_request, None),
        &generator,
        &parser,
        &consumer,
        &publisher,
    );
    assert!(matches!(first, ReconciliationResultV1::Generated { .. }));

    let replay = reconcile(
        &ReconciliationRequestV1::new(second_request, None),
        &generator,
        &parser,
        &consumer,
        &publisher,
    );
    assert_refusal(replay, FailureClassV1::InvalidBuckConsumerEvidence);
    assert_eq!(consumer.calls(), 2);
    assert_eq!(publisher.calls(), 0);
}

fn assert_refusal(result: ReconciliationResultV1, expected: FailureClassV1) {
    let ReconciliationResultV1::Refused { failure, .. } = result else {
        panic!("expected refusal");
    };
    assert_eq!(failure.class(), expected);
}
