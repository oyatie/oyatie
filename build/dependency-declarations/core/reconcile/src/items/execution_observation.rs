/// Qualified sandbox evidence for one completed generation attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationExecutionObservationV1 {
    request_id: DigestV1,
    invocation_id: DigestV1,
    execution_runtime_sha256: DigestV1,
    repository_reads_sha256: DigestV1,
    cargo_home_reads_sha256: DigestV1,
    observed_reads_sha256: DigestV1,
    observed_writes_sha256: DigestV1,
    attestation_sha256: DigestV1,
    access_fingerprint_sha256: DigestV1,
    receipt_sha256: DigestV1,
}

impl GenerationExecutionObservationV1 {
    /// Binds a qualified complete-access attestation to one invocation.
    #[must_use]
    pub fn completed(
        invocation: &GenerationInvocationV1<'_>,
        observed_reads_sha256: DigestV1,
        observed_writes_sha256: DigestV1,
        attestation_sha256: DigestV1,
    ) -> Self {
        let request = invocation.request();
        let execution_runtime_sha256 = request
            .tools()
            .execution_runtime()
            .identity_sha256();
        let repository_reads_sha256 = request.inputs().repository_reads().root_sha256();
        let cargo_home_reads_sha256 = request.inputs().cargo_home_reads().root_sha256();
        let access_fingerprint_sha256 = execution_access_fingerprint(
            request.request_id(),
            execution_runtime_sha256,
            repository_reads_sha256,
            cargo_home_reads_sha256,
            observed_reads_sha256,
            observed_writes_sha256,
        );
        let receipt_sha256 = execution_observation_receipt(
            invocation.invocation_id(),
            access_fingerprint_sha256,
            attestation_sha256,
        );
        Self {
            request_id: request.request_id(),
            invocation_id: invocation.invocation_id(),
            execution_runtime_sha256,
            repository_reads_sha256,
            cargo_home_reads_sha256,
            observed_reads_sha256,
            observed_writes_sha256,
            attestation_sha256,
            access_fingerprint_sha256,
            receipt_sha256,
        }
    }

    /// Returns the stable declared-plus-observed access identity.
    #[must_use]
    pub const fn access_fingerprint_sha256(&self) -> DigestV1 {
        self.access_fingerprint_sha256
    }

    /// Returns the invocation-specific sandbox receipt.
    #[must_use]
    pub const fn receipt_sha256(&self) -> DigestV1 {
        self.receipt_sha256
    }
}

pub(crate) fn validate_execution_observation(
    invocation: &GenerationInvocationV1<'_>,
    observation: &GenerationExecutionObservationV1,
) -> Result<(), FailureV1> {
    let request = invocation.request();
    let expected_fingerprint = execution_access_fingerprint(
        request.request_id(),
        request
            .tools()
            .execution_runtime()
            .identity_sha256(),
        request.inputs().repository_reads().root_sha256(),
        request.inputs().cargo_home_reads().root_sha256(),
        observation.observed_reads_sha256,
        observation.observed_writes_sha256,
    );
    let expected_receipt = execution_observation_receipt(
        invocation.invocation_id(),
        expected_fingerprint,
        observation.attestation_sha256,
    );
    if observation.request_id != request.request_id()
        || observation.invocation_id != invocation.invocation_id()
        || observation.execution_runtime_sha256
            != request
                .tools()
                .execution_runtime()
                .identity_sha256()
        || observation.repository_reads_sha256
            != request.inputs().repository_reads().root_sha256()
        || observation.cargo_home_reads_sha256
            != request.inputs().cargo_home_reads().root_sha256()
        || observation.access_fingerprint_sha256 != expected_fingerprint
        || observation.receipt_sha256 != expected_receipt
    {
        return Err(FailureV1::new(
            FailureClassV1::InvalidExecutionEvidence,
        ));
    }
    Ok(())
}

fn execution_access_fingerprint(
    request_id: DigestV1,
    execution_runtime_sha256: DigestV1,
    repository_reads_sha256: DigestV1,
    cargo_home_reads_sha256: DigestV1,
    observed_reads_sha256: DigestV1,
    observed_writes_sha256: DigestV1,
) -> DigestV1 {
    let mut hash = CanonicalHasherV1::new(b"build.generation-access-fingerprint.v1\0");
    hash.digest(request_id);
    hash.digest(execution_runtime_sha256);
    hash.digest(repository_reads_sha256);
    hash.digest(cargo_home_reads_sha256);
    hash.digest(observed_reads_sha256);
    hash.digest(observed_writes_sha256);
    hash.finish()
}

fn execution_observation_receipt(
    invocation_id: DigestV1,
    access_fingerprint_sha256: DigestV1,
    attestation_sha256: DigestV1,
) -> DigestV1 {
    let mut hash = CanonicalHasherV1::new(b"build.generation-execution-observation.v1\0");
    hash.digest(invocation_id);
    hash.digest(access_fingerprint_sha256);
    hash.digest(attestation_sha256);
    hash.finish()
}
