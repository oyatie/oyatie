#[path = "validation/semantic_maps.rs"]
mod semantic_maps;
mod support;

use dependency_declarations_reconcile::*;

use support::{
    FixedBuckConsumer, FixedProjection, ProjectionProfileVariation, ProviderArtifactFaultV1,
    RecordingPublisher, ScriptedGenerator, digest, generation_request_with_projection_variation,
    generation_request_with_provider_profile, graph, graph_with_fragment, rendered,
    rendered_fragment, valid_generation_request,
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
        ProviderArtifactFaultV1::SchemaSourceDigest,
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
        let projection = FixedProjection::new(graph("demo"), request.projection_profile_sha256());
        let consumer = FixedBuckConsumer::new();
        let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

        let result = reconcile(
            &ReconciliationRequestV1::new(request, None),
            &generator,
            &projection,
            &consumer,
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
fn maintained_projection_must_match_every_render_visible_field() {
    let request = valid_generation_request(false);
    let generated = graph("demo");
    let generator = ScriptedGenerator::new(vec![
        Ok((generated.clone(), rendered("demo"))),
        Ok((generated, rendered("demo"))),
    ]);
    let projection = FixedProjection::new(graph("different"), request.projection_profile_sha256());
    let consumer = FixedBuckConsumer::new();
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

    let result = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &generator,
        &projection,
        &consumer,
        &publisher,
    );
    let ReconciliationResultV1::Refused { failure, .. } = result else {
        panic!("expected parser mismatch refusal");
    };
    assert_eq!(failure.class(), FailureClassV1::InvalidGeneratedGraph);
}

#[test]
fn projection_profile_binds_every_variable_parse_contract_input() {
    let baseline = valid_generation_request(false).projection_profile_sha256();
    let varied = [
        generation_request_with_projection_variation(ProjectionProfileVariation::Renderer),
        generation_request_with_projection_variation(ProjectionProfileVariation::Parser),
        generation_request_with_projection_variation(ProjectionProfileVariation::Grammar),
        generation_request_with_provider_profile(
            "oyatie.reindeer.changed-recipe.v1",
            b"provider source",
            b"graph schema",
        ),
        generation_request_with_provider_profile(
            "oyatie.reindeer.source-adaptation.v1",
            b"changed provider source",
            b"graph schema",
        ),
        generation_request_with_provider_profile(
            "oyatie.reindeer.source-adaptation.v1",
            b"provider source",
            b"changed graph schema",
        ),
    ];

    for request in varied {
        assert_ne!(request.projection_profile_sha256(), baseline);
    }
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
        request.projection_profile_sha256(),
    );
    let consumer = FixedBuckConsumer::new();
    let publisher = RecordingPublisher::new(PublicationOutcomeV1::Unchanged);

    let result = reconcile(
        &ReconciliationRequestV1::new(request, None),
        &generator,
        &projection,
        &consumer,
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
fn semantic_callees_admit_dotted_names_without_admitting_malformed_paths() {
    for callee in ["rule", "cargo.rust_library", "prelude.rust.library"] {
        assert!(
            SemanticValueV1::call_named(
                callee,
                vec![("name".to_owned(), SemanticValueV1::string("demo").unwrap())],
            )
            .is_ok(),
            "{callee:?}",
        );
    }
    for callee in ["", ".rule", "rule.", "cargo..rust_library", "cargo-rule"] {
        assert_eq!(
            SemanticValueV1::call_named(
                callee,
                vec![("name".to_owned(), SemanticValueV1::string("demo").unwrap())],
            )
            .unwrap_err()
            .class(),
            FailureClassV1::InvalidGeneratedGraph,
            "{callee:?}",
        );
    }
}

#[test]
fn rendered_projection_drops_only_unobservable_kind_and_integer_source_type() {
    let semantic = |edition| {
        SemanticValueV1::call_named(
            "cargo.rust_library",
            vec![
                ("name".to_owned(), SemanticValueV1::string("demo").unwrap()),
                ("edition".to_owned(), edition),
            ],
        )
        .unwrap()
    };
    let graph = |kind, edition| {
        RuleGraphV1::try_new(
            Vec::new(),
            vec![RuleV1::new(0, kind, semantic(edition), digest(b"rule"))],
        )
        .unwrap()
    };
    let producer = graph(ReindeerRuleKindV1::Library, SemanticValueV1::signed(2024));
    let different_kind = graph(
        ReindeerRuleKindV1::RootPackage,
        SemanticValueV1::signed(2024),
    );
    let unsigned_source = graph(ReindeerRuleKindV1::Library, SemanticValueV1::unsigned(2024));

    assert_ne!(producer, different_kind);
    assert_ne!(producer, unsigned_source);
    assert_eq!(
        producer.rendered_projection().unwrap(),
        different_kind.rendered_projection().unwrap(),
    );
    assert_eq!(
        producer.rendered_projection().unwrap(),
        unsigned_source.rendered_projection().unwrap(),
    );
    assert_eq!(
        graph(
            ReindeerRuleKindV1::Library,
            SemanticValueV1::unsigned(u128::from(u32::MAX)),
        )
        .rendered_projection()
        .unwrap_err()
        .class(),
        FailureClassV1::InvalidGeneratedGraph,
    );
}
