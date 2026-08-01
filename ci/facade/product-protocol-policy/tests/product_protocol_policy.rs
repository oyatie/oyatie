#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ci_product_protocol_policy::{GATE_ID, evaluate_keyed};
use serde_json::Value;

fn json(path: &Path) -> Value {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn declared_path(variable: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("Buck must declare {variable} with $(location)"))
}

fn policy() -> Value {
    json(&declared_path("OYA_PRODUCT_PROTOCOL_POLICY"))
}

fn artifacts() -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "product_contract".to_owned(),
            json(&declared_path("OYA_PRODUCT_PROTOCOL_CONTRACT")),
        ),
        (
            "api_contract_ssot".to_owned(),
            json(&declared_path("OYA_API_CONTRACT_SSOT")),
        ),
        (
            "transport_profile".to_owned(),
            json(&declared_path("OYA_ENDPOINT_TRANSPORT_PROFILE")),
        ),
        (
            "root_hub".to_owned(),
            json(&declared_path("OYA_ROOT_HUB_POINTERS")),
        ),
    ])
}

fn replace_pointer(document: &mut Value, pointer: &str, replacement: Value) {
    let target = document
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("fixture pointer must resolve: {pointer}"));
    *target = replacement;
}

#[test]
fn live_product_protocol_contract_is_green() {
    let policy = policy();
    assert_eq!(policy["gate_id"], GATE_ID);
    assert_eq!(
        policy["artifacts"],
        serde_json::json!({
            "product_contract": "specs/product-protocol-contract.json",
            "api_contract_ssot": "specs/api-contract-ssot-canonical.json",
            "transport_profile": "network/ports/transport-profile/endpoint-transport-profile.contract.json",
            "root_hub": "specs/root-hub-pointers.json"
        }),
        "policy artifact identities must stay bound to the Buck-declared location inputs"
    );
    let findings = evaluate_keyed(&policy, &artifacts());
    assert!(findings.is_empty(), "live contract findings: {findings:#?}");
}

#[test]
fn named_transport_class_rules_are_order_independent() {
    let policy = policy();
    let mut observed = artifacts();
    observed
        .get_mut("transport_profile")
        .expect("transport profile")
        .get_mut("capability_classes")
        .and_then(Value::as_array_mut)
        .expect("capability classes")
        .reverse();

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "transport class ordering changed the verdict: {findings:#?}"
    );
}

#[test]
fn negative_fixture_corpus_fails_on_each_guarded_invariant() {
    let policy = policy();
    let baseline = artifacts();
    let fixtures = json(&declared_path("OYA_PRODUCT_PROTOCOL_NEGATIVE_CASES"));

    let cases = fixtures["cases"].as_array().expect("cases array");
    assert!(
        cases.len() >= 9,
        "negative fixture corpus unexpectedly small"
    );
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
    assert!(
        !findings.is_empty(),
        "empty policy must not certify an empty universe"
    );
}
