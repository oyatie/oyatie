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
//! Registry and contract inputs use small line-based extractors. Workspace
//! membership deliberately reuses `oya-workspace-members-kernel` so Cargo glob
//! semantics have one canonical implementation (ADR-0538).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use oya_intelligence_architecture_map_kernel::{
    ArchitectureMap, Edge, EdgeKind, MapError, Node, NodeId, NodeKind,
};
use oya_workspace_members_kernel::resolve_member_dirs;
use sha2::{Digest, Sha256};

pub const PRODUCER_VERSION: &str = "oya-intelligence-architecture-map-app/v2";

#[derive(Debug)]
pub enum MapBuildError {
    Io {
        path: PathBuf,
        source: String,
    },
    Map(MapError),
    WorkspaceMembers(String),
    InvalidExistingArtifact {
        path: PathBuf,
        reason: String,
    },
    CountDiscontinuity {
        previous_node_count: usize,
        proposed_node_count: usize,
        minimum_expected_node_count: usize,
    },
    IncompleteCoverage {
        missing_workspace_crate_ids: Vec<String>,
        orphan_crate_ids: Vec<String>,
    },
}

impl From<MapError> for MapBuildError {
    fn from(error: MapError) -> Self {
        Self::Map(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrateCoverageReport {
    pub resolved_workspace_crate_count: usize,
    pub represented_workspace_crate_count: usize,
    pub missing_workspace_crate_ids: Vec<String>,
    pub orphan_crate_ids: Vec<String>,
    /// Coverage in basis points (`10_000` = 100%) to keep comparisons exact.
    pub coverage_ratio_basis_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchitectureMapProvenance {
    pub producer_version: &'static str,
    pub source_digest_sha256: String,
}

#[derive(Debug)]
pub struct ArchitectureMapArtifact {
    pub map: ArchitectureMap,
    pub provenance: ArchitectureMapProvenance,
    pub coverage: CrateCoverageReport,
}

pub fn analyze_crate_coverage(
    resolved_members: &[String],
    represented_crate_ids: &[String],
    manifest_exists: impl Fn(&str) -> bool,
) -> CrateCoverageReport {
    let resolved: BTreeSet<&str> = resolved_members.iter().map(String::as_str).collect();
    let represented: BTreeSet<&str> = represented_crate_ids.iter().map(String::as_str).collect();

    let missing_workspace_crate_ids = resolved
        .difference(&represented)
        .map(|id| (*id).to_owned())
        .collect::<Vec<_>>();
    let orphan_crate_ids = represented
        .iter()
        .filter(|id| !resolved.contains(**id) || !manifest_exists(id))
        .map(|id| (*id).to_owned())
        .collect::<Vec<_>>();
    let represented_workspace_crate_count = resolved
        .intersection(&represented)
        .filter(|id| manifest_exists(id))
        .count();
    let coverage_ratio_basis_points = if resolved.is_empty() {
        10_000
    } else {
        ((represented_workspace_crate_count * 10_000) / resolved.len()) as u32
    };

    CrateCoverageReport {
        resolved_workspace_crate_count: resolved.len(),
        represented_workspace_crate_count,
        missing_workspace_crate_ids,
        orphan_crate_ids,
        coverage_ratio_basis_points,
    }
}

pub fn build_artifact(root: &Path) -> Result<ArchitectureMapArtifact, MapBuildError> {
    let resolved_members = resolve_member_dirs(root)
        .map_err(|error| MapBuildError::WorkspaceMembers(error.to_string()))?;
    let map = build_map(root)?;
    let represented_crate_ids = map
        .nodes_of_kind(NodeKind::Crate)
        .map(|node| node.id.0.clone())
        .collect::<Vec<_>>();
    let coverage = analyze_crate_coverage(&resolved_members, &represented_crate_ids, |id| {
        root.join(id).join("Cargo.toml").is_file()
    });
    Ok(ArchitectureMapArtifact {
        map,
        provenance: ArchitectureMapProvenance {
            producer_version: PRODUCER_VERSION,
            source_digest_sha256: source_digest(root, &resolved_members)?,
        },
        coverage,
    })
}

/// Build a full architecture map by walking the workspace at `root`.
pub fn build_map(root: &Path) -> Result<ArchitectureMap, MapBuildError> {
    let mut map = ArchitectureMap::new();

    // Crates from canonical, glob-expanded root workspace membership.
    for crate_path in resolve_member_dirs(root)
        .map_err(|error| MapBuildError::WorkspaceMembers(error.to_string()))?
    {
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

/// Emit a provenance-bearing artifact atomically after fail-closed continuity checks.
/// A new snapshot may not drop below 80% of the previous node count in one regeneration.
pub fn emit_artifact_json(
    artifact: &ArchitectureMapArtifact,
    out_path: &Path,
) -> Result<(), MapBuildError> {
    const MIN_RETAINED_NODE_PERCENT: usize = 80;

    if artifact.coverage.coverage_ratio_basis_points != 10_000
        || artifact.coverage.resolved_workspace_crate_count
            != artifact.coverage.represented_workspace_crate_count
        || !artifact.coverage.missing_workspace_crate_ids.is_empty()
        || !artifact.coverage.orphan_crate_ids.is_empty()
    {
        return Err(MapBuildError::IncompleteCoverage {
            missing_workspace_crate_ids: artifact.coverage.missing_workspace_crate_ids.clone(),
            orphan_crate_ids: artifact.coverage.orphan_crate_ids.clone(),
        });
    }

    if out_path.is_file() {
        let previous = read(out_path)?;
        let previous_node_count = snapshot_node_count(&previous).ok_or_else(|| {
            MapBuildError::InvalidExistingArtifact {
                path: out_path.to_path_buf(),
                reason: "nodes array is missing or malformed".to_owned(),
            }
        })?;
        let proposed_node_count = artifact.map.node_count();
        let minimum_expected_node_count =
            (previous_node_count * MIN_RETAINED_NODE_PERCENT).div_ceil(100);
        if proposed_node_count < minimum_expected_node_count {
            return Err(MapBuildError::CountDiscontinuity {
                previous_node_count,
                proposed_node_count,
                minimum_expected_node_count,
            });
        }
    }

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|error| MapBuildError::Io {
            path: parent.to_path_buf(),
            source: error.to_string(),
        })?;
    }
    let temp_name = format!(
        ".{}.tmp-{}",
        out_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("architecture-map.json"),
        std::process::id()
    );
    let temp_path = out_path.with_file_name(temp_name);
    fs::write(&temp_path, render_artifact_json(artifact)).map_err(|error| MapBuildError::Io {
        path: temp_path.clone(),
        source: error.to_string(),
    })?;
    fs::rename(&temp_path, out_path).map_err(|error| MapBuildError::Io {
        path: out_path.to_path_buf(),
        source: error.to_string(),
    })
}

fn snapshot_node_count(body: &str) -> Option<usize> {
    let after_nodes = body.split_once("\"nodes\"")?.1;
    let array_start = after_nodes.find('[')?;
    let after_start = &after_nodes[array_start + 1..];
    let array_end = after_start.find(']')?;
    Some(parse_json_string_array_values(&after_start[..array_end], "id").len())
}

fn render_json(map: &ArchitectureMap) -> String {
    render_json_with_provenance(map, None)
}

pub fn render_artifact_json(artifact: &ArchitectureMapArtifact) -> String {
    render_json_with_provenance(
        &artifact.map,
        Some((&artifact.provenance, &artifact.coverage)),
    )
}

fn render_json_with_provenance(
    map: &ArchitectureMap,
    provenance: Option<(&ArchitectureMapProvenance, &CrateCoverageReport)>,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"$schema_ref\": \"specs/knowledge-graph-schema.json\",\n");
    out.push_str("  \"_artifact_id\": \"architecture-map\",\n");
    out.push_str(
        "  \"_meta\": { \"emitter\": \"oya-intelligence-architecture-map-app::build_map\", \"purpose\": \"Generated architecture graph of crates, contracts, registries, and ownership edges for repository navigation and drift checks.\" },\n",
    );
    if let Some((provenance, coverage)) = provenance {
        out.push_str("  \"provenance\": {\n");
        out.push_str(&format!(
            "    \"producer_version\": \"{}\",\n",
            escape_json(provenance.producer_version)
        ));
        out.push_str(&format!(
            "    \"source_digest_sha256\": \"{}\",\n",
            provenance.source_digest_sha256
        ));
        out.push_str(&format!(
            "    \"resolved_workspace_crate_count\": {},\n",
            coverage.resolved_workspace_crate_count
        ));
        out.push_str(&format!(
            "    \"represented_workspace_crate_count\": {},\n",
            coverage.represented_workspace_crate_count
        ));
        out.push_str(&format!(
            "    \"coverage_ratio\": {}.{:04},\n",
            coverage.coverage_ratio_basis_points / 10_000,
            coverage.coverage_ratio_basis_points % 10_000
        ));
        out.push_str(&format!(
            "    \"missing_workspace_crate_ids\": {},\n",
            render_string_array(&coverage.missing_workspace_crate_ids)
        ));
        out.push_str(&format!(
            "    \"orphan_crate_ids\": {}\n",
            render_string_array(&coverage.orphan_crate_ids)
        ));
        out.push_str("  },\n");
    }
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

fn render_string_array(values: &[String]) -> String {
    let body = values
        .iter()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{body}]")
}

fn source_digest(root: &Path, resolved_members: &[String]) -> Result<String, MapBuildError> {
    let mut paths = vec![
        root.join("Cargo.toml"),
        root.join("registry/microservices.json"),
        root.join("registry/bounded-contexts.json"),
        root.join("registry/cedar-fragments.json"),
    ];
    paths.extend(
        resolved_members
            .iter()
            .map(|member| root.join(member).join("Cargo.toml")),
    );
    let contracts_dir = root.join("contracts");
    if contracts_dir.is_dir() {
        let entries = fs::read_dir(&contracts_dir).map_err(|error| MapBuildError::Io {
            path: contracts_dir.clone(),
            source: error.to_string(),
        })?;
        for entry in entries {
            let path = entry
                .map_err(|error| MapBuildError::Io {
                    path: contracts_dir.clone(),
                    source: error.to_string(),
                })?
                .path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".openapi.yaml"))
            {
                paths.push(path);
            }
        }
    }
    paths.sort();
    paths.dedup();

    let mut digest = Sha256::new();
    for path in paths {
        let relative = relativize(root, &path);
        digest.update((relative.len() as u64).to_be_bytes());
        digest.update(relative.as_bytes());
        if path.is_file() {
            let bytes = fs::read(&path).map_err(|error| MapBuildError::Io {
                path: path.clone(),
                source: error.to_string(),
            })?;
            digest.update((bytes.len() as u64).to_be_bytes());
            digest.update(bytes);
        } else {
            digest.update(0_u64.to_be_bytes());
        }
    }
    Ok(format!("{:x}", digest.finalize()))
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
    fn build_map_expands_workspace_member_globs_and_requires_manifests() {
        let tmpdir =
            std::env::temp_dir().join(format!("oya-arch-map-test-crates-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmpdir);
        fs::create_dir_all(tmpdir.join("crates/foo")).unwrap();
        fs::create_dir_all(tmpdir.join("crates/bar")).unwrap();
        fs::create_dir_all(tmpdir.join("crates/not-a-crate")).unwrap();
        fs::write(
            tmpdir.join("crates/foo/Cargo.toml"),
            "[package]\nname = \"foo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            tmpdir.join("crates/bar/Cargo.toml"),
            "[package]\nname = \"bar\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            tmpdir.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .unwrap();
        let map = build_map(&tmpdir).unwrap();
        let crates: Vec<&str> = map
            .nodes_of_kind(NodeKind::Crate)
            .map(|node| node.id.0.as_str())
            .collect();
        assert_eq!(crates, vec!["crates/bar", "crates/foo"]);
        let _ = fs::remove_dir_all(&tmpdir);
    }

    #[test]
    fn crate_coverage_reports_missing_members_and_orphan_snapshot_ids() {
        let resolved = vec!["crates/a".to_owned(), "crates/b".to_owned()];
        let represented = vec!["crates/a".to_owned(), "crates/orphan".to_owned()];

        let report = analyze_crate_coverage(&resolved, &represented, |id| id != "crates/orphan");

        assert_eq!(report.resolved_workspace_crate_count, 2);
        assert_eq!(report.represented_workspace_crate_count, 1);
        assert_eq!(report.missing_workspace_crate_ids, vec!["crates/b"]);
        assert_eq!(report.orphan_crate_ids, vec!["crates/orphan"]);
        assert_eq!(report.coverage_ratio_basis_points, 5_000);
    }

    #[test]
    fn built_artifact_emits_producer_provenance_and_complete_coverage() {
        let tmpdir = std::env::temp_dir().join(format!(
            "oya-arch-map-test-provenance-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmpdir);
        fs::create_dir_all(tmpdir.join("crates/a")).unwrap();
        fs::write(
            tmpdir.join("crates/a/Cargo.toml"),
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            tmpdir.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .unwrap();

        let artifact = build_artifact(&tmpdir).unwrap();
        let body = render_artifact_json(&artifact);

        assert_eq!(artifact.provenance.producer_version, PRODUCER_VERSION);
        assert_eq!(artifact.provenance.source_digest_sha256.len(), 64);
        assert!(
            artifact
                .provenance
                .source_digest_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        );
        assert_eq!(artifact.coverage.resolved_workspace_crate_count, 1);
        assert_eq!(artifact.coverage.represented_workspace_crate_count, 1);
        assert_eq!(artifact.coverage.coverage_ratio_basis_points, 10_000);
        assert!(artifact.coverage.missing_workspace_crate_ids.is_empty());
        assert!(artifact.coverage.orphan_crate_ids.is_empty());
        for field in [
            "\"producer_version\"",
            "\"source_digest_sha256\"",
            "\"resolved_workspace_crate_count\": 1",
            "\"represented_workspace_crate_count\": 1",
            "\"coverage_ratio\": 1.0000",
            "\"missing_workspace_crate_ids\": []",
            "\"orphan_crate_ids\": []",
        ] {
            assert!(body.contains(field), "missing provenance field {field}");
        }
        let _ = fs::remove_dir_all(&tmpdir);
    }

    #[test]
    fn artifact_emission_refuses_suspicious_node_count_discontinuity() {
        let tmpdir = std::env::temp_dir().join(format!(
            "oya-arch-map-test-count-discontinuity-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmpdir);
        fs::create_dir_all(&tmpdir).unwrap();
        let out_path = tmpdir.join("architecture-map.json");
        let old_nodes = (0..10)
            .map(|index| format!(r#"{{"id":"old-{index}","kind":"crate"}}"#))
            .collect::<Vec<_>>()
            .join(",");
        fs::write(
            &out_path,
            format!(r#"{{"nodes":[{old_nodes}],"edges":[]}}"#),
        )
        .unwrap();

        let mut map = ArchitectureMap::new();
        map.add_node(Node {
            id: NodeId("new".into()),
            kind: NodeKind::Crate,
            label: "new".into(),
            owning_team: None,
        })
        .unwrap();
        let artifact = ArchitectureMapArtifact {
            map,
            provenance: ArchitectureMapProvenance {
                producer_version: PRODUCER_VERSION,
                source_digest_sha256: "0".repeat(64),
            },
            coverage: CrateCoverageReport {
                resolved_workspace_crate_count: 1,
                represented_workspace_crate_count: 1,
                missing_workspace_crate_ids: vec![],
                orphan_crate_ids: vec![],
                coverage_ratio_basis_points: 10_000,
            },
        };

        let error = emit_artifact_json(&artifact, &out_path).unwrap_err();

        assert!(matches!(
            error,
            MapBuildError::CountDiscontinuity {
                previous_node_count: 10,
                proposed_node_count: 1,
                ..
            }
        ));
        assert!(fs::read_to_string(&out_path).unwrap().contains("old-9"));
        let _ = fs::remove_dir_all(&tmpdir);
    }

    #[test]
    fn artifact_emission_refuses_incomplete_or_orphaned_crate_coverage() {
        let tmpdir = std::env::temp_dir().join(format!(
            "oya-arch-map-test-incomplete-coverage-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmpdir);
        fs::create_dir_all(&tmpdir).unwrap();
        let out_path = tmpdir.join("architecture-map.json");
        let artifact = ArchitectureMapArtifact {
            map: ArchitectureMap::new(),
            provenance: ArchitectureMapProvenance {
                producer_version: PRODUCER_VERSION,
                source_digest_sha256: "0".repeat(64),
            },
            coverage: CrateCoverageReport {
                resolved_workspace_crate_count: 2,
                represented_workspace_crate_count: 0,
                missing_workspace_crate_ids: vec!["crates/missing".to_owned()],
                orphan_crate_ids: vec!["crates/orphan".to_owned()],
                coverage_ratio_basis_points: 0,
            },
        };

        let error = emit_artifact_json(&artifact, &out_path).unwrap_err();

        assert!(matches!(
            error,
            MapBuildError::IncompleteCoverage {
                missing_workspace_crate_ids,
                orphan_crate_ids,
            } if missing_workspace_crate_ids == vec!["crates/missing"]
                && orphan_crate_ids == vec!["crates/orphan"]
        ));
        assert!(!out_path.exists());
        let _ = fs::remove_dir_all(&tmpdir);
    }
}
