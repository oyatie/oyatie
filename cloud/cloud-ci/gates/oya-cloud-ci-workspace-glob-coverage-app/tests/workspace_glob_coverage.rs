// ADR-0538 cloud-ci-workspace-glob-coverage live-corpus gate. Runs the producer
// `--face workspace-glob-coverage`, then asserts the verdict is consistent with current
// findings. ADR-0083 Tier-3: integration tests assert with unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_workspace_glob_coverage_app::{Verdict, evaluate, evaluate_keyed};
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
fn workspace_glob_coverage_verdict_matches_the_live_corpus() {
    let root = repo_root();
    let face = run_producer_face(&root, "workspace-glob-coverage");
    let rows = face["rows"].as_array().expect("workspace-glob-coverage face rows");
    assert!(
        rows.len() > 500,
        "workspace-glob-coverage should enumerate root members + crate dirs, got {}",
        rows.len()
    );

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    eprintln!(
        "BORN-BLOCKING workspace-glob-coverage: rows={} total_findings={} verdict={:?}",
        rows.len(),
        findings.len(),
        verdict
    );

    if findings.is_empty() {
        assert_eq!(verdict, Verdict::Green, "no findings must mean GREEN");
    } else {
        assert_eq!(verdict, Verdict::Red, "findings present must mean RED");
    }
}
