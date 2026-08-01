#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ci_product_protocol_policy::{GATE_ID, evaluate_keyed};
use serde_json::Value;

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current dir");
    loop {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        assert!(dir.pop(), "repo root marker not found");
    }
}

fn json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn policy(root: &Path) -> Value {
    json(&root.join("ci/facade/product-protocol-policy/product-protocol-policy.json"))
}

fn artifacts(root: &Path, policy: &Value) -> BTreeMap<String, Value> {
    policy["artifacts"]
        .as_object()
        .expect("artifacts object")
        .iter()
        .map(|(name, path)| {
            let rel = path.as_str().expect("artifact path string");
            let resolved = if name == "transport_profile" {
                std::env::var_os("OYA_ENDPOINT_TRANSPORT_PROFILE")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| root.join(rel))
            } else {
                root.join(rel)
            };
            (name.clone(), json(&resolved))
        })
        .collect()
}

fn replace_pointer(document: &mut Value, pointer: &str, replacement: Value) {
    let target = document
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("fixture pointer must resolve: {pointer}"));
    *target = replacement;
}

#[test]
fn live_product_protocol_contract_is_green() {
    let root = repo_root();
    let policy = policy(&root);
    assert_eq!(policy["gate_id"], GATE_ID);
    let findings = evaluate_keyed(&policy, &artifacts(&root, &policy));
    assert!(findings.is_empty(), "live contract findings: {findings:#?}");
}

#[test]
fn negative_fixture_corpus_fails_on_each_guarded_invariant() {
    let root = repo_root();
    let policy = policy(&root);
    let baseline = artifacts(&root, &policy);
    let fixtures = json(&root.join(
        "ci/facade/product-protocol-policy/fixtures/negative-cases.json",
    ));

    let cases = fixtures["cases"].as_array().expect("cases array");
    assert!(cases.len() >= 9, "negative fixture corpus unexpectedly small");
    for case in cases {
        let name = case["name"].as_str().expect("case name");
        let artifact = case["artifact"].as_str().expect("artifact name");
        let pointer = case["pointer"].as_str().expect("JSON pointer");
        let expected_code = case["expected_code"].as_str().expect("expected code");
        let mut observed = baseline.clone();
        replace_pointer(
            observed.get_mut(artifact).expect("known artifact"),
            pointer,
            case["replacement"].clone(),
        );
        let findings = evaluate_keyed(&policy, &observed);
        assert!(
            findings.iter().any(|finding| finding.code == expected_code),
            "negative fixture {name} did not emit {expected_code}: {findings:#?}"
        );
    }
}

#[test]
fn malformed_or_empty_policy_fails_closed() {
    let findings = evaluate_keyed(&serde_json::json!({}), &BTreeMap::new());
    assert!(!findings.is_empty(), "empty policy must not certify an empty universe");
}
