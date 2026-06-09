// GATE-1 cloud-ci-cross-artifact-agreement: RED/GREEN fixture corpus + born-blocking
// live-corpus self-test. ADR-0083 Tier-3: integration tests use unwrap/expect to assert
// invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_cross_artifact_agreement_app::{Verdict, evaluate};
use serde_json::Value;

/// Walk up to the repo root (the dir holding specs/root-hub-pointers.json), matching the
/// existing kernel-test convention.
fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn fixture_dir() -> PathBuf {
    repo_root().join("specs/fixtures/cross-artifact-agreement")
}

fn load_json(path: &PathBuf) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn expected_violations(fixture: &Value) -> BTreeSet<String> {
    fixture["expected_violations"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn architecture_rule_set(architecture: &Value, context: &str) -> BTreeSet<String> {
    if let Some(required_rules) = architecture.get("required_rules") {
        return string_array_set(required_rules, &format!("{context}.required_rules"));
    }

    let mut rules = BTreeSet::new();
    for key in [
        "required_microservice_rules",
        "required_clean_architecture_rules",
        "required_api_first_rules",
        "required_hyperscaler_pattern_rules",
    ] {
        if let Some(values) = architecture.get(key) {
            rules.extend(string_array_set(values, &format!("{context}.{key}")));
        }
    }
    assert!(
        !rules.is_empty(),
        "{context} must define required_rules or split required_*_rules arrays"
    );
    rules
}

fn string_array_set(value: &Value, context: &str) -> BTreeSet<String> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("{context} must be an array"))
        .iter()
        .map(|item| {
            item.as_str()
                .unwrap_or_else(|| panic!("{context} contains a non-string item"))
                .to_owned()
        })
        .collect()
}

fn missing_from<'a>(expected: &'a BTreeSet<String>, actual: &'a BTreeSet<String>) -> Vec<&'a str> {
    expected
        .difference(actual)
        .map(String::as_str)
        .collect::<Vec<_>>()
}

fn assert_same_rule_set(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
    left_name: &str,
    right_name: &str,
) {
    assert_eq!(
        left,
        right,
        "{left_name} and {right_name} architecture rule sets drifted; missing_from_{left_name}={:?}; missing_from_{right_name}={:?}",
        missing_from(right, left),
        missing_from(left, right)
    );
}

#[test]
fn cross_artifact_fixtures_execute_red_green_cases() {
    let dir = fixture_dir();
    let mut tc_paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("tc-") && n.ends_with(".json"))
        })
        .collect();
    tc_paths.sort();
    assert!(
        !tc_paths.is_empty(),
        "cross-artifact-agreement fixture corpus must not be empty"
    );

    let mut seen_green = false;
    let mut seen_red = false;

    for path in &tc_paths {
        let fixture = load_json(path);
        let report = evaluate(&fixture);
        let expected = expected_violations(&fixture);
        let label = path.file_name().unwrap().to_string_lossy().to_string();

        match fixture["expected_verdict"].as_str() {
            Some("GREEN") => {
                seen_green = true;
                assert_eq!(
                    report.verdict,
                    Verdict::Green,
                    "{label} should be GREEN, got violations {:?}",
                    report.violations
                );
                assert!(
                    report.violations.is_empty(),
                    "{label} GREEN must have zero violations, got {:?}",
                    report.violations
                );
            }
            Some("RED") => {
                seen_red = true;
                assert_eq!(report.verdict, Verdict::Red, "{label} should be RED");
                assert_eq!(report.violations, expected, "{label} violations mismatch");
            }
            other => panic!("{label} has unsupported expected_verdict {other:?}"),
        }
    }

    assert!(
        seen_green && seen_red,
        "cross-artifact-agreement fixtures must include BOTH RED and GREEN cases"
    );
}

/// Productized false-green guard: planning closure is an architecture authority, so the
/// cloud-ci cross-artifact gate must fail whenever the contract, masterplan, or sequencing
/// sidecar carry different first-deliverable architecture rule sets. This is a pure
/// data-over-data check over JSON artifacts; it does not shell out to the legacy dev-cli gate.
#[test]
fn planning_closure_architecture_rules_agree_across_authority_artifacts() {
    let root = repo_root();
    let contract = load_json(&root.join("specs/planning-closure-contract.json"));
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let sequencing = load_json(&root.join("specs/master-plan-sequencing.json"));

    let contract_rules = architecture_rule_set(
        &contract["first_deliverable"]["architecture_exit_bar"],
        "specs/planning-closure-contract.json:first_deliverable.architecture_exit_bar",
    );
    let masterplan_rules = architecture_rule_set(
        &masterplan["planning_closure"]["first_deliverable"]["architecture_exit_bar"],
        "specs/masterplan.json:planning_closure.first_deliverable.architecture_exit_bar",
    );
    let sequencing_rules = architecture_rule_set(
        &sequencing["first_deliverable_ordering"]["architecture_exit_bar"],
        "specs/master-plan-sequencing.json:first_deliverable_ordering.architecture_exit_bar",
    );

    assert_same_rule_set(&contract_rules, &masterplan_rules, "contract", "masterplan");
    assert_same_rule_set(&contract_rules, &sequencing_rules, "contract", "sequencing");
}

