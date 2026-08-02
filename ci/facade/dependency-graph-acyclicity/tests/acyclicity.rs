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
    assert!(root.join(&policy.capability_registry_path).is_file());
    let raw = serde_json::from_str(
        &std::fs::read_to_string(root.join(&policy.dag_path)).expect("read live graph"),
    )
    .expect("parse live graph JSON");
    (raw, policy)
}

fn capability_registry() -> Value {
    let root = repo_root();
    let policy = load_policy(&root, DEFAULT_POLICY_PATH).expect("read live policy");
    serde_json::from_str(
        &std::fs::read_to_string(root.join(policy.capability_registry_path))
            .expect("read capability registry"),
    )
    .expect("parse capability registry")
}

fn live_schema() -> Value {
    let root = repo_root();
    let policy = load_policy(&root, DEFAULT_POLICY_PATH).expect("read live policy");
    serde_json::from_str(
        &std::fs::read_to_string(root.join(policy.schema_path)).expect("read live schema"),
    )
    .expect("parse live schema")
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
    report_with_schema(raw, &live_schema())
}

fn report_with_schema(raw: &Value, schema: &Value) -> Report {
    let dag = parse_dag(&serde_json::to_string(raw).expect("serialize graph"))
        .expect("structurally parse graph");
    evaluate_with_raw(&dag, raw, schema, &capability_registry())
}

fn graph_mut<'a>(raw: &'a mut Value, kind: &str) -> &'a mut Value {
    raw["graphs"]
        .as_array_mut()
        .expect("graphs")
        .iter_mut()
        .find(|graph| graph["kind"] == kind)
        .expect("graph kind")
}

fn replace_string(value: &mut Value, old: &str, new: &str) {
    match value {
        Value::Array(items) => {
            for item in items {
                replace_string(item, old, new);
            }
        }
        Value::Object(object) => {
            for item in object.values_mut() {
                replace_string(item, old, new);
            }
        }
        Value::String(current) if current == old => *current = new.to_owned(),
        _ => {}
    }
}

