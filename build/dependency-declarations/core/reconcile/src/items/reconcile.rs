use dependency_declarations_generation::{GenerationPort, RenderedDeclarationProjectionPort};
use dependency_declarations_publication::{PublicationCapabilityPort, PublicationPort};

/// Runs the pure two-attempt generation, projection, and optional publication transition.
pub fn reconcile<G, V, P>(
    request: &ReconciliationRequestV1,
    generator: &G,
    projector: &V,
    publisher: &P,
) -> ReconciliationResultV1
where
    G: for<'a> GenerationPort<GenerationInvocationV1<'a>, RawGenerationV1, GenerationPortErrorV1>,
    V: RenderedDeclarationProjectionPort<
            Projection = ParsedBuckProjectionV1,
            Error = ProjectionPortErrorV1,
        >,
    P: PublicationPort<PublicationRequestV1, PublicationObservationV1, PublicationPortErrorV1>,
    P: PublicationCapabilityPort<PublisherProfileV1>,
{
    let request_id = request.generation.request_id();
    if request
        .publish
        .as_ref()
        .is_some_and(|intent| !publisher.supports(&intent.publisher))
    {
        return refused(
            Some(request_id),
            FailureV1::new(FailureClassV1::UnsupportedPublicationProfile),
        );
    }
    let first_invocation =
        GenerationInvocationV1::new(&request.generation, GenerationAttemptV1::First);
    let first_raw = match generator.generate(&first_invocation) {
        Ok(value) => value,
        Err(error) => return refused(Some(request_id), error.failure()),
    };
    let first = match validate_raw_generation(&first_invocation, first_raw) {
        Ok(value) => value,
        Err(failure) => return refused(Some(request_id), failure),
    };

    let second_invocation =
        GenerationInvocationV1::new(&request.generation, GenerationAttemptV1::Second);
    let second_raw = match generator.generate(&second_invocation) {
        Ok(value) => value,
        Err(error) => return refused(Some(request_id), error.failure()),
    };
    let second = match validate_raw_generation(&second_invocation, second_raw) {
        Ok(value) => value,
        Err(failure) => return refused(Some(request_id), failure),
    };
    if let Err(failure) = compare_generations(&first, &second) {
        return refused(Some(request_id), failure);
    }

    let projection = match projector.project(&first.bytes) {
        Ok(value) => value,
        Err(error) => return refused(Some(request_id), error.failure()),
    };
    if let Err(failure) = validate_projection(&request.generation, &first, &projection) {
        return refused(Some(request_id), failure);
    }

    let output_length_bytes = match checked_u64(first.bytes.len(), internal_invariant()) {
        Ok(value) => value,
        Err(failure) => return refused(Some(request_id), failure),
    };
    let generation_id = generation_identity(
        &request.generation,
        &first,
        &projection,
        output_length_bytes,
    );
    let generation = ValidatedGenerationV1 {
        request_id,
        generation_id,
        output_sha256: first.output_sha256,
        output_length_bytes,
        provider_graph_sha256: first.provider_graph_sha256,
        graph_sha256: first.graph_sha256,
        graph: first.graph,
        bytes: first.bytes,
        validator: request.generation.validator(),
        attempts: [first.attempt_receipt_sha256, second.attempt_receipt_sha256],
        projection_receipt: projection.receipt_sha256,
    };

    let Some(intent) = request.publish.clone() else {
        return ReconciliationResultV1::Generated { generation };
    };
    let publication = PublicationRequestV1::new(generation, intent);
    let observation = match publisher.publish(&publication) {
        Ok(value) => value,
        Err(never) => match never {},
    };
    let (generation, intent) = publication.into_parts();
    let outcome = if valid_publication_outcome(&observation.outcome) {
        observation.outcome
    } else {
        PublicationOutcomeV1::Indeterminate {
            failure: internal_invariant(),
            replacement: ReplacementStateV1::Maybe,
            durability: DurabilityStateV1::Unknown,
        }
    };
    let attempt = publication_receipt(&generation, intent, outcome);
    ReconciliationResultV1::Published {
        generation,
        attempt,
    }
}

