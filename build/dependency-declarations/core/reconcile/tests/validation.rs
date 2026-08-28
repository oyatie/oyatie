use dependency_declarations_generation::GeneratedBuckParserPort;
use dependency_declarations_reconcile::{
    GeneratedArtifactObservationV1, GeneratedGraphValidationErrorV1, ParsedRuleGraphProjectionV1,
    ProducerRuleGraphV1, validate_parser_round_trip,
};

#[derive(Debug)]
struct ProducerGraph(Box<[u8]>);

impl ProducerRuleGraphV1 for ProducerGraph {
    fn canonical_full_field_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug)]
struct ParserProjection(Box<[u8]>);

impl ParsedRuleGraphProjectionV1 for ParserProjection {
    fn canonical_full_field_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FakeParserError {
    WrongInput,
    UnsupportedSyntax,
}

struct FakeParser {
    expected_input: &'static [u8],
    projection: &'static [u8],
    fail: bool,
}

impl GeneratedBuckParserPort<ParserProjection, FakeParserError> for FakeParser {
    fn parse(&self, rendered_buck: &[u8]) -> Result<ParserProjection, FakeParserError> {
        if rendered_buck != self.expected_input {
            return Err(FakeParserError::WrongInput);
        }
        if self.fail {
            return Err(FakeParserError::UnsupportedSyntax);
        }
        Ok(ParserProjection(self.projection.into()))
    }
}

fn artifact(
    graph: &'static [u8],
    rendered_buck: &'static [u8],
) -> GeneratedArtifactObservationV1<ProducerGraph> {
    GeneratedArtifactObservationV1::new(ProducerGraph(graph.into()), rendered_buck)
}

#[test]
fn exact_rendered_bytes_and_matching_projection_validate() {
    let artifact = artifact(b"kind=alias;name=crate", b"alias(name = \"crate\")\n");
    let parser = FakeParser {
        expected_input: b"alias(name = \"crate\")\n",
        projection: b"kind=alias;name=crate",
        fail: false,
    };

    let proof = validate_parser_round_trip(&artifact, &parser)
        .expect("matching independent projection must validate");

    assert_eq!(proof.rule_graph_sha256(), proof.parser_projection_sha256());
}

#[test]
fn lossy_parser_projection_refuses_generated_graph() {
    let artifact = artifact(
        b"kind=alias;name=crate;visibility=private",
        b"alias(name = \"crate\")\n",
    );
    let parser = FakeParser {
        expected_input: b"alias(name = \"crate\")\n",
        projection: b"kind=alias;name=crate",
        fail: false,
    };

    assert_eq!(
        validate_parser_round_trip(&artifact, &parser),
        Err(GeneratedGraphValidationErrorV1::ProjectionMismatch)
    );
}

#[test]
fn parser_refusal_remains_distinct_from_projection_mismatch() {
    let artifact = artifact(b"kind=unknown", b"unknown_rule()\n");
    let parser = FakeParser {
        expected_input: b"unknown_rule()\n",
        projection: b"",
        fail: true,
    };

    assert_eq!(
        validate_parser_round_trip(&artifact, &parser),
        Err(GeneratedGraphValidationErrorV1::Parser(
            FakeParserError::UnsupportedSyntax
        ))
    );
}
