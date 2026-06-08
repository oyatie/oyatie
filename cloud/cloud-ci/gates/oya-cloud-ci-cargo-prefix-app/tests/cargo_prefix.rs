// ADR-0017 cloud-ci-cargo-prefix: born-blocking self-test over TODAY's real corpus. Runs the
// producer `--face cargo-prefix` to resolve the first-party oya-* workspace members + package
// names, then asserts the gate's verdict MATCHES the live corpus: if any member's crate-id or
// package name fails the required prefix (or they disagree) the gate is RED and freezes the
// debt; if every first-party crate already conforms the gate is cleanly GREEN. The count is
// MEASURED + reported, not hardcoded. ADR-0083 Tier-3: integration tests assert via
// unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use oya_cloud_ci_cargo_prefix_app::{evaluate, evaluate_keyed, Verdict};

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

fn run_producer_face(root: &Path, face: &str) -> Value {
    let output = Command::new(env!("CARGO"))
        .arg("run")
        .arg("--quiet")
        .arg("-p")
        .arg("oya-cloud-ci-accounting-registry-app")
        .arg("--")
        .arg("--repo-root")
        .arg(root)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run oya-cloud-ci-accounting-registry-app");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

#[test]
fn cargo_prefix_verdict_matches_the_live_corpus() {
    let root = repo_root();
    let face = run_producer_face(&root, "cargo-prefix");
    let rows = face["rows"].as_array().expect("cargo-prefix face rows");
    assert!(
        rows.len() > 500,
        "the cargo-prefix face should enumerate the workspace's oya-* members, got {}",
        rows.len()
    );

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    eprintln!(
        "BORN-BLOCKING cargo-prefix: oya-* members={} total_findings={} verdict={:?}",
        rows.len(),
        findings.len(),
        verdict
    );

    // The verdict is whatever the live corpus dictates: RED iff there is at least one violation,
    // GREEN iff every enumerated first-party crate conforms. We assert the verdict and the
    // findings set are CONSISTENT (no false-green): non-empty findings <=> RED.
    if findings.is_empty() {
        assert_eq!(
            verdict,
            Verdict::Green,
            "no findings must mean GREEN (the gate cleanly passes when every crate conforms)"
        );
    } else {
        assert_eq!(
            verdict,
            Verdict::Red,
            "findings present must mean RED (the gate fires + freezes the debt)"
        );
    }
}
