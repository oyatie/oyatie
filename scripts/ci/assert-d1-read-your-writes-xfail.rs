//! AC-0.10b D1 read-your-writes XFAIL classifier.
//!
//! This checker is local/static evidence only. It verifies that the D1
//! read-your-writes probe is executable, consistency-token aware, and classified
//! as XFAIL until the real Phase-2 mechanism lands. It never runs live D1
//! conformance, posts statuses, mutates branch protection, adds an `oya` CLI
//! surface, or proves Phase-0 completion.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/d1-read-your-writes-xfail-registry.json";
const ROOT_BUCK: &str = "BUCK";
const FIXTURES: &[&str] = &[
    "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-good-xfail-classified-read-your-writes.json",
    "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-misclassified-green-without-phase2.json",
    "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-missing-consistency-token.json",
    "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-phase2-green-claim-without-live-evidence.json",
];
const FALSE_CLAIMS: &[&str] = &[
    "phase2_mechanism_landed",
    "live_d1_conformance_executed",
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];
const TRUE_REGISTRY_FLAGS: &[&str] = &[
    "d1_read_your_writes_xfail_measured",
    "executable_xfail_classifier_required",
    "consistency_token_required",
    "no_new_oya_cli_surface",
];
const KNOWN_VIOLATIONS: &[&str] = &[
    "xfail_misclassified_as_pass",
    "missing_consistency_token",
    "phase2_green_claim_without_live_evidence",
    "live_d1_conformance_claimed",
    "oya_cli_authority_route",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResult {
    // data_class: INTERNAL governance evidence path, not tenant data.
    pub path: String,
    // data_class: INTERNAL deterministic gate diagnostics, not tenant data.
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult {
    // data_class: INTERNAL governance fixture path, not tenant data.
    pub path: String,
    // data_class: INTERNAL expected local/static verdict marker, not live CI state.
    pub expected: String,
    // data_class: INTERNAL synthetic violation labels, not production findings.
    pub observed_violations: Vec<String>,
    // data_class: INTERNAL deterministic gate diagnostics, not tenant data.
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub registry: String,
    pub file_results: Vec<FileResult>,
    pub fixture_results: Vec<FixtureResult>,
    pub failures: Vec<String>,
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

fn compact_json_text(input: &str) -> String {
    input.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn bool_token(key: &str, value: bool) -> String {
    format!("\"{}\":{}", key, if value { "true" } else { "false" })
}

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&bool_token(key, value))
}

fn has_key(text: &str, key: &str) -> bool {
    compact_json_text(text).contains(&format!("\"{}\":", key))
}

fn value_after_key(chunk: &str, key: &str) -> Option<String> {
    let key_token = format!("\"{}\"", key);
    let after_key = chunk.split_once(&key_token)?.1;
    let after_colon = after_key.split_once(':')?.1.trim_start();
    let after_quote = after_colon.strip_prefix('"')?;
    let mut value = String::new();
    let mut escaped = false;
    for ch in after_quote.chars() {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(value),
            _ => value.push(ch),
        }
    }
    None
}

fn expected_violations(text: &str) -> Vec<String> {
    let Some(expected_key_start) = text.find("\"expected_violations\"") else {
        return Vec::new();
    };
    let Some(array_start_offset) = text[expected_key_start..].find('[') else {
        return Vec::new();
    };
    let array_start = expected_key_start + array_start_offset;
    let Some(array_end_offset) = text[array_start..].find(']') else {
        return Vec::new();
    };
    let expected_array = &text[array_start..=array_start + array_end_offset];
    KNOWN_VIOLATIONS
        .iter()
        .filter(|violation| expected_array.contains(&format!("\"{}\"", violation)))
        .map(|violation| (*violation).to_owned())
        .collect()
}

fn read_repo_file(repo_root: &Path, path: &str) -> Result<String, String> {
    fs::read_to_string(repo_root.join(path)).map_err(|error| format!("{path}: {error}"))
}

fn forbidden_oya_cli_route(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    ["oya gate", "oya verify", "oya vcs", "oya git"]
        .iter()
        .any(|needle| lower.contains(needle))
}

pub fn registry_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for flag in TRUE_REGISTRY_FLAGS {
        if !has_bool(text, flag, true) {
            failures.push(format!("missing_true_registry_flag_{flag}"));
        }
    }
    for claim in FALSE_CLAIMS {
        if !has_bool(text, claim, false) {
            failures.push(format!("forbidden_true_or_missing_claim_{claim}"));
        }
    }
    for required in [
        "AC-0.10b",
        "AC-0.10b-d1-read-your-writes-xfail",
        "//:d1-read-your-writes-xfail-check",
        "scripts/ci/assert-d1-read-your-writes-xfail.rs",
        "scripts/tests/d1_read_your_writes_xfail_check.rs",
        "specs/fixtures/phase0-d1-read-your-writes-xfail",
        "expected_failure_until_phase2_mechanism_lands",
        "//:phase0-red-green-fixture-contract-check",
        "//:phase0-automation-ratchet-check",
        "//:buck2-authority-policy-check",
    ] {
        if !text.contains(required) {
            failures.push(format!("registry_missing_anchor_{required}"));
        }
    }
    for path in FIXTURES {
        if !text.contains(path) {
            failures.push(format!("registry_missing_fixture_path_{path}"));
        }
    }
    for violation in KNOWN_VIOLATIONS
        .iter()
        .filter(|v| **v != "oya_cli_authority_route")
    {
        if !text.contains(violation) {
            failures.push(format!("registry_missing_violation_catalog_{violation}"));
        }
    }
    if forbidden_oya_cli_route(text) {
        failures.push("registry_maps_to_oya_cli_authority".to_owned());
    }
    failures
}

