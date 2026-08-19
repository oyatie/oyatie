//! Coverage, parser, and fail-closed tests.

use super::test_fixtures::*;
use super::test_receipts::*;
use super::*;
use serde_json::json;

#[test]
fn closure_facts_expose_raw_byte_equivalent_candidate_copy() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    let control = control_plane();
    for (index, entry) in control.entries.iter().enumerate() {
        let preparation_oid = oid(160 + index as u8);
        add_receipt(
            &mut source,
            PROTECTED,
            &entry.preparation_receipt_path,
            preparation_oid.clone(),
            &receipt_value(entry, false, None),
        );
        add_receipt(
            &mut source,
            CANDIDATE,
            &entry.closure_receipt_path,
            oid(170 + index as u8),
            &receipt_value(entry, true, Some(&preparation_oid)),
        );
    }
    let roadmap = control.entries[0].selectors[0].expected_inputs[0].clone();
    let copied_roadmap_oid = oid(179);
    source.blobs.insert(
        copied_roadmap_oid.clone(),
        source.blobs[&roadmap.predecessor_blob_oid].clone(),
    );
    for commit in [PROTECTED, CANDIDATE] {
        source.trees.get_mut(commit).unwrap().extend([
            TreeEntry {
                mode: "100755".to_owned(),
                kind: "blob".to_owned(),
                oid: copied_roadmap_oid.clone(),
                path: "copied/roadmap-body-a".to_owned(),
            },
            TreeEntry {
                mode: "100755".to_owned(),
                kind: "blob".to_owned(),
                oid: copied_roadmap_oid.clone(),
                path: "copied/roadmap-body-b".to_owned(),
            },
        ]);
    }
    let owners = control.entries[1].selectors[0].expected_inputs[0].clone();
    let copied_owners_oid = oid(182);
    source.blobs.insert(
        copied_owners_oid.clone(),
        source.blobs[&owners.predecessor_blob_oid].clone(),
    );
    source.trees.get_mut(CANDIDATE).unwrap().push(TreeEntry {
        mode: "100755".to_owned(),
        kind: "blob".to_owned(),
        oid: copied_owners_oid,
        path: "copied/owners-body".to_owned(),
    });

    let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
    let roadmap_fact = facts["scm_facts"]["retirement_receipt_object_facts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|fact| fact["scope_ref"] == json!("artifact:masterplan"))
        .and_then(|fact| fact["retired_inputs"].as_array())
        .and_then(|inputs| inputs.first())
        .unwrap();
    assert_eq!(
        roadmap_fact["candidate_equivalent_paths"],
        json!(["copied/roadmap-body-a", "copied/roadmap-body-b"])
    );
    assert_eq!(roadmap_fact["candidate_new_equivalent_paths"], json!([]));
    let owners_fact = facts["scm_facts"]["retirement_receipt_object_facts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|fact| fact["scope_ref"] == json!("ADR-0363"))
        .and_then(|fact| fact["retired_inputs"].as_array())
        .and_then(|inputs| {
            inputs
                .iter()
                .find(|input| input["path"] == json!(owners.path))
        })
        .unwrap();
    assert_eq!(
        owners_fact["candidate_equivalent_paths"],
        json!(["copied/owners-body"])
    );
    assert_eq!(
        owners_fact["candidate_new_equivalent_paths"],
        json!(["copied/owners-body"])
    );
}

#[test]
fn equivalence_scan_visits_each_filler_oid_once_for_full_input_population() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    let control = control_plane();
    for (index, entry) in control.entries.iter().enumerate() {
        let preparation_oid = oid(160 + index as u8);
        add_receipt(
            &mut source,
            PROTECTED,
            &entry.preparation_receipt_path,
            preparation_oid.clone(),
            &receipt_value(entry, false, None),
        );
        add_receipt(
            &mut source,
            CANDIDATE,
            &entry.closure_receipt_path,
            oid(170 + index as u8),
            &receipt_value(entry, true, Some(&preparation_oid)),
        );
    }
    let protected_filler_oid = oid(180);
    let shared_filler_oid = oid(181);
    source
        .blobs
        .insert(protected_filler_oid.clone(), b"protected filler".to_vec());
    source
        .blobs
        .insert(shared_filler_oid.clone(), b"shared filler".to_vec());
    source.trees.get_mut(PROTECTED).unwrap().extend([
        TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: protected_filler_oid.clone(),
            path: "filler/protected".to_owned(),
        },
        TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: shared_filler_oid.clone(),
            path: "filler/shared-protected".to_owned(),
        },
    ]);
    source.trees.get_mut(CANDIDATE).unwrap().push(TreeEntry {
        mode: "100644".to_owned(),
        kind: "blob".to_owned(),
        oid: shared_filler_oid.clone(),
        path: "filler/shared-candidate".to_owned(),
    });

    materialize_history_only_retirement_facts(&source, &context()).unwrap();

    let reads = source.read_counts.borrow();
    assert_eq!(reads.get(&protected_filler_oid), Some(&1));
    assert_eq!(reads.get(&shared_filler_oid), Some(&1));
}

