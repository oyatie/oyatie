//! Fail closed when the P0.0 CI-enforcement baseline omits executable fixtures.
//!
//! This checker is local/static fixture evidence only. It validates that every
//! checked-in Phase-0 baseline fixture is cataloged, RED/GREEN cases remain
//! paired and reachable through Buck2-owned gates, required-context metadata is
//! current, and the P0.0/Phase-0 claim boundary remains red. It never mutates
//! branch protection, posts statuses, or claims live required-context authority.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const BASELINE_DEFAULT: &str = "specs/phase0-ci-enforcement-baseline.json";
const MATRIX_PATH: &str = "specs/phase0-automation-matrix.json";
const COVERAGE_REGISTRY_PATH: &str = "specs/phase0-automation-coverage-registry.json";
const REQUIRED_CONTEXT_ROW_ID: &str = "AC-0.0-cloud-ci-required-context";
const REQUIRED_CONTEXT_VERIFICATION_COMMAND: &str = "buck2 build //:phase0-ci-enforcement-baseline-catalog-check //:phase0-required-status-source-check //:phase0-trusted-target-inventory-check //:phase0-result-bundle-output-check";
const REQUIRED_CONTEXT_TARGET_TERMS: &[&str] = &[
    "cloud-ci-required / oya-ci-required branch-protection context",
    "//:phase0-ci-enforcement-baseline-catalog-check",
    "//:phase0-required-status-source-check",
    "//:phase0-trusted-target-inventory-check",
    "//:phase0-result-bundle-output-check",
];
const REQUIRED_CONTEXT_CLAIM_BOUNDARY_TERMS: &[&str] = &[
    "live-read-only RED evidence",
    "not P0.0 green",
    "not protected-branch authority",
    "trusted cloud-ci/oya-ci",
];
const STALE_REQUIRED_CONTEXT_PHRASES: &[&str] = &[
    "live GitHub branch protection still lacks cloud-ci-required/oya-ci-required",
    "current branch protection still lacks cloud-ci-required/oya-ci-required",
];
const REQUIRED_CONTEXT_NARRATIVE_DOCS: &[&str] = &[
    "specs/phase0-automation-matrix.json",
    "specs/phase0-claim-evidence-map.json",
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
pub struct Config {
    pub repo_root: PathBuf,
    pub baseline: PathBuf,
    pub matrix: PathBuf,
    pub coverage_registry: PathBuf,
    pub required_context_docs: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub baseline: String,
    pub fixture_directory: String,
    pub fixture_count: usize,
    pub required_red_green_pairs: usize,
    pub result_bundle_fixtures: usize,
    pub result_bundle_check_target: String,
    pub trusted_target_inventory_fixtures: usize,
    pub trusted_target_inventory_check_target: String,
    pub required_status_source_fixtures: usize,
    pub local_fixture_contract_proven: bool,
    pub live_required_context_execution_proven: bool,
    pub protected_branch_authority_proven: bool,
    pub status_mutation_performed: bool,
    pub p0_0_green: bool,
    pub phase0_complete: bool,
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

pub fn default_config(repo_root: PathBuf) -> Config {
    Config {
        baseline: repo_root.join(BASELINE_DEFAULT),
        matrix: repo_root.join(MATRIX_PATH),
        coverage_registry: repo_root.join(COVERAGE_REGISTRY_PATH),
        required_context_docs: REQUIRED_CONTEXT_NARRATIVE_DOCS
            .iter()
            .map(|path| repo_root.join(path))
            .collect(),
        repo_root,
    }
}

pub fn evaluate(config: &Config) -> Evaluation {
    let mut evaluation = Evaluation {
        verdict: "FAIL".to_string(),
        baseline: rel(&config.repo_root, &config.baseline),
        fixture_directory: String::new(),
        fixture_count: 0,
        required_red_green_pairs: 0,
        result_bundle_fixtures: 0,
        result_bundle_check_target: String::new(),
        trusted_target_inventory_fixtures: 0,
        trusted_target_inventory_check_target: String::new(),
        required_status_source_fixtures: 0,
        local_fixture_contract_proven: false,
        live_required_context_execution_proven: false,
        protected_branch_authority_proven: false,
        status_mutation_performed: false,
        p0_0_green: false,
        phase0_complete: false,
        failures: Vec::new(),
    };

    let baseline = match load_json(&config.baseline) {
        Ok(value) => value,
        Err(error) => {
            evaluation
                .failures
                .push(format!("{}: {error}", evaluation.baseline));
            return evaluation;
        }
    };
    let Some(baseline_obj) = baseline.as_object() else {
        evaluation
            .failures
            .push("baseline: expected object".to_string());
        return evaluation;
    };
    let Some(fixture_set) = object_field(baseline_obj, "fixture_set", &mut evaluation.failures)
    else {
        finalize(&mut evaluation);
        return evaluation;
    };

    let fixture_dir = string_field(fixture_set, "fixture_directory", &mut evaluation.failures)
        .map(|path| repo_path(&config.repo_root, path))
        .unwrap_or_else(|| config.repo_root.join("<missing>"));
    evaluation.fixture_directory = rel(&config.repo_root, &fixture_dir);

    let actual_fixture_paths =
        list_json_files(&config.repo_root, &fixture_dir, &mut evaluation.failures);
    evaluation.fixture_count = actual_fixture_paths.len();
    let listed_fixture_paths = string_list_field(
        fixture_set,
        "all_fixture_paths",
        "fixture_set.all_fixture_paths",
        &mut evaluation.failures,
    );
    let listed_set: BTreeSet<String> = listed_fixture_paths.iter().cloned().collect();
    let actual_set: BTreeSet<String> = actual_fixture_paths.iter().cloned().collect();

    for item in actual_set.difference(&listed_set) {
        evaluation.failures.push(format!(
            "fixture_set.all_fixture_paths missing actual fixture: {item}"
        ));
    }
    for item in listed_set.difference(&actual_set) {
        evaluation.failures.push(format!(
            "fixture_set.all_fixture_paths lists missing fixture: {item}"
        ));
    }

    for field in [
        "current_red_result_fixture",
        "result_schema",
        "override_packet_schema",
        "trusted_target_inventory_schema",
        "tenant_fixture_contract",
    ] {
        let Some(value) = string_field(fixture_set, field, &mut evaluation.failures) else {
            continue;
        };
        let path = repo_path(&config.repo_root, value);
        if !path.is_file() {
            evaluation
                .failures
                .push(format!("fixture_set.{field}: missing path {value}"));
        }
        if path.starts_with(&fixture_dir) && !listed_set.contains(value) {
            evaluation.failures.push(format!(
                "fixture_set.{field}: {value} is not included in all_fixture_paths"
            ));
        }
    }

    for field in [
        "result_bundle_fixture_paths",
        "trusted_target_inventory_fixture_paths",
    ] {
        for value in string_list_field(
            fixture_set,
            field,
            &format!("fixture_set.{field}"),
            &mut evaluation.failures,
        ) {
            let path = repo_path(&config.repo_root, &value);
            if !path.is_file() {
                evaluation
                    .failures
                    .push(format!("fixture_set.{field}: missing path {value}"));
            }
            if path.starts_with(&fixture_dir) && !listed_set.contains(&value) {
                evaluation.failures.push(format!(
                    "fixture_set.{field}: {value} is not included in all_fixture_paths"
                ));
            }
        }
    }

    let fixture_by_basename: BTreeMap<String, String> = actual_fixture_paths
        .iter()
        .map(|path| (basename(path), path.clone()))
        .collect();
    let mut red_pair_basenames = BTreeSet::new();
    let mut reachable_fixture_basenames = BTreeSet::new();
    let pairs = array_field(
        fixture_set,
        "required_red_green_pairs",
        "fixture_set.required_red_green_pairs",
        &mut evaluation.failures,
    );
    evaluation.required_red_green_pairs = pairs.len();
    for (index, pair_json) in pairs.iter().enumerate() {
        let Some(pair) = pair_json.as_object() else {
            evaluation.failures.push(format!(
                "required_red_green_pairs[{index}]: expected object"
            ));
            continue;
        };
        let tc_id = optional_string(pair, "tc_id");
        let red = optional_string(pair, "red");
        let green = optional_string(pair, "green");
        for (key, value) in [("tc_id", tc_id), ("red", red), ("green", green)] {
            if value.is_none_or(str::is_empty) {
                evaluation.failures.push(format!(
                    "required_red_green_pairs[{index}].{key}: expected non-empty string"
                ));
            }
        }
        let (Some(red), Some(green)) = (red, green) else {
            continue;
        };
        let Some(red_path) = fixture_by_basename.get(red) else {
            evaluation.failures.push(format!(
                "required_red_green_pairs[{index}].red: unknown fixture {red}"
            ));
            continue;
        };
        let Some(green_path) = fixture_by_basename.get(green) else {
            evaluation.failures.push(format!(
                "required_red_green_pairs[{index}].green: unknown fixture {green}"
            ));
            continue;
        };
        red_pair_basenames.insert(red.to_string());
        reachable_fixture_basenames.insert(red.to_string());
        reachable_fixture_basenames.insert(green.to_string());
        let red_verdict = fixture_verdict(&config.repo_root, red_path, &mut evaluation.failures);
        let green_verdict =
            fixture_verdict(&config.repo_root, green_path, &mut evaluation.failures);
        if red_verdict.as_deref() != Some("RED") {
            evaluation.failures.push(format!(
                "{red}: red pair fixture must declare expected_verdict=RED, got {red_verdict:?}"
            ));
        }
        if green_verdict.as_deref() != Some("GREEN") {
            evaluation.failures.push(format!(
                "{green}: green pair fixture must declare expected_verdict=GREEN, got {green_verdict:?}"
            ));
        }
        if !tc_id.is_some_and(|value| value.starts_with("TC-0.0")) {
            evaluation.failures.push(format!(
                "required_red_green_pairs[{index}].tc_id: expected TC-0.0* id"
            ));
        }
    }

    for field in [
        "result_bundle_fixture_paths",
        "trusted_target_inventory_fixture_paths",
    ] {
        for value in string_list_field(
            fixture_set,
            field,
            &format!("fixture_set.{field}"),
            &mut Vec::new(),
        ) {
            reachable_fixture_basenames.insert(basename(&value));
        }
    }
    if let Some(current_red) = optional_string(fixture_set, "current_red_result_fixture") {
        reachable_fixture_basenames.insert(basename(current_red));
    }

    for path in &actual_fixture_paths {
        let base = basename(path);
        let data = match load_json(&repo_path(&config.repo_root, path)) {
            Ok(value) => value,
            Err(error) => {
                evaluation.failures.push(format!("{path}: {error}"));
                continue;
            }
        };
        let expected_verdict = data
            .as_object()
            .and_then(|object| optional_string(object, "expected_verdict"));
        if expected_verdict == Some("RED") && !red_pair_basenames.contains(&base) {
            evaluation.failures.push(format!(
                "{base}: RED expected_verdict fixture is not covered by required_red_green_pairs"
            ));
        }
        if matches!(expected_verdict, Some("RED" | "GREEN"))
            && !reachable_fixture_basenames.contains(&base)
        {
            evaluation.failures.push(format!(
                "{base}: expected_verdict fixture is not reachable from any catalog execution bucket"
            ));
        }
    }

    if let Some(current_red) = optional_string(fixture_set, "current_red_result_fixture") {
        let current_red_path = repo_path(&config.repo_root, current_red);
        if current_red_path.is_file() {
            match load_json(&current_red_path) {
                Ok(data) => {
                    let object = data.as_object();
                    if object.and_then(|item| optional_string(item, "observed_verdict"))
                        != Some("RED")
                    {
                        evaluation.failures.push(format!(
                            "{current_red}: current RED gap result must keep observed_verdict=RED"
                        ));
                    }
                    let boundary = object.and_then(|item| item.get("claim_boundary"));
                    if !object_bool_is(boundary, "p0_0_green", false)
                        || !object_bool_is(boundary, "phase0_complete", false)
                    {
                        evaluation.failures.push(format!(
                            "{current_red}: current RED gap result must declare p0_0_green=false and phase0_complete=false"
                        ));
                    }
                }
                Err(error) => evaluation.failures.push(format!("{current_red}: {error}")),
            }
        }
    }

    let result_bundle_paths = string_list_field(
        fixture_set,
        "result_bundle_fixture_paths",
        "fixture_set.result_bundle_fixture_paths",
        &mut evaluation.failures,
    );
    evaluation.result_bundle_fixtures = result_bundle_paths.len();
    evaluation.trusted_target_inventory_fixtures = string_list_field(
        fixture_set,
        "trusted_target_inventory_fixture_paths",
        "fixture_set.trusted_target_inventory_fixture_paths",
        &mut Vec::new(),
    )
    .len();
    let false_green_paths: Vec<&String> = result_bundle_paths
        .iter()
        .filter(|path| path.contains("false-green"))
        .collect();
    if false_green_paths.is_empty() {
        evaluation.failures.push(
            "fixture_set.result_bundle_fixture_paths: missing false-green result-bundle fixture"
                .to_string(),
        );
    }
    for value in false_green_paths {
        match load_json(&repo_path(&config.repo_root, value)) {
            Ok(data) => {
                let boundary = data
                    .as_object()
                    .and_then(|object| object.get("claim_boundary"));
                if !object_bool_is(boundary, "p0_0_green", true)
                    || !object_bool_is(boundary, "phase0_complete", true)
                {
                    evaluation.failures.push(format!(
                        "{value}: false-green fixture must exercise p0_0_green=true and phase0_complete=true"
                    ));
                }
            }
            Err(error) => evaluation.failures.push(format!("{value}: {error}")),
        }
    }

    let claim_boundary = baseline_obj.get("claim_boundary");
    if !object_bool_is(claim_boundary, "p0_0_green", false) {
        evaluation
            .failures
            .push("claim_boundary.p0_0_green must remain false in the RED gap packet".to_string());
    }
    if !object_bool_is(claim_boundary, "phase0_complete", false) {
        evaluation.failures.push(
            "claim_boundary.phase0_complete must remain false in the RED gap packet".to_string(),
        );
    }

    if let Some(automation_mapping) =
        object_field(baseline_obj, "automation_mapping", &mut evaluation.failures)
    {
        check_expected_string(
            automation_mapping,
            "source_app_binding_check_target",
            "//:phase0-required-status-source-check",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "source_app_binding_check_script",
            "scripts/ci/assert-required-status-source.rs",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "source_app_binding_test",
            "scripts/tests/phase0_required_status_source_check.rs",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "tenant_isolation_check_target",
            "//:phase0-tenant-isolation-fixture-check",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "tenant_isolation_check_script",
            "scripts/ci/assert-tenant-pipeline-isolation.py",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "tenant_isolation_test",
            "scripts/tests/phase0_tenant_isolation_fixture_check.test.sh",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "override_kill_switch_check_target",
            "//:phase0-override-kill-switch-check",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "override_kill_switch_check_script",
            "scripts/ci/assert-override-kill-switch.py",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "override_kill_switch_test",
            "scripts/tests/phase0_override_kill_switch_check.test.sh",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "trusted_target_inventory_check_target",
            "//:phase0-trusted-target-inventory-check",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "trusted_target_inventory_check_script",
            "scripts/ci/assert-trusted-target-inventory.py",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "trusted_target_inventory_test",
            "scripts/tests/phase0_trusted_target_inventory_check.test.sh",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "result_bundle_check_target",
            "//:phase0-result-bundle-output-check",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "result_bundle_check_script",
            "scripts/ci/assert-result-bundle-output.py",
            &mut evaluation.failures,
        );
        check_expected_string(
            automation_mapping,
            "result_bundle_test",
            "scripts/tests/phase0_result_bundle_output_check.test.sh",
            &mut evaluation.failures,
        );
        evaluation.result_bundle_check_target =
            optional_string(automation_mapping, "result_bundle_check_target")
                .unwrap_or_default()
                .to_string();
        evaluation.trusted_target_inventory_check_target =
            optional_string(automation_mapping, "trusted_target_inventory_check_target")
                .unwrap_or_default()
                .to_string();
    }

    check_required_context_matrix(config, &mut evaluation.failures);
    check_coverage_registry(config, &mut evaluation.failures);
    check_stale_required_context_docs(config, &mut evaluation.failures);
    check_required_status_source_fixtures(config, fixture_set, &mut evaluation);

    finalize(&mut evaluation);
    evaluation
}

fn finalize(evaluation: &mut Evaluation) {
    if evaluation.failures.is_empty() {
        evaluation.verdict = "PASS".to_string();
        evaluation.local_fixture_contract_proven = true;
    } else {
        evaluation.verdict = "FAIL".to_string();
        evaluation.local_fixture_contract_proven = false;
    }
    evaluation.live_required_context_execution_proven = false;
    evaluation.protected_branch_authority_proven = false;
    evaluation.status_mutation_performed = false;
    evaluation.p0_0_green = false;
    evaluation.phase0_complete = false;
}

fn check_required_context_matrix(config: &Config, failures: &mut Vec<String>) {
    let matrix = match load_json(&config.matrix) {
        Ok(value) => value,
        Err(error) => {
            failures.push(format!(
                "{}: {error}",
                rel(&config.repo_root, &config.matrix)
            ));
            return;
        }
    };
    let Some(matrix_obj) = matrix.as_object() else {
        failures.push(format!("{MATRIX_PATH}: expected object"));
        return;
    };
    let row = find_by_id(
        matrix_obj.get("seed_rows"),
        REQUIRED_CONTEXT_ROW_ID,
        &format!("{MATRIX_PATH}.seed_rows"),
        failures,
    );
    let Some(row) = row else { return };
    if optional_string(row, "verification_command") != Some(REQUIRED_CONTEXT_VERIFICATION_COMMAND) {
        failures.push(format!(
            "{REQUIRED_CONTEXT_ROW_ID} row must record combined Buck2 local verification command"
        ));
    }
    match optional_string(row, "target_gate_or_controller") {
        Some(target) => {
            for term in REQUIRED_CONTEXT_TARGET_TERMS {
                if !target.contains(term) {
                    failures.push(format!(
                        "{REQUIRED_CONTEXT_ROW_ID}.target_gate_or_controller missing {term:?}"
                    ));
                }
            }
        }
        None => failures.push(format!(
            "{REQUIRED_CONTEXT_ROW_ID}.target_gate_or_controller: expected string"
        )),
    }
    match optional_string(row, "claim_boundary") {
        Some(boundary) => {
            for term in REQUIRED_CONTEXT_CLAIM_BOUNDARY_TERMS {
                if !boundary.contains(term) {
                    failures.push(format!(
                        "{REQUIRED_CONTEXT_ROW_ID}.claim_boundary missing {term:?}"
                    ));
                }
            }
        }
        None => failures.push(format!(
            "{REQUIRED_CONTEXT_ROW_ID}.claim_boundary: expected string"
        )),
    }
    if !row.get("no_new_oya_cli_surface").is_some_and(Json::is_true) {
        failures.push(format!(
            "{REQUIRED_CONTEXT_ROW_ID}.no_new_oya_cli_surface must be true"
        ));
    }
}

fn check_coverage_registry(config: &Config, failures: &mut Vec<String>) {
    let registry = match load_json(&config.coverage_registry) {
        Ok(value) => value,
        Err(error) => {
            failures.push(format!(
                "{}: {error}",
                rel(&config.repo_root, &config.coverage_registry)
            ));
            return;
        }
    };
    let Some(registry_obj) = registry.as_object() else {
        failures.push(format!("{COVERAGE_REGISTRY_PATH}: expected object"));
        return;
    };
    let subject = find_by_id(
        registry_obj.get("coverage_subjects"),
        "AC-0.0",
        &format!("{COVERAGE_REGISTRY_PATH}.coverage_subjects"),
        failures,
    );
    let Some(subject) = subject else { return };
    let mapped_rows = subject.get("mapped_row_ids").and_then(Json::as_array);
    if !mapped_rows.is_some_and(|rows| {
        rows.iter()
            .any(|row| row.as_str() == Some(REQUIRED_CONTEXT_ROW_ID))
    }) {
        failures.push(format!(
            "AC-0.0 coverage subject must map {REQUIRED_CONTEXT_ROW_ID}"
        ));
    }
    match subject
        .get("verification_commands")
        .and_then(Json::as_object)
    {
        Some(commands) => {
            if optional_string(commands, REQUIRED_CONTEXT_ROW_ID)
                != Some(REQUIRED_CONTEXT_VERIFICATION_COMMAND)
            {
                failures.push(format!(
                    "AC-0.0 coverage subject must record combined Buck2 command for {REQUIRED_CONTEXT_ROW_ID}"
                ));
            }
        }
        None => failures
            .push("AC-0.0 coverage subject verification_commands must be an object".to_string()),
    }
}

fn check_stale_required_context_docs(config: &Config, failures: &mut Vec<String>) {
    for path in &config.required_context_docs {
        if !path.is_file() {
            failures.push(format!(
                "required-context narrative doc missing: {}",
                rel(&config.repo_root, path)
            ));
            continue;
        }
        let text = match fs::read_to_string(path) {
            Ok(value) => value,
            Err(error) => {
                failures.push(format!("{}: {error}", rel(&config.repo_root, path)));
                continue;
            }
        };
        for phrase in STALE_REQUIRED_CONTEXT_PHRASES {
            if text.contains(phrase) {
                failures.push(format!(
                    "{}: stale required-context gap phrase must be replaced with current context-present/source-app-unbound wording",
                    rel(&config.repo_root, path)
                ));
            }
        }
    }
}

fn check_required_status_source_fixtures(
    config: &Config,
    fixture_set: &BTreeMap<String, Json>,
    evaluation: &mut Evaluation,
) {
    let source_fixture_dir = string_field(
        fixture_set,
        "required_status_source_fixture_directory",
        &mut evaluation.failures,
    )
    .map(|path| repo_path(&config.repo_root, path))
    .unwrap_or_else(|| config.repo_root.join("<missing-required-status-source>"));
    let actual_source_fixture_paths = list_json_files(
        &config.repo_root,
        &source_fixture_dir,
        &mut evaluation.failures,
    );
    evaluation.required_status_source_fixtures = actual_source_fixture_paths.len();
    let listed_source_fixture_paths = string_list_field(
        fixture_set,
        "required_status_source_fixture_paths",
        "fixture_set.required_status_source_fixture_paths",
        &mut evaluation.failures,
    );
    let actual_set: BTreeSet<String> = actual_source_fixture_paths.iter().cloned().collect();
    let listed_set: BTreeSet<String> = listed_source_fixture_paths.iter().cloned().collect();
    for item in actual_set.difference(&listed_set) {
        evaluation.failures.push(format!(
            "fixture_set.required_status_source_fixture_paths missing actual fixture: {item}"
        ));
    }
    for item in listed_set.difference(&actual_set) {
        evaluation.failures.push(format!(
            "fixture_set.required_status_source_fixture_paths lists missing fixture: {item}"
        ));
    }
    for value in listed_source_fixture_paths {
        let path = repo_path(&config.repo_root, &value);
        if !path.is_file() {
            evaluation.failures.push(format!(
                "fixture_set.required_status_source_fixture_paths: missing path {value}"
            ));
            continue;
        }
        match load_json(&path) {
            Ok(data) => {
                if data
                    .as_object()
                    .and_then(|object| object.get("contexts"))
                    .is_none()
                {
                    evaluation.failures.push(format!(
                        "{value}: required-status source fixture must include contexts"
                    ));
                }
            }
            Err(error) => evaluation.failures.push(format!("{value}: {error}")),
        }
    }
}

fn find_by_id<'a>(
    value: Option<&'a Json>,
    item_id: &str,
    field: &str,
    failures: &mut Vec<String>,
) -> Option<&'a BTreeMap<String, Json>> {
    let Some(items) = value.and_then(Json::as_array) else {
        failures.push(format!("{field}: expected list"));
        return None;
    };
    let matches: Vec<&BTreeMap<String, Json>> = items
        .iter()
        .filter_map(Json::as_object)
        .filter(|item| optional_string(item, "id") == Some(item_id))
        .collect();
    if matches.len() != 1 {
        failures.push(format!(
            "{field}: expected exactly one {item_id} entry, found {}",
            matches.len()
        ));
        return None;
    }
    matches.first().copied()
}

