//! Markdown/doc parser substrate skeleton for corpus graph doc nodes.
//!
//! Accepted ADR-0517 requires one owned content-addressed AST/doc substrate;
//! proposed ADR-0541 explores documents and directives as first-class corpus
//! graph nodes but is not binding authority. This crate is the first
//! Markdown-only slice behind the accepted parser direction: it owns stable node
//! IDs for document headings and references without resolving links or executing
//! Markdown content.
//!
//! ## Threat model
//!
//! - Markdown, frontmatter, code fences, links, generated snippets, and embedded
//!   HTML are untrusted DATA, never instructions to the agent or parser.
//! - The parser never resolves external URLs, local paths, credentials, or
//!   ambient files while extracting references; link targets are recorded as
//!   strings and suspicious targets are tainted.
//! - Tenant namespaces wrap content identities externally and do not alter
//!   digest bytes; normalized source paths remain part of occurrence identity.
//! - Every node carries source path, byte span, parser version, stable ID, and a
//!   `WorkAreaNodeId` provenance bridge for auditability.
//! - Malformed frontmatter fails closed; executable HTML and exfil-like link
//!   targets are surfaced as tainted/rejected data instead of being executed.
//!
//! ## Coverage boundary
//!
//! This skeleton intentionally recognizes only ATX headings, inline links, and
//! link reference definitions. Other Markdown constructs are future parser
//! coverage, not silently claimed by this slice; additional adversarial fixtures
//! should harden encoded or obfuscated exfiltration targets before any
//! merge-blocking doc/directive invariant consumes these nodes.
//!
//! ## Dormant ADR decision IR boundary
//!
//! The typed ADR frontmatter parser below is an authority-neutral, fail-closed
//! foundation. It does not establish corpus completeness, current authority, a
//! consumer cutover, or permission to release `HOLD(Planning)`: unsupported
//! metadata is rejected rather than silently reinterpreted or dropped.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::{collections::BTreeMap, fmt};

use sha2::{Digest, Sha256};
use work_area_tree_kernel::{
    NodeContentHash, NodeLocator, SourceSpan, WorkAreaHash, WorkAreaNodeId, WorkAreaTreeError,
};

/// Parser version included in every node-id preimage.
pub const DOC_PARSER_VERSION: &str = "corpus-doc-parser-v1";

/// Pure, authority-neutral receipts over explicitly selected ADR source blobs.
///
/// This module deliberately has no filesystem, git, process, network, or clock
/// dependency. An admission layer must select and bind immutable inputs before
/// calling [`census::build_receipt`]; a receipt never promotes that selection,
/// changes authority, releases `HOLD(Planning)`, or dispatches a roadmap.
pub mod census {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fmt;

    use sha2::{Digest, Sha256};

    use super::{
        AdrByteSpan, AdrParseError, AdrParseInput, DOC_PARSER_VERSION, parse_adr_decision,
    };

    /// Stable identity for direct `docs/decisions/ADR-*.md` source selection.
    pub const SELECTOR_ID: &str = "docs-decisions-direct-adr-v1";
    const PARSER_API: &str = "corpus-doc-parser::parse_adr_decision";
    const DIAGNOSTIC_POLICY: &str = "first-error-only";
    const COMPILED_PARSER_SOURCES: [(&str, &[u8]); 1] = [(
        "governance/corpus/doc-parser/src/lib.rs",
        include_bytes!("lib.rs"),
    )];

