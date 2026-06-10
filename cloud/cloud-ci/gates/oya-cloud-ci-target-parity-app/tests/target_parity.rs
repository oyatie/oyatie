// cloud-ci-target-parity: born-blocking self-test over TODAY's real corpus. Runs the
// accounting-registry producer `--face target-parity`, then asserts the measured G011 debt:
// all workspace members have BUCK files, while 614 members with Rust test code lack rust_test.
// ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_target_parity_app::{Verdict, evaluate, evaluate_keyed};
use serde_json::Value;

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

#[test]
fn target_parity_face_reports_live_corpus_debt() {
    let root = repo_root();
    let face = run_producer_face(&root, "target-parity");
    let rows = face["rows"].as_array().expect("target-parity face rows");
    assert!(
        rows.len() >= 817,
        "the target-parity face should enumerate at least the G011 base workspace members, got {}",
        rows.len()
    );

    let findings = evaluate_keyed(&face);
    let missing_buck: BTreeSet<String> = findings
        .iter()
        .filter(|finding| finding.code == "member_missing_buck")
        .map(|finding| finding.key.clone())
        .collect();
    let unwired_tests: BTreeSet<String> = findings
        .iter()
        .filter(|finding| finding.code == "member_test_code_without_rust_test_target")
        .map(|finding| finding.key.clone())
        .collect();

    eprintln!(
        "TARGET-PARITY live corpus: members={} member_missing_buck={} member_test_code_without_rust_test_target={}",
        rows.len(),
        missing_buck.len(),
        unwired_tests.len()
    );

    assert!(
        missing_buck.is_empty(),
        "member_missing_buck is born-blocking empty today: {missing_buck:?}"
    );
    assert_eq!(
        unwired_tests.len(),
        614,
        "G011 baseline debt must be exactly the mechanically-derived set"
    );
    assert_eq!(evaluate(&face).verdict, Verdict::Red);
}
