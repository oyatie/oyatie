/// A maintained parser's bounded, full-field projection of rendered BUCK.
pub trait ParsedRuleGraphProjectionV1 {
    /// Returns the projection's canonical encoding of every admitted field.
    fn canonical_full_field_bytes(&self) -> &[u8];
}

/// A fail-closed error from independent generated-graph validation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GeneratedGraphValidationErrorV1<E> {
    /// The independent parser refused the rendered BUCK bytes.
    Parser(E),
    /// The parser projection omitted, added, or changed a producer graph field.
    ProjectionMismatch,
}

impl<E> std::fmt::Display for GeneratedGraphValidationErrorV1<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parser(error) => write!(formatter, "independent BUCK parser refused: {error}"),
            Self::ProjectionMismatch => {
                formatter.write_str("parser projection differs from producer rule graph")
            }
        }
    }
}

impl<E> std::error::Error for GeneratedGraphValidationErrorV1<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parser(error) => Some(error),
            Self::ProjectionMismatch => None,
        }
    }
}

/// Evidence that independently parsed bytes project to the producer graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParserRoundTripProofV1 {
    rule_graph_sha256: DigestV1,
    parser_projection_sha256: DigestV1,
}

impl ParserRoundTripProofV1 {
    /// Returns the digest of the producer's complete typed graph encoding.
    #[must_use]
    pub const fn rule_graph_sha256(self) -> DigestV1 {
        self.rule_graph_sha256
    }

    /// Returns the digest of the maintained parser's complete projection.
    #[must_use]
    pub const fn parser_projection_sha256(self) -> DigestV1 {
        self.parser_projection_sha256
    }
}

/// Parses the exact producer-rendered bytes and compares every canonical field.
///
/// The producer graph remains primary. Parser equality is an independent
/// cross-check and is not Buck2 configured-consumer qualification.
pub fn validate_parser_round_trip<G, Projection, ParserError, Parser>(
    artifact: &GeneratedArtifactObservationV1<G>,
    parser: &Parser,
) -> Result<ParserRoundTripProofV1, GeneratedGraphValidationErrorV1<ParserError>>
where
    G: ProducerRuleGraphV1,
    Projection: ParsedRuleGraphProjectionV1,
    Parser: dependency_declarations_generation::GeneratedBuckParserPort<Projection, ParserError>,
{
    let projection = parser
        .parse(artifact.rendered_buck())
        .map_err(GeneratedGraphValidationErrorV1::Parser)?;
    let graph_bytes = artifact.graph().canonical_full_field_bytes();
    let projection_bytes = projection.canonical_full_field_bytes();

    if graph_bytes != projection_bytes {
        return Err(GeneratedGraphValidationErrorV1::ProjectionMismatch);
    }

    Ok(ParserRoundTripProofV1 {
        rule_graph_sha256: DigestV1::sha256(graph_bytes),
        parser_projection_sha256: DigestV1::sha256(projection_bytes),
    })
}
