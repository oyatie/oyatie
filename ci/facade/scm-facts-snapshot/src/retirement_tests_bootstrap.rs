//! Bootstrap and atomic-population tests.

use super::test_fixtures::*;
use super::test_receipts::*;
use super::*;
use serde_json::json;

#[test]
fn bootstrap_is_candidate_bound_three_row_empty_and_deterministic() {
    let source = fixture();
    let first = materialize_history_only_retirement_facts(&source, &context()).unwrap();
    let second = materialize_history_only_retirement_facts(&source, &context()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first["receipts"], json!([]));
    assert_eq!(
        first["scm_facts"]["retirement_receipt_object_facts"],
        json!([])
    );
    assert_eq!(
        first["scm_facts"]["retirement_receipt_coverage"]["scopes"],
        json!([])
    );
    assert_eq!(
        first["scm_facts"]["retirement_control_plane_context"]["bootstrap"],
        json!(true)
    );
    assert!(
        first["scm_facts"]["retirement_control_plane_context"]["protected_control_plane_blob_oid"]
            .is_null()
    );
    assert!(
        first["scm_facts"]["retirement_control_plane_context"]["candidate_control_plane_blob_oid"]
            .as_str()
            .is_some()
    );
    assert_eq!(
        first["scm_facts"]["retirement_control_plane_context"]["control_plane_entries"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let rendered = to_canonical_json(&first).unwrap();
    assert!(!rendered.contains("verdict"));
    assert!(!rendered.contains("PASS"));
    assert!(!rendered.contains("roadmap_author"));
}

#[test]
fn bootstrap_rejects_nonempty_receipt_population() {
    let mut source = fixture();
    let receipt_oid = oid(99);
    source
        .blobs
        .insert(receipt_oid.clone(), br#"{"artifact_id":"x"}"#.to_vec());
    source.trees.get_mut(CANDIDATE).unwrap().push(TreeEntry {
        mode: "100644".to_owned(),
        kind: "blob".to_owned(),
        oid: receipt_oid,
        path: MASTERPLAN_PREPARATION_PATH.to_owned(),
    });
    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(error.contains("not atomic") || error.contains("may not add receipts"));
}

#[test]
fn prepared_new_is_atomic_three_receipt_facts_without_projection_or_copies() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    add_current_bodies(&mut source, CANDIDATE);
    for (index, entry) in control_plane().entries.iter().enumerate() {
        add_receipt(
            &mut source,
            CANDIDATE,
            &entry.preparation_receipt_path,
            oid(100 + index as u8),
            &receipt_value(entry, false, None),
        );
    }

    let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
    assert_eq!(facts["receipts"].as_array().unwrap().len(), 3);
    assert!(
        facts["scm_facts"]["retirement_control_plane_context"]
            .get("lifecycle_state")
            .is_none()
    );
    assert_eq!(
        facts["scm_facts"]["retirement_receipt_coverage"]["scopes"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let adr_0388_scope = facts["scm_facts"]["retirement_receipt_coverage"]["scopes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|scope| scope["scope_ref"] == json!("ADR-0388"))
        .unwrap();
    let adr_0388_selectors = adr_0388_scope["selectors"].as_array().unwrap();
    assert_eq!(adr_0388_selectors.len(), 3);
    assert!(
        adr_0388_selectors
            .iter()
            .all(|selector| selector["selector_type"] == json!("exact"))
    );
    assert_eq!(
        adr_0388_selectors
            .iter()
            .map(|selector| selector["selector"].as_str().unwrap())
            .collect::<BTreeSet<_>>(),
        control_plane()
            .entries
            .iter()
            .find(|entry| entry.scope_ref == "ADR-0388")
            .unwrap()
            .selectors
            .iter()
            .flat_map(|selector| selector.expected_inputs.iter())
            .map(|input| input.path.as_str())
            .collect::<BTreeSet<_>>()
    );
    for fact in facts["scm_facts"]["retirement_receipt_object_facts"]
        .as_array()
        .unwrap()
    {
        assert_eq!(fact["receipt_state"], json!("prepared-new"));
        assert_eq!(
            fact["predecessor_context"]["source"],
            json!("control-plane-predecessor")
        );
        for input in fact["retired_inputs"].as_array().unwrap() {
            assert_eq!(input["mode"], json!("100644"));
            assert_eq!(input["candidate_equivalent_paths"], json!([]));
            assert_eq!(input["candidate_new_equivalent_paths"], json!([]));
        }
    }
    let protected_context = facts["scm_facts"]["protected_scm_context"]
        .as_object()
        .unwrap();
    assert!(!protected_context.contains_key("prepared_receipt_paths"));
    assert!(!protected_context.contains_key("control_plane_entries"));
    let rendered = to_canonical_json(&facts).unwrap();
    assert!(!rendered.contains("closure_projection"));
    assert!(!rendered.contains("verdict"));
}

#[test]
fn closed_carried_and_prepared_new_scopes_are_an_admitted_atomic_population() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    add_current_bodies(&mut source, CANDIDATE);
    let control = control_plane();
    let closed_paths = control.entries[0]
        .selectors
        .iter()
        .flat_map(|selector| selector.expected_inputs.iter())
        .map(|input| input.path.as_str())
        .collect::<BTreeSet<_>>();
    for commit in [PROTECTED, CANDIDATE] {
        source
            .trees
            .get_mut(commit)
            .unwrap()
            .retain(|tree_entry| !closed_paths.contains(tree_entry.path.as_str()));
    }
    add_closed_carried_receipt(
        &mut source,
        &control.entries[0],
        oid(190),
        oid(191),
        oid(192),
    );
    for (index, entry) in control.entries.iter().skip(1).enumerate() {
        add_receipt(
            &mut source,
            CANDIDATE,
            &entry.preparation_receipt_path,
            oid(193 + index as u8),
            &receipt_value(entry, false, None),
        );
    }

    let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
    let states = facts["receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|receipt| receipt["receipt_state"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(states, BTreeSet::from(["closed-carried", "prepared-new"]));
    assert!(
        facts["scm_facts"]["retirement_control_plane_context"]
            .get("lifecycle_state")
            .is_none()
    );
}

#[test]
fn partial_active_receipt_population_fails_closed() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    add_current_bodies(&mut source, CANDIDATE);
    let entry = &control_plane().entries[0];
    add_receipt(
        &mut source,
        CANDIDATE,
        &entry.preparation_receipt_path,
        oid(196),
        &receipt_value(entry, false, None),
    );

    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(error.contains("not atomic"), "unexpected error: {error}");
}
