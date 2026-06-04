//! Validate AC-0.12 Phase-0 aggregate-exit fixtures locally.
//!
//! This checker is local/static fixture evidence only. It proves the aggregate
//! exit shape fails closed when any required Phase-0 subcondition is false,
//! missing, or unknown. It never posts statuses, mutates branch protection, or
//! claims P0.0 green / Phase-0 completion / production readiness.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const GOOD_FIXTURE: &str =
    "specs/fixtures/phase0-exit-gate/tc-0.12-good-all-subconditions-green.json";
const SINGLE_FALSE_FIXTURE: &str =
    "specs/fixtures/phase0-exit-gate/tc-0.12-bad-single-false-subconditions.json";
const DEFAULT_FIXTURES: &[&str] = &[
    "specs/fixtures/phase0-exit-gate/tc-0.12-good-all-subconditions-green.json",
    "specs/fixtures/phase0-exit-gate/tc-0.12-bad-single-false-subconditions.json",
    "specs/fixtures/phase0-exit-gate/tc-0.12-bad-missing-required-subcondition.json",
    "specs/fixtures/phase0-exit-gate/tc-0.12-current-red-p0-0-live-context-missing.json",
];

#[derive(Clone, Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    fn as_object(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Json::Object(value) => Some(value),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(value) => Some(value),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(value) => Some(value),
            _ => None,
        }
    }

    fn is_true(&self) -> bool {
        matches!(self, Json::Bool(true))
    }

    fn is_false(&self) -> bool {
        matches!(self, Json::Bool(false))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseResult {
    pub case_id: String,
    pub forced_false: Option<String>,
    pub observed_verdict: String,
    pub violations: Vec<String>,
    pub missing_subconditions: Vec<String>,
    pub false_or_non_true_subconditions: Vec<String>,
    pub case_passed: bool,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult {
    pub path: String,
    pub fixture_id: String,
    pub expected_verdict: String,
    pub observed_verdict: String,
    pub violations: Vec<String>,
    pub missing_subconditions: Vec<String>,
    pub false_or_non_true_subconditions: Vec<String>,
    pub case_results: Vec<CaseResult>,
    pub fixture_passed: bool,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub authority_boundary: String,
    pub required_subcondition_count: usize,
    pub required_subconditions: Vec<String>,
    pub single_false_case_count: usize,
    pub fixture_results: Vec<FixtureResult>,
    pub local_fixture_contract_proven: bool,
    pub aggregate_exit_local_static_proven: bool,
    pub aggregate_exit_live: bool,
    pub protected_branch_authority_proven: bool,
    pub status_mutation_performed: bool,
    pub live_required_context_execution_proven: bool,
    pub p0_0_green: bool,
    pub phase0_complete: bool,
    pub production_ready: bool,
    pub hyperscaler_grade: bool,
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
        if self.index == start {
            Err(format!("invalid JSON number at byte {start}"))
        } else {
            Ok(String::from_utf8_lossy(&self.text[start..self.index]).into_owned())
        }
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
        if self.try_consume(expected) {
            Ok(())
        } else {
            Err(format!(
                "expected {} at byte {}",
                expected as char, self.index
            ))
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

    fn peek(&self) -> Option<u8> {
        self.text.get(self.index).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.index += 1;
        Some(byte)
    }
}

fn parse_json_file(path: &Path) -> Result<BTreeMap<String, Json>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    match Parser::new(&text).parse()? {
        Json::Object(object) => Ok(object),
        _ => Err(format!("{}: expected JSON object", path.display())),
    }
}

fn get<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> Option<&'a Json> {
    object.get(key)
}

fn string_field(object: &BTreeMap<String, Json>, key: &str) -> Option<String> {
    get(object, key)
        .and_then(Json::as_str)
        .map(ToOwned::to_owned)
}

fn object_field<'a>(
    object: &'a BTreeMap<String, Json>,
    key: &str,
) -> Option<&'a BTreeMap<String, Json>> {
    get(object, key).and_then(Json::as_object)
}

