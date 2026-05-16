// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// repository invariants for the semantic knowledge-graph registry.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate has workspace parent")
        .parent()
        .expect("workspace has repo root")
        .to_path_buf()
}

fn semantic_graph() -> Value {
    let path = repo_root().join("registry/knowledge-graph-semantic.json");
    serde_json::from_str(&fs::read_to_string(path).expect("semantic KG registry is readable"))
        .expect("semantic KG registry is JSON")
}

#[test]
fn compliance_control_evidence_freshness_chain_is_graph_traversable() {
    let graph = semantic_graph();
    let node_types = graph["node_types"]
        .as_object()
        .expect("node_types is an object");
    for node_type in ["ComplianceControl", "SatisfiedByLane", "EvidencePath"] {
        assert!(
            node_types.contains_key(node_type),
            "missing node_type {node_type}"
        );
    }

    assert_edge_allows(
        &graph,
        "satisfied_by_lane",
        "ComplianceControl",
        "SatisfiedByLane",
    );
    assert_edge_allows(&graph, "evidence_path", "SatisfiedByLane", "EvidencePath");

    let queries = graph["read_side_query_examples"]
        .as_array()
        .expect("queries are an array");
    let freshness_query = queries
        .iter()
        .find(|query| query["name"] == "show_compliance_evidence_freshness")
        .expect("freshness query exists");
    let query_sketch = freshness_query["query_sketch"]
        .as_str()
        .expect("query sketch is a string");
    assert!(query_sketch.contains("satisfied_by_lane_edges"));
    assert!(query_sketch.contains("evidence_path_edges"));
    assert!(query_sketch.contains("freshness"));
}

fn assert_edge_allows(graph: &Value, edge: &str, source_type: &str, target_type: &str) {
    let edge_type = &graph["edge_types"][edge];
    assert_eq!(edge_type["direction"], "directed");
    let sources = edge_type["source_node_types"]
        .as_array()
        .expect("source_node_types is array");
    let targets = edge_type["target_node_types"]
        .as_array()
        .expect("target_node_types is array");
    assert!(
        sources.iter().any(|node| node == source_type),
        "{edge} does not allow source {source_type}"
    );
    assert!(
        targets.iter().any(|node| node == target_type),
        "{edge} does not allow target {target_type}"
    );
}
