#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use ci_dependency_graph_acyclicity::{
    DEFAULT_POLICY_PATH, GATE_ID, Policy, Report, Verdict, evaluate_with_raw, load_policy,
    parse_dag,
};
use serde_json::{Value, json};

const GRAPH_KINDS: [&str; 5] = [
    "genesis",
    "new_cell_provisioning",
    "steady_state_request",
    "control_data_publication",
    "failure_brownout_propagation",
];

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(DEFAULT_POLICY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn load_live() -> (Value, Policy) {
    let root = repo_root();
    let policy = load_policy(&root, DEFAULT_POLICY_PATH).expect("read live policy");
    assert!(root.join(&policy.dag_path).is_file());
    assert!(root.join(&policy.schema_path).is_file());
    let raw = serde_json::from_str(
        &std::fs::read_to_string(root.join(&policy.dag_path)).expect("read live graph"),
    )
    .expect("parse live graph JSON");
    (raw, policy)
}

fn fixture_cases() -> Value {
    let root = repo_root();
    serde_json::from_str(
        &std::fs::read_to_string(
            root.join("ci/facade/dependency-graph-acyclicity/tests/fixtures/graph-v2-cases.json"),
        )
        .expect("read graph-v2 fixture corpus"),
    )
    .expect("parse graph-v2 fixture corpus")
}

fn report(raw: &Value) -> Report {
    let dag = parse_dag(&serde_json::to_string(raw).expect("serialize graph"))
        .expect("structurally parse graph");
    evaluate_with_raw(&dag, raw)
}

fn graph_mut<'a>(raw: &'a mut Value, kind: &str) -> &'a mut Value {
    raw["graphs"]
        .as_array_mut()
        .expect("graphs")
        .iter_mut()
        .find(|graph| graph["kind"] == kind)
        .expect("graph kind")
}

fn assert_red_code(raw: &Value, code: &str) {
    let evaluated = report(raw);
    assert_eq!(evaluated.verdict, Verdict::Red, "expected RED: {code}");
    assert!(
        evaluated
            .findings
            .iter()
            .any(|finding| finding.code == code),
        "expected finding {code}, got {:?}",
        evaluated.findings
    );
}

#[test]
fn live_policy_and_graph_v2_are_green() {
    let (raw, policy) = load_live();
    assert_eq!(policy.gate_id, GATE_ID);
    assert_eq!(raw["version"], "2.0.0");
    assert_eq!(raw["schema"], "specs/substrate-dependency-dag.schema.json");
    let kinds: Vec<&str> = raw["graphs"]
        .as_array()
        .expect("graphs")
        .iter()
        .map(|graph| graph["kind"].as_str().expect("kind"))
        .collect();
    assert_eq!(kinds, GRAPH_KINDS);
    let evaluated = report(&raw);
    assert_eq!(
        evaluated.verdict,
        Verdict::Green,
        "{:?}",
        evaluated.findings
    );
}

#[test]
fn missing_and_extra_graph_kinds_are_red() {
    let (mut missing, _) = load_live();
    missing["graphs"].as_array_mut().unwrap().pop();
    assert_red_code(&missing, "dag_graph_kind_set");

    let (mut extra, _) = load_live();
    extra["graphs"].as_array_mut().unwrap().push(json!({
        "kind": "sixth_graph",
        "edge_semantics": "invalid",
        "edges": []
    }));
    assert_red_code(&extra, "dag_graph_kind_set");
}

#[test]
fn duplicate_unknown_and_cross_kind_edges_are_red() {
    let (mut duplicate, _) = load_live();
    let first = duplicate["dependency_units"][0].clone();
    duplicate["dependency_units"]
        .as_array_mut()
        .unwrap()
        .push(first);
    assert_red_code(&duplicate, "dag_duplicate_unit");

    let (mut unknown, _) = load_live();
    graph_mut(&mut unknown, "genesis")["edges"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "graph_kind": "genesis", "from": "unknown.face", "to": "cell.envelope"
        }));
    assert_red_code(&unknown, "dag_edge_unknown_unit");

    let (mut contaminated, _) = load_live();
    graph_mut(&mut contaminated, "genesis")["edges"][0]["graph_kind"] =
        json!("steady_state_request");
    assert_red_code(&contaminated, "dag_cross_kind_edge");
}

#[test]
fn only_steady_state_request_must_be_acyclic() {
    let (mut self_loop, _) = load_live();
    graph_mut(&mut self_loop, "steady_state_request")["edges"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "graph_kind": "steady_state_request",
            "from": "cell.envelope",
            "to": "cell.envelope",
            "dependency_weight": 1.0,
            "cascade_rule": "FULL",
            "version_compatibility_range": "^2.0",
            "cedar_permit_fragment": "fixture-self-loop"
        }));
    assert_red_code(&self_loop, "dag_cycle");

    let (mut cycle, _) = load_live();
    graph_mut(&mut cycle, "steady_state_request")["edges"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "graph_kind": "steady_state_request",
            "from": "cell.envelope",
            "to": "iam.local-verifier",
            "dependency_weight": 1.0,
            "cascade_rule": "FULL",
            "version_compatibility_range": "^2.0",
            "cedar_permit_fragment": "fixture-cycle"
        }));
    assert_red_code(&cycle, "dag_cycle");

    let (mut reverse_outside_graph3, _) = load_live();
    graph_mut(&mut reverse_outside_graph3, "genesis")["edges"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "graph_kind": "genesis", "from": "cell.genesis", "to": "external.genesis-roots"
        }));
    assert_eq!(
        report(&reverse_outside_graph3).verdict,
        Verdict::Green,
        "cycles/reverse directions outside graph 3 are allowed"
    );
}

#[test]
fn malformed_or_mismatched_failure_closure_is_red() {
    let (mut malformed, _) = load_live();
    graph_mut(&mut malformed, "failure_brownout_propagation")["edges"][0]
        .as_object_mut()
        .unwrap()
        .remove("impact_rule");
    assert_red_code(&malformed, "dag_failure_edge_malformed");

    let (mut missing, _) = load_live();
    graph_mut(&mut missing, "failure_brownout_propagation")["edges"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert_red_code(&missing, "dag_failure_closure_mismatch");

    let (mut wrong_impact, _) = load_live();
    graph_mut(&mut wrong_impact, "failure_brownout_propagation")["edges"][0]["impact_rule"] =
        json!("INDEPENDENT");
    assert_red_code(&wrong_impact, "dag_failure_closure_mismatch");
}

#[test]
fn failure_graph_is_the_exact_reverse_transitive_closure() {
    let (raw, _) = load_live();
    let evaluated = report(&raw);
    assert!(
        !evaluated
            .findings
            .iter()
            .any(|finding| finding.code == "dag_failure_closure_mismatch"),
        "failure graph must equal the computed max-min reverse closure: {:?}",
        evaluated.findings
    );
}

#[test]
fn fixture_corpus_covers_every_required_red_and_green_class() {
    let cases = fixture_cases();
    let actual: Vec<&str> = cases["cases"]
        .as_array()
        .expect("fixture cases")
        .iter()
        .map(|case| case["id"].as_str().expect("fixture id"))
        .collect();
    assert_eq!(
        actual,
        [
            "red-missing-kind",
            "red-extra-kind",
            "red-duplicate-unit",
            "red-unknown-endpoint",
            "red-cross-kind-contamination",
            "red-graph3-self-loop",
            "red-graph3-cycle",
            "red-malformed-failure-edge",
            "red-missing-failure-closure-edge",
            "red-mismatched-failure-impact",
            "green-reverse-direction-outside-graph3",
            "green-exact-failure-closure",
        ]
    );
}
