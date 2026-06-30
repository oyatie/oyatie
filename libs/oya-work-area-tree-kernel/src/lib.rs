//! Stable `WorkAreaTree` contract for the owned AST substrate.
//!
//! ADR-0517 chooses a single owned, rowan-style, content-addressed AST
//! substrate read by every consumer. ADR-0520 makes `WorkAreaTree` one of the
//! W1 interfaces that must be locked before the parser implementation lands;
//! ADR-0521 sequences the owned parser itself into W2. This crate therefore
//! defines only the consumer seam and node-identity vocabulary. It does **not**
//! parse Rust, Markdown, or any other source language.
//!
//! The vocabulary is deliberately content-addressed at two levels:
//!
//! - [`WorkAreaHash`] names the whole work area: the future SCM change id,
//!   buck2/RBE cache key, and CD artifact hash.
//! - [`NodeContentHash`] names an AST node's canonical content.
//! - [`WorkAreaNodeId`] combines those hashes with a stable source locator so
//!   identical node content at different source locations remains addressable
//!   by consumers that need leases, affected sets, gates, or documentation
//!   tracking.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

/// Byte length for the W1 SHA-256 content-address vocabulary.
pub const SHA256_BYTES: usize = 32;
/// Hex-encoded length for the W1 SHA-256 content-address vocabulary.
pub const SHA256_HEX_LEN: usize = SHA256_BYTES * 2;

/// Content-addressed hash of a whole work area.
///
/// ADR-0517 binds this one hash to the future SCM change id, buck2/RBE cache
/// key, and CD artifact hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorkAreaHash {
    bytes: [u8; SHA256_BYTES], // data_class: INTERNAL_ONLY
}

impl WorkAreaHash {
    /// Hash algorithm locked into the W1 vocabulary.
    pub const ALGORITHM: &'static str = "sha256";

    #[must_use]
    pub const fn from_bytes(bytes: [u8; SHA256_BYTES]) -> Self {
        Self { bytes }
    }

    /// Parse a lowercase or uppercase 64-character SHA-256 hex string.
    ///
    /// # Errors
    /// Returns [`WorkAreaTreeError::InvalidSha256Hex`] for non-hex or wrong-length input.
    pub fn from_hex(hex: &str) -> Result<Self, WorkAreaTreeError> {
        parse_sha256_hex(hex).map(Self::from_bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; SHA256_BYTES] {
        &self.bytes
    }

    #[must_use]
    pub fn to_hex(&self) -> String {
        encode_sha256_hex(self.bytes)
    }
}

impl fmt::Display for WorkAreaHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

/// Content-addressed hash of one AST node's canonical content.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NodeContentHash {
    bytes: [u8; SHA256_BYTES], // data_class: INTERNAL_ONLY
}

impl NodeContentHash {
    /// Hash algorithm locked into the W1 vocabulary.
    pub const ALGORITHM: &'static str = "sha256";

    #[must_use]
    pub const fn from_bytes(bytes: [u8; SHA256_BYTES]) -> Self {
        Self { bytes }
    }

    /// Parse a lowercase or uppercase 64-character SHA-256 hex string.
    ///
    /// # Errors
    /// Returns [`WorkAreaTreeError::InvalidSha256Hex`] for non-hex or wrong-length input.
    pub fn from_hex(hex: &str) -> Result<Self, WorkAreaTreeError> {
        parse_sha256_hex(hex).map(Self::from_bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; SHA256_BYTES] {
        &self.bytes
    }

    #[must_use]
    pub fn to_hex(&self) -> String {
        encode_sha256_hex(self.bytes)
    }
}

impl fmt::Display for NodeContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

/// Half-open byte range `[start_byte, end_byte)` in a source artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SourceSpan {
    start_byte: u64, // data_class: INTERNAL_ONLY
    end_byte: u64,   // data_class: INTERNAL_ONLY
}

impl SourceSpan {
    /// Construct a non-empty half-open byte range.
    ///
    /// # Errors
    /// Returns [`WorkAreaTreeError::InvalidSpan`] when `start_byte >= end_byte`.
    pub const fn new(start_byte: u64, end_byte: u64) -> Result<Self, WorkAreaTreeError> {
        if start_byte >= end_byte {
            Err(WorkAreaTreeError::InvalidSpan {
                start_byte,
                end_byte,
            })
        } else {
            Ok(Self {
                start_byte,
                end_byte,
            })
        }
    }

