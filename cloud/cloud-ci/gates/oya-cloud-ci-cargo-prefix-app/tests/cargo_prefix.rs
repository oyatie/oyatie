// ADR-0017 cloud-ci-cargo-prefix: born-blocking self-test over TODAY's real corpus. Runs the
// producer `--face cargo-prefix` to resolve every in-scope first-party workspace member candidate
// + package name, then asserts the gate's verdict MATCHES the live corpus: if any member's
// crate-id or package name fails the required prefix (or they disagree) the gate is RED and
// freezes the debt; if every first-party crate already conforms the gate is cleanly GREEN. The
// count is MEASURED + reported, not hardcoded. ADR-0083 Tier-3: integration tests assert via
// unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use oya_cloud_ci_cargo_prefix_app::{Verdict, evaluate, evaluate_keyed};

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

fn hermetic_binary(root: &Path, env_name: &str, value: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = value else {
        return Err(format!(
            "FAIL-CLOSED: missing {env_name}; Cargo fallback is forbidden"
        ));
    };
    Ok(if Path::new(bin).is_absolute() {
        PathBuf::from(bin)
    } else {
        root.join(bin)
    })
}

#[test]
fn hermetic_binary_envs_are_required_for_gate() {
    let root = Path::new("/repo");
    let producer = hermetic_binary(root, "OYA_CI_PRODUCER_BIN", None)
        .expect_err("missing producer env must fail closed");
    let emitter = hermetic_binary(root, "OYA_CI_SCM_FACTS_EMITTER_BIN", None)
        .expect_err("missing emitter env must fail closed");
    assert!(producer.contains("OYA_CI_PRODUCER_BIN"));
    assert!(emitter.contains("OYA_CI_SCM_FACTS_EMITTER_BIN"));
}

/// Run the scm-facts emitter, then run the producer to emit a single face to stdout,
/// HERMETICALLY. The binaries must be provided by `OYA_CI_SCM_FACTS_EMITTER_BIN` and
/// `OYA_CI_PRODUCER_BIN`; missing env fails closed so tests cannot silently fall back to Cargo or
/// stale committed generated faces.
fn run_scm_facts_emitter(root: &Path) -> PathBuf {
    let scm_facts = std::env::temp_dir().join(format!(
        "oya-cargo-prefix-scm-facts-{}.generated.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&scm_facts);

    let emitter_bin = std::env::var("OYA_CI_SCM_FACTS_EMITTER_BIN").ok();
    let bin = hermetic_binary(root, "OYA_CI_SCM_FACTS_EMITTER_BIN", emitter_bin.as_deref())
        .unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(bin)
        .arg("--repo-root")
        .arg(root)
        .arg("--out")
        .arg(&scm_facts)
        .current_dir(root)
        .output()
        .expect("run scm-facts emitter binary");
    assert!(
        output.status.success(),
        "scm-facts emitter failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    scm_facts
}

fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = run_scm_facts_emitter(root);
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = hermetic_binary(root, "OYA_CI_PRODUCER_BIN", producer_bin.as_deref())
        .unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(bin)
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run producer binary");
    let _ = std::fs::remove_file(&scm_facts);
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
        "the cargo-prefix face should enumerate the workspace member candidates, got {}",
        rows.len()
    );

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    eprintln!(
        "BORN-BLOCKING cargo-prefix: member_candidates={} total_findings={} verdict={:?}",
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
