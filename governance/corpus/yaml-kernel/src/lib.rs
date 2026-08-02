//! # corpus-yaml-kernel — the YAML slice of the corpus liveness graph
//!
//! Turns one YAML file into content-addressed graph **nodes** and typed **edges**. This is the
//! first non-Rust artifact class in the corpus graph, and it is the class that carries the coverage
//! gap: 5,864 tracked YAML files against 2,472 Rust files, and Rust is already ~97% build-graph
//! covered while most YAML is not.
//!
//! ## PURE
//! Zero I/O. Every entry point takes the file path and its bytes as DATA. The adapter that reads
//! files and writes shards is the `corpus-yaml-facts` binary in `corpus-extract`, which is what a
//! buck2 action actually invokes. `#![forbid(unsafe_code)]`, no clock, no rand, no net, no ambient
//! filesystem access — so a face is a pure function of its declared inputs, which is exactly the
//! property buck2 action caching relies on.
//!
//! ## Identity is NOMINAL; the digest is SHALLOW
//! Neither Kythe (`VName` = five plain strings) nor Glean (opaque per-DB fact ids) content-addresses
//! node *identity*, and ADR-0541 D1 already ratified the same split. So [`NodeId`] is a readable
//! tuple that survives reformatting and unrelated edits, and [`Node::digest`] is a blake3 hash over
//! that node's OWN pre-image with **zero child digests**.
//!
//! That absence is load-bearing. A Merkle-style roll-up would make every edit churn the root, which
//! is the parent-churn pathology neither hyperscaler accepted. There is deliberately no aggregate
//! "file rolls up its entries" digest here: a [`NodeKind::File`] digest is blake3 of the file bytes,
//! an [`NodeKind::Entry`] digest is blake3 of that entry's own path + scalar. Editing one key churns
//! one Entry (and the File, because file bytes genuinely changed) and nothing else.
//!
//! ## Two edge kinds, because only two are load-bearing
//! Kythe ships 11+ edge kinds to serve IDE cross-reference and type hierarchy; we build neither.
//! [`EdgeKind::Contains`] answers "what does this target contain", [`EdgeKind::Refs`] answers "what
//! references this file". Reachability is the BFS closure over their union — a *query*, not a third
//! edge kind, because a materialized reachability edge is a view that goes stale (which is precisely
//! what the hand-maintained `specs/reachability-registry.json` does today).
//!
//! ## Fail-loud, not fail-empty
//! An extractor that silently sees nothing emits an empty face, which is indistinguishable from a
//! clean one. [`YamlFacts::opaque`] carries `corpus_core::OpaqueReason` per unparseable document so
//! an empty face and a broken extractor are distinguishable. Callers MUST treat a non-empty
//! `opaque` as fail-open-with-report, never as clean.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use corpus_core::{ContentHash, OpaqueReason};
use saphyr::{LoadableYamlNode, Scalar, Yaml};
use serde::{Deserialize, Serialize};

/// The schema version stamped into every [`GraphFace`].
///
/// Distinguishes shape from content when two producers are in flight during an ADR-0541 D2
/// verified-equivalence migration. Bump on any change to the serialized field set.
pub const SCHEMA_VERSION: u32 = 1;

/// The coarse kind of a graph node.
///
/// Deliberately three variants. Per-domain detail belongs in a per-domain fact type joined on
/// [`NodeId`], never contorted into this enum — a YAML key has no `visibility` and a Rust `fn` has
/// no `effect: permit`, so forcing both into one flat struct is the contortion to avoid.
// ponytail: no `Domain` discriminator field. `container` is a repo-relative file path, which already
// disambiguates a YAML entry from a JSON or Rust one. Add a Domain enum only when two extractors
// genuinely collide on one container, which cannot happen while container is a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// A build-graph node: the buck2 target whose action produced this shard.
    Target,
    /// A tracked file. The universal reachability leaf, and the only node kind an edge may dangle
    /// onto (a `Refs` edge naming a file that does not exist is the defect we want representable).
    File,
    /// A named, addressable location inside a structured file — here, a YAML scalar's key path.
    Entry,
}

impl NodeKind {
    /// The stable lowercase wire tag, matching the serde rename. Used in digest pre-images so a
    /// digest never depends on Rust's `Debug` formatting.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            NodeKind::Target => "target",
            NodeKind::File => "file",
            NodeKind::Entry => "entry",
        }
    }
}