fn object_field<'a>(
    object: &'a BTreeMap<String, Json>,
    field: &str,
    failures: &mut Vec<String>,
) -> Option<&'a BTreeMap<String, Json>> {
    match object.get(field).and_then(Json::as_object) {
        Some(value) => Some(value),
        None => {
            failures.push(format!("{field}: expected object"));
            None
        }
    }
}

fn array_field<'a>(
    object: &'a BTreeMap<String, Json>,
    field: &str,
    label: &str,
    failures: &mut Vec<String>,
) -> Vec<&'a Json> {
    match object.get(field).and_then(Json::as_array) {
        Some(value) => value.iter().collect(),
        None => {
            failures.push(format!("{label}: expected list"));
            Vec::new()
        }
    }
}

fn string_field<'a>(
    object: &'a BTreeMap<String, Json>,
    field: &str,
    failures: &mut Vec<String>,
) -> Option<&'a str> {
    match object.get(field).and_then(Json::as_str) {
        Some(value) => Some(value),
        None => {
            failures.push(format!("fixture_set.{field}: expected string"));
            None
        }
    }
}

fn string_list_field(
    object: &BTreeMap<String, Json>,
    field: &str,
    label: &str,
    failures: &mut Vec<String>,
) -> Vec<String> {
    let Some(items) = object.get(field).and_then(Json::as_array) else {
        failures.push(format!("{label}: expected list"));
        return Vec::new();
    };
    let mut out = Vec::new();
    for (index, item) in items.iter().enumerate() {
        match item.as_str() {
            Some(value) => out.push(value.to_string()),
            None => failures.push(format!("{label}[{index}]: expected string")),
        }
    }
    out
}