#[test]
fn unexpected_receipt_root_path_is_explicit_fact_not_silently_ignored() {
    let mut source = fixture();
    let unexpected_oid = oid(180);
    source
        .blobs
        .insert(unexpected_oid.clone(), b"unexpected".to_vec());
    source.trees.get_mut(CANDIDATE).unwrap().push(TreeEntry {
        mode: "100644".to_owned(),
        kind: "blob".to_owned(),
        oid: unexpected_oid,
        path: "evidence/history-only-retirement/unexpected.json".to_owned(),
    });

    let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
    assert_eq!(
        facts["scm_facts"]["retirement_control_plane_context"]["unexpected_candidate_receipt_paths"],
        json!(["evidence/history-only-retirement/unexpected.json"])
    );
}

#[test]
fn independently_read_protected_control_plane_divergence_fails_closed() {
    let mut source = fixture();
    let protected_oid = oid(181);
    source.blobs.insert(protected_oid.clone(), b"{}".to_vec());
    source.trees.get_mut(PROTECTED).unwrap().push(TreeEntry {
        mode: "100644".to_owned(),
        kind: "blob".to_owned(),
        oid: protected_oid,
        path: CONTROL_PLANE_PATH.to_owned(),
    });

    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(error.contains("not immutable-identical"));
}

#[test]
fn rejects_non_first_parent_protected_base() {
    let mut source = fixture();
    source.first_parent_override = Some(PREDECESSOR.to_owned());
    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(error.contains("not candidate first parent"));
}

#[test]
fn rejects_mutated_predecessor_raw_bytes() {
    let mut source = fixture();
    source.blobs.insert(oid(10), b"changed".to_vec());
    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(error.contains("raw-byte binding mismatch"));
}

#[test]
fn rejects_candidate_declared_non_regular_predecessor_mode() {
    let mut source = fixture();
    let control_oid = source
        .trees
        .get(CANDIDATE)
        .unwrap()
        .iter()
        .find(|entry| entry.path == CONTROL_PLANE_PATH)
        .unwrap()
        .oid
        .clone();
    let mut control = control_plane();
    control.entries[0].selectors[0].expected_inputs[0].mode = "100755".to_owned();
    source.blobs.insert(
        control_oid,
        to_canonical_json(&serde_json::to_value(control).unwrap())
            .unwrap()
            .into_bytes(),
    );
    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(error.contains("must declare mode 100644"));
}

#[test]
fn selector_rejects_unexpected_matching_candidate_path() {
    let mut source = fixture();
    source.trees.get_mut(CANDIDATE).unwrap().push(TreeEntry {
        mode: "100644".to_owned(),
        kind: "blob".to_owned(),
        oid: oid(201),
        path: "docs/ideas/archive/unlisted.md".to_owned(),
    });
    source.blobs.insert(oid(201), b"unlisted".to_vec());

    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(
        error.contains("selector coverage"),
        "unexpected error: {error}"
    );
}

#[test]
fn selector_rejects_unexpected_matching_predecessor_or_protected_path() {
    for tree in [PREDECESSOR, PROTECTED] {
        let mut source = fixture();
        source.trees.get_mut(tree).unwrap().push(TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: oid(if tree == PREDECESSOR { 202 } else { 203 }),
            path: "docs/ideas/archive/unlisted.md".to_owned(),
        });
        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(
            error.contains("selector coverage"),
            "unexpected error: {error}"
        );
    }
}
