// cloud-ci-license-policy live-corpus and evaluator tests. ADR-0083 Tier-3: integration tests
// assert via unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use ci_license_policy::{Verdict, evaluate, evaluate_keyed};
use serde_json::{Value, json};

fn row(package_name: &str, license: Option<&str>) -> Value {
    json!({
        "package_name": package_name,
        "manifest_path": format!("libs/{package_name}/Cargo.toml"),
        "license": license,
    })
}

#[test]
fn accepted_workspace_licenses_are_green() {
    let input = json!({ "rows": [
        row("oya-good-domain", Some("Apache-2.0")),
        row("oya-mit-kernel", Some("MIT OR BSD-3-Clause")),
    ] });

    assert_eq!(evaluate(&input).verdict, Verdict::Green);
    assert!(evaluate_keyed(&input).is_empty());
}

#[test]
fn forbidden_review_unknown_and_missing_licenses_are_stable_findings() {
    let input = json!({ "rows": [
        row("oya-gpl-domain", Some("GPL-3.0")),
        row("oya-lgpl-domain", Some("LGPL-3.0")),
        row("oya-vendor-domain", Some("Vendor-Commercial")),
        row("oya-missing-domain", None),
        row("oya-blank-domain", Some("   ")),
    ] });

    let findings = evaluate_keyed(&input);
    let pairs = findings
        .iter()
        .map(|finding| (finding.code.as_str(), finding.key.as_str()))
        .collect::<Vec<_>>();

    assert!(pairs.contains(&("license_policy_forbidden_license", "oya-gpl-domain")));
    assert!(pairs.contains(&("license_policy_review_required", "oya-lgpl-domain")));
    assert!(pairs.contains(&("license_policy_unknown_license", "oya-vendor-domain")));
    assert!(pairs.contains(&("license_policy_missing_license", "oya-missing-domain")));
    assert!(pairs.contains(&("license_policy_missing_license", "oya-blank-domain")));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn empty_corpus_is_red_to_prevent_false_green() {
    let input = json!({ "rows": [] });
    let findings = evaluate_keyed(&input);
    assert_eq!(findings.len(), 1, "got {findings:?}");
    let finding = findings.iter().next().unwrap();
    assert_eq!(finding.code, "license_policy_no_workspace_members");
    assert_eq!(finding.key, "<empty-license-policy-corpus>");
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn evaluate_is_bare_projection_of_evaluate_keyed() {
    let input = json!({ "rows": [
        row("oya-gpl-domain", Some("GPL-3.0")),
        row("oya-vendor-domain", Some("Vendor-Commercial")),
    ] });
    let projected = evaluate_keyed(&input)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    assert_eq!(evaluate(&input).violations, projected);
}

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

fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = materialized_scm_facts(root);
    assert!(
        scm_facts.is_file(),
        "missing materialized scm-facts face at {}; run the producer-regen/materialization boundary before this gate",
        scm_facts.display()
    );
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let mut command = Command::new(
        producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}")),
    );

    let output = command
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
fn license_policy_verdict_matches_the_live_workspace() {
    let root = repo_root();
    let face = run_producer_face(&root, "license-policy");
    let rows = face["rows"].as_array().expect("license-policy face rows");
    assert!(
        rows.len() > 500,
        "the license-policy face should enumerate workspace packages, got {}",
        rows.len()
    );

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    eprintln!(
        "BORN-BLOCKING license-policy: workspace_packages={} total_findings={} verdict={verdict:?}",
        rows.len(),
        findings.len(),
    );

    if findings.is_empty() {
        assert_eq!(
            verdict,
            Verdict::Green,
            "no findings must mean GREEN (the gate cleanly passes when every package conforms)"
        );
    } else {
        assert_eq!(
            verdict,
            Verdict::Red,
            "blocking findings present must mean RED (the gate fires + freezes that scoped debt)"
        );
    }
}
