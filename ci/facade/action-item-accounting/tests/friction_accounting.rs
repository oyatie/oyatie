// ADR-0544 FRIC-total-accounting: born-blocking self-test over TODAY's real friction ledger. The
// test collects the policy-declared ledger, folds the event-sourced append rows, and asserts:
//   * the closure-integrity frozen-empty codes (duplicate primary, unknown status, gate-id mismatch)
//     are EMPTY on the live corpus — any new occurrence fails closed;
//   * the schema/disposition/closure legacy codes match the committed shrink-only baseline EXACTLY
//     (set equality), so a NEW friction triggering one of them is born-blocking (not in the baseline)
//     while frozen legacy debt only shrinks. The baseline is a reviewed, NON-regenerated artifact
//     (FRIC-1781112000 anti-laundering): a baseline edited in the same PR cannot launder new debt.
//   * each baselined code stays under an independent reviewed CEILING that only moves DOWN.
// RED fixtures prove every violation class fails closed without a filesystem.
// ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ci_action_item_accounting::{
    FIXUPTASK_V2_CANDIDATE_JSONL_PATH, LEGACY_FRICTION_MAPPING_PATH,
    LEGACY_FRICTION_PROTECTED_FACTS_PATH, Verdict, collect_observed_frictions, evaluate,
    evaluate_keyed, evaluate_legacy_friction_admission, evaluate_legacy_friction_materialized_gate,
    fixuptask_v2, fixuptask_v2_digest, legacy_friction_adapter,
};
use serde_json::{Value, json};

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

fn gate_dir(root: &Path) -> PathBuf {
    root.join("ci/facade/action-item-accounting")
}

fn load_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

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
                (1..=189)
                    .map(|index| format!("FRIC-{index:03}"))
                    .collect::<Vec<_>>()
                    .join("\n")
                    .as_bytes(),
            ),
            "candidate_present": true,
            "candidate_digest": digest
        }
    }})
}

fn keys_for(
    findings: &BTreeSet<ci_action_item_accounting::Finding>,
    code: &str,
) -> BTreeSet<String> {
    findings
        .iter()
        .filter(|finding| finding.code == code)
        .map(|finding| finding.key.clone())
        .collect()
}

