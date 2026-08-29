use dependency_declarations_reconcile::*;

const GRAPH_DOMAIN: &[u8] = b"reindeer.rule-graph.v1\0";
const RECEIPT_DOMAIN: &[u8] = b"reindeer.generated-artifact.v1\0";
const TRANSPORT_MAGIC: &[u8] = b"REINDEER_GENERATED_ARTIFACT_V1\0";

#[derive(Clone, Copy)]
pub enum ProviderArtifactFaultV1 {
    TransportMagic,
    TruncatedTransport,
    TrailingTransportByte,
    OversizedRenderedLength,
    Invocation,
    Receipt,
    SourceRevision,
    AdaptationRecipe,
    SchemaSourceDigest,
    SemanticSchemaDigest,
    TrailingGraphByte,
    NoncontiguousPosition,
    UnknownRuleKind,
    UnknownSemanticValue,
    UnsortedMap,
    UnsortedNamedFields,
}

pub fn raw_provider_artifact(
    invocation: &GenerationInvocationV1<'_>,
    graph: &RuleGraphV1,
    rendered: Vec<u8>,
    stderr: Vec<u8>,
) -> RawGenerationV1 {
    raw_provider_artifact_with_fault(invocation, graph, rendered, stderr, None)
}

pub fn raw_provider_artifact_with_fault(
    invocation: &GenerationInvocationV1<'_>,
    graph: &RuleGraphV1,
    rendered: Vec<u8>,
    stderr: Vec<u8>,
    fault: Option<ProviderArtifactFaultV1>,
) -> RawGenerationV1 {
    let mut graph_bytes = encode_graph(invocation.request(), graph, fault);
    if matches!(fault, Some(ProviderArtifactFaultV1::TrailingGraphByte)) {
        graph_bytes.push(0);
    }
    let invocation_id = if matches!(fault, Some(ProviderArtifactFaultV1::Invocation)) {
        "sha256:wrong-invocation".to_owned()
    } else {
        invocation.invocation_id().to_string()
    };
    let mut receipt = provider_receipt(&invocation_id, &graph_bytes, &rendered);
    if matches!(fault, Some(ProviderArtifactFaultV1::Receipt)) {
        receipt = DigestV1::of(b"wrong provider receipt");
    }
    let mut transport = TRANSPORT_MAGIC.to_vec();
    if matches!(fault, Some(ProviderArtifactFaultV1::TransportMagic)) {
        transport[0] = b'X';
    }
    framed(&mut transport, invocation_id.as_bytes());
    framed(&mut transport, &graph_bytes);
    if matches!(
        fault,
        Some(ProviderArtifactFaultV1::OversizedRenderedLength)
    ) {
        length(&mut transport, ValidationBoundsV1::MAX_OUTPUT_BYTES + 1);
    } else {
        framed(&mut transport, &rendered);
    }
    transport.extend_from_slice(&receipt.bytes());
    if matches!(fault, Some(ProviderArtifactFaultV1::TruncatedTransport)) {
        transport.pop();
    }
    if matches!(fault, Some(ProviderArtifactFaultV1::TrailingTransportByte)) {
        transport.push(0);
    }
    RawGenerationV1::unverified_provider_artifact(transport, stderr)
}