fn validate_raw_generation(
    invocation: &GenerationInvocationV1<'_>,
    raw: RawGenerationV1,
) -> Result<AdmittedGenerationV1, FailureV1> {
    if raw.artifact_transport.len() > ValidationBoundsV1::MAX_PROVIDER_TRANSPORT_BYTES
        || raw.stderr.len() > ValidationBoundsV1::MAX_STDERR_BYTES
    {
        return Err(FailureV1::new(FailureClassV1::GeneratorOutputTooLarge));
    }
    let artifact = decode_provider_artifact_v1(&raw.artifact_transport)?;
    let expected_invocation_id = invocation.invocation_id().to_string();
    if artifact.invocation_id != expected_invocation_id {
        return Err(invalid_graph());
    }
    let expected_provider_receipt =
        provider_artifact_receipt_v1(artifact.invocation_id, artifact.graph, artifact.rendered)?;
    if artifact.receipt_sha256 != expected_provider_receipt {
        return Err(invalid_graph());
    }
    let graph = decode_provider_graph_v1(invocation.request(), artifact.graph)?;
    let provider_graph_sha256 = DigestV1::of(artifact.graph);
    let graph_sha256 = graph.sha256();
    let output_sha256 = DigestV1::of(artifact.rendered);
    let provider_graph_length = checked_u64(artifact.graph.len(), internal_invariant())?;
    let output_length = checked_u64(artifact.rendered.len(), internal_invariant())?;
    let attempt_receipt_sha256 = generation_attempt_receipt(&GenerationAttemptEvidenceV1 {
        invocation_id: invocation.invocation_id(),
        provider_receipt_sha256: artifact.receipt_sha256,
        provider_graph_sha256,
        provider_graph_length,
        graph_sha256,
        graph_length: graph.encoded_length_bytes(),
        output_sha256,
        output_length,
    });
    Ok(AdmittedGenerationV1 {
        provider_graph_bytes: artifact.graph.into(),
        provider_graph_sha256,
        graph,
        graph_sha256,
        bytes: artifact.rendered.into(),
        output_sha256,
        provider_receipt_sha256: artifact.receipt_sha256,
        attempt_receipt_sha256,
    })
}

fn compare_generations(
    first: &AdmittedGenerationV1,
    second: &AdmittedGenerationV1,
) -> Result<(), FailureV1> {
    if first.output_sha256 == second.output_sha256 && first.bytes != second.bytes {
        return Err(internal_invariant());
    }
    if first.graph_sha256 == second.graph_sha256 && first.graph != second.graph {
        return Err(internal_invariant());
    }
    if first.provider_graph_sha256 == second.provider_graph_sha256
        && first.provider_graph_bytes != second.provider_graph_bytes
    {
        return Err(internal_invariant());
    }
    if first.bytes != second.bytes
        || first.provider_graph_bytes != second.provider_graph_bytes
        || first.graph != second.graph
    {
        return Err(FailureV1::new(FailureClassV1::NondeterministicOutput));
    }
    if first.provider_receipt_sha256 == second.provider_receipt_sha256
        || first.attempt_receipt_sha256 == second.attempt_receipt_sha256
    {
        return Err(internal_invariant());
    }
    Ok(())
}

fn validate_projection(
    request: &GenerationRequestV1,
    generated: &AdmittedGenerationV1,
    projection: &ParsedBuckProjectionV1,
) -> Result<(), FailureV1> {
    let mut receipt = CanonicalHasherV1::new(b"build.declaration-projection.v1\0");
    receipt.digest(projection.parser_identity);
    receipt.digest(projection.graph.sha256());
    receipt.digest(DigestV1::of(&generated.bytes));
    if projection.parser_identity != request.parser_identity()
        || projection.graph_sha256 != projection.graph.sha256()
        || projection.output_sha256 != generated.output_sha256
        || projection.receipt_sha256 != receipt.finish()
    {
        return Err(internal_invariant());
    }
    if generated.graph_sha256 == projection.graph_sha256 && generated.graph != projection.graph {
        return Err(internal_invariant());
    }
    if generated.graph != projection.graph {
        return Err(invalid_graph());
    }
    Ok(())
}

fn valid_publication_outcome(outcome: &PublicationOutcomeV1) -> bool {
    match outcome {
        PublicationOutcomeV1::Unchanged | PublicationOutcomeV1::Replaced => true,
        PublicationOutcomeV1::Failed {
            failure,
            replacement,
        } => {
            failure.class().is_publication_attempt_failure()
                && *replacement == ReplacementStateV1::No
        }
        PublicationOutcomeV1::Indeterminate {
            failure,
            replacement,
            durability,
        } => {
            failure.class().is_publication_attempt_failure()
                && *replacement == ReplacementStateV1::Maybe
                && *durability == DurabilityStateV1::Unknown
        }
    }
}

fn refused(request_id: Option<DigestV1>, failure: FailureV1) -> ReconciliationResultV1 {
    ReconciliationResultV1::Refused {
        request_id,
        failure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_claimed_digest_never_replaces_exact_byte_comparison() {
        let graph = RuleGraphV1::try_new(Vec::new(), Vec::new()).unwrap();
        let digest = DigestV1::of(b"claimed");
        let raw = |bytes: &[u8]| AdmittedGenerationV1 {
            provider_graph_bytes: Box::new([]),
            provider_graph_sha256: digest,
            graph: graph.clone(),
            graph_sha256: graph.sha256(),
            bytes: bytes.into(),
            output_sha256: digest,
            provider_receipt_sha256: digest,
            attempt_receipt_sha256: digest,
        };

        assert_eq!(
            compare_generations(&raw(b"first"), &raw(b"second")),
            Err(internal_invariant())
        );
    }
}
