//! AC-0.11 who-gates-the-gates meta-check.
//!
//! This checker is local/static evidence only. It proves integrity gates carry
//! their own known-bad fixtures, non-vacuous pass guards, and self-mutation
//! tests before they can be counted as Phase-0 evidence. It never runs live CI,
//! posts statuses, mutates branch protection, adds an `oya` CLI surface, or
//! proves Phase-0 completion.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/who-gates-gates-registry.json";
const ROOT_BUCK: &str = "BUCK";
const RED_GREEN_CONTRACT: &str = "specs/red-green-fixture-contract.json";
const RED_GREEN_HARNESS: &str = "scripts/tests/red_green_fixture_contract_check.test.sh";
const FIXTURES: &[&str] = &[
    "specs/fixtures/phase0-who-gates-gates/tc-0.11-good-known-bad-meta-gate.json",
    "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-missing-known-bad-fixture.json",
    "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-vacuous-pass-condition.json",
    "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-missing-self-mutation-test.json",
    "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-oya-cli-authority-route.json",
];
const FALSE_CLAIMS: &[&str] = &[
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];
const TRUE_REGISTRY_FLAGS: &[&str] = &[
    "who_gates_gates_measured",
    "known_bad_fixture_required",
    "vacuous_pass_guard_required",
    "gate_self_mutation_test_required",
    "no_new_oya_cli_surface",
];
const KNOWN_VIOLATIONS: &[&str] = &[
    "missing_known_bad_fixture",
    "vacuous_pass_condition",
    "missing_self_mutation_test",
    "oya_cli_authority_route",
    "forbidden_green_claim",
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
    // data_class: INTERNAL local/static verdict marker, not live CI state.
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
        "AC-0.11",
        "AC-0.11-who-gates-gates",
        "//:who-gates-gates-check",
        "scripts/ci/assert-who-gates-gates.rs",
        "scripts/tests/who_gates_gates_check.rs",
        "specs/fixtures/phase0-who-gates-gates",
        "known_bad_fixture_required",
        "vacuous_pass_guard_required",
        "gate_self_mutation_test_required",
        "remove-red-marker",
        "stale-marker-text",
        "missing-target",
        "p0-green",
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
    for violation in KNOWN_VIOLATIONS {
        if !text.contains(violation) {
            failures.push(format!("registry_missing_violation_catalog_{violation}"));
        }
    }
    if forbidden_oya_cli_route(text) {
        failures.push("registry_maps_to_oya_cli_authority".to_owned());
    }
    failures
}

pub fn actual_gate_surface_failures(
    red_green: &str,
    red_green_harness: &str,
    buck: &str,
) -> Vec<String> {
    let mut failures = Vec::new();
    let contract_entry_count = red_green.matches("\"buck2_target\": \"//:").count();
    let red_marker_count = red_green.matches("\"red_markers\"").count();
    let green_marker_count = red_green.matches("\"green_markers\"").count();
    if contract_entry_count < 26 {
        failures.push("red_green_contract_entry_count_too_low".to_owned());
    }
    if red_marker_count < contract_entry_count || green_marker_count < contract_entry_count {
        failures.push("red_green_contract_missing_marker_class".to_owned());
    }
    for required in [
        "d1-read-your-writes-xfail",
        "structural-lock-revert",
        "cross-artifact-agreement",
        "effective-dating-kernel",
        "d1-seam-contracts",
    ] {
        if !red_green.contains(required) {
            failures.push(format!(
                "red_green_contract_missing_integrity_gate_{required}"
            ));
        }
    }
    for mutation in [
        "remove-red-marker",
        "stale-marker-text",
        "missing-target",
        "p0-green",
    ] {
        if !red_green_harness.contains(mutation) {
            failures.push(format!(
                "red_green_harness_missing_self_mutation_{mutation}"
            ));
        }
    }
    for required in [
        "name = \"who-gates-gates-check\"",
        "scripts/ci/assert-who-gates-gates.rs",
        "scripts/tests/who_gates_gates_check.rs",
        "specs/who-gates-gates-registry.json",
        "specs/fixtures/phase0-who-gates-gates/*.json",
        "rustc --edition=2021 -D warnings",
    ] {
        if !buck.contains(required) {
            failures.push(format!("buck_missing_anchor_{required}"));
        }
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
        "gate_under_test",
        "pass_condition",
        "known_bad_fixtures",
        "self_mutation_tests",
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
        Some("PASS" | "FAIL") => {}
        Some(value) => failures.push(format!("invalid_expected_verdict_{value}")),
        None => failures.push("missing_expected_verdict".to_owned()),
    }
    failures
}

pub fn fixture_observed_violations(text: &str) -> Vec<String> {
    let compact = compact_json_text(text);
    let lower = text.to_ascii_lowercase();
    let mut violations = Vec::new();
    if !compact.contains("\"known_bad_fixtures\":[")
        || compact.contains("\"known_bad_fixtures\":[]")
    {
        violations.push("missing_known_bad_fixture".to_owned());
    }
    if compact.contains("\"pass_condition\":\"always_true\"")
        || compact.contains("\"vacuous_pass\":true")
        || lower.contains("no assertions")
    {
        violations.push("vacuous_pass_condition".to_owned());
    }
    if !compact.contains("\"self_mutation_tests\":[")
        || compact.contains("\"self_mutation_tests\":[]")
    {
        violations.push("missing_self_mutation_test".to_owned());
    }
    for claim in FALSE_CLAIMS {
        if has_bool(text, claim, true) {
            violations.push("forbidden_green_claim".to_owned());
            break;
        }
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
    if expected == "PASS" && !observed_violations.is_empty() {
        failures.push(format!(
            "{path}:pass_fixture_has_policy_violations:{}",
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

pub fn evaluate(repo_root: &Path, registry: &str) -> Result<Evaluation, String> {
    let registry_text = read_repo_file(repo_root, registry)?;
    let buck_text = read_repo_file(repo_root, ROOT_BUCK)?;
    let red_green_text = read_repo_file(repo_root, RED_GREEN_CONTRACT)?;
    let red_green_harness_text = read_repo_file(repo_root, RED_GREEN_HARNESS)?;
    let mut file_results = Vec::new();
    let mut fixture_results = Vec::new();
    let mut failures = Vec::new();

    let registry_file_failures = registry_failures(&registry_text);
    failures.extend(registry_file_failures.clone());
    file_results.push(FileResult {
        path: registry.to_owned(),
        failures: registry_file_failures,
    });

    let actual_failures =
        actual_gate_surface_failures(&red_green_text, &red_green_harness_text, &buck_text);
    failures.extend(actual_failures.clone());
    file_results.push(FileResult {
        path: RED_GREEN_CONTRACT.to_owned(),
        failures: actual_failures,
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
    format!(
        "[{}]",
        items
            .iter()
            .map(|item| format!("{{\"path\":\"{}\",\"expected\":\"{}\",\"observed_violations\":{},\"failures\":{}}}", json_escape(&item.path), json_escape(&item.expected), render_string_array(&item.observed_violations), render_string_array(&item.failures)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn render_json(evaluation: &Evaluation) -> String {
    let pass_fixture_count = evaluation
        .fixture_results
        .iter()
        .filter(|item| item.expected == "PASS")
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
            "\"authority_boundary\":\"AC-0.11 local/static who-gates-the-gates fixture evidence only; no live CI, status mutation, live required-context authority, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven\",",
            "\"who_gates_gates_measured\":{},",
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
            "\"pass_fixture_count\":{},",
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
        pass_fixture_count,
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
            "who-gates-the-gates check failed: {}",
            evaluation.failures.join(",")
        ))
    }
}