fn object_array_field<'a>(
    object: &'a BTreeMap<String, Json>,
    key: &str,
) -> Vec<&'a BTreeMap<String, Json>> {
    get(object, key)
        .and_then(Json::as_array)
        .map(|items| items.iter().filter_map(Json::as_object).collect())
        .unwrap_or_default()
}

fn string_array_field(object: &BTreeMap<String, Json>, key: &str) -> Vec<String> {
    get(object, key)
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Json::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn json_string(input: &str) -> String {
    format!("\"{}\"", json_escape(input))
}

fn json_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn json_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn display_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|value| value.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn path_from(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

pub fn required_subconditions(root: &Path) -> Result<Vec<String>, String> {
    let good = parse_json_file(&path_from(root, GOOD_FIXTURE))?;
    let single_false = parse_json_file(&path_from(root, SINGLE_FALSE_FIXTURE))?;
    let mut good_names = object_field(&good, "subconditions")
        .map(|subconditions| subconditions.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    good_names.sort();
    let mut declared_names = string_array_field(&single_false, "subcondition_names");
    declared_names.sort();
    if good_names != declared_names {
        Err(format!(
            "aggregate fixture required subcondition lists diverge between {GOOD_FIXTURE} and {SINGLE_FALSE_FIXTURE}: good={good_names:?} declared={declared_names:?}"
        ))
    } else {
        Ok(declared_names)
    }
}

fn expected_verdict(fixture: &BTreeMap<String, Json>) -> String {
    match string_field(fixture, "expected_verdict").as_deref() {
        Some("GREEN") => "GREEN".to_string(),
        _ => "RED".to_string(),
    }
}

fn fixture_id(fixture: &BTreeMap<String, Json>, path: &Path) -> String {
    string_field(fixture, "fixture_id")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string())
        })
}

fn evaluate_subconditions(
    subconditions: Option<&BTreeMap<String, Json>>,
    required: &BTreeSet<String>,
) -> (bool, Vec<String>, Vec<String>, Vec<String>) {
    let Some(subconditions) = subconditions else {
        return (
            false,
            vec!["missing_or_malformed_subconditions".to_string()],
            required.iter().cloned().collect(),
            Vec::new(),
        );
    };
    let present: BTreeSet<String> = subconditions.keys().cloned().collect();
    let missing: Vec<String> = required.difference(&present).cloned().collect();
    let unknown: Vec<String> = present.difference(required).cloned().collect();
    let false_or_invalid: Vec<String> = required
        .intersection(&present)
        .filter(|name| !subconditions.get(*name).is_some_and(Json::is_true))
        .cloned()
        .collect();

    let mut violations = Vec::new();
    if !missing.is_empty() {
        violations.push("missing_required_subcondition".to_string());
    }
    if !unknown.is_empty() {
        violations.push("unknown_subcondition".to_string());
    }
    if !false_or_invalid.is_empty() {
        violations.push("false_or_non_true_subcondition".to_string());
    }
    (violations.is_empty(), violations, missing, false_or_invalid)
}

fn claim_boundary_violations(
    fixture: &BTreeMap<String, Json>,
    observed_green: bool,
) -> Vec<String> {
    let mut violations = Vec::new();
    if let Some(boundary) = object_field(fixture, "claim_boundary") {
        if get(boundary, "p0_0_green").is_some_and(Json::is_true)
            || get(boundary, "phase0_complete").is_some_and(Json::is_true)
        {
            violations.push("fixture_claims_current_phase0_green".to_string());
        }
        if !observed_green
            && (!get(boundary, "p0_0_green").is_some_and(Json::is_false)
                || !get(boundary, "phase0_complete").is_some_and(Json::is_false))
        {
            violations.push("red_fixture_missing_false_claim_boundary".to_string());
        }
    }
    violations
}

