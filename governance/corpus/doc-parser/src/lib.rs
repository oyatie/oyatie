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

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod adr;

use std::fmt;

use sha2::{Digest, Sha256};
use work_area_tree_kernel::{
    NodeContentHash, NodeLocator, SourceSpan, WorkAreaHash, WorkAreaNodeId, WorkAreaTreeError,
};

pub use adr::{
    AdrAffectedSurface, AdrByteSpan, AdrContentHash, AdrDecision, AdrDecision as AdrDecisionIr,
    AdrDeliverable, AdrFrontmatterField, AdrFrontmatterValue, AdrId, AdrParseError, AdrParseInput,
    AdrReference, AdrTenant, AdrTenantIdentity, CanonicalAdrId, TenantAdrDecision,
    parse_adr_decision,
};

/// Parser version included in every node-id preimage.
pub const DOC_PARSER_VERSION: &str = "corpus-doc-parser-v1";

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
