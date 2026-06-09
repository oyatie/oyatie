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

/// Run the producer to emit a single face to stdout, HERMETICALLY (no `env!("CARGO")`, the
/// compile-time cargo-only macro that breaks the buck2 build). The producer binary is resolved
/// at RUNTIME: under buck2 from `OYA_CI_PRODUCER_BIN` (the `$(exe ...)`-substituted built
/// binary), else under cargo via the runtime `CARGO` env var. The producer reads the committed
/// scm-facts face (a declared input); it never calls git.
fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts =
        root.join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json");
    let output = if let Ok(bin) = std::env::var("OYA_CI_PRODUCER_BIN") {
        let bin = if Path::new(&bin).is_absolute() { PathBuf::from(bin) } else { root.join(bin) };
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