fn baseline_keys(baseline: &Value, code: &str) -> BTreeSet<String> {
    baseline["codes"][code]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn live_friction_ledger_meets_the_closed_loop_contract() {
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("friction-accounting-policy.json"));
    let baseline = load_json(&gate_dir(&root).join("friction-accounting-baseline.json"));

    let observed = collect_observed_frictions(&root, &policy)
        .expect("read-only ledger collection should not need temp files or cleanup");
    let row_count = observed["rows"].as_array().expect("rows").len();
    assert!(
        row_count >= 60,
        "the live friction ledger should carry at least the go-live census; got {row_count}"
    );

    let findings = evaluate_keyed(&policy, &observed);

    // 1. Born-blocking-empty codes: NO key allowed on the live corpus. Two frozen-empty closure
    //    codes plus friction_no_disposition, which the orphan-update refactor made clean today (every
    //    surviving friction either declares an enforcement_fix, is accepted-risk, or is an orphan
    //    accounted under friction_orphan_update_row). Any new occurrence of any of these fails closed.
    for code in [
        "friction_policy_gate_id_mismatch",
        "friction_unknown_status",
        "friction_duplicate_primary_row",
        "friction_no_disposition",
    ] {
        let keys = keys_for(&findings, code);
        assert!(
            keys.is_empty(),
            "{code} is born-blocking empty on the live corpus; got {keys:?}"
        );
    }

    // 2. Shrink-only legacy codes: measured legacy debt must equal the committed baseline EXACTLY.
    //    A NEW friction hitting one of these codes adds a key NOT in the baseline -> set inequality
    //    -> RED. Fixing a legacy friction removes its measured key -> the test forces the baseline to
    //    shrink in the SAME PR (settle discipline). The baseline is reviewed, not regenerated.
    const SHRINK_ONLY_CODES: [&str; 4] = [
        "friction_missing_required_field",
        "friction_orphan_update_row",
        "friction_closed_without_evidence",
        "friction_accepted_risk_without_evidence",
    ];
    // Reviewed ceilings: NOT derived from any generated artifact; only ever edited DOWN. They are an
    // independent growth tripwire on top of the set-equality check (FRIC-1781112000): even if a future
    // PR tried to grow the baseline to absorb new debt, breaching a ceiling fails closed.
    const CEILINGS: [(&str, usize); 4] = [
        ("friction_missing_required_field", 4),
        ("friction_orphan_update_row", 3),
        ("friction_closed_without_evidence", 2),
        ("friction_accepted_risk_without_evidence", 7),
    ];

    for code in SHRINK_ONLY_CODES {
        let measured = keys_for(&findings, code);
        let frozen = baseline_keys(&baseline, code);
        assert_eq!(
            measured, frozen,
            "{code}: measured legacy debt must equal the committed baseline EXACTLY; a new key means \
             born-blocking new debt (add the evidence/field instead of growing the baseline), a \
             removed key means a fixed friction (shrink the baseline in this PR)"
        );
    }
    for (code, ceiling) in CEILINGS {
        let measured = keys_for(&findings, code).len();
        assert!(
            measured <= ceiling,
            "{code} debt grew past the reviewed ceiling ({measured} > {ceiling}); new debt is \
             born-blocking — fix the friction, do not raise the ceiling"
        );
    }

    eprintln!(
        "FRICTION-ACCOUNTING live corpus: rows={row_count} missing_field={} orphan_update={} \
         closed_without_evidence={} accepted_risk_without_evidence={}",
        keys_for(&findings, "friction_missing_required_field").len(),
        keys_for(&findings, "friction_orphan_update_row").len(),
        keys_for(&findings, "friction_closed_without_evidence").len(),
        keys_for(&findings, "friction_accepted_risk_without_evidence").len(),
    );
}

#[test]
fn policy_gate_id_matches_the_crate_contract() {
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("friction-accounting-policy.json"));
    assert_eq!(
        policy["gate_id"].as_str(),
        Some("cloud-ci-friction-accounting")
    );
}

