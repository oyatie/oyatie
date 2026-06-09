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

const MANIFEST_ENV: &str = "OYA_CI_GENERATED_ARTIFACT_MANIFEST";
const SCM_FACTS_ENV: &str = "OYA_CI_GENERATED_ARTIFACT_SCM_FACTS";

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

fn input_path(env_name: &str, repo_relative_path: &str) -> PathBuf {
    if let Some(path) = std::env::var_os(env_name).filter(|value| !value.is_empty()) {
        return PathBuf::from(path);
    }

    repo_root().join(repo_relative_path)
}

fn read_json(path: PathBuf) -> Value {
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("parse {} as JSON: {e}", path.display()))
}

#[test]
fn live_generated_artifacts_are_declared_in_the_control_plane() {
    let manifest = read_json(input_path(
        MANIFEST_ENV,
        "registry/generated-artifact-control-plane.json",
    ));
    let scm_facts = read_json(input_path(
        SCM_FACTS_ENV,
        "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json",
    ));

    let findings = evaluate_keyed(&manifest, &scm_facts);
    assert_eq!(
        evaluate(&manifest, &scm_facts).verdict,
        Verdict::Green,
        "generated-artifact control-plane findings: {findings:#?}"
    );
}
