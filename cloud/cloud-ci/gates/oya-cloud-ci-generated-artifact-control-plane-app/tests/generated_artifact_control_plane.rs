// cloud-ci-generated-artifact-control-plane live-corpus gate.
//
// The test is intentionally hermetic and product-shaped: read the repo-authored generated
// artifact policy manifest plus the SCM facts snapshot materialized from the candidate tree,
// then run the Rust predicate. It does not call git, does not invoke a CI-provider API, and does
// not depend on a local merge driver. That makes the same test shape portable to any project
// adopting oya-ci.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};

use oya_cloud_ci_generated_artifact_control_plane_app::{
    Verdict, evaluate_keyed, evaluate_keyed_with_frozen_references,
    evaluate_with_frozen_references, frozen_reference_face_paths_keyed,
};

const MANIFEST_ENV: &str = "OYA_CI_GENERATED_ARTIFACT_MANIFEST";
const SCM_FACTS_ENV: &str = "OYA_CI_GENERATED_ARTIFACT_SCM_FACTS";
// The firewall ratchet policy is the authoritative, repo-agnostic source of the frozen-reference
// set (ADR-0551 `frozen_reference.face_path`). Adopters override the location; the default is the
// committed oyatie firewall policy.
const RATCHET_POLICY_ENV: &str = "OYA_CI_GENERATED_ARTIFACT_RATCHET_POLICY";
const RATCHET_POLICY_DEFAULT_PATH: &str =
    "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/ratchet-policy.json";

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

fn live_frozen_reference_paths() -> BTreeSet<String> {
    let ratchet_policy = read_json(input_path(RATCHET_POLICY_ENV, RATCHET_POLICY_DEFAULT_PATH));
    let (paths, findings) = frozen_reference_face_paths_keyed(std::iter::once(&ratchet_policy));
    assert!(
        findings.is_empty(),
        "ratchet policy frozen-reference findings: {findings:#?}"
    );
    paths
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
    let frozen_reference_paths = live_frozen_reference_paths();

    let findings =
        evaluate_keyed_with_frozen_references(&manifest, &scm_facts, &frozen_reference_paths);
    assert_eq!(
        evaluate_with_frozen_references(&manifest, &scm_facts, &frozen_reference_paths).verdict,
        Verdict::Green,
        "generated-artifact control-plane findings: {findings:#?}"
    );
}

#[test]
fn live_firewall_frozen_reference_must_stay_committed() {
    // #828 make-it-impossible guard, evaluated live against the committed ratchet policy + manifest.
    // The firewall frozen-reference face MUST NOT be declared de-commit class in the control plane:
    // de-committing it empties the merge-base ratchet baseline (frozen_empty) and deadlocks the
    // merge queue (the #828 dev regression, hotfixed by #830).
    let manifest = read_json(input_path(
        MANIFEST_ENV,
        "registry/generated-artifact-control-plane.json",
    ));
    let scm_facts = read_json(input_path(
        SCM_FACTS_ENV,
        "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json",
    ));
    let frozen_reference_paths = live_frozen_reference_paths();
    assert!(
        !frozen_reference_paths.is_empty(),
        "ratchet policy must declare at least one frozen reference"
    );

    let findings =
        evaluate_keyed_with_frozen_references(&manifest, &scm_facts, &frozen_reference_paths);
    assert!(
        !findings
            .iter()
            .any(|finding| finding.code == "frozen_reference_artifact_must_stay_committed"),
        "a firewall frozen reference is declared de-commit class in the control plane; findings: {findings:#?}"
    );
}

#[test]
fn stale_scm_facts_for_deleted_generated_outputs_are_red() {
    let manifest = read_json(input_path(
        MANIFEST_ENV,
        "registry/generated-artifact-control-plane.json",
    ));
    let stale_paths = [
        "oya/app-shell-frontend/generated/hr-api.d.ts",
        "oya/app-shell-frontend/generated/ops-workspace-shell-v1.patched.yaml",
        "oya/app-shell-frontend/generated/ops-workspace-shell.d.ts",
    ];
    let scm_facts = json!({
        "schema": "oya-ci/scm-facts/v1",
        "tracked_paths": stale_paths,
        "last_touch_commit": {}
    });

    let findings = evaluate_keyed(&manifest, &scm_facts);
    for stale_path in stale_paths {
        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_tracked_generated_output_not_declared"
                    && finding.key == stale_path
            }),
            "stale generated path {stale_path} must stay visible to the gate; findings: {findings:#?}"
        );
    }
}

#[test]
fn live_manifest_covers_gitignore_generated_output_conventions() {
    let manifest = read_json(input_path(
        MANIFEST_ENV,
        "registry/generated-artifact-control-plane.json",
    ));
    let generated_convention_paths = [
        "app/generated/client.d.ts",
        "app/__generated__/client.ts",
        "jvm/generated-sources/Foo.java",
        "sdk/generated-types/client.ts",
        "bridge/gen/client.rs",
        "registry/example.generated.json",
        "openapi/client.generated.ts",
        "openapi/client.generated.d.ts",
        "proto/example.pb.go",
        "proto/example.pb.rs",
    ];
    let scm_facts = json!({
        "schema": "oya-ci/scm-facts/v1",
        "tracked_paths": generated_convention_paths,
        "last_touch_commit": {}
    });

    let findings = evaluate_keyed(&manifest, &scm_facts);
    for path in generated_convention_paths {
        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_tracked_generated_output_not_declared"
                    && finding.key == path
            }),
            "generated convention path {path} must stay visible to the control-plane gate; findings: {findings:#?}"
        );
    }

    let gitkeep_scm_facts = json!({
        "schema": "oya-ci/scm-facts/v1",
        "tracked_paths": ["app/generated/.gitkeep"],
        "last_touch_commit": {}
    });
    let gitkeep_findings = evaluate_keyed(&manifest, &gitkeep_scm_facts);
    assert!(
        !gitkeep_findings.iter().any(|finding| {
            finding.code == "generated_artifact_tracked_generated_output_not_declared"
                && finding.key == "app/generated/.gitkeep"
        }),
        "placeholder .gitkeep files must remain excluded from generated-output findings; findings: {gitkeep_findings:#?}"
    );
}