fn optional_string<'a>(object: &'a BTreeMap<String, Json>, field: &str) -> Option<&'a str> {
    object.get(field).and_then(Json::as_str)
}

fn check_expected_string(
    object: &BTreeMap<String, Json>,
    field: &str,
    expected: &str,
    failures: &mut Vec<String>,
) {
    if optional_string(object, field) != Some(expected) {
        failures.push(format!("automation_mapping.{field} must be {expected}"));
    }
}

fn object_bool_is(value: Option<&Json>, field: &str, expected: bool) -> bool {
    let Some(object) = value.and_then(Json::as_object) else {
        return false;
    };
    match object.get(field) {
        Some(value) if expected => value.is_true(),
        Some(value) => value.is_false(),
        None => false,
    }
}

fn fixture_verdict(repo_root: &Path, path: &str, failures: &mut Vec<String>) -> Option<String> {
    match load_json(&repo_path(repo_root, path)) {
        Ok(data) => data
            .as_object()
            .and_then(|object| optional_string(object, "expected_verdict"))
            .map(str::to_string),
        Err(error) => {
            failures.push(format!("{path}: {error}"));
            None
        }
    }
}

fn list_json_files(repo_root: &Path, dir: &Path, failures: &mut Vec<String>) -> Vec<String> {
    let mut paths = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            failures.push(format!("{}: {error}", rel(repo_root, dir)));
            return paths;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                failures.push(format!("{}: {error}", rel(repo_root, dir)));
                continue;
            }
        };
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
            paths.push(rel(repo_root, &path));
        }
    }
    paths.sort();
    paths
}

