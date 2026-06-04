//! Validate P0.0 structured result-bundle fixture coverage without live claims.
//!
//! This checker is local/static fixture evidence only. It proves that the
//! checked-in structured result schema and RED/false-green fixtures exercise
//! result-bundle authority boundaries. It does not post statuses and does not
//! claim live cloud-ci execution, protected branch authority, P0.0 green, or
//! Phase-0 completion.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;

pub const REQUIRED_BUNDLE_FIELDS: [&str; 7] = [
    "candidate_sha",
    "required_context",
    "producer",
    "fixture_results",
    "observed_verdict",
    "provenance",
    "claim_boundary",
];

const REQUIRED_PRODUCER_FIELDS: [&str; 3] = ["context", "kind", "trusted_control_state"];
const REQUIRED_FIXTURE_RESULT_FIELDS: [&str; 4] = [
    "fixture_id",
    "expected_verdict",
    "observed_verdict",
    "violations",
];
const REQUIRED_CONTEXTS: [&str; 2] = ["cloud-ci-required", "oya-ci-required"];
const ALLOWED_CONTEXT_VALUES: [&str; 3] = ["cloud-ci-required", "missing", "oya-ci-required"];
const TRUSTED_PRODUCER_KINDS: [&str; 2] = ["minimal_rust_bridge_adapter", "oya-ci-controller"];
const EXPECTED_FALSE_GREEN_VIOLATIONS: [&str; 8] = [
    "candidate_bytes_can_weaken_result",
    "candidate_sourced_gate_definition",
    "fixture_result_mismatch",
    "green_bundle_without_green_fixture_results",
    "green_claim_boundary_without_live_authority",
    "missing_cloud_ci_required_context",
    "red_expected_fixture_missing_violations",
    "untrusted_or_legacy_status_producer",
];
const CURRENT_RED_REQUIRED_VIOLATIONS: [&str; 4] = [
    "candidate_bytes_can_weaken_result",
    "candidate_sourced_gate_definition",
    "missing_cloud_ci_required_context",
    "untrusted_or_legacy_status_producer",
];