    /// One already-selected immutable blob. The builder does not retrieve it.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CensusSource {
        pub kind: CensusSourceKind,
        pub path: String,
        pub blob_oid: String,
        pub bytes: Vec<u8>,
    }

    /// Role played by a caller-supplied source blob.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CensusSourceKind {
        Decision,
        Parser,
    }

    /// Complete immutable input to the pure receipt builder.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CensusInput {
        pub repository_commit: String,
        pub repository_tree: String,
        pub docs_tree: String,
        pub selector_id: String,
        pub parser_commit: String,
        pub parser_sources: Vec<CensusSource>,
        pub decision_sources: Vec<CensusSource>,
    }

    /// Fail-closed structural violations for a proposed receipt.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CensusViolation {
        InvalidObjectId,
        SelectorPath,
        DuplicatePath,
        SourceKind,
        ParserSource,
    }

    impl fmt::Display for CensusViolation {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                Self::InvalidObjectId => "invalid immutable git object id",
                Self::SelectorPath => "source path is outside the direct ADR selector",
                Self::DuplicatePath => "selector produced duplicate source paths",
                Self::SourceKind => "source has the wrong census role",
                Self::ParserSource => "parser source set is invalid",
            })
        }
    }

    impl std::error::Error for CensusViolation {}

    /// First parser error retained as diagnostic data only.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FirstError {
        kind: String,
        span: Option<(u64, u64)>,
        raw: String,
    }

    impl FirstError {
        #[must_use]
        pub fn kind(&self) -> &str {
            &self.kind
        }
        #[must_use]
        pub const fn span(&self) -> Option<(u64, u64)> {
            self.span
        }
        #[must_use]
        pub fn raw(&self) -> &str {
            &self.raw
        }
    }

    /// One selected ADR and its deterministic parse outcome.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CensusEntry {
        path: String,
        blob_oid: String,
        sha256: String,
        outcome: String,
        first_error: Option<FirstError>,
    }

    impl CensusEntry {
        #[must_use]
        pub fn path(&self) -> &str {
            &self.path
        }
        #[must_use]
        pub fn blob_oid(&self) -> &str {
            &self.blob_oid
        }
        #[must_use]
        pub fn sha256(&self) -> &str {
            &self.sha256
        }
        #[must_use]
        pub fn outcome(&self) -> &str {
            &self.outcome
        }
        #[must_use]
        pub fn first_error(&self) -> Option<&FirstError> {
            self.first_error.as_ref()
        }
    }

    /// Deterministic non-authoritative census receipt.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CensusReceipt {
        repository_commit: String,
        repository_tree: String,
        docs_tree: String,
        selector_id: String,
        parser_commit: String,
        entries: Vec<CensusEntry>,
        parsed_count: usize,
        rejected_count: usize,
        first_error_kind_totals: BTreeMap<String, usize>,
        aggregate_fold: String,
        canonical_digest: String,
        canonical_bytes: Vec<u8>,
    }

    impl CensusReceipt {
        #[must_use]
        pub fn repository_commit(&self) -> &str {
            &self.repository_commit
        }
        #[must_use]
        pub fn repository_tree(&self) -> &str {
            &self.repository_tree
        }
        #[must_use]
        pub fn docs_tree(&self) -> &str {
            &self.docs_tree
        }
        #[must_use]
        pub fn selector_id(&self) -> &str {
            &self.selector_id
        }
        #[must_use]
        pub fn parser_commit(&self) -> &str {
            &self.parser_commit
        }
        #[must_use]
        pub fn entries(&self) -> &[CensusEntry] {
            &self.entries
        }
        #[must_use]
        pub const fn parsed_count(&self) -> usize {
            self.parsed_count
        }
        #[must_use]
        pub const fn rejected_count(&self) -> usize {
            self.rejected_count
        }
        #[must_use]
        pub fn first_error_kind_totals(&self) -> &BTreeMap<String, usize> {
            &self.first_error_kind_totals
        }
        #[must_use]
        pub fn aggregate_fold(&self) -> &str {
            &self.aggregate_fold
        }
        #[must_use]
        pub fn canonical_digest(&self) -> &str {
            &self.canonical_digest
        }
        #[must_use]
        pub fn canonical_bytes(&self) -> &[u8] {
            &self.canonical_bytes
        }
        #[must_use]
        pub const fn claim_ceiling(&self) -> &'static str {
            "BLOCKED/HOLD"
        }
    }

    /// Builds a receipt solely from explicit, immutable caller-supplied blobs.
    ///
    /// # Errors
    /// Returns a fail-closed violation for malformed bindings, wrong roles,
    /// duplicate paths, selectors outside the exact direct-child ADR shape, or
    /// parser source bytes not equal to this compiled module.
    pub fn build_receipt(input: &CensusInput) -> Result<CensusReceipt, CensusViolation> {
        for oid in [
            &input.repository_commit,
            &input.repository_tree,
            &input.docs_tree,
            &input.parser_commit,
        ] {
            if !is_lower_hex_oid(oid) {
                return Err(CensusViolation::InvalidObjectId);
            }
        }
        if input.selector_id != SELECTOR_ID {
            return Err(CensusViolation::SelectorPath);
        }
        let parser_sources = validate_parser_sources(&input.parser_sources)?;

        let mut sources = input.decision_sources.clone();
        sources.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
        let mut paths = BTreeSet::new();
        let mut entries = Vec::with_capacity(sources.len());
        for source in sources {
            if source.kind != CensusSourceKind::Decision {
                return Err(CensusViolation::SourceKind);
            }
            if !is_direct_adr_path(&source.path) {
                return Err(CensusViolation::SelectorPath);
            }
            if !is_lower_hex_oid(&source.blob_oid) {
                return Err(CensusViolation::InvalidObjectId);
            }
            if !paths.insert(source.path.clone()) {
                return Err(CensusViolation::DuplicatePath);
            }
            entries.push(parse_entry(source));
        }

        let parsed_count = entries
            .iter()
            .filter(|entry| entry.outcome == "parsed")
            .count();
        let rejected_count = entries.len() - parsed_count;
        let mut error_totals = BTreeMap::<String, usize>::new();
        for entry in &entries {
            if let Some(error) = &entry.first_error {
                *error_totals.entry(error.kind.clone()).or_default() += 1;
            }
        }
        let entry_folds = entries.iter().map(entry_json).collect::<Vec<_>>();
        let aggregate_fold = aggregate_entry_folds(entry_folds.iter().map(String::as_str));
        let body = canonical_body(
            input,
            &parser_sources,
            &entries,
            parsed_count,
            rejected_count,
            &error_totals,
            &aggregate_fold,
        );
        let canonical_digest = sha256_hex(std::iter::once(body.as_bytes()));
        let canonical_bytes =
            format!("{{{body},\"canonical_digest\":\"{canonical_digest}\"}}\n").into_bytes();
        Ok(CensusReceipt {
            repository_commit: input.repository_commit.clone(),
            repository_tree: input.repository_tree.clone(),
            docs_tree: input.docs_tree.clone(),
            selector_id: input.selector_id.clone(),
            parser_commit: input.parser_commit.clone(),
            entries,
            parsed_count,
            rejected_count,
            first_error_kind_totals: error_totals,
            aggregate_fold,
            canonical_digest,
            canonical_bytes,
        })
    }

    fn validate_parser_sources(
        sources: &[CensusSource],
    ) -> Result<Vec<CensusSource>, CensusViolation> {
        if sources.len() != COMPILED_PARSER_SOURCES.len() {
            return Err(CensusViolation::ParserSource);
        }
        let mut normalized = sources.to_vec();
        normalized.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
        for (source, (path, bytes)) in normalized.iter().zip(COMPILED_PARSER_SOURCES) {
            if source.kind != CensusSourceKind::Parser
                || source.path != path
                || !is_lower_hex_oid(&source.blob_oid)
                || source.bytes.as_slice() != bytes
            {
                return Err(CensusViolation::ParserSource);
            }
        }
        Ok(normalized)
    }

    fn parse_entry(source: CensusSource) -> CensusEntry {
        let sha256 = sha256_hex(std::iter::once(source.bytes.as_slice()));
        let parsed = std::str::from_utf8(&source.bytes)
            .map_err(|_| FirstError {
                kind: "NonUtf8".to_owned(),
                span: None,
                raw: "ADR blob is not valid UTF-8".to_owned(),
            })
            .and_then(|text| {
                parse_adr_decision(&AdrParseInput::new(&source.path, text)).map_err(first_error)
            });
        match parsed {
            Ok(_) => CensusEntry {
                path: source.path,
                blob_oid: source.blob_oid,
                sha256,
                outcome: "parsed".to_owned(),
                first_error: None,
            },
            Err(error) => CensusEntry {
                path: source.path,
                blob_oid: source.blob_oid,
                sha256,
                outcome: "rejected".to_owned(),
                first_error: Some(error),
            },
        }
    }

    fn first_error(error: AdrParseError) -> FirstError {
        let (kind, span) = match &error {
            AdrParseError::MissingLeadingFrontmatter => ("MissingLeadingFrontmatter", None),
            AdrParseError::UnterminatedFrontmatter => ("UnterminatedFrontmatter", None),
            AdrParseError::DuplicateFrontmatterKey { span, .. } => {
                ("DuplicateFrontmatterKey", Some(*span))
            }
            AdrParseError::InvalidFrontmatter { span, .. } => ("InvalidFrontmatter", Some(*span)),
            AdrParseError::UnsupportedFrontmatterNesting { span } => {
                ("UnsupportedFrontmatterNesting", Some(*span))
            }
            AdrParseError::InvalidAdrReference { span, .. } => ("InvalidAdrReference", Some(*span)),
            AdrParseError::InvalidAdrId { span, .. } => ("InvalidAdrId", Some(*span)),
            AdrParseError::AdrIdPathMismatch { .. } => ("AdrIdPathMismatch", None),
            AdrParseError::InvalidDate { span, .. } => ("InvalidDate", Some(*span)),
            AdrParseError::InvalidSourcePath { .. } => ("InvalidSourcePath", None),
            AdrParseError::InvalidAdrHeading { span, .. } => ("InvalidAdrHeading", Some(*span)),
            AdrParseError::MissingRequiredField { .. } => ("MissingRequiredField", None),
            AdrParseError::InvalidTenant { .. } => ("InvalidTenant", None),
        };
        FirstError {
            kind: kind.to_owned(),
            span: span.map(|value| (value.start(), value.end())),
            raw: error.to_string(),
        }
    }

    fn canonical_body(
        input: &CensusInput,
        parser_sources: &[CensusSource],
        entries: &[CensusEntry],
        parsed_count: usize,
        rejected_count: usize,
        error_totals: &BTreeMap<String, usize>,
        aggregate_fold: &str,
    ) -> String {
        let parser_hashes = parser_sources
            .iter()
            .map(source_digest)
            .map(|value| json_string(&value))
            .collect::<Vec<_>>()
            .join(",");
        let entries = entries.iter().map(entry_json).collect::<Vec<_>>().join(",");
        let errors = error_totals
            .iter()
            .map(|(kind, count)| format!("{}:{count}", json_string(kind)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "\"aggregate_fold\":{},\"claim_ceiling\":{},\"diagnostic_policy\":{},\"docs_tree\":{},\"entries\":[{entries}],\"first_error_kinds\":{{{errors}}},\"parser_api\":{},\"parser_commit\":{},\"parser_source_hashes\":[{parser_hashes}],\"parser_version\":{},\"repository_commit\":{},\"repository_tree\":{},\"selector\":{},\"totals\":{{\"parsed\":{parsed_count},\"rejected\":{rejected_count}}}",
            json_string(aggregate_fold),
            json_string("BLOCKED/HOLD"),
            json_string(DIAGNOSTIC_POLICY),
            json_string(&input.docs_tree),
            json_string(PARSER_API),
            json_string(&input.parser_commit),
            json_string(DOC_PARSER_VERSION),
            json_string(&input.repository_commit),
            json_string(&input.repository_tree),
            json_string(&input.selector_id),
        )
    }

    fn entry_json(entry: &CensusEntry) -> String {
        let error = entry.first_error.as_ref().map_or_else(
            || "null".to_owned(),
            |error| {
                let span = error.span.map_or_else(
                    || "null".to_owned(),
                    |(start, end)| format!("[{start},{end}]"),
                );
                format!(
                    "{{\"kind\":{},\"raw\":{},\"span\":{span}}}",
                    json_string(&error.kind),
                    json_string(&error.raw)
                )
            },
        );
        format!(
            "{{\"blob_oid\":{},\"first_error\":{error},\"outcome\":{},\"path\":{},\"sha256\":{}}}",
            json_string(&entry.blob_oid),
            json_string(&entry.outcome),
            json_string(&entry.path),
            json_string(&entry.sha256)
        )
    }

    fn aggregate_entry_folds<'a>(entry_folds: impl Iterator<Item = &'a str>) -> String {
        let mut digest = Sha256::new();
        digest.update(b"oyatie:census:entry-fold:v1\\0");
        for entry_fold in entry_folds {
            let bytes = entry_fold.as_bytes();
            digest.update((bytes.len() as u64).to_be_bytes());
            digest.update(bytes);
        }
        format!("{:x}", digest.finalize())
    }

    fn source_digest(source: &CensusSource) -> String {
        format!(
            "{}:{}:{}",
            source.path,
            source.blob_oid,
            sha256_hex(std::iter::once(source.bytes.as_slice()))
        )
    }

    fn json_string(value: &str) -> String {
        use std::fmt::Write;
        let mut out = String::with_capacity(value.len() + 2);
        out.push('"');
        for character in value.chars() {
            match character {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                character if character <= '\u{1f}' => {
                    let _ = write!(out, "\\u{:04x}", character as u32);
                }
                character => out.push(character),
            }
        }
        out.push('"');
        out
    }

    fn is_direct_adr_path(path: &str) -> bool {
        path.starts_with("docs/decisions/ADR-")
            && path.ends_with(".md")
            && !path["docs/decisions/".len()..].contains('/')
            && !path.contains("..")
            && path.is_ascii()
    }

    fn is_lower_hex_oid(value: &str) -> bool {
        value.len() == 40
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }

    fn sha256_hex<'a>(parts: impl Iterator<Item = &'a [u8]>) -> String {
        let mut digest = Sha256::new();
        for part in parts {
            digest.update(part);
        }
        format!("{:x}", digest.finalize())
    }
}

/// Input to the Markdown document parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocParseInput {
    tenant_id: String,   // data_class: INTERNAL_ONLY
    source_path: String, // data_class: INTERNAL_ONLY
    source: String,      // data_class: INTERNAL_ONLY
}

impl DocParseInput {
    #[must_use]
    pub fn new(
        tenant_id: impl Into<String>,
        source_path: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            source_path: source_path.into(),
            source: source.into(),
        }
    }

    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    #[must_use]
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Parsed document tree slice.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedDocument {
    nodes: Vec<DocNode>, // data_class: INTERNAL_ONLY
}

impl ParsedDocument {
    #[must_use]
    pub fn nodes(&self) -> &[DocNode] {
        &self.nodes
    }
}

/// One extracted document node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocNode {
    stable_id: String,                 // data_class: INTERNAL_ONLY
    work_area_node_id: WorkAreaNodeId, // data_class: INTERNAL_ONLY
    kind: DocNodeKind,                 // data_class: INTERNAL_ONLY
    text: String,                      // data_class: INTERNAL_ONLY
    target: Option<String>,            // data_class: INTERNAL_ONLY
    span: (u64, u64),                  // data_class: INTERNAL_ONLY
    taint: Option<TaintReason>,        // data_class: INTERNAL_ONLY
}

