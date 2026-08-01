#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use ci_product_protocol_policy::{GATE_ID, evaluate_keyed};
use serde_json::Value;

fn json(path: &Path) -> Value {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn text(variable: &str) -> String {
    let path = declared_path(variable);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
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
        (
            "manifest_schema".to_owned(),
            json(&declared_path("OYA_MICROSERVICE_MANIFEST_SCHEMA")),
        ),
        (
            "master_plan_sequencing".to_owned(),
            json(&declared_path("OYA_MASTER_PLAN_SEQUENCING")),
        ),
    ])
}

fn section<'a>(document: &'a str, heading: &str) -> &'a str {
    let body = document
        .split_once(heading)
        .unwrap_or_else(|| panic!("missing authority section {heading}"))
        .1;
    body.split("\n## ").next().expect("section body")
}

fn frontmatter(document: &str) -> &str {
    document
        .strip_prefix("---\n")
        .and_then(|body| body.split_once("\n---\n"))
        .map(|(frontmatter, _)| frontmatter)
        .expect("ADR must carry YAML frontmatter")
}

fn assert_public_protocol_reconciliation(adr_id: &str, document: &str, heading: &str) {
    let frontmatter = frontmatter(document);
    assert!(
        frontmatter.contains(&format!("id: {adr_id}")),
        "{adr_id} identity drifted"
    );
    assert!(
        frontmatter.contains("status: Accepted"),
        "{adr_id} must remain Accepted"
    );
    assert!(
        frontmatter.contains("ADR-0632"),
        "{adr_id} must relate to ADR-0632"
    );

    let reconciliation = section(document, heading)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    for required in [
        "openapi 3.2.0",
        "signed/versioned webhook",
        "asyncapi/cloudevents",
        "sse",
        "websocket",
        "graphql",
        "grpc-web",
        "connect",
        "internal-only",
        "grpc/proto3",
        "http/2",
    ] {
        assert!(
            reconciliation.contains(required),
            "{adr_id} reconciliation must cover {required}"
        );
    }

    let normalized_document = document
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    for contradiction in [
        "public rest/grpc",
        "public http/grpc",
        "external http/grpc",
        "every public µservice rpc (http + grpc",
        "proto3 services exposed externally",
        "proto3 reserved field oyatie_version",
    ] {
        assert!(
            !normalized_document.contains(contradiction),
            "{adr_id} reintroduced the public RPC contradiction {contradiction}"
        );
    }
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
            "root_hub": "specs/root-hub-pointers.json",
            "manifest_schema": "specs/microservices/manifest-schema.json",
            "master_plan_sequencing": "specs/master-plan-sequencing.json"
        }),
        "policy artifact identities must stay bound to the Buck-declared location inputs"
    );
    let findings = evaluate_keyed(&policy, &artifacts());
    assert!(findings.is_empty(), "live contract findings: {findings:#?}");
}