/// The stable, human-readable node identity. NOT a digest.
///
/// Editing file A changes no `NodeId` in file B; reformatting a file changes no `NodeId` at all.
/// This is the SCIP-over-LSIF lesson and ADR-0541 D1 precedent #7: fusing identity with content is
/// what LSIF got wrong.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId {
    /// Repo-relative path of the owning container. For a `File` node this IS the file; for an
    /// `Entry` it is the file the entry lives in; for a `Target` it is the buck2 target label.
    pub container: String,
    /// Path within the container: the `/`-joined YAML key path (sequence elements use their index).
    /// Empty for `File` and `Target` nodes, whose container is already the whole identity.
    pub path: String,
    /// The node kind.
    pub kind: NodeKind,
}

impl NodeId {
    /// A `File` node for a repo-relative path.
    #[must_use]
    pub fn file(container: impl Into<String>) -> Self {
        NodeId {
            container: container.into(),
            path: String::new(),
            kind: NodeKind::File,
        }
    }

    /// A `Target` node for a buck2 target label.
    #[must_use]
    pub fn target(label: impl Into<String>) -> Self {
        NodeId {
            container: label.into(),
            path: String::new(),
            kind: NodeKind::Target,
        }
    }

    /// An `Entry` node for a key path inside a file.
    #[must_use]
    pub fn entry(container: impl Into<String>, path: impl Into<String>) -> Self {
        NodeId {
            container: container.into(),
            path: path.into(),
            kind: NodeKind::Entry,
        }
    }
}

/// A graph node: nominal identity plus its own SHALLOW content-address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Node {
    /// The node's stable identity.
    pub id: NodeId,
    /// blake3 over this node's OWN canonical pre-image. Never includes a child's digest.
    pub digest: ContentHash,
}

/// The two relations the corpus graph needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Structural containment, exactly one level down: target→file, file→entry. The spanning forest
    /// a reachability BFS walks.
    Contains,
    /// One node names another. Here: a YAML scalar whose text is a repo-relative path.
    Refs,
}

/// A typed edge between two nodes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Edge {
    /// The relation this edge asserts.
    pub kind: EdgeKind,
    /// The naming node.
    pub src: NodeId,
    /// The named node, as a NAME rather than a fact pointer.
    ///
    /// A `dst` with no matching [`Node`] is a DANGLING reference: legal, representable, and exactly
    /// the ADR-0541 D2 reference-integrity defect class. Making this a fact id would render the very
    /// defect we want to detect unrepresentable.
    pub dst: NodeId,
}

/// One shard of the corpus graph: the output of exactly one buck2 extraction action.
///
/// Sharding is what keeps this free of the global-registry contention that has repeatedly wedged
/// repo-wide moves: one action emits one shard, there is no global graph file to serialize against,
/// and the merged graph is a query-time concatenation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphFace {
    /// [`SCHEMA_VERSION`] at the time of production.
    pub schema_version: u32,
    /// The buck2 target label whose action produced this shard.
    pub target: String,
    /// Sorted and de-duplicated, so the serialized bytes are order-independent.
    pub nodes: Vec<Node>,
    /// Sorted and de-duplicated.
    pub edges: Vec<Edge>,
    /// Index-integrity attestation (ADR-0541 D3). NON-EMPTY MEANS THE FACE IS INCOMPLETE.
    pub opaque: Vec<OpaqueReason>,
}

impl GraphFace {
    /// Build a shard from raw parts, sorting and de-duplicating for a canonical result.
    #[must_use]
    pub fn new(
        target: impl Into<String>,
        mut nodes: Vec<Node>,
        mut edges: Vec<Edge>,
        mut opaque: Vec<OpaqueReason>,
    ) -> Self {
        nodes.sort();
        nodes.dedup();
        edges.sort();
        edges.dedup();
        opaque.sort();
        opaque.dedup();
        GraphFace {
            schema_version: SCHEMA_VERSION,
            target: target.into(),
            nodes,
            edges,
            opaque,
        }
    }

    /// Serialize to canonical pretty JSON.
    ///
    /// Determinism holds because every field is a string, integer, or enum (no floats reach the
    /// face — a YAML float only ever contributes to a hex digest), the vectors are sorted, and serde
    /// emits struct fields in declaration order.
    ///
    /// Deliberately serializes the typed struct directly and NEVER routes bytes through
    /// `serde_json::Value`: reindeer unions `serde_json`'s `preserve_order` feature workspace-wide,
    /// which swaps `Value`'s map from `BTreeMap` to `IndexMap`. Structs are unaffected by that
    /// feature; `Value` is not. `face_producer_never_uses_serde_json_value` pins this.
    ///
    /// # Errors
    /// Returns the underlying `serde_json` error if serialization fails.
    pub fn to_canonical_json(&self) -> Result<String, serde_json::Error> {
        let mut out = serde_json::to_string_pretty(self)?;
        out.push('\n');
        Ok(out)
    }
}