pub fn fixture_policy_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for required in [
        "tc_id",
        "expected_verdict",
        "expected_violations",
        "claim_boundary",
        "read_your_writes_probe",
        "mutation_result",
        "observed_read",
        "classification",
        "phase2_mechanism_landed",
        "live_d1_conformance_executed",
        "no_new_oya_cli_surface",
    ] {
        if !has_key(text, required) && !text.contains(required) {
            failures.push(format!("fixture_missing_anchor_{required}"));
        }
    }
    if !has_bool(text, "p0_0_green", false) || !has_bool(text, "phase0_complete", false) {
        failures.push("fixture_missing_false_completion_claim_boundary".to_owned());
    }
    match value_after_key(text, "expected_verdict").as_deref() {
        Some("XFAIL" | "FAIL") => {}
        Some(value) => failures.push(format!("invalid_expected_verdict_{value}")),
        None => failures.push("missing_expected_verdict".to_owned()),
    }
    failures
}

pub fn fixture_observed_violations(text: &str) -> Vec<String> {
    let compact = compact_json_text(text);
    let mut violations = Vec::new();
    if !compact.contains("\"consistency_token\":") {
        violations.push("missing_consistency_token".to_owned());
    }
    let classification = value_after_key(text, "classification").unwrap_or_default();
    if classification == "PASS" && !compact.contains("\"phase2_mechanism_landed\":true") {
        violations.push("xfail_misclassified_as_pass".to_owned());
    }
    if compact.contains("\"phase2_mechanism_landed\":true") {
        violations.push("phase2_green_claim_without_live_evidence".to_owned());
    }
    if compact.contains("\"live_d1_conformance_executed\":true") {
        violations.push("live_d1_conformance_claimed".to_owned());
    }
    if forbidden_oya_cli_route(text) {
        violations.push("oya_cli_authority_route".to_owned());
    }
    violations.sort();
    violations.dedup();
    violations
}

fn evaluate_fixture(path: &str, text: &str) -> FixtureResult {
    let expected =
        value_after_key(text, "expected_verdict").unwrap_or_else(|| "<missing>".to_owned());
    let observed_violations = fixture_observed_violations(text);
    let expected_violations = expected_violations(text);
    let mut failures = fixture_policy_failures(text);
    if expected == "XFAIL" && !observed_violations.is_empty() {
        failures.push(format!(
            "{path}:xfail_fixture_has_policy_violations:{}",
            observed_violations.join(",")
        ));
    }
    if expected == "FAIL" {
        if expected_violations.is_empty() {
            failures.push(format!("{path}:bad_fixture_missing_expected_violations"));
        }
        for violation in &expected_violations {
            if !observed_violations
                .iter()
                .any(|observed| observed == violation)
            {
                failures.push(format!(
                    "{path}:expected_violation_not_observed:{violation}"
                ));
            }
        }
    }
    FixtureResult {
        path: path.to_owned(),
        expected,
        observed_violations,
        failures,
    }
}

pub fn buck_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for required in [
        "name = \"d1-read-your-writes-xfail-check\"",
        "scripts/ci/assert-d1-read-your-writes-xfail.rs",
        "scripts/tests/d1_read_your_writes_xfail_check.rs",
        "specs/d1-read-your-writes-xfail-registry.json",
        "specs/fixtures/phase0-d1-read-your-writes-xfail/*.json",
        "rustc --edition=2024 -D warnings",
    ] {
        if !text.contains(required) {
            failures.push(format!("buck_missing_anchor_{required}"));
        }
    }
    failures
}

