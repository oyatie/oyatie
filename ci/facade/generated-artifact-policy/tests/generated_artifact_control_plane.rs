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

use ci_generated_artifact_policy::{
    FROZEN_REFERENCE_SOURCE_REGENERATE, Verdict, evaluate_keyed,
    evaluate_keyed_with_frozen_references, evaluate_with_frozen_references,
    frozen_reference_face_paths_keyed,
};

const MANIFEST_ENV: &str = "OYA_CI_GENERATED_ARTIFACT_MANIFEST";
const SCHEMA_ENV: &str = "OYA_CI_GENERATED_ARTIFACT_SCHEMA";
const SCM_FACTS_ENV: &str = "OYA_CI_GENERATED_ARTIFACT_SCM_FACTS";
// The firewall ratchet policy is the authoritative, repo-agnostic source of the frozen-reference
// set (ADR-0551 `frozen_reference.face_path`). Adopters override the location; the default is the
// committed oyatie firewall policy.
const RATCHET_POLICY_ENV: &str = "OYA_CI_GENERATED_ARTIFACT_RATCHET_POLICY";
const RATCHET_POLICY_DEFAULT_PATH: &str = "ci/facade/baseline-ratchet/ratchet-policy.json";

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

fn schema_enum_values<'a>(schema: &'a Value, property: &str) -> BTreeSet<&'a str> {
    schema
        .get("$defs")
        .and_then(|defs| defs.get("artifact"))
        .and_then(|artifact| artifact.get("properties"))
        .and_then(|properties| properties.get(property))
        .and_then(|property| property.get("enum"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("schema missing artifact.{property}.enum"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("schema artifact.{property}.enum entry is not a string"))
        })
        .collect()
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
fn live_schema_accepts_manifest_materialization_modes() {
    let manifest = read_json(input_path(
        MANIFEST_ENV,
        "registry/generated-artifact-control-plane.json",
    ));
    let schema = read_json(input_path(
        SCHEMA_ENV,
        "specs/generated-artifact-control-plane.schema.json",
    ));
    let allowed_materialization_modes = schema_enum_values(&schema, "materialization_mode");
    let allowed_merge_policies = schema_enum_values(&schema, "merge_policy");

    assert!(
        allowed_materialization_modes.contains("not-tracked-in-git"),
        "schema must declare the ADR-0595/ADR-0604 de-commit materialization mode"
    );

    let artifacts = manifest
        .get("artifacts")
        .and_then(Value::as_array)
        .expect("live generated-artifact manifest must contain artifacts");
    for artifact in artifacts {
        let artifact_id = artifact
            .get("artifact_id")
            .and_then(Value::as_str)
            .expect("live generated-artifact row must have artifact_id");
        let materialization_mode = artifact
            .get("materialization_mode")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{artifact_id} missing materialization_mode"));
        assert!(
            allowed_materialization_modes.contains(materialization_mode),
            "{artifact_id} uses materialization_mode {materialization_mode:?} not declared by specs/generated-artifact-control-plane.schema.json"
        );

        let merge_policy = artifact
            .get("merge_policy")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{artifact_id} missing merge_policy"));
        assert!(
            allowed_merge_policies.contains(merge_policy),
            "{artifact_id} uses merge_policy {merge_policy:?} not declared by specs/generated-artifact-control-plane.schema.json"
        );
    }
}

