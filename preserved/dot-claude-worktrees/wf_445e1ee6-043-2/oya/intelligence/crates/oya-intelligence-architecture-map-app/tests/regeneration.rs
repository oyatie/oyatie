//! Merge authority for `registry/graph/architecture-map.json`: the committed
//! face must byte-equal what the producer derives from the live workspace.
//!
//! This is the check that was missing while the face had no runnable producer:
//! the file was maintained by hand and by codemods, and drifted until 19 of its
//! 453 ids pointed at `microservices/`, a root that no longer exists.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use intelligence_architecture_map_kernel::NodeKind;
use oya_intelligence_architecture_map_app::{
    DEFAULT_OUTPUT, RunOutcome, build_map, render_json, run,
};

fn repo_root() -> PathBuf {
    std::env::current_dir()
        .expect("current dir readable")
        .ancestors()
        .find(|dir| dir.join(DEFAULT_OUTPUT).exists() && dir.join("Cargo.toml").exists())
        .expect("could not locate Oyatie repo root from current dir")
        .to_path_buf()
}

#[test]
fn committed_architecture_map_matches_the_producer() {
    let root = repo_root();
    let outcome = run(&root, &root.join(DEFAULT_OUTPUT), true).expect("producer runs");
    assert_eq!(
        outcome,
        RunOutcome::Clean,
        "{DEFAULT_OUTPUT} drifted from the derived map; regenerate it with \
         `buck2 run //oya/intelligence/crates/oya-intelligence-architecture-map-app:oya-intelligence-architecture-map -- --write`"
    );
}

#[test]
fn regeneration_is_byte_deterministic() {
    let root = repo_root();
    let first = render_json(&build_map(&root).expect("build"));
    let second = render_json(&build_map(&root).expect("build"));
    assert_eq!(first, second, "map rendering must be deterministic");
}

/// Corpus floor: a collector that silently resolves nothing must not read as
/// clean. The live workspace has hundreds of member crates and several
/// registry-derived nodes.
#[test]
fn derived_map_meets_corpus_floor() {
    let map = build_map(&repo_root()).expect("build");
    assert!(
        map.node_count() >= 300,
        "architecture-map collector under-collected: {} nodes (<300 floor)",
        map.node_count()
    );
    assert!(
        map.edge_count() >= 5,
        "architecture-map collector produced {} edges (<5 floor)",
        map.edge_count()
    );
}

/// Crate and contract node ids ARE repo-relative paths, so every one must
/// exist on disk. This is the invariant the committed face violated for its 19
/// `microservices/...` ids. (Microservice / bounded-context / cedar-fragment
/// ids are registry keys, not paths, and are excluded by kind.)
#[test]
fn path_shaped_node_ids_exist_on_disk() {
    let root = repo_root();
    let map = build_map(&root).expect("build");
    let dangling: Vec<&str> = map
        .nodes_of_kind(NodeKind::Crate)
        .chain(map.nodes_of_kind(NodeKind::OpenApiContract))
        .map(|node| node.id.0.as_str())
        .filter(|id| !Path::new(&root).join(id).exists())
        .collect();
    assert!(
        dangling.is_empty(),
        "node ids point at missing paths: {dangling:?}"
    );
}
