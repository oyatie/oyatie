mod support;

use dependency_declarations_generation::{DeclarationProviderCapabilityPort, GenerationPort};
use dependency_declarations_reconcile::*;

use support::{
    FixedBuckConsumer, FixedProjection, ProjectionProfileVariation, RecordingPublisher,
    ScriptedGenerator, generation_request_with_projection_variation,
    generation_request_with_provider_profile, graph, rendered, valid_generation_request,
};

#[test]
fn generation_port_failures_map_to_stable_failure_classes() {
    let cases = [
        (
            GenerationPortErrorV1::InputChanged,
            FailureClassV1::InputChanged,
        ),
        (
            GenerationPortErrorV1::MissingFixup,
            FailureClassV1::MissingFixup,
        ),
        (
            GenerationPortErrorV1::GeneratorUnavailable,
            FailureClassV1::GeneratorUnavailable,
        ),
        (
            GenerationPortErrorV1::GeneratorFailed,
            FailureClassV1::GeneratorFailed,
        ),
        (
            GenerationPortErrorV1::GeneratorTimedOut,
            FailureClassV1::GeneratorTimedOut,
        ),
        (
            GenerationPortErrorV1::GeneratorOutputTooLarge,
            FailureClassV1::GeneratorOutputTooLarge,
        ),
        (
            GenerationPortErrorV1::UndeclaredAccess,
            FailureClassV1::UndeclaredGenerationAccess,
        ),
        (
            GenerationPortErrorV1::InternalInvariant,
            FailureClassV1::InternalInvariant,
        ),
    ];
    for (port_error, expected) in cases {
        let request = valid_generation_request(false);
        let generator = ScriptedGenerator::new(vec![Err(port_error)]);
        let parser = FixedProjection::new(graph("demo"), request.projection_profile_sha256());
        let consumer = FixedBuckConsumer::new();
        let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);
        let result = reconcile(
            &ReconciliationRequestV1::new(request, None),
            &generator,
            &parser,
            &consumer,
            &publisher,
        );
        let ReconciliationResultV1::Refused {
            request_id,
            failure,
        } = result
        else {
            panic!("expected refusal");
        };
        assert!(request_id.is_some());
        assert_eq!(failure.class(), expected);
    }
}

#[test]
fn unsupported_generation_provider_refuses_before_every_effect() {
    let request = valid_generation_request(false);
    let generator = ScriptedGenerator::unsupported(vec![
        Ok((graph("demo"), rendered("demo"))),
        Ok((graph("demo"), rendered("demo"))),
    ]);
    let parser = FixedProjection::new(graph("demo"), request.projection_profile_sha256());
    let consumer = FixedBuckConsumer::new();
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

    let result = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &generator,
        &parser,
        &consumer,
        &publisher,
    );

    assert_refusal(result, FailureClassV1::UnsupportedGenerationProfile);
    assert!(generator.invocations().is_empty());
    assert_eq!(parser.calls(), 0);
    assert_eq!(consumer.calls(), 0);
    assert_eq!(publisher.calls(), 0);
}

#[test]
fn changed_grammar_or_provider_profile_refuses_before_every_effect() {
    let admitted_profile = valid_generation_request(false).projection_profile_sha256();
    let changed = [
        generation_request_with_projection_variation(ProjectionProfileVariation::Grammar),
        generation_request_with_provider_profile(
            "oyatie.reindeer.changed-recipe.v1",
            b"provider source",
            b"graph schema",
        ),
    ];
    for request in changed {
        let generator = ScriptedGenerator::new(vec![
            Ok((graph("demo"), rendered("demo"))),
            Ok((graph("demo"), rendered("demo"))),
        ]);
        let parser = FixedProjection::new(graph("demo"), admitted_profile);
        let consumer = FixedBuckConsumer::new();
        let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

        let result = reconcile(
            &ReconciliationRequestV1::new(request, None),
            &generator,
            &parser,
            &consumer,
            &publisher,
        );

        assert_refusal(result, FailureClassV1::UnsupportedProjectionProfile);
        assert!(generator.invocations().is_empty());
        assert_eq!(parser.calls(), 0);
        assert_eq!(consumer.calls(), 0);
        assert_eq!(publisher.calls(), 0);
    }
}