impl DocNode {
    #[must_use]
    pub fn stable_id(&self) -> &str {
        &self.stable_id
    }

    #[must_use]
    pub const fn work_area_node_id(&self) -> &WorkAreaNodeId {
        &self.work_area_node_id
    }

    #[must_use]
    pub const fn kind(&self) -> &DocNodeKind {
        &self.kind
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    #[must_use]
    pub const fn span(&self) -> (u64, u64) {
        self.span
    }

    #[must_use]
    pub const fn taint(&self) -> Option<&TaintReason> {
        self.taint.as_ref()
    }
}

/// Document node kind vocabulary for the Markdown slice.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocNodeKind {
    Document,
    Heading { level: u8 },
    Reference,
    Rejected,
}

impl DocNodeKind {
    fn stable_tag(&self) -> String {
        match self {
            Self::Document => "document".to_owned(),
            Self::Heading { level } => format!("heading:{level}"),
            Self::Reference => "reference".to_owned(),
            Self::Rejected => "rejected".to_owned(),
        }
    }
}

/// Taint/rejection reason surfaced as data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaintReason {
    ForbiddenLinkTarget,
    ExecutableHtml,
}

impl TaintReason {
    const fn stable_tag(&self) -> &'static str {
        match self {
            Self::ForbiddenLinkTarget => "forbidden-link-target",
            Self::ExecutableHtml => "executable-html",
        }
    }
}

/// Parser errors. Fatal errors fail closed and produce no graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocParseError {
    InvalidSourcePath(String),
    MalformedFrontmatter,
    NodeIdentity(WorkAreaTreeError),
}

impl fmt::Display for DocParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSourcePath(path) => write!(f, "invalid document source path {path:?}"),
            Self::MalformedFrontmatter => write!(f, "malformed markdown frontmatter"),
            Self::NodeIdentity(error) => write!(f, "document node identity failure: {error}"),
        }
    }
}

impl std::error::Error for DocParseError {}

impl From<WorkAreaTreeError> for DocParseError {
    fn from(value: WorkAreaTreeError) -> Self {
        Self::NodeIdentity(value)
    }
}

/// Parse headings and references from Markdown into content-addressed doc nodes.
///
/// # Errors
/// Returns [`DocParseError`] when provenance is invalid or frontmatter is malformed.
pub fn parse_markdown_doc(input: &DocParseInput) -> Result<ParsedDocument, DocParseError> {
    let content_start = content_start_after_frontmatter(input.source())?;
    validate_document_source_path(input.source_path())?;
    let work_area_hash = WorkAreaHash::from_bytes(sha256_frame(&[
        b"work-area",
        DOC_PARSER_VERSION.as_bytes(),
        input.source_path().as_bytes(),
        input.source().as_bytes(),
    ]));

    let mut nodes = Vec::new();
    if !input.source().is_empty() {
        nodes.push(build_node(
            input,
            work_area_hash,
            NodeDraft::new(DocNodeKind::Document, 0, input.source().len()),
        )?);
    }

    let mut cursor = content_start;
    let mut in_code_fence = false;
    while cursor < input.source().len() {
        let (line, line_start, next_cursor) = next_line(input.source(), cursor);
        let trimmed = line.trim();
        if is_code_fence(trimmed) {
            in_code_fence = !in_code_fence;
            cursor = next_cursor;
            continue;
        }

        if !in_code_fence {
            if let Some((level, title)) = parse_atx_heading(line) {
                nodes.push(build_node(
                    input,
                    work_area_hash,
                    NodeDraft::new(
                        DocNodeKind::Heading { level },
                        line_start,
                        line_start + line.len(),
                    )
                    .with_text(title),
                )?);
            }

            if contains_executable_html(trimmed) {
                nodes.push(build_node(
                    input,
                    work_area_hash,
                    NodeDraft::new(DocNodeKind::Rejected, line_start, line_start + line.len())
                        .with_text(trimmed.to_owned())
                        .with_taint(TaintReason::ExecutableHtml),
                )?);
            }

            for reference in parse_references(line) {
                let taint = is_forbidden_link_target(&reference.target)
                    .then_some(TaintReason::ForbiddenLinkTarget);
                nodes.push(build_node(
                    input,
                    work_area_hash,
                    NodeDraft::new(
                        DocNodeKind::Reference,
                        line_start + reference.start,
                        line_start + reference.end,
                    )
                    .with_text(reference.text)
                    .with_target(reference.target)
                    .with_optional_taint(taint),
                )?);
            }
        }

        cursor = next_cursor;
    }

    Ok(ParsedDocument { nodes })
}

/// Validate provenance even when the source produces no document nodes.
///
/// `NodeLocator` owns the canonical repository-relative path contract. The
/// temporary span is only used to invoke that contract and is never surfaced
/// in a parsed document.
fn validate_document_source_path(source_path: &str) -> Result<(), DocParseError> {
    let validation_span = SourceSpan::new(0, 1)?;
    match NodeLocator::new(source_path, validation_span) {
        Ok(_) => Ok(()),
        Err(WorkAreaTreeError::InvalidArtifactPath(path)) => {
            Err(DocParseError::InvalidSourcePath(path))
        }
        Err(error) => Err(DocParseError::NodeIdentity(error)),
    }
}

fn content_start_after_frontmatter(source: &str) -> Result<usize, DocParseError> {
    if !source.starts_with("---\n") && !source.starts_with("---\r\n") {
        return Ok(0);
    }

    let mut cursor = next_line(source, 0).2;
    while cursor < source.len() {
        let (line, _line_start, next_cursor) = next_line(source, cursor);
        if line.trim_end_matches('\r') == "---" {
            return Ok(next_cursor);
        }
        cursor = next_cursor;
    }
    Err(DocParseError::MalformedFrontmatter)
}

fn next_line(source: &str, cursor: usize) -> (&str, usize, usize) {
    let remainder = &source[cursor..];
    if let Some(newline) = remainder.find('\n') {
        let line_end = cursor + newline;
        (&source[cursor..line_end], cursor, line_end + 1)
    } else {
        (remainder, cursor, source.len())
    }
}

fn is_code_fence(trimmed: &str) -> bool {
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

fn parse_atx_heading(line: &str) -> Option<(u8, String)> {
    let bytes = line.as_bytes();
    let mut level = 0_usize;
    while level < bytes.len() && bytes[level] == b'#' && level < 6 {
        level += 1;
    }
    if level == 0
        || bytes
            .get(level)
            .is_none_or(|byte| !byte.is_ascii_whitespace())
    {
        return None;
    }
    let title = line[level..]
        .trim()
        .trim_end_matches('#')
        .trim_end()
        .to_owned();
    if title.is_empty() {
        None
    } else {
        Some((level as u8, title))
    }
}

fn contains_executable_html(trimmed: &str) -> bool {
    let lower = trimmed.to_ascii_lowercase();
    lower.contains("<script") || lower.contains("javascript:")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedReference {
    text: String,
    target: String,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeDraft {
    kind: DocNodeKind,
    text: String,
    target: Option<String>,
    start: usize,
    end: usize,
    taint: Option<TaintReason>,
}

impl NodeDraft {
    fn new(kind: DocNodeKind, start: usize, end: usize) -> Self {
        Self {
            kind,
            text: String::new(),
            target: None,
            start,
            end,
            taint: None,
        }
    }

    fn with_text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    fn with_taint(mut self, taint: TaintReason) -> Self {
        self.taint = Some(taint);
        self
    }

    fn with_optional_taint(mut self, taint: Option<TaintReason>) -> Self {
        self.taint = taint;
        self
    }
}

fn parse_references(line: &str) -> Vec<ParsedReference> {
    let mut references = parse_reference_definition(line);
    let bytes = line.as_bytes();
    let mut cursor = 0;
    while cursor < line.len() {
        let Some(open_rel) = line[cursor..].find('[') else {
            break;
        };
        let open = cursor + open_rel;
        if open > 0 && bytes[open - 1] == b'!' {
            cursor = open + 1;
            continue;
        }
        let Some(close_rel) = line[open + 1..].find(']') else {
            break;
        };
        let close = open + 1 + close_rel;
        if bytes.get(close + 1) != Some(&b'(') {
            cursor = close + 1;
            continue;
        }
        let target_start = close + 2;
        let Some(target_end_rel) = line[target_start..].find(')') else {
            break;
        };
        let target_end = target_start + target_end_rel;
        let text = line[open + 1..close].trim().to_owned();
        let target = line[target_start..target_end].trim().to_owned();
        if !text.is_empty() && !target.is_empty() {
            references.push(ParsedReference {
                text,
                target,
                start: open,
                end: target_end + 1,
            });
        }
        cursor = target_end + 1;
    }
    references
}

fn parse_reference_definition(line: &str) -> Vec<ParsedReference> {
    let trimmed_start = line.trim_start();
    let leading_ws = line.len() - trimmed_start.len();
    if !trimmed_start.starts_with('[') {
        return Vec::new();
    }
    let Some(close_rel) = trimmed_start.find("]:") else {
        return Vec::new();
    };
    let text = trimmed_start[1..close_rel].trim();
    let target_start = close_rel + 2;
    let target_tail = &trimmed_start[target_start..];
    let target_leading_ws = target_tail
        .char_indices()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx))
        .unwrap_or(target_tail.len());
    let target = target_tail[target_leading_ws..]
        .split_whitespace()
        .next()
        .unwrap_or_default();
    if text.is_empty() || target.is_empty() {
        return Vec::new();
    }
    vec![ParsedReference {
        text: text.to_owned(),
        target: target.to_owned(),
        start: leading_ws,
        end: leading_ws + target_start + target_leading_ws + target.len(),
    }]
}

fn is_forbidden_link_target(target: &str) -> bool {
    let normalized = target.trim().to_ascii_lowercase();
    normalized.starts_with("file:")
        || normalized.starts_with("javascript:")
        || normalized.starts_with("env:")
        || normalized.starts_with("secret:")
        || normalized.starts_with('/')
        || normalized.starts_with('~')
        || normalized.contains("169.254.169.254")
        || normalized.contains("/.ssh/")
        || normalized.contains("keychain")
}

fn build_node(
    input: &DocParseInput,
    work_area_hash: WorkAreaHash,
    draft: NodeDraft,
) -> Result<DocNode, DocParseError> {
    let NodeDraft {
        kind,
        text,
        target,
        start,
        end,
        taint,
    } = draft;
    let span = SourceSpan::new(start as u64, end as u64)?;
    let locator = match NodeLocator::new(input.source_path(), span) {
        Ok(locator) => locator,
        Err(WorkAreaTreeError::InvalidArtifactPath(path)) => {
            return Err(DocParseError::InvalidSourcePath(path));
        }
        Err(error) => return Err(DocParseError::NodeIdentity(error)),
    };
    let kind_tag = kind.stable_tag();
    let target_for_hash = target.as_deref().unwrap_or_default();
    let taint_for_hash = taint
        .as_ref()
        .map(TaintReason::stable_tag)
        .unwrap_or("clean");
    let span_for_hash = format!("{start}:{end}");
    let node_hash = NodeContentHash::from_bytes(sha256_frame(&[
        b"node",
        DOC_PARSER_VERSION.as_bytes(),
        input.source_path().as_bytes(),
        kind_tag.as_bytes(),
        text.as_bytes(),
        target_for_hash.as_bytes(),
        taint_for_hash.as_bytes(),
    ]));
    let stable_id = format!(
        "docnode:v1:sha256:{}:{}:{}",
        work_area_hash.to_hex(),
        node_hash.to_hex(),
        span_for_hash
    );
    let work_area_node_id = WorkAreaNodeId::new(work_area_hash, node_hash, locator);
    Ok(DocNode {
        stable_id,
        work_area_node_id,
        kind,
        text,
        target,
        span: (start as u64, end as u64),
        taint,
    })
}

fn sha256_frame(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for part in parts {
        let len = (part.len() as u64).to_be_bytes();
        hasher.update(len);
        hasher.update(part);
    }
    hasher.finalize().into()
}

/// Input for parsing one ADR source document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrParseInput {
    source_path: String,
    source: String,
}

