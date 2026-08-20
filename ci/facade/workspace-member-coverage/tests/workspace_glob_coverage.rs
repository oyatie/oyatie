// ADR-0538 cloud-ci-workspace-glob-coverage live-corpus gate. Runs the producer
// `--face workspace-glob-coverage`, then asserts the verdict is consistent with current
// findings. ADR-0083 Tier-3: integration tests assert with unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_workspace_member_coverage::{Verdict, evaluate, evaluate_keyed};
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
fn workspace_glob_coverage_verdict_matches_the_live_corpus() {
    let root = repo_root();
    let face = run_producer_face(&root, "workspace-glob-coverage");
    let rows = face["rows"]
        .as_array()
        .expect("workspace-glob-coverage face rows");
    assert!(
        rows.len() > 500,
        "workspace-glob-coverage should enumerate root members + crate dirs, got {}",
        rows.len()
    );

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    eprintln!(
        "workspace-glob-coverage: rows={} total_findings={} verdict={:?}",
        rows.len(),
        findings.len(),
        verdict
    );

    // The kernel's findings and verdict must agree. This is a real property, but it is a
    // property of the KERNEL, not of the tree — it holds whether the tree has 0 findings or
    // 800. On its own it was the whole assertion, which is why this gate printed
    // "BORN-BLOCKING ... verdict=Red" and exited 0 with seven live violations standing.
    if findings.is_empty() {
        assert_eq!(verdict, Verdict::Green, "no findings must mean GREEN");
    } else {
        assert_eq!(verdict, Verdict::Red, "findings present must mean RED");
    }

    // The assertion that is actually about the tree.
    let baseline = load_baseline(&root);
    let live: BTreeSet<String> = findings
        .iter()
        .map(|finding| format!("{}::{}", finding.code, finding.key))
        .collect();

    let unbaselined: Vec<&String> = live.difference(&baseline).collect();
    assert!(
        unbaselined.is_empty(),
        "{} workspace-glob-coverage violation(s) not in {}: {:#?}\n\
         Each is a crate manifest outside the resolved member set, a non-glob members entry, \
         or a malformed row. A crate that is not a workspace member is not compiled by \
         `cargo nextest run --workspace`, so it is invisible to EVERY gate at once.",
        unbaselined.len(),
        BASELINE_REL,
        unbaselined,
    );

    // Two-sided. A frozen key that stopped being produced is either debt genuinely paid off
    // or a scan that collapsed, and a one-sided rule cannot tell those apart — so the burn-down
    // must be re-frozen in the same change, as a reviewable diff of named keys.
    let disappeared: Vec<&String> = baseline.difference(&live).collect();
    assert!(
        disappeared.is_empty(),
        "{} baselined violation(s) are no longer produced: {:#?}\n\
         If they were fixed, delete them from {} in this same change. If the scan narrowed \
         instead, that is the regression this assertion exists to catch.",
        disappeared.len(),
        disappeared,
        BASELINE_REL,
    );
}

const BASELINE_REL: &str = "ci/facade/workspace-member-coverage/workspace-glob-coverage-baseline.json";

fn load_baseline(root: &Path) -> BTreeSet<String> {
    let path = root.join(BASELINE_REL);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let parsed: Value = serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
    let frozen = parsed["frozen"]
        .as_array()
        .unwrap_or_else(|| panic!("{} must carry a `frozen` array", path.display()));
    assert!(
        !frozen.is_empty(),
        "{} has an empty `frozen` array. That is legal only when the tree is genuinely clean; \
         if it was emptied to silence this gate, restore it.",
        path.display()
    );
    frozen
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("{} frozen entries must be strings", path.display()))
                .to_owned()
        })
        .collect()
}
