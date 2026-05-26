// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_vcs_ast_index_kernel::{
    AstIndex, AstIndexError, AstSymbol, AstSymbolDraft, AstSymbolKind, ByteRange, DependencyEdge,
    DependencyKind, IndexAdmissionDecision, IndexAdmissionInput, ParserStatus, PromotionTarget,
    TextRange, evaluate_index_admission, validate_fresh_cache_key,
};
use oya_vcs_kernel::{ArtifactPointer, ArtifactSelectorKind, SymbolId, SymbolLanguage};

fn digest_for_fixture(source: &str) -> String {
    let mut out = String::from("sha256:");
    let bytes = source.as_bytes();
    for i in 0..64 {
        let b = bytes.get(i % bytes.len().max(1)).copied().unwrap_or(0);
        out.push(char::from_digit(((b as usize + i) % 16) as u32, 16).unwrap());
    }
    out
}

fn artifact(path: &str) -> ArtifactPointer {
    ArtifactPointer::file(path).expect("valid artifact")
}

fn symbol(path: &str, name: &str) -> SymbolId {
    SymbolId::new(SymbolLanguage::Rust, artifact(path), name).expect("valid symbol")
}

fn ast_symbol(path: &str, name: &str, kind: AstSymbolKind, source: &str) -> AstSymbol {
    let artifact = artifact(path);
    AstSymbol::new(AstSymbolDraft {
        symbol_id: SymbolId::new(SymbolLanguage::Rust, artifact.clone(), name).unwrap(),
        artifact,
        kind,
        byte_range: ByteRange::new(0, source.len() as u32).unwrap(),
        text_range: TextRange::new(1, 0, 1, source.len() as u32).unwrap(),
        source_digest: digest_for_fixture(source),
        parser_version: "fixture-rust-parser-v1".into(),
    })
    .expect("valid symbol")
}

#[test]
fn symbol_id_and_range_contracts_are_stable() {
    let left = symbol("crates/a/src/lib.rs", "module::claim");
    let right = symbol("crates/a/src/lib.rs", "module::claim");
    let review_range = TextRange::new(4, 2, 5, 0).unwrap();

    assert_eq!(left, right);
    assert_eq!(review_range.normalized_key(), "4:2-5:0");
    assert_eq!(ByteRange::new(9, 9), Err(AstIndexError::InvalidRange));
}

