// cloud-ci-generated-artifact-control-plane live-corpus gate.
//
// The test is intentionally hermetic and product-shaped: read the repo-authored generated
// artifact policy manifest plus the committed SCM facts snapshot, then run the Rust predicate.
// It does not call git, does not invoke a CI-provider API, and does not depend on a local merge
// driver. That makes the same test shape portable to any project adopting oya-ci.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use oya_cloud_ci_generated_artifact_control_plane_app::{Verdict, evaluate, evaluate_keyed};

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

fn read_json(path: PathBuf) -> Value {
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("parse {} as JSON: {e}", path.display()))
}

#[test]
fn live_generated_artifacts_are_declared_in_the_control_plane() {
    let root = repo_root();
    let manifest = read_json(root.join("registry/generated-artifact-control-plane.json"));
    let scm_facts = read_json(root.join(
        "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/git-facts.generated.json",
    ));

    let findings = evaluate_keyed(&manifest, &scm_facts);
    assert_eq!(
        evaluate(&manifest, &scm_facts).verdict,
        Verdict::Green,
        "generated-artifact control-plane findings: {findings:#?}"
    );
}
