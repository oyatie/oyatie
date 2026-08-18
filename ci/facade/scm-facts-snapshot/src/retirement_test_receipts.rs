//! Receipt-construction helpers for retirement-facts tests.

use super::test_fixtures::*;
use super::*;
use serde_json::{Value, json};

pub(crate) fn control_oid(source: &FakeSource) -> String {
    source
        .trees
        .get(CANDIDATE)
        .unwrap()
        .iter()
        .find(|entry| entry.path == CONTROL_PLANE_PATH)
        .unwrap()
        .oid
        .clone()
}

pub(crate) fn add_protected_control_plane(source: &mut FakeSource) {
    let control_oid = control_oid(source);
    source.trees.get_mut(PROTECTED).unwrap().push(TreeEntry {
        mode: "100644".to_owned(),
        kind: "blob".to_owned(),
        oid: control_oid,
        path: CONTROL_PLANE_PATH.to_owned(),
    });
}

pub(crate) fn add_current_bodies(source: &mut FakeSource, commit: &str) {
    let bodies = source.trees.get(PREDECESSOR).unwrap().clone();
    source.trees.get_mut(commit).unwrap().extend(bodies);
}

pub(crate) fn receipt_value(
    entry: &ControlPlaneEntry,
    closure: bool,
    preparation_blob: Option<&str>,
) -> Value {
    let authority = match entry.scope_ref.as_str() {
        "artifact:masterplan" => "/specs/masterplan.json",
        "ADR-0363" => "ADR-0363",
        "ADR-0388" => "ADR-0388",
        scope => panic!("unexpected retirement scope in test fixture: {scope}"),
    };
    let disposition = if closure {
        "retired-git-history-only"
    } else {
        "prepared-for-history-only-retirement"
    };
    let retired_inputs = entry
        .selectors
        .iter()
        .flat_map(|selector| selector.expected_inputs.iter())
        .map(|input| {
            json!({
                "path": input.path,
                "predecessor_blob_oid": input.predecessor_blob_oid,
                "sha256": input.sha256,
                "byte_count": input.byte_count,
                "successor_refs": [authority],
                "disposition": disposition,
            })
        })
        .collect::<Vec<_>>();
    let expected_absent_paths = retired_inputs
        .iter()
        .filter_map(|input| input.get("path").and_then(Value::as_str))
        .collect::<Vec<_>>();
    let verification_contract = if closure {
        json!({
            "expected_absent_paths": expected_absent_paths,
            "expected_tracked_readable_archive_directory_count": 0,
            "required_gates": [
                "cloud-ci-cross-artifact-agreement/history-only-retirement-receipt",
                "cloud-ci-cross-artifact-agreement/idea-archive-transition",
            ],
        })
    } else {
        json!({})
    };
    let repository_effect = if closure { "history-only" } else { "prepared" };
    let mut value = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "artifact_id": if closure { &entry.closure_artifact_id } else { &entry.preparation_artifact_id },
        "artifact_type": if closure { "history-only-retirement-closure-receipt" } else { "migration-closure-receipt" },
        "status": if closure { "history-only-retired-nonauthoritative" } else { "prepared-for-history-only-retirement" },
        "recorded_at": "2026-07-24",
        "scope_ref": entry.scope_ref,
        "authority": {
            "decisions": [authority],
            "planning_state": "HOLD(Planning)",
            "dispatch_authorized": false,
            "completion_claims_promoted": 0,
        },
        "baseline": {
            "commit_oid": PREDECESSOR,
            "tree_oid": PREDECESSOR_TREE,
        },
        "retired_inputs": retired_inputs,
        "provenance": {
            "content_store": "authorized Git object history only",
            "readable_tracked_copy_retained": false,
            "readable_archive_directory_retained": false,
            "tombstone_content_retained": false,
            "receipt_reproduces_retired_content": false,
        },
        "verification_contract": verification_contract,
        "effects": {
            "repository_effect": repository_effect,
            "runtime_effect": "none",
            "roadmap_effect": "none",
            "planning_hold_effect": "HOLD(Planning)",
        },
    });
    if let Some(blob) = preparation_blob {
        value["protected_preparation"] = json!({
            "receipt_path": entry.preparation_receipt_path,
            "receipt_blob_oid": blob,
        });
    }
    value
}

pub(crate) fn add_receipt(
    source: &mut FakeSource,
    commit: &str,
    path: &str,
    blob_oid: String,
    receipt: &Value,
) {
    source.blobs.insert(
        blob_oid.clone(),
        to_canonical_json(receipt).unwrap().into_bytes(),
    );
    source.trees.get_mut(commit).unwrap().push(TreeEntry {
        mode: "100644".to_owned(),
        kind: "blob".to_owned(),
        oid: blob_oid,
        path: path.to_owned(),
    });
}

pub(crate) fn add_closed_carried_receipt(
    source: &mut FakeSource,
    entry: &ControlPlaneEntry,
    preparation_oid: String,
    closure_oid: String,
    history_commit: String,
) {
    let preparation = receipt_value(entry, false, None);
    source.blobs.insert(
        preparation_oid.clone(),
        to_canonical_json(&preparation).unwrap().into_bytes(),
    );
    source.history.insert(
        entry.preparation_receipt_path.clone(),
        vec![history_commit.clone()],
    );
    source.trees.insert(
        history_commit,
        vec![TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: preparation_oid.clone(),
            path: entry.preparation_receipt_path.clone(),
        }],
    );
    let closure = receipt_value(entry, true, Some(&preparation_oid));
    add_receipt(
        source,
        PROTECTED,
        &entry.closure_receipt_path,
        closure_oid.clone(),
        &closure,
    );
    add_receipt(
        source,
        CANDIDATE,
        &entry.closure_receipt_path,
        closure_oid,
        &closure,
    );
}