impl AdrParseInput {
    #[must_use]
    pub fn new(source_path: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            source: source.into(),
        }
    }

    #[must_use]
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// A byte range into the original, unmodified source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrByteSpan {
    start: u64,
    end: u64,
}

impl AdrByteSpan {
    #[must_use]
    pub const fn start(self) -> u64 {
        self.start
    }
    #[must_use]
    pub const fn end(self) -> u64 {
        self.end
    }
}

/// Parsed scalar or flat list frontmatter value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdrFrontmatterValue {
    Scalar(String),
    List(Vec<String>),
    Null,
    Empty,
}

/// A retained frontmatter field, including its source spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrFrontmatterField {
    key: String,
    raw_value: String,
    value_span: AdrByteSpan,
    value: AdrFrontmatterValue,
}

impl AdrFrontmatterField {
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }
    #[must_use]
    pub fn raw_value(&self) -> &str {
        &self.raw_value
    }
    #[must_use]
    pub const fn value_span(&self) -> AdrByteSpan {
        self.value_span
    }
    #[must_use]
    pub const fn value(&self) -> &AdrFrontmatterValue {
        &self.value
    }
}

/// A canonical ADR identifier (`ADR-` followed by exactly four ASCII digits).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrId(String);

impl AdrId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Numeric component of this canonical `ADR-NNNN` identifier.
    #[must_use]
    pub fn number(&self) -> u16 {
        let digits = &self.0[4..];
        let mut number = 0_u16;
        for byte in digits.bytes() {
            number = number
                .saturating_mul(10)
                .saturating_add(u16::from(byte - b'0'));
        }
        number
    }
}

/// Compatibility name for a validated canonical ADR identifier.
pub type CanonicalAdrId = AdrId;

/// A typed relationship edge with a canonical identifier and source-field span.
///
/// Item spelling is normalized to the exact `ADR-NNNN` token. The containing
/// [`AdrFrontmatterField`] retains the complete raw spelling, quotes, comments,
/// and byte span for provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrReference {
    id: AdrId,
    raw_value: String,
    field_span: AdrByteSpan,
}

impl AdrReference {
    #[must_use]
    pub const fn id(&self) -> &AdrId {
        &self.id
    }
    #[must_use]
    pub fn raw_value(&self) -> &str {
        &self.raw_value
    }
    #[must_use]
    pub const fn field_span(&self) -> AdrByteSpan {
        self.field_span
    }
}

/// A named group of affected surface values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrAffectedSurface {
    category: String,
    values: Vec<String>,
}

impl AdrAffectedSurface {
    #[must_use]
    pub fn category(&self) -> &str {
        &self.category
    }
    #[must_use]
    pub fn values(&self) -> &[String] {
        &self.values
    }
}

/// One ADR delivery commitment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrDeliverable {
    id: String,
    description: Option<String>,
    exit_criteria: Option<String>,
    verified_by: Option<String>,
}

impl AdrDeliverable {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    #[must_use]
    pub fn exit_criteria(&self) -> Option<&str> {
        self.exit_criteria.as_deref()
    }
    #[must_use]
    pub fn verified_by(&self) -> Option<&str> {
        self.verified_by.as_deref()
    }
}

/// SHA-256 digest of canonical ADR source bytes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrContentHash([u8; 32]);

impl AdrContentHash {
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

/// Namespace associated externally with an immutable ADR decision.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrTenant(String);

impl AdrTenant {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Tenant-scoped identity that leaves content hashing unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrTenantIdentity {
    tenant: AdrTenant,
    content_hash: AdrContentHash,
}

impl AdrTenantIdentity {
    #[must_use]
    pub const fn tenant(&self) -> &AdrTenant {
        &self.tenant
    }
    #[must_use]
    pub const fn content_hash(&self) -> &AdrContentHash {
        &self.content_hash
    }
}

/// Externally namespaced ADR decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantAdrDecision {
    decision: AdrDecision,
    identity: AdrTenantIdentity,
}

impl TenantAdrDecision {
    #[must_use]
    pub const fn decision(&self) -> &AdrDecision {
        &self.decision
    }
    #[must_use]
    pub const fn identity(&self) -> &AdrTenantIdentity {
        &self.identity
    }
}

/// A complete parsed ADR decision IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrDecision {
    source_path: String,
    id: AdrId,
    title: String,
    status: String,
    date: String,
    owner: String,
    fields: Vec<AdrFrontmatterField>,
    edges: BTreeMap<String, Vec<AdrReference>>,
    affected_surfaces: Vec<AdrAffectedSurface>,
    deliverables: Vec<AdrDeliverable>,
    canonical_bytes: Vec<u8>,
    content_hash: AdrContentHash,
}

/// Compatibility name for the dormant canonical ADR decision representation.
pub type AdrDecisionIr = AdrDecision;

impl AdrDecision {
    #[must_use]
    pub fn source_path(&self) -> &str {
        &self.source_path
    }
    #[must_use]
    pub const fn id(&self) -> &AdrId {
        &self.id
    }
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }
    #[must_use]
    pub fn date(&self) -> &str {
        &self.date
    }
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }
    #[must_use]
    pub fn fields(&self) -> &[AdrFrontmatterField] {
        &self.fields
    }
    #[must_use]
    pub fn field(&self, key: &str) -> Option<&AdrFrontmatterField> {
        self.fields.iter().find(|field| field.key == key)
    }
    #[must_use]
    pub fn depends_on(&self) -> &[AdrReference] {
        self.edge("depends_on")
    }
    #[must_use]
    pub fn supersedes(&self) -> &[AdrReference] {
        self.edge("supersedes")
    }
    #[must_use]
    pub fn superseded_by(&self) -> &[AdrReference] {
        self.edge("superseded_by")
    }
    #[must_use]
    pub fn amends(&self) -> &[AdrReference] {
        self.edge("amends")
    }
    #[must_use]
    pub fn amended_by(&self) -> &[AdrReference] {
        self.edge("amended_by")
    }
    #[must_use]
    pub fn related(&self) -> &[AdrReference] {
        self.edge("related")
    }
    #[must_use]
    pub fn affected_surfaces(&self) -> &[AdrAffectedSurface] {
        &self.affected_surfaces
    }
    #[must_use]
    pub fn deliverables(&self) -> &[AdrDeliverable] {
        &self.deliverables
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub const fn content_hash(&self) -> &AdrContentHash {
        &self.content_hash
    }

    /// Wrap this content identity in a tenant namespace without reparsing it.
    ///
    /// # Errors
    /// Returns [`AdrParseError::InvalidTenant`] for an empty or malformed tenant namespace.
    pub fn within_tenant(
        self,
        tenant: impl Into<String>,
    ) -> Result<TenantAdrDecision, AdrParseError> {
        let tenant = tenant.into();
        if tenant.is_empty()
            || tenant
                .bytes()
                .any(|byte| byte.is_ascii_whitespace() || byte == b'/')
        {
            return Err(AdrParseError::InvalidTenant { tenant });
        }
        Ok(TenantAdrDecision {
            identity: AdrTenantIdentity {
                tenant: AdrTenant(tenant),
                content_hash: self.content_hash.clone(),
            },
            decision: self,
        })
    }

    fn edge(&self, key: &str) -> &[AdrReference] {
        self.edges.get(key).map_or(&[], Vec::as_slice)
    }
}