fn load_json(path: &Path) -> Result<Json, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read failed: {error}"))?;
    Parser::new(&text)
        .parse()
        .map_err(|error| format!("parse failed: {error}"))
}

fn repo_path(repo_root: &Path, path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn rel(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

pub fn to_json(evaluation: &Evaluation) -> String {
    format!(
        concat!(
            "{{",
            "\"baseline\":{},",
            "\"claim_boundary\":{{\"p0_0_green\":false,\"phase0_complete\":false}},",
            "\"failures\":{},",
            "\"fixture_count\":{},",
            "\"fixture_directory\":{},",
            "\"live_required_context_execution_proven\":false,",
            "\"local_fixture_contract_proven\":{},",
            "\"phase0_complete\":false,",
            "\"p0_0_green\":false,",
            "\"protected_branch_authority_proven\":false,",
            "\"required_red_green_pairs\":{},",
            "\"required_status_source_fixtures\":{},",
            "\"result_bundle_check_target\":{},",
            "\"result_bundle_fixtures\":{},",
            "\"status_mutation_performed\":false,",
            "\"trusted_target_inventory_check_target\":{},",
            "\"trusted_target_inventory_fixtures\":{},",
            "\"verdict\":{}",
            "}}"
        ),
        json_string(&evaluation.baseline),
        json_string_array(&evaluation.failures),
        evaluation.fixture_count,
        json_string(&evaluation.fixture_directory),
        evaluation.local_fixture_contract_proven,
        evaluation.required_red_green_pairs,
        evaluation.required_status_source_fixtures,
        json_string(&evaluation.result_bundle_check_target),
        evaluation.result_bundle_fixtures,
        json_string(&evaluation.trusted_target_inventory_check_target),
        evaluation.trusted_target_inventory_fixtures,
        json_string(&evaluation.verdict),
    )
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
    let mut repo_root = env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("current directory"));
    let mut baseline: Option<PathBuf> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(args.next().expect("--repo-root requires a value"));
            }
            "--baseline" => {
                baseline = Some(PathBuf::from(
                    args.next().expect("--baseline requires a value"),
                ));
            }
            "--json" => {}
            other => panic!("unknown argument {other}"),
        }
    }
    let repo_root = repo_root.canonicalize().unwrap_or(repo_root);
    let mut config = default_config(repo_root.clone());
    if let Some(path) = baseline {
        config.baseline = repo_path(&repo_root, path);
    }
    let evaluation = evaluate(&config);
    println!("{}", to_json(&evaluation));
    if evaluation.verdict != "PASS" {
        eprintln!("phase0-ci-baseline-catalog: RED");
        for failure in &evaluation.failures {
            eprintln!("- {failure}");
        }
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn repo_root() -> PathBuf {
        env::var_os("OYA_REPO_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| env::current_dir().unwrap())
    }

    fn temp_path(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        path.push(format!(
            "oya-phase0-ci-baseline-{label}-{}-{nanos}-{counter}.json",
            std::process::id()
        ));
        path
    }

    fn read_repo_file(path: &str) -> String {
        fs::read_to_string(repo_root().join(path))
            .unwrap_or_else(|error| panic!("read {path}: {error}"))
    }

    fn checked_in_config() -> Config {
        default_config(repo_root())
    }

    fn evaluate_baseline(path: &Path) -> Evaluation {
        let mut config = checked_in_config();
        config.baseline = path.to_path_buf();
        evaluate(&config)
    }

    fn replace_first(text: &str, old: &str, new: &str) -> String {
        assert!(text.contains(old), "mutation source not found: {old}");
        text.replacen(old, new, 1)
    }

    fn assert_fails_with(label: &str, expected: &str, mutate: impl FnOnce(String) -> String) {
        let path = temp_path(label);
        fs::write(
            &path,
            mutate(read_repo_file("specs/phase0-ci-enforcement-baseline.json")),
        )
        .unwrap();
        let evaluation = evaluate_baseline(&path);
        let _ = fs::remove_file(path);
        assert_eq!(evaluation.verdict, "FAIL", "{label} should fail");
        let joined = evaluation.failures.join("\n");
        assert!(
            joined.contains(expected),
            "{label} missing {expected:?}:\n{joined}"
        );
        let json = to_json(&evaluation);
        assert!(json.contains(r#""p0_0_green":false"#));
        assert!(json.contains(r#""phase0_complete":false"#));
    }

    #[test]
    fn checked_in_baseline_catalog_passes() {
        let evaluation = evaluate(&checked_in_config());
        assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
        assert_eq!(evaluation.fixture_count, 15);
        assert_eq!(evaluation.required_red_green_pairs, 11);
        assert_eq!(evaluation.result_bundle_fixtures, 2);
        assert_eq!(evaluation.required_status_source_fixtures, 7);
        assert_eq!(
            evaluation.result_bundle_check_target,
            "//:phase0-result-bundle-output-check"
        );
        assert_eq!(
            evaluation.trusted_target_inventory_check_target,
            "//:phase0-trusted-target-inventory-check"
        );
        assert!(evaluation.local_fixture_contract_proven);
        assert!(!evaluation.live_required_context_execution_proven);
        assert!(!evaluation.protected_branch_authority_proven);
        assert!(!evaluation.status_mutation_performed);
        assert!(!evaluation.p0_0_green);
        assert!(!evaluation.phase0_complete);
        let json = to_json(&evaluation);
        assert!(json.contains(r#""verdict":"PASS""#));
        assert!(json.contains(r#""local_fixture_contract_proven":true"#));
    }

    #[test]
    fn catalog_records_rust_checker_path() {
        let baseline = read_repo_file("specs/phase0-ci-enforcement-baseline.json");
        let fixture_contract = read_repo_file("specs/red-green-fixture-contract.json");
        assert!(baseline.contains("scripts/tests/phase0_ci_enforcement_baseline_catalog_check.rs"));
        assert!(
            fixture_contract
                .contains("scripts/tests/phase0_ci_enforcement_baseline_catalog_check.rs")
        );
    }

    #[test]
    fn missing_fixture_catalog_entry_fails() {
        assert_fails_with(
            "missing-fixture",
            "fixture_set.all_fixture_paths missing actual fixture",
            |text| {
                replace_first(
                    &text,
                    "      \"specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1-bad-missing-required-context.json\",\n",
                    "",
                )
            },
        );
    }

    #[test]
    fn claim_boundary_green_fails() {
        assert_fails_with(
            "claim-boundary-green",
            "claim_boundary.p0_0_green must remain false",
            |text| replace_first(&text, "\"p0_0_green\": false", "\"p0_0_green\": true"),
        );
    }

    #[test]
    fn unknown_red_pair_fixture_fails() {
        assert_fails_with(
            "unknown-red-pair",
            "unknown fixture missing-red-fixture.json",
            |text| {
                replace_first(
                    &text,
                    "\"red\": \"tc-0.0.1-bad-missing-required-context.json\"",
                    "\"red\": \"missing-red-fixture.json\"",
                )
            },
        );
    }

    #[test]
    fn stale_required_context_narrative_fails() {
        let stale_doc = temp_path("stale-doc");
        fs::write(
            &stale_doc,
            "live GitHub branch protection still lacks cloud-ci-required/oya-ci-required",
        )
        .unwrap();
        let mut config = checked_in_config();
        config.required_context_docs = vec![stale_doc.clone()];
        let evaluation = evaluate(&config);
        let _ = fs::remove_file(stale_doc);
        assert_eq!(evaluation.verdict, "FAIL");
        assert!(
            evaluation
                .failures
                .join("\n")
                .contains("stale required-context gap phrase")
        );
        assert!(!evaluation.p0_0_green);
        assert!(!evaluation.phase0_complete);
    }
}
