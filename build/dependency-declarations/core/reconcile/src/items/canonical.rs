use std::error::Error;
use std::fmt;

use sha2::{Digest as _, Sha256};

/// A SHA-256 digest whose byte representation is part of the v1 contract.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DigestV1([u8; 32]);

impl DigestV1 {
    /// Hashes the exact bytes without text normalization or ambient metadata.
    #[must_use]
    pub fn sha256(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    /// Returns the 32 raw digest bytes.
    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl fmt::Display for DigestV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("sha256:")?;
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// The generator-owned typed graph surface consumed by the pure Build core.
///
/// The concrete graph and its full-field encoder remain in the patched
/// producer. This trait does not authorize reconstructing a graph from BUCK
/// text or supplying a caller-authored expected graph.
pub trait ProducerRuleGraphV1 {
    /// Returns the producer's bounded canonical encoding of every graph field.
    fn canonical_full_field_bytes(&self) -> &[u8];
}

/// One producer invocation's graph and bytes rendered from that graph.
#[derive(Debug)]
pub struct GeneratedArtifactObservationV1<G> {
    graph: G,
    rendered_buck: Box<[u8]>,
}

impl<G> GeneratedArtifactObservationV1<G> {
    /// Records the single artifact returned by one producer invocation.
    pub fn new<B>(graph: G, rendered_buck: B) -> Self
    where
        B: Into<Box<[u8]>>,
    {
        Self {
            graph,
            rendered_buck: rendered_buck.into(),
        }
    }

    /// Borrows the producer-owned typed graph without replacing its equality.
    #[must_use]
    pub const fn graph(&self) -> &G {
        &self.graph
    }

    /// Borrows the exact rendered BUCK bytes from the same invocation.
    #[must_use]
    pub fn rendered_buck(&self) -> &[u8] {
        &self.rendered_buck
    }
}

/// The surface on which two clean producer runs disagreed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationDeterminismSurfaceV1 {
    /// The exact rendered BUCK bytes differ.
    RenderedBuck,
    /// The producer's complete typed graph encoding differs.
    ProducerGraph,
}

/// A fail-closed comparison error for two producer runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GenerationDeterminismErrorV1 {
    /// The exact rendered BUCK bytes differ.
    RenderedBuckMismatch,
    /// A full graph field differs even if producer sort keys compare equal.
    ProducerGraphMismatch,
    /// A length cannot be represented by the v1 unsigned 64-bit contract.
    LengthOverflow(GenerationDeterminismSurfaceV1),
}

impl fmt::Display for GenerationDeterminismErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RenderedBuckMismatch => formatter.write_str("rendered BUCK bytes differ"),
            Self::ProducerGraphMismatch => formatter.write_str("producer rule graph fields differ"),
            Self::LengthOverflow(surface) => {
                write!(formatter, "{surface:?} length exceeds the v1 bound")
            }
        }
    }
}

impl Error for GenerationDeterminismErrorV1 {}

/// Evidence that two observed runs have identical bytes and complete graphs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TwoRunGenerationProofV1 {
    rendered_buck_sha256: DigestV1,
    rendered_buck_length_bytes: u64,
    producer_graph_sha256: DigestV1,
    producer_graph_length_bytes: u64,
}

impl TwoRunGenerationProofV1 {
    /// Returns the digest of the exact rendered BUCK bytes.
    #[must_use]
    pub const fn rendered_buck_sha256(self) -> DigestV1 {
        self.rendered_buck_sha256
    }

    /// Returns the exact rendered BUCK length.
    #[must_use]
    pub const fn rendered_buck_length_bytes(self) -> u64 {
        self.rendered_buck_length_bytes
    }

    /// Returns the digest of the producer's complete canonical graph bytes.
    #[must_use]
    pub const fn producer_graph_sha256(self) -> DigestV1 {
        self.producer_graph_sha256
    }

    /// Returns the complete canonical graph encoding length.
    #[must_use]
    pub const fn producer_graph_length_bytes(self) -> u64 {
        self.producer_graph_length_bytes
    }
}

/// Compares two independent producer observations without using graph
/// `PartialEq`, digests as a byte substitute, or rendered-text reconstruction.
pub fn compare_generation_runs<G>(
    left: &GeneratedArtifactObservationV1<G>,
    right: &GeneratedArtifactObservationV1<G>,
) -> Result<TwoRunGenerationProofV1, GenerationDeterminismErrorV1>
where
    G: ProducerRuleGraphV1,
{
    if left.rendered_buck != right.rendered_buck {
        return Err(GenerationDeterminismErrorV1::RenderedBuckMismatch);
    }

    let left_graph = left.graph.canonical_full_field_bytes();
    let right_graph = right.graph.canonical_full_field_bytes();
    if left_graph != right_graph {
        return Err(GenerationDeterminismErrorV1::ProducerGraphMismatch);
    }

    let rendered_buck_length_bytes = u64::try_from(left.rendered_buck.len()).map_err(|_| {
        GenerationDeterminismErrorV1::LengthOverflow(GenerationDeterminismSurfaceV1::RenderedBuck)
    })?;
    let producer_graph_length_bytes = u64::try_from(left_graph.len()).map_err(|_| {
        GenerationDeterminismErrorV1::LengthOverflow(GenerationDeterminismSurfaceV1::ProducerGraph)
    })?;

    Ok(TwoRunGenerationProofV1 {
        rendered_buck_sha256: DigestV1::sha256(&left.rendered_buck),
        rendered_buck_length_bytes,
        producer_graph_sha256: DigestV1::sha256(left_graph),
        producer_graph_length_bytes,
    })
}
