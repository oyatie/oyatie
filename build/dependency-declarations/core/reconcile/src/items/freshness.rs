#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PresentGeneratedArtifactV1 {
    bytes: Box<[u8]>,
    length_bytes: u64,
    sha256: DigestV1,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum GeneratedArtifactContentV1 {
    Missing,
    Present(PresentGeneratedArtifactV1),
}

/// Exact observed state of one generated artifact at the generation request snapshot.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GeneratedArtifactObservationV1 {
    request_id: DigestV1,
    path: CanonicalPathV1,
    content: GeneratedArtifactContentV1,
}

impl GeneratedArtifactObservationV1 {
    /// Captures bounded bytes and computes their identity instead of trusting metadata.
    pub fn try_present(
        request_id: DigestV1,
        path: CanonicalPathV1,
        bytes: Vec<u8>,
    ) -> Result<Self, FailureV1> {
        if bytes.len() > ValidationBoundsV1::MAX_OUTPUT_BYTES {
            return Err(invalid_request());
        }
        let length_bytes = checked_u64(bytes.len(), invalid_request())?;
        let sha256 = DigestV1::of(&bytes);
        Ok(Self {
            request_id,
            path,
            content: GeneratedArtifactContentV1::Present(PresentGeneratedArtifactV1 {
                bytes: bytes.into_boxed_slice(),
                length_bytes,
                sha256,
            }),
        })
    }

    /// Records that the exact snapshot contains no artifact at the validated path.
    #[must_use]
    pub const fn absent(request_id: DigestV1, path: CanonicalPathV1) -> Self {
        Self {
            request_id,
            path,
            content: GeneratedArtifactContentV1::Missing,
        }
    }
}

/// Content freshness relative to one fully validated generation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum GeneratedArtifactFreshnessStateV1 {
    Current = 0,
    Drifted = 1,
    Missing = 2,
}

/// Identity-bearing, consumer-neutral result for one generated artifact.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GeneratedArtifactFreshnessV1 {
    receipt_sha256: DigestV1,
    request_id: DigestV1,
    generation_id: DigestV1,
    path: CanonicalPathV1,
    expected_length_bytes: u64,
    expected_sha256: DigestV1,
    observed_length_bytes: Option<u64>,
    observed_sha256: Option<DigestV1>,
    state: GeneratedArtifactFreshnessStateV1,
}

impl GeneratedArtifactFreshnessV1 {
    #[must_use]
    pub const fn receipt_sha256(&self) -> DigestV1 {
        self.receipt_sha256
    }

    #[must_use]
    pub const fn request_id(&self) -> DigestV1 {
        self.request_id
    }

    #[must_use]
    pub const fn generation_id(&self) -> DigestV1 {
        self.generation_id
    }

    #[must_use]
    pub const fn path(&self) -> &CanonicalPathV1 {
        &self.path
    }

    #[must_use]
    pub const fn expected_length_bytes(&self) -> u64 {
        self.expected_length_bytes
    }

    #[must_use]
    pub const fn expected_sha256(&self) -> DigestV1 {
        self.expected_sha256
    }

    #[must_use]
    pub const fn observed_length_bytes(&self) -> Option<u64> {
        self.observed_length_bytes
    }

    #[must_use]
    pub const fn observed_sha256(&self) -> Option<DigestV1> {
        self.observed_sha256
    }

    #[must_use]
    pub const fn state(&self) -> GeneratedArtifactFreshnessStateV1 {
        self.state
    }
}

/// Compares one exact snapshot observation with one validated generation.
pub fn assess_generated_artifact_freshness(
    generation: &ValidatedGenerationV1,
    observation: &GeneratedArtifactObservationV1,
) -> Result<GeneratedArtifactFreshnessV1, FailureV1> {
    if observation.request_id != generation.request_id {
        return Err(FailureV1::new(FailureClassV1::InputChanged));
    }
    let actual_length = checked_u64(generation.bytes.len(), internal_invariant())?;
    if actual_length != generation.output_length_bytes
        || DigestV1::of(&generation.bytes) != generation.output_sha256
    {
        return Err(internal_invariant());
    }

    let (state, observed_length_bytes, observed_sha256) = match &observation.content {
        GeneratedArtifactContentV1::Missing => {
            (GeneratedArtifactFreshnessStateV1::Missing, None, None)
        }
        GeneratedArtifactContentV1::Present(observed) => {
            if observed.sha256 == generation.output_sha256 && observed.bytes != generation.bytes {
                return Err(internal_invariant());
            }
            let state = if observed.bytes == generation.bytes {
                GeneratedArtifactFreshnessStateV1::Current
            } else {
                GeneratedArtifactFreshnessStateV1::Drifted
            };
            (state, Some(observed.length_bytes), Some(observed.sha256))
        }
    };

    let mut hash = CanonicalHasherV1::new(b"build.generated-artifact-freshness.v1\0");
    hash.digest(generation.request_id);
    hash.digest(generation.generation_id);
    hash.string(observation.path.as_str())?;
    hash.u64(generation.output_length_bytes);
    hash.digest(generation.output_sha256);
    match (observed_length_bytes, observed_sha256) {
        (Some(length), Some(digest)) => {
            hash.tag(1);
            hash.u64(length);
            hash.digest(digest);
        }
        (None, None) => hash.tag(0),
        _ => return Err(internal_invariant()),
    }
    hash.tag(state as u8);

    Ok(GeneratedArtifactFreshnessV1 {
        receipt_sha256: hash.finish(),
        request_id: generation.request_id,
        generation_id: generation.generation_id,
        path: observation.path.clone(),
        expected_length_bytes: generation.output_length_bytes,
        expected_sha256: generation.output_sha256,
        observed_length_bytes,
        observed_sha256,
        state,
    })
}

#[cfg(test)]
mod freshness_tests {
    use super::*;

    #[test]
    fn claimed_digest_never_replaces_observed_byte_equality() {
        let bytes: Box<[u8]> = b"expected".as_slice().into();
        let output_sha256 = DigestV1::of(&bytes);
        let graph = RuleGraphV1::try_new(Vec::new(), Vec::new()).unwrap();
        let request_id = DigestV1::of(b"request");
        let generation = ValidatedGenerationV1 {
            request_id,
            generation_id: DigestV1::of(b"generation"),
            output_sha256,
            output_length_bytes: u64::try_from(bytes.len()).unwrap(),
            provider_graph_sha256: DigestV1::of(b"provider graph"),
            graph_sha256: graph.sha256(),
            execution_fingerprint_sha256: DigestV1::of(b"execution"),
            graph,
            bytes,
            validator: ValidatorProfileV1::ReindeerBuckV1,
            attempts: [DigestV1::of(b"first"), DigestV1::of(b"second")],
            projection_receipt: DigestV1::of(b"projection"),
        };
        let observation = GeneratedArtifactObservationV1 {
            request_id,
            path: CanonicalPathV1::try_new("third-party/BUCK").unwrap(),
            content: GeneratedArtifactContentV1::Present(PresentGeneratedArtifactV1 {
                bytes: b"different".as_slice().into(),
                length_bytes: 9,
                sha256: output_sha256,
            }),
        };

        let failure =
            assess_generated_artifact_freshness(&generation, &observation).unwrap_err();

        assert_eq!(failure.class(), FailureClassV1::InternalInvariant);
    }
}
