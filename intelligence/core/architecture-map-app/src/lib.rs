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
    let expanded = workspace_members_kernel::scan_member_dirs(root)
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

mod parse;
mod render;

use parse::*;
use render::render_json;

#[cfg(test)]
mod tests {
    use super::*;

    /// A `members` glob must become the crates it selects, never a literal node.
    ///
    /// Regression: `*/core/*` was emitted verbatim, so the map held 41 nodes (15 of them globs)
    /// instead of the workspace, and the freshness gate could not see it because it diffs the
    /// committed map against this same emitter.
    ///
    /// The workspace under test is built here rather than read from the checkout. A test that
    /// walks out of its own inputs to find the repository root cannot state what it read, and
    /// buck2, which gives a target only the srcs it declares, refuses to compile it at all.
    #[test]
    fn build_map_expands_member_globs_into_real_crates() {
        let root = std::env::temp_dir().join(format!("arch-map-test-globs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for member in ["alpha/core/one", "beta/core/two"] {
            fs::create_dir_all(root.join(member).join("src")).unwrap();
            fs::write(
                root.join(member).join("Cargo.toml"),
                format!(
                    "[package]\nname = \"{}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
                    member.replace('/', "-")
                ),
            )
            .unwrap();
            fs::write(root.join(member).join("src/lib.rs"), "").unwrap();
        }
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"*/core/*\"]\n",
        )
        .unwrap();

        let map = build_map(&root).unwrap();
        let mut crates: Vec<&str> = map
            .nodes_of_kind(NodeKind::Crate)
            .map(|node| node.id.0.as_str())
            .collect();
        crates.sort_unstable();
        assert_eq!(
            crates,
            ["alpha/core/one", "beta/core/two"],
            "the glob must expand to the crates it selects, not serialize as a node"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn build_map_handles_missing_files() {
        // Build map against a tempdir with NO files — should produce an empty
        // map without panicking. (Cargo.toml is required, so we create just that.)
        let tmpdir = std::env::temp_dir().join(format!("arch-map-test-{}", std::process::id()));
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
            std::env::temp_dir().join(format!("arch-map-test-crates-{}", std::process::id()));
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
            std::env::temp_dir().join(format!("arch-map-test-emit-{}", std::process::id()));
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
