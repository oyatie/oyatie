// §2.5#4 cloud-ci-bnf-layer-suffix: born-blocking self-test over TODAY's real corpus.
// Per the firewall doctrine ("a firewall that doesn't block today is the facade we're killing"),
// this runs the producer `--face bnf-layer-suffix` to resolve the live first-party oya-* crate
// names, then asserts the gate FIRES — there are non-canonical trailing segments in the tree
// today (the ~79 BNF-debt crates, baseline-block-on-new, burned down before L1 office). The
// count is MEASURED, not hardcoded. ADR-0083 Tier-3: integration tests assert via unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use oya_cloud_ci_bnf_layer_suffix_app::{evaluate, evaluate_keyed, Verdict};

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
/// git-facts face (a declared input); it never calls git.
fn run_producer_face(root: &Path, face: &str) -> Value {
    let git_facts =
        root.join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/git-facts.generated.json");
    let output = if let Ok(bin) = std::env::var("OYA_CI_PRODUCER_BIN") {
        let bin = if Path::new(&bin).is_absolute() { PathBuf::from(bin) } else { root.join(bin) };
        Command::new(bin)
            .arg("--repo-root")
            .arg(root)
            .arg("--git-facts")
            .arg(&git_facts)
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
            .arg("--git-facts")
            .arg(&git_facts)
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
fn bnf_layer_suffix_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();
    let face = run_producer_face(&root, "bnf-layer-suffix");
    let rows = face["rows"].as_array().expect("bnf face rows");
    assert!(
        rows.len() > 500,
        "the bnf face should enumerate the workspace's oya-* crates, got {}",
        rows.len()
    );

    let findings = evaluate_keyed(&face);
    let unknown_role = findings
        .iter()
        .filter(|f| f.code == "bnf_unknown_role")
        .count();

    eprintln!(
        "BORN-BLOCKING bnf-layer-suffix: oya-* crates={} total_findings={} bnf_unknown_role={}",
        rows.len(),
        findings.len(),
        unknown_role
    );

    assert_eq!(
        evaluate(&face).verdict,
        Verdict::Red,
        "GATE must go RED on today's corpus (non-canonical trailing segments exist)"
    );
    assert!(
        unknown_role > 0,
        "the live corpus must surface at least one non-canonical layer suffix (bnf_unknown_role)"
    );
}
