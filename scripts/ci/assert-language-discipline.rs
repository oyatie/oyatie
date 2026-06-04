//! AC-0.4 language-discipline seed gate.
//!
//! This checker is intentionally Rust-first for the new gate surface. It reads a
//! small machine-readable registry plus checked-in GOOD/BAD fixtures, blocks new
//! candidate-authored Python/shell sprawl outside the allowlist, and keeps all
//! live-authority/readiness claims false. It is local/static evidence only.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/language-discipline-registry.json";

const FIXTURES: &[&str] = &[
    "specs/fixtures/phase0-language-discipline/tc-0.4-good-allowlisted-bootstrap-shell-edit.json",
    "specs/fixtures/phase0-language-discipline/tc-0.4-bad-new-python-under-scripts.json",
    "specs/fixtures/phase0-language-discipline/tc-0.4-bad-new-shell-test-sprawl.json",
    "specs/fixtures/phase0-language-discipline/tc-0.4-good-non-script-change.json",
];

const BACKLOG_OFFENDERS: &[&str] = &[
    "scripts/tests/cloud_control_plane_operation_contract_check.py",
    "scripts/tests/cloud_dogfood_ci_toolchain_lane_reconciliation_check.py",
    "scripts/tests/cloud_enforceability_facets_check.py",
    "scripts/tests/cloud_hyperscaler_parity_taxonomy_check.py",
    "scripts/tests/cloud_observability_slo_evidence_check.py",
    "scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py",
    "scripts/tests/cloud_resource_contract_parity_catalog_check.py",
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedFile {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult {
    pub path: String,
    pub expected: String,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub registry: String,
    pub fixture_count: usize,
    pub backlog_offender_count: usize,
    pub fixture_results: Vec<FixtureResult>,
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

fn required_bool_token(key: &str, value: bool) -> String {
    format!("\"{}\":{}", key, if value { "true" } else { "false" })
}

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&required_bool_token(key, value))
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

pub fn changed_files_from_fixture(text: &str) -> Vec<ChangedFile> {
    text.split('{')
        .filter_map(|chunk| {
            if !chunk.contains("\"path\"") || !chunk.contains("\"status\"") {
                return None;
            }
            let path = value_after_key(chunk, "path")?;
            let status = value_after_key(chunk, "status")?;
            Some(ChangedFile { path, status })
        })
        .collect()
}

fn is_new_status(status: &str) -> bool {
    matches!(status, "A" | "AM" | "??")
}

fn is_allowlisted_shell(path: &str) -> bool {
    path.starts_with("tools/hook-bootstrap/") && path.ends_with(".sh")
}

pub fn fixture_policy_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for claim in FALSE_CLAIMS {
        if !has_bool(text, claim, false) {
            failures.push(format!("fixture_forbidden_true_or_missing_claim_{}", claim));
        }
    }

    let files = changed_files_from_fixture(text);
    if files.is_empty() {
        failures.push("fixture_missing_changed_files".to_owned());
    }

    for file in files {
        if !is_new_status(&file.status) {
            continue;
        }
        if file.path.ends_with(".py") {
            failures.push(format!("new_python_outside_allowlist:{}", file.path));
        }
        if file.path.ends_with(".sh") && !is_allowlisted_shell(&file.path) {
            failures.push(format!("new_shell_outside_allowlist:{}", file.path));
        }
    }

    failures
}

pub fn registry_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for claim in FALSE_CLAIMS {
        if !has_bool(text, claim, false) {
            failures.push(format!("forbidden_true_or_missing_claim_{}", claim));
        }
    }

    for (key, expected) in [
        ("language_discipline_allowlist_measured", true),
        ("new_python_outside_allowlist_blocked", true),
        ("new_shell_outside_allowlist_blocked", true),
        ("offender_inventory_recorded", true),
    ] {
        if !has_bool(text, key, expected) {
            failures.push(format!("missing_registry_boundary_{}", key));
        }
    }

    for token in [
        "//:language-discipline-check",
        "scripts/ci/assert-language-discipline.rs",
        "scripts/tests/language_discipline_check.rs",
        "Buck2-native LLVM source-based coverage",
        "Tarpaulin is not the canonical coverage surface",
        "dual Cargo.toml + Buck2/Reindeer",
        "trusted cloud-ci/oya-ci",
        "hyperscaler-oriented",
    ] {
        if !text.contains(token) {
            failures.push(format!("missing_required_registry_anchor:{}", token));
        }
    }

    if !compact_json_text(text).contains("\"new_python_or_shell_gate_surface_added\":false") {
        failures.push("missing_no_new_python_or_shell_gate_surface_anchor".to_owned());
    }

    if !compact_json_text(text).contains("\"t0_4_cloud_check_backlog_count\":7") {
        failures.push("missing_t0_4_cloud_check_backlog_count_7".to_owned());
    }

    for path in BACKLOG_OFFENDERS {
        if !text.contains(path) {
            failures.push(format!("missing_required_backlog_offender:{}", path));
        }
    }

    for fixture in FIXTURES {
        if !text.contains(fixture) {
            failures.push(format!("missing_registered_fixture:{}", fixture));
        }
    }

    failures
}

