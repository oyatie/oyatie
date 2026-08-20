// cloud-ci-catalog-liveness live-corpus gate. Runs the producer `--face catalog-liveness`, then
// asserts the gate verdict matches the current registry catalog corpus (born-blocking with an
// EMPTY frozen baseline: post-PR-C1/PR-C2 there are ZERO silently-stale records). The RED-fixture
// leg synthesizes a fake dead-unmarked record and asserts the gate goes RED — proving the gate
// DISCRIMINATES (it is not inert/always-green). ADR-0083 Tier-3: integration tests assert with
// unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use ci_service_catalog_parity::{Verdict, evaluate, evaluate_keyed};
use serde_json::{Value, json};

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

#[test]
fn retired_cloud_os_domain_catalog_rows_do_not_return() {
    let root = repo_root();
    for relative in [
        "registry/catalog/oya-cloud-os-cluster-mgmt-domain.yaml",
        "registry/catalog/oya-cloud-os-kubernetes-domain.yaml",
        "registry/catalog/oya-cloud-os-secrets-domain.yaml",
        "registry/catalog/oya-cloud-os-trustd-domain.yaml",
    ] {
        assert!(
            !root.join(relative).exists(),
            "deleted cloud-os domain catalog identity must not be revived or laundered with a non-live marker: {relative}"
        );
    }
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
fn catalog_liveness_verdict_matches_the_live_catalog() {
    let root = repo_root();
    let face = run_producer_face(&root, "catalog-liveness");
    let rows = face["rows"].as_array().expect("catalog-liveness face rows");
    let live_crates = face["live_crates"]
        .as_array()
        .expect("catalog-liveness face live_crates");
    assert!(
        rows.len() > 500,
        "the catalog-liveness face should enumerate the catalog, got {}",
        rows.len()
    );
    assert!(
        live_crates.len() > 100,
        "the catalog-liveness face should enumerate governed live crates, got {}",
        live_crates.len()
    );

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    let live = rows
        .iter()
        .filter(|r| r["is_live"].as_bool() == Some(true))
        .count();
    let marked = rows
        .iter()
        .filter(|r| {
            r["is_live"].as_bool() != Some(true)
                && r["marker"].as_str().is_some_and(|m| !m.trim().is_empty())
        })
        .count();
    eprintln!(
        "BORN-BLOCKING catalog-liveness: catalog_records={} live={} dead_but_marked={} \
         governed_live_crates={} total_findings={} verdict={:?}",
        rows.len(),
        live,
        marked,
        live_crates.len(),
        findings.len(),
        verdict
    );

    assert!(
        findings.is_empty(),
        "every catalog record must be live OR explicitly marked non-live: {findings:?}"
    );
    assert_eq!(verdict, Verdict::Green);
}

/// RED-fixture (mandatory, proves the gate is NOT inert): inject ONE synthetic dead-unmarked
/// record into the live face and assert the gate goes RED with the synthetic key surfaced. Without
/// this the gate could be silently always-green.
#[test]
fn synthetic_dead_unmarked_record_makes_the_gate_red() {
    let root = repo_root();
    let mut face = run_producer_face(&root, "catalog-liveness");

    let rows = face["rows"]
        .as_array_mut()
        .expect("catalog-liveness face rows");
    rows.push(json!({
        "crate_id": "synthetic-dead-unmarked-cap",
        "source_path": "registry/catalog/synthetic-dead-unmarked-cap.yaml",
        "is_live": false,
        "marker": Value::Null,
    }));

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    eprintln!(
        "RED-FIXTURE catalog-liveness: injected synthetic-dead-unmarked-cap -> findings={:?} verdict={:?}",
        findings, verdict
    );

    assert_eq!(
        verdict,
        Verdict::Red,
        "a synthetic dead-unmarked record must make the gate RED"
    );
    assert!(
        findings.iter().any(|f| {
            f.code == "catalog_record_no_live_crate_unmarked"
                && f.key == "synthetic-dead-unmarked-cap"
        }),
        "the synthetic dead-unmarked record must be surfaced as a finding: {findings:?}"
    );
}

#[test]
fn synthetic_stale_source_crate_makes_the_gate_red() {
    let root = repo_root();
    let mut face = run_producer_face(&root, "catalog-liveness");

    let rows = face["rows"]
        .as_array_mut()
        .expect("catalog-liveness face rows");
    rows.push(json!({
        "crate_id": "synthetic-live-stale-source",
        "source_path": "registry/catalog/synthetic-live-stale-source.yaml",
        "is_live": true,
        "marker": Value::Null,
        "source_crate": "crates/old-synthetic-live-stale-source/Cargo.toml",
        "source_crate_exists": false,
    }));
    let live_crates = face["live_crates"]
        .as_array_mut()
        .expect("catalog-liveness face live_crates");
    live_crates.push(json!({
        "crate_id": "synthetic-live-stale-source",
        "member_path": "synthetic/live-stale-source",
        "has_catalog_row": true,
        "exemption": Value::Null,
    }));

    let findings = evaluate_keyed(&face);
    assert!(
        findings.iter().any(|f| {
            f.code == "catalog_record_source_crate_missing"
                && f.key == "synthetic-live-stale-source"
        }),
        "the synthetic stale source_crate must be surfaced as a finding: {findings:?}"
    );
    assert_eq!(evaluate(&face).verdict, Verdict::Red);
}

#[test]
fn synthetic_live_crate_without_row_makes_the_gate_red() {
    let root = repo_root();
    let mut face = run_producer_face(&root, "catalog-liveness");

    let live_crates = face["live_crates"]
        .as_array_mut()
        .expect("catalog-liveness face live_crates");
    live_crates.push(json!({
        "crate_id": "synthetic-live-without-row",
        "member_path": "synthetic/live-without-row",
        "has_catalog_row": false,
        "exemption": Value::Null,
    }));

    let findings = evaluate_keyed(&face);
    assert!(
        findings.iter().any(|f| {
            f.code == "catalog_live_crate_without_row" && f.key == "synthetic-live-without-row"
        }),
        "the synthetic live crate without row must be surfaced as a finding: {findings:?}"
    );
    assert_eq!(evaluate(&face).verdict, Verdict::Red);
}

/// A dead BUT explicitly-marked synthetic record stays GREEN — the gate enforces
/// live-OR-MARKED, not live-only (it must not false-RED the legitimately-marked records).
#[test]
fn synthetic_dead_marked_record_keeps_the_gate_green() {
    let root = repo_root();
    let mut face = run_producer_face(&root, "catalog-liveness");

    let rows = face["rows"]
        .as_array_mut()
        .expect("catalog-liveness face rows");
    rows.push(json!({
        "crate_id": "synthetic-dead-marked-cap",
        "source_path": "registry/catalog/synthetic-dead-marked-cap.yaml",
        "is_live": false,
        "marker": "retired-compatibility-row-no-crate",
        "source_crate": "crates/synthetic-dead-marked-cap/Cargo.toml",
        "source_crate_exists": false,
    }));

    let findings = evaluate_keyed(&face);
    assert!(
        findings.is_empty(),
        "a dead BUT marked record must keep the gate GREEN: {findings:?}"
    );
    assert_eq!(evaluate(&face).verdict, Verdict::Green);
}