/// One uninterpreted top-level ADR frontmatter field retained for provenance.
///
/// This type deliberately exposes only the original field spelling and its byte
/// range. Consumers must not infer authority, lifecycle, or decision semantics
/// from opaque metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrOpaqueFrontmatterField {
    key: String,
    span: AdrByteSpan,
    raw_bytes: Vec<u8>,
}

impl AdrOpaqueFrontmatterField {
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    #[must_use]
    pub const fn span(&self) -> AdrByteSpan {
        self.span
    }

    #[must_use]
    pub fn raw_bytes(&self) -> &[u8] {
        &self.raw_bytes
    }
}

/// Minimal, authority-neutral envelope for ADR provenance consumers.
///
/// The envelope authenticates complete source bytes and projects only `id`,
/// `status`, and ordered `supersedes` references. Every other top-level field
/// remains opaque source data; this is intentionally not an [`AdrDecision`] and
/// must never be used to populate the strict decision IR or its census.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrAuthorityEnvelope {
    source_path: String,
    frontmatter_span: AdrByteSpan,
    id: AdrId,
    status: String,
    supersedes: Vec<AdrReference>,
    opaque_fields: Vec<AdrOpaqueFrontmatterField>,
    canonical_bytes: Vec<u8>,
    content_hash: AdrContentHash,
}

impl AdrAuthorityEnvelope {
    #[must_use]
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    /// Exact byte range of the leading frontmatter, including both fences.
    #[must_use]
    pub const fn frontmatter_span(&self) -> AdrByteSpan {
        self.frontmatter_span
    }

    #[must_use]
    pub const fn id(&self) -> &AdrId {
        &self.id
    }

    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    #[must_use]
    pub fn supersedes(&self) -> &[AdrReference] {
        &self.supersedes
    }

    #[must_use]
    pub fn opaque_fields(&self) -> &[AdrOpaqueFrontmatterField] {
        &self.opaque_fields
    }

    #[must_use]
    pub fn opaque_field(&self, key: &str) -> Option<&AdrOpaqueFrontmatterField> {
        self.opaque_fields.iter().find(|field| field.key == key)
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[must_use]
    pub const fn content_hash(&self) -> &AdrContentHash {
        &self.content_hash
    }
}

#[derive(Debug, Clone)]
struct AuthorityEnvelopeField {
    key: String,
    span: AdrByteSpan,
    value_span: AdrByteSpan,
    first_line_value: String,
    first_line_end: usize,
}

#[derive(Debug, Clone)]
struct AuthorityEnvelopeFrontmatter {
    fields: Vec<AuthorityEnvelopeField>,
    span: AdrByteSpan,
}

/// Fail-closed parsing errors for ADR frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdrParseError {
    MissingLeadingFrontmatter,
    UnterminatedFrontmatter,
    DuplicateFrontmatterKey {
        key: String,
        span: AdrByteSpan,
    },
    InvalidFrontmatter {
        message: String,
        span: AdrByteSpan,
    },
    UnsupportedFrontmatterNesting {
        span: AdrByteSpan,
    },
    InvalidAdrReference {
        key: String,
        value: String,
        span: AdrByteSpan,
    },
    InvalidAdrId {
        value: String,
        span: AdrByteSpan,
    },
    AdrIdPathMismatch {
        id: String,
        source_path: String,
    },
    InvalidDate {
        value: String,
        span: AdrByteSpan,
    },
    InvalidSourcePath {
        source_path: String,
    },
    InvalidAdrHeading {
        message: String,
        span: AdrByteSpan,
    },
    MissingRequiredField {
        key: String,
    },
    InvalidTenant {
        tenant: String,
    },
}

impl fmt::Display for AdrParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLeadingFrontmatter => {
                f.write_str("ADR frontmatter must begin at byte zero")
            }
            Self::UnterminatedFrontmatter => f.write_str("ADR frontmatter is unterminated"),
            Self::DuplicateFrontmatterKey { key, .. } => {
                write!(f, "duplicate ADR frontmatter key {key:?}")
            }
            Self::InvalidFrontmatter { message, .. } => {
                write!(f, "invalid ADR frontmatter: {message}")
            }
            Self::UnsupportedFrontmatterNesting { .. } => {
                f.write_str("unsupported ADR frontmatter nesting")
            }
            Self::InvalidAdrReference { key, value, .. } => {
                write!(f, "invalid ADR reference {value:?} in {key}")
            }
            Self::InvalidAdrId { value, .. } => {
                write!(f, "invalid canonical ADR identifier {value:?}")
            }
            Self::AdrIdPathMismatch { id, source_path } => {
                write!(
                    f,
                    "ADR identifier {id} does not match source path {source_path:?}"
                )
            }
            Self::InvalidDate { value, .. } => {
                write!(f, "invalid ADR calendar date {value:?}")
            }
            Self::InvalidSourcePath { source_path } => {
                write!(
                    f,
                    "invalid repository-relative ADR source path {source_path:?}"
                )
            }
            Self::InvalidAdrHeading { message, .. } => {
                write!(f, "invalid ADR heading: {message}")
            }
            Self::MissingRequiredField { key } => {
                write!(f, "missing required ADR frontmatter field {key}")
            }
            Self::InvalidTenant { tenant } => write!(f, "invalid ADR tenant namespace {tenant:?}"),
        }
    }
}

impl std::error::Error for AdrParseError {}

/// Parse strict leading ADR frontmatter into a content-addressed decision IR.
///
/// # Errors
/// Returns an error for malformed, nested beyond the supported contract, duplicate,
/// or semantically invalid frontmatter. No partial decision is returned.
pub fn parse_adr_decision(input: &AdrParseInput) -> Result<AdrDecision, AdrParseError> {
    validate_source_path(input.source_path())?;
    if !input.source.starts_with("---\n") && !input.source.starts_with("---\r\n") {
        return Err(AdrParseError::MissingLeadingFrontmatter);
    }
    let mut parser = FrontmatterParser::new(input.source());
    parser.parse()?;
    let id_field = require_field(&parser.fields, "id")?;
    let id = AdrId(require_scalar_field(id_field)?.to_owned());
    if !is_adr_id(id.as_str()) {
        return Err(AdrParseError::InvalidAdrId {
            value: id.0,
            span: id_field.value_span,
        });
    }
    validate_id_matches_path(&id, input.source_path())?;
    let status = require_scalar(&parser.fields, "status")?.to_owned();
    let date_field = require_field(&parser.fields, "date")?;
    let date = require_scalar_field(date_field)?.to_owned();
    if !is_calendar_date(&date) {
        return Err(AdrParseError::InvalidDate {
            value: date,
            span: date_field.value_span,
        });
    }
    let owner = require_scalar(&parser.fields, "owner")?.to_owned();
    let title = heading_title(input.source(), parser.cursor, &id)?;
    let mut edges = BTreeMap::new();
    for key in [
        "depends_on",
        "supersedes",
        "superseded_by",
        "amends",
        "amended_by",
        "related",
    ] {
        let values = parser
            .fields
            .iter()
            .find(|field| field.key == key)
            .map(field_values)
            .unwrap_or_default();
        let mut references = Vec::with_capacity(values.len());
        for value in values {
            if !is_adr_id(&value) {
                return Err(AdrParseError::InvalidAdrReference {
                    key: key.to_owned(),
                    value,
                    span: parser.field_span(key),
                });
            }
            references.push(AdrReference {
                id: AdrId(value.clone()),
                raw_value: value,
                field_span: parser.field_span(key),
            });
        }
        edges.insert(key.to_owned(), references);
    }
    let canonical_bytes = input.source.as_bytes().to_vec();
    let digest = Sha256::digest(&canonical_bytes);
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(&digest);
    Ok(AdrDecision {
        source_path: input.source_path.clone(),
        id,
        title,
        status,
        date,
        owner,
        fields: parser.fields,
        edges,
        affected_surfaces: parser.affected_surfaces,
        deliverables: parser.deliverables,
        canonical_bytes,
        content_hash: AdrContentHash(bytes),
    })
}

