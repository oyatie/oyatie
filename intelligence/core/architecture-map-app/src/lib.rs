//! Foundry architecture-map app — filesystem walker that builds an
//! ArchitectureMap from the live workspace and emits it as JSON to
//! `registry/graph/architecture-map.json` (or any path).
//!
//! Sources walked:
//!   - root Cargo.toml `members = [...]` → Crate nodes
//!   - registry/microservices.json → Microservice nodes
//!   - registry/bounded-contexts.json → BoundedContext nodes
//!     (+ `Contains` edges from owning microservice)
//!   - contracts/*.openapi.yaml → OpenApiContract nodes
//!     (+ `Exposes` edges from BC if declared)
//!   - registry/cedar-fragments.json → CedarFragment nodes
//!     (+ `Governs` edges to OpenAPI contracts they protect)
//!
//! Pure std-only: parses each input via small line-based extractors.
//! No serde, no toml-rs, no yaml-rs deps. Aligns with the
//! "support-everything-ourselves with 0-to-minimal-dependency" policy.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use intelligence_architecture_map_kernel::{
    ArchitectureMap, Edge, EdgeKind, MapError, Node, NodeId, NodeKind,
};

#[derive(Debug)]
pub enum MapBuildError {
    Io { path: PathBuf, source: String },
    Map(MapError),
}

impl From<MapError> for MapBuildError {
    fn from(error: MapError) -> Self {
        Self::Map(error)
    }
}

/// Build a full architecture map by walking the workspace at `root`.
pub fn build_map(root: &Path) -> Result<ArchitectureMap, MapBuildError> {
    let mut map = ArchitectureMap::new();

    // Crates from the root workspace, with member GLOBS EXPANDED.
    //
    // This previously fed `parse_cargo_members` straight into the node set, so a `members` entry
    // like `*/core/*` became a literal node named `*/core/*` and the hundreds of crates it selects
    // got no node at all — `intelligence/core/api` among them. The map lost nearly the whole
    // workspace while the freshness gate stayed green, because that gate compares the committed
    // map against output from this same emitter: both sides were equally wrong, so nothing could
    // notice. Expansion goes through the Cargo-faithful member kernel that the workspace gates
    // already use, so the node set matches what Cargo itself resolves.
    let cargo_toml = root.join("Cargo.toml");
    let _ = read(&cargo_toml)?;
    let expanded = oya_workspace_members_kernel::scan_member_dirs(root)
        .map(|scan| scan.member_dirs)
        .unwrap_or_default();
    for crate_path in expanded {
        let id = NodeId(crate_path.clone());
        let label = crate_path
            .rsplit('/')
            .next()
            .unwrap_or(&crate_path)
            .to_string();
        let _ = map.add_node(Node {
            id,
            kind: NodeKind::Crate,
            label,
            owning_team: None,
        });
    }

    // Microservices.
    let microservices_path = root.join("registry/microservices.json");
    if microservices_path.exists() {
        let text = read(&microservices_path)?;
        for ms in parse_json_string_array_values(&text, "microservice_id") {
            let _ = map.add_node(Node {
                id: NodeId(ms.clone()),
                kind: NodeKind::Microservice,
                label: ms,
                owning_team: None,
            });
        }
    }

    // Bounded contexts (+ contains edges).
    let bc_path = root.join("registry/bounded-contexts.json");
    if bc_path.exists() {
        let text = read(&bc_path)?;
        for (bc_id, microservice_id) in parse_bc_pairs(&text) {
            let _ = map.add_node(Node {
                id: NodeId(bc_id.clone()),
                kind: NodeKind::BoundedContext,
                label: bc_id.clone(),
                owning_team: None,
            });
            if map.node(&NodeId(microservice_id.clone())).is_some() {
                let _ = map.add_edge(Edge {
                    source: NodeId(microservice_id),
                    target: NodeId(bc_id),
                    kind: EdgeKind::Contains,
                });
            }
        }
    }

    // OpenAPI contracts.
    let contracts_dir = root.join("contracts");
    let mut contract_ids: BTreeSet<String> = BTreeSet::new();
    if contracts_dir.is_dir() {
        for entry in fs::read_dir(&contracts_dir).map_err(|error| MapBuildError::Io {
            path: contracts_dir.clone(),
            source: error.to_string(),
        })? {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !name.ends_with(".openapi.yaml") {
                continue;
            }
            // Use the workspace-relative path as the node id.
            let rel = relativize(root, &path);
            contract_ids.insert(rel.clone());
            let _ = map.add_node(Node {
                id: NodeId(rel.clone()),
                kind: NodeKind::OpenApiContract,
                label: name.to_string(),
                owning_team: None,
            });
        }
    }

    // Cedar fragments (+ Governs edges to contracts in consumed_by_openapi[]).
    let cedar_path = root.join("registry/cedar-fragments.json");
    if cedar_path.exists() {
        let text = read(&cedar_path)?;
        for (fragment_id, consumed_by) in parse_fragment_consumed_pairs(&text) {
            let frag_node_id = NodeId(fragment_id.clone());
            let _ = map.add_node(Node {
                id: frag_node_id.clone(),
                kind: NodeKind::CedarFragment,
                label: fragment_id,
                owning_team: None,
            });
            for contract in consumed_by {
                if contract_ids.contains(&contract) {
                    let _ = map.add_edge(Edge {
                        source: frag_node_id.clone(),
                        target: NodeId(contract),
                        kind: EdgeKind::Governs,
                    });
                }
            }
        }
    }

    Ok(map)
}