#[test]
fn impacted_tests_and_build_closure_are_semantic_not_whole_tree() {
    let mut index = AstIndex::default();
    let production = ast_symbol(
        "crates/a/src/lib.rs",
        "module::claim",
        AstSymbolKind::Function,
        "fn claim() {}",
    );
    let unit_test = ast_symbol(
        "crates/a/tests/claim.rs",
        "claim_rejects_conflict",
        AstSymbolKind::Test,
        "#[test] fn x() {}",
    );
    let route = ast_symbol(
        "contracts/openapi/vcs.yaml",
        "operation::claim",
        AstSymbolKind::ContractOperation,
        "post: /claim",
    );
    let changed = production.symbol_id.clone();

    index.insert_symbol(production.clone()).unwrap();
    index.insert_symbol(unit_test.clone()).unwrap();
    index.insert_symbol(route.clone()).unwrap();
    index
        .add_dependency(
            DependencyEdge::new(
                unit_test.symbol_id.clone(),
                production.symbol_id.clone(),
                DependencyKind::Tests,
            )
            .unwrap(),
        )
        .unwrap();
    index
        .add_dependency(
            DependencyEdge::new(
                route.symbol_id.clone(),
                production.symbol_id.clone(),
                DependencyKind::ConsumesContract,
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(
        index.impacted_tests(std::slice::from_ref(&changed)),
        vec![unit_test.symbol_id]
    );
    let build_artifacts = index.impacted_build_artifacts(&[changed]);
    assert!(
        build_artifacts
            .iter()
            .any(|artifact| artifact.path == "crates/a/tests/claim.rs")
    );
    assert!(
        build_artifacts
            .iter()
            .any(|artifact| artifact.path == "contracts/openapi/vcs.yaml")
    );
}

#[test]
fn semantic_conflict_is_symmetric() {
    let mut index = AstIndex::default();
    let production = ast_symbol(
        "crates/a/src/lib.rs",
        "module::claim",
        AstSymbolKind::Function,
        "fn claim() {}",
    );
    let unit_test = ast_symbol(
        "crates/a/tests/claim.rs",
        "claim_rejects_conflict",
        AstSymbolKind::Test,
        "#[test] fn x() {}",
    );
    index.insert_symbol(production.clone()).unwrap();
    index.insert_symbol(unit_test.clone()).unwrap();
    index
        .add_dependency(
            DependencyEdge::new(
                unit_test.symbol_id.clone(),
                production.symbol_id.clone(),
                DependencyKind::Tests,
            )
            .unwrap(),
        )
        .unwrap();

    assert!(index.semantic_conflict(
        std::slice::from_ref(&production.symbol_id),
        std::slice::from_ref(&unit_test.symbol_id)
    ));
    assert!(index.semantic_conflict(&[unit_test.symbol_id], &[production.symbol_id]));
}

#[test]
fn stale_cache_key_blocks_promotion() {
    let previous = ast_symbol(
        "crates/a/src/lib.rs",
        "module::claim",
        AstSymbolKind::Function,
        "fn claim() {}",
    );
    let recomputed = ast_symbol(
        "crates/a/src/lib.rs",
        "module::claim",
        AstSymbolKind::Function,
        "fn claim2() {}",
    );

    assert_eq!(
        validate_fresh_cache_key(&previous.stable_cache_key(), &recomputed.stable_cache_key()),
        Err(AstIndexError::StaleCacheKey)
    );
}

#[test]
fn parser_failure_without_pointer_scope_blocks_production() {
    assert_eq!(
        evaluate_index_admission(IndexAdmissionInput {
            artifact: artifact("crates/a/src/lib.rs"),
            parser_status: ParserStatus::Failed,
            explicit_pointer_scope: None,
            target: PromotionTarget::Production,
        }),
        Err(AstIndexError::ParserFailureWithoutPointerScope)
    );
}

#[test]
fn parser_fallback_rejects_unrelated_pointer_scope() {
    let unrelated = ArtifactPointer::new(
        "contracts/openapi/other.yaml",
        ArtifactSelectorKind::OpenApiOperation,
        Some("POST /claim".into()),
    )
    .unwrap();

    assert_eq!(
        evaluate_index_admission(IndexAdmissionInput {
            artifact: artifact("contracts/openapi/vcs.yaml"),
            parser_status: ParserStatus::Failed,
            explicit_pointer_scope: Some(unrelated),
            target: PromotionTarget::Production,
        }),
        Err(AstIndexError::InvalidPointerScope)
    );
}

#[test]
fn parser_fallback_rejects_whole_file_scope() {
    assert_eq!(
        evaluate_index_admission(IndexAdmissionInput {
            artifact: artifact("contracts/openapi/vcs.yaml"),
            parser_status: ParserStatus::Failed,
            explicit_pointer_scope: Some(artifact("contracts/openapi/vcs.yaml")),
            target: PromotionTarget::Production,
        }),
        Err(AstIndexError::InvalidPointerScope)
    );
}

#[test]
fn explicit_pointer_scope_allows_contract_parser_fallback() {
    let pointer = ArtifactPointer::new(
        "contracts/openapi/vcs.yaml",
        ArtifactSelectorKind::OpenApiOperation,
        Some("POST /claim".into()),
    )
    .unwrap();

    assert_eq!(
        evaluate_index_admission(IndexAdmissionInput {
            artifact: artifact("contracts/openapi/vcs.yaml"),
            parser_status: ParserStatus::Failed,
            explicit_pointer_scope: Some(pointer),
            target: PromotionTarget::Production,
        }),
        Ok(IndexAdmissionDecision::Admit)
    );
}
