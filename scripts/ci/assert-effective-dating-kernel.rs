//! AC-0.6 effective-dating kernel seed gate.
//!
//! This checker is local/static evidence only. It verifies that the ontology
//! kernel exposes bitemporal effective-dating primitives, that Buck2 target
//! wiring exists, and that GOOD/BAD fixtures preserve the T0.6 boundaries. It
//! never runs live CI, posts statuses, mutates branch protection, or proves
//! Phase-0 completion.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/effective-dating-kernel-registry.json";
const KERNEL_MODULE: &str = "oya/ontology/crates/oya-ontology-kernel/src/effective_dating.rs";
const KERNEL_EXPORTS: &str = "oya/ontology/crates/oya-ontology-kernel/src/lib.rs";
const KERNEL_BUCK: &str = "oya/ontology/crates/oya-ontology-kernel/BUCK";
const ROOT_BUCK: &str = "BUCK";

const FIXTURES: &[&str] = &[
    "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-good-effective-dating-kernel.json",
    "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-bad-overlapping-valid-time.json",
    "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-bad-clock-skew-nondeterministic.json",
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
    "effective_dating_kernel_measured",
    "bitemporal_type_exposed",
    "valid_time_transaction_time_tested",
    "as_of_query_tested",
    "overlap_rejection_tested",
    "open_ended_range_tested",
    "clock_skew_determinism_tested",
];

