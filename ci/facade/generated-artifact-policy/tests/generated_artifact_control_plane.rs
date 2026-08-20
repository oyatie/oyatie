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

/// The exact command that materializes the `*.generated.json` faces locally — the same binary the
/// CI "Materialize cloud-ci generated faces" step runs, minus `--github-event` (which only reads
/// `GITHUB_EVENT_PATH`).
const MATERIALIZE_CMD: &str = "buck2 run \
     //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin \
     -- --repo-root .";

fn read_json(path: PathBuf) -> Value {
    // A missing GENERATED face is the one failure mode an author hits every time and can act on,
    // so it gets the instruction instead of a bare `os error 2`. These faces are the ADR-0604
    // de-commit class: not tracked in git, materialized by CI, therefore absent in ANY clean
    // worktree — which meant this gate produced no local signal at all and could not be used to
    // check a change before pushing. Tracked inputs (manifest, schema, ratchet policy) keep the
    // plain read error; for those, "missing" really is just missing.
    let bytes = fs::read(&path).unwrap_or_else(|e| {
        let generated = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".generated.json"));
        assert!(
            !generated,
            "read {}: {e}\n\nThis is a GENERATED face (ADR-0604 de-commit class): it is not \
             tracked in git and is absent in a clean worktree. Materialize it, then re-run this \
             gate:\n\n    {MATERIALIZE_CMD}\n\nOr point the gate at an existing face with \
             {SCM_FACTS_ENV}=<path>.",
            path.display()
        );
        panic!("read {}: {e}", path.display())
    });
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
fn census_epoch_receipt_declares_every_active_historical_and_event_identity_input() {
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
                == Some("cloud-ci-adr-census-epoch-receipt")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matching.len(),
        1,
        "ADR census epoch receipt must be registered exactly once"
    );

    let source_inputs = matching[0]
        .get("source_inputs")
        .and_then(Value::as_array)
        .expect("ADR census epoch receipt must declare source inputs")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("ADR census epoch source input must be a string")
        })
        .collect::<BTreeSet<_>>();
    for required in [
        "registry/adr-census-epoch/control-plane.json",
        "specs/adr-census-epoch-control-plane.schema.json",
        "specs/adr-census-epoch-receipt.schema.json",
        "historical implementation commit afff4dade737b1833153c4f45d8defdfa2b328a8",
        "corpus commit 1fa09da22be819b062881eb59252f4dd4c6b550a",
        "repository tree d7b15539396db21b219d68779362850cce9afa8f",
        "docs tree fbf3f8d4b9ecf30b2272f37871e8152a616eed5a",
        "decisions tree 7c7c371697d2a7009e3d43b16235518d00ac33ea",
        "parser commit a2b326eebd418ae970847b5e1bca3782c61c52ab",
        "parser tree 0cdece525bc54f83ec51d3ba67a4308d0ce43812",
        "parser blob ab3884dbf4a657869fd87920b016cc4734a1c27f",
        ".github/workflows/oya-ci-required.yml",
    ] {
        assert!(
            source_inputs.contains(required),
            "ADR census epoch receipt must declare active historical or event transport input {required}"
        );
    }

    let input_contract = matching[0]["generator"]["input_contract"]
        .as_array()
        .expect("ADR census epoch generator must declare an input contract")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("ADR census epoch generator input must be a string")
        })
        .collect::<BTreeSet<_>>();
    assert!(
        input_contract.contains("scm-event-identity"),
        "ADR census epoch generator must declare controller-supplied SCM event identity"
    );

    let final_tree_validation = matching[0]["final_tree_validation"]
        .as_str()
        .expect("ADR census epoch receipt must declare final-tree validation");
    for required in [
        "selects the immutable revision",
        "excluded from the squash-stable P2 receipt core",
    ] {
        assert!(
            final_tree_validation.contains(required),
            "ADR census epoch validation contract must state that event identity {required}"
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
            "registry/history-only-retirement/control-plane.json",
            "specs/history-only-retirement-control-plane.schema.json",
            "specs/history-only-retirement-facts.schema.json",
            ".github/workflows/oya-ci-required.yml",
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
    assert_eq!(
        row.pointer("/generator/input_contract"),
        Some(&json!([
            "repo-root",
            "full-depth-scm",
            "scm-event-identity",
            "declared-source-inputs"
        ]))
    );
}

#[test]
fn active_artifact_contract_graph_is_the_exact_controller_owned_untracked_face() {
    let manifest = read_json(input_path(
        MANIFEST_ENV,
        "registry/generated-artifact-control-plane.json",
    ));
    let artifacts = manifest["artifacts"]
        .as_array()
        .expect("live generated-artifact manifest must contain artifacts");
    let matching = artifacts
        .iter()
        .filter(|artifact| {
            artifact["artifact_id"].as_str() == Some("active-artifact-contract-edges")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        matching.len(),
        1,
        "graph projection must be registered exactly once"
    );
    let row = matching[0];
    assert_eq!(
        row["path"].as_str(),
        Some("registry/graph/active-artifact-contract-edges.json")
    );
    assert_eq!(
        row["materialization_mode"].as_str(),
        Some("not-tracked-in-git")
    );
    assert_eq!(
        row["merge_policy"].as_str(),
        Some("never-manual-merge-regenerate-from-source-tree")
    );
    assert_eq!(
        row.pointer("/generator/runner").and_then(Value::as_str),
        Some("buck2")
    );
    assert_eq!(
        row.pointer("/generator/generator_target")
            .and_then(Value::as_str),
        Some("//marketplace/facade/dev-cli:oya")
    );
    assert_eq!(
        row.pointer("/generator/operation_id")
            .and_then(Value::as_str),
        Some("emit-active-artifact-contract-graph-edges")
    );
    assert_eq!(
        row.pointer("/generator/output_mode")
            .and_then(Value::as_str),
        Some("declared-artifact-path-write")
    );
    assert!(
        manifest["generated_path_rules"]
            .as_array()
            .expect("path rules")
            .iter()
            .any(|rule| {
                rule["rule_kind"].as_str() == Some("path_suffix")
                    && rule["pattern"].as_str()
                        == Some("registry/graph/active-artifact-contract-edges.json")
            }),
        "the graph projection requires an exact generated-path rule"
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
