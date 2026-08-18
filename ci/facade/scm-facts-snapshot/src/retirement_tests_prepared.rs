//! Prepared-new and closure-binding tests.

use super::test_fixtures::*;
use super::test_receipts::*;
use super::*;
use serde_json::json;

#[test]
fn unsupported_heterogeneous_active_lifecycles_fail_closed() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    let control = control_plane();
    add_closed_carried_receipt(
        &mut source,
        &control.entries[0],
        oid(197),
        oid(198),
        oid(199),
    );
    let preparation_oid = oid(200);
    add_receipt(
        &mut source,
        PROTECTED,
        &control.entries[1].preparation_receipt_path,
        preparation_oid.clone(),
        &receipt_value(&control.entries[1], false, None),
    );
    add_receipt(
        &mut source,
        CANDIDATE,
        &control.entries[1].closure_receipt_path,
        oid(201),
        &receipt_value(&control.entries[1], true, Some(&preparation_oid)),
    );
    add_closed_carried_receipt(
        &mut source,
        &control.entries[2],
        oid(202),
        oid(203),
        oid(204),
    );

    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(
        error.contains("not an admitted atomic population"),
        "unexpected error: {error}"
    );
}

#[test]
fn prepared_new_never_projects_raw_receipt_bodies_or_authority_lookalikes() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    add_current_bodies(&mut source, CANDIDATE);
    for (index, entry) in control_plane().entries.iter().enumerate() {
        let mut receipt = receipt_value(entry, false, None);
        match index {
            0 => receipt["retired_body"] = json!("TOP-SECRET-RETIRED-BODY"),
            1 => receipt["PASS"] = json!(true),
            2 => {
                receipt["authority"]["roadmap_dispatch_authorized"] = json!(true);
                receipt["qualified_human_authority"] = json!({"verdict": "PASS"});
            }
            _ => unreachable!(),
        }
        add_receipt(
            &mut source,
            CANDIDATE,
            &entry.preparation_receipt_path,
            oid(106 + index as u8),
            &receipt,
        );
    }

    let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
    let expected_keys = BTreeSet::from([
        "artifact_id",
        "baseline_commit_oid",
        "baseline_tree_oid",
        "candidate_receipt_blob_oid",
        "candidate_receipt_sha256",
        "receipt_path",
        "receipt_state",
        "scope_ref",
    ]);
    for receipt in facts["receipts"].as_array().unwrap() {
        let actual_keys: BTreeSet<&str> = receipt
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(actual_keys, expected_keys);
    }
    let rendered = to_canonical_json(&facts).unwrap();
    for forbidden in [
        "TOP-SECRET-RETIRED-BODY",
        "retired_body",
        "PASS",
        "roadmap_dispatch_authorized",
        "qualified_human_authority",
        "verdict",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "controller facts leaked hostile receipt field {forbidden:?}: {rendered}"
        );
    }
}

#[test]
fn emitter_rejects_arbitrary_and_absolute_generated_facts_paths_before_git() {
    for output_path in [
        Path::new("ci/facade/scm-facts-snapshot/not-canonical.generated.json"),
        Path::new(
            "./ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json",
        ),
        Path::new("/tmp/history-only-retirement-facts.generated.json"),
    ] {
        let error = emit_history_only_retirement_facts(Path::new("."), &context(), output_path)
            .unwrap_err();
        assert!(
            error.contains("exact canonical repo-relative generated facts path"),
            "unexpected error for {output_path:?}: {error}"
        );
    }
}

#[test]
fn prepared_new_rejects_receipt_baseline_other_than_control_plane_predecessor() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    add_current_bodies(&mut source, CANDIDATE);
    for (index, entry) in control_plane().entries.iter().enumerate() {
        let mut receipt = receipt_value(entry, false, None);
        if index == 0 {
            receipt["baseline"] = json!({
                "commit_oid": PROTECTED,
                "tree_oid": PROTECTED_TREE,
            });
        }
        add_receipt(
            &mut source,
            CANDIDATE,
            &entry.preparation_receipt_path,
            oid(103 + index as u8),
            &receipt,
        );
    }

    let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
    assert!(error.contains("immutable control-plane predecessor"));
}

#[test]
fn closure_new_binds_each_candidate_closure_to_protected_preparation() {
    let mut source = fixture();
    add_protected_control_plane(&mut source);
    add_current_bodies(&mut source, PROTECTED);
    let control = control_plane();
    for (index, entry) in control.entries.iter().enumerate() {
        let preparation_oid = oid(110 + index as u8);
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
            oid(120 + index as u8),
            &receipt_value(entry, true, Some(&preparation_oid)),
        );
    }

    let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
    assert!(
        facts["scm_facts"]["retirement_control_plane_context"]
            .get("lifecycle_state")
            .is_none()
    );
    assert_eq!(
        facts["scm_facts"]["protected_scm_context"]["protected_preparation_receipts"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    for fact in facts["scm_facts"]["retirement_receipt_object_facts"]
        .as_array()
        .unwrap()
    {
        assert_eq!(fact["receipt_state"], json!("closure-new"));
        assert_eq!(
            fact["predecessor_context"]["source"],
            json!("protected-preparation-receipt")
        );
        assert!(fact["preparation_receipt_path"].as_str().is_some());
        for input in fact["retired_inputs"].as_array().unwrap() {
            assert_eq!(input["protected_path_exists"], json!(true));
            assert_eq!(input["candidate_path_exists"], json!(false));
        }
    }
}