/// Parse an immutable ADR authority envelope without constructing a decision IR.
///
/// The parser accepts arbitrary nested metadata as opaque bytes, while failing
/// closed for malformed fences, duplicate top-level keys, or malformed selected
/// fields. It deliberately does not validate or project any metadata other than
/// the identity, lifecycle spelling, and ordered supersession references.
///
/// # Errors
/// Returns an error when the source path, leading/closing frontmatter fence,
/// top-level field surface, or selected field shape is invalid.
pub fn parse_adr_authority_envelope(
    input: &AdrParseInput,
) -> Result<AdrAuthorityEnvelope, AdrParseError> {
    validate_source_path(input.source_path())?;
    let frontmatter = parse_authority_envelope_fields(input.source())?;
    let fields = &frontmatter.fields;
    let id_field = require_authority_field(fields, "id")?;
    let id = AdrId(authority_scalar(input.source(), id_field)?);
    if !is_adr_id(id.as_str()) {
        return Err(AdrParseError::InvalidAdrId {
            value: id.0,
            span: id_field.value_span,
        });
    }
    validate_id_matches_path(&id, input.source_path())?;
    let status = authority_scalar(input.source(), require_authority_field(fields, "status")?)?;
    let supersedes = fields
        .iter()
        .find(|field| field.key == "supersedes")
        .map(|field| authority_supersedes(input.source(), field))
        .transpose()?
        .unwrap_or_default();

    let opaque_fields = fields
        .iter()
        .filter(|field| !matches!(field.key.as_str(), "id" | "status" | "supersedes"))
        .map(|field| {
            let start = field.span.start as usize;
            let end = field.span.end as usize;
            AdrOpaqueFrontmatterField {
                key: field.key.clone(),
                span: field.span,
                raw_bytes: input.source().as_bytes()[start..end].to_vec(),
            }
        })
        .collect();
    let canonical_bytes = input.source().as_bytes().to_vec();
    let digest = Sha256::digest(&canonical_bytes);
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(&digest);
    Ok(AdrAuthorityEnvelope {
        source_path: input.source_path().to_owned(),
        frontmatter_span: frontmatter.span,
        id,
        status,
        supersedes,
        opaque_fields,
        canonical_bytes,
        content_hash: AdrContentHash(bytes),
    })
}

fn parse_authority_envelope_fields(
    source: &str,
) -> Result<AuthorityEnvelopeFrontmatter, AdrParseError> {
    if !source.starts_with("---\n") && !source.starts_with("---\r\n") {
        return Err(AdrParseError::MissingLeadingFrontmatter);
    }
    let mut cursor = next_line(source, 0).2;
    let mut fields: Vec<AuthorityEnvelopeField> = Vec::new();
    while cursor < source.len() {
        let (line, start, next) = next_line(source, cursor);
        let line = line.trim_end_matches('\r');
        if line == "---" {
            if let Some(previous) = fields.last_mut() {
                previous.span.end = start as u64;
            }
            return Ok(AuthorityEnvelopeFrontmatter {
                fields,
                span: span(0, next),
            });
        }
        if line.starts_with('\t') {
            return Err(AdrParseError::InvalidFrontmatter {
                message: "tabs are not supported".to_owned(),
                span: span(start, start + line.len()),
            });
        }
        if line.starts_with(' ') {
            let indentation = line
                .bytes()
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .count();
            if line.as_bytes()[..indentation].contains(&b'\t') {
                return Err(AdrParseError::InvalidFrontmatter {
                    message: "tabs are not supported".to_owned(),
                    span: span(start, start + line.len()),
                });
            }
            if indentation < 2 || !indentation.is_multiple_of(2) {
                return Err(AdrParseError::InvalidFrontmatter {
                    message: "indentation must be a multiple of two spaces".to_owned(),
                    span: span(start, start + line.len()),
                });
            }
            if fields.is_empty() {
                return Err(AdrParseError::InvalidFrontmatter {
                    message: "frontmatter must begin with a top-level key".to_owned(),
                    span: span(start, start + line.len()),
                });
            }
            cursor = next;
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            if fields.is_empty() {
                return Err(AdrParseError::InvalidFrontmatter {
                    message: "frontmatter must begin with a top-level key".to_owned(),
                    span: span(start, start + line.len()),
                });
            }
            cursor = next;
            continue;
        }
        let Some((key, raw_value)) = line.split_once(':') else {
            return Err(AdrParseError::InvalidFrontmatter {
                message: "expected key: value".to_owned(),
                span: span(start, start + line.len()),
            });
        };
        if !valid_key(key) {
            return Err(AdrParseError::InvalidFrontmatter {
                message: "invalid key".to_owned(),
                span: span(start, start + line.len()),
            });
        }
        if fields.iter().any(|field| field.key == key) {
            return Err(AdrParseError::DuplicateFrontmatterKey {
                key: key.to_owned(),
                span: span(start, start + line.len()),
            });
        }
        if let Some(previous) = fields.last_mut() {
            previous.span.end = start as u64;
        }
        let first_line_end = next;
        fields.push(AuthorityEnvelopeField {
            key: key.to_owned(),
            span: span(start, source.len()),
            value_span: span(start + key.len() + 1, start + line.len()),
            first_line_value: raw_value.to_owned(),
            first_line_end,
        });
        cursor = next;
    }
    Err(AdrParseError::UnterminatedFrontmatter)
}

fn require_authority_field<'a>(
    fields: &'a [AuthorityEnvelopeField],
    key: &str,
) -> Result<&'a AuthorityEnvelopeField, AdrParseError> {
    fields.iter().find(|field| field.key == key).ok_or_else(|| {
        AdrParseError::MissingRequiredField {
            key: key.to_owned(),
        }
    })
}

fn authority_scalar(source: &str, field: &AuthorityEnvelopeField) -> Result<String, AdrParseError> {
    if source[field.first_line_end..field.span.end as usize]
        .lines()
        .any(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
    {
        return Err(AdrParseError::InvalidFrontmatter {
            message: format!("{} must be a non-empty scalar", field.key),
            span: field.span,
        });
    }
    let value = parse_value(&field.first_line_value).map_err(|message| {
        AdrParseError::InvalidFrontmatter {
            message,
            span: field.value_span,
        }
    })?;
    match value {
        AdrFrontmatterValue::Scalar(value) if !value.is_empty() => Ok(value),
        _ => Err(AdrParseError::InvalidFrontmatter {
            message: format!("{} must be a non-empty scalar", field.key),
            span: field.value_span,
        }),
    }
}

fn authority_supersedes(
    source: &str,
    field: &AuthorityEnvelopeField,
) -> Result<Vec<AdrReference>, AdrParseError> {
    let nested = &source[field.first_line_end..field.span.end as usize];
    let value = parse_value(&field.first_line_value).map_err(|message| {
        AdrParseError::InvalidFrontmatter {
            message,
            span: field.value_span,
        }
    })?;
    let values = if nested.trim().is_empty() {
        field_value_list(value)
    } else {
        if !matches!(value, AdrFrontmatterValue::Empty) {
            return Err(AdrParseError::InvalidFrontmatter {
                message: "supersedes block list requires an empty parent value".to_owned(),
                span: field.value_span,
            });
        }
        let mut values = Vec::new();
        for line in nested.lines() {
            let line = line.trim_end_matches('\r');
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some(raw) = line.strip_prefix("  - ") else {
                return Err(AdrParseError::UnsupportedFrontmatterNesting { span: field.span });
            };
            values.push(parse_scalar(raw).map_err(|message| {
                AdrParseError::InvalidFrontmatter {
                    message,
                    span: field.span,
                }
            })?);
        }
        values
    };
    values
        .into_iter()
        .map(|value| {
            if !is_adr_id(&value) {
                return Err(AdrParseError::InvalidAdrReference {
                    key: "supersedes".to_owned(),
                    value,
                    span: field.span,
                });
            }
            Ok(AdrReference {
                id: AdrId(value.clone()),
                raw_value: value,
                field_span: field.span,
            })
        })
        .collect()
}

fn require_scalar<'a>(
    fields: &'a [AdrFrontmatterField],
    key: &str,
) -> Result<&'a str, AdrParseError> {
    require_scalar_field(require_field(fields, key)?)
}

fn require_field<'a>(
    fields: &'a [AdrFrontmatterField],
    key: &str,
) -> Result<&'a AdrFrontmatterField, AdrParseError> {
    let Some(field) = fields.iter().find(|field| field.key == key) else {
        return Err(AdrParseError::MissingRequiredField {
            key: key.to_owned(),
        });
    };
    Ok(field)
}

fn require_scalar_field(field: &AdrFrontmatterField) -> Result<&str, AdrParseError> {
    match &field.value {
        AdrFrontmatterValue::Scalar(value) if !value.is_empty() => Ok(value),
        _ => Err(AdrParseError::InvalidFrontmatter {
            message: format!("{} must be a non-empty scalar", field.key),
            span: field.value_span,
        }),
    }
}

fn field_values(field: &AdrFrontmatterField) -> Vec<String> {
    match &field.value {
        AdrFrontmatterValue::Scalar(value) => vec![value.clone()],
        AdrFrontmatterValue::List(values) => values.clone(),
        AdrFrontmatterValue::Null | AdrFrontmatterValue::Empty => Vec::new(),
    }
}

fn is_adr_id(value: &str) -> bool {
    value.len() == 8
        && value.starts_with("ADR-")
        && value.as_bytes()[4..].iter().all(u8::is_ascii_digit)
}

fn validate_id_matches_path(id: &AdrId, source_path: &str) -> Result<(), AdrParseError> {
    let file_name = source_path.rsplit('/').next().unwrap_or(source_path);
    let matches = file_name == format!("{}.md", id.as_str())
        || file_name
            .strip_prefix(id.as_str())
            .is_some_and(|suffix| suffix.starts_with('-') && suffix.ends_with(".md"));
    if matches {
        Ok(())
    } else {
        Err(AdrParseError::AdrIdPathMismatch {
            id: id.as_str().to_owned(),
            source_path: source_path.to_owned(),
        })
    }
}