/// Emit `map` as JSON to `out_path`. Std-only writer: no serde.
pub fn emit_json(map: &ArchitectureMap, out_path: &Path) -> Result<(), MapBuildError> {
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|error| MapBuildError::Io {
            path: parent.to_path_buf(),
            source: error.to_string(),
        })?;
    }
    let body = render_json(map);
    fs::write(out_path, body).map_err(|error| MapBuildError::Io {
        path: out_path.to_path_buf(),
        source: error.to_string(),
    })
}

fn render_json(map: &ArchitectureMap) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"$schema_ref\": \"specs/knowledge-graph-schema.json\",\n");
    out.push_str("  \"_artifact_id\": \"architecture-map\",\n");
    out.push_str(
        "  \"_meta\": { \"emitter\": \"intelligence-architecture-map-app::build_map\", \"purpose\": \"Generated architecture graph of crates, contracts, registries, and ownership edges for repository navigation and drift checks.\" },\n",
    );
    out.push_str("  \"nodes\": [\n");
    let nodes: Vec<&Node> = map.nodes().collect();
    for (i, node) in nodes.iter().enumerate() {
        let trailing = if i + 1 == nodes.len() { "" } else { "," };
        out.push_str(&format!(
            "    {{ \"id\": \"{}\", \"kind\": \"{}\", \"label\": \"{}\" }}{}\n",
            escape_json(&node.id.0),
            node.kind.name(),
            escape_json(&node.label),
            trailing
        ));
    }
    out.push_str("  ],\n");
    out.push_str("  \"edges\": [\n");
    let edges = map.edges();
    for (i, edge) in edges.iter().enumerate() {
        let trailing = if i + 1 == edges.len() { "" } else { "," };
        out.push_str(&format!(
            "    {{ \"source\": \"{}\", \"target\": \"{}\", \"kind\": \"{}\" }}{}\n",
            escape_json(&edge.source.0),
            escape_json(&edge.target.0),
            edge.kind.name(),
            trailing
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn read(path: &Path) -> Result<String, MapBuildError> {
    fs::read_to_string(path).map_err(|error| MapBuildError::Io {
        path: path.to_path_buf(),
        source: error.to_string(),
    })
}

fn relativize(root: &Path, full: &Path) -> String {
    full.strip_prefix(root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| full.display().to_string())
}

/// Parse the `members = [...]` array out of root Cargo.toml.
/// Std-only TOML scan: find `[workspace]` table, then the `members` key, then
/// collect each quoted string until the closing `]`.
fn parse_cargo_members(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_workspace = false;
    let mut in_members = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace]" {
            in_workspace = true;
            continue;
        }
        if in_workspace && trimmed.starts_with('[') && trimmed != "[workspace]" {
            in_workspace = false;
            in_members = false;
            continue;
        }
        if !in_workspace {
            continue;
        }
        if trimmed.starts_with("members") && trimmed.contains('[') {
            in_members = true;
            continue;
        }
        if !in_members {
            continue;
        }
        if trimmed.starts_with(']') {
            in_members = false;
            continue;
        }
        // Match `"path",` or `'path',`.
        let inner = trimmed.trim_end_matches(',').trim_end_matches('"');
        if let Some(stripped) = inner.strip_prefix('"') {
            out.push(stripped.to_string());
        } else if let Some(stripped) = inner.strip_prefix('\'') {
            out.push(stripped.trim_end_matches('\'').to_string());
        }
    }
    out
}

/// Parse a JSON document and collect every value of `field_name` that appears
/// as a string field anywhere in the document. Used for "microservice_id"
/// extraction from microservices.json.
fn parse_json_string_array_values(text: &str, field_name: &str) -> Vec<String> {
    let needle = format!("\"{field_name}\"");
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(idx) = rest.find(&needle) {
        rest = &rest[idx + needle.len()..];
        let Some(colon) = rest.find(':') else { break };
        let after = rest[colon + 1..].trim_start();
        if let Some(value) = read_json_string(after) {
            out.push(value);
        }
    }
    out
}

