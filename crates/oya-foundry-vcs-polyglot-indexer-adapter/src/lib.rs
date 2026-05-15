//! Polyglot AST/indexer adapters for the Oya VCS semantic index.
//!
//! This crate is intentionally std-only. It provides production-shaped adapter
//! boundaries and deterministic line scanners that normalize symbols through the
//! pure VCS and AST-index kernels. Real parser-backed adapters can replace these
//! scanners without changing the kernel-facing admission or impact contracts.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use oya_foundry_vcs_ast_index_kernel::{
    AstIndex, AstIndexError, AstSymbol, AstSymbolDraft, AstSymbolKind, ByteRange, DependencyEdge,
    DependencyKind, IndexAdmissionDecision, IndexAdmissionInput, ParserStatus, PromotionTarget,
    TextRange, evaluate_index_admission,
};
use oya_foundry_vcs_kernel::{
    ArtifactPointer, ArtifactSelectorKind, SymbolId, SymbolLanguage, VcsKernelError,
};

const PARSER_VERSION: &str = "oya-polyglot-line-scanner-v1";
const PARSER_FAILURE_SENTINEL: &str = "OYA_PARSER_FAIL";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ProductionSurface {
    Rust,
    TypeScript,
    JavaScript,
    Swift,
    Kotlin,
    CSharp,
    C,
    Cpp,
    WinUiXaml,
    SchemaContract,
    Config,
    Unsupported,
}

impl ProductionSurface {
    pub fn language(self) -> SymbolLanguage {
        match self {
            Self::Rust => SymbolLanguage::Rust,
            Self::TypeScript => SymbolLanguage::TypeScript,
            Self::JavaScript => SymbolLanguage::JavaScript,
            Self::Swift => SymbolLanguage::Swift,
            Self::Kotlin => SymbolLanguage::Kotlin,
            Self::CSharp => SymbolLanguage::CSharp,
            Self::C | Self::Cpp => SymbolLanguage::Cpp,
            Self::WinUiXaml => SymbolLanguage::Xaml,
            Self::SchemaContract => SymbolLanguage::OpenApi,
            Self::Config => SymbolLanguage::Config,
            Self::Unsupported => SymbolLanguage::Unknown,
        }
    }