    #[must_use]
    pub const fn start_byte(&self) -> u64 {
        self.start_byte
    }

    #[must_use]
    pub const fn end_byte(&self) -> u64 {
        self.end_byte
    }
}

/// Stable source location for a node inside a work area.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NodeLocator {
    artifact_path: String, // data_class: INTERNAL_ONLY
    span: SourceSpan,      // data_class: INTERNAL_ONLY
}

impl NodeLocator {
    /// Construct a repo-relative source locator.
    ///
    /// # Errors
    /// Returns [`WorkAreaTreeError::InvalidArtifactPath`] for empty, absolute,
    /// backslash-separated, or control-character paths.
    pub fn new(
        artifact_path: impl Into<String>,
        span: SourceSpan,
    ) -> Result<Self, WorkAreaTreeError> {
        let artifact_path = artifact_path.into();
        if artifact_path.is_empty()
            || artifact_path.starts_with('/')
            || artifact_path.contains('\\')
            || artifact_path.chars().any(char::is_control)
        {
            return Err(WorkAreaTreeError::InvalidArtifactPath(artifact_path));
        }
        Ok(Self {
            artifact_path,
            span,
        })
    }

    #[must_use]
    pub fn artifact_path(&self) -> &str {
        &self.artifact_path
    }

    #[must_use]
    pub const fn span(&self) -> &SourceSpan {
        &self.span
    }
}

/// Consumer-facing node kind vocabulary.
///
/// The W2 parser may add richer language-specific details behind the same
/// trait; consumers can still reason about these stable categories.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NodeKind {
    Root,
    File,
    Syntax,
    Token,
    Trivia,
    Unknown(String),
}

/// Content-addressed identity of a node inside one work area.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorkAreaNodeId {
    work_area_hash: WorkAreaHash, // data_class: INTERNAL_ONLY
    node_hash: NodeContentHash,   // data_class: INTERNAL_ONLY
    locator: NodeLocator,         // data_class: INTERNAL_ONLY
}

impl WorkAreaNodeId {
    #[must_use]
    pub const fn new(
        work_area_hash: WorkAreaHash,
        node_hash: NodeContentHash,
        locator: NodeLocator,
    ) -> Self {
        Self {
            work_area_hash,
            node_hash,
            locator,
        }
    }

    #[must_use]
    pub const fn work_area_hash(&self) -> &WorkAreaHash {
        &self.work_area_hash
    }

    #[must_use]
    pub const fn node_hash(&self) -> &NodeContentHash {
        &self.node_hash
    }

    #[must_use]
    pub const fn locator(&self) -> &NodeLocator {
        &self.locator
    }
}

/// Minimal node record returned through [`WorkAreaTree`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkAreaNode {
    id: WorkAreaNodeId, // data_class: INTERNAL_ONLY
    kind: NodeKind,     // data_class: INTERNAL_ONLY
}

impl WorkAreaNode {
    #[must_use]
    pub const fn new(id: WorkAreaNodeId, kind: NodeKind) -> Self {
        Self { id, kind }
    }

    #[must_use]
    pub const fn id(&self) -> &WorkAreaNodeId {
        &self.id
    }

    #[must_use]
    pub fn kind(&self) -> NodeKind {
        self.kind.clone()
    }
}

/// Errors surfaced by the W1 WorkAreaTree contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkAreaTreeError {
    InvalidSha256Hex { value: String },
    InvalidSpan { start_byte: u64, end_byte: u64 },
    InvalidArtifactPath(String),
    NodeNotFound,
    Adapter(String),
}

