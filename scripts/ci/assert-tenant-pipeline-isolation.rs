//! Validate P0.0 tenant-pipeline isolation fixture coverage without live claims.
//!
//! This checker is local/static fixture evidence only. It proves that the
//! checked-in contract and baseline GOOD/BAD fixtures exercise required
//! tenant-pipeline separation surfaces; it does not claim live cloud-ci
//! execution, tenant-facing readiness, security readiness, or Phase-0
//! completion.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;

pub const REQUIRED_SURFACES: [&str; 11] = [
    "identity",
    "secrets",
    "runners",
    "workspaces",
    "caches",
    "artifacts",
    "logs_evidence",
    "release_ledgers",
    "deploy_targets",
    "status_callbacks",
    "audit_events",
];

const DEFAULT_CONTRACT: &str = "specs/toolchain-tenant-isolation-fixtures.json";
const DEFAULT_GOOD_BASELINE_FIXTURE: &str =
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json";
const DEFAULT_BAD_BASELINE_FIXTURE: &str =
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.3-bad-cross-tenant-shared-cache.json";
const AUTHORITY_BOUNDARY: &str = "tenant-isolation fixture evidence only; this checker never claims live cloud-ci execution or tenant-facing readiness";

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
pub struct ContractFixtureResult {
    pub fixture_id: String,
    pub expected_verdict: String,
    pub missing_surfaces: Vec<String>,
    pub shared_surfaces: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaselineFixtureResult {
    pub fixture_id: String,
    pub expected_verdict: String,
    pub observed_tenant_violations: Vec<String>,
    pub tenant_surfaces_complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub authority_boundary: String,
    pub contract: String,
    pub required_surfaces: Vec<String>,
    pub contract_fixture_results: Vec<ContractFixtureResult>,
    pub baseline_fixture_results: Vec<BaselineFixtureResult>,
    pub local_fixture_contract_proven: bool,
    pub live_required_context_execution_proven: bool,
    pub tenant_facing_ready: bool,
    pub security_ready: bool,
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

fn surface_aliases(surface: &str) -> &'static [&'static str] {
    match surface {
        "identity" => &["identity"],
        "secrets" => &["secrets", "secret_scope", "secret_lease"],
        "runners" => &["runners", "runner_pool"],
        "workspaces" => &["workspaces", "workspace_volume"],
        "caches" => &["caches", "cache_namespace"],
        "artifacts" => &["artifacts", "artifact_namespace"],
        "logs_evidence" => &["logs_evidence", "log_evidence_namespace"],
        "release_ledgers" => &["release_ledgers", "release_ledger"],
        "deploy_targets" => &["deploy_targets", "deploy_target"],
        "status_callbacks" => &["status_callbacks", "status_callback_identity"],
        "audit_events" => &["audit_events", "audit_event_stream"],
        _ => &[],
    }
}

fn surface_is_present(required: &str, observed: &BTreeSet<String>) -> bool {
    surface_aliases(required)
        .iter()
        .any(|alias| observed.contains(*alias))
}

pub fn missing_required_surfaces(separated_surfaces: &[String]) -> Vec<String> {
    let observed = separated_surfaces.iter().cloned().collect::<BTreeSet<_>>();
    REQUIRED_SURFACES
        .iter()
        .filter(|surface| !surface_is_present(surface, &observed))
        .map(|surface| (*surface).to_string())
        .collect()
}

fn internal_bypass_without_breakglass(model: &BTreeMap<String, Json>) -> bool {
    match object_field(model, "internal_bypass") {
        Some(Json::Object(object)) => object
            .get("allowed_without_ttl_breakglass")
            .and_then(Json::as_bool)
            .is_some_and(|value| value),
        Some(Json::String(value)) => {
            let lowered = value.to_ascii_lowercase();
            !lowered.contains("ttl")
                || !lowered.contains("breakglass")
                || !lowered.contains("audit")
        }
        _ => false,
    }
}

pub fn evaluate_tenant_model(model: &BTreeMap<String, Json>) -> Vec<String> {
    let mut separated = string_list(object_field(model, "separate_surfaces"));
    separated.extend(string_list(object_field(model, "partitioned_surfaces")));
    let mut violations = Vec::new();
    if !missing_required_surfaces(&separated).is_empty() {
        violations.push("tenant_surface_separation_incomplete".to_string());
    }
    if !string_list(object_field(model, "shared_surfaces")).is_empty() {
        violations.push("tenant_surfaces_shared".to_string());
    }
    if internal_bypass_without_breakglass(model) {
        violations.push("internal_bypass_without_breakglass".to_string());
    }
    violations
}

fn require(condition: bool, failures: &mut Vec<String>, message: String) {
    if !condition {
        failures.push(message);
    }
}

fn validate_contract(contract: &Json, failures: &mut Vec<String>) -> Vec<ContractFixtureResult> {
    let Some(object) = contract.as_object() else {
        failures.push("contract must be a JSON object".to_string());
        return Vec::new();
    };

    let required = string_list(object_field(object, "required_separation_surfaces"));
    for surface in REQUIRED_SURFACES {
        require(
            required.iter().any(|item| item == surface),
            failures,
            format!("contract.required_separation_surfaces missing {surface}"),
        );
    }

    let fixtures_value = object_field(object, "fixtures");
    let Some(fixtures) = fixtures_value.and_then(Json::as_array) else {
        failures.push("contract.fixtures must be a list".to_string());
        return Vec::new();
    };

    let mut seen_green = false;
    let mut seen_red = false;
    let mut results = Vec::new();
    for (index, fixture) in fixtures.iter().enumerate() {
        let Some(fixture_object) = fixture.as_object() else {
            failures.push(format!("contract.fixtures[{index}] must be an object"));
            continue;
        };
        let fixture_id =
            string_field(fixture_object, "fixture_id").unwrap_or_else(|| format!("index-{index}"));
        let verdict = string_field(fixture_object, "expected_verdict").unwrap_or_default();
        if verdict == "GREEN" {
            seen_green = true;
            let expected_violations =
                string_list(object_field(fixture_object, "expected_violations"));
            let separated = string_list(object_field(fixture_object, "separate_surfaces"));
            let missing = missing_required_surfaces(&separated);
            require(
                expected_violations.is_empty(),
                failures,
                format!("{fixture_id}: GREEN fixture must not list expected violations"),
            );
            require(
                missing.is_empty(),
                failures,
                format!("{fixture_id}: GREEN fixture missing separated surfaces {missing:?}"),
            );
            require(
                string_field(fixture_object, "breakglass").is_some_and(|value| !value.is_empty()),
                failures,
                format!("{fixture_id}: GREEN fixture missing breakglass contract"),
            );
            require(
                string_field(fixture_object, "separation_model")
                    .is_some_and(|value| !value.is_empty()),
                failures,
                format!("{fixture_id}: GREEN fixture missing separation_model"),
            );
            results.push(ContractFixtureResult {
                fixture_id,
                expected_verdict: verdict,
                missing_surfaces: missing,
                shared_surfaces: Vec::new(),
            });
        } else if verdict == "RED" {
            seen_red = true;
            let expected_violations =
                string_list(object_field(fixture_object, "expected_violations"));
            let shared = string_list(object_field(fixture_object, "shared_surfaces"));
            require(
                !expected_violations.is_empty(),
                failures,
                format!("{fixture_id}: RED fixture must list expected violations"),
            );
            require(
                !shared.is_empty(),
                failures,
                format!("{fixture_id}: RED fixture must expose shared surfaces"),
            );
            require(
                object_field(fixture_object, "internal_bypass_without_breakglass")
                    .and_then(Json::as_bool)
                    .is_some_and(|value| value),
                failures,
                format!("{fixture_id}: RED fixture must cover internal bypass without breakglass"),
            );
            results.push(ContractFixtureResult {
                fixture_id,
                expected_verdict: verdict,
                missing_surfaces: Vec::new(),
                shared_surfaces: shared,
            });
        } else {
            failures.push(format!(
                "{fixture_id}: unsupported expected_verdict {verdict:?}"
            ));
        }
    }

    require(
        seen_green,
        failures,
        "contract must include a GREEN target fixture".to_string(),
    );
    require(
        seen_red,
        failures,
        "contract must include a RED negative fixture".to_string(),
    );
    results
}

fn validate_baseline_fixture(
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
            observed_tenant_violations: Vec::new(),
            tenant_surfaces_complete: false,
        };
    };
    let fixture_id = string_field(object, "fixture_id").unwrap_or_else(|| path.to_string());
    require(
        string_field(object, "expected_verdict").as_deref() == Some(expected_verdict),
        failures,
        format!("{fixture_id}: expected_verdict must be {expected_verdict}"),
    );

    let model_object = object_field(object, "tenant_pipeline_model").and_then(Json::as_object);
    if model_object.is_none() {
        failures.push(format!(
            "{fixture_id}: tenant_pipeline_model must be an object"
        ));
    }
    let empty = BTreeMap::new();
    let model = model_object.unwrap_or(&empty);
    let observed_violations = evaluate_tenant_model(model);
    let expected_violations = string_list(object_field(object, "expected_violations"));
    if expected_verdict == "GREEN" {
        require(
            observed_violations.is_empty(),
            failures,
            format!("{fixture_id}: GREEN tenant model has violations {observed_violations:?}"),
        );
        require(
            expected_violations.is_empty(),
            failures,
            format!("{fixture_id}: GREEN fixture must not list expected violations"),
        );
    } else {
        let tenant_violation_set = BTreeSet::from([
            "tenant_surface_separation_incomplete".to_string(),
            "tenant_surfaces_shared".to_string(),
            "internal_bypass_without_breakglass".to_string(),
        ]);
        let expected_set = expected_violations.iter().cloned().collect::<BTreeSet<_>>();
        let observed_set = observed_violations.iter().cloned().collect::<BTreeSet<_>>();
        require(
            !observed_violations.is_empty(),
            failures,
            format!("{fixture_id}: RED tenant model must produce tenant violations"),
        );
        require(
            tenant_violation_set.is_subset(&expected_set),
            failures,
            format!(
                "{fixture_id}: RED fixture expected_violations must include all tenant isolation violation classes"
            ),
        );
        require(
            observed_set.is_subset(&expected_set),
            failures,
            format!("{fixture_id}: observed tenant violations not listed in expected_violations"),
        );
    }

    let mut separated = string_list(object_field(model, "separate_surfaces"));
    separated.extend(string_list(object_field(model, "partitioned_surfaces")));
    BaselineFixtureResult {
        fixture_id,
        expected_verdict: expected_verdict.to_string(),
        observed_tenant_violations: observed_violations,
        tenant_surfaces_complete: missing_required_surfaces(&separated).is_empty(),
    }
}