    pub fn is_supported(self) -> bool {
        self != Self::Unsupported
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRequest {
    pub path: String,                                    // data_class: INTERNAL_ONLY
    pub source: String,                                  // data_class: INTERNAL_ONLY
    pub target: PromotionTarget,                         // data_class: INTERNAL_ONLY
    pub explicit_pointer_scope: Option<ArtifactPointer>, // data_class: INTERNAL_ONLY
}

impl IndexRequest {
    pub fn new(
        path: impl Into<String>,
        source: impl Into<String>,
        target: PromotionTarget,
    ) -> Self {
        Self {
            path: path.into(),
            source: source.into(),
            target,
            explicit_pointer_scope: None,
        }
    }

    pub fn with_pointer_scope(mut self, pointer: ArtifactPointer) -> Self {
        self.explicit_pointer_scope = Some(pointer);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedArtifact {
    pub artifact: ArtifactPointer,           // data_class: INTERNAL_ONLY
    pub surface: ProductionSurface,          // data_class: INTERNAL_ONLY
    pub parser_status: ParserStatus,         // data_class: INTERNAL_ONLY
    pub admission: IndexAdmissionDecision,   // data_class: INTERNAL_ONLY
    pub symbols: Vec<AstSymbol>,             // data_class: INTERNAL_ONLY
    pub dependencies: Vec<SymbolDependency>, // data_class: INTERNAL_ONLY
    pub parser_diagnostics: Vec<String>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SymbolDependency {
    pub from: SymbolId,       // data_class: INTERNAL_ONLY
    pub to: SymbolId,         // data_class: INTERNAL_ONLY
    pub kind: DependencyKind, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffInput {
    pub indexed_artifacts: Vec<IndexedArtifact>, // data_class: INTERNAL_ONLY
    pub changed_symbols: Vec<SymbolId>,          // data_class: INTERNAL_ONLY
    pub target: PromotionTarget,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolyglotDiffMap {
    pub changed_symbols: Vec<SymbolId>,  // data_class: INTERNAL_ONLY
    pub impacted_symbols: Vec<SymbolId>, // data_class: INTERNAL_ONLY
    pub impacted_tests: Vec<SymbolId>,   // data_class: INTERNAL_ONLY
    pub dependency_edges: Vec<SymbolDependency>, // data_class: INTERNAL_ONLY
    pub promotion_blockers: Vec<PromotionBlocker>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PromotionBlocker {
    UnsupportedProductionSurface { path: String },
    ParserFailureWithoutExplicitPointer { path: String },
    UnknownDependencyEndpoint { from: String, to: String },
    NoSymbolsExtracted { path: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterError {
    Kernel(VcsKernelError),
    Ast(AstIndexError),
    UnsupportedProductionSurface { path: String },
    ParserFailureWithoutExplicitPointer { path: String },
}

impl From<VcsKernelError> for AdapterError {
    fn from(value: VcsKernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<AstIndexError> for AdapterError {
    fn from(value: AstIndexError) -> Self {
        Self::Ast(value)
    }
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for AdapterError {}

pub trait PolyglotIndexerAdapter {
    fn surface(&self) -> ProductionSurface;
    fn index(&self, request: IndexRequest) -> Result<IndexedArtifact, AdapterError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DeterministicPolyglotAdapter;

impl PolyglotIndexerAdapter for DeterministicPolyglotAdapter {
    fn surface(&self) -> ProductionSurface {
        ProductionSurface::Unsupported
    }

    fn index(&self, request: IndexRequest) -> Result<IndexedArtifact, AdapterError> {
        index_source(request)
    }
}

pub fn index_source(request: IndexRequest) -> Result<IndexedArtifact, AdapterError> {
    let surface = surface_for_path(&request.path);
    if !surface.is_supported() && request.target == PromotionTarget::Production {
        return Err(AdapterError::UnsupportedProductionSurface { path: request.path });
    }

    let artifact = ArtifactPointer::file(&request.path)?;
    let parser_status = if request.source.contains(PARSER_FAILURE_SENTINEL) {
        ParserStatus::Failed
    } else {
        ParserStatus::Parsed
    };
    let admission = evaluate_index_admission(IndexAdmissionInput {
        artifact: artifact.clone(),
        parser_status,
        explicit_pointer_scope: request.explicit_pointer_scope.clone(),
        target: request.target,
    })
    .map_err(|error| match error {
        AstIndexError::ParserFailureWithoutPointerScope => {
            AdapterError::ParserFailureWithoutExplicitPointer {
                path: request.path.clone(),
            }
        }
        other => AdapterError::Ast(other),
    })?;

    if parser_status == ParserStatus::Failed {
        if let Some(pointer) = request.explicit_pointer_scope {
            let symbol = symbol_from_parts(
                surface.language(),
                pointer.clone(),
                &fallback_symbol_path(&pointer),
                AstSymbolKind::Unknown,
                &request.source,
                1,
                0,
                1,
                1,
            )?;
            return Ok(IndexedArtifact {
                artifact,
                surface,
                parser_status,
                admission,
                symbols: vec![symbol],
                dependencies: Vec::new(),
                parser_diagnostics: vec![
                    "parser failure admitted by explicit pointer scope".into(),
                ],
            });
        }
        return Ok(IndexedArtifact {
            artifact,
            surface,
            parser_status,
            admission,
            symbols: Vec::new(),
            dependencies: Vec::new(),
            parser_diagnostics: vec![
                "non-production parser failure admitted without pointer scope".into(),
            ],
        });
    }

    let symbols = scan_symbols(surface, &request.path, &request.source)?;
    let dependencies = scan_dependencies(&symbols, &request.source)?;
    Ok(IndexedArtifact {
        artifact,
        surface,
        parser_status,
        admission,
        symbols,
        dependencies,
        parser_diagnostics: Vec::new(),
    })
}

pub fn build_diff_map(input: DiffInput) -> PolyglotDiffMap {
    let mut index = AstIndex::default();
    let mut dependency_edges = Vec::new();
    let mut blockers = BTreeSet::new();
    let changed_symbols = dedup_symbols(input.changed_symbols);

    for artifact in &input.indexed_artifacts {
        if input.target == PromotionTarget::Production && !artifact.surface.is_supported() {
            blockers.insert(PromotionBlocker::UnsupportedProductionSurface {
                path: artifact.artifact.path.clone(),
            });
        }
        if input.target == PromotionTarget::Production
            && artifact.parser_status == ParserStatus::Failed
            && artifact.parser_diagnostics.is_empty()
        {
            blockers.insert(PromotionBlocker::ParserFailureWithoutExplicitPointer {
                path: artifact.artifact.path.clone(),
            });
        }
        if input.target == PromotionTarget::Production && artifact.symbols.is_empty() {
            blockers.insert(PromotionBlocker::NoSymbolsExtracted {
                path: artifact.artifact.path.clone(),
            });
        }
        for symbol in &artifact.symbols {
            if let Err(AstIndexError::DuplicateSymbol) = index.insert_symbol(symbol.clone()) {
                continue;
            }
        }
    }

    for artifact in &input.indexed_artifacts {
        for dependency in &artifact.dependencies {
            dependency_edges.push(dependency.clone());
            let edge = DependencyEdge::new(
                dependency.from.clone(),
                dependency.to.clone(),
                dependency.kind,
            );
            match edge.and_then(|edge| index.add_dependency(edge)) {
                Ok(()) => {}
                Err(AstIndexError::UnknownDependencyEndpoint) => {
                    blockers.insert(PromotionBlocker::UnknownDependencyEndpoint {
                        from: dependency.from.value.clone(),
                        to: dependency.to.value.clone(),
                    });
                }
                Err(_) => {}
            }
        }
    }

    let impacted_symbols = index.impacted_symbols(&changed_symbols);
    let impacted_tests = index.impacted_tests(&changed_symbols);
    PolyglotDiffMap {
        changed_symbols,
        impacted_symbols,
        impacted_tests,
        dependency_edges,
        promotion_blockers: blockers.into_iter().collect(),
    }
}

pub fn surface_for_path(path: &str) -> ProductionSurface {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".rs") {
        ProductionSurface::Rust
    } else if lower.ends_with(".ts") || lower.ends_with(".tsx") {
        ProductionSurface::TypeScript
    } else if lower.ends_with(".js") || lower.ends_with(".jsx") {
        ProductionSurface::JavaScript
    } else if lower.ends_with(".swift") {
        ProductionSurface::Swift
    } else if lower.ends_with(".kt") || lower.ends_with(".kts") {
        ProductionSurface::Kotlin
    } else if lower.ends_with(".cs") {
        ProductionSurface::CSharp
    } else if lower.ends_with(".c") || lower.ends_with(".h") {
        ProductionSurface::C
    } else if lower.ends_with(".cc")
        || lower.ends_with(".cpp")
        || lower.ends_with(".cxx")
        || lower.ends_with(".hpp")
        || lower.ends_with(".hh")
    {
        ProductionSurface::Cpp
    } else if lower.ends_with(".xaml") {
        ProductionSurface::WinUiXaml
    } else if lower.ends_with("openapi.yaml")
        || lower.ends_with("openapi.yml")
        || lower.ends_with("asyncapi.yaml")
        || lower.ends_with("asyncapi.yml")
        || lower.ends_with(".proto")
        || lower.ends_with("schema.json")
    {
        ProductionSurface::SchemaContract
    } else if lower.ends_with(".json")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
        || lower.ends_with(".toml")
        || lower.ends_with(".config")
    {
        ProductionSurface::Config
    } else {
        ProductionSurface::Unsupported
    }
}

fn scan_symbols(
    surface: ProductionSurface,
    path: &str,
    source: &str,
) -> Result<Vec<AstSymbol>, AdapterError> {
    match surface {
        ProductionSurface::Rust => scan_keyword_symbols(
            path,
            source,
            SymbolLanguage::Rust,
            &[
                ("fn", AstSymbolKind::Function),
                ("struct", AstSymbolKind::Struct),
                ("enum", AstSymbolKind::Enum),
                ("trait", AstSymbolKind::Trait),
                ("mod", AstSymbolKind::Module),
            ],
        ),
        ProductionSurface::TypeScript => scan_keyword_symbols(
            path,
            source,
            SymbolLanguage::TypeScript,
            &[
                ("function", AstSymbolKind::Function),
                ("class", AstSymbolKind::Class),
                ("interface", AstSymbolKind::Interface),
                ("type", AstSymbolKind::Type),
                ("const", AstSymbolKind::Function),
                ("export function", AstSymbolKind::Function),
                ("export class", AstSymbolKind::Class),
                ("export interface", AstSymbolKind::Interface),
            ],
        ),
        ProductionSurface::JavaScript => scan_keyword_symbols(
            path,
            source,
            SymbolLanguage::JavaScript,
            &[
                ("function", AstSymbolKind::Function),
                ("class", AstSymbolKind::Class),
                ("const", AstSymbolKind::Function),
                ("export function", AstSymbolKind::Function),
            ],
        ),
        ProductionSurface::Swift => scan_keyword_symbols(
            path,
            source,
            SymbolLanguage::Swift,
            &[
                ("func", AstSymbolKind::Function),
                ("class", AstSymbolKind::Class),
                ("struct", AstSymbolKind::Struct),
                ("enum", AstSymbolKind::Enum),
                ("protocol", AstSymbolKind::Interface),
            ],
        ),
        ProductionSurface::Kotlin => scan_keyword_symbols(
            path,
            source,
            SymbolLanguage::Kotlin,
            &[
                ("fun", AstSymbolKind::Function),
                ("class", AstSymbolKind::Class),
                ("interface", AstSymbolKind::Interface),
                ("object", AstSymbolKind::Struct),
            ],
        ),
        ProductionSurface::CSharp => scan_keyword_symbols(
            path,
            source,
            SymbolLanguage::CSharp,
            &[
                ("class", AstSymbolKind::Class),
                ("interface", AstSymbolKind::Interface),
                ("struct", AstSymbolKind::Struct),
                ("void", AstSymbolKind::Method),
                ("public", AstSymbolKind::Method),
                ("private", AstSymbolKind::Method),
            ],
        ),
        ProductionSurface::C | ProductionSurface::Cpp => {
            scan_c_like(path, source, surface.language())
        }
        ProductionSurface::WinUiXaml => scan_xaml(path, source),
        ProductionSurface::SchemaContract => scan_schema_contract(path, source),
        ProductionSurface::Config => scan_config(path, source),
        ProductionSurface::Unsupported => Ok(Vec::new()),
    }
}

fn scan_keyword_symbols(
    path: &str,
    source: &str,
    language: SymbolLanguage,
    rules: &[(&str, AstSymbolKind)],
) -> Result<Vec<AstSymbol>, AdapterError> {
    let artifact = ArtifactPointer::file(path)?;
    let mut symbols = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = trim_visibility(line.trim());
        for (keyword, kind) in rules {
            if let Some(rest) = trimmed.strip_prefix(keyword)
                && let Some(name) = first_identifier(rest)
            {
                let kind = if name.to_ascii_lowercase().contains("test") {
                    AstSymbolKind::Test
                } else {
                    *kind
                };
                let symbol_path = normalized_symbol_path(language, path, &name, kind);
                symbols.push(line_symbol(
                    language,
                    artifact.clone(),
                    &symbol_path,
                    kind,
                    source,
                    line_idx,
                    line,
                )?);
                break;
            }
        }
        if let Some((name, kind)) = scan_test_name(trimmed) {
            let symbol_path = normalized_symbol_path(language, path, &name, kind);
            symbols.push(line_symbol(
                language,
                artifact.clone(),
                &symbol_path,
                kind,
                source,
                line_idx,
                line,
            )?);
        }
    }
    Ok(dedup_ast_symbols(symbols))
}

fn scan_c_like(
    path: &str,
    source: &str,
    language: SymbolLanguage,
) -> Result<Vec<AstSymbol>, AdapterError> {
    let artifact = ArtifactPointer::file(path)?;
    let mut symbols = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix("class ")
            .or_else(|| trimmed.strip_prefix("struct "))
            .and_then(first_identifier)
        {
            symbols.push(line_symbol(
                language,
                artifact.clone(),
                &format!("type::{name}"),
                AstSymbolKind::Struct,
                source,
                line_idx,
                line,
            )?);
        } else if trimmed.ends_with('{')
            && trimmed.contains('(')
            && trimmed.contains(')')
            && let Some(name) = function_name_before_paren(trimmed)
        {
            symbols.push(line_symbol(
                language,
                artifact.clone(),
                &format!("fn::{name}"),
                AstSymbolKind::Function,
                source,
                line_idx,
                line,
            )?);
        }
    }
    Ok(dedup_ast_symbols(symbols))
}

fn scan_xaml(path: &str, source: &str) -> Result<Vec<AstSymbol>, AdapterError> {
    let artifact = ArtifactPointer::file(path)?;
    let mut symbols = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        for attr in ["x:Class=\"", "x:Name=\"", "Name=\"", "Click=\""] {
            if let Some(value) = attribute_value(line, attr) {
                let kind = if attr == "Click=\"" {
                    AstSymbolKind::Binding
                } else {
                    AstSymbolKind::Component
                };
                let selector = format!("{}{}", attr.trim_end_matches("=\""), value);
                let pointer =
                    ArtifactPointer::new(path, ArtifactSelectorKind::XamlBinding, Some(selector))?;
                symbols.push(line_symbol(
                    SymbolLanguage::Xaml,
                    pointer,
                    &format!("xaml::{value}"),
                    kind,
                    source,
                    line_idx,
                    line,
                )?);
            }
        }
    }
    if symbols.is_empty() && source.contains('<') {
        symbols.push(line_symbol(
            SymbolLanguage::Xaml,
            artifact,
            "xaml::<file>",
            AstSymbolKind::Component,
            source,
            0,
            source.lines().next().unwrap_or("<xaml />"),
        )?);
    }
    Ok(dedup_ast_symbols(symbols))
}

fn scan_schema_contract(path: &str, source: &str) -> Result<Vec<AstSymbol>, AdapterError> {
    let language = if path.ends_with(".proto") {
        SymbolLanguage::Protobuf
    } else if path.contains("asyncapi") {
        SymbolLanguage::AsyncApi
    } else if path.ends_with(".json") {
        SymbolLanguage::Json
    } else {
        SymbolLanguage::OpenApi
    };
    let selector_kind = match language {
        SymbolLanguage::Protobuf => ArtifactSelectorKind::ProtobufSymbol,
        SymbolLanguage::AsyncApi => ArtifactSelectorKind::AsyncApiChannel,
        SymbolLanguage::Json => ArtifactSelectorKind::JsonPointer,
        _ => ArtifactSelectorKind::OpenApiOperation,
    };
    let mut symbols = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let candidate = trimmed
            .strip_prefix("operationId:")
            .or_else(|| trimmed.strip_prefix("operation_id:"))
            .or_else(|| trimmed.strip_prefix("message "))
            .or_else(|| trimmed.strip_prefix("rpc "))
            .or_else(|| trimmed.strip_prefix("\"$id\":"));
        if let Some(raw) = candidate {
            let name =
                sanitize_identifier(raw.trim_matches(|ch| ch == '"' || ch == ',' || ch == '{'));
            if !name.is_empty() {
                let pointer = ArtifactPointer::new(path, selector_kind, Some(name.clone()))?;
                symbols.push(line_symbol(
                    language,
                    pointer,
                    &format!("contract::{name}"),
                    AstSymbolKind::ContractOperation,
                    source,
                    line_idx,
                    line,
                )?);
            }
        }
    }
    Ok(dedup_ast_symbols(symbols))
}

fn scan_config(path: &str, source: &str) -> Result<Vec<AstSymbol>, AdapterError> {
    let (language, selector_kind) = if path.ends_with(".toml") {
        (SymbolLanguage::Toml, ArtifactSelectorKind::TomlTable)
    } else if path.ends_with(".json") {
        (SymbolLanguage::Json, ArtifactSelectorKind::JsonPointer)
    } else if path.ends_with(".yaml") || path.ends_with(".yml") {
        (SymbolLanguage::Yaml, ArtifactSelectorKind::YamlPointer)
    } else {
        (
            SymbolLanguage::Config,
            ArtifactSelectorKind::PackageManifest,
        )
    };
    let mut symbols = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let key = if trimmed.starts_with('[') && trimmed.ends_with(']') {
            Some(
                trimmed
                    .trim_matches(|ch| ch == '[' || ch == ']')
                    .to_string(),
            )
        } else if let Some((left, _)) = trimmed.split_once(':').or_else(|| trimmed.split_once('='))
        {
            let sanitized = sanitize_identifier(left.trim_matches('"'));
            (!sanitized.is_empty()).then_some(sanitized)
        } else {
            None
        };
        if let Some(key) = key {
            let selector = if language == SymbolLanguage::Json {
                format!("/{key}")
            } else {
                key.clone()
            };
            let pointer = ArtifactPointer::new(path, selector_kind, Some(selector))?;
            symbols.push(line_symbol(
                language,
                pointer,
                &format!("config::{key}"),
                AstSymbolKind::ConfigEntry,
                source,
                line_idx,
                line,
            )?);
        }
    }
    Ok(dedup_ast_symbols(symbols))
}

fn scan_dependencies(
    symbols: &[AstSymbol],
    source: &str,
) -> Result<Vec<SymbolDependency>, AdapterError> {
    let mut dependencies = Vec::new();
    let by_tail: BTreeMap<String, SymbolId> = symbols
        .iter()
        .map(|symbol| {
            (
                symbol
                    .symbol_id
                    .symbol_path
                    .rsplit("::")
                    .next()
                    .unwrap_or("")
                    .to_string(),
                symbol.symbol_id.clone(),
            )
        })
        .collect();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some((from_name, to_name, kind)) = parse_dependency_directive(trimmed)
            && let (Some(from), Some(to)) = (by_tail.get(&from_name), by_tail.get(&to_name))
        {
            dependencies.push(SymbolDependency {
                from: from.clone(),
                to: to.clone(),
                kind,
            });
        }
    }
    Ok(dependencies)
}

fn line_symbol(
    language: SymbolLanguage,
    artifact: ArtifactPointer,
    symbol_path: &str,
    kind: AstSymbolKind,
    source: &str,
    line_idx: usize,
    line: &str,
) -> Result<AstSymbol, AdapterError> {
    let start = byte_offset_for_line(source, line_idx) as u32;
    let end = start + line.len().max(1) as u32;
    symbol_from_parts(
        language,
        artifact,
        symbol_path,
        kind,
        source,
        line_idx as u32 + 1,
        0,
        line_idx as u32 + 1,
        line.len().max(1) as u32,
    )
    .and_then(|symbol| {
        if symbol.byte_range.start == start && symbol.byte_range.end == end {
            Ok(symbol)
        } else {
            symbol_from_parts(
                language,
                symbol.artifact,
                symbol_path,
                kind,
                source,
                line_idx as u32 + 1,
                0,
                line_idx as u32 + 1,
                line.len().max(1) as u32,
            )
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn symbol_from_parts(
    language: SymbolLanguage,
    artifact: ArtifactPointer,
    symbol_path: &str,
    kind: AstSymbolKind,
    source: &str,
    start_line: u32,
    start_column: u32,
    end_line: u32,
    end_column: u32,
) -> Result<AstSymbol, AdapterError> {
    let line_start = start_line.saturating_sub(1) as usize;
    let start = byte_offset_for_line(source, line_start) as u32;
    let end = (start + end_column.max(1)).max(start + 1);
    let symbol_id = SymbolId::new(language, artifact.clone(), symbol_path)?;
    Ok(AstSymbol::new(AstSymbolDraft {
        symbol_id,
        artifact,
        kind,
        byte_range: ByteRange::new(start, end)?,
        text_range: TextRange::new(start_line, start_column, end_line, end_column)?,
        source_digest: source_digest(source),
        parser_version: PARSER_VERSION.into(),
    })?)
}

fn normalized_symbol_path(
    language: SymbolLanguage,
    path: &str,
    name: &str,
    _kind: AstSymbolKind,
) -> String {
    let namespace = path
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .split('.')
        .next()
        .unwrap_or("file");
    let name = sanitize_identifier(name);
    format!("{}::{namespace}::{name}", language.as_str())
        .replace('/', "::")
        .replace('-', "_")
}

fn fallback_symbol_path(pointer: &ArtifactPointer) -> String {
    let selector = pointer.selector.as_deref().unwrap_or("explicit");
    format!("fallback::{}", sanitize_identifier(selector))
}

fn source_digest(source: &str) -> String {
    let mut state: u64 = 0xcbf29ce484222325;
    for byte in source.as_bytes() {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(0x100000001b3);
    }
    let mut hex = String::with_capacity(71);
    hex.push_str("sha256:");
    for idx in 0..8 {
        hex.push_str(&format!(
            "{:016x}",
            state.wrapping_add((idx as u64).wrapping_mul(0x9e3779b97f4a7c15))
        ));
    }
    hex.truncate(71);
    hex
}

fn byte_offset_for_line(source: &str, line_idx: usize) -> usize {
    source
        .lines()
        .take(line_idx)
        .map(|line| line.len() + 1)
        .sum()
}

fn trim_visibility(value: &str) -> &str {
    value
        .strip_prefix("pub ")
        .or_else(|| value.strip_prefix("export "))
        .or_else(|| value.strip_prefix("public "))
        .or_else(|| value.strip_prefix("private "))
        .or_else(|| value.strip_prefix("internal "))
        .unwrap_or(value)
}

fn first_identifier(rest: &str) -> Option<String> {
    let rest = rest.trim_start_matches(|ch: char| ch.is_whitespace() || ch == '<');
    let ident: String = rest
        .chars()
        .skip_while(|ch| !is_identifier_start(*ch))
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '.')
        .collect();
    (!ident.is_empty()).then_some(sanitize_identifier(&ident))
}

fn function_name_before_paren(line: &str) -> Option<String> {
    let before = line.split('(').next()?.trim();
    let name = before.split_whitespace().last()?;
    (!name.is_empty()).then(|| sanitize_identifier(name))
}

fn attribute_value(line: &str, attr: &str) -> Option<String> {
    let start = line.find(attr)? + attr.len();
    let value = line[start..].split('"').next()?;
    (!value.is_empty()).then(|| sanitize_identifier(value))
}

fn scan_test_name(trimmed: &str) -> Option<(String, AstSymbolKind)> {
    let lowered = trimmed.to_ascii_lowercase();
    if lowered.contains("test") {
        first_identifier(trimmed).map(|name| (name, AstSymbolKind::Test))
    } else {
        None
    }
}

fn parse_dependency_directive(line: &str) -> Option<(String, String, DependencyKind)> {
    let directive = line
        .strip_prefix("// oya-dep:")
        .or_else(|| line.strip_prefix("# oya-dep:"))
        .or_else(|| line.strip_prefix("<!-- oya-dep:"))?;
    let directive = directive.trim_end_matches("-->").trim();
    let (from, rest) = directive.split_once("->")?;
    let (to, kind) = rest.split_once(':').unwrap_or((rest, "calls"));
    Some((
        sanitize_identifier(from),
        sanitize_identifier(to),
        dependency_kind(kind.trim()),
    ))
}

fn dependency_kind(value: &str) -> DependencyKind {
    match value {
        "tests" => DependencyKind::Tests,
        "imports" => DependencyKind::Imports,
        "implements" => DependencyKind::Implements,
        "extends" => DependencyKind::Extends,
        "consumes-contract" => DependencyKind::ConsumesContract,
        "xaml-binding" => DependencyKind::XamlBindingToCSharpSymbol,
        "schema-codegen" => DependencyKind::SchemaToCodegen,
        _ => DependencyKind::Calls,
    }
}

fn sanitize_identifier(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch: char| {
            ch == '(' || ch == ')' || ch == '{' || ch == '}' || ch == ';' || ch == ':' || ch == ','
        })
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn dedup_ast_symbols(symbols: Vec<AstSymbol>) -> Vec<AstSymbol> {
    let mut seen = BTreeSet::new();
    symbols
        .into_iter()
        .filter(|symbol| seen.insert(symbol.symbol_id.value.clone()))
        .collect()
}

fn dedup_symbols(symbols: Vec<SymbolId>) -> Vec<SymbolId> {
    let mut seen = BTreeSet::new();
    symbols
        .into_iter()
        .filter(|symbol| seen.insert(symbol.value.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prod(path: &str, source: &str) -> IndexedArtifact {
        index_source(IndexRequest::new(path, source, PromotionTarget::Production))
            .expect("fixture indexes")
    }

    fn names(indexed: &IndexedArtifact) -> Vec<String> {
        indexed
            .symbols
            .iter()
            .map(|symbol| symbol.symbol_id.symbol_path.clone())
            .collect()
    }

    #[test]
    fn normalizes_symbols_across_polyglot_fixture_corpus() {
        let fixtures = [
            (
                "src/lib.rs",
                "pub fn claim() {}",
                SymbolLanguage::Rust,
                "rust::lib::claim",
            ),
            (
                "web/app.ts",
                "export function route() {}",
                SymbolLanguage::TypeScript,
                "typescript::app::route",
            ),
            (
                "web/app.js",
                "function hydrate() {}",
                SymbolLanguage::JavaScript,
                "javascript::app::hydrate",
            ),
            (
                "ios/App.swift",
                "func render() {}",
                SymbolLanguage::Swift,
                "swift::App::render",
            ),
            (
                "android/App.kt",
                "fun launch() {}",
                SymbolLanguage::Kotlin,
                "kotlin::App::launch",
            ),
            (
                "win/App.cs",
                "public class Shell {}",
                SymbolLanguage::CSharp,
                "csharp::App::Shell",
            ),
            (
                "native/app.c",
                "int boot() {",
                SymbolLanguage::Cpp,
                "fn::boot",
            ),
            (
                "native/app.cpp",
                "class Engine {",
                SymbolLanguage::Cpp,
                "type::Engine",
            ),
        ];
        for (path, source, language, expected) in fixtures {
            let indexed = prod(path, source);
            assert_eq!(indexed.symbols[0].language, language);
            assert!(
                names(&indexed).contains(&expected.into()),
                "missing {expected}"
            );
        }

        let xaml = prod(
            "win/MainWindow.xaml",
            "<Window x:Class=\"Demo.MainWindow\"><Button x:Name=\"SaveButton\" Click=\"Save_Click\" /></Window>",
        );
        assert_eq!(xaml.symbols[0].language, SymbolLanguage::Xaml);
        assert!(
            names(&xaml)
                .iter()
                .any(|name| name == "xaml::Demo.MainWindow")
        );

        let schema = prod("contracts/openapi.yaml", "operationId: createClaim");
        assert_eq!(schema.symbols[0].language, SymbolLanguage::OpenApi);
        assert_eq!(schema.symbols[0].kind, AstSymbolKind::ContractOperation);

        let config = prod("config/app.toml", "[server]\nport = 8080");
        assert_eq!(config.symbols[0].language, SymbolLanguage::Toml);
        assert_eq!(config.symbols[0].kind, AstSymbolKind::ConfigEntry);
    }

    #[test]
    fn polyglot_diff_maps_dependencies_tests_and_promotion_blockers() {
        let app = prod(
            "web/app.ts",
            "function route() {}\nfunction route_test() {}\n// oya-dep:route_test->route:tests",
        );
        let changed = app
            .symbols
            .iter()
            .find(|symbol| symbol.symbol_id.symbol_path.ends_with("::route"))
            .expect("route symbol")
            .symbol_id
            .clone();

        let diff = build_diff_map(DiffInput {
            indexed_artifacts: vec![app],
            changed_symbols: vec![changed.clone()],
            target: PromotionTarget::Production,
        });

        assert_eq!(diff.changed_symbols, vec![changed]);
        assert_eq!(diff.dependency_edges.len(), 1);
        assert_eq!(diff.impacted_tests.len(), 1);
        assert!(diff.promotion_blockers.is_empty());
    }

    #[test]
    fn unsupported_production_surface_blocks() {
        let err = index_source(IndexRequest::new(
            "bin/plugin.elvish",
            "fn unknown",
            PromotionTarget::Production,
        ))
        .expect_err("unsupported production surface blocks");

        assert_eq!(
            err,
            AdapterError::UnsupportedProductionSurface {
                path: "bin/plugin.elvish".into()
            }
        );
    }

    #[test]
    fn parser_failure_without_explicit_pointer_blocks_production() {
        let err = index_source(IndexRequest::new(
            "web/app.ts",
            PARSER_FAILURE_SENTINEL,
            PromotionTarget::Production,
        ))
        .expect_err("parser failure needs pointer");

        assert_eq!(
            err,
            AdapterError::ParserFailureWithoutExplicitPointer {
                path: "web/app.ts".into()
            }
        );
    }

    #[test]
    fn parser_failure_uses_explicit_pointer_fallback() {
        let pointer = ArtifactPointer::new(
            "contracts/openapi.yaml",
            ArtifactSelectorKind::OpenApiOperation,
            Some("POST /claims".into()),
        )
        .expect("valid pointer");
        let indexed = index_source(
            IndexRequest::new(
                "contracts/openapi.yaml",
                PARSER_FAILURE_SENTINEL,
                PromotionTarget::Production,
            )
            .with_pointer_scope(pointer.clone()),
        )
        .expect("explicit pointer admits fallback");

        assert_eq!(indexed.parser_status, ParserStatus::Failed);
        assert_eq!(indexed.symbols[0].artifact, pointer);
        assert!(
            indexed.symbols[0]
                .symbol_id
                .symbol_path
                .starts_with("fallback::POST__claims")
        );
    }
}