#[test]
fn history_only_retirement_facts_is_the_exact_controller_owned_untracked_face() {
    let manifest = read_json(input_path(
        MANIFEST_ENV,
        "registry/generated-artifact-control-plane.json",
    ));
    let artifacts = manifest
        .get("artifacts")
        .and_then(Value::as_array)
        .expect("live generated-artifact manifest must contain artifacts");
    let matching = artifacts
        .iter()
        .filter(|artifact| {
            artifact.get("artifact_id").and_then(Value::as_str)
                == Some("history-only-retirement-facts")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        matching.len(),
        1,
        "history-only retirement facts must be registered exactly once"
    );
    let row = matching[0];
    assert_eq!(
        row.get("path").and_then(Value::as_str),
        Some("ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json")
    );
    assert_eq!(
        row.get("artifact_class").and_then(Value::as_str),
        Some("scm-facts-boundary-snapshot")
    );
    assert_eq!(
        row.get("materialization_mode").and_then(Value::as_str),
        Some("not-tracked-in-git")
    );
    assert_eq!(
        row.get("merge_policy").and_then(Value::as_str),
        Some("never-manual-merge-regenerate-from-source-tree")
    );
    assert_eq!(
        row.get("source_inputs"),
        Some(&json!([
            "registry/history-only-retirement-control-plane.json",
            "specs/history-only-retirement-control-plane.schema.json",
            "specs/history-only-retirement-facts.schema.json",
            "full-depth SCM checkout"
        ]))
    );
    assert_eq!(
        row.pointer("/generator/runner").and_then(Value::as_str),
        Some("buck2")
    );
    assert_eq!(
        row.pointer("/generator/generator_target")
            .and_then(Value::as_str),
        Some("//ci/facade/scm-facts-snapshot:ci-scm-facts-snapshot")
    );
    assert_eq!(
        row.pointer("/generator/operation_id")
            .and_then(Value::as_str),
        Some("emit-history-only-retirement-facts")
    );
    assert_eq!(
        row.pointer("/generator/output_mode")
            .and_then(Value::as_str),
        Some("declared-artifact-path-write")
    );
}

#[test]
fn live_generated_artifacts_are_declared_in_the_control_plane() {
    let manifest = read_json(input_path(
        MANIFEST_ENV,
        "registry/generated-artifact-control-plane.json",
    ));
    let scm_facts = read_json(input_path(
        SCM_FACTS_ENV,
        "ci/facade/artifact-inventory-registry/scm-facts.generated.json",
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
fn live_firewall_frozen_reference_is_regenerate_from_source_adr_0616() {
    // ADR-0616 (supersedes ADR-0596): the firewall frozen reference is REGENERATED from the
    // merge-base source, so the live ratchet policy declares
    // `frozen_reference.source: regenerate-from-merge-base-source`. That EXCLUDES it from the
    // committed-git-blob frozen-reference set (there is no committed blob to empty, so #828 stays
    // impossible), and the control plane may declare it de-commit class WITHOUT firing the
    // must-stay-committed guard.
    let manifest = read_json(input_path(
        MANIFEST_ENV,
        "registry/generated-artifact-control-plane.json",
    ));
    let scm_facts = read_json(input_path(
        SCM_FACTS_ENV,
        "ci/facade/artifact-inventory-registry/scm-facts.generated.json",
    ));

    // The live ratchet policy MUST declare regenerate-from-merge-base-source (the migration is real,
    // not a parse failure that silently empties the set).
    let ratchet_policy = read_json(input_path(RATCHET_POLICY_ENV, RATCHET_POLICY_DEFAULT_PATH));
    assert_eq!(
        ratchet_policy["frozen_reference"]["source"].as_str(),
        Some(FROZEN_REFERENCE_SOURCE_REGENERATE),
        "ADR-0616: the live ratchet policy must declare regenerate-from-merge-base-source"
    );

    // The committed-git-blob frozen-reference set is therefore EMPTY (the only frozen reference has
    // migrated to regenerate-from-source).
    let committed_blob_frozen_refs = live_frozen_reference_paths();
    assert!(
        committed_blob_frozen_refs.is_empty(),
        "ADR-0616: a regenerate-from-source frozen reference must be excluded from the \
         committed-git-blob set; got {committed_blob_frozen_refs:?}"
    );

    // The de-committed frozen reference must NOT fire the must-stay-committed guard.
    let findings =
        evaluate_keyed_with_frozen_references(&manifest, &scm_facts, &committed_blob_frozen_refs);
    assert!(
        !findings
            .iter()
            .any(|finding| finding.code == "frozen_reference_artifact_must_stay_committed"),
        "the regenerate-from-source frozen reference must not fire the must-stay-committed guard; \
         findings: {findings:#?}"
    );
}

#[test]
fn stale_scm_facts_for_deleted_generated_outputs_are_red() {
    let manifest = read_json(input_path(
        MANIFEST_ENV,
        "registry/generated-artifact-control-plane.json",
    ));
    let stale_paths = [
        "legacy/generated/hr-api.d.ts",
        "legacy/generated/ops-workspace-shell-v1.patched.yaml",
        "legacy/generated/ops-workspace-shell.d.ts",
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
