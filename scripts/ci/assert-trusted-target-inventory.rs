//! Validate P0.0 trusted target-inventory fixture coverage without live claims.
//!
//! This checker is local/static fixture evidence only. It proves that the
//! checked-in trusted target-inventory schema and baseline GOOD/BAD fixtures
//! exercise the requirement that Buck2 build/test targets come from trusted
//! dev/controller state before candidate checkout. It does not claim live
//! cloud-ci execution, protected branch authority, P0.0 green, or Phase-0
//! completion.

#[allow(dead_code)]
#[path = "assert-result-bundle-output.rs"]
mod json_support;

pub use json_support::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;

pub const REQUIRED_INVENTORY_FIELDS: [&str; 12] = [
    "candidate_sha",
    "claim_boundary",
    "inventory_source",
    "captured_before_candidate_checkout",
    "candidate_checkout_after_inventory",
    "no_candidate_authored_discovery",
    "build_targets",
    "test_targets",
    "expected_verdict",
    "expected_violations",
    "fixture_id",
    "source_test",
];

const TRUSTED_SOURCE: &str = "trusted_dev_or_controller_state";
const CANDIDATE_SOURCE: &str = "candidate_pr_bytes";
const TRUSTED_TARGET_VIOLATIONS: [&str; 6] = [
    "candidate_can_author_target_inventory",
    "empty_required_targets",
    "green_claim_boundary_without_live_authority",
    "inventory_not_captured_before_candidate_checkout",
    "malformed_buck2_target",
    "target_inventory_not_trusted",
];

const DEFAULT_SCHEMA: &str = "specs/phase0-trusted-target-inventory-schema.json";
const DEFAULT_GOOD_FIXTURE: &str =
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-good-trusted-target-inventory.json";
const DEFAULT_BAD_FIXTURE: &str = "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-bad-candidate-sourced-target-inventory.json";
const AUTHORITY_BOUNDARY: &str = "trusted target-inventory fixture evidence only; this checker never claims live cloud-ci execution or protected-branch authority";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaSummary {
    pub required_fields: Vec<String>,
    pub inventory_source_values: Vec<String>,
    pub expected_verdict_values: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureValidation {
    pub fixture_id: String,
    pub expected_verdict: String,
    pub shape_failures: Vec<String>,
    pub observed_trusted_target_violations: Vec<String>,
    pub malformed_targets: Vec<String>,
    pub trusted_inventory: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub authority_boundary: String,
    pub schema: String,
    pub required_inventory_fields: Vec<String>,
    pub required_trusted_target_violations: Vec<String>,
    pub schema_summary: SchemaSummary,
    pub fixture_results: Vec<FixtureValidation>,
    pub local_fixture_contract_proven: bool,
    pub candidate_pr_bytes_are_data_only_locally_proven: bool,
    pub trusted_target_inventory_live_authority_proven: bool,
    pub trusted_controller_inventory_live: bool,
    pub live_required_context_execution_proven: bool,
    pub p0_0_green: bool,
    pub phase0_complete: bool,
    pub verdict: String,
    pub failures: Vec<String>,
}

pub fn parse_json(text: &str) -> Result<Json, String> {
    json_support::parse_json(text)
}

pub fn load_json(path: &str) -> Result<Json, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {path} failed: {error}"))?;
    parse_json(&text).map_err(|error| format!("parse {path} failed: {error}"))
}

fn object_field<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> Option<&'a Json> {
    object.get(key)
}

fn string_field(object: &BTreeMap<String, Json>, key: &str) -> Option<String> {
    object_field(object, key)
        .and_then(Json::as_str)
        .map(str::to_string)
}

fn bool_field(object: &BTreeMap<String, Json>, key: &str) -> Option<bool> {
    object_field(object, key).and_then(Json::as_bool)
}

fn string_list(value: Option<&Json>) -> Vec<String> {
    value
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Json::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn enum_values(properties: &BTreeMap<String, Json>, field: &str) -> BTreeSet<String> {
    object_field(properties, field)
        .and_then(Json::as_object)
        .map(|object| string_list(object_field(object, "enum")))
        .unwrap_or_default()
        .into_iter()
        .collect()
}

fn set_from(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn require(condition: bool, failures: &mut Vec<String>, message: String) {
    if !condition {
        failures.push(message);
    }
}

fn is_hex_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn allowed_chars(value: &str, allowed: fn(u8) -> bool) -> bool {
    value.bytes().all(allowed)
}

fn prefix_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-')
}

fn package_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'/' | b'-')
}

fn target_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(byte, b'_' | b'+' | b'=' | b'.' | b',' | b'@' | b'~' | b'-')
}

