//! Minimal safe Rust parser skeleton behind [`WorkAreaTree`].
//!
//! ADR-0517 chooses one owned, rowan-style, content-addressed AST substrate and
//! ADR-0521 sequences the concrete parser into W2 behind the W1
//! [`WorkAreaTree`] seam. This crate is the first parser skeleton slice: it is
//! safe Rust, hand-rolled, and deliberately narrow. It recognizes the one
//! committed fixture shape (`pub fn answer() -> u32 { 42 }`), produces typed
//! nodes with byte spans, and exposes them through [`WorkAreaTree`].
//!
//! Non-claims: no full Rust grammar, no Markdown parser, no rewrites, no query
//! engine, and no affected-set/SCM implementation.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

use sha2::{Digest, Sha256};
use work_area_tree_kernel::{
    NodeContentHash, NodeKind, NodeLocator, SourceSpan, WorkAreaHash, WorkAreaNode, WorkAreaNodeId,
    WorkAreaTree, WorkAreaTreeError,
};

/// A typed Rust syntax node kind emitted by the skeleton parser.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RustSyntaxKind {
    SourceFile,
    Function { name: String },
    VisibilityPub,
    FnKeyword,
    Identifier { name: String },
    Parameters,
    Block,
}

impl RustSyntaxKind {
    fn tag(&self) -> &'static str {
        match self {
            RustSyntaxKind::SourceFile => "source_file",
            RustSyntaxKind::Function { .. } => "function",
            RustSyntaxKind::VisibilityPub => "visibility_pub",
            RustSyntaxKind::FnKeyword => "fn_keyword",
            RustSyntaxKind::Identifier { .. } => "identifier",
            RustSyntaxKind::Parameters => "parameters",
            RustSyntaxKind::Block => "block",
        }
    }

    fn work_area_kind(&self) -> NodeKind {
        match self {
            RustSyntaxKind::SourceFile => NodeKind::Root,
            RustSyntaxKind::Function { .. }
            | RustSyntaxKind::Parameters
            | RustSyntaxKind::Block => NodeKind::Syntax,
            RustSyntaxKind::VisibilityPub
            | RustSyntaxKind::FnKeyword
            | RustSyntaxKind::Identifier { .. } => NodeKind::Token,
        }
    }
}

/// A typed parser node plus its stable WorkArea identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustTypedNode {
    id: WorkAreaNodeId,      // data_class: INTERNAL_ONLY
    kind: RustSyntaxKind,    // data_class: INTERNAL_ONLY
    source_text: String,     // data_class: INTERNAL_ONLY
    source_span: SourceSpan, // data_class: INTERNAL_ONLY
}

impl RustTypedNode {
    /// Stable WorkArea node id for this typed node.
    #[must_use]
    pub const fn id(&self) -> &WorkAreaNodeId {
        &self.id
    }

    /// Parser-specific node kind.
    #[must_use]
    pub const fn kind(&self) -> &RustSyntaxKind {
        &self.kind
    }

    /// Exact source text covered by this node.
    #[must_use]
    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    /// Half-open byte span in the parsed artifact.
    #[must_use]
    pub const fn source_span(&self) -> &SourceSpan {
        &self.source_span
    }

    fn as_work_area_node(&self) -> WorkAreaNode {
        WorkAreaNode::new(self.id.clone(), self.kind.work_area_kind())
    }
}

/// A parsed Rust file exposed through the W1 [`WorkAreaTree`] seam.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedRustWorkAreaTree {
    work_area_hash: WorkAreaHash, // data_class: INTERNAL_ONLY
    root_id: WorkAreaNodeId,      // data_class: INTERNAL_ONLY
    nodes: BTreeMap<WorkAreaNodeId, RustTypedNode>, // data_class: INTERNAL_ONLY
    children: BTreeMap<WorkAreaNodeId, Vec<WorkAreaNodeId>>, // data_class: INTERNAL_ONLY
}

impl ParsedRustWorkAreaTree {
    /// Return a parser-specific typed node by stable id.
    #[must_use]
    pub fn typed_node(&self, id: &WorkAreaNodeId) -> Option<&RustTypedNode> {
        self.nodes.get(id)
    }

    /// Return every typed node in deterministic id order.
    pub fn typed_nodes(&self) -> impl Iterator<Item = &RustTypedNode> {
        self.nodes.values()
    }
}

