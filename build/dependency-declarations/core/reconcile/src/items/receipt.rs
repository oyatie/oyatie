fn generation_identity(
    request: &GenerationRequestV1,
    generated: &AdmittedGenerationV1,
    projection: &ParsedBuckProjectionV1,
    consumer_qualification: &BuckConsumerQualificationObservationV1,
    output_length_bytes: u64,
) -> DigestV1 {
    let mut hash = CanonicalHasherV1::new(b"build.declaration-generation.v1\0");
    hash.digest(request.request_id());
    hash.digest(generated.output_sha256);
    hash.u64(output_length_bytes);
    hash.digest(generated.provider_graph_sha256);
    hash.digest(generated.graph_sha256);
    hash.digest(generated.execution_fingerprint_sha256);
    hash.digest(projection.receipt_sha256);
    hash.digest(consumer_qualification.fingerprint_sha256());
    hash.tag(match request.validator() {
        ValidatorProfileV1::ReindeerBuckV1 => 0,
    });
    hash.finish()
}

fn publication_receipt(
    generation: &ValidatedGenerationV1,
    intent: PublicationIntentV1,
    outcome: PublicationOutcomeV1,
) -> PublicationAttemptReceiptV1 {
    let mut hash = CanonicalHasherV1::new(b"build.declaration-publication.v1\0");
    hash.digest(generation.generation_id);
    match intent.expected_preimage {
        Some(value) => {
            hash.tag(1);
            hash.digest(value);
        }
        None => hash.tag(0),
    }
    hash.tag(intent.publisher as u8);
    encode_publication_outcome(&outcome, &mut hash);
    PublicationAttemptReceiptV1 {
        attempt_id: hash.finish(),
        generation_id: generation.generation_id,
        expected_preimage: intent.expected_preimage,
        publisher: intent.publisher,
        outcome,
    }
}

fn encode_publication_outcome(outcome: &PublicationOutcomeV1, hash: &mut CanonicalHasherV1) {
    match outcome {
        PublicationOutcomeV1::Unchanged => hash.tag(0),
        PublicationOutcomeV1::Replaced => hash.tag(1),
        PublicationOutcomeV1::Failed { failure, .. } => {
            hash.tag(2);
            hash.tag(failure.class().tag());
            hash.tag(0);
        }
        PublicationOutcomeV1::Indeterminate { failure, .. } => {
            hash.tag(3);
            hash.tag(failure.class().tag());
            hash.tag(1);
            hash.tag(0);
        }
    }
}