fn validate_case(case: &BTreeMap<String, Json>, required: &BTreeSet<String>) -> CaseResult {
    let (observed_green, violations, missing, false_or_invalid) =
        evaluate_subconditions(object_field(case, "subconditions"), required);
    let case_id = string_field(case, "case_id").unwrap_or_else(|| "unknown-case".to_string());
    let forced_false = string_field(case, "forced_false");
    let mut failures = Vec::new();

    if string_field(case, "expected_verdict").as_deref() == Some("RED") && observed_green {
        failures.push(format!("{case_id}: RED aggregate subcondition case passed"));
    }
    if let Some(forced) = &forced_false {
        if !required.contains(forced) {
            failures.push(format!(
                "{case_id}: forced_false is not a required subcondition"
            ));
        }
        if !false_or_invalid.contains(forced) {
            failures.push(format!(
                "{case_id}: forced_false was not observed false_or_non_true"
            ));
        }
        if violations != ["false_or_non_true_subcondition".to_string()]
            || false_or_invalid != [forced.clone()]
        {
            failures.push(format!(
                "{case_id}: single_false_case_not_exactly_one_false_subcondition forced_false={forced} observed_false_or_non_true={false_or_invalid:?} violations={violations:?}"
            ));
        }
    }

    CaseResult {
        case_id,
        forced_false,
        observed_verdict: if observed_green { "GREEN" } else { "RED" }.to_string(),
        violations,
        missing_subconditions: missing,
        false_or_non_true_subconditions: false_or_invalid,
        case_passed: failures.is_empty(),
        failures,
    }
}

fn is_single_false_path(path: &Path) -> bool {
    path.file_name()
        .map(|name| name == "tc-0.12-bad-single-false-subconditions.json")
        .unwrap_or(false)
}

pub fn validate_fixture(root: &Path, path: &Path, required_names: &[String]) -> FixtureResult {
    let required: BTreeSet<String> = required_names.iter().cloned().collect();
    let fixture = match parse_json_file(path) {
        Ok(fixture) => fixture,
        Err(error) => {
            return FixtureResult {
                path: display_path(path, root),
                fixture_id: path.to_string_lossy().to_string(),
                expected_verdict: "RED".to_string(),
                observed_verdict: "RED".to_string(),
                violations: vec![format!("fixture_parse_error:{error}")],
                missing_subconditions: Vec::new(),
                false_or_non_true_subconditions: Vec::new(),
                case_results: Vec::new(),
                fixture_passed: false,
                failures: vec![format!("fixture_parse_error:{error}")],
            };
        }
    };

    let fid = fixture_id(&fixture, path);
    let expected = expected_verdict(&fixture);
    let (observed_green, violations, missing, false_or_invalid) =
        evaluate_subconditions(object_field(&fixture, "subconditions"), &required);
    let mut failures = Vec::new();

    if expected == "GREEN" && !observed_green {
        failures.push(format!(
            "{fid}: GREEN aggregate fixture produced violations {violations:?}"
        ));
    }
    if expected == "RED" && observed_green {
        failures.push(format!("{fid}: RED aggregate fixture passed"));
    }

    let expected_false: BTreeSet<String> =
        string_array_field(&fixture, "expected_false_or_missing_subconditions")
            .into_iter()
            .collect();
    if !expected_false.is_empty() {
        let observed_false: BTreeSet<String> = missing
            .iter()
            .chain(false_or_invalid.iter())
            .cloned()
            .collect();
        let unobserved: Vec<String> = expected_false
            .difference(&observed_false)
            .cloned()
            .collect();
        if !unobserved.is_empty() {
            failures.push(format!(
                "{fid}: expected false/missing subconditions not observed {unobserved:?}"
            ));
        }
    }

    failures.extend(
        claim_boundary_violations(&fixture, observed_green)
            .into_iter()
            .map(|violation| format!("{fid}: {violation}")),
    );

    let mut case_results = Vec::new();
    if is_single_false_path(path)
        || fixture.contains_key("subcondition_names")
        || fixture.contains_key("cases")
    {
        let mut declared = string_array_field(&fixture, "subcondition_names");
        declared.sort();
        if declared != required_names {
            failures.push(format!(
                "{fid}: subcondition_names do not mirror required set"
            ));
        }
        let cases = object_array_field(&fixture, "cases");
        let forced: Vec<String> = cases
            .iter()
            .filter_map(|case| string_field(case, "forced_false"))
            .collect();
        let forced_set: BTreeSet<String> = forced.iter().cloned().collect();
        let missing_cases: Vec<String> = required.difference(&forced_set).cloned().collect();
        let duplicate_cases: Vec<String> = forced_set
            .iter()
            .filter(|name| forced.iter().filter(|item| *item == *name).count() > 1)
            .cloned()
            .collect();
        if !missing_cases.is_empty() {
            failures.push(format!(
                "{fid}: missing_case_for_required_subcondition {missing_cases:?}"
            ));
        }
        if !duplicate_cases.is_empty() {
            failures.push(format!(
                "{fid}: duplicate_case_for_required_subcondition {duplicate_cases:?}"
            ));
        }
        for case in cases {
            let result = validate_case(case, &required);
            failures.extend(result.failures.clone());
            case_results.push(result);
        }
    }

    FixtureResult {
        path: display_path(path, root),
        fixture_id: fid,
        expected_verdict: expected,
        observed_verdict: if observed_green { "GREEN" } else { "RED" }.to_string(),
        violations,
        missing_subconditions: missing,
        false_or_non_true_subconditions: false_or_invalid,
        case_results,
        fixture_passed: failures.is_empty(),
        failures,
    }
}

