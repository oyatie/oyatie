//! Validate P0.0 override/kill-switch fixture coverage without live claims.
//!
//! This checker is local/static fixture evidence only. It proves that the
//! checked-in override packet schema and baseline GOOD/BAD fixtures exercise
//! required TTL, reviewer acknowledgment, audit-chain event, owner,
//! blast-radius, revert/fix follow-up, affected-context, degraded-gate, and
//! no-new-oya-CLI surfaces. It does not claim live cloud-ci execution,
//! protected-flow override authority, P0.0 green, or Phase-0 completion.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;

pub const REQUIRED_PACKET_FIELDS: [&str; 10] = [
    "action",
    "affected_contexts",
    "degraded_gate_ids",
    "ttl_expires_at",
    "reviewer_acknowledgment",
    "audit_chain_event",
    "owner",
    "blast_radius_statement",
    "revert_or_fix_follow_up",
    "no_new_oya_cli_surface",
];

const ALLOWED_CONTEXTS: [&str; 2] = ["cloud-ci-required", "oya-ci-required"];
const EXPECTED_ACTION: &str = "temporarily_disable_or_degrade_gate";
const OVERRIDE_VIOLATIONS: [&str; 3] = [
    "override_missing_context_or_gate",
    "override_missing_ttl_reviewer_audit_or_revert",
    "override_new_oya_cli_surface",
];
const DEFAULT_SCHEMA: &str = "specs/phase0-override-packet-schema.json";
const DEFAULT_GOOD_BASELINE_FIXTURE: &str =
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json";
const DEFAULT_BAD_BASELINE_FIXTURE: &str =
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.2-bad-override-without-ttl-audit.json";
const AUTHORITY_BOUNDARY: &str = "override/kill-switch fixture evidence only; this checker never claims live protected-flow override authority or status mutation";

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaSummary {
    pub required_fields: Vec<String>,
    pub allowed_contexts: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaselineFixtureResult {
    pub fixture_id: String,
    pub expected_verdict: String,
    pub observed_override_violations: Vec<String>,
    pub override_packet_valid: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub authority_boundary: String,
    pub schema: String,
    pub required_packet_fields: Vec<String>,
    pub required_override_violations: Vec<String>,
    pub schema_summary: SchemaSummary,
    pub baseline_fixture_results: Vec<BaselineFixtureResult>,
    pub local_fixture_contract_proven: bool,
    pub live_required_context_execution_proven: bool,
    pub protected_flow_override_live: bool,
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

fn non_empty_string(value: Option<&Json>) -> bool {
    value
        .and_then(Json::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn require(condition: bool, failures: &mut Vec<String>, message: String) {
    if !condition {
        failures.push(message);
    }
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
            allowed_contexts: Vec::new(),
        };
    };
    let required = string_list(object_field(object, "required"));
    for field in REQUIRED_PACKET_FIELDS {
        require(
            required.iter().any(|item| item == field),
            failures,
            format!("schema.required missing {field}"),
        );
    }
    require(
        object_field(object, "additionalProperties").and_then(Json::as_bool) == Some(false),
        failures,
        "schema.additionalProperties must be false".to_string(),
    );

    let properties = object_field(object, "properties").and_then(Json::as_object);
    if properties.is_none() {
        failures.push("schema.properties must be an object".to_string());
    }
    let empty = BTreeMap::new();
    let properties = properties.unwrap_or(&empty);

    let action = object_field(properties, "action").and_then(Json::as_object);
    let action_enum = action
        .map(|object| string_list(object_field(object, "enum")))
        .unwrap_or_default();
    require(
        action_enum.iter().any(|item| item == EXPECTED_ACTION),
        failures,
        format!("schema.action.enum missing {EXPECTED_ACTION}"),
    );

    let allowed_contexts = object_field(properties, "affected_contexts")
        .and_then(Json::as_object)
        .and_then(|object| object_field(object, "items"))
        .and_then(Json::as_object)
        .map(|object| string_list(object_field(object, "enum")))
        .unwrap_or_default();
    let allowed_set = allowed_contexts.iter().cloned().collect::<BTreeSet<_>>();
    require(
        ALLOWED_CONTEXTS
            .iter()
            .all(|context| allowed_set.contains(*context)),
        failures,
        "schema.affected_contexts must allow cloud-ci-required and oya-ci-required".to_string(),
    );

    let no_new_oya = object_field(properties, "no_new_oya_cli_surface").and_then(Json::as_object);
    require(
        no_new_oya
            .and_then(|object| object_field(object, "const"))
            .and_then(Json::as_bool)
            == Some(true),
        failures,
        "schema.no_new_oya_cli_surface.const must be true".to_string(),
    );

    let mut allowed_contexts_sorted = allowed_contexts;
    allowed_contexts_sorted.sort();
    SchemaSummary {
        required_fields: required,
        allowed_contexts: allowed_contexts_sorted,
    }
}

pub fn override_packet_violations(packet: &BTreeMap<String, Json>) -> Vec<String> {
    let mut violations = Vec::new();
    if string_field(packet, "action").as_deref() != Some(EXPECTED_ACTION) {
        violations.push("override_invalid_action".to_string());
    }

    let contexts = string_list(object_field(packet, "affected_contexts"));
    let gates = string_list(object_field(packet, "degraded_gate_ids"));
    let allowed_context_set = ALLOWED_CONTEXTS.iter().copied().collect::<BTreeSet<_>>();
    if contexts.is_empty()
        || contexts
            .iter()
            .any(|context| !allowed_context_set.contains(context.as_str()))
        || gates.is_empty()
    {
        violations.push("override_missing_context_or_gate".to_string());
    }

    let required_text_fields = [
        "reviewer_acknowledgment",
        "audit_chain_event",
        "owner",
        "blast_radius_statement",
        "revert_or_fix_follow_up",
    ];
    if !timestamp_valid(object_field(packet, "ttl_expires_at"))
        || required_text_fields
            .iter()
            .any(|field| !non_empty_string(object_field(packet, field)))
    {
        violations.push("override_missing_ttl_reviewer_audit_or_revert".to_string());
    }

    if object_field(packet, "no_new_oya_cli_surface").and_then(Json::as_bool) != Some(true) {
        violations.push("override_new_oya_cli_surface".to_string());
    }
    violations
}

fn validate_fixture(
    fixture: &Json,
    path: &str,
    expected_verdict: &str,
    failures: &mut Vec<String>,
) -> BaselineFixtureResult {
    let Some(object) = fixture.as_object() else {
        failures.push(format!("{path}: expected JSON object"));
        return BaselineFixtureResult {
            fixture_id: path.to_string(),
            expected_verdict: expected_verdict.to_string(),
            observed_override_violations: Vec::new(),
            override_packet_valid: false,
        };
    };
    let fixture_id = string_field(object, "fixture_id").unwrap_or_else(|| path.to_string());
    require(
        string_field(object, "expected_verdict").as_deref() == Some(expected_verdict),
        failures,
        format!("{fixture_id}: expected_verdict must be {expected_verdict}"),
    );

    let packet_object = object_field(object, "override_packet").and_then(Json::as_object);
    if packet_object.is_none() {
        failures.push(format!("{fixture_id}: override_packet must be an object"));
    }
    let empty = BTreeMap::new();
    let packet = packet_object.unwrap_or(&empty);
    let observed_violations = override_packet_violations(packet);
    let expected_violations = string_list(object_field(object, "expected_violations"));
    if expected_verdict == "GREEN" {
        require(
            observed_violations.is_empty(),
            failures,
            format!("{fixture_id}: GOOD override packet has violations {observed_violations:?}"),
        );
        require(
            expected_violations.is_empty(),
            failures,
            format!("{fixture_id}: GOOD fixture must not list expected violations"),
        );
    } else {
        let required_set = OVERRIDE_VIOLATIONS
            .iter()
            .map(|value| (*value).to_string())
            .collect::<BTreeSet<_>>();
        let expected_set = expected_violations.iter().cloned().collect::<BTreeSet<_>>();
        let observed_set = observed_violations.iter().cloned().collect::<BTreeSet<_>>();
        require(
            !observed_violations.is_empty(),
            failures,
            format!("{fixture_id}: RED override packet must produce violations"),
        );
        require(
            required_set.is_subset(&expected_set),
            failures,
            format!(
                "{fixture_id}: RED fixture expected_violations must include all override violation classes"
            ),
        );
        require(
            observed_set.is_subset(&expected_set),
            failures,
            format!("{fixture_id}: observed override violations not listed in expected_violations"),
        );
    }

    BaselineFixtureResult {
        fixture_id,
        expected_verdict: expected_verdict.to_string(),
        override_packet_valid: observed_violations.is_empty(),
        observed_override_violations: observed_violations,
    }
}

pub fn evaluate_sources(
    schema: &Json,
    good_baseline_fixture: &Json,
    bad_baseline_fixture: &Json,
    schema_path: &str,
    good_baseline_path: &str,
    bad_baseline_path: &str,
) -> Report {
    let mut failures = Vec::new();
    let schema_summary = validate_schema(schema, &mut failures);
    let baseline_fixture_results = vec![
        validate_fixture(
            good_baseline_fixture,
            good_baseline_path,
            "GREEN",
            &mut failures,
        ),
        validate_fixture(
            bad_baseline_fixture,
            bad_baseline_path,
            "RED",
            &mut failures,
        ),
    ];
    Report {
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
        schema: schema_path.to_string(),
        required_packet_fields: REQUIRED_PACKET_FIELDS
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
        required_override_violations: OVERRIDE_VIOLATIONS
            .iter()
            .map(|violation| (*violation).to_string())
            .collect(),
        schema_summary,
        baseline_fixture_results,
        local_fixture_contract_proven: failures.is_empty(),
        live_required_context_execution_proven: false,
        protected_flow_override_live: false,
        p0_0_green: false,
        phase0_complete: false,
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_string(),
        failures,
    }
}

pub fn evaluate_paths(
    schema_path: &str,
    good_baseline_path: &str,
    bad_baseline_path: &str,
) -> Result<Report, String> {
    let schema = load_json(schema_path)?;
    let good = load_json(good_baseline_path)?;
    let bad = load_json(bad_baseline_path)?;
    Ok(evaluate_sources(
        &schema,
        &good,
        &bad,
        schema_path,
        good_baseline_path,
        bad_baseline_path,
    ))
}

pub fn to_json(report: &Report) -> String {
    format!(
        concat!(
            "{{",
            "\"authority_boundary\":{},",
            "\"baseline_fixture_results\":{},",
            "\"failures\":{},",
            "\"live_required_context_execution_proven\":false,",
            "\"local_fixture_contract_proven\":{},",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"protected_flow_override_live\":false,",
            "\"required_override_violations\":{},",
            "\"required_packet_fields\":{},",
            "\"schema\":{},",
            "\"schema_summary\":{},",
            "\"verdict\":{}",
            "}}"
        ),
        json_string(&report.authority_boundary),
        baseline_results_json(&report.baseline_fixture_results),
        string_array_json(&report.failures),
        bool_json(report.local_fixture_contract_proven),
        string_array_json(&report.required_override_violations),
        string_array_json(&report.required_packet_fields),
        json_string(&report.schema),
        schema_summary_json(&report.schema_summary),
        json_string(&report.verdict),
    )
}

fn schema_summary_json(summary: &SchemaSummary) -> String {
    format!(
        "{{\"allowed_contexts\":{},\"required_fields\":{}}}",
        string_array_json(&summary.allowed_contexts),
        string_array_json(&summary.required_fields),
    )
}

fn baseline_results_json(results: &[BaselineFixtureResult]) -> String {
    format!(
        "[{}]",
        results
            .iter()
            .map(|result| {
                format!(
                    concat!(
                        "{{",
                        "\"expected_verdict\":{},",
                        "\"fixture_id\":{},",
                        "\"observed_override_violations\":{},",
                        "\"override_packet_valid\":{}",
                        "}}"
                    ),
                    json_string(&result.expected_verdict),
                    json_string(&result.fixture_id),
                    string_array_json(&result.observed_override_violations),
                    bool_json(result.override_packet_valid),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
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
    let mut good_baseline_fixture = DEFAULT_GOOD_BASELINE_FIXTURE.to_string();
    let mut bad_baseline_fixture = DEFAULT_BAD_BASELINE_FIXTURE.to_string();
    let mut emit_json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--schema" => schema = args.next().expect("--schema requires a value"),
            "--good-baseline-fixture" => {
                good_baseline_fixture = args
                    .next()
                    .expect("--good-baseline-fixture requires a value")
            }
            "--bad-baseline-fixture" => {
                bad_baseline_fixture = args
                    .next()
                    .expect("--bad-baseline-fixture requires a value")
            }
            "--json" => emit_json = true,
            other => panic!("unknown argument {other}"),
        }
    }
    let report = evaluate_paths(&schema, &good_baseline_fixture, &bad_baseline_fixture)
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