const DEFAULT_SCHEMA: &str = "specs/phase0-ci-enforcement-result-schema.json";
const DEFAULT_CURRENT_RED_FIXTURE: &str =
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-current-red-gap-result.json";
const DEFAULT_FALSE_GREEN_FIXTURE: &str =
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.4-bad-result-bundle-false-green.json";
const AUTHORITY_BOUNDARY: &str = "structured result-bundle fixture evidence only; this checker never posts statuses or claims live required-context authority";

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    pub fn as_object(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Json::Object(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Json::Number(value) | Json::String(value) => value.parse::<i64>().ok(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaSummary {
    pub required_fields: Vec<String>,
    pub required_context_values: Vec<String>,
    pub observed_verdict_values: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultBundleValidation {
    pub path: String,
    pub observed_verdict: String,
    pub shape_failures: Vec<String>,
    pub observed_result_bundle_violations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub authority_boundary: String,
    pub schema: String,
    pub required_bundle_fields: Vec<String>,
    pub required_false_green_violations: Vec<String>,
    pub current_red_required_violations: Vec<String>,
    pub schema_summary: SchemaSummary,
    pub current_red_result: ResultBundleValidation,
    pub false_green_result: ResultBundleValidation,
    pub local_fixture_contract_proven: bool,
    pub structured_result_bundle_live: bool,
    pub trusted_status_producer_live: bool,
    pub protected_branch_authority_proven: bool,
    pub status_mutation_performed: bool,
    pub live_required_context_execution_proven: bool,
    pub p0_0_green: bool,
    pub phase0_complete: bool,
    pub verdict: String,
    pub failures: Vec<String>,
}

struct Parser<'a> {
    text: &'a [u8],
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text: text.as_bytes(),
            index: 0,
        }
    }

    fn parse(mut self) -> Result<Json, String> {
        let value = self.parse_value()?;
        self.skip_ws();
        if self.index == self.text.len() {
            Ok(value)
        } else {
            Err(format!("unexpected trailing JSON at byte {}", self.index))
        }
    }

    fn parse_value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'\"') => self.parse_string().map(Json::String),
            Some(b't') => self.expect_literal(b"true", Json::Bool(true)),
            Some(b'f') => self.expect_literal(b"false", Json::Bool(false)),
            Some(b'n') => self.expect_literal(b"null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(Json::Number),
            Some(byte) => Err(format!(
                "unexpected JSON byte {} at byte {}",
                byte as char, self.index
            )),
            None => Err("unexpected end of JSON".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<Json, String> {
        self.consume(b'{')?;
        let mut object = BTreeMap::new();
        self.skip_ws();
        if self.try_consume(b'}') {
            return Ok(Json::Object(object));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.consume(b':')?;
            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_ws();
            if self.try_consume(b'}') {
                break;
            }
            self.consume(b',')?;
        }
        Ok(Json::Object(object))
    }

    fn parse_array(&mut self) -> Result<Json, String> {
        self.consume(b'[')?;
        let mut array = Vec::new();
        self.skip_ws();
        if self.try_consume(b']') {
            return Ok(Json::Array(array));
        }
        loop {
            array.push(self.parse_value()?);
            self.skip_ws();
            if self.try_consume(b']') {
                break;
            }
            self.consume(b',')?;
        }
        Ok(Json::Array(array))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume(b'\"')?;
        let mut out = String::new();
        while let Some(byte) = self.next() {
            match byte {
                b'\"' => return Ok(out),
                b'\\' => {
                    let escaped = self
                        .next()
                        .ok_or_else(|| "unterminated JSON string escape".to_string())?;
                    match escaped {
                        b'\"' => out.push('\"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => out.push(
                            char::from_u32(self.parse_unicode_escape()?).unwrap_or('\u{fffd}'),
                        ),
                        other => {
                            return Err(format!(
                                "invalid JSON string escape {} at byte {}",
                                other as char, self.index
                            ));
                        }
                    }
                }
                other => out.push(other as char),
            }
        }
        Err("unterminated JSON string".to_string())
    }

    fn parse_unicode_escape(&mut self) -> Result<u32, String> {
        let mut value = 0u32;
        for _ in 0..4 {
            let byte = self
                .next()
                .ok_or_else(|| "short JSON unicode escape".to_string())?;
            value = value * 16
                + match byte {
                    b'0'..=b'9' => (byte - b'0') as u32,
                    b'a'..=b'f' => (byte - b'a' + 10) as u32,
                    b'A'..=b'F' => (byte - b'A' + 10) as u32,
                    _ => return Err("invalid JSON unicode escape".to_string()),
                };
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<String, String> {
        let start = self.index;
        if self.peek() == Some(b'-') {
            self.index += 1;
        }
        self.consume_digits();
        if self.peek() == Some(b'.') {
            self.index += 1;
            self.consume_digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.index += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.index += 1;
            }
            self.consume_digits();
        }
        std::str::from_utf8(&self.text[start..self.index])
            .map(|value| value.to_string())
            .map_err(|error| format!("invalid JSON number utf8: {error}"))
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.index += 1;
        }
    }

    fn expect_literal(&mut self, literal: &[u8], value: Json) -> Result<Json, String> {
        if self.text.get(self.index..self.index + literal.len()) == Some(literal) {
            self.index += literal.len();
            Ok(value)
        } else {
            Err(format!("expected literal at byte {}", self.index))
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.index += 1;
        }
    }

    fn consume(&mut self, expected: u8) -> Result<(), String> {
        match self.next() {
            Some(actual) if actual == expected => Ok(()),
            Some(actual) => Err(format!(
                "expected {} at byte {}, got {}",
                expected as char,
                self.index.saturating_sub(1),
                actual as char
            )),
            None => Err(format!("expected {} at end of JSON", expected as char)),
        }
    }

    fn try_consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> Option<u8> {
        let value = self.peek()?;
        self.index += 1;
        Some(value)
    }

    fn peek(&self) -> Option<u8> {
        self.text.get(self.index).copied()
    }
}

pub fn parse_json(text: &str) -> Result<Json, String> {
    Parser::new(text).parse()
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

fn require(condition: bool, failures: &mut Vec<String>, message: String) {
    if !condition {
        failures.push(message);
    }
}

fn set_from(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn is_hex_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn timestamp_valid(value: Option<&Json>) -> bool {
    let Some(value) = value.and_then(Json::as_str) else {
        return false;
    };
    let bytes = value.as_bytes();
    bytes.len() == 20
        && bytes[0] == b'2'
        && bytes[1] == b'0'
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        })
}

fn validate_schema(schema: &Json, failures: &mut Vec<String>) -> SchemaSummary {
    let Some(object) = schema.as_object() else {
        failures.push("schema must be a JSON object".to_string());
        return SchemaSummary {
            required_fields: Vec::new(),
            required_context_values: Vec::new(),
            observed_verdict_values: Vec::new(),
        };
    };
    let required = string_list(object_field(object, "required"));
    for field in REQUIRED_BUNDLE_FIELDS {
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
    for field in REQUIRED_BUNDLE_FIELDS {
        require(
            properties.contains_key(field),
            failures,
            format!("schema.properties missing {field}"),
        );
    }

    let candidate_sha = object_field(properties, "candidate_sha").and_then(Json::as_object);
    let candidate_sha_empty = BTreeMap::new();
    let candidate_sha = candidate_sha.unwrap_or(&candidate_sha_empty);
    require(
        object_field(candidate_sha, "minLength").and_then(Json::as_i64) == Some(40)
            && object_field(candidate_sha, "maxLength").and_then(Json::as_i64) == Some(40),
        failures,
        "schema.candidate_sha must require exactly 40 characters".to_string(),
    );
    require(
        string_field(candidate_sha, "pattern").as_deref() == Some("^[0-9a-fA-F]{40}$"),
        failures,
        "schema.candidate_sha must require 40 hexadecimal characters".to_string(),
    );

    let required_context_values = enum_values(properties, "required_context");
    require(
        required_context_values == set_from(&ALLOWED_CONTEXT_VALUES),
        failures,
        "schema.required_context enum must be exactly cloud-ci-required, oya-ci-required, and missing".to_string(),
    );
    let observed_verdict_values = enum_values(properties, "observed_verdict");
    require(
        observed_verdict_values == set_from(&["GREEN", "RED"]),
        failures,
        "schema.observed_verdict enum must be exactly GREEN and RED".to_string(),
    );

    let producer = object_field(properties, "producer").and_then(Json::as_object);
    let producer_empty = BTreeMap::new();
    let producer = producer.unwrap_or(&producer_empty);
    let producer_required = string_list(object_field(producer, "required"))
        .into_iter()
        .collect::<BTreeSet<_>>();
    require(
        set_from(&REQUIRED_PRODUCER_FIELDS).is_subset(&producer_required),
        failures,
        "schema.producer must require context, kind, and trusted_control_state".to_string(),
    );
    require(
        bool_field(producer, "additionalProperties") == Some(false),
        failures,
        "schema.producer.additionalProperties must be false".to_string(),
    );
    let producer_properties = object_field(producer, "properties").and_then(Json::as_object);
    let producer_properties_empty = BTreeMap::new();
    let producer_properties = producer_properties.unwrap_or(&producer_properties_empty);
    require(
        enum_values(producer_properties, "context") == set_from(&ALLOWED_CONTEXT_VALUES),
        failures,
        "schema.producer.context enum must be exactly cloud-ci-required, oya-ci-required, and missing".to_string(),
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

    let fixture_results = object_field(properties, "fixture_results").and_then(Json::as_object);
    let fixture_results_empty = BTreeMap::new();
    let fixture_results = fixture_results.unwrap_or(&fixture_results_empty);
    require(
        object_field(fixture_results, "minItems").and_then(Json::as_i64) == Some(1),
        failures,
        "schema.fixture_results.minItems must be 1".to_string(),
    );
    let items = object_field(fixture_results, "items").and_then(Json::as_object);
    let items_empty = BTreeMap::new();
    let items = items.unwrap_or(&items_empty);
    let item_properties = object_field(items, "properties").and_then(Json::as_object);
    let item_properties_empty = BTreeMap::new();
    let item_properties = item_properties.unwrap_or(&item_properties_empty);
    let item_required = string_list(object_field(items, "required"))
        .into_iter()
        .collect::<BTreeSet<_>>();
    require(
        set_from(&REQUIRED_FIXTURE_RESULT_FIELDS).is_subset(&item_required),
        failures,
        "schema.fixture_results.items must require fixture_id, expected_verdict, observed_verdict, and violations".to_string(),
    );
    require(
        bool_field(items, "additionalProperties") == Some(false),
        failures,
        "schema.fixture_results.items.additionalProperties must be false".to_string(),
    );
    require(
        enum_values(item_properties, "expected_verdict") == set_from(&["GREEN", "RED"]),
        failures,
        "schema.fixture_results.items.expected_verdict enum must be exactly GREEN and RED"
            .to_string(),
    );
    require(
        enum_values(item_properties, "observed_verdict") == set_from(&["GREEN", "RED"]),
        failures,
        "schema.fixture_results.items.observed_verdict enum must be exactly GREEN and RED"
            .to_string(),
    );

    let provenance = object_field(properties, "provenance").and_then(Json::as_object);
    let provenance_empty = BTreeMap::new();
    let provenance = provenance.unwrap_or(&provenance_empty);
    let provenance_required = string_list(object_field(provenance, "required"))
        .into_iter()
        .collect::<BTreeSet<_>>();
    require(
        set_from(&["recorded_at", "sources"]).is_subset(&provenance_required),
        failures,
        "schema.provenance must require recorded_at and sources".to_string(),
    );
    require(
        bool_field(provenance, "additionalProperties") == Some(false),
        failures,
        "schema.provenance.additionalProperties must be false".to_string(),
    );

    SchemaSummary {
        required_fields: required,
        required_context_values: required_context_values.into_iter().collect(),
        observed_verdict_values: observed_verdict_values.into_iter().collect(),
    }
}

fn bundle_shape_failures(bundle: &BTreeMap<String, Json>) -> Vec<String> {
    let mut failures = Vec::new();
    let required_fields = set_from(&REQUIRED_BUNDLE_FIELDS);
    let unknown = bundle
        .keys()
        .filter(|key| !required_fields.contains(*key))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        failures.push(format!("unexpected top-level fields: {unknown:?}"));
    }
    if !string_field(bundle, "candidate_sha").is_some_and(|value| is_hex_sha(&value)) {
        failures.push("candidate_sha must be a 40-character hexadecimal SHA".to_string());
    }
    match object_field(bundle, "provenance").and_then(Json::as_object) {
        Some(provenance) => {
            if !timestamp_valid(object_field(provenance, "recorded_at")) {
                failures.push(
                    "provenance.recorded_at must be an ISO-8601 UTC second timestamp".to_string(),
                );
            }
            if string_list(object_field(provenance, "sources")).is_empty() {
                failures.push("provenance.sources must be a non-empty string array".to_string());
            }
        }
        None => failures.push("provenance must be an object".to_string()),
    }
    failures
}

pub fn evaluate_result_bundle(bundle: &Json) -> Vec<String> {
    let Some(bundle) = bundle.as_object() else {
        return vec!["missing_or_malformed_result_bundle".to_string()];
    };
    let mut violations = Vec::new();
    if !string_field(bundle, "candidate_sha").is_some_and(|value| is_hex_sha(&value)) {
        violations.push("invalid_candidate_sha".to_string());
    }

    let required_context = string_field(bundle, "required_context");
    if !required_context
        .as_deref()
        .is_some_and(|context| REQUIRED_CONTEXTS.contains(&context))
    {
        violations.push("missing_cloud_ci_required_context".to_string());
    }

    let producer = object_field(bundle, "producer").and_then(Json::as_object);
    let producer_empty = BTreeMap::new();
    let producer = producer.unwrap_or(&producer_empty);
    let producer_context = string_field(producer, "context");
    if bool_field(producer, "trusted_control_state") != Some(true)
        || !string_field(producer, "kind")
            .as_deref()
            .is_some_and(|kind| TRUSTED_PRODUCER_KINDS.contains(&kind))
        || producer_context != required_context
        || !producer_context
            .as_deref()
            .is_some_and(|context| REQUIRED_CONTEXTS.contains(&context))
    {
        violations.push("untrusted_or_legacy_status_producer".to_string());
    }
    if string_field(producer, "candidate_bytes_policy").as_deref() != Some("untrusted_input_only") {
        violations.push("candidate_bytes_can_weaken_result".to_string());
    }
    if string_field(producer, "gate_definition_source").as_deref()
        != Some("trusted_dev_or_controller_state")
    {
        violations.push("candidate_sourced_gate_definition".to_string());
    }

    let Some(fixture_results) = object_field(bundle, "fixture_results").and_then(Json::as_array)
    else {
        violations.push("missing_or_malformed_result_bundle".to_string());
        return sorted_unique(violations);
    };
    if fixture_results.is_empty() {
        violations.push("missing_or_malformed_result_bundle".to_string());
        return sorted_unique(violations);
    }

    let mut all_fixture_results_green = true;
    for fixture in fixture_results {
        let Some(fixture) = fixture.as_object() else {
            violations.push("missing_or_malformed_result_bundle".to_string());
            all_fixture_results_green = false;
            continue;
        };
        let expected = string_field(fixture, "expected_verdict");
        let observed = string_field(fixture, "observed_verdict");
        let expected_ok = expected
            .as_deref()
            .is_some_and(|value| matches!(value, "RED" | "GREEN"));
        let observed_ok = observed
            .as_deref()
            .is_some_and(|value| matches!(value, "RED" | "GREEN"));
        if !expected_ok || !observed_ok || expected != observed {
            violations.push("fixture_result_mismatch".to_string());
        }
        if expected.as_deref() == Some("RED")
            && string_list(object_field(fixture, "violations")).is_empty()
        {
            violations.push("red_expected_fixture_missing_violations".to_string());
        }
        if expected.as_deref() != Some("GREEN") || observed.as_deref() != Some("GREEN") {
            all_fixture_results_green = false;
        }
    }

    let observed_verdict = string_field(bundle, "observed_verdict");
    let claim_boundary = object_field(bundle, "claim_boundary").and_then(Json::as_object);
    let claim_boundary_empty = BTreeMap::new();
    let claim_boundary = claim_boundary.unwrap_or(&claim_boundary_empty);
    let claims_p0_green = bool_field(claim_boundary, "p0_0_green") == Some(true);
    let claims_phase0_complete = bool_field(claim_boundary, "phase0_complete") == Some(true);

    if observed_verdict.as_deref() == Some("GREEN") && !all_fixture_results_green {
        violations.push("green_bundle_without_green_fixture_results".to_string());
    }
    if observed_verdict.as_deref() == Some("GREEN") && !violations.is_empty() && claims_p0_green {
        violations.push("green_claim_boundary_without_live_authority".to_string());
    }
    if observed_verdict.as_deref() == Some("RED") && (claims_p0_green || claims_phase0_complete) {
        violations.push("red_bundle_claims_green_boundary".to_string());
    }

    sorted_unique(violations)
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn validate_current_red(
    bundle: &Json,
    path: &str,
    failures: &mut Vec<String>,
) -> ResultBundleValidation {
    let Some(object) = bundle.as_object() else {
        failures.push("current RED result bundle: expected JSON object".to_string());
        return ResultBundleValidation {
            path: path.to_string(),
            observed_verdict: String::new(),
            shape_failures: vec!["expected JSON object".to_string()],
            observed_result_bundle_violations: vec![
                "missing_or_malformed_result_bundle".to_string(),
            ],
        };
    };
    let shape_failures = bundle_shape_failures(object);
    for failure in &shape_failures {
        failures.push(format!("current RED result bundle: {failure}"));
    }
    require(
        string_field(object, "observed_verdict").as_deref() == Some("RED"),
        failures,
        "current RED result bundle must keep observed_verdict=RED".to_string(),
    );
    let boundary = object_field(object, "claim_boundary").and_then(Json::as_object);
    let boundary_empty = BTreeMap::new();
    let boundary = boundary.unwrap_or(&boundary_empty);
    require(
        bool_field(boundary, "p0_0_green") == Some(false)
            && bool_field(boundary, "phase0_complete") == Some(false),
        failures,
        "current RED result bundle must keep p0_0_green=false and phase0_complete=false"
            .to_string(),
    );
    let violations = evaluate_result_bundle(bundle);
    require(
        !violations
            .iter()
            .any(|violation| violation == "missing_or_malformed_result_bundle"),
        failures,
        "current RED result bundle must remain schema-shaped and non-empty".to_string(),
    );
    let violation_set = violations.iter().cloned().collect::<BTreeSet<_>>();
    require(
        set_from(&CURRENT_RED_REQUIRED_VIOLATIONS).is_subset(&violation_set),
        failures,
        "current RED result bundle must expose missing-context, untrusted-producer, candidate-bytes, and candidate-sourced violations".to_string(),
    );
    ResultBundleValidation {
        path: path.to_string(),
        observed_verdict: string_field(object, "observed_verdict").unwrap_or_default(),
        shape_failures,
        observed_result_bundle_violations: violations,
    }
}

fn validate_false_green(
    bundle: &Json,
    path: &str,
    failures: &mut Vec<String>,
) -> ResultBundleValidation {
    let Some(object) = bundle.as_object() else {
        failures.push("false-green result bundle: expected JSON object".to_string());
        return ResultBundleValidation {
            path: path.to_string(),
            observed_verdict: String::new(),
            shape_failures: vec!["expected JSON object".to_string()],
            observed_result_bundle_violations: vec![
                "missing_or_malformed_result_bundle".to_string(),
            ],
        };
    };
    let shape_failures = bundle_shape_failures(object);
    for failure in &shape_failures {
        failures.push(format!("false-green result bundle: {failure}"));
    }
    require(
        string_field(object, "observed_verdict").as_deref() == Some("GREEN"),
        failures,
        "false-green result bundle fixture must exercise observed_verdict=GREEN".to_string(),
    );
    let boundary = object_field(object, "claim_boundary").and_then(Json::as_object);
    let boundary_empty = BTreeMap::new();
    let boundary = boundary.unwrap_or(&boundary_empty);
    require(
        bool_field(boundary, "p0_0_green") == Some(true)
            && bool_field(boundary, "phase0_complete") == Some(true),
        failures,
        "false-green result bundle must exercise p0_0_green=true and phase0_complete=true"
            .to_string(),
    );
    let violations = evaluate_result_bundle(bundle);
    require(
        !violations
            .iter()
            .any(|violation| violation == "missing_or_malformed_result_bundle"),
        failures,
        "false-green result bundle must remain schema-shaped and non-empty".to_string(),
    );
    let violation_set = violations.iter().cloned().collect::<BTreeSet<_>>();
    require(
        set_from(&EXPECTED_FALSE_GREEN_VIOLATIONS).is_subset(&violation_set),
        failures,
        "false-green result bundle must expose all required false-green violation classes"
            .to_string(),
    );
    ResultBundleValidation {
        path: path.to_string(),
        observed_verdict: string_field(object, "observed_verdict").unwrap_or_default(),
        shape_failures,
        observed_result_bundle_violations: violations,
    }
}

pub fn evaluate_sources(
    schema: &Json,
    current_red_fixture: &Json,
    false_green_fixture: &Json,
    schema_path: &str,
    current_red_path: &str,
    false_green_path: &str,
) -> Report {
    let mut failures = Vec::new();
    let schema_summary = validate_schema(schema, &mut failures);
    let current_red_result =
        validate_current_red(current_red_fixture, current_red_path, &mut failures);
    let false_green_result =
        validate_false_green(false_green_fixture, false_green_path, &mut failures);
    Report {
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
        schema: schema_path.to_string(),
        required_bundle_fields: REQUIRED_BUNDLE_FIELDS
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
        required_false_green_violations: EXPECTED_FALSE_GREEN_VIOLATIONS
            .iter()
            .map(|violation| (*violation).to_string())
            .collect(),
        current_red_required_violations: CURRENT_RED_REQUIRED_VIOLATIONS
            .iter()
            .map(|violation| (*violation).to_string())
            .collect(),
        schema_summary,
        current_red_result,
        false_green_result,
        local_fixture_contract_proven: failures.is_empty(),
        structured_result_bundle_live: false,
        trusted_status_producer_live: false,
        protected_branch_authority_proven: false,
        status_mutation_performed: false,
        live_required_context_execution_proven: false,
        p0_0_green: false,
        phase0_complete: false,
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_string(),
        failures,
    }
}

pub fn evaluate_paths(
    schema_path: &str,
    current_red_path: &str,
    false_green_path: &str,
) -> Result<Report, String> {
    let schema = load_json(schema_path)?;
    let current_red = load_json(current_red_path)?;
    let false_green = load_json(false_green_path)?;
    Ok(evaluate_sources(
        &schema,
        &current_red,
        &false_green,
        schema_path,
        current_red_path,
        false_green_path,
    ))
}

pub fn to_json(report: &Report) -> String {
    format!(
        concat!(
            "{{",
            "\"authority_boundary\":{},",
            "\"current_red_required_violations\":{},",
            "\"current_red_result\":{},",
            "\"failures\":{},",
            "\"false_green_result\":{},",
            "\"live_required_context_execution_proven\":false,",
            "\"local_fixture_contract_proven\":{},",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"protected_branch_authority_proven\":false,",
            "\"required_bundle_fields\":{},",
            "\"required_false_green_violations\":{},",
            "\"schema\":{},",
            "\"schema_summary\":{},",
            "\"status_mutation_performed\":false,",
            "\"structured_result_bundle_live\":false,",
            "\"trusted_status_producer_live\":false,",
            "\"verdict\":{}",
            "}}"
        ),
        json_string(&report.authority_boundary),
        string_array_json(&report.current_red_required_violations),
        result_validation_json(&report.current_red_result),
        string_array_json(&report.failures),
        result_validation_json(&report.false_green_result),
        bool_json(report.local_fixture_contract_proven),
        string_array_json(&report.required_bundle_fields),
        string_array_json(&report.required_false_green_violations),
        json_string(&report.schema),
        schema_summary_json(&report.schema_summary),
        json_string(&report.verdict),
    )
}

fn schema_summary_json(summary: &SchemaSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"observed_verdict_values\":{},",
            "\"required_context_values\":{},",
            "\"required_fields\":{}",
            "}}"
        ),
        string_array_json(&summary.observed_verdict_values),
        string_array_json(&summary.required_context_values),
        string_array_json(&summary.required_fields),
    )
}

fn result_validation_json(result: &ResultBundleValidation) -> String {
    format!(
        concat!(
            "{{",
            "\"observed_result_bundle_violations\":{},",
            "\"observed_verdict\":{},",
            "\"path\":{},",
            "\"shape_failures\":{}",
            "}}"
        ),
        string_array_json(&result.observed_result_bundle_violations),
        json_string(&result.observed_verdict),
        json_string(&result.path),
        string_array_json(&result.shape_failures),
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
    let mut current_red_fixture = DEFAULT_CURRENT_RED_FIXTURE.to_string();
    let mut false_green_fixture = DEFAULT_FALSE_GREEN_FIXTURE.to_string();
    let mut emit_json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--schema" => schema = args.next().expect("--schema requires a value"),
            "--current-red-fixture" => {
                current_red_fixture = args.next().expect("--current-red-fixture requires a value")
            }
            "--false-green-fixture" => {
                false_green_fixture = args.next().expect("--false-green-fixture requires a value")
            }
            "--json" => emit_json = true,
            other => panic!("unknown argument {other}"),
        }
    }
    let report = evaluate_paths(&schema, &current_red_fixture, &false_green_fixture)
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