pub fn evaluate(root: &Path, registry: &str) -> Result<Evaluation, String> {
    let registry_path = root.join(registry);
    let registry_text = fs::read_to_string(&registry_path)
        .map_err(|err| format!("read registry {}: {}", registry_path.display(), err))?;
    let mut failures = registry_failures(&registry_text);

    let mut fixture_results = Vec::new();
    for fixture in FIXTURES {
        let fixture_path = root.join(fixture);
        let fixture_text = fs::read_to_string(&fixture_path)
            .map_err(|err| format!("read fixture {}: {}", fixture_path.display(), err))?;
        let expected = value_after_key(&fixture_text, "expected_verdict")
            .unwrap_or_else(|| "<missing>".to_owned());
        let fixture_failures = fixture_policy_failures(&fixture_text);

        match expected.as_str() {
            "PASS" if !fixture_failures.is_empty() => failures.push(format!(
                "fixture_expected_pass_failed:{}:{}",
                fixture,
                fixture_failures.join("|")
            )),
            "FAIL" if fixture_failures.is_empty() => {
                failures.push(format!("fixture_expected_fail_passed:{}", fixture))
            }
            "FAIL" => {
                if fixture_text.contains("new_python_outside_allowlist")
                    && !fixture_failures
                        .iter()
                        .any(|failure| failure.starts_with("new_python_outside_allowlist"))
                {
                    failures.push(format!(
                        "fixture_missing_expected_python_failure:{}",
                        fixture
                    ));
                }
                if fixture_text.contains("new_shell_outside_allowlist")
                    && !fixture_failures
                        .iter()
                        .any(|failure| failure.starts_with("new_shell_outside_allowlist"))
                {
                    failures.push(format!(
                        "fixture_missing_expected_shell_failure:{}",
                        fixture
                    ));
                }
            }
            "PASS" => {}
            _ => failures.push(format!(
                "fixture_missing_or_invalid_expected_verdict:{}",
                fixture
            )),
        }

        fixture_results.push(FixtureResult {
            path: (*fixture).to_owned(),
            expected,
            failures: fixture_failures,
        });
    }

    let verdict = if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned();
    Ok(Evaluation {
        verdict,
        registry: registry.to_owned(),
        fixture_count: fixture_results.len(),
        backlog_offender_count: BACKLOG_OFFENDERS.len(),
        fixture_results,
        failures,
    })
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Config, String> {
    let mut repo_root = PathBuf::from(".");
    let mut registry = DEFAULT_REGISTRY.to_owned();
    let mut json = false;

    let mut iter = args.into_iter();
    let _program = iter.next();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--repo-root requires a value".to_owned())?,
                );
            }
            "--registry" => {
                registry = iter
                    .next()
                    .ok_or_else(|| "--registry requires a value".to_owned())?;
            }
            "--json" => json = true,
            "-h" | "--help" => return Err(
                "usage: assert-language-discipline [--repo-root DIR] [--registry PATH] [--json]"
                    .to_owned(),
            ),
            other => return Err(format!("unknown argument: {}", other)),
        }
    }

    Ok(Config {
        repo_root,
        registry,
        json,
    })
}

pub fn evaluation_json(evaluation: &Evaluation) -> String {
    let fixture_json = evaluation
        .fixture_results
        .iter()
        .map(|fixture| {
            let failures = fixture
                .failures
                .iter()
                .map(|failure| format!("\"{}\"", json_escape(failure)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"expected\":\"{}\",\"failures\":[{}],\"path\":\"{}\"}}",
                json_escape(&fixture.expected),
                failures,
                json_escape(&fixture.path)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let failures_json = evaluation
        .failures
        .iter()
        .map(|failure| format!("\"{}\"", json_escape(failure)))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"backlog_offender_count\":{},\"claim_boundary\":{{\"hyperscaler_grade\":false,\"live_required_context_execution_proven\":false,\"p0_0_green\":false,\"phase0_complete\":false,\"production_ready\":false,\"protected_branch_authority_proven\":false,\"status_mutation_performed\":false}},\"failures\":[{}],\"fixture_count\":{},\"fixture_results\":[{}],\"registry\":\"{}\",\"verdict\":\"{}\"}}",
        evaluation.backlog_offender_count,
        failures_json,
        evaluation.fixture_count,
        fixture_json,
        json_escape(&evaluation.registry),
        evaluation.verdict
    )
}

fn print_human(evaluation: &Evaluation) {
    println!("language-discipline: {}", evaluation.verdict);
    println!("registry: {}", evaluation.registry);
    println!("fixtures: {}", evaluation.fixture_count);
    println!("backlog_offenders: {}", evaluation.backlog_offender_count);
    for fixture in &evaluation.fixture_results {
        println!(
            "fixture {} expected={} failures={}",
            fixture.path,
            fixture.expected,
            fixture.failures.len()
        );
    }
    for failure in &evaluation.failures {
        eprintln!("- {}", failure);
    }
}

fn main() {
    let config = match parse_args(env::args()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(2);
        }
    };

    match evaluate(&config.repo_root, &config.registry) {
        Ok(evaluation) => {
            if config.json {
                println!("{}", evaluation_json(&evaluation));
            } else {
                print_human(&evaluation);
            }
            if evaluation.failures.is_empty() {
                std::process::exit(0);
            }
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("language-discipline: RED");
            eprintln!("- {}", err);
            std::process::exit(1);
        }
    }
}