pub fn evaluate(root: &Path, fixture_overrides: &[PathBuf]) -> Evaluation {
    let (required_names, mut failures) = match required_subconditions(root) {
        Ok(names) => (names, Vec::new()),
        Err(error) => (Vec::new(), vec![error]),
    };
    let fixture_paths: Vec<PathBuf> = if fixture_overrides.is_empty() {
        DEFAULT_FIXTURES
            .iter()
            .map(|path| path_from(root, path))
            .collect()
    } else {
        fixture_overrides
            .iter()
            .map(|path| {
                if path.is_absolute() {
                    path.clone()
                } else {
                    root.join(path)
                }
            })
            .collect()
    };
    let results: Vec<FixtureResult> = fixture_paths
        .iter()
        .map(|path| validate_fixture(root, path, &required_names))
        .collect();
    for result in &results {
        failures.extend(result.failures.clone());
    }
    let case_count = results
        .iter()
        .find(|result| {
            result
                .path
                .ends_with("tc-0.12-bad-single-false-subconditions.json")
        })
        .map(|result| result.case_results.len())
        .unwrap_or(0);
    let unique_failures: Vec<String> = failures
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let pass = unique_failures.is_empty();

    Evaluation {
        authority_boundary: "AC-0.12 aggregate-exit local/static fixture evidence only; this checker never posts statuses, mutates branch protection, or claims live Phase-0 completion".to_string(),
        required_subcondition_count: required_names.len(),
        required_subconditions: required_names,
        single_false_case_count: case_count,
        fixture_results: results,
        local_fixture_contract_proven: pass,
        aggregate_exit_local_static_proven: pass,
        aggregate_exit_live: false,
        protected_branch_authority_proven: false,
        status_mutation_performed: false,
        live_required_context_execution_proven: false,
        p0_0_green: false,
        phase0_complete: false,
        production_ready: false,
        hyperscaler_grade: false,
        verdict: if pass { "PASS" } else { "FAIL" }.to_string(),
        failures: unique_failures,
    }
}

fn case_result_json(result: &CaseResult) -> String {
    format!(
        concat!(
            "{{",
            "\"case_id\":{},",
            "\"case_passed\":{},",
            "\"failures\":{},",
            "\"false_or_non_true_subconditions\":{},",
            "\"forced_false\":{},",
            "\"missing_subconditions\":{},",
            "\"observed_verdict\":{},",
            "\"violations\":{}",
            "}}"
        ),
        json_string(&result.case_id),
        json_bool(result.case_passed),
        json_string_array(&result.failures),
        json_string_array(&result.false_or_non_true_subconditions),
        result
            .forced_false
            .as_ref()
            .map(|value| json_string(value))
            .unwrap_or_else(|| "null".to_string()),
        json_string_array(&result.missing_subconditions),
        json_string(&result.observed_verdict),
        json_string_array(&result.violations),
    )
}