fn is_buck2_target(value: &str) -> bool {
    let Some(double_slash_index) = value.find("//") else {
        return false;
    };
    let prefix = &value[..double_slash_index];
    if !prefix.is_empty() && !allowed_chars(prefix, prefix_char) {
        return false;
    }
    let after_slash = &value[double_slash_index + 2..];
    let Some(colon_index) = after_slash.rfind(':') else {
        return false;
    };
    let package = &after_slash[..colon_index];
    let name = &after_slash[colon_index + 1..];
    !name.is_empty()
        && allowed_chars(package, package_char)
        && allowed_chars(name, target_name_char)
}

fn target_list(value: Option<&Json>) -> Vec<String> {
    string_list(value)
}

fn malformed_targets(targets: &[String]) -> Vec<String> {
    targets
        .iter()
        .filter(|target| !is_buck2_target(target))
        .cloned()
        .collect()
}

fn validate_schema(schema: &Json, failures: &mut Vec<String>) -> SchemaSummary {
    let Some(object) = schema.as_object() else {
        failures.push("schema must be a JSON object".to_string());
        return SchemaSummary {
            required_fields: Vec::new(),
            inventory_source_values: Vec::new(),
            expected_verdict_values: Vec::new(),
        };
    };
    let required = string_list(object_field(object, "required"));
    for field in REQUIRED_INVENTORY_FIELDS {
        require(
            required.iter().any(|item| item == field),
            failures,
            format!("schema.required missing {field}"),
        );
    }
    require(
        bool_field(object, "additionalProperties") == Some(false),
        failures,
        "schema.additionalProperties must be false".to_string(),
    );

    let properties = object_field(object, "properties").and_then(Json::as_object);
    if properties.is_none() {
        failures.push("schema.properties must be an object".to_string());
    }
    let empty = BTreeMap::new();
    let properties = properties.unwrap_or(&empty);
    for field in REQUIRED_INVENTORY_FIELDS {
        require(
            properties.contains_key(field),
            failures,
            format!("schema.properties missing {field}"),
        );
    }

    let source_enum = enum_values(properties, "inventory_source");
    require(
        source_enum == set_from(&[TRUSTED_SOURCE, CANDIDATE_SOURCE]),
        failures,
        "schema.inventory_source enum must be exactly trusted_dev_or_controller_state and candidate_pr_bytes".to_string(),
    );
    let verdict_enum = enum_values(properties, "expected_verdict");
    require(
        verdict_enum == set_from(&["GREEN", "RED"]),
        failures,
        "schema.expected_verdict enum must be exactly GREEN and RED".to_string(),
    );

    let claim_boundary = object_field(properties, "claim_boundary").and_then(Json::as_object);
    let claim_boundary_empty = BTreeMap::new();
    let claim_boundary = claim_boundary.unwrap_or(&claim_boundary_empty);
    let claim_required = string_list(object_field(claim_boundary, "required"))
        .into_iter()
        .collect::<BTreeSet<_>>();
    require(
        set_from(&["p0_0_green", "phase0_complete"]).is_subset(&claim_required),
        failures,
        "schema.claim_boundary must require p0_0_green and phase0_complete".to_string(),
    );
    require(
        bool_field(claim_boundary, "additionalProperties") == Some(false),
        failures,
        "schema.claim_boundary.additionalProperties must be false".to_string(),
    );

    for field in ["build_targets", "test_targets"] {
        let target_field = object_field(properties, field).and_then(Json::as_object);
        let target_field_empty = BTreeMap::new();
        let target_field = target_field.unwrap_or(&target_field_empty);
        require(
            object_field(target_field, "minItems").and_then(Json::as_i64) == Some(1),
            failures,
            format!("schema.{field}.minItems must be 1"),
        );
    }

    SchemaSummary {
        required_fields: required,
        inventory_source_values: source_enum.into_iter().collect(),
        expected_verdict_values: verdict_enum.into_iter().collect(),
    }
}

fn inventory_fixture_shape_failures(fixture: &BTreeMap<String, Json>) -> Vec<String> {
    let mut failures = Vec::new();
    let required_fields = set_from(&REQUIRED_INVENTORY_FIELDS);
    let unknown = fixture
        .keys()
        .filter(|key| !required_fields.contains(*key))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        failures.push(format!("unexpected top-level fields: {unknown:?}"));
    }
    if !string_field(fixture, "candidate_sha").is_some_and(|value| is_hex_sha(&value)) {
        failures.push("candidate_sha must be a 40-character hexadecimal SHA".to_string());
    }
    failures
}

