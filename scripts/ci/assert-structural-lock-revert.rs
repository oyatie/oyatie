//! AC-0.9 structural-lock/revert seed gate.
//!
//! This checker is local/static evidence only. It verifies that structural
//! work has serialized path ownership, protected-flow revert evidence, RED/GREEN
//! fixtures for overlap/stale/false-authority cases, and explicit non-claim
//! boundaries. It never runs live CI, posts statuses, mutates branch protection,
//! adds an `oya` CLI surface, or proves Phase-0 completion.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/structural-lock-revert-registry.json";
const ROOT_BUCK: &str = "BUCK";

const FIXTURES: &[&str] = &[
    "specs/fixtures/phase0-structural-lock-revert/tc-0.9-good-serialized-structural-lock-revert.json",
    "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-missing-protected-revert-evidence.json",
    "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-overlapping-structural-lanes.json",
    "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-mechanical-lock-claim.json",
    "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-stale-lock-ttl.json",
];

const FALSE_CLAIMS: &[&str] = &[
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
    "mechanical_structural_lock_proven",
];

const TRUE_REGISTRY_FLAGS: &[&str] = &[
    "structural_lock_revert_measured",
    "serialized_structural_work_required",
    "protected_flow_revert_evidence_required",
    "advisory_lock_boundary_enforced",
    "no_new_oya_cli_surface",
];

const REQUIRED_PACKET_FIELDS: &[&str] = &[
    "packet_id",
    "candidate_branch",
    "candidate_sha",
    "classification",
    "status",
    "owner_lane",
    "lock_scope",
    "lock_acquired_at",
    "lock_expires_at",
    "locked_path_globs",
    "parallel_lane_policy",
    "protected_flow_revert",
    "authority",
];

const KNOWN_FIXTURE_VIOLATIONS: &[&str] = &[
    "missing_protected_flow_revert_evidence",
    "structural_path_overlap_detected",
    "parallel_structural_lane_not_serialized",
    "forbidden_mechanical_structural_lock_claim",
    "status_mutation_performed",
    "lock_ttl_not_future_or_expired",
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
    // data_class: INTERNAL local/static gate verdict, not branch-protection authority.
    pub verdict: String,
    // data_class: INTERNAL registry path, not tenant data.
    pub registry: String,
    // data_class: INTERNAL file-level diagnostics, not tenant data.
    pub file_results: Vec<FileResult>,
    // data_class: INTERNAL fixture diagnostics, not tenant data.
    pub fixture_results: Vec<FixtureResult>,
    // data_class: INTERNAL flattened diagnostics, not tenant data.
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub repo_root: PathBuf,
    pub registry: String,
    pub json: bool,
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

fn read_repo_file(repo_root: &Path, path: &str) -> Result<String, String> {
    fs::read_to_string(repo_root.join(path)).map_err(|error| format!("{path}: {error}"))
}

fn forbidden_oya_cli_route(text: &str) -> bool {
    ["oya gate", "oya verify", "oya vcs", "oya git"]
        .iter()
        .any(|needle| text.contains(needle))
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
        "AC-0.9",
        "AC-0.9-structural-lock-revert",
        "//:structural-lock-revert-check",
        "scripts/ci/assert-structural-lock-revert.rs",
        "scripts/tests/structural_lock_revert_check.rs",
        "specs/fixtures/phase0-structural-lock-revert",
        "//:phase0-red-green-fixture-contract-check",
        "//:phase0-automation-ratchet-check",
        "//:buck2-authority-policy-check",
    ] {
        if !text.contains(required) {
            failures.push(format!("registry_missing_anchor_{required}"));
        }
    }
    for field in REQUIRED_PACKET_FIELDS {
        if !text.contains(&format!("\"{field}\"")) {
            failures.push(format!("registry_missing_required_packet_field_{field}"));
        }
    }
    for path in FIXTURES {
        if !text.contains(path) {
            failures.push(format!("registry_missing_fixture_path_{path}"));
        }
    }
    for violation in KNOWN_FIXTURE_VIOLATIONS
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
        "structural_lock_contract_version",
        "claim_boundary",
        "structural_lock_packet",
        "candidate_branch",
        "candidate_sha",
        "classification",
        "owner_lane",
        "lock_scope",
        "lock_acquired_at",
        "lock_expires_at",
        "locked_path_globs",
        "parallel_lane_policy",
        "authority",
        "no_new_oya_cli_surface",
        "required_context",
        "oya-ci-required",
    ] {
        if !has_key(text, required) && !text.contains(required) {
            failures.push(format!("fixture_missing_anchor_{required}"));
        }
    }
    match value_after_key(text, "expected_verdict").as_deref() {
        Some("PASS" | "FAIL") => {}
        Some(value) => failures.push(format!("invalid_expected_verdict_{value}")),
        None => failures.push("missing_expected_verdict".to_owned()),
    }
    if !has_bool(text, "p0_0_green", false) {
        failures.push("fixture_missing_false_p0_0_green".to_owned());
    }
    if !has_bool(text, "phase0_complete", false) {
        failures.push("fixture_missing_false_phase0_complete".to_owned());
    }
    failures
}