#[test]
fn live_adr_authority_reconciliation_is_green() {
    let policy = policy();
    let heading = policy["authority_reconciliation"]["section_heading"]
        .as_str()
        .expect("accepted ADR section heading");
    let accepted = [
        ("ADR-0157", "OYA_ADR_0157"),
        ("ADR-0167", "OYA_ADR_0167"),
        ("ADR-0176", "OYA_ADR_0176"),
        ("ADR-0182", "OYA_ADR_0182"),
        ("ADR-0258", "OYA_ADR_0258"),
    ];
    let declared = policy["authority_reconciliation"]["accepted_adrs"]
        .as_array()
        .expect("accepted ADR inventory")
        .iter()
        .map(|row| row["id"].as_str().expect("ADR id"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        declared,
        accepted.iter().map(|(id, _)| *id).collect(),
        "policy inventory must cover the complete W0-B Accepted ADR correction map"
    );
    for (adr_id, variable) in accepted {
        assert_public_protocol_reconciliation(adr_id, &text(variable), heading);
    }

    let proposed = text("OYA_ADR_0246");
    let proposed_frontmatter = frontmatter(&proposed);
    assert!(proposed_frontmatter.contains("id: ADR-0246"));
    assert!(proposed_frontmatter.contains("status: Proposed"));
    assert!(proposed_frontmatter.contains("ADR-0632"));
    let proposed_heading = policy["authority_reconciliation"]["proposed_section_heading"]
        .as_str()
        .expect("proposed ADR section heading");
    let clarification = section(&proposed, proposed_heading).to_ascii_lowercase();
    for required in [
        "remains **proposed**",
        "does not accept",
        "if accepted",
        "internal grpc/proto3 over http/2",
        "public and compatibility surface",
        "public grpc",
    ] {
        assert!(
            clarification.contains(required),
            "ADR-0246 clarification must preserve proposal semantics for {required}"
        );
    }
}

#[test]
fn manifest_schema_keeps_public_contracts_closed_and_grpc_internal() {
    let schema = json(&declared_path("OYA_MICROSERVICE_MANIFEST_SCHEMA"));
    let contract_properties = schema
        .pointer("/properties/contracts/properties")
        .and_then(Value::as_object)
        .expect("contract properties");
    assert!(!contract_properties.contains_key("graphql"));
    assert!(!contract_properties.contains_key("proto"));
    assert!(contract_properties.contains_key("internal_grpc"));

    let public_version_files = schema
        .pointer("/properties/tenant_version_pinning/properties/public_surface_files/properties")
        .and_then(Value::as_object)
        .expect("public version-file properties");
    assert_eq!(
        public_version_files.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from(["asyncapi", "openapi"]),
        "public version carriers must not admit GraphQL or proto3"
    );
}

#[test]
fn queued_15v_has_no_public_proto_or_rpc_discovery_carrier() {
    let plan = json(&declared_path("OYA_MASTER_PLAN_SEQUENCING"));
    let plan_rendered = serde_json::to_string(&plan)
        .expect("master plan must serialize")
        .to_ascii_lowercase();
    for contradiction in [
        "public rest/asyncapi/proto3",
        "proto3 services exposed externally",
        "proto3 reserved field oyatie_version",
        "versionsservice",
    ] {
        assert!(
            !plan_rendered.contains(contradiction),
            "master-plan sequencing reintroduced {contradiction}"
        );
    }
    let wave = plan
        .pointer("/realignment_wave_sequence/waves_15_plus/sub_wave_landings/15V-API-Versioning-Adoption")
        .expect("15V wave");
    let rendered = serde_json::to_string(wave)
        .expect("15V must serialize")
        .to_ascii_lowercase();
    assert!(!rendered.contains("contracts/*.proto"));
    for required in [
        "openapi 3.2.0",
        "signed/versioned webhook",
        "asyncapi 3.1.0",
        "sse",
        "websocket",
        "internal-mesh grpc/proto3 over http/2",
        "exempt",
    ] {
        assert!(rendered.contains(required), "15V must preserve {required}");
    }
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
    let required_codes = policy["rules"]
        .as_array()
        .expect("rules array")
        .iter()
        .map(|rule| rule["code"].as_str().expect("rule code"))
        .collect::<BTreeSet<_>>();
    let fixture_codes = cases
        .iter()
        .map(|case| case["expected_code"].as_str().expect("expected code"))
        .collect::<BTreeSet<_>>();
    assert!(
        required_codes == fixture_codes,
        "negative fixtures must cover every policy rule exactly by code; missing={:?}, unknown={:?}",
        required_codes
            .difference(&fixture_codes)
            .collect::<Vec<_>>(),
        fixture_codes
            .difference(&required_codes)
            .collect::<Vec<_>>()
    );
    let fixture_names = cases
        .iter()
        .map(|case| case["name"].as_str().expect("case name"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        fixture_names.len(),
        cases.len(),
        "fixture names must be unique"
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
