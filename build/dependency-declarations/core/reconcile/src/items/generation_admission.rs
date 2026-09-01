fn validate_raw_generation(
    invocation: &GenerationInvocationV1<'_>,
    raw: RawGenerationV1,
) -> Result<AdmittedGenerationV1, FailureV1> {
    if raw.artifact_transport.len() > ValidationBoundsV1::MAX_PROVIDER_TRANSPORT_BYTES
        || raw.stderr.len() > ValidationBoundsV1::MAX_STDERR_BYTES
    {
        return Err(FailureV1::new(FailureClassV1::GeneratorOutputTooLarge));
    }
    validate_execution_observation(invocation, &raw.execution)?;
    let execution_fingerprint_sha256 = raw.execution.access_fingerprint_sha256();
    let execution_receipt_sha256 = raw.execution.receipt_sha256();
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
        execution_fingerprint_sha256,
        execution_receipt_sha256,
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
        execution_fingerprint_sha256,
        execution_receipt_sha256,
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
    if first.execution_fingerprint_sha256 != second.execution_fingerprint_sha256 {
        return Err(FailureV1::new(
            FailureClassV1::NondeterministicExecution,
        ));
    }
    if first.provider_receipt_sha256 == second.provider_receipt_sha256
        || first.execution_receipt_sha256 == second.execution_receipt_sha256
        || first.attempt_receipt_sha256 == second.attempt_receipt_sha256
    {
        return Err(internal_invariant());
    }
    Ok(())
}