impl fmt::Display for WorkAreaTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSha256Hex { value } => write!(
                f,
                "invalid sha256 content address {value:?}; expected {SHA256_HEX_LEN} hex chars"
            ),
            Self::InvalidSpan {
                start_byte,
                end_byte,
            } => write!(
                f,
                "invalid source span: start_byte {start_byte} must be less than end_byte {end_byte}"
            ),
            Self::InvalidArtifactPath(path) => {
                write!(f, "invalid repo-relative artifact path {path:?}")
            }
            Self::NodeNotFound => write!(f, "work-area node not found"),
            Self::Adapter(detail) => write!(f, "work-area tree adapter failure: {detail}"),
        }
    }
}

impl std::error::Error for WorkAreaTreeError {}

/// Stable consumer seam for owned AST implementations.
///
/// W1 locks this trait. W2 parsers and future substrate adapters implement it;
/// consumers should depend on this trait rather than parser internals.
pub trait WorkAreaTree: Send + Sync {
    #[must_use]
    fn work_area_hash(&self) -> WorkAreaHash;

    #[must_use]
    fn root_id(&self) -> WorkAreaNodeId;

    /// Return a node by stable id.
    ///
    /// # Errors
    /// Returns [`WorkAreaTreeError::NodeNotFound`] when the id is outside this
    /// tree, or [`WorkAreaTreeError::Adapter`] for implementation-specific
    /// storage failures.
    fn node(&self, id: &WorkAreaNodeId) -> Result<WorkAreaNode, WorkAreaTreeError>;

    /// Return child node ids in canonical source order.
    ///
    /// # Errors
    /// Returns [`WorkAreaTreeError::NodeNotFound`] when the parent id is outside
    /// this tree, or [`WorkAreaTreeError::Adapter`] for implementation-specific
    /// storage failures.
    fn child_ids(&self, id: &WorkAreaNodeId) -> Result<Vec<WorkAreaNodeId>, WorkAreaTreeError>;
}

fn encode_sha256_hex(bytes: [u8; SHA256_BYTES]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(SHA256_HEX_LEN);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn parse_sha256_hex(hex: &str) -> Result<[u8; SHA256_BYTES], WorkAreaTreeError> {
    if hex.len() != SHA256_HEX_LEN {
        return Err(WorkAreaTreeError::InvalidSha256Hex {
            value: hex.to_owned(),
        });
    }

    let mut bytes = [0_u8; SHA256_BYTES];
    let raw = hex.as_bytes();
    let mut index = 0;
    while index < SHA256_BYTES {
        let hi = hex_value(raw[index * 2]);
        let lo = hex_value(raw[index * 2 + 1]);
        match (hi, lo) {
            (Some(hi), Some(lo)) => bytes[index] = (hi << 4) | lo,
            _ => {
                return Err(WorkAreaTreeError::InvalidSha256Hex {
                    value: hex.to_owned(),
                });
            }
        }
        index += 1;
    }
    Ok(bytes)
}

const fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_hex_round_trips() {
        let hash = WorkAreaHash::from_bytes([0xab; SHA256_BYTES]);
        let hex = hash.to_hex();
        assert_eq!(hex.len(), SHA256_HEX_LEN);
        assert_eq!(WorkAreaHash::from_hex(&hex).unwrap(), hash);
        assert_eq!(
            NodeContentHash::from_hex(&hex).unwrap().as_bytes(),
            hash.as_bytes()
        );
    }

    #[test]
    fn invalid_hex_rejected() {
        assert!(matches!(
            WorkAreaHash::from_hex("not-a-sha"),
            Err(WorkAreaTreeError::InvalidSha256Hex { .. })
        ));
        let mut invalid = "00".repeat(SHA256_BYTES);
        invalid.replace_range(0..1, "x");
        assert!(matches!(
            NodeContentHash::from_hex(&invalid),
            Err(WorkAreaTreeError::InvalidSha256Hex { .. })
        ));
    }

    #[test]
    fn locator_requires_repo_relative_path() {
        let span = SourceSpan::new(1, 2).unwrap();
        assert!(NodeLocator::new("src/lib.rs", span).is_ok());
        assert!(matches!(
            NodeLocator::new("/src/lib.rs", span),
            Err(WorkAreaTreeError::InvalidArtifactPath(_))
        ));
        assert!(matches!(
            NodeLocator::new("src\\lib.rs", span),
            Err(WorkAreaTreeError::InvalidArtifactPath(_))
        ));
    }
}