/// Parse `bc_id` + `microservice_id` pairs from bounded-contexts.json.
fn parse_bc_pairs(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current_bc: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(v) = json_field(trimmed, "bc_id") {
            current_bc = Some(v);
            continue;
        }
        if let Some(v) = json_field(trimmed, "microservice_id")
            && let Some(bc) = current_bc.take()
        {
            out.push((bc, v));
        }
    }
    out
}

/// Parse `fragment_id` + `consumed_by_openapi[]` pairs from cedar-fragments.json.
fn parse_fragment_consumed_pairs(text: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_consumed: Vec<String> = Vec::new();
    let mut in_consumed_array = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(v) = json_field(trimmed, "fragment_id") {
            if let Some(prev) = current_id.take() {
                out.push((prev, std::mem::take(&mut current_consumed)));
            }
            current_id = Some(v);
            continue;
        }
        if trimmed.starts_with("\"consumed_by_openapi\"") && trimmed.ends_with('[') {
            in_consumed_array = true;
            continue;
        }
        if in_consumed_array {
            if trimmed.starts_with(']') {
                in_consumed_array = false;
                continue;
            }
            let inner = trimmed.trim_end_matches(',');
            if let Some(stripped) = inner.strip_prefix('"')
                && let Some(value) = stripped.strip_suffix('"')
            {
                current_consumed.push(value.to_string());
            }
        }
    }
    if let Some(id) = current_id {
        out.push((id, current_consumed));
    }
    out
}

/// Return the string value of `"key": "value"` on this trimmed line, if it
/// matches.
fn json_field(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let after_key = line.strip_prefix(&needle)?.trim_start().strip_prefix(':')?;
    read_json_string(after_key.trim_start())
}

