//! Closure-link and carried-receipt tests.

use super::test_fixtures::*;
use super::test_receipts::*;
use super::*;
use serde_json::json;

#[test]
fn closure_new_rejects_candidate_link_to_wrong_protected_preparation_blob() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    let control = control_plane();
    for (index, entry) in control.entries.iter().enumerate() {
        let preparation_oid = oid(113 + index as u8);
        add_receipt(
            &mut source,
            PROTECTED,
            &entry.preparation_receipt_path,
            preparation_oid.clone(),
            &receipt_value(entry, false, None),
        );
        let linked_oid = if index == 0 {
            oid(119)
        } else {
            preparation_oid
        };
        add_receipt(
            &mut source,
            CANDIDATE,
            &entry.closure_receipt_path,
            oid(120 + index as u8),
            &receipt_value(entry, true, Some(&linked_oid)),
        );
    }

    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(error.contains("links unexpected protected preparation blob"));
}

#[test]
fn closure_new_rejects_protected_preparation_with_different_baseline() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    let control = control_plane();
    for (index, entry) in control.entries.iter().enumerate() {
        let preparation_oid = oid(123 + index as u8);
        let mut preparation = receipt_value(entry, false, None);
        if index == 0 {
            preparation["baseline"] = json!({
                "commit_oid": PROTECTED,
                "tree_oid": PROTECTED_TREE,
            });
        }
        add_receipt(
            &mut source,
            PROTECTED,
            &entry.preparation_receipt_path,
            preparation_oid.clone(),
            &preparation,
        );
        add_receipt(
            &mut source,
            CANDIDATE,
            &entry.closure_receipt_path,
            oid(126 + index as u8),
            &receipt_value(entry, true, Some(&preparation_oid)),
        );
    }

    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(error.contains("immutable control-plane predecessor"));
}

#[test]
fn closed_carried_uses_reachable_linked_preparation_history() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    let control = control_plane();
    for (index, entry) in control.entries.iter().enumerate() {
        let preparation_oid = oid(130 + index as u8);
        let closure_oid = oid(140 + index as u8);
        let preparation = receipt_value(entry, false, None);
        source.blobs.insert(
            preparation_oid.clone(),
            to_canonical_json(&preparation).unwrap().into_bytes(),
        );
        let history_commit = oid(150 + index as u8);
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
            &mut source,
            PROTECTED,
            &entry.closure_receipt_path,
            closure_oid.clone(),
            &closure,
        );
        add_receipt(
            &mut source,
            CANDIDATE,
            &entry.closure_receipt_path,
            closure_oid,
            &closure,
        );
    }

    let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
    assert!(
        facts["scm_facts"]["retirement_control_plane_context"]
            .get("lifecycle_state")
            .is_none()
    );
    for fact in facts["scm_facts"]["retirement_receipt_object_facts"]
        .as_array()
        .unwrap()
    {
        assert_eq!(fact["receipt_state"], json!("closed-carried"));
        assert_eq!(
            fact["predecessor_context"]["source"],
            json!("linked-preparation-history")
        );
        for input in fact["retired_inputs"].as_array().unwrap() {
            assert_eq!(input["protected_path_exists"], json!(false));
            assert_eq!(input["candidate_path_exists"], json!(false));
        }
    }
}

#[test]
fn closed_carried_rejects_linked_preparation_with_different_baseline() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    let control = control_plane();
    for (index, entry) in control.entries.iter().enumerate() {
        let preparation_oid = oid(153 + index as u8);
        let closure_oid = oid(156 + index as u8);
        let mut preparation = receipt_value(entry, false, None);
        if index == 0 {
            preparation["baseline"] = json!({
                "commit_oid": PROTECTED,
                "tree_oid": PROTECTED_TREE,
            });
        }
        source.blobs.insert(
            preparation_oid.clone(),
            to_canonical_json(&preparation).unwrap().into_bytes(),
        );
        let history_commit = oid(159 + index as u8);
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
            &mut source,
            PROTECTED,
            &entry.closure_receipt_path,
            closure_oid.clone(),
            &closure,
        );
        add_receipt(
            &mut source,
            CANDIDATE,
            &entry.closure_receipt_path,
            closure_oid,
            &closure,
        );
    }

    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(error.contains("immutable control-plane predecessor"));
}