#[test]
fn live_action_item_gate_consumes_the_canonical_materialized_scm_snapshot() {
    let root = repo_root();
    let findings = evaluate_legacy_friction_materialized_gate(&root)
        .expect("missing or unreadable canonical SCM facts must fail the action-item gate");
    assert!(
        findings.is_empty(),
        "FixupTask v2 admission must be green before the legacy ledger changes: {findings:#?}"
    );
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
    std::fs::write(
        root.join(FIXUPTASK_V2_CANDIDATE_JSONL_PATH),
        concat!(
            "{\"_meta\":\"registry header\"}\n",
            "{\"id\":\"F-V2-GATE\",\"title\":\"gate fixture\",\"priority\":\"high\",\"status\":\"open\",\"source_session\":\"session\",\"source_change_id\":\"change\",\"named_in\":\"ADR-0621\",\"created_at\":\"2026-07-21T00:00:00Z\",\"accountable_owner\":\"owner\",\"accountable_role\":\"role\",\"acceptance_criteria\":\"criterion\",\"verification_path\":\"buck2 test\",\"blocker_for\":\"none\"}\n"
        ),
    )
    .expect("write candidate JSONL");
    std::fs::write(
        root.join(LEGACY_FRICTION_MAPPING_PATH),
        "{\"source\":\"git:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:.omc/ultragoal/friction-ledger.jsonl\",\"entries\":[{\"predecessor_id\":\"FRIC-1\",\"target_fixuptask_id\":\"F-V2-GATE\"}]}",
    )
    .expect("write candidate mapping");
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

// ---------------------------------------------------------------------------
// RED fixtures: each violation class fails closed without touching the filesystem.
// ---------------------------------------------------------------------------

fn fixture_policy() -> Value {
    json!({
        "gate_id": "cloud-ci-friction-accounting",
        "required_primary_fields": ["id", "seen_at", "friction", "enforcement_fix", "status"],
        "status_match": "prefix",
        "terminal_requires_evidence": true,
        "accepted_risk_requires_evidence": true,
        "status_taxonomy": {
            "open": "open",
            "escalated-to-leader": "accepted-risk",
            "RESOLVED": "terminal"
        }
    })
}

fn good_open(id: &str) -> Value {
    json!({
        "id": id, "seen_at": "2026-06-10", "friction": "x",
        "pipeline_defect": "y", "enforcement_fix": "wire a gate", "status": "open"
    })
}

#[test]
fn red_unregistered_status_fails_closed() {
    let mut row = good_open("FRIC-NEW");
    row["status"] = json!("brand-new-unmapped-status");
    let findings = evaluate_keyed(&fixture_policy(), &json!({ "rows": [row] }));
    assert!(
        findings
            .iter()
            .any(|f| f.code == "friction_unknown_status" && f.key == "FRIC-NEW")
    );
    assert_eq!(
        evaluate(&fixture_policy(), &json!({ "rows": [good_open("ok")] })).verdict,
        Verdict::Green
    );
}

#[test]
fn red_duplicate_primary_id_fails_closed() {
    let findings = evaluate_keyed(
        &fixture_policy(),
        &json!({ "rows": [good_open("FRIC-DUP"), good_open("FRIC-DUP")] }),
    );
    assert!(
        findings
            .iter()
            .any(|f| f.code == "friction_duplicate_primary_row" && f.key == "FRIC-DUP")
    );
}

#[test]
fn red_blank_enforcement_fix_fails_closed() {
    let mut row = good_open("FRIC-ND");
    row["enforcement_fix"] = json!("");
    let findings = evaluate_keyed(&fixture_policy(), &json!({ "rows": [row] }));
    assert!(
        findings
            .iter()
            .any(|f| f.code == "friction_no_disposition" && f.key == "FRIC-ND")
    );
    assert!(
        findings
            .iter()
            .any(|f| f.code == "friction_missing_required_field" && f.key == "FRIC-ND")
    );
}

#[test]
fn red_closed_without_evidence_fails_closed() {
    let mut row = good_open("FRIC-CLOSED");
    row["status"] = json!("RESOLVED");
    let findings = evaluate_keyed(&fixture_policy(), &json!({ "rows": [row] }));
    assert!(
        findings
            .iter()
            .any(|f| f.code == "friction_closed_without_evidence" && f.key == "FRIC-CLOSED")
    );
}

#[test]
fn red_accepted_risk_without_evidence_fails_closed() {
    let mut row = good_open("FRIC-HELD");
    row["status"] = json!("escalated-to-leader-for-force-complete");
    let findings = evaluate_keyed(&fixture_policy(), &json!({ "rows": [row] }));
    assert!(
        findings.iter().any(|f| {
            f.code == "friction_accepted_risk_without_evidence" && f.key == "FRIC-HELD"
        })
    );
}

#[test]
fn red_orphan_update_only_friction_fails_closed() {
    // The evasion the CRITICAL review caught: a friction logged ONLY as update-shaped rows (no
    // anchoring primary) would otherwise fold to a clean terminal-with-evidence state and pass every
    // check. The orphan code makes the missing primary itself the (sole) violation for that id.
    let orphan = json!({
        "id": "FRIC-ORPHAN",
        "seen_at": "2026-06-10",
        "status_update": "RESOLVED",
        "enforcement_fix": "looks disposed",
        "evidence": "looks closed"
    });
    let findings = evaluate_keyed(&fixture_policy(), &json!({ "rows": [orphan] }));
    assert!(
        findings
            .iter()
            .any(|f| f.code == "friction_orphan_update_row" && f.key == "FRIC-ORPHAN"),
        "an update-only friction must fail closed as an orphan: {findings:#?}"
    );
    // It is the ONLY finding for that id (the orphan supersedes the downstream class checks).
    assert_eq!(
        findings.iter().filter(|f| f.key == "FRIC-ORPHAN").count(),
        1,
        "orphan should emit exactly one finding for the id: {findings:#?}"
    );
}

#[test]
fn red_baseline_is_shrink_only_new_debt_breaks_set_equality() {
    // Drive the REAL evaluator (not hand-built sets): a fixture ledger carrying a frozen legacy debt
    // key PLUS a new debt key must produce a measured set that is NOT equal to a baseline containing
    // only the legacy key — the exact set-equality the live-repo test enforces. This proves the
    // anti-laundering check rejects new debt rather than asserting std-lib set inequality.
    let legacy = good_open("FRIC-LEGACY"); // measured open+disposed -> green; stand-in anchor
    let mut legacy_debt = good_open("FRIC-LEGACY-DEBT");
    legacy_debt["status"] = json!("RESOLVED"); // terminal, no evidence -> closed_without_evidence
    let mut new_debt = good_open("FRIC-NEW-DEBT");
    new_debt["status"] = json!("RESOLVED"); // terminal, no evidence -> closed_without_evidence (NEW)

    let findings = evaluate_keyed(
        &fixture_policy(),
        &json!({ "rows": [legacy, legacy_debt, new_debt] }),
    );
    let measured: BTreeSet<String> = findings
        .iter()
        .filter(|f| f.code == "friction_closed_without_evidence")
        .map(|f| f.key.clone())
        .collect();
    // The "committed baseline" froze only the legacy debt key.
    let frozen: BTreeSet<String> = ["FRIC-LEGACY-DEBT".to_owned()].into_iter().collect();
    assert_ne!(
        measured, frozen,
        "a NEW closed-without-evidence key must break baseline set-equality, not be absorbed"
    );
    assert!(
        measured.contains("FRIC-NEW-DEBT"),
        "the evaluator must surface the new debt key: {measured:?}"
    );
}

#[test]
fn violation_codes_const_covers_every_emitted_code() {
    // Guard against VIOLATION_CODES drifting from what the evaluator actually emits (review LOW-7):
    // exercise every code at least once and assert each emitted code is declared in the const.
    let declared: BTreeSet<&str> = ci_action_item_accounting::VIOLATION_CODES
        .into_iter()
        .collect();
    let mut bad_policy = fixture_policy();
    bad_policy["gate_id"] = json!("cloud-ci-wrong");
    let mut missing = good_open("FRIC-M");
    missing["seen_at"] = json!("");
    let mut closed = good_open("FRIC-C");
    closed["status"] = json!("RESOLVED");
    let mut held = good_open("FRIC-H");
    held["status"] = json!("escalated-to-leader");
    let mut unknown = good_open("FRIC-UK");
    unknown["status"] = json!("never-seen-status");
    let orphan = json!({"id": "FRIC-O", "seen_at": "2026-06-10", "status_update": "RESOLVED"});
    let rows = json!({ "rows": [
        missing, closed, held, unknown, orphan,
        good_open("FRIC-DD"), good_open("FRIC-DD"),
    ]});
    let findings = evaluate_keyed(&bad_policy, &rows);
    let emitted: BTreeSet<&str> = findings.iter().map(|f| f.code.as_str()).collect();
    for code in &emitted {
        assert!(
            declared.contains(code),
            "evaluator emitted `{code}` which is not in VIOLATION_CODES"
        );
    }
    // And confirm this fixture actually exercised most of the surface (no silent under-coverage).
    assert!(
        emitted.len() >= 6,
        "expected broad code coverage, got {emitted:?}"
    );
}