#[test]
fn unsupported_buck_consumer_refuses_before_every_effect() {
    let request = valid_generation_request(false);
    let generator = ScriptedGenerator::new(vec![
        Ok((graph("demo"), rendered("demo"))),
        Ok((graph("demo"), rendered("demo"))),
    ]);
    let parser = FixedProjection::new(graph("demo"), request.projection_profile_sha256());
    let consumer = FixedBuckConsumer::unsupported();
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

    let result = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &generator,
        &parser,
        &consumer,
        &publisher,
    );

    assert_refusal(result, FailureClassV1::UnsupportedBuckConsumerProfile);
    assert!(generator.invocations().is_empty());
    assert_eq!(parser.calls(), 0);
    assert_eq!(consumer.calls(), 0);
    assert_eq!(publisher.calls(), 0);
}

#[test]
fn buck_consumer_failures_map_to_stable_failure_classes() {
    let cases = [
        (
            BuckConsumerPortErrorV1::Unavailable,
            FailureClassV1::BuckConsumerUnavailable,
        ),
        (
            BuckConsumerPortErrorV1::QueryFailed,
            FailureClassV1::BuckConsumerQueryFailed,
        ),
        (
            BuckConsumerPortErrorV1::ConsumptionFailed,
            FailureClassV1::BuckConsumerConsumptionFailed,
        ),
        (
            BuckConsumerPortErrorV1::TimedOut,
            FailureClassV1::BuckConsumerTimedOut,
        ),
        (
            BuckConsumerPortErrorV1::OutputTooLarge,
            FailureClassV1::BuckConsumerOutputTooLarge,
        ),
        (
            BuckConsumerPortErrorV1::InternalInvariant,
            FailureClassV1::InternalInvariant,
        ),
    ];
    for (port_error, expected) in cases {
        let request = valid_generation_request(false);
        let generator = ScriptedGenerator::new(vec![
            Ok((graph("demo"), rendered("demo"))),
            Ok((graph("demo"), rendered("demo"))),
        ]);
        let parser = FixedProjection::new(graph("demo"), request.projection_profile_sha256());
        let consumer = FixedBuckConsumer::failing(port_error);
        let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

        let result = reconcile(
            &ReconciliationRequestV1::new(request, None),
            &generator,
            &parser,
            &consumer,
            &publisher,
        );

        assert_refusal(result, expected);
        assert_eq!(generator.invocations().len(), 2);
        assert_eq!(parser.calls(), 1);
        assert_eq!(consumer.calls(), 1);
        assert_eq!(publisher.calls(), 0);
    }
}

#[test]
fn arbitrary_unicode_path_inputs_are_total() {
    for value in ["é", "规则/Δ.rs", "💾/crate.toml", "nul\0inside"] {
        let result = std::panic::catch_unwind(|| CanonicalPathV1::try_new(value));
        assert!(result.is_ok());
    }
}

#[test]
fn bounded_arbitrary_provider_bytes_refuse_without_panicking() {
    for seed in 0_u8..16 {
        for length in 0..256 {
            let transport = (0..length)
                .map(|index| seed.wrapping_add((index as u8).wrapping_mul(31)))
                .collect();
            let request = valid_generation_request(false);
            let generator = RawTransportGenerator { transport };
            let parser = FixedProjection::new(graph("demo"), request.projection_profile_sha256());
            let consumer = FixedBuckConsumer::new();
            let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                reconcile(
                    &ReconciliationRequestV1::new(request, None),
                    &generator,
                    &parser,
                    &consumer,
                    &publisher,
                )
            }));
            let ReconciliationResultV1::Refused { failure, .. } = result.unwrap() else {
                panic!("arbitrary provider bytes must refuse");
            };
            assert_eq!(failure.class(), FailureClassV1::InvalidGeneratedGraph);
        }
    }
}

struct RawTransportGenerator {
    transport: Vec<u8>,
}

fn assert_refusal(result: ReconciliationResultV1, expected: FailureClassV1) {
    let ReconciliationResultV1::Refused { failure, .. } = result else {
        panic!("expected refusal");
    };
    assert_eq!(failure.class(), expected);
}

impl<'a> GenerationPort<GenerationInvocationV1<'a>, RawGenerationV1, GenerationPortErrorV1>
    for RawTransportGenerator
{
    fn generate(
        &self,
        request: &GenerationInvocationV1<'a>,
    ) -> Result<RawGenerationV1, GenerationPortErrorV1> {
        let execution = GenerationExecutionObservationV1::completed(
            request,
            DigestV1::of(b"observed reads"),
            DigestV1::of(b"observed writes"),
            request.invocation_id(),
        );
        Ok(RawGenerationV1::unverified_provider_artifact(
            self.transport.clone(),
            Vec::new(),
            execution,
        ))
    }
}

impl DeclarationProviderCapabilityPort<GenerationRequestV1> for RawTransportGenerator {
    fn supports(&self, _profile: &GenerationRequestV1) -> bool {
        true
    }
}