impl WorkAreaTree for ParsedRustWorkAreaTree {
    fn work_area_hash(&self) -> WorkAreaHash {
        self.work_area_hash
    }

    fn root_id(&self) -> WorkAreaNodeId {
        self.root_id.clone()
    }

    fn node(&self, id: &WorkAreaNodeId) -> Result<WorkAreaNode, WorkAreaTreeError> {
        self.nodes
            .get(id)
            .map(RustTypedNode::as_work_area_node)
            .ok_or(WorkAreaTreeError::NodeNotFound)
    }

    fn child_ids(&self, id: &WorkAreaNodeId) -> Result<Vec<WorkAreaNodeId>, WorkAreaTreeError> {
        if !self.nodes.contains_key(id) {
            return Err(WorkAreaTreeError::NodeNotFound);
        }
        Ok(self.children.get(id).cloned().unwrap_or_default())
    }
}

/// Parser errors for the minimal Rust skeleton.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RustParseError {
    EmptySource,
    MissingFunction,
    MissingIdentifier,
    MissingParameters,
    MissingBlock,
    UnclosedDelimiter { delimiter: char, offset: usize },
    InvalidWorkAreaNode(WorkAreaTreeError),
}

impl fmt::Display for RustParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RustParseError::EmptySource => {
                write!(f, "rust parser skeleton requires non-empty source")
            }
            RustParseError::MissingFunction => {
                write!(f, "rust parser skeleton did not find a `fn` item")
            }
            RustParseError::MissingIdentifier => {
                write!(f, "rust parser skeleton expected a function identifier")
            }
            RustParseError::MissingParameters => {
                write!(
                    f,
                    "rust parser skeleton expected a parenthesized parameter list"
                )
            }
            RustParseError::MissingBlock => {
                write!(f, "rust parser skeleton expected a function block")
            }
            RustParseError::UnclosedDelimiter { delimiter, offset } => write!(
                f,
                "rust parser skeleton found unclosed delimiter {delimiter:?} starting at byte {offset}"
            ),
            RustParseError::InvalidWorkAreaNode(error) => {
                write!(
                    f,
                    "rust parser skeleton produced invalid WorkAreaTree node: {error}"
                )
            }
        }
    }
}

impl std::error::Error for RustParseError {}

