// GATE-2 cloud-ci-total-accounting: RED/GREEN fixture corpus + born-blocking live-corpus
// self-test. ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use oya_cloud_ci_total_accounting_app::{evaluate, Verdict};

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
    repo_root().join("specs/fixtures/total-accounting")
}

fn load_json(path: &PathBuf) -> Value {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
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

#[test]
fn total_accounting_fixtures_execute_red_green_cases() {
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
    assert!(!tc_paths.is_empty(), "total-accounting fixture corpus must not be empty");

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
                assert_eq!(
                    report.verdict,
                    Verdict::Red,
                    "{label} should be RED"
                );
                // The contract: report.violations EXACTLY equals expected_violations.
                assert_eq!(
                    report.violations, expected,
                    "{label} violations mismatch"
                );
            }
            other => panic!("{label} has unsupported expected_verdict {other:?}"),
        }
    }

    assert!(
        seen_green && seen_red,
        "total-accounting fixtures must include BOTH RED and GREEN cases"
    );
}

/// Born-blocking self-test: GATE-2 must go RED on TODAY's real corpus. Per the firewall
/// doctrine, "a firewall that doesn't block today is the facade we're killing." This runs
/// the producer over the live tree and asserts the gate flags the real defects:
/// - broadly `unowned` (live `find -name OWNERS` = 0 tree-wide)
/// - the oya-foundry-* residue as `unjustified` (ADR-0363 claims it was "eradicated")
/// - the oya-governance-* crates broadly `unreachable`
///
/// Counts are MEASURED, not hardcoded (the plan's 780/57 were not re-derived this session).
#[test]
fn gate2_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();

    // Regenerate the registry from the live tree via the producer binary (in --stdout mode).
    let registry = run_producer_stdout(&root);
    let rows = registry["rows"].as_array().expect("registry rows");
    assert!(
        rows.len() > 1000,
        "live registry should account thousands of paths, got {}",
        rows.len()
    );

    // Evaluate the live registry through the gate.
    let report = evaluate(&registry);

    // The live corpus is RED — and specifically for the real systemic gaps.
    assert_eq!(
        report.verdict,
        Verdict::Red,
        "GATE-2 MUST go RED on today's corpus (the firewall must block today)"
    );
    assert!(
        report.violations.contains("unowned"),
        "live corpus has 0 OWNERS files tree-wide -> unowned must fire"
    );
    assert!(
        report.violations.contains("unreachable"),
        "unwired governance crates -> unreachable must fire"
    );

    // Count the real exhibits for the evidence digest.
    let unowned = rows
        .iter()
        .filter(|r| r["owner"].is_null())
        .count();
    let unreachable = rows
        .iter()
        .filter(|r| {
            r["reachable_from"]
                .as_array()
                .map(Vec::is_empty)
                .unwrap_or(true)
        })
        .count();
    let foundry_residue = rows
        .iter()
        .filter(|r| {
            r["path"]
                .as_str()
                .is_some_and(|p| p.contains("oya-foundry"))
        })
        .count();
    let governance_unreachable = rows
        .iter()
        .filter(|r| {
            r["path"].as_str().is_some_and(|p| p.contains("oya-governance"))
                && r["reachable_from"]
                    .as_array()
                    .map(Vec::is_empty)
                    .unwrap_or(true)
        })
        .count();

    // These are the live exhibits; surface them so the test output is the evidence.
    eprintln!(
        "BORN-BLOCKING live-corpus counts: total_rows={} unowned={} unreachable={} foundry_residue={} governance_unreachable={}",
        rows.len(),
        unowned,
        unreachable,
        foundry_residue,
        governance_unreachable
    );

    assert!(unowned > 1000, "owner gap is systemic, got {unowned}");
}

fn run_producer_stdout(root: &Path) -> Value {
    // Use the workspace-built producer binary. cargo run keeps this hermetic to the
    // workspace toolchain; --stdout regenerates the registry without writing files.
    let output = Command::new(env!("CARGO"))
        .arg("run")
        .arg("--quiet")
        .arg("-p")
        .arg("oya-cloud-ci-accounting-registry-app")
        .arg("--")
        .arg("--repo-root")
        .arg(root)
        .arg("--stdout")
        .current_dir(root)
        .output()
        .expect("run oya-cloud-ci-accounting-registry-app");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer stdout is valid JSON")
}