pub fn fixture_observed_violations(text: &str) -> Vec<String> {
    let compact = compact_json_text(text);
    let mut violations = Vec::new();
    if !compact.contains("\"protected_flow_revert\":{") {
        violations.push("missing_protected_flow_revert_evidence".to_owned());
    }
    if compact.contains("\"path_overlap_detected\":true") {
        violations.push("structural_path_overlap_detected".to_owned());
    }
    if compact.contains("\"one_lane_one_path\":false")
        || compact.contains("\"conflicting_parallel_lanes\":[\"")
    {
        violations.push("parallel_structural_lane_not_serialized".to_owned());
    }
    if compact.contains("\"classification\":\"mechanically_enforced\"")
        || compact.contains("\"status\":\"mechanical_lock_proven\"")
        || compact.contains("\"mechanical_structural_lock_proven\":true")
        || compact.contains("\"lock_output_is_advisory_until_p0_0\":false")
    {
        violations.push("forbidden_mechanical_structural_lock_claim".to_owned());
    }
    if compact.contains("\"status_mutation_performed\":true") {
        violations.push("status_mutation_performed".to_owned());
    }
    let acquired_at = value_after_key(text, "lock_acquired_at");
    let expires_at = value_after_key(text, "lock_expires_at");
    match (acquired_at, expires_at) {
        (Some(acquired), Some(expires)) if expires <= acquired => {
            violations.push("lock_ttl_not_future_or_expired".to_owned());
        }
        (None, _) | (_, None) => violations.push("lock_ttl_not_future_or_expired".to_owned()),
        _ => {}
    }
    if forbidden_oya_cli_route(text) {
        violations.push("oya_cli_authority_route".to_owned());
    }
    violations.sort();
    violations.dedup();
    violations
}

fn fixture_expected_violations(text: &str) -> Vec<String> {
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
    KNOWN_FIXTURE_VIOLATIONS
        .iter()
        .filter(|violation| expected_array.contains(&format!("\"{}\"", violation)))
        .map(|violation| (*violation).to_owned())
        .collect()
}

fn evaluate_fixture(path: &str, text: &str) -> FixtureResult {
    let expected =
        value_after_key(text, "expected_verdict").unwrap_or_else(|| "<missing>".to_owned());
    let observed_violations = fixture_observed_violations(text);
    let expected_violations = fixture_expected_violations(text);
    let mut failures = fixture_policy_failures(text);
    if expected == "PASS" && !observed_violations.is_empty() {
        failures.push(format!(
            "{path}:good_fixture_has_violations:{}",
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
        "name = \"structural-lock-revert-check\"",
        "scripts/ci/assert-structural-lock-revert.rs",
        "scripts/tests/structural_lock_revert_check.rs",
        "specs/structural-lock-revert-registry.json",
        "specs/fixtures/phase0-structural-lock-revert/*.json",
        "rustc --edition=2024 -D warnings",
    ] {
        if !text.contains(required) {
            failures.push(format!("buck_missing_anchor_{required}"));
        }
    }
    failures
}

pub fn evaluate(repo_root: &Path, registry: &str) -> Result<Evaluation, String> {
    let mut file_results = Vec::new();
    let mut fixture_results = Vec::new();
    let mut failures = Vec::new();

    let registry_text = read_repo_file(repo_root, registry)?;
    let registry_file_failures = registry_failures(&registry_text);
    failures.extend(registry_file_failures.clone());
    file_results.push(FileResult {
        path: registry.to_owned(),
        failures: registry_file_failures,
    });

    let buck_text = read_repo_file(repo_root, ROOT_BUCK)?;
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
    let rendered = items
        .iter()
        .map(|item| format!("\"{}\"", json_escape(item)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{rendered}]")
}

fn render_file_results(items: &[FileResult]) -> String {
    let rendered = items
        .iter()
        .map(|item| {
            format!(
                "{{\"path\":\"{}\",\"failures\":{}}}",
                json_escape(&item.path),
                render_string_array(&item.failures)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{rendered}]")
}

fn render_fixture_results(items: &[FixtureResult]) -> String {
    let rendered = items
        .iter()
        .map(|item| {
            format!(
                "{{\"path\":\"{}\",\"expected\":\"{}\",\"observed_violations\":{},\"failures\":{}}}",
                json_escape(&item.path),
                json_escape(&item.expected),
                render_string_array(&item.observed_violations),
                render_string_array(&item.failures)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{rendered}]")
}

fn render_json(evaluation: &Evaluation) -> String {
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
            "\"authority_boundary\":\"AC-0.9 local/static structural-lock/revert fixture evidence only; no status mutation, live required-context authority, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven\",",
            "\"structural_lock_revert_measured\":{},",
            "\"status_mutation_performed\":false,",
            "\"protected_branch_authority_proven\":false,",
            "\"live_required_context_execution_proven\":false,",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"production_ready\":false,",
            "\"hyperscaler_grade\":false,",
            "\"mechanical_structural_lock_proven\":false,",
            "\"registry\":\"{}\",",
            "\"file_results\":{},",
            "\"fixture_results\":{},",
            "\"fixture_count\":{},",
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
        bad_fixture_count,
        observed_violation_count,
        evaluation.verdict,
        render_string_array(&evaluation.failures)
    )
}

fn parse_args() -> Config {
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
            "--help" | "-h" => {
                println!(
                    "usage: assert-structural-lock-revert [--repo-root DIR] [--registry PATH] [--json]"
                );
                std::process::exit(0);
            }
            other => panic!("unknown argument {other}"),
        }
    }
    Config {
        repo_root,
        registry,
        json,
    }
}

fn main() -> Result<(), String> {
    let config = parse_args();
    let evaluation = evaluate(&config.repo_root, &config.registry)?;
    let rendered = render_json(&evaluation);
    if config.json || evaluation.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if evaluation.verdict == "PASS" {
        Ok(())
    } else {
        Err(format!(
            "structural-lock/revert check failed: {}",
            evaluation.failures.join(",")
        ))
    }
}
