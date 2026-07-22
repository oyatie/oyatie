#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use ci_action_item_accounting::fixuptask_v2::{
    CANDIDATE_REGISTRY_PATH, PROTECTED_FACTS_PATH, evaluate_materialized_gate,
};
use serde_json::json;

fn fixture_row() -> &'static str {
    "{\"id\":\"F-DURABLE\",\"title\":\"durable fixture\",\"priority\":\"high\",\"status\":\"open\",\"source_session\":\"session\",\"source_change_id\":\"change\",\"named_in\":\"ADR-0621\",\"created_at\":\"2026-07-22T00:00:00Z\",\"accountable_owner\":\"owner\",\"accountable_role\":\"role\",\"acceptance_criteria\":\"criterion\",\"verification_path\":\"buck2 test\",\"blocker_for\":\"none\"}\n"
}

#[test]
fn durable_admission_is_green_without_any_predecessor_body() {
    let root = std::env::temp_dir().join(format!("fixuptask-v2-durable-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::create_dir_all(root.join("ci/facade/scm-facts-snapshot")).expect("facts parent");
    let candidate = fixture_row();
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), candidate).expect("candidate registry");
    let facts = json!({ "fixuptask_v2_durable": {
        "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "merge_base_rows": [],
        "candidate_registry_digest": ci_action_item_accounting::fixuptask_v2_digest(candidate.as_bytes()),
        "evaluation_time": "2026-07-22T00:00:00Z"
    }});
    std::fs::write(
        root.join(PROTECTED_FACTS_PATH),
        serde_json::to_vec(&facts).expect("serialize protected facts"),
    )
    .expect("protected facts");

    assert!(
        evaluate_materialized_gate(&PathBuf::from(&root))
            .expect("durable admission reads only registry and protected facts")
            .is_empty()
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn durable_admission_rejects_a_facts_digest_for_other_registry_bytes() {
    let candidate = fixture_row();
    let facts = json!({ "fixuptask_v2_durable": {
        "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "merge_base_rows": [],
        "candidate_registry_digest": ci_action_item_accounting::fixuptask_v2_digest(b"other bytes"),
        "evaluation_time": "2026-07-22T00:00:00Z"
    }});
    let findings = ci_action_item_accounting::fixuptask_v2::evaluate_admission(
        &facts,
        &json!({ "rows": [serde_json::from_str::<serde_json::Value>(fixture_row()).expect("row")] }),
        candidate.as_bytes(),
    );
    assert!(
        findings
            .iter()
            .any(|finding| { finding.code == "fixuptask_v2_candidate_registry_digest_mismatch" })
    );
}
