// ADR-0017 cloud-ci-cargo-prefix: scoped self-test over TODAY's real corpus. Runs the producer
// `--face cargo-prefix` to resolve every in-scope first-party workspace member candidate +
// package name + cargo_prefix_scope, then asserts the gate's verdict matches the blocking-scoped
// findings. Advisory-scoped de-branded candidates remain visible coverage but do not create
// born-blocking baseline debt. The count is MEASURED + reported, not hardcoded. ADR-0083 Tier-3:
// integration tests assert via unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use ci_crate_name_prefix::{Verdict, evaluate, evaluate_keyed};

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

fn producer_binary(root: &Path, value: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = value else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    ci_path_resolver_adapters::resolve_cargo_test_binary(root, std::ffi::OsStr::new(bin))
}

fn materialized_scm_facts(root: &Path) -> PathBuf {
    root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json")
}

#[test]
fn producer_binary_env_is_required_for_gate() {
    let root = Path::new("/repo");
    let producer = producer_binary(root, None).expect_err("missing producer env must fail closed");
    assert!(producer.contains("OYA_CI_PRODUCER_BIN"));
}

/// Run the producer to emit a single face to stdout from the materialized scm-facts snapshot.
/// The test deliberately does not run the scm-facts emitter: that binary is the single ambient-git
/// boundary and must run before gate tests, not inside this `rust_test`.
fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = materialized_scm_facts(root);
    assert!(
        scm_facts.is_file(),
        "missing materialized scm-facts face at {}; run the producer-regen/materialization boundary before this gate",
        scm_facts.display()
    );
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}"));
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

    let advisory_rows = rows
        .iter()
        .filter(|row| row.get("cargo_prefix_scope").and_then(Value::as_str) == Some("advisory"))
        .count();

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    eprintln!(
        "cargo-prefix: member_candidates={} advisory_candidates={} blocking_findings={} verdict={:?}",
        rows.len(),
        advisory_rows,
        findings.len(),
        verdict
    );

    // The verdict follows the blocking-scoped findings only: advisory de-brand candidates are
    // coverage rows, not baseline-block-on-new debt. Assert consistency (no false-green):
    // non-empty blocking findings <=> RED.
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
            "blocking findings present must mean RED (the gate fires + freezes that scoped debt)"
        );
    }
}