fn apply_fixture_mutation(raw: &mut Value, mutation: &str) {
    match mutation {
        "none" => {}
        "remove_graph_kind" => {
            raw["graphs"].as_array_mut().unwrap().pop();
        }
        "append_unknown_graph_kind" => raw["graphs"].as_array_mut().unwrap().push(json!({
            "kind": "sixth_graph", "edge_semantics": "invalid", "edges": []
        })),
        "duplicate_dependency_unit" => {
            let first = raw["dependency_units"][0].clone();
            raw["dependency_units"].as_array_mut().unwrap().push(first);
        }
        "remove_dependency_unit" => {
            raw["dependency_units"].as_array_mut().unwrap().pop();
        }
        "append_twentieth_dependency_unit" => {
            raw["dependency_units"].as_array_mut().unwrap().push(json!({
                "id": "network.fixture-twentieth",
                "capability": "network",
                "runtime_face": "fixture-twentieth",
                "plane": "B0",
                "purpose": "executable mutation fixture"
            }));
        }
        "unknown_capability" => {
            raw["dependency_units"][0]["capability"] = json!("not-a-capability");
        }
        "remove_runtime_face" => {
            raw["dependency_units"][0]
                .as_object_mut()
                .unwrap()
                .remove("runtime_face");
        }
        "unknown_runtime_face" => {
            raw["dependency_units"][0]["runtime_face"] = json!("unknown-bootstrap");
        }
        "id_face_mismatch" => {
            raw["dependency_units"][1]["runtime_face"] = json!("genesis");
        }
        "capability_id_mismatch" => {
            raw["dependency_units"][1]["capability"] = json!("network");
        }
        "consistent_unit_replacement" => {
            raw["dependency_units"][0]["id"] = json!("network.fabric-bootstrap");
            raw["dependency_units"][0]["runtime_face"] = json!("fabric-bootstrap");
            replace_string(raw, "network.bootstrap", "network.fabric-bootstrap");
        }
        "unknown_endpoint" => graph_mut(raw, "genesis")["edges"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "graph_kind": "genesis", "from": "unknown.face", "to": "cell.envelope"
            })),
        "cross_kind_contamination" => {
            graph_mut(raw, "genesis")["edges"][0]["graph_kind"] =
                json!("steady_state_request");
        }
        "graph3_self_loop" => graph_mut(raw, "steady_state_request")["edges"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "graph_kind": "steady_state_request", "from": "cell.envelope", "to": "cell.envelope",
                "dependency_weight": 1.0, "cascade_rule": "FULL",
                "version_compatibility_range": "^2.0", "cedar_permit_fragment": "fixture-self-loop"
            })),
        "graph3_cycle" => graph_mut(raw, "steady_state_request")["edges"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "graph_kind": "steady_state_request", "from": "cell.envelope", "to": "iam.local-verifier",
                "dependency_weight": 1.0, "cascade_rule": "FULL",
                "version_compatibility_range": "^2.0", "cedar_permit_fragment": "fixture-cycle"
            })),
        "remove_failure_impact_rule" => {
            graph_mut(raw, "failure_brownout_propagation")["edges"][0]
                .as_object_mut()
                .unwrap()
                .remove("impact_rule");
        }
        "remove_failure_closure_edge" => {
            graph_mut(raw, "failure_brownout_propagation")["edges"]
                .as_array_mut()
                .unwrap()
                .pop();
        }
        "change_failure_impact" => {
            graph_mut(raw, "failure_brownout_propagation")["edges"][0]["impact_rule"] =
                json!("INDEPENDENT");
        }
        "reverse_direction_outside_graph3" => graph_mut(raw, "genesis")["edges"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "graph_kind": "genesis", "from": "cell.genesis", "to": "network.bootstrap"
            })),
        "composition_sum" => {
            graph_mut(raw, "failure_brownout_propagation")["composition"] = json!("sum");
        }
        "forward_closure_direction" => {
            raw["failure_impact_composition"]["closure_direction"] =
                json!("forward_transitive_closure");
        }
        "remove_doctrine_adrs" => {
            raw.as_object_mut().unwrap().remove("doctrine_adrs");
        }
        "remove_path_rule" => {
            raw["failure_impact_composition"]
                .as_object_mut()
                .unwrap()
                .remove("path_rule");
        }
        "request_weight_wrong_type" => {
            graph_mut(raw, "steady_state_request")["edges"][0]["dependency_weight"] =
                json!("heavy");
        }
        "request_weight_out_of_range" => {
            graph_mut(raw, "steady_state_request")["edges"][0]["dependency_weight"] = json!(1.1);
        }
        "request_weight_zero" => {
            graph_mut(raw, "steady_state_request")["edges"][0]["dependency_weight"] = json!(0);
        }
        "remove_forbidden_reason" => {
            graph_mut(raw, "steady_state_request")["forbidden_edges_assertion"][0]
                .as_object_mut()
                .unwrap()
                .remove("reason");
        }
        other => panic!("unknown executable fixture mutation {other}"),
    }
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
    let root = repo_root();
    let schema: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(&policy.schema_path)).expect("read live schema"),
    )
    .expect("parse live schema JSON");
    let registry = capability_registry();
    assert_eq!(policy.gate_id, GATE_ID);
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(registry["closed"], true);
    assert_eq!(registry["registry_kind"], "capability");
    assert_eq!(registry["capabilities"].as_array().unwrap().len(), 24);
    assert_eq!(raw["version"], "2.0.0");
    assert_eq!(raw["schema"], "specs/substrate-dependency-dag.schema.json");
    assert_eq!(raw["dependency_units"].as_array().unwrap().len(), 19);
    assert!(
        raw["dependency_units"]
            .as_array()
            .unwrap()
            .iter()
            .all(|unit| unit.get("runtime_face").is_some_and(Value::is_string))
    );
    assert_eq!(raw["external_anchors"].as_array().unwrap().len(), 1);
    let kinds: Vec<&str> = raw["graphs"]
        .as_array()
        .expect("graphs")
        .iter()
        .map(|graph| graph["kind"].as_str().expect("kind"))
        .collect();
    assert_eq!(kinds, GRAPH_KINDS);
    let dag = parse_dag(&serde_json::to_string(&raw).expect("serialize graph"))
        .expect("structurally parse graph");
    let evaluated = evaluate_with_raw(&dag, &raw, &schema, &registry);
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

    let (mut reordered, _) = load_live();
    reordered["graphs"].as_array_mut().unwrap().swap(0, 1);
    assert_red_code(&reordered, "dag_graph_kind_set");
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
fn dependency_unit_count_capabilities_and_runtime_faces_are_closed() {
    let (mut eighteen, _) = load_live();
    eighteen["dependency_units"].as_array_mut().unwrap().pop();
    assert_red_code(&eighteen, "dag_dependency_unit_set");

    let (mut twenty, _) = load_live();
    twenty["dependency_units"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "id": "network.bootstrap",
            "capability": "network",
            "runtime_face": "bootstrap",
            "plane": "B0",
            "purpose": "fixture-only twentieth unit"
        }));
    assert_red_code(&twenty, "dag_dependency_unit_set");

    let (mut unknown_capability, _) = load_live();
    unknown_capability["dependency_units"][1]["capability"] = json!("not-a-capability");
    assert_red_code(&unknown_capability, "dag_unknown_capability");

    let (mut missing_runtime_face, _) = load_live();
    missing_runtime_face["dependency_units"][1]
        .as_object_mut()
        .unwrap()
        .remove("runtime_face");
    assert_red_code(&missing_runtime_face, "dag_schema_violation");

    let (mut unknown_face, _) = load_live();
    unknown_face["dependency_units"][0]["runtime_face"] = json!("unknown-bootstrap");
    assert_red_code(&unknown_face, "dag_dependency_unit_authority_mismatch");

    let (mut id_face_mismatch, _) = load_live();
    id_face_mismatch["dependency_units"][1]["runtime_face"] = json!("genesis");
    assert_red_code(&id_face_mismatch, "dag_dependency_unit_authority_mismatch");

    let (mut capability_id_mismatch, _) = load_live();
    capability_id_mismatch["dependency_units"][1]["capability"] = json!("network");
    assert_red_code(
        &capability_id_mismatch,
        "dag_dependency_unit_authority_mismatch",
    );

    let (mut replacement, _) = load_live();
    replacement["dependency_units"][0]["id"] = json!("network.fabric-bootstrap");
    replacement["dependency_units"][0]["runtime_face"] = json!("fabric-bootstrap");
    replace_string(
        &mut replacement,
        "network.bootstrap",
        "network.fabric-bootstrap",
    );
    assert_red_code(&replacement, "dag_dependency_unit_authority_mismatch");
}