/// Born-blocking self-test: GATE-1 must go RED on TODAY's real corpus. Per the firewall
/// doctrine, "a firewall that doesn't block today is the facade we're killing." This runs
/// the producer's decision-crosswalk face over the live tree and asserts the gate flags the
/// real defects:
/// - `generated_face_drift` — catalog.json axes_count:6 vs contracts.json axes_count:7
/// - `dual_decision_collision` — the two ADR-0377 files
/// - `supersession_half_edge` — ADR-0511 supersedes ADR-0359 while ADR-0359 omits it
///
/// Counts are MEASURED, not hardcoded.
#[test]
fn gate1_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();
    let crosswalk = run_producer_face(&root, "decision-crosswalk");

    let report = evaluate(&crosswalk);

    assert_eq!(
        report.verdict,
        Verdict::Red,
        "GATE-1 MUST go RED on today's corpus (the firewall must block today)"
    );
    assert!(
        report.violations.contains("generated_face_drift"),
        "catalog.json axes_count:6 vs contracts.json axes_count:7 -> generated_face_drift must fire"
    );
    assert!(
        report.violations.contains("dual_decision_collision"),
        "two ADR-0377 files -> dual_decision_collision must fire"
    );
    assert!(
        report.violations.contains("supersession_half_edge"),
        "a non-reciprocal supersession edge -> supersession_half_edge must fire"
    );

    // Count the real exhibits for the evidence digest.
    let decisions = crosswalk["decisions"].as_array().expect("decisions");
    let dup_ids = crosswalk["duplicate_ids"]
        .as_array()
        .expect("duplicate_ids");
    let axes = crosswalk["generated_face_axes"]
        .as_object()
        .expect("generated_face_axes");
    let unpropagated = decisions
        .iter()
        .filter(|d| {
            d["status"]
                .as_str()
                .is_some_and(|s| s.eq_ignore_ascii_case("accepted"))
                && (d["in_spec"].as_bool().unwrap_or(false)
                    || d["in_masterplan"].as_bool().unwrap_or(false)
                    || d["in_roadmap"].as_bool().unwrap_or(false))
                && !(d["in_spec"].as_bool().unwrap_or(false)
                    && d["in_masterplan"].as_bool().unwrap_or(false)
                    && d["in_roadmap"].as_bool().unwrap_or(false))
        })
        .count();

    eprintln!(
        "BORN-BLOCKING live-corpus counts: decisions={} duplicate_ids={:?} axes={:?} unpropagated_decision={} violations={:?}",
        decisions.len(),
        dup_ids,
        axes,
        unpropagated,
        report.violations
    );
}

/// Run the producer to emit a single face to stdout, HERMETICALLY (no `env!("CARGO")`, the
/// compile-time cargo-only macro that breaks the buck2 build). The producer binary is resolved
/// at RUNTIME: under buck2 from `OYA_CI_PRODUCER_BIN` (the `$(exe ...)`-substituted built
/// binary), else under cargo via the runtime `CARGO` env var. The producer reads the committed
/// scm-facts face (a declared input); it never calls git.
fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root
        .join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json");
    let output = if let Ok(bin) = std::env::var("OYA_CI_PRODUCER_BIN") {
        let bin = if Path::new(&bin).is_absolute() {
            PathBuf::from(bin)
        } else {
            root.join(bin)
        };
        Command::new(bin)
            .arg("--repo-root")
            .arg(root)
            .arg("--scm-facts")
            .arg(&scm_facts)
            .arg("--stdout")
            .arg("--face")
            .arg(face)
            .current_dir(root)
            .output()
            .expect("run producer binary")
    } else {
        Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()))
            .arg("run")
            .arg("--quiet")
            .arg("-p")
            .arg("oya-cloud-ci-accounting-registry-app")
            .arg("--")
            .arg("--repo-root")
            .arg(root)
            .arg("--scm-facts")
            .arg(&scm_facts)
            .arg("--stdout")
            .arg("--face")
            .arg(face)
            .current_dir(root)
            .output()
            .expect("cargo run oya-cloud-ci-accounting-registry-app")
    };
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}
