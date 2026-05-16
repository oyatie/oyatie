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

fn kinetic_graph() -> Value {
    let path = repo_root().join("registry/knowledge-graph-kinetic.json");
    serde_json::from_str(&fs::read_to_string(path).expect("kinetic KG registry is readable"))
        .expect("kinetic KG registry is JSON")
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

#[test]
fn incident_closeout_root_cause_pr_chain_is_graph_traversable() {
    let graph = semantic_graph();
    let node_types = graph["node_types"]
        .as_object()
        .expect("node_types is an object");
    for node_type in ["Incident", "CausedByChange", "PullRequest"] {
        assert!(
            node_types.contains_key(node_type),
            "missing node_type {node_type}"
        );
    }

    assert_edge_allows(&graph, "caused_by_change", "Incident", "CausedByChange");
    assert_edge_allows(&graph, "change_from_pr", "CausedByChange", "PullRequest");

    let queries = graph["read_side_query_examples"]
        .as_array()
        .expect("queries are an array");
    let root_cause_query = queries
        .iter()
        .find(|query| query["name"] == "trace_incident_root_cause_prs")
        .expect("incident root-cause query exists");
    let query_sketch = root_cause_query["query_sketch"]
        .as_str()
        .expect("query sketch is a string");
    assert!(query_sketch.contains("caused_by_change_edges"));
    assert!(query_sketch.contains("change_from_pr_edges"));
    assert!(query_sketch.contains("pull_request_id"));
}

#[test]
fn close_incident_action_emits_root_cause_pr_chain() {
    let graph = kinetic_graph();
    let close_incident = &graph["action_types"]["CloseIncident"];
    assert_eq!(close_incident["input_node_type"], "Incident");
    assert_eq!(close_incident["emits_audit_topic"], "oya.incident.closed");

    let required_fields = close_incident["required_fields"]
        .as_array()
        .expect("required_fields is an array");
    for field in [
        "incident_id",
        "root_cause_change_id",
        "pull_request_id",
        "evidence_ref",
        "closed_at",
    ] {
        assert!(
            required_fields.iter().any(|required| required == field),
            "CloseIncident missing required field {field}"
        );
    }

    let invariants = close_incident["validator_invariants_to_recheck"]
        .as_array()
        .expect("validator invariants are an array");
    assert!(
        invariants
            .iter()
            .any(|invariant| invariant == "I16-incident-closeout-has-root-cause-pr-chain")
    );

    let workflow_actions = graph["workflows"]["incident_closeout"]["actions_in_order"]
        .as_array()
        .expect("incident_closeout actions are an array");
    assert!(workflow_actions.iter().any(|action| {
        action
            .as_str()
            .is_some_and(|text| text.contains("CloseIncident"))
    }));
}

#[test]
fn blast_radius_query_maps_diff_to_reachable_entities_and_review_signals() {
    let graph = semantic_graph();
    let queries = graph["read_side_query_examples"]
        .as_array()
        .expect("queries are an array");
    let blast_radius_query = queries
        .iter()
        .find(|query| query["name"] == "reachable_entity_types_for_diff")
        .expect("blast-radius query exists");
    let query_sketch = blast_radius_query["query_sketch"]
        .as_str()
        .expect("query sketch is a string");

    for required_term in [
        "WITH RECURSIVE",
        "candidate_diff_files",
        "reachable_entity_types",
        "downstream_consumers",
        "blast_radius_score",
        "public_api_touched",
        "kernel_layer_touched",
        "secret_or_admission_touched",
        "audit_chain_schema_touched",
        "ADR_introduced_or_amended_count",
        "cargo_deps_changed_count",
        "lines_touched",
        "new_files_count",
        "new_crates_count",
        "breaking_change_marker",
        "meta_review_triggered",
    ] {
        assert!(
            query_sketch.contains(required_term),
            "blast-radius query missing {required_term}"
        );
    }
}

#[test]
fn agent_decision_provenance_chain_is_graph_traversable() {
    let graph = semantic_graph();
    let node_types = graph["node_types"]
        .as_object()
        .expect("node_types is an object");
    assert!(node_types.contains_key("AgentDecision"));

    assert_edge_allows(&graph, "caused_by", "AgentDecision", "AgentDecision");

    let invariants = graph["invariants"]
        .as_array()
        .expect("invariants are an array");
    assert!(invariants.iter().any(|invariant| {
        invariant["id"] == "I17-agent-decision-chain-is-traversable"
            && invariant["rule"]
                .as_str()
                .is_some_and(|rule| rule.contains("caused_by edges"))
    }));

    let queries = graph["read_side_query_examples"]
        .as_array()
        .expect("queries are an array");
    let decision_query = queries
        .iter()
        .find(|query| query["name"] == "trace_agent_decision_chain")
        .expect("agent decision chain query exists");
    let query_sketch = decision_query["query_sketch"]
        .as_str()
        .expect("query sketch is a string");
    for required_term in [
        "WITH RECURSIVE",
        "decision_chain",
        "agent_decisions",
        "caused_by_edges",
        "evidence_ref",
    ] {
        assert!(
            query_sketch.contains(required_term),
            "agent decision query missing {required_term}"
        );
    }
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
