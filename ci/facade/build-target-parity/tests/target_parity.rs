// cloud-ci-target-parity: born-blocking self-test over TODAY's real corpus. Runs the
// accounting-registry producer `--face target-parity`, then asserts the measured G011 debt
// equals the committed gate baseline exactly (set equality, not a hardcoded count): all
// workspace members have BUCK files, and the unwired-test set matches the frozen baseline
// face byte-for-byte, so wiring PRs must settle the baseline in the same change (FRIC-1781116000).
// ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_build_target_parity::{Verdict, evaluate, evaluate_keyed};
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

fn producer_binary(root: &Path, producer_bin: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = producer_bin else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    ci_path_resolver_adapters::resolve_cargo_test_binary(root, std::ffi::OsStr::new(bin))
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_binary(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(err.contains("OYA_CI_PRODUCER_BIN"));
}

fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
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
    let baseline_path =
        root.join("ci/facade/artifact-inventory-registry/gate-baseline.generated.json");
    let baseline: Value = serde_json::from_slice(
        &std::fs::read(&baseline_path).expect("read committed gate baseline"),
    )
    .expect("gate baseline is valid JSON");
    let baseline_keys: BTreeSet<String> = baseline["gates"]["cloud-ci-target-parity"]
        ["member_test_code_without_rust_test_target"]["keys"]
        .as_array()
        .expect("baseline keys array")
        .iter()
        .map(|key| key.as_str().expect("baseline key is a string").to_owned())
        .collect();
    assert_eq!(
        unwired_tests, baseline_keys,
        "measured G011 debt must equal the committed baseline exactly; wiring changes must \
         regenerate the baseline face in the same PR (settle protocol)"
    );
    // Independent growth tripwire (codex review of PR #676, FRIC-1781112000): the
    // merge-base ratchet now provides the structural comparison — the cloud-ci-firewall
    // gate evaluates this code's keys against the gate-baseline face frozen at
    // `git merge-base <base_ref> HEAD` (ADR-0551), so a same-PR baseline regen can no
    // longer launder new unwired-test debt. This ceiling stays as defense-in-depth: it is
    // NOT derived from any generated artifact and only ever moves DOWN, via an explicitly
    // reviewed edit. Slice PRs never touch it.
    const DEBT_CEILING: usize = 565;
    assert!(
        unwired_tests.len() <= DEBT_CEILING,
        "G011 debt grew past the reviewed ceiling ({} > {DEBT_CEILING}); new unwired-test \
         debt is born-blocking — wire the rust_test target instead of regenerating the baseline",
        unwired_tests.len()
    );
    // Unconditionally Red while the campaign runs: flipping to Green is a one-way
    // transition that must be its own reviewed change, never an emergent side effect
    // of producer/baseline drift (codex review of PR #676, finding 2).
    assert_eq!(evaluate(&face).verdict, Verdict::Red);
}