pub fn evaluate_sources(
    contract: &Json,
    good_baseline_fixture: &Json,
    bad_baseline_fixture: &Json,
    contract_path: &str,
    good_baseline_path: &str,
    bad_baseline_path: &str,
) -> Report {
    let mut failures = Vec::new();
    let contract_fixture_results = validate_contract(contract, &mut failures);
    let baseline_fixture_results = vec![
        validate_baseline_fixture(
            good_baseline_fixture,
            good_baseline_path,
            "GREEN",
            &mut failures,
        ),
        validate_baseline_fixture(
            bad_baseline_fixture,
            bad_baseline_path,
            "RED",
            &mut failures,
        ),
    ];
    Report {
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
        contract: contract_path.to_string(),
        required_surfaces: REQUIRED_SURFACES
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
        contract_fixture_results,
        baseline_fixture_results,
        local_fixture_contract_proven: failures.is_empty(),
        live_required_context_execution_proven: false,
        tenant_facing_ready: false,
        security_ready: false,
        p0_0_green: false,
        phase0_complete: false,
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_string(),
        failures,
    }
}

pub fn evaluate_paths(
    contract_path: &str,
    good_baseline_path: &str,
    bad_baseline_path: &str,
) -> Result<Report, String> {
    let contract = load_json(contract_path)?;
    let good = load_json(good_baseline_path)?;
    let bad = load_json(bad_baseline_path)?;
    Ok(evaluate_sources(
        &contract,
        &good,
        &bad,
        contract_path,
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
            "\"contract\":{},",
            "\"contract_fixture_results\":{},",
            "\"failures\":{},",
            "\"live_required_context_execution_proven\":false,",
            "\"local_fixture_contract_proven\":{},",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"required_surfaces\":{},",
            "\"security_ready\":false,",
            "\"tenant_facing_ready\":false,",
            "\"verdict\":{}",
            "}}"
        ),
        json_string(&report.authority_boundary),
        baseline_results_json(&report.baseline_fixture_results),
        json_string(&report.contract),
        contract_results_json(&report.contract_fixture_results),
        string_array_json(&report.failures),
        bool_json(report.local_fixture_contract_proven),
        string_array_json(&report.required_surfaces),
        json_string(&report.verdict),
    )
}

fn contract_results_json(results: &[ContractFixtureResult]) -> String {
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
                        "\"missing_surfaces\":{},",
                        "\"shared_surfaces\":{}",
                        "}}"
                    ),
                    json_string(&result.expected_verdict),
                    json_string(&result.fixture_id),
                    string_array_json(&result.missing_surfaces),
                    string_array_json(&result.shared_surfaces),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
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
                        "\"observed_tenant_violations\":{},",
                        "\"tenant_surfaces_complete\":{}",
                        "}}"
                    ),
                    json_string(&result.expected_verdict),
                    json_string(&result.fixture_id),
                    string_array_json(&result.observed_tenant_violations),
                    bool_json(result.tenant_surfaces_complete),
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
    let mut contract = DEFAULT_CONTRACT.to_string();
    let mut good_baseline_fixture = DEFAULT_GOOD_BASELINE_FIXTURE.to_string();
    let mut bad_baseline_fixture = DEFAULT_BAD_BASELINE_FIXTURE.to_string();
    let mut emit_json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--contract" => contract = args.next().expect("--contract requires a value"),
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
    let report = evaluate_paths(&contract, &good_baseline_fixture, &bad_baseline_fixture)
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