fn validate_source_path(source_path: &str) -> Result<(), AdrParseError> {
    let Some(file_name) = source_path.strip_prefix("docs/decisions/") else {
        return Err(AdrParseError::InvalidSourcePath {
            source_path: source_path.to_owned(),
        });
    };
    if file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name == "."
        || file_name == ".."
    {
        return Err(AdrParseError::InvalidSourcePath {
            source_path: source_path.to_owned(),
        });
    }
    Ok(())
}

fn heading_title(source: &str, start: usize, id: &AdrId) -> Result<String, AdrParseError> {
    let mut cursor = start;
    while cursor < source.len() {
        let rest = &source[cursor..];
        let line_end = rest
            .find('\n')
            .map_or(source.len(), |offset| cursor + offset);
        let mut content_end = line_end;
        if content_end > cursor && source.as_bytes()[content_end - 1] == b'\r' {
            content_end -= 1;
        }
        let line = &source[cursor..content_end];
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("<!--") {
            let Some(value) = trimmed.strip_prefix("# ") else {
                return Err(AdrParseError::InvalidAdrHeading {
                    message: "first content line must be an H1".to_owned(),
                    span: span(cursor, content_end),
                });
            };
            let Some((heading_id, title)) = split_heading(value) else {
                return Err(AdrParseError::InvalidAdrHeading {
                    message: "H1 must use '<id>: <title>' or '<id> — <title>'".to_owned(),
                    span: span(cursor, content_end),
                });
            };
            if heading_id.trim() != id.as_str() {
                return Err(AdrParseError::InvalidAdrHeading {
                    message: format!(
                        "H1 id {:?} does not match {}",
                        heading_id.trim(),
                        id.as_str()
                    ),
                    span: span(cursor, content_end),
                });
            }
            let title = title.trim();
            if title.is_empty() {
                return Err(AdrParseError::InvalidAdrHeading {
                    message: "H1 title must be non-empty".to_owned(),
                    span: span(cursor, content_end),
                });
            }
            return Ok(title.to_owned());
        }
        cursor = if line_end < source.len() {
            line_end + 1
        } else {
            line_end
        };
    }
    Err(AdrParseError::InvalidAdrHeading {
        message: "ADR body must contain an H1".to_owned(),
        span: span(start, source.len()),
    })
}

fn split_heading(value: &str) -> Option<(&str, &str)> {
    match (value.find(':'), value.find(" — ")) {
        (Some(colon), Some(dash)) if dash < colon => {
            Some((&value[..dash], &value[dash + " — ".len()..]))
        }
        (Some(colon), _) => Some((&value[..colon], &value[colon + 1..])),
        (None, Some(dash)) => Some((&value[..dash], &value[dash + " — ".len()..])),
        (None, None) => None,
    }
}

fn is_calendar_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes[..4].iter().all(u8::is_ascii_digit)
        || !bytes[5..7].iter().all(u8::is_ascii_digit)
        || !bytes[8..].iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    let number = |digits: &[u8]| {
        digits
            .iter()
            .fold(0_u16, |value, digit| value * 10 + u16::from(*digit - b'0'))
    };
    let year = number(&bytes[..4]);
    let month = number(&bytes[5..7]);
    let day = number(&bytes[8..]);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&day)
}

struct FrontmatterParser<'a> {
    source: &'a str,
    cursor: usize,
    fields: Vec<AdrFrontmatterField>,
    affected_surfaces: Vec<AdrAffectedSurface>,
    deliverables: Vec<AdrDeliverable>,
}

impl<'a> FrontmatterParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            fields: Vec::new(),
            affected_surfaces: Vec::new(),
            deliverables: Vec::new(),
        }
    }

    fn parse(&mut self) -> Result<(), AdrParseError> {
        let (_, _, next) = self.line();
        self.cursor = next;
        let mut active: Option<String> = None;
        while self.cursor < self.source.len() {
            let (line, start, next) = self.line();
            self.cursor = next;
            if line == "---" {
                return Ok(());
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with(' ') || line.starts_with('\t') {
                self.parse_indented(line, start, active.as_deref())?;
                continue;
            }
            let Some((key, raw_value)) = line.split_once(':') else {
                return Err(self.invalid("expected key: value", start, line.len()));
            };
            if !valid_key(key) {
                return Err(self.invalid("invalid key", start, line.len()));
            }
            if self.fields.iter().any(|field| field.key == key) {
                return Err(AdrParseError::DuplicateFrontmatterKey {
                    key: key.to_owned(),
                    span: span(start, start + line.len()),
                });
            }
            let value_start = start + key.len() + 1;
            let value = parse_value(raw_value)
                .map_err(|message| self.invalid(&message, value_start, raw_value.len()))?;
            let supported_structured_value = match &value {
                AdrFrontmatterValue::Empty | AdrFrontmatterValue::Null => true,
                AdrFrontmatterValue::List(values) => values.is_empty(),
                AdrFrontmatterValue::Scalar(_) => false,
            };
            if matches!(key, "affected_surfaces" | "deliverables") && !supported_structured_value {
                return Err(self.invalid(
                    "structured field requires an empty value, null, [], or supported block form",
                    value_start,
                    raw_value.len(),
                ));
            }
            self.fields.push(AdrFrontmatterField {
                key: key.to_owned(),
                raw_value: raw_value.to_owned(),
                value_span: span(value_start, start + line.len()),
                value,
            });
            active = Some(key.to_owned());
        }
        Err(AdrParseError::UnterminatedFrontmatter)
    }

    fn parse_indented(
        &mut self,
        line: &str,
        start: usize,
        active: Option<&str>,
    ) -> Result<(), AdrParseError> {
        let Some(active) = active else {
            return Err(self.invalid("indented value without a key", start, line.len()));
        };
        if line.starts_with(" ") && !line.starts_with("  ") {
            return Err(self.invalid(
                "indentation must be a multiple of two spaces",
                start,
                line.len(),
            ));
        }
        if line.starts_with("\t") {
            return Err(self.invalid("tabs are not supported", start, line.len()));
        }
        if matches!(active, "affected_surfaces" | "deliverables")
            && self
                .fields
                .iter()
                .find(|field| field.key == active)
                .is_none_or(|field| !matches!(field.value, AdrFrontmatterValue::Empty))
        {
            return Err(self.invalid(
                "structured child cannot follow an inline, null, or [] parent value",
                start,
                line.len(),
            ));
        }
        self.extend_field_raw(active, start, line.len())?;
        if let Some(style) = self.block_scalar_style(active) {
            return self.parse_block_scalar(active, style, line, start);
        }
        match active {
            "affected_surfaces" => self.parse_surface(line, start),
            "deliverables" => self.parse_deliverable(line, start),
            _ => self.parse_list_item(active, line, start),
        }
    }

    fn extend_field_raw(
        &mut self,
        key: &str,
        line_start: usize,
        line_len: usize,
    ) -> Result<(), AdrParseError> {
        let Some(field_index) = self.fields.iter().position(|field| field.key == key) else {
            return Err(self.invalid("missing active frontmatter field", line_start, line_len));
        };
        let raw_start = self.fields[field_index].value_span.start as usize;
        let raw_end = line_start + line_len;
        self.fields[field_index].raw_value = self.source[raw_start..raw_end].to_owned();
        self.fields[field_index].value_span = span(raw_start, raw_end);
        Ok(())
    }

    fn block_scalar_style(&self, key: &str) -> Option<char> {
        self.fields
            .iter()
            .find(|field| field.key == key)
            .and_then(|field| field.raw_value.lines().next())
            .map(str::trim)
            .and_then(|value| match value {
                ">" => Some('>'),
                "|" => Some('|'),
                _ => None,
            })
    }

    fn parse_block_scalar(
        &mut self,
        key: &str,
        style: char,
        line: &str,
        start: usize,
    ) -> Result<(), AdrParseError> {
        let Some(content) = line.strip_prefix("  ") else {
            return Err(self.invalid(
                "block scalar content requires two-space indentation",
                start,
                line.len(),
            ));
        };
        let Some(field_index) = self.fields.iter().position(|field| field.key == key) else {
            return Err(self.invalid("missing block scalar field", start, line.len()));
        };
        let raw_start = self.fields[field_index].value_span.start as usize;
        let raw_end = start + line.len();
        let raw_value = self.source[raw_start..raw_end].to_owned();
        let field = &mut self.fields[field_index];
        field.raw_value = raw_value;
        field.value_span = span(raw_start, raw_end);
        match &mut field.value {
            AdrFrontmatterValue::Empty => {
                field.value = AdrFrontmatterValue::Scalar(content.to_owned());
            }
            AdrFrontmatterValue::Scalar(value) => {
                if style == '|' || content.is_empty() {
                    value.push('\n');
                } else if !value.is_empty() && !value.ends_with('\n') {
                    value.push(' ');
                }
                value.push_str(content);
            }
            AdrFrontmatterValue::List(_) | AdrFrontmatterValue::Null => {
                return Err(self.invalid("invalid block scalar state", start, line.len()));
            }
        }
        Ok(())
    }

    fn parse_list_item(
        &mut self,
        key: &str,
        line: &str,
        start: usize,
    ) -> Result<(), AdrParseError> {
        let Some(value) = line.strip_prefix("  - ") else {
            return Err(AdrParseError::UnsupportedFrontmatterNesting {
                span: span(start, start + line.len()),
            });
        };
        if value.contains(':') {
            return Err(AdrParseError::UnsupportedFrontmatterNesting {
                span: span(start, start + line.len()),
            });
        }
        let value = parse_scalar(value)
            .map_err(|message| self.invalid(&message, start + 4, value.len()))?;
        let Some(field) = self.fields.iter_mut().find(|field| field.key == key) else {
            return Err(self.invalid("missing list field", start, line.len()));
        };
        match &mut field.value {
            AdrFrontmatterValue::Empty => field.value = AdrFrontmatterValue::List(vec![value]),
            AdrFrontmatterValue::List(values) => values.push(value),
            _ => {
                return Err(self.invalid(
                    "block list requires an empty key value",
                    start,
                    line.len(),
                ));
            }
        }
        Ok(())
    }

    fn parse_surface(&mut self, line: &str, start: usize) -> Result<(), AdrParseError> {
        if let Some(value) = line.strip_prefix("    - ") {
            if value.contains(':') {
                return Err(AdrParseError::UnsupportedFrontmatterNesting {
                    span: span(start, start + line.len()),
                });
            }
            let parsed_value = parse_scalar(value)
                .map_err(|message| self.invalid(&message, start + 6, value.len()))?;
            let Some(surface) = self.affected_surfaces.last_mut() else {
                return Err(self.invalid("surface list item without category", start, line.len()));
            };
            surface.values.push(parsed_value);
            return Ok(());
        }
        if let Some((category, raw)) = line
            .strip_prefix("  ")
            .and_then(|value| value.split_once(':'))
        {
            if !valid_key(category) {
                return Err(self.invalid("invalid affected surface category", start, line.len()));
            }
            if is_block_scalar_marker(raw) {
                return Err(self.invalid(
                    "block scalar values are not supported for affected surface categories",
                    start + 2 + category.len() + 1,
                    raw.len(),
                ));
            }
            let values = field_value_list(parse_value(raw).map_err(|message| {
                self.invalid(&message, start + 2 + category.len() + 1, raw.len())
            })?);
            if self
                .affected_surfaces
                .iter()
                .any(|surface| surface.category == category)
            {
                return Err(AdrParseError::DuplicateFrontmatterKey {
                    key: category.to_owned(),
                    span: span(start, start + line.len()),
                });
            }
            self.affected_surfaces.push(AdrAffectedSurface {
                category: category.to_owned(),
                values,
            });
            return Ok(());
        }
        Err(AdrParseError::UnsupportedFrontmatterNesting {
            span: span(start, start + line.len()),
        })
    }

    fn parse_deliverable(&mut self, line: &str, start: usize) -> Result<(), AdrParseError> {
        if let Some(raw) = line.strip_prefix("  - id: ") {
            let id = parse_scalar(raw)
                .map_err(|message| self.invalid(&message, start + 8, raw.len()))?;
            self.deliverables.push(AdrDeliverable {
                id,
                description: None,
                exit_criteria: None,
                verified_by: None,
            });
            return Ok(());
        }
        let Some((key, raw)) = line
            .strip_prefix("    ")
            .and_then(|value| value.split_once(':'))
        else {
            return Err(AdrParseError::UnsupportedFrontmatterNesting {
                span: span(start, start + line.len()),
            });
        };
        let value = parse_scalar(raw)
            .map_err(|message| self.invalid(&message, start + 4 + key.len() + 1, raw.len()))?;
        let Some(deliverable) = self.deliverables.last_mut() else {
            return Err(self.invalid("deliverable property without item", start, line.len()));
        };
        let slot = match key {
            "description" => &mut deliverable.description,
            "exit_criteria" => &mut deliverable.exit_criteria,
            "verified_by" => &mut deliverable.verified_by,
            _ => {
                return Err(AdrParseError::UnsupportedFrontmatterNesting {
                    span: span(start, start + line.len()),
                });
            }
        };
        if slot.replace(value).is_some() {
            return Err(AdrParseError::DuplicateFrontmatterKey {
                key: key.to_owned(),
                span: span(start, start + line.len()),
            });
        }
        Ok(())
    }

    fn line(&self) -> (&'a str, usize, usize) {
        let rest = &self.source[self.cursor..];
        let line_end = rest
            .find('\n')
            .map_or(self.source.len(), |offset| self.cursor + offset);
        let mut content_end = line_end;
        if content_end > self.cursor && self.source.as_bytes()[content_end - 1] == b'\r' {
            content_end -= 1;
        }
        let next = if line_end < self.source.len() {
            line_end + 1
        } else {
            line_end
        };
        (&self.source[self.cursor..content_end], self.cursor, next)
    }

    fn invalid(&self, message: &str, start: usize, len: usize) -> AdrParseError {
        AdrParseError::InvalidFrontmatter {
            message: message.to_owned(),
            span: span(start, start + len),
        }
    }
    fn field_span(&self, key: &str) -> AdrByteSpan {
        self.fields
            .iter()
            .find(|field| field.key == key)
            .map_or(AdrByteSpan { start: 0, end: 0 }, |field| field.value_span)
    }
}

