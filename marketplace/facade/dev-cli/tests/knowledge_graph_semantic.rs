// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// repository invariants for the semantic knowledge-graph registry.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("specs/masterplan.json").is_file()
                && candidate.join("HANDOFF.md").is_file()
        })
        .expect("repo root")
        .to_path_buf()
}

fn semantic_graph() -> Value {
    // Migrated from registry/knowledge-graph-semantic.json per ADR-0130.
    // Type system now lives in specs/microservices/ontology.json#type_system.
    let path = repo_root().join("specs/microservices/ontology.json");
    let ontology: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("ontology spec is readable"))
            .expect("ontology spec is JSON");
    ontology["type_system"]
        .as_object()
        .expect("type_system section exists in ontology.json")
        .clone()
        .into()
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

#[test]
fn acceptance_criterion_test_ci_lane_chain_is_graph_traversable() {
    let graph = semantic_graph();
    let node_types = graph["node_types"]
        .as_object()
        .expect("node_types is an object");
    for node_type in ["AcceptanceCriterion", "Test", "CILaneRun"] {
        assert!(
            node_types.contains_key(node_type),
            "missing node_type {node_type}"
        );
    }

    assert_edge_allows(&graph, "covered_by_test", "AcceptanceCriterion", "Test");
    assert_edge_allows(&graph, "executed_by_ci_lane", "Test", "CILaneRun");

    let invariants = graph["invariants"]
        .as_array()
        .expect("invariants are an array");
    assert!(invariants.iter().any(|invariant| {
        invariant["id"] == "I18-acceptance-criterion-has-test-ci-chain"
            && invariant["rule"]
                .as_str()
                .is_some_and(|rule| rule.contains("covered_by_test"))
    }));

    let queries = graph["read_side_query_examples"]
        .as_array()
        .expect("queries are an array");
    let trace_query = queries
        .iter()
        .find(|query| query["name"] == "trace_acceptance_test_ci_chain")
        .expect("acceptance/test/CI trace query exists");
    let query_sketch = trace_query["query_sketch"]
        .as_str()
        .expect("query sketch is a string");
    for required_term in [
        "acceptance_criterion_id",
        "covered_by_test_edges",
        "executed_by_ci_lane_edges",
        "ci_lane_run_id",
    ] {
        assert!(
            query_sketch.contains(required_term),
            "acceptance/test/CI query missing {required_term}"
        );
    }
}

#[test]
fn changeset_pr_audit_row_provenance_chain_is_graph_traversable() {
    let graph = semantic_graph();
    let node_types = graph["node_types"]
        .as_object()
        .expect("node_types is an object");
    for node_type in ["ChangeSet", "PullRequest", "AuditChainRow"] {
        assert!(
            node_types.contains_key(node_type),
            "missing node_type {node_type}"
        );
    }

    assert_edge_allows(&graph, "merged_as_pr", "ChangeSet", "PullRequest");
    assert_edge_allows(&graph, "emitted_audit_row", "PullRequest", "AuditChainRow");

    let invariants = graph["invariants"]
        .as_array()
        .expect("invariants are an array");
    assert!(invariants.iter().any(|invariant| {
        invariant["id"] == "I19-changeset-pr-audit-row-provenance"
            && invariant["rule"]
                .as_str()
                .is_some_and(|rule| rule.contains("emitted_audit_row"))
    }));

    let queries = graph["read_side_query_examples"]
        .as_array()
        .expect("queries are an array");
    let trace_query = queries
        .iter()
        .find(|query| query["name"] == "trace_changeset_pr_audit_row")
        .expect("changeset/PR/audit-row trace query exists");
    let query_sketch = trace_query["query_sketch"]
        .as_str()
        .expect("query sketch is a string");
    for required_term in [
        "changeset_id",
        "merged_as_pr_edges",
        "pull_request_id",
        "emitted_audit_row_edges",
        "audit_chain_row_id",
    ] {
        assert!(
            query_sketch.contains(required_term),
            "changeset/PR/audit-row query missing {required_term}"
        );
    }
}

#[test]
fn ontology_product_edges_have_declared_sources_and_targets() {
    let graph = semantic_graph();
    let product_nodes = graph["product_nodes"]
        .as_array()
        .expect("product_nodes are an array");
    let product_node_ids: std::collections::BTreeSet<&str> = product_nodes
        .iter()
        .map(|node| node["id"].as_str().expect("product node id is a string"))
        .collect();
    for required_node in [
        "community",
        "messenger",
        "mail",
        "workflow",
        "ontology",
        "foundry",
    ] {
        assert!(
            product_node_ids.contains(required_node),
            "missing product node {required_node}"
        );
    }

    for edge_type in [
        "child_of",
        "ecosystem_integration",
        "dual_context_isolation",
    ] {
        assert!(
            graph["edge_types"].get(edge_type).is_some(),
            "missing edge_type {edge_type}"
        );
    }

    let product_edges = graph["product_edges"]
        .as_array()
        .expect("product_edges are an array");
    assert!(!product_edges.is_empty());
    for edge in product_edges {
        let source = edge["source"].as_str().expect("edge source is a string");
        assert!(
            product_node_ids.contains(source),
            "edge source {source} is dangling"
        );
        let target = edge["target"].as_str().expect("edge target is a string");
        assert!(
            product_node_ids.contains(target),
            "edge target {target} is dangling"
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