const KERNEL_TOKENS: &[&str] = &[
    "pub struct EffectiveInstant",
    "pub struct EffectiveTimeRange",
    "pub struct BitemporalRange",
    "pub struct EffectiveDatedVersion",
    "pub struct EffectiveDatedHistory",
    "pub enum EffectiveDatingError",
    "pub fn as_of",
    "OverlappingBitemporalRange",
    "NoVersionAtAsOf",
    "out_of_order_transaction_time_inserts_are_sorted_and_queryable",
    "property_grid_exercises_validity_range_overlap_boundaries",
    "data_class:",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResult {
    pub path: String,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult {
    pub path: String,
    pub expected: String,
    pub observed_violations: Vec<String>,
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

fn read_repo_file(repo_root: &Path, path: &str) -> Result<String, String> {
    fs::read_to_string(repo_root.join(path)).map_err(|error| format!("{path}: {error}"))
}

pub fn registry_failures(registry: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for flag in TRUE_REGISTRY_FLAGS {
        if !has_bool(registry, flag, true) {
            failures.push(format!("missing_true_registry_flag_{flag}"));
        }
    }
    for claim in FALSE_CLAIMS {
        if !has_bool(registry, claim, false) {
            failures.push(format!("forbidden_true_or_missing_claim_{claim}"));
        }
    }
    for token in [
        "//:effective-dating-kernel-check",
        "//oya/ontology/crates/oya-ontology-kernel:effective-dating-kernel-tests",
        "Buck2-native LLVM source-based coverage",
        "Tarpaulin is not the canonical coverage surface",
        "Dual Cargo.toml + Buck2/Reindeer",
        "rustc -C instrument-coverage",
    ] {
        if !registry.contains(token) {
            failures.push(format!("registry_missing_token_{token}"));
        }
    }
    failures
}

pub fn kernel_source_failures(source: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for token in KERNEL_TOKENS {
        if !source.contains(token) {
            failures.push(format!("kernel_missing_token_{token}"));
        }
    }
    for token in ["[start, end_exclusive)", "conflicts_with", "sort_versions"] {
        if !source.contains(token) {
            failures.push(format!("kernel_missing_temporal_semantic_{token}"));
        }
    }
    failures
}

pub fn export_failures(exports: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for token in [
        "pub mod effective_dating;",
        "pub use effective_dating::",
        "BitemporalRange",
        "EffectiveDatedHistory",
        "EffectiveTimeRange",
    ] {
        if !exports.contains(token) {
            failures.push(format!("kernel_exports_missing_{token}"));
        }
    }
    failures
}

pub fn buck_failures(kernel_buck: &str, root_buck: &str) -> Vec<String> {
    let mut failures = Vec::new();
    if !kernel_buck.contains("effective-dating-kernel-tests") {
        failures.push("kernel_buck_missing_effective-dating-kernel-tests".to_owned());
    }
    if !kernel_buck.contains("rustc --edition=2024 -D warnings") {
        failures.push("kernel_buck_missing_rustc_test_warning_gate".to_owned());
    }
    for token in [
        "effective-dating-kernel-check",
        "effective-dating-kernel-tests",
    ] {
        if !root_buck.contains(token) {
            failures.push(format!("root_buck_missing_{token}"));
        }
    }
    if !root_buck.contains(
        "rustc --edition=2021 -D warnings scripts/tests/effective_dating_kernel_check.rs --test",
    ) {
        failures.push("root_buck_missing_rust_fixture_harness".to_owned());
    }
    failures
}

pub fn fixture_policy_failures(fixture: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for claim in FALSE_CLAIMS {
        if !has_bool(fixture, claim, false) {
            failures.push(format!("forbidden_true_or_missing_claim_{claim}"));
        }
    }
    let expected_pass = fixture.contains("\"expected_verdict\": \"PASS\"");
    let expected_fail = fixture.contains("\"expected_verdict\": \"FAIL\"");
    if expected_pass {
        for flag in TRUE_REGISTRY_FLAGS
            .iter()
            .filter(|flag| **flag != "effective_dating_kernel_measured")
        {
            if !has_bool(fixture, flag, true) {
                failures.push(format!("good_fixture_missing_true_flag_{flag}"));
            }
        }
        for token in [
            "TC-0.6-GOOD-effective-dating-kernel-bitemporal-as-of",
            "//oya/ontology/crates/oya-ontology-kernel:effective-dating-kernel-tests",
            "//:effective-dating-kernel-check",
        ] {
            if !fixture.contains(token) {
                failures.push(format!("good_fixture_missing_token_{token}"));
            }
        }
    } else if expected_fail {
        if !fixture.contains("overlapping_bitemporal_range")
            && !fixture.contains("clock_skew_determinism_missing")
        {
            failures.push("bad_fixture_missing_expected_violation".to_owned());
        }
    } else {
        failures.push("fixture_missing_expected_verdict".to_owned());
    }
    failures
}

fn fixture_observed_violations(fixture: &str) -> Vec<String> {
    [
        "overlapping_bitemporal_range",
        "clock_skew_determinism_missing",
    ]
    .iter()
    .filter(|token| fixture.contains(**token))
    .map(|token| (*token).to_owned())
    .collect()
}

fn evaluate_path(path: &str, failures: Vec<String>) -> FileResult {
    FileResult {
        path: path.to_owned(),
        failures,
    }
}

pub fn evaluate(repo_root: &Path, registry_path: &str) -> Result<Evaluation, String> {
    let registry = read_repo_file(repo_root, registry_path)?;
    let kernel_source = read_repo_file(repo_root, KERNEL_MODULE)?;
    let exports = read_repo_file(repo_root, KERNEL_EXPORTS)?;
    let kernel_buck = read_repo_file(repo_root, KERNEL_BUCK)?;
    let root_buck = read_repo_file(repo_root, ROOT_BUCK)?;

    let file_results = vec![
        evaluate_path(registry_path, registry_failures(&registry)),
        evaluate_path(KERNEL_MODULE, kernel_source_failures(&kernel_source)),
        evaluate_path(KERNEL_EXPORTS, export_failures(&exports)),
        evaluate_path(
            KERNEL_BUCK,
            buck_failures(&kernel_buck, &root_buck)
                .into_iter()
                .filter(|failure| failure.starts_with("kernel_"))
                .collect(),
        ),
        evaluate_path(
            ROOT_BUCK,
            buck_failures(&kernel_buck, &root_buck)
                .into_iter()
                .filter(|failure| failure.starts_with("root_"))
                .collect(),
        ),
    ];

    let fixture_results = FIXTURES
        .iter()
        .map(|path| {
            let fixture = read_repo_file(repo_root, path)?;
            let expected = if fixture.contains("\"expected_verdict\": \"PASS\"") {
                "PASS"
            } else if fixture.contains("\"expected_verdict\": \"FAIL\"") {
                "FAIL"
            } else {
                "UNKNOWN"
            };
            Ok(FixtureResult {
                path: (*path).to_owned(),
                expected: expected.to_owned(),
                observed_violations: fixture_observed_violations(&fixture),
                failures: fixture_policy_failures(&fixture),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let failures = file_results
        .iter()
        .flat_map(|result| {
            result
                .failures
                .iter()
                .map(|failure| format!("{}:{failure}", result.path))
        })
        .chain(fixture_results.iter().flat_map(|result| {
            result
                .failures
                .iter()
                .map(|failure| format!("{}:{failure}", result.path))
        }))
        .collect::<Vec<_>>();

    let verdict = if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned();
    Ok(Evaluation {
        verdict,
        registry: registry_path.to_owned(),
        file_results,
        fixture_results,
        failures,
    })
}

fn array_json(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn to_json(evaluation: &Evaluation) -> String {
    let files = evaluation
        .file_results
        .iter()
        .map(|result| {
            format!(
                "{{\"path\":\"{}\",\"failures\":{}}}",
                json_escape(&result.path),
                array_json(&result.failures)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let fixtures = evaluation
        .fixture_results
        .iter()
        .map(|result| {
            format!(
                "{{\"path\":\"{}\",\"expected\":\"{}\",\"observed_violations\":{},\"failures\":{}}}",
                json_escape(&result.path),
                json_escape(&result.expected),
                array_json(&result.observed_violations),
                array_json(&result.failures)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"verdict\":\"{}\",\"registry\":\"{}\",\"effective_dating_kernel_measured\":true,\"contract_count\":5,\"fixture_count\":{},\"file_results\":[{}],\"fixture_results\":[{}],\"failures\":{}}}",
        json_escape(&evaluation.verdict),
        json_escape(&evaluation.registry),
        evaluation.fixture_results.len(),
        files,
        fixtures,
        array_json(&evaluation.failures)
    )
}

pub fn parse_args() -> Config {
    let mut repo_root = env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut registry = DEFAULT_REGISTRY.to_owned();
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
            }
            "--registry" => {
                registry = args.next().unwrap_or_else(|| DEFAULT_REGISTRY.to_owned());
            }
            "--json" => json = true,
            _ => {}
        }
    }
    Config {
        repo_root,
        registry,
        json,
    }
}

fn main() {
    let config = parse_args();
    let evaluation = match evaluate(&config.repo_root, &config.registry) {
        Ok(evaluation) => evaluation,
        Err(error) => Evaluation {
            verdict: "FAIL".to_owned(),
            registry: config.registry,
            file_results: Vec::new(),
            fixture_results: Vec::new(),
            failures: vec![error],
        },
    };
    if config.json {
        println!("{}", to_json(&evaluation));
    } else {
        println!("{}", evaluation.verdict);
        for failure in &evaluation.failures {
            eprintln!("{failure}");
        }
    }
    if evaluation.verdict != "PASS" {
        std::process::exit(1);
    }
}