/// The extraction result for one YAML file: its nodes, its edges, and what it could NOT resolve.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct YamlFacts {
    /// Clean nodes resolved from the file.
    pub nodes: Vec<Node>,
    /// Clean edges resolved from the file.
    pub edges: Vec<Edge>,
    /// Why part of the file could not be resolved. Non-empty means the face is INCOMPLETE.
    pub opaque: Vec<OpaqueReason>,
}

/// The NUL separator for digest pre-images. NUL cannot appear in a path or a YAML scalar we emit,
/// so the pre-image is unambiguous and no field-boundary collision is possible.
const FIELD_SEP: u8 = 0;

/// Normalize a buck2 `$SRCS` entry into a REPO-RELATIVE path.
///
/// buck2 expands `$SRCS` to PACKAGE-relative paths, and prefixes them with `./` (so a genrule in
/// `os/core/machine-config-domain` sees `././testdata/x.yaml`). Node identity must be repo-relative
/// or a `Refs` edge from one package can never resolve to a `File` node in another — the graph
/// would silently fragment into per-package islands that all look internally consistent.
///
/// `prefix` is the package's repo-relative directory (empty for the root package).
#[must_use]
pub fn repo_relative(prefix: &str, src: &str) -> String {
    let mut rest = src;
    while let Some(trimmed) = rest.strip_prefix("./") {
        rest = trimmed;
    }
    let prefix = prefix.trim_matches('/');
    if prefix.is_empty() {
        rest.to_owned()
    } else {
        format!("{prefix}/{rest}")
    }
}

/// Extract graph facts from one YAML file.
///
/// `path` is the repo-relative path (it becomes the node container, so it must be the repo-relative
/// form, not a `buck-out` materialization path). `source` is the file's UTF-8 contents.
///
/// A parse failure yields an [`OpaqueReason::ParseError`] and the `File` node WITHOUT entries —
/// never a silently clean empty result.
#[must_use]
pub fn extract(path: &str, source: &str) -> YamlFacts {
    let file_id = NodeId::file(path);
    let mut nodes = vec![Node {
        id: file_id.clone(),
        digest: ContentHash::of(source.as_bytes()),
    }];
    let mut edges = Vec::new();
    let mut opaque = Vec::new();

    match Yaml::load_from_str(source) {
        Ok(docs) => {
            for (index, doc) in docs.iter().enumerate() {
                // A multi-document file prefixes each document with its index so two documents
                // cannot collide on one key path.
                let root = if docs.len() > 1 {
                    index.to_string()
                } else {
                    String::new()
                };
                walk(path, &root, doc, &file_id, &mut nodes, &mut edges);
            }
        }
        Err(error) => {
            opaque.push(OpaqueReason::ParseError(format!("{path}: {error}")));
        }
    }

    YamlFacts {
        nodes,
        edges,
        opaque,
    }
}

/// Recursively walk a YAML node, emitting an `Entry` node per scalar leaf.
fn walk(
    path: &str,
    key_path: &str,
    node: &Yaml<'_>,
    file_id: &NodeId,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
) {
    match node {
        Yaml::Mapping(map) => {
            for (key, value) in map {
                let Some(key) = scalar_key(key) else {
                    continue;
                };
                walk(path, &join(key_path, &key), value, file_id, nodes, edges);
            }
        }
        Yaml::Sequence(items) => {
            for (index, item) in items.iter().enumerate() {
                walk(
                    path,
                    &join(key_path, &index.to_string()),
                    item,
                    file_id,
                    nodes,
                    edges,
                );
            }
        }
        Yaml::Value(scalar) => {
            let text = render(scalar);
            let entry_id = NodeId::entry(path, key_path);

            // SHALLOW pre-image: this entry's own container + key path + scalar text. No child
            // digest, no parent digest — editing a sibling key cannot churn this one.
            let mut pre = Vec::new();
            pre.extend_from_slice(path.as_bytes());
            pre.push(FIELD_SEP);
            pre.extend_from_slice(key_path.as_bytes());
            pre.push(FIELD_SEP);
            pre.extend_from_slice(text.as_bytes());

            nodes.push(Node {
                id: entry_id.clone(),
                digest: ContentHash::of(&pre),
            });
            edges.push(Edge {
                kind: EdgeKind::Contains,
                src: file_id.clone(),
                dst: entry_id.clone(),
            });

            if let Scalar::String(value) = scalar {
                if let Some(target) = repo_path_reference(value) {
                    edges.push(Edge {
                        kind: EdgeKind::Refs,
                        src: entry_id,
                        dst: NodeId::file(target),
                    });
                }
            }
        }
        // Alias/Tagged/Representation/BadValue carry no addressable scalar leaf for v1. They are not
        // routed to `opaque` because they are structurally present and resolvable later, unlike a
        // parse failure which loses the whole document.
        _ => {}
    }
}

