/// One of the two required clean generation attempts.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum GenerationAttemptV1 {
    First = 0,
    Second = 1,
}

/// One deterministic generation invocation identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenerationInvocationV1<'a> {
    request_id: DigestV1,
    invocation_id: DigestV1,
    attempt: GenerationAttemptV1,
    request: &'a GenerationRequestV1,
}

impl<'a> GenerationInvocationV1<'a> {
    pub(crate) fn new(request: &'a GenerationRequestV1, attempt: GenerationAttemptV1) -> Self {
        let request_id = request.request_id();
        let mut hash = CanonicalHasherV1::new(b"build.declaration-invocation.v1\0");
        hash.digest(request_id);
        hash.tag(attempt as u8);
        Self {
            request_id,
            invocation_id: hash.finish(),
            attempt,
            request,
        }
    }

    /// Returns the identity that the adapter must echo in its artifact.
    #[must_use]
    pub const fn invocation_id(&self) -> DigestV1 {
        self.invocation_id
    }

    /// Returns the generation request identity carried into the provider run.
    #[must_use]
    pub const fn request_id(&self) -> DigestV1 {
        self.request_id
    }

    /// Returns which of the two clean-root attempts this invocation represents.
    #[must_use]
    pub const fn attempt(&self) -> GenerationAttemptV1 {
        self.attempt
    }

    /// Returns the complete immutable request without copying its input bytes.
    #[must_use]
    pub const fn request(&self) -> &GenerationRequestV1 {
        self.request
    }
}

/// Untrusted generator output returned across the generation port.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawGenerationV1 {
    pub(crate) artifact_transport: Box<[u8]>,
    pub(crate) stderr: Box<[u8]>,
    pub(crate) execution: GenerationExecutionObservationV1,
}

impl RawGenerationV1 {
    /// Carries exact untrusted producer stdout into the pure validator.
    #[must_use]
    pub fn unverified_provider_artifact(
        artifact_transport: Vec<u8>,
        stderr: Vec<u8>,
        execution: GenerationExecutionObservationV1,
    ) -> Self {
        Self {
            artifact_transport: artifact_transport.into_boxed_slice(),
            stderr: stderr.into_boxed_slice(),
            execution,
        }
    }
}

pub(crate) struct AdmittedGenerationV1 {
    pub(crate) provider_graph_bytes: Box<[u8]>,
    pub(crate) provider_graph_sha256: DigestV1,
    pub(crate) graph: RuleGraphV1,
    pub(crate) graph_sha256: DigestV1,
    pub(crate) bytes: Box<[u8]>,
    pub(crate) output_sha256: DigestV1,
    pub(crate) provider_receipt_sha256: DigestV1,
    pub(crate) execution_fingerprint_sha256: DigestV1,
    pub(crate) execution_receipt_sha256: DigestV1,
    pub(crate) attempt_receipt_sha256: DigestV1,
}

/// Untrusted independent parser projection returned across its port.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedBuckProjectionV1 {
    pub(crate) profile_sha256: DigestV1,
    pub(crate) graph: RenderedRuleGraphV1,
    pub(crate) graph_sha256: DigestV1,
    pub(crate) output_sha256: DigestV1,
    pub(crate) receipt_sha256: DigestV1,
}

impl ParsedBuckProjectionV1 {
    /// Builds a self-consistent projection receipt over exact rendered bytes.
    pub fn for_projection(
        profile_sha256: DigestV1,
        graph: RenderedRuleGraphV1,
        rendered_buck: &[u8],
    ) -> Self {
        let graph_sha256 = graph.sha256();
        let output_sha256 = DigestV1::of(rendered_buck);
        let mut hash = CanonicalHasherV1::new(b"build.declaration-projection.v1\0");
        hash.digest(profile_sha256);
        hash.digest(graph_sha256);
        hash.digest(output_sha256);
        Self {
            profile_sha256,
            graph,
            graph_sha256,
            output_sha256,
            receipt_sha256: hash.finish(),
        }
    }
}

/// One graph-and-byte generation admitted by both independent proofs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedGenerationV1 {
    pub(crate) request_id: DigestV1,
    pub(crate) generation_id: DigestV1,
    pub(crate) output_sha256: DigestV1,
    pub(crate) output_length_bytes: u64,
    pub(crate) provider_graph_sha256: DigestV1,
    pub(crate) graph_sha256: DigestV1,
    pub(crate) execution_fingerprint_sha256: DigestV1,
    pub(crate) graph: RuleGraphV1,
    pub(crate) bytes: Box<[u8]>,
    pub(crate) validator: ValidatorProfileV1,
    pub(crate) attempts: [DigestV1; 2],
    pub(crate) projection_receipt: DigestV1,
}

impl ValidatedGenerationV1 {
    /// Returns the stable generation identity.
    #[must_use]
    pub const fn generation_id(&self) -> DigestV1 {
        self.generation_id
    }

    /// Returns distinct receipts for the two independent invocations.
    #[must_use]
    pub const fn attempts(&self) -> &[DigestV1; 2] {
        &self.attempts
    }

    /// Returns the exact validated output bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub(crate) struct GenerationAttemptEvidenceV1 {
    pub(crate) invocation_id: DigestV1,
    pub(crate) provider_receipt_sha256: DigestV1,
    pub(crate) execution_fingerprint_sha256: DigestV1,
    pub(crate) execution_receipt_sha256: DigestV1,
    pub(crate) provider_graph_sha256: DigestV1,
    pub(crate) provider_graph_length: u64,
    pub(crate) graph_sha256: DigestV1,
    pub(crate) graph_length: u64,
    pub(crate) output_sha256: DigestV1,
    pub(crate) output_length: u64,
}

pub(crate) fn generation_attempt_receipt(evidence: &GenerationAttemptEvidenceV1) -> DigestV1 {
    let mut hash = CanonicalHasherV1::new(b"build.declaration-attempt.v1\0");
    hash.digest(evidence.invocation_id);
    hash.digest(evidence.provider_receipt_sha256);
    hash.digest(evidence.execution_fingerprint_sha256);
    hash.digest(evidence.execution_receipt_sha256);
    hash.digest(evidence.provider_graph_sha256);
    hash.u64(evidence.provider_graph_length);
    hash.digest(evidence.graph_sha256);
    hash.u64(evidence.graph_length);
    hash.digest(evidence.output_sha256);
    hash.u64(evidence.output_length);
    hash.finish()
}
