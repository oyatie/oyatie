//! Nonbinding ADR-0622 prototype and legacy-adapter coverage.
//!
//! This stays outside the conventional ADR-0544 required gate while ADR-0622 remains Proposed.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;

use ci_action_item_accounting::{
    FIXUPTASK_V2_CANDIDATE_JSONL_PATH, LEGACY_FRICTION_MAPPING_PATH,
    LEGACY_FRICTION_PROTECTED_FACTS_PATH, evaluate_legacy_friction_admission, fixuptask_v2,
    fixuptask_v2_digest, legacy_friction_adapter,
};
use serde_json::{Value, json};

fn materialized_fixuptask_v2_facts(candidate_ledger: &[u8]) -> Value {
    let predecessor_ids: Vec<String> = (1..=189).map(|index| format!("FRIC-{index:03}")).collect();
    let digest = fixuptask_v2_digest(candidate_ledger);
    json!({ "fixuptask_v2_admission": {
        "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "merge_base_rows": [],
        "predecessor_source": "git:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:.omc/ultragoal/friction-ledger.jsonl",
        "predecessor_ids": predecessor_ids,
        "evaluation_time": "2026-07-21T00:00:00Z",
        "legacy_ledger": {
            "path": ".omc/ultragoal/friction-ledger.jsonl",
            "merge_base_blob": "cccccccccccccccccccccccccccccccccccccccc",
            "merge_base_digest": digest,
            "predecessor_ids_digest": fixuptask_v2_digest(
                (1..=189).map(|index| format!("FRIC-{index:03}")).collect::<Vec<_>>().join("\n").as_bytes(),
            ),
            "candidate_present": true,
            "candidate_digest": digest
        }
    }})
}

#[test]
fn live_durable_gate_consumes_the_canonical_materialized_scm_snapshot() {
    let mut root = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if root.join("specs/root-hub-pointers.json").is_file() {
            let findings = fixuptask_v2::evaluate_materialized_gate(&root)
                .expect("missing or unreadable canonical SCM facts must fail the durable gate");
            assert!(
                findings.is_empty(),
                "FixupTask v2 durable admission must be green: {findings:#?}"
            );
            return;
        }
        assert!(
            root.pop(),
            "failed to locate repo root from test current_dir"
        );
    }
    panic!("failed to locate repo root from test current_dir");
}

#[test]
fn fixuptask_v2_admission_is_wired_through_the_materialized_gate_inputs() {
    assert_eq!(
        legacy_friction_adapter::GATE_ID,
        "cloud-ci-legacy-friction-adapter"
    );
    let root = std::env::temp_dir().join(format!(
        "ci-action-item-accounting-v2-gate-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    for path in [
        FIXUPTASK_V2_CANDIDATE_JSONL_PATH,
        LEGACY_FRICTION_MAPPING_PATH,
        LEGACY_FRICTION_PROTECTED_FACTS_PATH,
        ".omc/ultragoal/friction-ledger.jsonl",
    ] {
        std::fs::create_dir_all(root.join(path).parent().expect("input parent"))
            .expect("create materialized input parent");
    }
    std::fs::write(root.join(FIXUPTASK_V2_CANDIDATE_JSONL_PATH), concat!("{\"_meta\":\"registry header\"}\n", "{\"id\":\"F-V2-GATE\",\"title\":\"gate fixture\",\"priority\":\"high\",\"status\":\"open\",\"source_session\":\"session\",\"source_change_id\":\"change\",\"named_in\":\"ADR-0621\",\"created_at\":\"2026-07-21T00:00:00Z\",\"accountable_owner\":\"owner\",\"accountable_role\":\"role\",\"acceptance_criteria\":\"criterion\",\"verification_path\":\"buck2 test\",\"blocker_for\":\"none\"}\n")).expect("write candidate JSONL");
    std::fs::write(root.join(LEGACY_FRICTION_MAPPING_PATH), "{\"source\":\"git:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:.omc/ultragoal/friction-ledger.jsonl\",\"entries\":[{\"predecessor_id\":\"FRIC-1\",\"target_fixuptask_id\":\"F-V2-GATE\"}]}").expect("write candidate mapping");
    std::fs::write(root.join(".omc/ultragoal/friction-ledger.jsonl"), "legacy")
        .expect("write unchanged legacy ledger");
    std::fs::write(
        root.join(LEGACY_FRICTION_PROTECTED_FACTS_PATH),
        serde_json::to_string(&materialized_fixuptask_v2_facts(b"legacy"))
            .expect("serialize SCM-materialized facts"),
    )
    .expect("write SCM-materialized facts");
    assert!(
        legacy_friction_adapter::evaluate_materialized_gate(&root)
            .expect("materialized adapter must read all three gate inputs")
            .is_empty()
    );
    std::fs::remove_dir_all(root).expect("remove test inputs");
}

#[test]
fn legacy_adapter_v2_findings_cannot_diverge_from_the_durable_kernel() {
    let candidate = json!({ "rows": [7] });
    let protected = materialized_fixuptask_v2_facts(b"legacy");
    let legacy = evaluate_legacy_friction_admission(&protected, &candidate, Some(b"legacy"), None)
        .into_iter()
        .map(|finding| (finding.code, finding.key))
        .collect::<BTreeSet<_>>();
    let durable = fixuptask_v2::evaluate_fixuptasks_v2_at(
        &json!({ "rows": [] }),
        &candidate,
        "2026-07-21T00:00:00Z",
    )
    .into_iter()
    .map(|finding| (finding.code, finding.key))
    .collect::<BTreeSet<_>>();
    assert_eq!(
        legacy, durable,
        "legacy adapter must delegate v2 validation to the durable kernel"
    );
}