fn encode_graph(
    request: &GenerationRequestV1,
    graph: &RuleGraphV1,
    fault: Option<ProviderArtifactFaultV1>,
) -> Vec<u8> {
    let provider = request.tools().qualification().provider_graph();
    let mut output = GRAPH_DOMAIN.to_vec();
    let source_revision = if matches!(fault, Some(ProviderArtifactFaultV1::SourceRevision)) {
        b"wrong-source-revision".as_slice()
    } else {
        request.tools().generator().source_revision().as_bytes()
    };
    framed(&mut output, source_revision);
    let adaptation_recipe = if matches!(fault, Some(ProviderArtifactFaultV1::AdaptationRecipe)) {
        b"wrong-adaptation-recipe".as_slice()
    } else {
        provider.adaptation_recipe_id().as_bytes()
    };
    framed(&mut output, adaptation_recipe);
    let schema_source_sha256 = if matches!(fault, Some(ProviderArtifactFaultV1::SchemaSourceDigest))
    {
        DigestV1::of(b"wrong provider schema source").bytes()
    } else {
        provider.schema_source_sha256().bytes()
    };
    output.extend_from_slice(&schema_source_sha256);
    let semantic_schema_sha256 =
        if matches!(fault, Some(ProviderArtifactFaultV1::SemanticSchemaDigest)) {
            DigestV1::of(b"wrong semantic schema").bytes()
        } else {
            provider.semantic_schema_sha256().bytes()
        };
    output.extend_from_slice(&semantic_schema_sha256);
    framed(&mut output, graph.prefix());
    length(&mut output, graph.rules().len());
    for (index, rule) in graph.rules().iter().enumerate() {
        let position = if index == 0
            && matches!(fault, Some(ProviderArtifactFaultV1::NoncontiguousPosition))
        {
            1
        } else {
            rule.position()
        };
        output.extend_from_slice(&position.to_be_bytes());
        output.push(
            if index == 0 && matches!(fault, Some(ProviderArtifactFaultV1::UnknownRuleKind)) {
                u8::MAX
            } else {
                rule.kind() as u8
            },
        );
        if index == 0 && matches!(fault, Some(ProviderArtifactFaultV1::UnknownSemanticValue)) {
            output.push(u8::MAX);
        } else if index == 0 && matches!(fault, Some(ProviderArtifactFaultV1::UnsortedMap)) {
            output.push(8);
            length(&mut output, 2);
            tagged_text(&mut output, 4, "b");
            output.push(0);
            tagged_text(&mut output, 4, "a");
            output.push(0);
        } else if index == 0 && matches!(fault, Some(ProviderArtifactFaultV1::UnsortedNamedFields))
        {
            output.push(9);
            framed(&mut output, b"rule");
            output.push(1);
            length(&mut output, 2);
            framed(&mut output, b"b");
            output.push(0);
            framed(&mut output, b"a");
            output.push(0);
        } else {
            encode_value(&mut output, rule.semantic());
        }
        output.extend_from_slice(&rule.rendered_sha256().bytes());
    }
    output
}

fn encode_value(output: &mut Vec<u8>, value: &SemanticValueV1) {
    match value.view() {
        SemanticValueRefV1::None => output.push(0),
        SemanticValueRefV1::Bool(value) => output.extend_from_slice(&[1, u8::from(value)]),
        SemanticValueRefV1::Signed(value) => {
            output.push(2);
            output.extend_from_slice(&value.to_be_bytes());
        }
        SemanticValueRefV1::Unsigned(value) => {
            output.push(3);
            output.extend_from_slice(&value.to_be_bytes());
        }
        SemanticValueRefV1::String(value) => tagged_text(output, 4, value),
        SemanticValueRefV1::Identifier(value) => tagged_text(output, 5, value),
        SemanticValueRefV1::List(values) => encode_sequence(output, 6, values),
        SemanticValueRefV1::Tuple(values) => encode_sequence(output, 7, values),
        SemanticValueRefV1::Map(entries) => {
            output.push(8);
            length(output, entries.len());
            for (key, value) in entries {
                encode_value(output, key);
                encode_value(output, value);
            }
        }
        SemanticValueRefV1::Call { callee, arguments } => {
            output.push(9);
            framed(output, callee.as_bytes());
            match arguments {
                CallArgumentsRefV1::Positional(values) => {
                    output.push(0);
                    length(output, values.len());
                    for value in values {
                        encode_value(output, value);
                    }
                }
                CallArgumentsRefV1::Named(fields) => {
                    output.push(1);
                    length(output, fields.len());
                    for (name, value) in fields {
                        framed(output, name.as_bytes());
                        encode_value(output, value);
                    }
                }
            }
        }
    }
}

fn encode_sequence(output: &mut Vec<u8>, tag: u8, values: &[SemanticValueV1]) {
    output.push(tag);
    length(output, values.len());
    for value in values {
        encode_value(output, value);
    }
}

fn tagged_text(output: &mut Vec<u8>, tag: u8, value: &str) {
    output.push(tag);
    framed(output, value.as_bytes());
}

fn provider_receipt(invocation_id: &str, graph: &[u8], rendered: &[u8]) -> DigestV1 {
    let mut preimage = RECEIPT_DOMAIN.to_vec();
    framed(&mut preimage, invocation_id.as_bytes());
    framed(&mut preimage, graph);
    framed(&mut preimage, rendered);
    DigestV1::of(&preimage)
}

fn framed(output: &mut Vec<u8>, value: &[u8]) {
    length(output, value.len());
    output.extend_from_slice(value);
}

fn length(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&u64::try_from(value).unwrap().to_be_bytes());
}