/// Read a JSON string starting at the leading `"`. Strips trailing `,` if any.
fn read_json_string(input: &str) -> Option<String> {
    let after_quote = input.strip_prefix('"')?;
    let close = after_quote.find('"')?;
    Some(after_quote[..close].to_string())
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cargo_members_basic() {
        let toml = r#"[workspace]
resolver = "2"
members = [
  "crates/a",
  "crates/b",
  "crates/c-d-e"
]

[workspace.dependencies]
serde = "1"
"#;
        let members = parse_cargo_members(toml);
        assert_eq!(members, vec!["crates/a", "crates/b", "crates/c-d-e"]);
    }

    #[test]
    fn parse_cargo_members_empty() {
        assert!(parse_cargo_members("").is_empty());
        assert!(parse_cargo_members("[package]\nname = \"foo\"\n").is_empty());
    }

    #[test]
    fn parse_json_string_array_values_extracts_field() {
        let json = r#"{"a": [{"microservice_id": "ops"}, {"microservice_id": "foundry"}]}"#;
        let out = parse_json_string_array_values(json, "microservice_id");
        assert_eq!(out, vec!["ops", "foundry"]);
    }

    #[test]
    fn parse_bc_pairs_extracts_pairs() {
        let json = r#"{
  "bounded_contexts": [
    {
      "bc_id": "ops/docs-portal",
      "microservice_id": "ops"
    },
    {
      "bc_id": "ops/workspace",
      "microservice_id": "ops"
    }
  ]
}"#;
        let pairs = parse_bc_pairs(json);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("ops/docs-portal".to_string(), "ops".to_string()));
        assert_eq!(pairs[1], ("ops/workspace".to_string(), "ops".to_string()));
    }

    #[test]
    fn parse_fragment_consumed_pairs_extracts_pairs() {
        let json = r#"{
  "fragments": [
    {
      "fragment_id": "ops-internal-public",
      "consumed_by_openapi": [
        "contracts/ops-workspace-shell-v1.openapi.yaml",
        "contracts/ops-docs-v1.openapi.yaml"
      ]
    },
    {
      "fragment_id": "ops-tenant-private",
      "consumed_by_openapi": [
        "contracts/ops-workspace-shell-v1.openapi.yaml"
      ]
    }
  ]
}"#;
        let pairs = parse_fragment_consumed_pairs(json);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0, "ops-internal-public");
        assert_eq!(pairs[0].1.len(), 2);
        assert_eq!(pairs[1].0, "ops-tenant-private");
        assert_eq!(pairs[1].1.len(), 1);
    }

    #[test]
    fn render_json_round_trip_shape() {
        let mut map = ArchitectureMap::new();
        map.add_node(Node {
            id: NodeId("a".into()),
            kind: NodeKind::Microservice,
            label: "a".into(),
            owning_team: None,
        })
        .unwrap();
        map.add_node(Node {
            id: NodeId("a/b".into()),
            kind: NodeKind::BoundedContext,
            label: "a/b".into(),
            owning_team: None,
        })
        .unwrap();
        map.add_edge(Edge {
            source: NodeId("a".into()),
            target: NodeId("a/b".into()),
            kind: EdgeKind::Contains,
        })
        .unwrap();
        let body = render_json(&map);
        assert!(body.contains("\"_artifact_id\": \"architecture-map\""));
        assert!(body.contains("\"id\": \"a\""));
        assert!(body.contains("\"kind\": \"microservice\""));
        assert!(body.contains("\"kind\": \"bounded-context\""));
        assert!(body.contains("\"source\": \"a\""));
        assert!(body.contains("\"target\": \"a/b\""));
        assert!(body.contains("\"kind\": \"contains\""));
    }

    #[test]
    fn json_field_extracts_string_value() {
        assert_eq!(
            json_field(r#""foo": "bar","#, "foo"),
            Some("bar".to_string())
        );
        assert_eq!(
            json_field(r#""foo": "bar""#, "foo"),
            Some("bar".to_string())
        );
        assert!(json_field(r#""other": "bar""#, "foo").is_none());
    }

    /// A `members` glob must become the crates it selects, never a literal node.
    ///
    /// Regression: `*/core/*` was emitted verbatim, so the map held 41 nodes (15 of them globs)
    /// instead of the workspace, and the freshness gate could not see it because it diffs the
    /// committed map against this same emitter.
    #[test]
    fn build_map_expands_member_globs_into_real_crates() {
        let map = build_map(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(3)
                .unwrap(),
        )
        .expect("live workspace map");
        let glob_nodes: Vec<&str> = map
            .nodes_of_kind(NodeKind::Crate)
            .filter(|n| n.id.0.contains('*'))
            .map(|n| n.id.0.as_str())
            .collect();
        assert!(
            glob_nodes.is_empty(),
            "member globs must be expanded, not serialized as nodes: {glob_nodes:?}"
        );
        let crate_nodes = map.nodes_of_kind(NodeKind::Crate).count();
        assert!(
            crate_nodes > 500,
            "expected the expanded workspace (882 members today), got {crate_nodes} crate nodes"
        );
    }

    #[test]
    fn build_map_handles_missing_files() {
        // Build map against a tempdir with NO files — should produce an empty
        // map without panicking. (Cargo.toml is required, so we create just that.)
        let tmpdir = std::env::temp_dir().join(format!("oya-arch-map-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmpdir);
        fs::create_dir_all(&tmpdir).unwrap();
        fs::write(tmpdir.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let map = build_map(&tmpdir).unwrap();
        assert_eq!(map.node_count(), 0);
        assert_eq!(map.edge_count(), 0);
        let _ = fs::remove_dir_all(&tmpdir);
    }

    #[test]
    fn build_map_populates_crate_nodes() {
        let tmpdir =
            std::env::temp_dir().join(format!("oya-arch-map-test-crates-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmpdir);
        fs::create_dir_all(&tmpdir).unwrap();
        fs::write(
            tmpdir.join("Cargo.toml"),
            "[workspace]\nmembers = [\n  \"crates/foo\",\n  \"crates/bar\"\n]\n",
        )
        .unwrap();
        // The members must EXIST on disk. Member resolution is Cargo-faithful now, and Cargo does
        // not treat a declared-but-absent directory as a crate; the old literal parse counted the
        // strings alone, which is exactly the defect that let `*/core/*` become a node.
        for member in ["crates/foo", "crates/bar"] {
            fs::create_dir_all(tmpdir.join(member)).unwrap();
            fs::write(
                tmpdir.join(member).join("Cargo.toml"),
                "[package]\nname = \"x\"\nedition = \"2024\"\nversion = \"0.1.0\"\n",
            )
            .unwrap();
        }
        let map = build_map(&tmpdir).unwrap();
        let crates: Vec<&Node> = map.nodes_of_kind(NodeKind::Crate).collect();
        assert_eq!(crates.len(), 2);
        let _ = fs::remove_dir_all(&tmpdir);
    }

    #[test]
    fn emit_json_writes_file() {
        let tmpdir =
            std::env::temp_dir().join(format!("oya-arch-map-test-emit-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmpdir);
        fs::create_dir_all(&tmpdir).unwrap();
        let mut map = ArchitectureMap::new();
        map.add_node(Node {
            id: NodeId("x".into()),
            kind: NodeKind::Crate,
            label: "x".into(),
            owning_team: None,
        })
        .unwrap();
        let out_path = tmpdir.join("out.json");
        emit_json(&map, &out_path).unwrap();
        assert!(out_path.exists());
        let body = fs::read_to_string(&out_path).unwrap();
        assert!(body.contains("\"id\": \"x\""));
        let _ = fs::remove_dir_all(&tmpdir);
    }
}