fn fixture_result_json(result: &FixtureResult) -> String {
    let cases = result
        .case_results
        .iter()
        .map(case_result_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"case_results\":[{}],",
            "\"expected_verdict\":{},",
            "\"failures\":{},",
            "\"false_or_non_true_subconditions\":{},",
            "\"fixture_id\":{},",
            "\"fixture_passed\":{},",
            "\"missing_subconditions\":{},",
            "\"observed_verdict\":{},",
            "\"path\":{},",
            "\"violations\":{}",
            "}}"
        ),
        cases,
        json_string(&result.expected_verdict),
        json_string_array(&result.failures),
        json_string_array(&result.false_or_non_true_subconditions),
        json_string(&result.fixture_id),
        json_bool(result.fixture_passed),
        json_string_array(&result.missing_subconditions),
        json_string(&result.observed_verdict),
        json_string(&result.path),
        json_string_array(&result.violations),
    )
}

pub fn to_json(evaluation: &Evaluation) -> String {
    let fixtures = evaluation
        .fixture_results
        .iter()
        .map(fixture_result_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"aggregate_exit_live\":{},",
            "\"aggregate_exit_local_static_proven\":{},",
            "\"authority_boundary\":{},",
            "\"failures\":{},",
            "\"fixture_results\":[{}],",
            "\"hyperscaler_grade\":{},",
            "\"live_required_context_execution_proven\":{},",
            "\"local_fixture_contract_proven\":{},",
            "\"p0_0_green\":{},",
            "\"phase0_complete\":{},",
            "\"production_ready\":{},",
            "\"protected_branch_authority_proven\":{},",
            "\"required_subcondition_count\":{},",
            "\"required_subconditions\":{},",
            "\"single_false_case_count\":{},",
            "\"status_mutation_performed\":{},",
            "\"verdict\":{}",
            "}}"
        ),
        json_bool(evaluation.aggregate_exit_live),
        json_bool(evaluation.aggregate_exit_local_static_proven),
        json_string(&evaluation.authority_boundary),
        json_string_array(&evaluation.failures),
        fixtures,
        json_bool(evaluation.hyperscaler_grade),
        json_bool(evaluation.live_required_context_execution_proven),
        json_bool(evaluation.local_fixture_contract_proven),
        json_bool(evaluation.p0_0_green),
        json_bool(evaluation.phase0_complete),
        json_bool(evaluation.production_ready),
        json_bool(evaluation.protected_branch_authority_proven),
        evaluation.required_subcondition_count,
        json_string_array(&evaluation.required_subconditions),
        evaluation.single_false_case_count,
        json_bool(evaluation.status_mutation_performed),
        json_string(&evaluation.verdict),
    )
}

fn usage() -> &'static str {
    "usage: assert-phase0-aggregate-exit [--repo-root PATH] [--fixture PATH ...] [--json]"
}

pub fn run_cli() -> i32 {
    let mut repo_root = PathBuf::from(".");
    let mut fixtures = Vec::new();
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = args.next() else {
                    eprintln!("{}", usage());
                    return 2;
                };
                repo_root = PathBuf::from(value);
            }
            "--fixture" => {
                let Some(value) = args.next() else {
                    eprintln!("{}", usage());
                    return 2;
                };
                fixtures.push(PathBuf::from(value));
            }
            "--json" => json = true,
            "-h" | "--help" => {
                println!("{}", usage());
                return 0;
            }
            other => {
                eprintln!("unexpected argument: {other}\n{}", usage());
                return 2;
            }
        }
    }
    let root = repo_root.canonicalize().unwrap_or(repo_root);
    let evaluation = evaluate(&root, &fixtures);
    let rendered = to_json(&evaluation);
    if json || evaluation.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if evaluation.verdict == "PASS" { 0 } else { 1 }
}

#[cfg(not(test))]
fn main() {
    std::process::exit(run_cli());
}
