// §2.5#7 cloud-ci-manifest-hygiene: born-blocking self-test over TODAY's real corpus. Runs the
// producer `--face manifest-hygiene` to resolve the per-crate manifest flags, then asserts the
// gate FIRES — some first-party oya-* crates miss a §2.5#7 field today (the frozen baseline,
// shrink-only). The count is MEASURED, not hardcoded. ADR-0083 Tier-3: integration tests assert
// via unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use oya_cloud_ci_manifest_hygiene_app::{Verdict, evaluate, evaluate_keyed};

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
fn manifest_hygiene_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();
    let face = run_producer_face(&root, "manifest-hygiene");
    let rows = face["rows"].as_array().expect("manifest-hygiene face rows");
    assert!(
        rows.len() > 500,
        "the manifest-hygiene face should enumerate the workspace's oya-* crates, got {}",
        rows.len()
    );

    let findings = evaluate_keyed(&face);
    eprintln!(
        "BORN-BLOCKING manifest-hygiene: oya-* crates={} total_findings={}",
        rows.len(),
        findings.len()
    );

    assert_eq!(
        evaluate(&face).verdict,
        Verdict::Red,
        "GATE must go RED on today's corpus (some crates miss a §2.5#7 field)"
    );
    assert!(
        !findings.is_empty(),
        "the live corpus must surface at least one manifest-hygiene violation"
    );
}
