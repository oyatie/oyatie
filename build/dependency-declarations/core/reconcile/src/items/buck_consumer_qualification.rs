/// One whole-artifact Buck consumer qualification invocation.
#[derive(Clone, Copy, Debug)]
pub struct BuckConsumerQualificationInvocationV1<'a> {
    request: &'a GenerationRequestV1,
    rendered_buck: &'a [u8],
    invocation_id: DigestV1,
    output_sha256: DigestV1,
    provider_graph_sha256: DigestV1,
    projection_receipt_sha256: DigestV1,
}

impl<'a> BuckConsumerQualificationInvocationV1<'a> {
    fn new(
        request: &'a GenerationRequestV1,
        generated: &'a AdmittedGenerationV1,
        projection: &ParsedBuckProjectionV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.buck-consumer-invocation.v1\0");
        hash.digest(request.request_id());
        hash.digest(request.tools().qualification().buck_consumer().identity_sha256());
        hash.digest(generated.output_sha256);
        hash.digest(generated.provider_graph_sha256);
        hash.digest(projection.receipt_sha256);
        Self {
            request,
            rendered_buck: &generated.bytes,
            invocation_id: hash.finish(),
            output_sha256: generated.output_sha256,
            provider_graph_sha256: generated.provider_graph_sha256,
            projection_receipt_sha256: projection.receipt_sha256,
        }
    }

    #[must_use]
    pub const fn request(&self) -> &GenerationRequestV1 {
        self.request
    }

    #[must_use]
    pub const fn rendered_buck(&self) -> &[u8] {
        self.rendered_buck
    }

    #[must_use]
    pub const fn invocation_id(&self) -> DigestV1 {
        self.invocation_id
    }

    #[must_use]
    pub const fn output_sha256(&self) -> DigestV1 {
        self.output_sha256
    }
}

/// Provider-produced evidence from configured queries and representative consumption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuckConsumerQualificationObservationV1 {
    request_id: DigestV1,
    invocation_id: DigestV1,
    profile_sha256: DigestV1,
    output_sha256: DigestV1,
    provider_graph_sha256: DigestV1,
    projection_receipt_sha256: DigestV1,
    query_result_sha256: DigestV1,
    consumption_result_sha256: DigestV1,
    execution_receipt_sha256: DigestV1,
    fingerprint_sha256: DigestV1,
    receipt_sha256: DigestV1,
}

impl BuckConsumerQualificationObservationV1 {
    /// Binds canonical query and consumption results to one execution receipt.
    #[must_use]
    pub fn completed(
        invocation: &BuckConsumerQualificationInvocationV1<'_>,
        query_result_sha256: DigestV1,
        consumption_result_sha256: DigestV1,
        execution_receipt_sha256: DigestV1,
    ) -> Self {
        let request_id = invocation.request.request_id();
        let profile_sha256 = invocation
            .request
            .tools()
            .qualification()
            .buck_consumer()
            .identity_sha256();
        let fingerprint_sha256 = buck_consumer_qualification_fingerprint(
            invocation,
            query_result_sha256,
            consumption_result_sha256,
        );
        let receipt_sha256 = buck_consumer_qualification_receipt(
            invocation,
            fingerprint_sha256,
            execution_receipt_sha256,
        );
        Self {
            request_id,
            invocation_id: invocation.invocation_id,
            profile_sha256,
            output_sha256: invocation.output_sha256,
            provider_graph_sha256: invocation.provider_graph_sha256,
            projection_receipt_sha256: invocation.projection_receipt_sha256,
            query_result_sha256,
            consumption_result_sha256,
            execution_receipt_sha256,
            fingerprint_sha256,
            receipt_sha256,
        }
    }

    /// Returns the deterministic configured-query and consumption-result identity.
    #[must_use]
    pub const fn fingerprint_sha256(&self) -> DigestV1 {
        self.fingerprint_sha256
    }

    #[must_use]
    pub const fn receipt_sha256(&self) -> DigestV1 {
        self.receipt_sha256
    }
}

fn validate_buck_consumer_qualification(
    invocation: &BuckConsumerQualificationInvocationV1<'_>,
    observation: &BuckConsumerQualificationObservationV1,
) -> Result<(), FailureV1> {
    let profile_sha256 = invocation
        .request
        .tools()
        .qualification()
        .buck_consumer()
        .identity_sha256();
    let fingerprint_sha256 = buck_consumer_qualification_fingerprint(
        invocation,
        observation.query_result_sha256,
        observation.consumption_result_sha256,
    );
    let receipt_sha256 = buck_consumer_qualification_receipt(
        invocation,
        fingerprint_sha256,
        observation.execution_receipt_sha256,
    );
    if observation.request_id != invocation.request.request_id()
        || observation.invocation_id != invocation.invocation_id
        || observation.profile_sha256 != profile_sha256
        || observation.output_sha256 != invocation.output_sha256
        || observation.provider_graph_sha256 != invocation.provider_graph_sha256
        || observation.projection_receipt_sha256 != invocation.projection_receipt_sha256
        || observation.fingerprint_sha256 != fingerprint_sha256
        || observation.receipt_sha256 != receipt_sha256
    {
        return Err(FailureV1::new(
            FailureClassV1::InvalidBuckConsumerEvidence,
        ));
    }
    Ok(())
}

fn buck_consumer_qualification_fingerprint(
    invocation: &BuckConsumerQualificationInvocationV1<'_>,
    query_result_sha256: DigestV1,
    consumption_result_sha256: DigestV1,
) -> DigestV1 {
    let mut hash = CanonicalHasherV1::new(b"build.buck-consumer-fingerprint.v1\0");
    hash.digest(invocation.invocation_id);
    hash.digest(query_result_sha256);
    hash.digest(consumption_result_sha256);
    hash.finish()
}

fn buck_consumer_qualification_receipt(
    invocation: &BuckConsumerQualificationInvocationV1<'_>,
    fingerprint_sha256: DigestV1,
    execution_receipt_sha256: DigestV1,
) -> DigestV1 {
    let mut hash = CanonicalHasherV1::new(b"build.buck-consumer-qualification.v1\0");
    hash.digest(invocation.invocation_id);
    hash.digest(fingerprint_sha256);
    hash.digest(execution_receipt_sha256);
    hash.finish()
}
