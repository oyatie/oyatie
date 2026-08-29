mod support;

use dependency_declarations_generation::GenerationPort;
use dependency_declarations_reconcile::*;

use support::{
    FixedProjection, RecordingPublisher, ScriptedGenerator, graph, valid_generation_request,
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
            GenerationPortErrorV1::InternalInvariant,
            FailureClassV1::InternalInvariant,
        ),
    ];
    for (port_error, expected) in cases {
        let request = valid_generation_request(false);
        let generator = ScriptedGenerator::new(vec![Err(port_error)]);
        let parser = FixedProjection::new(graph("demo"), request.parser_identity());
        let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);
        let result = reconcile(
            &ReconciliationRequestV1::new(request, None),
            &generator,
            &parser,
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
            let parser = FixedProjection::new(graph("demo"), request.parser_identity());
            let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                reconcile(
                    &ReconciliationRequestV1::new(request, None),
                    &generator,
                    &parser,
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

impl<'a> GenerationPort<GenerationInvocationV1<'a>, RawGenerationV1, GenerationPortErrorV1>
    for RawTransportGenerator
{
    fn generate(
        &self,
        _request: &GenerationInvocationV1<'a>,
    ) -> Result<RawGenerationV1, GenerationPortErrorV1> {
        Ok(RawGenerationV1::unverified_provider_artifact(
            self.transport.clone(),
            Vec::new(),
        ))
    }
}
