#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use ci_action_item_accounting::fixuptask_v2::{
    CANDIDATE_REGISTRY_PATH, CollectError, PROTECTED_FACTS_PATH, evaluate_materialized_gate,
};
use serde_json::json;

fn fixture_row() -> &'static str {
    "{\"id\":\"F-DURABLE\",\"title\":\"durable fixture\",\"priority\":\"high\",\"status\":\"open\",\"source_session\":\"session\",\"source_change_id\":\"change\",\"named_in\":\"ADR-0621\",\"created_at\":\"2026-07-22T00:00:00Z\",\"accountable_owner\":\"owner\",\"accountable_role\":\"role\",\"acceptance_criteria\":\"criterion\",\"verification_path\":\"buck2 test\",\"blocker_for\":\"none\"}\n"
}

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

#[test]
fn durable_admission_rejects_duplicate_object_keys_despite_matching_rows_and_digest() {
    let root = std::env::temp_dir().join(format!(
        "fixuptask-v2-duplicate-keys-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::create_dir_all(root.join("ci/facade/scm-facts-snapshot")).expect("facts parent");
    let candidate = fixture_row().replacen(
        "\"title\":\"durable fixture\"",
        "\"title\":\"durable fixture\",\"title\":\"durable fixture\"",
        1,
    );
    let merge_base_row = serde_json::from_str::<serde_json::Value>(fixture_row()).expect("row");
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), &candidate).expect("candidate registry");
    let facts = json!({ "fixuptask_v2_durable": {
        "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "merge_base_rows": [merge_base_row],
        "candidate_registry_digest": ci_action_item_accounting::fixuptask_v2_digest(candidate.as_bytes()),
        "evaluation_time": "2026-07-22T00:00:00Z"
    }});
    std::fs::write(
        root.join(PROTECTED_FACTS_PATH),
        serde_json::to_vec(&facts).expect("facts json"),
    )
    .expect("protected facts");

    let error = evaluate_materialized_gate(&root).expect_err("duplicate object key must reject");
    assert!(matches!(error, CollectError::Parse { line: 1, .. }));
    assert!(error.to_string().contains("duplicate object key: title"));
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn durable_admission_rejects_duplicate_protected_fact_keys() {
    let root = std::env::temp_dir().join(format!(
        "fixuptask-v2-duplicate-protected-keys-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::create_dir_all(root.join("ci/facade/scm-facts-snapshot")).expect("facts parent");
    let candidate = fixture_row();
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), candidate).expect("candidate registry");
    let digest = ci_action_item_accounting::fixuptask_v2_digest(candidate.as_bytes());
    let facts = format!(
        "{{\"fixuptask_v2_durable\":{{\"merge_base\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"merge_base\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"merge_base_tree\":\"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\",\"merge_base_rows\":[],\"candidate_registry_digest\":\"{digest}\",\"evaluation_time\":\"2026-07-22T00:00:00Z\"}}}}"
    );
    std::fs::write(root.join(PROTECTED_FACTS_PATH), facts).expect("protected facts");

    let error = evaluate_materialized_gate(&root).expect_err("duplicate protected key must reject");
    assert!(matches!(error, CollectError::Parse { line: 1, .. }));
    assert!(
        error
            .to_string()
            .contains("duplicate object key: merge_base")
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn live_durable_gate_evaluates_the_workspace_registry_and_protected_facts() {
    let findings = evaluate_materialized_gate(&repo_root())
        .expect("workspace registry and protected facts must be readable");
    assert!(findings.is_empty(), "live durable findings: {findings:#?}");
}

#[test]
fn materialized_gate_fails_closed_when_protected_facts_are_missing() {
    let root =
        std::env::temp_dir().join(format!("fixuptask-v2-missing-facts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), fixture_row()).expect("candidate registry");

    assert!(matches!(
        evaluate_materialized_gate(&root),
        Err(CollectError::Io(_))
    ));
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn materialized_gate_fails_closed_on_invalid_protected_facts() {
    let root =
        std::env::temp_dir().join(format!("fixuptask-v2-invalid-facts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::create_dir_all(root.join("ci/facade/scm-facts-snapshot")).expect("facts parent");
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), fixture_row()).expect("candidate registry");
    std::fs::write(root.join(PROTECTED_FACTS_PATH), "{}").expect("protected facts");

    let findings = evaluate_materialized_gate(&root).expect("invalid facts produce findings");
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "fixuptask_v2_protected_facts_missing")
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn materialized_gate_fails_closed_on_stale_facts() {
    let root =
        std::env::temp_dir().join(format!("fixuptask-v2-stale-facts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::create_dir_all(root.join("ci/facade/scm-facts-snapshot")).expect("facts parent");
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), fixture_row()).expect("candidate registry");
    let facts = json!({ "fixuptask_v2_durable": {
        "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "merge_base_rows": [],
        "candidate_registry_digest": ci_action_item_accounting::fixuptask_v2_digest(b"stale"),
        "evaluation_time": "2026-07-22T00:00:00Z"
    }});
    std::fs::write(
        root.join(PROTECTED_FACTS_PATH),
        serde_json::to_vec(&facts).expect("facts json"),
    )
    .expect("protected facts");

    let findings = evaluate_materialized_gate(&root).expect("stale facts produce findings");
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "fixuptask_v2_candidate_registry_digest_mismatch")
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn materialized_gate_fails_closed_on_invalid_candidate_registry_row() {
    let root = std::env::temp_dir().join(format!(
        "fixuptask-v2-invalid-candidate-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::create_dir_all(root.join("ci/facade/scm-facts-snapshot")).expect("facts parent");
    let candidate = b"{\"id\":\"F-INVALID\"}\n";
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), candidate).expect("candidate registry");
    let facts = json!({ "fixuptask_v2_durable": {
        "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "merge_base_rows": [],
        "candidate_registry_digest": ci_action_item_accounting::fixuptask_v2_digest(candidate),
        "evaluation_time": "2026-07-22T00:00:00Z"
    }});
    std::fs::write(
        root.join(PROTECTED_FACTS_PATH),
        serde_json::to_vec(&facts).expect("facts json"),
    )
    .expect("protected facts");

    let findings = evaluate_materialized_gate(&root).expect("invalid candidate produces findings");
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "fixuptask_v2_schema_required_field")
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn materialized_gate_rejects_duplicate_candidate_ids() {
    let root = std::env::temp_dir().join(format!(
        "fixuptask-v2-duplicate-candidate-ids-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::create_dir_all(root.join("ci/facade/scm-facts-snapshot")).expect("facts parent");
    let candidate = format!("{}{}", fixture_row(), fixture_row());
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), &candidate).expect("candidate registry");
    let facts = json!({ "fixuptask_v2_durable": {
        "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "merge_base_rows": [],
        "candidate_registry_digest": ci_action_item_accounting::fixuptask_v2_digest(candidate.as_bytes()),
        "evaluation_time": "2026-07-22T00:00:00Z"
    }});
    std::fs::write(
        root.join(PROTECTED_FACTS_PATH),
        serde_json::to_vec(&facts).expect("facts json"),
    )
    .expect("protected facts");

    let findings = evaluate_materialized_gate(&root).expect("candidate registry is readable");
    assert!(
        findings.iter().any(
            |finding| finding.code == "fixuptask_v2_duplicate_id" && finding.key == "F-DURABLE"
        )
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn materialized_gate_rejects_deletion_of_a_protected_row() {
    let root = std::env::temp_dir().join(format!(
        "fixuptask-v2-protected-row-deletion-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::create_dir_all(root.join("ci/facade/scm-facts-snapshot")).expect("facts parent");
    let candidate = "";
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), candidate).expect("candidate registry");
    let protected_row = serde_json::from_str::<serde_json::Value>(fixture_row()).expect("row");
    let facts = json!({ "fixuptask_v2_durable": {
        "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "merge_base_rows": [protected_row],
        "candidate_registry_digest": ci_action_item_accounting::fixuptask_v2_digest(candidate.as_bytes()),
        "evaluation_time": "2026-07-22T00:00:00Z"
    }});
    std::fs::write(
        root.join(PROTECTED_FACTS_PATH),
        serde_json::to_vec(&facts).expect("facts json"),
    )
    .expect("protected facts");

    let findings = evaluate_materialized_gate(&root).expect("candidate registry is readable");
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "fixuptask_v2_row_deleted" && finding.key == "F-DURABLE")
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn materialized_gate_rejects_expired_accepted_risk() {
    let root = std::env::temp_dir().join(format!(
        "fixuptask-v2-expired-accepted-risk-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::create_dir_all(root.join("ci/facade/scm-facts-snapshot")).expect("facts parent");
    let candidate = fixture_row().replacen(
        "\"status\":\"open\"",
        "\"status\":\"accepted-risk\"",
        1,
    ).replacen(
        "\"blocker_for\":\"none\"}",
        "\"blocker_for\":\"none\",\"qualified_human_decision_ref\":\"ADR-0622\",\"accepted_risk_expires_at\":\"2026-07-21T23:59:59Z\",\"accepted_risk_evidence\":\"waiver\"}",
        1,
    );
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), &candidate).expect("candidate registry");
    let facts = json!({ "fixuptask_v2_durable": {
        "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "merge_base_rows": [],
        "candidate_registry_digest": ci_action_item_accounting::fixuptask_v2_digest(candidate.as_bytes()),
        "evaluation_time": "2026-07-22T00:00:00Z"
    }});
    std::fs::write(
        root.join(PROTECTED_FACTS_PATH),
        serde_json::to_vec(&facts).expect("facts json"),
    )
    .expect("protected facts");

    let findings = evaluate_materialized_gate(&root).expect("candidate registry is readable");
    assert!(findings.iter().any(
        |finding| finding.code == "fixuptask_v2_accepted_risk_expired"
            && finding.key == "F-DURABLE"
    ));
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn materialized_gate_reports_the_malformed_jsonl_line_number() {
    let root = std::env::temp_dir().join(format!(
        "fixuptask-v2-malformed-jsonl-line-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("registry")).expect("registry parent");
    std::fs::create_dir_all(root.join("ci/facade/scm-facts-snapshot")).expect("facts parent");
    let candidate = format!("{}{{malformed json}}\n", fixture_row());
    std::fs::write(root.join(CANDIDATE_REGISTRY_PATH), &candidate).expect("candidate registry");
    let facts = json!({ "fixuptask_v2_durable": {
        "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "merge_base_rows": [],
        "candidate_registry_digest": ci_action_item_accounting::fixuptask_v2_digest(candidate.as_bytes()),
        "evaluation_time": "2026-07-22T00:00:00Z"
    }});
    std::fs::write(
        root.join(PROTECTED_FACTS_PATH),
        serde_json::to_vec(&facts).expect("facts json"),
    )
    .expect("protected facts");

    let error = evaluate_materialized_gate(&root).expect_err("malformed line must reject");
    assert!(matches!(error, CollectError::Parse { line: 2, .. }));
    std::fs::remove_dir_all(root).expect("cleanup");
}