pub fn inventory_fixture_violations(fixture: &Json) -> Vec<String> {
    let Some(fixture) = fixture.as_object() else {
        return vec!["missing_or_malformed_trusted_target_inventory".to_string()];
    };
    let mut violations = Vec::new();
    if string_field(fixture, "inventory_source").as_deref() != Some(TRUSTED_SOURCE) {
        violations.push("target_inventory_not_trusted".to_string());
    }
    if bool_field(fixture, "captured_before_candidate_checkout") != Some(true)
        || bool_field(fixture, "candidate_checkout_after_inventory") != Some(true)
    {
        violations.push("inventory_not_captured_before_candidate_checkout".to_string());
    }
    if bool_field(fixture, "no_candidate_authored_discovery") != Some(true) {
        violations.push("candidate_can_author_target_inventory".to_string());
    }

    let build_targets = target_list(object_field(fixture, "build_targets"));
    let test_targets = target_list(object_field(fixture, "test_targets"));
    if build_targets.is_empty() || test_targets.is_empty() {
        violations.push("empty_required_targets".to_string());
    }
    if !malformed_targets(&[build_targets, test_targets].concat()).is_empty() {
        violations.push("malformed_buck2_target".to_string());
    }

    let claim_boundary = object_field(fixture, "claim_boundary").and_then(Json::as_object);
    let claim_boundary_empty = BTreeMap::new();
    let claim_boundary = claim_boundary.unwrap_or(&claim_boundary_empty);
    if bool_field(claim_boundary, "p0_0_green") != Some(false)
        || bool_field(claim_boundary, "phase0_complete") != Some(false)
    {
        violations.push("green_claim_boundary_without_live_authority".to_string());
    }
    sorted_unique(violations)
}

fn validate_fixture(
    fixture: &Json,
    path: &str,
    expected_verdict: &str,
    failures: &mut Vec<String>,
) -> FixtureValidation {
    let Some(object) = fixture.as_object() else {
        failures.push(format!("{path}: expected JSON object"));
        return FixtureValidation {
            fixture_id: path.to_string(),
            expected_verdict: expected_verdict.to_string(),
            shape_failures: vec!["expected JSON object".to_string()],
            observed_trusted_target_violations: vec![
                "missing_or_malformed_trusted_target_inventory".to_string(),
            ],
            malformed_targets: Vec::new(),
            trusted_inventory: false,
        };
    };
    let fixture_id = string_field(object, "fixture_id").unwrap_or_else(|| path.to_string());
    require(
        string_field(object, "expected_verdict").as_deref() == Some(expected_verdict),
        failures,
        format!("{fixture_id}: expected_verdict must be {expected_verdict}"),
    );
    let shape_failures = inventory_fixture_shape_failures(object);
    for shape_failure in &shape_failures {
        failures.push(format!("{fixture_id}: {shape_failure}"));
    }
    let observed_violations = inventory_fixture_violations(fixture);
    let expected_violations = string_list(object_field(object, "expected_violations"))
        .into_iter()
        .collect::<BTreeSet<_>>();
    if expected_verdict == "GREEN" {
        require(
            observed_violations.is_empty(),
            failures,
            format!(
                "{fixture_id}: GOOD trusted-target inventory has violations {observed_violations:?}"
            ),
        );
        require(
            expected_violations.is_empty(),
            failures,
            format!("{fixture_id}: GOOD fixture must not list expected violations"),
        );
    } else {
        require(
            !observed_violations.is_empty(),
            failures,
            format!("{fixture_id}: RED trusted-target inventory must produce violations"),
        );
        require(
            set_from(&TRUSTED_TARGET_VIOLATIONS).is_subset(&expected_violations),
            failures,
            format!(
                "{fixture_id}: RED fixture expected_violations must include all trusted-target violation classes"
            ),
        );
        let observed_set = observed_violations.iter().cloned().collect::<BTreeSet<_>>();
        require(
            observed_set.is_subset(&expected_violations),
            failures,
            format!(
                "{fixture_id}: observed trusted-target violations not listed in expected_violations"
            ),
        );
    }
    let build_targets = target_list(object_field(object, "build_targets"));
    let test_targets = target_list(object_field(object, "test_targets"));
    FixtureValidation {
        fixture_id,
        expected_verdict: expected_verdict.to_string(),
        shape_failures: shape_failures.clone(),
        observed_trusted_target_violations: observed_violations.clone(),
        malformed_targets: malformed_targets(&[build_targets, test_targets].concat()),
        trusted_inventory: shape_failures.is_empty() && observed_violations.is_empty(),
    }
}