pub fn evaluate(repo_root: &Path, registry: &str) -> Result<Evaluation, String> {
    let registry_text = read_repo_file(repo_root, registry)?;
    let buck_text = read_repo_file(repo_root, ROOT_BUCK)?;
    let mut file_results = Vec::new();
    let mut fixture_results = Vec::new();
    let mut failures = Vec::new();
    let registry_file_failures = registry_failures(&registry_text);
    failures.extend(registry_file_failures.clone());
    file_results.push(FileResult {
        path: registry.to_owned(),
        failures: registry_file_failures,
    });
    let buck_file_failures = buck_failures(&buck_text);
    failures.extend(buck_file_failures.clone());
    file_results.push(FileResult {
        path: ROOT_BUCK.to_owned(),
        failures: buck_file_failures,
    });
    for path in FIXTURES {
        let fixture_text = read_repo_file(repo_root, path)?;
        let result = evaluate_fixture(path, &fixture_text);
        failures.extend(result.failures.clone());
        fixture_results.push(result);
    }
    let verdict = if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned();
    Ok(Evaluation {
        verdict,
        registry: registry.to_owned(),
        file_results,
        fixture_results,
        failures,
    })
}

fn render_string_array(items: &[String]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|item| format!("\"{}\"", json_escape(item)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn render_file_results(items: &[FileResult]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|item| format!(
                "{{\"path\":\"{}\",\"failures\":{}}}",
                json_escape(&item.path),
                render_string_array(&item.failures)
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn render_fixture_results(items: &[FixtureResult]) -> String {
    format!("[{}]", items.iter().map(|item| format!("{{\"path\":\"{}\",\"expected\":\"{}\",\"observed_violations\":{},\"failures\":{}}}", json_escape(&item.path), json_escape(&item.expected), render_string_array(&item.observed_violations), render_string_array(&item.failures))).collect::<Vec<_>>().join(","))
}

fn render_json(evaluation: &Evaluation) -> String {
    let xfail_fixture_count = evaluation
        .fixture_results
        .iter()
        .filter(|item| item.expected == "XFAIL")
        .count();
    let bad_fixture_count = evaluation
        .fixture_results
        .iter()
        .filter(|item| item.expected == "FAIL")
        .count();
    let observed_violation_count = evaluation
        .fixture_results
        .iter()
        .map(|item| item.observed_violations.len())
        .sum::<usize>();
    format!(
        concat!(
            "{{",
            "\"authority_boundary\":\"AC-0.10b local/static D1 read-your-writes XFAIL fixture evidence only; no live D1 conformance, status mutation, live required-context authority, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven\",",
            "\"d1_read_your_writes_xfail_measured\":{},",
            "\"phase2_mechanism_landed\":false,",
            "\"live_d1_conformance_executed\":false,",
            "\"status_mutation_performed\":false,",
            "\"protected_branch_authority_proven\":false,",
            "\"live_required_context_execution_proven\":false,",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"production_ready\":false,",
            "\"hyperscaler_grade\":false,",
            "\"registry\":\"{}\",",
            "\"file_results\":{},",
            "\"fixture_results\":{},",
            "\"fixture_count\":{},",
            "\"xfail_fixture_count\":{},",
            "\"bad_fixture_count\":{},",
            "\"observed_violation_count\":{},",
            "\"verdict\":\"{}\",",
            "\"failures\":{}",
            "}}"
        ),
        evaluation.verdict == "PASS",
        json_escape(&evaluation.registry),
        render_file_results(&evaluation.file_results),
        render_fixture_results(&evaluation.fixture_results),
        evaluation.fixture_results.len(),
        xfail_fixture_count,
        bad_fixture_count,
        observed_violation_count,
        evaluation.verdict,
        render_string_array(&evaluation.failures)
    )
}

fn parse_args() -> (PathBuf, String, bool) {
    let mut repo_root = PathBuf::from(".");
    let mut registry = DEFAULT_REGISTRY.to_owned();
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(args.next().expect("--repo-root requires value"))
            }
            "--registry" => registry = args.next().expect("--registry requires value"),
            "--json" => json = true,
            other => panic!("unknown argument {other}"),
        }
    }
    (repo_root, registry, json)
}

fn main() -> Result<(), String> {
    let (repo_root, registry, json) = parse_args();
    let evaluation = evaluate(&repo_root, &registry)?;
    let rendered = render_json(&evaluation);
    if json || evaluation.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if evaluation.verdict == "PASS" {
        Ok(())
    } else {
        Err(format!(
            "D1 read-your-writes XFAIL check failed: {}",
            evaluation.failures.join(",")
        ))
    }
}
