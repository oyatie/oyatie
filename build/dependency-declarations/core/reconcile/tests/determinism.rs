mod support;

use dependency_declarations_reconcile::*;

use support::{
    FixedProjection, RecordingPublisher, ScriptedGenerator, graph, graph_with_fragment, rendered,
    rendered_fragment, valid_generation_request,
};

#[test]
fn two_independent_invocations_produce_one_validated_generation() {
    let request = valid_generation_request(false);
    let parser_identity = request.parser_identity();
    let graph_value = graph("demo");
    let generator = ScriptedGenerator::with_stderr(
        vec![
            Ok((graph_value.clone(), rendered("demo"))),
            Ok((graph_value.clone(), rendered("demo"))),
        ],
        b"first bounded diagnostic".to_vec(),
    );
    let projection = FixedProjection::new(graph_value, parser_identity);
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

    let result = reconcile(
        &ReconciliationRequestV1::new(request.clone(), None),
        &generator,
        &projection,
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
    assert_eq!(publisher.calls(), 0);

    let replay_generator = ScriptedGenerator::with_stderr(
        vec![
            Ok((graph("demo"), rendered("demo"))),
            Ok((graph("demo"), rendered("demo"))),
        ],
        b"different bounded diagnostic".to_vec(),
    );
    let replay_projection = FixedProjection::new(graph("demo"), parser_identity);
    let replay = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &replay_generator,
        &replay_projection,
        &publisher,
    );
    let ReconciliationResultV1::Generated {
        generation: replay_generation,
    } = replay
    else {
        panic!("expected a replayed generation");
    };
    assert_eq!(generation, replay_generation);
}

#[test]
fn byte_or_full_graph_disagreement_refuses_before_projection() {
    let request = valid_generation_request(false);
    let parser = FixedProjection::new(graph("demo"), request.parser_identity());
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
        &publisher,
    );
    assert_refusal(result, FailureClassV1::NondeterministicOutput);
    assert_eq!(parser.calls(), 0);
}

fn assert_refusal(result: ReconciliationResultV1, expected: FailureClassV1) {
    let ReconciliationResultV1::Refused { failure, .. } = result else {
        panic!("expected refusal");
    };
    assert_eq!(failure.class(), expected);
}