#[test]
fn canonical_schema_authority_cannot_be_replaced_or_weakened() {
    let (raw, _) = load_live();
    let canonical = live_schema();
    let mut mutations = Vec::new();

    mutations.push((
        "rejecting replacement",
        json!({"$schema": "https://json-schema.org/draft/2020-12/schema", "not": {}}),
    ));

    let mut prefix_items = canonical.clone();
    prefix_items["properties"]["graphs"]
        .as_object_mut()
        .unwrap()
        .remove("prefixItems");
    mutations.push(("prefixItems", prefix_items));

    let mut items_false = canonical.clone();
    items_false["properties"]["graphs"]["items"] = json!(true);
    mutations.push(("items:false", items_false));

    let mut required = canonical.clone();
    required["required"].as_array_mut().unwrap().pop();
    mutations.push(("required", required));

    let mut additional_properties = canonical.clone();
    additional_properties["additionalProperties"] = json!(true);
    mutations.push(("additionalProperties", additional_properties));

    let mut const_keyword = canonical.clone();
    const_keyword["properties"]["version"]["const"] = json!("2.x");
    mutations.push(("const", const_keyword));

    let mut range = canonical.clone();
    range["$defs"]["request_edge"]["properties"]["dependency_weight"]["exclusiveMinimum"] =
        json!(-1);
    mutations.push(("range", range));

    let mut value_type = canonical;
    value_type["$defs"]["request_edge"]["properties"]["dependency_weight"]["type"] =
        json!("string");
    mutations.push(("type", value_type));

    for (name, schema) in mutations {
        let evaluated = report_with_schema(&raw, &schema);
        assert_eq!(evaluated.verdict, Verdict::Red, "schema mutation {name}");
        assert!(
            evaluated
                .findings
                .iter()
                .any(|finding| finding.code == "dag_schema_authority_mismatch"),
            "schema mutation {name}: {:?}",
            evaluated.findings
        );
    }
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
fn composition_direction_and_required_schema_fields_are_red() {
    let (mut sum, _) = load_live();
    graph_mut(&mut sum, "failure_brownout_propagation")["composition"] = json!("sum");
    assert_red_code(&sum, "dag_schema_violation");

    let (mut forward, _) = load_live();
    forward["failure_impact_composition"]["closure_direction"] =
        json!("forward_transitive_closure");
    assert_red_code(&forward, "dag_schema_violation");

    let (mut missing_doctrine, _) = load_live();
    missing_doctrine
        .as_object_mut()
        .unwrap()
        .remove("doctrine_adrs");
    assert_red_code(&missing_doctrine, "dag_schema_violation");

    let (mut stale_graph_adr, _) = load_live();
    stale_graph_adr["doctrine_adrs"][4] = json!("ADR-0631");
    assert_red_code(&stale_graph_adr, "dag_schema_violation");

    let (mut missing_path_rule, _) = load_live();
    missing_path_rule["failure_impact_composition"]
        .as_object_mut()
        .unwrap()
        .remove("path_rule");
    assert_red_code(&missing_path_rule, "dag_schema_violation");

    let (mut extra_property, _) = load_live();
    extra_property["schema_escape_hatch"] = json!(true);
    assert_red_code(&extra_property, "dag_schema_violation");

    let (mut wrong_const, _) = load_live();
    wrong_const["version"] = json!("2.x");
    assert_red_code(&wrong_const, "dag_schema_violation");
}

#[test]
fn request_metadata_types_ranges_and_forbidden_assertions_are_red() {
    for invalid_weight in [json!("heavy"), json!(-0.1), json!(0), json!(1.1)] {
        let (mut raw, _) = load_live();
        graph_mut(&mut raw, "steady_state_request")["edges"][0]["dependency_weight"] =
            invalid_weight;
        assert_red_code(&raw, "dag_edge_malformed");
    }

    let (mut numeric_version, _) = load_live();
    graph_mut(&mut numeric_version, "steady_state_request")["edges"][0]["version_compatibility_range"] =
        json!(2);
    assert_red_code(&numeric_version, "dag_edge_malformed");

    let (mut empty_cedar, _) = load_live();
    graph_mut(&mut empty_cedar, "steady_state_request")["edges"][0]["cedar_permit_fragment"] =
        json!("");
    assert_red_code(&empty_cedar, "dag_edge_malformed");

    let (mut missing_reason, _) = load_live();
    graph_mut(&mut missing_reason, "steady_state_request")["forbidden_edges_assertion"][0]
        .as_object_mut()
        .unwrap()
        .remove("reason");
    assert_red_code(&missing_reason, "dag_schema_violation");

    let (mut numeric_reason, _) = load_live();
    graph_mut(&mut numeric_reason, "steady_state_request")["forbidden_edges_assertion"][0]["reason"] =
        json!(7);
    assert_red_code(&numeric_reason, "dag_schema_violation");
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
fn fixture_corpus_executes_every_declared_mutation_and_expected_verdict() {
    let cases = fixture_cases();
    let cases = cases["cases"].as_array().expect("fixture cases");
    assert!(!cases.is_empty(), "fixture corpus must not be decorative");
    for case in cases {
        let id = case["id"].as_str().expect("fixture id");
        let mutation = case["mutation"].as_str().expect("fixture mutation");
        let expected = case["expected"].as_str().expect("fixture expected");
        let (mut raw, _) = load_live();
        apply_fixture_mutation(&mut raw, mutation);
        let evaluated = report(&raw);
        if expected == "GREEN" {
            assert_eq!(evaluated.verdict, Verdict::Green, "fixture {id}");
        } else {
            assert_eq!(evaluated.verdict, Verdict::Red, "fixture {id}");
            assert!(
                evaluated
                    .findings
                    .iter()
                    .any(|finding| finding.code == expected),
                "fixture {id}: expected {expected}, got {:?}",
                evaluated.findings
            );
        }
    }
}
