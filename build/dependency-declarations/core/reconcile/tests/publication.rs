mod support;

use dependency_declarations_reconcile::*;

use support::{
    FixedProjection, RecordingPublisher, ScriptedGenerator, digest, graph, rendered,
    valid_generation_request,
};

#[test]
fn attempted_publication_gets_an_outcome_bound_receipt() {
    let first = publication_attempt(PublicationOutcomeV1::Replaced);
    let replay = publication_attempt(PublicationOutcomeV1::Replaced);
    let different = publication_attempt(PublicationOutcomeV1::Unchanged);

    assert_eq!(first.attempt_id(), replay.attempt_id());
    assert_ne!(first.attempt_id(), different.attempt_id());
    assert_eq!(first.outcome(), &PublicationOutcomeV1::Replaced);
}

fn publication_attempt(outcome: PublicationOutcomeV1) -> PublicationAttemptReceiptV1 {
    let generation_request = valid_generation_request(false);
    let intent = PublicationIntentV1::new(
        Some(digest(b"old destination")),
        PublisherProfileV1::MacosApfsV1,
    );
    let graph = graph("demo");
    let generator = ScriptedGenerator::new(vec![
        Ok((graph.clone(), rendered("demo"))),
        Ok((graph.clone(), rendered("demo"))),
    ]);
    let parser = FixedProjection::new(graph, generation_request.parser_identity());
    let publisher = RecordingPublisher::new(outcome);

    let result = reconcile(
        &ReconciliationRequestV1::new(generation_request, Some(intent)),
        &generator,
        &parser,
        &publisher,
    );
    let ReconciliationResultV1::Published {
        generation,
        attempt,
    } = result
    else {
        panic!("expected published result");
    };
    assert_eq!(publisher.calls(), 1);
    assert_eq!(attempt.generation_id(), generation.generation_id());
    attempt
}

#[test]
fn impossible_publication_failure_shape_still_gets_an_indeterminate_receipt() {
    let generation_request = valid_generation_request(false);
    let intent = PublicationIntentV1::new(None, PublisherProfileV1::LinuxExt4V1);
    let graph = graph("demo");
    let generator = ScriptedGenerator::new(vec![
        Ok((graph.clone(), rendered("demo"))),
        Ok((graph.clone(), rendered("demo"))),
    ]);
    let parser = FixedProjection::new(graph, generation_request.parser_identity());
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Failed {
        failure: FailureV1::new(FailureClassV1::GeneratorFailed),
        replacement: ReplacementStateV1::No,
    });

    let result = reconcile(
        &ReconciliationRequestV1::new(generation_request, Some(intent)),
        &generator,
        &parser,
        &publisher,
    );
    let ReconciliationResultV1::Published { attempt, .. } = result else {
        panic!("expected an attempted-publication receipt");
    };
    assert!(matches!(
        attempt.outcome(),
        PublicationOutcomeV1::Indeterminate {
            failure,
            replacement: ReplacementStateV1::Maybe,
            durability: DurabilityStateV1::Unknown,
        } if failure.class() == FailureClassV1::InternalInvariant
    ));
}

#[test]
fn unsupported_profile_refuses_before_a_publication_attempt() {
    let generation_request = valid_generation_request(false);
    let intent = PublicationIntentV1::new(None, PublisherProfileV1::LinuxXfsV1);
    let graph = graph("demo");
    let generator = ScriptedGenerator::new(vec![
        Ok((graph.clone(), rendered("demo"))),
        Ok((graph.clone(), rendered("demo"))),
    ]);
    let parser = FixedProjection::new(graph, generation_request.parser_identity());
    let publisher = RecordingPublisher::unsupported(PublicationOutcomeV1::Unchanged);

    let result = reconcile(
        &ReconciliationRequestV1::new(generation_request, Some(intent)),
        &generator,
        &parser,
        &publisher,
    );
    let ReconciliationResultV1::Refused { failure, .. } = result else {
        panic!("expected unsupported-profile refusal");
    };
    assert_eq!(
        failure.class(),
        FailureClassV1::UnsupportedPublicationProfile
    );
    assert!(generator.invocations().is_empty());
    assert_eq!(parser.calls(), 0);
    assert_eq!(publisher.calls(), 0);
}
