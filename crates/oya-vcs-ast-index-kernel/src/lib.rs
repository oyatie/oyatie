//! Oya VCS AST index kernel.
//!
//! Language-neutral AST/range/dependency contracts used by semantic claims,
//! review mapping, impacted-test selection, cache invalidation, and conflict
//! detection. Parser implementations are adapters; this kernel is std-only and
//! has no tree-sitter, compiler, filesystem, Git, or CI dependency.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use oya_vcs_kernel::{
    ArtifactPointer, ArtifactSelectorKind, SymbolId, SymbolLanguage, VcsKernelError,
};

const AST_INDEX_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AstSymbolKind {
    Crate,
    Module,
    Package,
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    Type,
    Class,
    Interface,
    Component,
    Route,
    Test,
    ConfigEntry,
    ContractOperation,
    Policy,
    Migration,
    Resource,
    Binding,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DependencyKind {
    Calls,
    Imports,
    Implements,
    Extends,
    Tests,
    GeneratesClient,
    ConsumesContract,
    FfiBinding,
    RouteToApi,
    SchemaToCodegen,
    XamlBindingToCSharpSymbol,
    PackageDependency,
    CiJobToArtifact,
    OwnsArtifact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionTarget {
    Dev,
    Staging,
    Production,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ByteRange {
    pub start: u32, // data_class: INTERNAL_ONLY
    pub end: u32,   // data_class: INTERNAL_ONLY
}

impl ByteRange {
    pub fn new(start: u32, end: u32) -> Result<Self, AstIndexError> {
        if end <= start {
            return Err(AstIndexError::InvalidRange);
        }
        Ok(Self { start, end })
    }

    pub fn contains(&self, byte_offset: u32) -> bool {
        self.start <= byte_offset && byte_offset < self.end
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextRange {
    pub start_line: u32,   // data_class: INTERNAL_ONLY
    pub start_column: u32, // data_class: INTERNAL_ONLY
    pub end_line: u32,     // data_class: INTERNAL_ONLY
    pub end_column: u32,   // data_class: INTERNAL_ONLY
}

impl TextRange {
    pub fn new(
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
    ) -> Result<Self, AstIndexError> {
        if start_line == 0 || end_line == 0 {
            return Err(AstIndexError::InvalidRange);
        }
        if end_line < start_line || (end_line == start_line && end_column <= start_column) {
            return Err(AstIndexError::InvalidRange);
        }
        Ok(Self {
            start_line,
            start_column,
            end_line,
            end_column,
        })
    }

    pub fn normalized_key(&self) -> String {
        format!(
            "{}:{}-{}:{}",
            self.start_line, self.start_column, self.end_line, self.end_column
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstSymbol {
    pub symbol_id: SymbolId,       // data_class: INTERNAL_ONLY
    pub artifact: ArtifactPointer, // data_class: INTERNAL_ONLY
    pub language: SymbolLanguage,  // data_class: INTERNAL_ONLY
    pub kind: AstSymbolKind,       // data_class: INTERNAL_ONLY
    pub byte_range: ByteRange,     // data_class: INTERNAL_ONLY
    pub text_range: TextRange,     // data_class: INTERNAL_ONLY
    pub source_digest: String,     // data_class: INTERNAL_ONLY
    pub parser_version: String,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AstSymbolDraft {
    pub symbol_id: SymbolId,       // data_class: INTERNAL_ONLY
    pub artifact: ArtifactPointer, // data_class: INTERNAL_ONLY
    pub kind: AstSymbolKind,       // data_class: INTERNAL_ONLY
    pub byte_range: ByteRange,     // data_class: INTERNAL_ONLY
    pub text_range: TextRange,     // data_class: INTERNAL_ONLY
    pub source_digest: String,     // data_class: INTERNAL_ONLY
    pub parser_version: String,    // data_class: INTERNAL_ONLY
}

impl AstSymbol {
    pub fn new(draft: AstSymbolDraft) -> Result<Self, AstIndexError> {
        if draft.symbol_id.artifact != draft.artifact {
            return Err(AstIndexError::ArtifactMismatch);
        }
        if draft.parser_version.trim().is_empty() {
            return Err(AstIndexError::EmptyParserVersion);
        }
        validate_source_digest(&draft.source_digest)?;
        Ok(Self {
            language: draft.symbol_id.language,
            source_digest: draft.source_digest,
            schema_version: AST_INDEX_SCHEMA_VERSION,
            symbol_id: draft.symbol_id,
            artifact: draft.artifact,
            kind: draft.kind,
            byte_range: draft.byte_range,
            text_range: draft.text_range,
            parser_version: draft.parser_version,
        })
    }

    pub fn stable_cache_key(&self) -> CacheKey {
        CacheKey {
            symbol_id: self.symbol_id.clone(),
            source_digest: self.source_digest.clone(),
            parser_version: self.parser_version.clone(),
            schema_version: self.schema_version,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CacheKey {
    pub symbol_id: SymbolId,    // data_class: INTERNAL_ONLY
    pub source_digest: String,  // data_class: INTERNAL_ONLY
    pub parser_version: String, // data_class: INTERNAL_ONLY
    pub schema_version: u32,    // data_class: INTERNAL_ONLY
}

impl CacheKey {
    pub fn is_stale_after(&self, next: &CacheKey) -> bool {
        self.symbol_id != next.symbol_id
            || self.source_digest != next.source_digest
            || self.parser_version != next.parser_version
            || self.schema_version != next.schema_version
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DependencyEdge {
    pub from: SymbolId,       // data_class: INTERNAL_ONLY
    pub to: SymbolId,         // data_class: INTERNAL_ONLY
    pub kind: DependencyKind, // data_class: INTERNAL_ONLY
}

impl DependencyEdge {
    pub fn new(from: SymbolId, to: SymbolId, kind: DependencyKind) -> Result<Self, AstIndexError> {
        if from == to {
            return Err(AstIndexError::SelfDependency);
        }
        Ok(Self { from, to, kind })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AstIndex {
    symbols: BTreeMap<String, AstSymbol>, // data_class: INTERNAL_ONLY
    reverse_edges: BTreeMap<String, BTreeSet<String>>, // data_class: INTERNAL_ONLY
    edges: BTreeSet<DependencyEdge>,      // data_class: INTERNAL_ONLY
}

impl AstIndex {
    pub fn insert_symbol(&mut self, symbol: AstSymbol) -> Result<(), AstIndexError> {
        if self.symbols.contains_key(&symbol.symbol_id.value) {
            return Err(AstIndexError::DuplicateSymbol);
        }
        self.symbols.insert(symbol.symbol_id.value.clone(), symbol);
        Ok(())
    }

    pub fn add_dependency(&mut self, edge: DependencyEdge) -> Result<(), AstIndexError> {
        if !self.symbols.contains_key(&edge.from.value)
            || !self.symbols.contains_key(&edge.to.value)
        {
            return Err(AstIndexError::UnknownDependencyEndpoint);
        }
        self.reverse_edges
            .entry(edge.to.value.clone())
            .or_default()
            .insert(edge.from.value.clone());
        self.edges.insert(edge);
        Ok(())
    }

    pub fn symbol(&self, symbol_id: &SymbolId) -> Option<&AstSymbol> {
        self.symbols.get(&symbol_id.value)
    }

    pub fn symbol_at_byte(
        &self,
        artifact: &ArtifactPointer,
        byte_offset: u32,
    ) -> Option<&AstSymbol> {
        self.symbols
            .values()
            .filter(|symbol| {
                &symbol.artifact == artifact && symbol.byte_range.contains(byte_offset)
            })
            .min_by_key(|symbol| symbol.byte_range.end - symbol.byte_range.start)
    }

    pub fn impacted_symbols(&self, changed_symbols: &[SymbolId]) -> Vec<SymbolId> {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        for symbol in changed_symbols {
            seen.insert(symbol.value.clone());
            queue.push_back(symbol.value.clone());
        }

        while let Some(current) = queue.pop_front() {
            if let Some(dependents) = self.reverse_edges.get(&current) {
                for dependent in dependents {
                    if seen.insert(dependent.clone()) {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        seen.into_iter()
            .filter_map(|value| {
                self.symbols
                    .get(&value)
                    .map(|symbol| symbol.symbol_id.clone())
            })
            .collect()
    }

    pub fn impacted_tests(&self, changed_symbols: &[SymbolId]) -> Vec<SymbolId> {
        self.impacted_symbols(changed_symbols)
            .into_iter()
            .filter(|symbol_id| {
                self.symbols
                    .get(&symbol_id.value)
                    .is_some_and(|symbol| symbol.kind == AstSymbolKind::Test)
            })
            .collect()
    }

    pub fn impacted_build_artifacts(&self, changed_symbols: &[SymbolId]) -> Vec<ArtifactPointer> {
        let mut artifacts = BTreeSet::new();
        for symbol_id in self.impacted_symbols(changed_symbols) {
            if let Some(symbol) = self.symbols.get(&symbol_id.value) {
                artifacts.insert(symbol.artifact.clone());
            }
        }
        artifacts.into_iter().collect()
    }

    pub fn semantic_conflict(&self, left: &[SymbolId], right: &[SymbolId]) -> bool {
        let left_impacted: BTreeSet<String> = self
            .impacted_symbols(left)
            .into_iter()
            .map(|symbol| symbol.value)
            .collect();
        let right_impacted: BTreeSet<String> = self
            .impacted_symbols(right)
            .into_iter()
            .map(|symbol| symbol.value)
            .collect();
        let left_changed: BTreeSet<String> =
            left.iter().map(|symbol| symbol.value.clone()).collect();
        let right_changed: BTreeSet<String> =
            right.iter().map(|symbol| symbol.value.clone()).collect();
        !left_impacted.is_disjoint(&right_changed) || !right_impacted.is_disjoint(&left_changed)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParserStatus {
    Parsed,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexAdmissionInput {
    pub artifact: ArtifactPointer,   // data_class: INTERNAL_ONLY
    pub parser_status: ParserStatus, // data_class: INTERNAL_ONLY
    pub explicit_pointer_scope: Option<ArtifactPointer>, // data_class: INTERNAL_ONLY
    pub target: PromotionTarget,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexAdmissionDecision {
    Admit,
    NonProductionGapAccepted,
}

pub fn evaluate_index_admission(
    input: IndexAdmissionInput,
) -> Result<IndexAdmissionDecision, AstIndexError> {
    match (
        input.parser_status,
        input.target,
        input.explicit_pointer_scope,
    ) {
        (ParserStatus::Parsed, _, _) => Ok(IndexAdmissionDecision::Admit),
        (ParserStatus::Failed, PromotionTarget::Production, None) => {
            Err(AstIndexError::ParserFailureWithoutPointerScope)
        }
        (ParserStatus::Failed, PromotionTarget::Production, Some(scope)) => {
            validate_explicit_pointer_scope(&input.artifact, &scope)?;
            Ok(IndexAdmissionDecision::Admit)
        }
        (ParserStatus::Failed, _, _) => Ok(IndexAdmissionDecision::NonProductionGapAccepted),
    }
}

pub fn validate_fresh_cache_key(
    previous: &CacheKey,
    recomputed: &CacheKey,
) -> Result<(), AstIndexError> {
    if previous.is_stale_after(recomputed) {
        Err(AstIndexError::StaleCacheKey)
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AstIndexError {
    Kernel(VcsKernelError),
    InvalidRange,
    ArtifactMismatch,
    EmptyParserVersion,
    DuplicateSymbol,
    SelfDependency,
    UnknownDependencyEndpoint,
    ParserFailureWithoutPointerScope,
    StaleCacheKey,
    InvalidSourceDigest,
    InvalidPointerScope,
}

impl From<VcsKernelError> for AstIndexError {
    fn from(value: VcsKernelError) -> Self {
        Self::Kernel(value)
    }
}

impl fmt::Display for AstIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for AstIndexError {}

fn validate_explicit_pointer_scope(
    artifact: &ArtifactPointer,
    scope: &ArtifactPointer,
) -> Result<(), AstIndexError> {
    if scope.path == artifact.path
        && scope.selector_kind != ArtifactSelectorKind::WholeFile
        && scope.selector.is_some()
    {
        Ok(())
    } else {
        Err(AstIndexError::InvalidPointerScope)
    }
}

fn validate_source_digest(value: &str) -> Result<(), AstIndexError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(AstIndexError::InvalidSourceDigest);
    };
    if hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(AstIndexError::InvalidSourceDigest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_vcs_kernel::{ArtifactPointer, SymbolLanguage};

    fn artifact(path: &str) -> ArtifactPointer {
        ArtifactPointer::file(path).expect("valid artifact")
    }

    fn rust_symbol(path: &str, symbol_path: &str) -> SymbolId {
        SymbolId::new(SymbolLanguage::Rust, artifact(path), symbol_path).expect("valid symbol id")
    }

    fn digest_for_fixture(source: &str) -> String {
        let mut out = String::from("sha256:");
        let bytes = source.as_bytes();
        for i in 0..64 {
            let b = bytes.get(i % bytes.len().max(1)).copied().unwrap_or(0);
            out.push(char::from_digit(((b as usize + i) % 16) as u32, 16).unwrap());
        }
        out
    }

    fn ast_symbol(
        path: &str,
        symbol_path: &str,
        kind: AstSymbolKind,
        start: u32,
        end: u32,
        source: &str,
    ) -> AstSymbol {
        let artifact = artifact(path);
        AstSymbol::new(AstSymbolDraft {
            symbol_id: SymbolId::new(SymbolLanguage::Rust, artifact.clone(), symbol_path)
                .expect("valid symbol id"),
            artifact,
            kind,
            byte_range: ByteRange::new(start, end).expect("valid byte range"),
            text_range: TextRange::new(1, start, 1, end).expect("valid text range"),
            source_digest: digest_for_fixture(source),
            parser_version: "rust-tree-sitter-fixture-v1".into(),
        })
        .expect("valid AST symbol")
    }

    #[test]
    fn symbol_ids_are_stable_for_same_pointer_and_path() {
        let left = rust_symbol("crates/example/src/lib.rs", "module::claim");
        let right = rust_symbol("crates/example/src/lib.rs", "module::claim");

        assert_eq!(left, right);
        assert!(left.value.starts_with("sym:v1|language:4:rust|"));
    }

    #[test]
    fn ranges_normalize_to_review_mapping_key() {
        let range = TextRange::new(10, 4, 12, 1).expect("valid range");

        assert_eq!(range.normalized_key(), "10:4-12:1");
        assert!(ByteRange::new(20, 25).unwrap().contains(24));
        assert_eq!(ByteRange::new(25, 25), Err(AstIndexError::InvalidRange));
    }

    #[test]
    fn impacted_tests_and_build_artifacts_follow_reverse_dependencies() {
        let mut index = AstIndex::default();
        let function = ast_symbol(
            "src/lib.rs",
            "module::claim",
            AstSymbolKind::Function,
            0,
            10,
            "fn claim() {}",
        );
        let test = ast_symbol(
            "tests/claim.rs",
            "claim_rejects_conflict",
            AstSymbolKind::Test,
            0,
            20,
            "#[test]",
        );
        let generated_client = ast_symbol(
            "contracts/openapi/vcs.yaml",
            "operation::claim",
            AstSymbolKind::ContractOperation,
            0,
            30,
            "operationId: claim",
        );
        let changed = function.symbol_id.clone();
        index.insert_symbol(function.clone()).unwrap();
        index.insert_symbol(test.clone()).unwrap();
        index.insert_symbol(generated_client.clone()).unwrap();
        index
            .add_dependency(
                DependencyEdge::new(
                    test.symbol_id.clone(),
                    function.symbol_id.clone(),
                    DependencyKind::Tests,
                )
                .unwrap(),
            )
            .unwrap();
        index
            .add_dependency(
                DependencyEdge::new(
                    generated_client.symbol_id.clone(),
                    function.symbol_id.clone(),
                    DependencyKind::ConsumesContract,
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(
            index.impacted_tests(std::slice::from_ref(&changed)),
            vec![test.symbol_id]
        );
        let artifacts = index.impacted_build_artifacts(&[changed]);
        assert!(
            artifacts
                .iter()
                .any(|artifact| artifact.path == "tests/claim.rs")
        );
        assert!(
            artifacts
                .iter()
                .any(|artifact| artifact.path == "contracts/openapi/vcs.yaml")
        );
    }

    #[test]
    fn semantic_conflict_is_symmetric_for_reverse_dependencies() {
        let mut index = AstIndex::default();
        let function = ast_symbol(
            "src/lib.rs",
            "module::claim",
            AstSymbolKind::Function,
            0,
            10,
            "fn claim() {}",
        );
        let test = ast_symbol(
            "tests/claim.rs",
            "claim_rejects_conflict",
            AstSymbolKind::Test,
            0,
            20,
            "#[test]",
        );
        index.insert_symbol(function.clone()).unwrap();
        index.insert_symbol(test.clone()).unwrap();
        index
            .add_dependency(
                DependencyEdge::new(
                    test.symbol_id.clone(),
                    function.symbol_id.clone(),
                    DependencyKind::Tests,
                )
                .unwrap(),
            )
            .unwrap();

        assert!(index.semantic_conflict(
            std::slice::from_ref(&function.symbol_id),
            std::slice::from_ref(&test.symbol_id),
        ));
        assert!(index.semantic_conflict(&[test.symbol_id], &[function.symbol_id]));
    }

    #[test]
    fn cache_key_invalidates_when_source_digest_changes() {
        let previous = ast_symbol(
            "src/lib.rs",
            "module::claim",
            AstSymbolKind::Function,
            0,
            10,
            "fn claim() {}",
        );
        let next = ast_symbol(
            "src/lib.rs",
            "module::claim",
            AstSymbolKind::Function,
            0,
            11,
            "fn claim2() {}",
        );

        assert!(
            previous
                .stable_cache_key()
                .is_stale_after(&next.stable_cache_key())
        );
        assert_eq!(
            validate_fresh_cache_key(&previous.stable_cache_key(), &next.stable_cache_key()),
            Err(AstIndexError::StaleCacheKey)
        );
    }

    #[test]
    fn parser_failure_without_explicit_pointer_scope_blocks_production() {
        let decision = evaluate_index_admission(IndexAdmissionInput {
            artifact: artifact("src/lib.rs"),
            parser_status: ParserStatus::Failed,
            explicit_pointer_scope: None,
            target: PromotionTarget::Production,
        });

        assert_eq!(
            decision,
            Err(AstIndexError::ParserFailureWithoutPointerScope)
        );
    }

    #[test]
    fn production_parser_gap_rejects_unrelated_pointer_scope() {
        let unrelated = ArtifactPointer::new(
            "contracts/other.yaml",
            ArtifactSelectorKind::OpenApiOperation,
            Some("POST /claim".into()),
        )
        .expect("valid pointer");
        let decision = evaluate_index_admission(IndexAdmissionInput {
            artifact: artifact("contracts/openapi/vcs.yaml"),
            parser_status: ParserStatus::Failed,
            explicit_pointer_scope: Some(unrelated),
            target: PromotionTarget::Production,
        });

        assert_eq!(decision, Err(AstIndexError::InvalidPointerScope));
    }

    #[test]
    fn production_parser_gap_rejects_whole_file_pointer_scope() {
        let decision = evaluate_index_admission(IndexAdmissionInput {
            artifact: artifact("contracts/openapi/vcs.yaml"),
            parser_status: ParserStatus::Failed,
            explicit_pointer_scope: Some(artifact("contracts/openapi/vcs.yaml")),
            target: PromotionTarget::Production,
        });

        assert_eq!(decision, Err(AstIndexError::InvalidPointerScope));
    }

    #[test]
    fn explicit_pointer_scope_allows_production_parser_gap() {
        let pointer = ArtifactPointer::new(
            "contracts/openapi/vcs.yaml",
            oya_vcs_kernel::ArtifactSelectorKind::OpenApiOperation,
            Some("POST /claims".into()),
        )
        .expect("valid pointer");
        let decision = evaluate_index_admission(IndexAdmissionInput {
            artifact: artifact("contracts/openapi/vcs.yaml"),
            parser_status: ParserStatus::Failed,
            explicit_pointer_scope: Some(pointer),
            target: PromotionTarget::Production,
        });

        assert_eq!(decision, Ok(IndexAdmissionDecision::Admit));
    }
}