/// Join a key-path segment, avoiding a leading separator at the document root.
fn join(prefix: &str, segment: &str) -> String {
    if prefix.is_empty() {
        segment.to_owned()
    } else {
        format!("{prefix}/{segment}")
    }
}

/// The text of a mapping key, if the key is a scalar. Non-scalar keys (a YAML complex key) are
/// skipped rather than invented.
fn scalar_key(key: &Yaml<'_>) -> Option<String> {
    match key {
        Yaml::Value(scalar) => Some(render(scalar)),
        _ => None,
    }
}

/// Render a scalar to stable text for the digest pre-image.
///
/// Floats render through Rust's shortest-round-trip `f64` Display, which is deterministic. No float
/// ever reaches the serialized face — only the hex digest computed from this text does.
fn render(scalar: &Scalar<'_>) -> String {
    match scalar {
        Scalar::Null => "null".to_owned(),
        Scalar::Boolean(value) => value.to_string(),
        Scalar::Integer(value) => value.to_string(),
        Scalar::FloatingPoint(value) => value.into_inner().to_string(),
        Scalar::String(value) => value.to_string(),
    }
}

/// Does this scalar name a repo-relative file path? Returns the path if so.
///
/// Conservative on purpose: a false `Refs` edge would make the dangling-reference query cry wolf,
/// and that query is the whole point of having edges.
// ponytail: syntactic heuristic, no filesystem probe (the kernel is pure and cannot look). The
// ceiling: it misses paths without an extension (`docs/decisions`) and misses references expressed
// as IDs rather than paths (`ADR-0541`). Upgrade path is a per-schema reference map keyed by YAML
// key path, which needs a schema registry that does not exist yet — not a better regex.
fn repo_path_reference(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty()
        || value.contains(char::is_whitespace)
        || value.contains("://")
        || value.starts_with('/')
        || !value.contains('/')
    {
        return None;
    }
    // Require a file extension on the last segment, so `some/prefix` (a namespace) is not mistaken
    // for a file. `..` is rejected: a repo-relative reference never escapes the root.
    let last = value.rsplit('/').next()?;
    if value.contains("..") || !last.contains('.') || last.starts_with('.') || last.ends_with('.') {
        return None;
    }
    Some(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    // buck2 hands genrules PACKAGE-relative `./`-prefixed paths. Identity must be repo-relative, or
    // a Refs edge can never resolve across packages and the graph fragments into islands that each
    // look internally consistent.
    #[test]
    fn srcs_paths_normalize_to_repo_relative() {
        assert_eq!(
            repo_relative("os/core/machine-config-domain", "././testdata/x.yaml"),
            "os/core/machine-config-domain/testdata/x.yaml"
        );
        assert_eq!(repo_relative("registry", "./adr/x.yaml"), "registry/adr/x.yaml");
        assert_eq!(repo_relative("", "./x.yaml"), "x.yaml");
        assert_eq!(repo_relative("specs/", "x.yaml"), "specs/x.yaml");
        // Already-normalized input is a fixed point, so double normalization is harmless.
        assert_eq!(repo_relative("", "a/b.yaml"), "a/b.yaml");
    }

    #[test]
    fn file_node_digests_the_bytes() {
        let facts = extract("a/b.yaml", "k: v\n");
        let file = facts
            .nodes
            .iter()
            .find(|n| n.id.kind == NodeKind::File)
            .unwrap();
        assert_eq!(file.id.container, "a/b.yaml");
        assert_eq!(file.digest, ContentHash::of(b"k: v\n"));
    }

    #[test]
    fn entry_nodes_carry_the_key_path() {
        let facts = extract("a.yaml", "metadata:\n  name: thing\n");
        let entry = facts
            .nodes
            .iter()
            .find(|n| n.id.kind == NodeKind::Entry)
            .unwrap();
        assert_eq!(entry.id.path, "metadata/name");
    }

    #[test]
    fn sequences_index_their_elements() {
        let facts = extract("a.yaml", "roots:\n  - one\n  - two\n");
        let paths: Vec<&str> = facts
            .nodes
            .iter()
            .filter(|n| n.id.kind == NodeKind::Entry)
            .map(|n| n.id.path.as_str())
            .collect();
        assert_eq!(paths, vec!["roots/0", "roots/1"]);
    }

    // The anti-Merkle commitment: a sibling edit must not churn an unrelated entry's digest. If a
    // roll-up digest is ever added, this test fails — which is the point.
    #[test]
    fn sibling_edit_does_not_churn_an_unrelated_entry_digest() {
        let before = extract("a.yaml", "one: keep\ntwo: old\n");
        let after = extract("a.yaml", "one: keep\ntwo: NEW\n");
        let pick = |facts: &YamlFacts, path: &str| {
            facts
                .nodes
                .iter()
                .find(|n| n.id.path == path)
                .unwrap()
                .digest
                .clone()
        };
        assert_eq!(pick(&before, "one"), pick(&after, "one"));
        assert_ne!(pick(&before, "two"), pick(&after, "two"));
    }

    #[test]
    fn reformatting_preserves_node_identity() {
        let a = extract("a.yaml", "k:   v\n");
        let b = extract("a.yaml", "k: v\n");
        let ids = |facts: &YamlFacts| -> Vec<NodeId> {
            facts.nodes.iter().map(|n| n.id.clone()).collect()
        };
        assert_eq!(ids(&a), ids(&b));
    }

    #[test]
    fn path_valued_scalars_emit_a_refs_edge() {
        let facts = extract("a.yaml", "spec: docs/decisions/ADR-0541.md\n");
        let refs: Vec<&Edge> = facts
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Refs)
            .collect();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].dst, NodeId::file("docs/decisions/ADR-0541.md"));
    }

    #[test]
    fn non_paths_do_not_emit_refs_edges() {
        for source in [
            "a: plain\n",
            "a: https://example.com/x.md\n",
            "a: /etc/passwd\n",
            "a: some/prefix\n",
            "a: ../escape.md\n",
            "a: has space/x.md\n",
        ] {
            let facts = extract("a.yaml", source);
            assert!(
                !facts.edges.iter().any(|e| e.kind == EdgeKind::Refs),
                "unexpected Refs edge from {source:?}"
            );
        }
    }

    // Fail-loud, not fail-empty: a broken file must be distinguishable from a clean empty one.
    #[test]
    fn unparseable_yaml_is_opaque_not_silently_clean() {
        let facts = extract("bad.yaml", "a:\n\t- tab indent is illegal\n");
        assert!(!facts.opaque.is_empty(), "parse failure must be reported");
        assert_eq!(facts.opaque[0].category(), "parse_error");
        assert!(!facts.nodes.iter().any(|n| n.id.kind == NodeKind::Entry));
    }

    #[test]
    fn face_serialization_is_deterministic_regardless_of_input_order() {
        let facts = extract("a.yaml", "b: 2\na: 1\n");
        let forward = GraphFace::new(
            "//x:y",
            facts.nodes.clone(),
            facts.edges.clone(),
            facts.opaque.clone(),
        );
        let mut reversed_nodes = facts.nodes.clone();
        reversed_nodes.reverse();
        let mut reversed_edges = facts.edges.clone();
        reversed_edges.reverse();
        let backward = GraphFace::new("//x:y", reversed_nodes, reversed_edges, facts.opaque);
        assert_eq!(
            forward.to_canonical_json().unwrap(),
            backward.to_canonical_json().unwrap()
        );
    }

    #[test]
    fn canonical_json_is_2_space_lf_trailing_newline() {
        let face = GraphFace::new("//x:y", vec![], vec![], vec![]);
        let json = face.to_canonical_json().unwrap();
        assert!(json.ends_with("}\n"), "trailing newline required");
        assert!(!json.contains('\r'), "LF only");
        assert!(json.contains("\n  \"schema_version\""), "2-space indent");
    }

    // F5 guard: reindeer unions serde_json's `preserve_order` feature workspace-wide, which changes
    // `Value`'s map type. Structs are immune; `Value` is not. Pin that the producer never uses it.
    #[test]
    fn face_producer_never_uses_serde_json_value() {
        let source = include_str!("lib.rs");
        let producer = source
            .split("mod tests")
            .next()
            .expect("lib.rs has a non-test prefix");
        // CODE lines only. Comments legitimately name the type while explaining why it is avoided,
        // and an earlier version of this test fired on its own doc comment — the probe was wrong,
        // not the code. Scanning prose is how a check ends up measuring itself.
        let offenders: Vec<&str> = producer
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .filter(|line| line.contains("serde_json::Value"))
            .collect();
        assert!(
            offenders.is_empty(),
            "the face producer must serialize the typed struct, never serde_json::Value: {offenders:?}"
        );
    }
}
