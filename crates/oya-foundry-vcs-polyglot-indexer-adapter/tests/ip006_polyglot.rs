use oya_foundry_vcs_ast_index_kernel::{DependencyKind, PromotionTarget};
use oya_foundry_vcs_kernel::{ArtifactPointer, ArtifactSelectorKind};
use oya_foundry_vcs_polyglot_indexer_adapter::{
    AdapterError, DeterministicPolyglotAdapter, DiffInput, IndexRequest, PolyglotIndexerAdapter,
    ProductionSurface, build_diff_map, index_source, surface_for_path,
};

#[test]
fn public_adapter_extracts_diff_impact_from_polyglot_dependencies() {
    let adapter = DeterministicPolyglotAdapter;
    let indexed = adapter
        .index(IndexRequest::new(
            "web/app.ts",
            "function route() {}\nfunction route_test() {}\n// oya-dep:route_test->route:tests",
            PromotionTarget::Production,
        ))
        .expect("adapter indexes TypeScript fixture");
    let route = indexed
        .symbols
        .iter()
        .find(|symbol| symbol.symbol_id.symbol_path.ends_with("::route"))
        .expect("route symbol")
        .symbol_id
        .clone();

    let impact = build_diff_map(DiffInput {
        indexed_artifacts: vec![indexed],
        changed_symbols: vec![route],
        target: PromotionTarget::Production,
    });

    assert_eq!(impact.dependency_edges[0].kind, DependencyKind::Tests);
    assert_eq!(impact.impacted_tests.len(), 1);
    assert!(impact.promotion_blockers.is_empty());
}

#[test]
fn public_adapter_blocks_unknown_prod_surface_and_accepts_explicit_parser_gap() {
    assert_eq!(
        surface_for_path("native/unknown.elvish"),
        ProductionSurface::Unsupported
    );
    assert_eq!(
        index_source(IndexRequest::new(
            "native/unknown.elvish",
            "unknown",
            PromotionTarget::Production,
        )),
        Err(AdapterError::UnsupportedProductionSurface {
            path: "native/unknown.elvish".into()
        })
    );

    let pointer = ArtifactPointer::new(
        "contracts/openapi.yaml",
        ArtifactSelectorKind::OpenApiOperation,
        Some("GET /status".into()),
    )
    .expect("valid pointer");
    let fallback = index_source(
        IndexRequest::new(
            "contracts/openapi.yaml",
            "OYA_PARSER_FAIL",
            PromotionTarget::Production,
        )
        .with_pointer_scope(pointer),
    )
    .expect("explicit pointer admits fallback");

    assert_eq!(fallback.symbols.len(), 1);
    assert_eq!(fallback.parser_diagnostics.len(), 1);
}
