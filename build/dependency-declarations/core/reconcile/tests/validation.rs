mod support;

use dependency_declarations_reconcile::*;

use support::{
    FixedProjection, ProviderArtifactFaultV1, RecordingPublisher, ScriptedGenerator, digest, graph,
    graph_with_fragment, rendered, rendered_fragment, valid_generation_request,
};

#[test]
fn provider_transport_refuses_wrong_identity_receipt_schema_and_shape() {
    let faults = [
        ProviderArtifactFaultV1::TransportMagic,
        ProviderArtifactFaultV1::TruncatedTransport,
        ProviderArtifactFaultV1::TrailingTransportByte,
        ProviderArtifactFaultV1::OversizedRenderedLength,
        ProviderArtifactFaultV1::Invocation,
        ProviderArtifactFaultV1::Receipt,
        ProviderArtifactFaultV1::SourceRevision,
        ProviderArtifactFaultV1::AdaptationRecipe,
        ProviderArtifactFaultV1::SourceDigest,
        ProviderArtifactFaultV1::SemanticSchemaDigest,
        ProviderArtifactFaultV1::TrailingGraphByte,
        ProviderArtifactFaultV1::NoncontiguousPosition,
        ProviderArtifactFaultV1::UnknownRuleKind,
        ProviderArtifactFaultV1::UnknownSemanticValue,
        ProviderArtifactFaultV1::UnsortedMap,
        ProviderArtifactFaultV1::UnsortedNamedFields,
    ];
    for fault in faults {
        let request = valid_generation_request(false);
        let generator =
            ScriptedGenerator::with_fault(vec![Ok((graph("demo"), rendered("demo")))], fault);
        let projection = FixedProjection::new(graph("demo"), request.parser_identity());
        let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

        let result = reconcile(
            &ReconciliationRequestV1::new(request, None),
            &generator,
            &projection,
            &publisher,
        );
        let ReconciliationResultV1::Refused { failure, .. } = result else {
            panic!("expected provider transport refusal");
        };
        assert_eq!(failure.class(), FailureClassV1::InvalidGeneratedGraph);
        assert_eq!(generator.invocations().len(), 1);
        assert_eq!(projection.calls(), 0);
    }
}

#[test]
fn maintained_projection_must_match_every_graph_field() {
    let request = valid_generation_request(false);
    let generated = graph("demo");
    let generator = ScriptedGenerator::new(vec![
        Ok((generated.clone(), rendered("demo"))),
        Ok((generated, rendered("demo"))),
    ]);
    let projection = FixedProjection::new(graph("different"), request.parser_identity());
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

    let result = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &generator,
        &projection,
        &publisher,
    );
    let ReconciliationResultV1::Refused { failure, .. } = result else {
        panic!("expected parser mismatch refusal");
    };
    assert_eq!(failure.class(), FailureClassV1::InvalidGeneratedGraph);
}

#[test]
fn maintained_projection_fragment_digest_must_bind_rendered_bytes() {
    let request = valid_generation_request(false);
    let generated = graph("demo");
    let generator = ScriptedGenerator::new(vec![
        Ok((generated.clone(), rendered_fragment(b"unrelated"))),
        Ok((generated, rendered_fragment(b"unrelated"))),
    ]);
    let projection = FixedProjection::new(
        graph_with_fragment("demo", b"unrelated"),
        request.parser_identity(),
    );
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

    let result = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &generator,
        &projection,
        &publisher,
    );
    let ReconciliationResultV1::Refused { failure, .. } = result else {
        panic!("expected graph-to-render refusal");
    };
    assert_eq!(failure.class(), FailureClassV1::InvalidGeneratedGraph);
    assert_eq!(generator.invocations().len(), 2);
    assert_eq!(projection.calls(), 1);
}

#[test]
fn duplicate_target_identity_and_noncontiguous_positions_refuse() {
    let semantic = SemanticValueV1::call_named(
        "rust_library",
        vec![("name".to_owned(), SemanticValueV1::string("same").unwrap())],
    )
    .unwrap();
    let first = RuleV1::new(
        0,
        ReindeerRuleKindV1::Library,
        semantic.clone(),
        digest(b"first"),
    );
    let duplicate = RuleV1::new(
        1,
        ReindeerRuleKindV1::Alias,
        semantic.clone(),
        digest(b"second"),
    );
    assert_eq!(
        RuleGraphV1::try_new(Vec::new(), vec![first, duplicate])
            .unwrap_err()
            .class(),
        FailureClassV1::InvalidGeneratedGraph
    );

    let skipped = RuleV1::new(2, ReindeerRuleKindV1::Library, semantic, digest(b"third"));
    assert_eq!(
        RuleGraphV1::try_new(Vec::new(), vec![skipped])
            .unwrap_err()
            .class(),
        FailureClassV1::InvalidGeneratedGraph
    );
}

#[test]
fn semantic_value_bounds_fail_without_panicking() {
    assert_eq!(
        SemanticValueV1::string("x".repeat(ValidationBoundsV1::MAX_STRING_BYTES + 1))
            .unwrap_err()
            .class(),
        FailureClassV1::InvalidGeneratedGraph
    );
    let values = vec![SemanticValueV1::none(); ValidationBoundsV1::MAX_LIST_ENTRIES + 1];
    assert_eq!(
        SemanticValueV1::list(values).unwrap_err().class(),
        FailureClassV1::InvalidGeneratedGraph
    );

    let mut nested = SemanticValueV1::none();
    for _ in 1..ValidationBoundsV1::MAX_VALUE_DEPTH {
        nested = SemanticValueV1::list(vec![nested]).unwrap();
    }
    assert_eq!(
        SemanticValueV1::list(vec![nested]).unwrap_err().class(),
        FailureClassV1::InvalidGeneratedGraph
    );
}

#[test]
fn named_semantic_fields_are_canonical_and_duplicate_free() {
    let value = |fields: &[(&str, &str)]| {
        SemanticValueV1::call_named(
            "rule",
            fields
                .iter()
                .map(|(name, value)| ((*name).to_owned(), SemanticValueV1::string(*value).unwrap()))
                .collect(),
        )
    };
    assert_eq!(
        value(&[("b", "second"), ("a", "first")]).unwrap(),
        value(&[("a", "first"), ("b", "second")]).unwrap()
    );
    assert_eq!(
        value(&[("a", "first"), ("a", "second")])
            .unwrap_err()
            .class(),
        FailureClassV1::InvalidGeneratedGraph
    );
}

#[test]
fn semantic_maps_are_canonical_and_duplicate_free() {
    let entry = |key: &str, value: &str| {
        (
            SemanticValueV1::string(key).unwrap(),
            SemanticValueV1::string(value).unwrap(),
        )
    };
    assert_eq!(
        SemanticValueV1::map(vec![entry("b", "second"), entry("a", "first")]).unwrap(),
        SemanticValueV1::map(vec![entry("a", "first"), entry("b", "second")]).unwrap()
    );
    assert_eq!(
        SemanticValueV1::map(vec![entry("a", "first"), entry("a", "second")])
            .unwrap_err()
            .class(),
        FailureClassV1::InvalidGeneratedGraph
    );
}
