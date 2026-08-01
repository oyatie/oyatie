#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use ci_reset_eligibility_policy::{evaluate, evaluate_schema};
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
    panic!("failed to locate repo root")
}

fn read_json(path: &std::path::Path) -> Value {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

#[test]
fn live_w0d_observation_is_fail_closed_and_requires_preservation_migration() {
    let root = repo_root();
    let policy =
        read_json(&root.join("ci/facade/reset-eligibility-policy/reset-eligibility-policy.json"));
    let artifact_path = policy["artifact_path"].as_str().expect("artifact_path");
    let schema_path = policy["schema_path"].as_str().expect("schema_path");
    let schema = read_json(&root.join(schema_path));
    let artifact = read_json(&root.join(artifact_path));
    let mut findings = evaluate_schema(&policy, &schema);
    findings.extend(evaluate(&policy, &artifact, 1_785_610_769));
    assert!(
        findings.is_empty(),
        "reset eligibility gate RED:\n{}",
        findings
            .iter()
            .map(|finding| format!("{} {}: {}", finding.code, finding.key, finding.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(artifact["decision"]["eligible"], false);
    assert_eq!(artifact["decision"]["mode"], "preservation-migration");
}