/// Parse one Rust source artifact into a minimal typed tree behind [`WorkAreaTree`].
///
/// The skeleton recognizes the first top-level `pub fn` / `fn` item and emits:
/// source file → function → visibility/`fn`/identifier/parameters/block.
///
/// # Errors
/// Returns [`RustParseError`] when the input is empty, lacks the supported
/// function shape, or would produce an invalid [`WorkAreaTree`] node locator.
pub fn parse_rust_source(
    artifact_path: impl Into<String>,
    source: &str,
) -> Result<ParsedRustWorkAreaTree, RustParseError> {
    if source.is_empty() {
        return Err(RustParseError::EmptySource);
    }

    let artifact_path = artifact_path.into();
    let work_area_hash = WorkAreaHash::from_bytes(sha256_parts(&[
        b"work-area-rust-parser:v1",
        artifact_path.as_bytes(),
        source.as_bytes(),
    ]));
    let slices = parse_first_function(source)?;

    let root = build_node(
        work_area_hash,
        &artifact_path,
        RustSyntaxKind::SourceFile,
        source,
        SpanBytes {
            start: 0,
            end: source.len(),
        },
    )?;
    let function = build_node(
        work_area_hash,
        &artifact_path,
        RustSyntaxKind::Function {
            name: slices.name.clone(),
        },
        source,
        slices.function,
    )?;

    let mut function_children = Vec::new();
    let mut nodes = BTreeMap::new();
    let root_children = vec![function.id.clone()];
    nodes.insert(root.id.clone(), root.clone());
    nodes.insert(function.id.clone(), function.clone());

    if let Some(visibility) = slices.visibility {
        let node = build_node(
            work_area_hash,
            &artifact_path,
            RustSyntaxKind::VisibilityPub,
            source,
            visibility,
        )?;
        function_children.push(node.id.clone());
        nodes.insert(node.id.clone(), node);
    }

    for (kind, span) in [
        (RustSyntaxKind::FnKeyword, slices.fn_keyword),
        (
            RustSyntaxKind::Identifier { name: slices.name },
            slices.identifier,
        ),
        (RustSyntaxKind::Parameters, slices.parameters),
        (RustSyntaxKind::Block, slices.block),
    ] {
        let node = build_node(work_area_hash, &artifact_path, kind, source, span)?;
        function_children.push(node.id.clone());
        nodes.insert(node.id.clone(), node);
    }

    let children = BTreeMap::from([
        (root.id.clone(), root_children),
        (function.id.clone(), function_children),
    ]);

    Ok(ParsedRustWorkAreaTree {
        work_area_hash,
        root_id: root.id,
        nodes,
        children,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SpanBytes {
    start: usize,
    end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FunctionSlices {
    function: SpanBytes,
    visibility: Option<SpanBytes>,
    fn_keyword: SpanBytes,
    identifier: SpanBytes,
    parameters: SpanBytes,
    block: SpanBytes,
    name: String,
}

fn parse_first_function(source: &str) -> Result<FunctionSlices, RustParseError> {
    let mut cursor = skip_whitespace(source, 0);
    let visibility = if starts_with_keyword(source, cursor, "pub") {
        let span = SpanBytes {
            start: cursor,
            end: cursor + "pub".len(),
        };
        cursor = skip_whitespace(source, span.end);
        Some(span)
    } else {
        None
    };

    if !starts_with_keyword(source, cursor, "fn") {
        return Err(RustParseError::MissingFunction);
    }
    let fn_keyword = SpanBytes {
        start: cursor,
        end: cursor + "fn".len(),
    };
    cursor = skip_whitespace(source, fn_keyword.end);

    let identifier_end =
        take_identifier(source, cursor).ok_or(RustParseError::MissingIdentifier)?;
    let identifier = SpanBytes {
        start: cursor,
        end: identifier_end,
    };
    let name = source[identifier.start..identifier.end].to_owned();
    cursor = skip_whitespace(source, identifier.end);

    if source.as_bytes().get(cursor).copied() != Some(b'(') {
        return Err(RustParseError::MissingParameters);
    }
    let parameters_end = find_matching_delimiter(source, cursor, '(', ')')? + ')'.len_utf8();
    let parameters = SpanBytes {
        start: cursor,
        end: parameters_end,
    };

    let block_start =
        find_byte(source, parameters.end, b'{').ok_or(RustParseError::MissingBlock)?;
    let block_end = find_matching_delimiter(source, block_start, '{', '}')? + '}'.len_utf8();
    let block = SpanBytes {
        start: block_start,
        end: block_end,
    };

    Ok(FunctionSlices {
        function: SpanBytes {
            start: visibility.map_or(fn_keyword.start, |span| span.start),
            end: block.end,
        },
        visibility,
        fn_keyword,
        identifier,
        parameters,
        block,
        name,
    })
}

fn build_node(
    work_area_hash: WorkAreaHash,
    artifact_path: &str,
    kind: RustSyntaxKind,
    source: &str,
    span: SpanBytes,
) -> Result<RustTypedNode, RustParseError> {
    let source_span = SourceSpan::new(span.start as u64, span.end as u64)
        .map_err(RustParseError::InvalidWorkAreaNode)?;
    let locator = NodeLocator::new(artifact_path, source_span)
        .map_err(RustParseError::InvalidWorkAreaNode)?;
    let source_text = source[span.start..span.end].to_owned();
    let start = span.start.to_string();
    let end = span.end.to_string();
    let node_hash = NodeContentHash::from_bytes(sha256_parts(&[
        b"work-area-rust-parser-node:v1",
        kind.tag().as_bytes(),
        artifact_path.as_bytes(),
        start.as_bytes(),
        end.as_bytes(),
        source_text.as_bytes(),
    ]));
    let id = WorkAreaNodeId::new(work_area_hash, node_hash, locator);

    Ok(RustTypedNode {
        id,
        kind,
        source_text,
        source_span,
    })
}

fn skip_whitespace(source: &str, mut offset: usize) -> usize {
    while let Some(byte) = source.as_bytes().get(offset).copied() {
        if !byte.is_ascii_whitespace() {
            break;
        }
        offset += 1;
    }
    offset
}

fn starts_with_keyword(source: &str, offset: usize, keyword: &str) -> bool {
    let Some(rest) = source.get(offset..) else {
        return false;
    };
    if !rest.starts_with(keyword) {
        return false;
    }
    let before_ok = offset == 0
        || source
            .as_bytes()
            .get(offset - 1)
            .is_none_or(|byte| !is_identifier_continue(*byte));
    let after = offset + keyword.len();
    let after_ok = source
        .as_bytes()
        .get(after)
        .is_none_or(|byte| !is_identifier_continue(*byte));
    before_ok && after_ok
}

fn take_identifier(source: &str, offset: usize) -> Option<usize> {
    let first = source.as_bytes().get(offset).copied()?;
    if !is_identifier_start(first) {
        return None;
    }
    let mut end = offset + 1;
    while let Some(byte) = source.as_bytes().get(end).copied() {
        if !is_identifier_continue(byte) {
            break;
        }
        end += 1;
    }
    Some(end)
}

const fn is_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

const fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

fn find_byte(source: &str, start: usize, needle: u8) -> Option<usize> {
    source
        .as_bytes()
        .get(start..)?
        .iter()
        .position(|byte| *byte == needle)
        .map(|relative| start + relative)
}

fn find_matching_delimiter(
    source: &str,
    open_offset: usize,
    open: char,
    close: char,
) -> Result<usize, RustParseError> {
    let mut depth = 0_usize;
    for (relative, ch) in source[open_offset..].char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Ok(open_offset + relative);
            }
        }
    }
    Err(RustParseError::UnclosedDelimiter {
        delimiter: open,
        offset: open_offset,
    })
}

fn sha256_parts(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
        hasher.update([0]);
    }
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(&digest);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_PATH: &str =
        "governance/corpus/work-area-rust-parser/fixtures/minimal_function.rs.txt";
    const FIXTURE_SOURCE: &str = include_str!("../fixtures/minimal_function.rs.txt");

    #[test]
    fn fixture_produces_typed_nodes_behind_work_area_tree() {
        let tree = parse_rust_source(FIXTURE_PATH, FIXTURE_SOURCE).expect("fixture parses");
        let root_id = tree.root_id();
        let root = tree.typed_node(&root_id).expect("typed root");
        assert_eq!(root.kind(), &RustSyntaxKind::SourceFile);
        assert_eq!(
            tree.node(&root_id).expect("root node").kind(),
            NodeKind::Root
        );

        let root_children = tree.child_ids(&root_id).expect("root children");
        assert_eq!(root_children.len(), 1);
        let function_id = &root_children[0];
        let function = tree.typed_node(function_id).expect("typed function");
        assert_eq!(
            function.kind(),
            &RustSyntaxKind::Function {
                name: "answer".to_owned()
            }
        );
        assert_eq!(function.source_text(), "pub fn answer() -> u32 { 42 }");
        assert_eq!(
            tree.node(function_id).expect("work-area function").kind(),
            NodeKind::Syntax
        );

        let function_children = tree.child_ids(function_id).expect("function children");
        let child_kinds: Vec<RustSyntaxKind> = function_children
            .iter()
            .map(|id| tree.typed_node(id).expect("typed child").kind().clone())
            .collect();
        assert_eq!(
            child_kinds,
            vec![
                RustSyntaxKind::VisibilityPub,
                RustSyntaxKind::FnKeyword,
                RustSyntaxKind::Identifier {
                    name: "answer".to_owned()
                },
                RustSyntaxKind::Parameters,
                RustSyntaxKind::Block,
            ]
        );
    }

    #[test]
    fn typed_tree_reports_missing_ids_through_work_area_tree() {
        let tree = parse_rust_source(FIXTURE_PATH, FIXTURE_SOURCE).expect("fixture parses");
        let missing_span = SourceSpan::new(1, 2).expect("span");
        let missing_locator = NodeLocator::new(FIXTURE_PATH, missing_span).expect("locator");
        let missing = WorkAreaNodeId::new(
            tree.work_area_hash(),
            NodeContentHash::from_bytes([99; 32]),
            missing_locator,
        );

        assert_eq!(tree.node(&missing), Err(WorkAreaTreeError::NodeNotFound));
        assert_eq!(
            tree.child_ids(&missing),
            Err(WorkAreaTreeError::NodeNotFound)
        );
    }

    #[test]
    fn unsupported_shapes_fail_honestly() {
        assert_eq!(
            parse_rust_source("fixtures/empty.rs", ""),
            Err(RustParseError::EmptySource)
        );
        assert_eq!(
            parse_rust_source("fixtures/not_fn.rs", "pub struct NotYet;"),
            Err(RustParseError::MissingFunction)
        );
    }
}