pub fn evaluate_sources(
    schema: &Json,
    good_fixture: &Json,
    bad_fixture: &Json,
    schema_path: &str,
) -> Report {
    let mut failures = Vec::new();
    let schema_summary = validate_schema(schema, &mut failures);
    let fixture_results = vec![
        validate_fixture(good_fixture, DEFAULT_GOOD_FIXTURE, "GREEN", &mut failures),
        validate_fixture(bad_fixture, DEFAULT_BAD_FIXTURE, "RED", &mut failures),
    ];
    let local_fixture_contract_proven = failures.is_empty();
    Report {
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
        schema: schema_path.to_string(),
        required_inventory_fields: REQUIRED_INVENTORY_FIELDS
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
        required_trusted_target_violations: TRUSTED_TARGET_VIOLATIONS
            .iter()
            .map(|violation| (*violation).to_string())
            .collect(),
        schema_summary,
        fixture_results,
        local_fixture_contract_proven,
        candidate_pr_bytes_are_data_only_locally_proven: local_fixture_contract_proven,
        trusted_target_inventory_live_authority_proven: false,
        trusted_controller_inventory_live: false,
        live_required_context_execution_proven: false,
        p0_0_green: false,
        phase0_complete: false,
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_string(),
        failures,
    }
}

pub fn evaluate_paths(
    schema_path: &str,
    good_path: &str,
    bad_path: &str,
) -> Result<Report, String> {
    let schema = load_json(schema_path)?;
    let good = load_json(good_path)?;
    let bad = load_json(bad_path)?;
    Ok(evaluate_sources(&schema, &good, &bad, schema_path))
}

pub fn to_json(report: &Report) -> String {
    format!(
        concat!(
            "{{",
            "\"authority_boundary\":{},",
            "\"candidate_pr_bytes_are_data_only_locally_proven\":{},",
            "\"failures\":{},",
            "\"fixture_results\":{},",
            "\"live_required_context_execution_proven\":false,",
            "\"local_fixture_contract_proven\":{},",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"required_inventory_fields\":{},",
            "\"required_trusted_target_violations\":{},",
            "\"schema\":{},",
            "\"schema_summary\":{},",
            "\"trusted_controller_inventory_live\":false,",
            "\"trusted_target_inventory_live_authority_proven\":false,",
            "\"verdict\":{}",
            "}}"
        ),
        json_string(&report.authority_boundary),
        bool_json(report.candidate_pr_bytes_are_data_only_locally_proven),
        string_array_json(&report.failures),
        fixture_results_json(&report.fixture_results),
        bool_json(report.local_fixture_contract_proven),
        string_array_json(&report.required_inventory_fields),
        string_array_json(&report.required_trusted_target_violations),
        json_string(&report.schema),
        schema_summary_json(&report.schema_summary),
        json_string(&report.verdict),
    )
}

fn schema_summary_json(summary: &SchemaSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"expected_verdict_values\":{},",
            "\"inventory_source_values\":{},",
            "\"required_fields\":{}",
            "}}"
        ),
        string_array_json(&summary.expected_verdict_values),
        string_array_json(&summary.inventory_source_values),
        string_array_json(&summary.required_fields),
    )
}

fn fixture_results_json(results: &[FixtureValidation]) -> String {
    format!(
        "[{}]",
        results
            .iter()
            .map(fixture_validation_json)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn fixture_validation_json(result: &FixtureValidation) -> String {
    format!(
        concat!(
            "{{",
            "\"expected_verdict\":{},",
            "\"fixture_id\":{},",
            "\"malformed_targets\":{},",
            "\"observed_trusted_target_violations\":{},",
            "\"shape_failures\":{},",
            "\"trusted_inventory\":{}",
            "}}"
        ),
        json_string(&result.expected_verdict),
        json_string(&result.fixture_id),
        string_array_json(&result.malformed_targets),
        string_array_json(&result.observed_trusted_target_violations),
        string_array_json(&result.shape_failures),
        bool_json(result.trusted_inventory),
    )
}

fn string_array_json(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn bool_json(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('\"');
    out
}

fn main() {
    let mut schema = DEFAULT_SCHEMA.to_string();
    let mut good_fixture = DEFAULT_GOOD_FIXTURE.to_string();
    let mut bad_fixture = DEFAULT_BAD_FIXTURE.to_string();
    let mut emit_json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--schema" => schema = args.next().expect("--schema requires a value"),
            "--good-fixture" => {
                good_fixture = args.next().expect("--good-fixture requires a value")
            }
            "--bad-fixture" => bad_fixture = args.next().expect("--bad-fixture requires a value"),
            "--json" => emit_json = true,
            other => panic!("unknown argument {other}"),
        }
    }
    let report = evaluate_paths(&schema, &good_fixture, &bad_fixture)
        .unwrap_or_else(|error| panic!("{error}"));
    let rendered = to_json(&report);
    if emit_json || report.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if report.verdict != "PASS" {
        std::process::exit(1);
    }
}