fn span(start: usize, end: usize) -> AdrByteSpan {
    AdrByteSpan {
        start: start as u64,
        end: end as u64,
    }
}
fn valid_key(key: &str) -> bool {
    let mut bytes = key.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
fn field_value_list(value: AdrFrontmatterValue) -> Vec<String> {
    match value {
        AdrFrontmatterValue::Scalar(value) => vec![value],
        AdrFrontmatterValue::List(values) => values,
        AdrFrontmatterValue::Null | AdrFrontmatterValue::Empty => Vec::new(),
    }
}

fn is_block_scalar_marker(raw: &str) -> bool {
    matches!(
        strip_comment(raw).trim_start().as_bytes().first(),
        Some(b'|' | b'>')
    )
}

fn parse_value(raw: &str) -> Result<AdrFrontmatterValue, String> {
    let value = strip_comment(raw).trim();
    if value.is_empty() {
        return Ok(AdrFrontmatterValue::Empty);
    }
    if matches!(value, ">" | "|") {
        return Ok(AdrFrontmatterValue::Empty);
    }
    if matches!(value, "null" | "~") {
        return Ok(AdrFrontmatterValue::Null);
    }
    if value.starts_with('[') {
        if !value.ends_with(']') {
            return Err("unterminated inline list".to_owned());
        }
        let inner = &value[1..value.len() - 1];
        if inner.trim().is_empty() {
            return Ok(AdrFrontmatterValue::List(Vec::new()));
        }
        let mut values = Vec::new();
        for item in split_list(inner)? {
            values.push(parse_scalar(item)?);
        }
        return Ok(AdrFrontmatterValue::List(values));
    }
    Ok(AdrFrontmatterValue::Scalar(parse_scalar(value)?))
}

fn parse_scalar(raw: &str) -> Result<String, String> {
    let value = strip_comment(raw).trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    let bytes = value.as_bytes();
    if matches!(bytes.first(), Some(b'\'' | b'\"')) {
        let quote = bytes[0];
        if bytes.len() < 2 || bytes.last().is_none_or(|last| *last != quote) {
            return Err("unterminated quoted scalar".to_owned());
        }
        let inner = &value[1..value.len() - 1];
        return Ok(if quote == b'\"' {
            unescape_double(inner)?
        } else {
            inner.replace("''", "'")
        });
    }
    if value.contains('[') || value.contains(']') || value.contains('{') || value.contains('}') {
        return Err("unsupported scalar syntax".to_owned());
    }
    Ok(value.to_owned())
}

fn strip_comment(value: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote == Some('"') {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '\"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if character == '#'
            && quote.is_none()
            && (index == 0
                || value[..index]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace))
        {
            return &value[..index];
        }
    }
    value
}

fn split_list(value: &str) -> Result<Vec<&str>, String> {
    let mut items = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote == Some('"') {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '\"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if character == ',' && quote.is_none() {
            items.push(&value[start..index]);
            start = index + 1;
        }
    }
    if quote.is_some() {
        return Err("unterminated quoted list item".to_owned());
    }
    items.push(&value[start..]);
    Ok(items)
}

fn unescape_double(value: &str) -> Result<String, String> {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        let Some(escaped) = chars.next() else {
            return Err("unterminated escape".to_owned());
        };
        output.push(match escaped {
            '\\' => '\\',
            '\"' => '\"',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            _ => return Err("unsupported escape".to_owned()),
        });
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{DocParseError, DocParseInput, parse_markdown_doc};

    #[test]
    fn zero_structure_documents_still_require_a_valid_provenance_path() {
        let valid = parse_markdown_doc(&DocParseInput::new(
            "tenant",
            "docs/decisions/ADR-0001.md",
            "",
        ))
        .expect("an empty document with canonical provenance remains valid");
        assert!(valid.nodes().is_empty());

        for path in [
            "",
            "/docs/decisions/ADR-0001.md",
            "docs/decisions/../private.md",
        ] {
            assert!(matches!(
                parse_markdown_doc(&DocParseInput::new("tenant", path, "")),
                Err(DocParseError::InvalidSourcePath(rejected)) if rejected == path
            ));
        }

        assert!(matches!(
            parse_markdown_doc(&DocParseInput::new(
                "tenant",
                "docs/decisions/../private.md",
                "plain prose without a heading or reference",
            )),
            Err(DocParseError::InvalidSourcePath(rejected))
                if rejected == "docs/decisions/../private.md"
        ));
    }

    #[test]
    fn malformed_frontmatter_precedes_invalid_path_validation() {
        assert_eq!(
            parse_markdown_doc(&DocParseInput::new(
                "tenant",
                "docs/decisions/../private.md",
                "---\nid: ADR-0001\n",
            )),
            Err(DocParseError::MalformedFrontmatter)
        );
    }
}
