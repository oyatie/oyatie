// cloud-ci-license-policy live-corpus and evaluator tests. ADR-0083 Tier-3: integration tests
// assert via unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_license_policy_app::{Verdict, evaluate, evaluate_keyed};
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

fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root
        .join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json");
    let (mut command, run_description) = if let Ok(bin) = std::env::var("OYA_CI_PRODUCER_BIN") {
        let bin = if Path::new(&bin).is_absolute() {
            PathBuf::from(bin)
        } else {
            root.join(bin)
        };
        (Command::new(bin), "run producer binary")
    } else {
        let mut command =
            Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()));
        command
            .arg("run")
            .arg("--quiet")
            .arg("-p")
            .arg("oya-cloud-ci-accounting-registry-app")
            .arg("--");
        (command, "cargo run oya-cloud-ci-accounting-registry-app")
    };

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
        .expect(run_description);

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
}
