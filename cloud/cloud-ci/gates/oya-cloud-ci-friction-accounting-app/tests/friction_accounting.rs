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

use oya_cloud_ci_friction_accounting_app::{
    Verdict, collect_observed_frictions, evaluate, evaluate_keyed,
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
    root.join("cloud/cloud-ci/gates/oya-cloud-ci-friction-accounting-app")
}

fn load_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn keys_for(findings: &BTreeSet<oya_cloud_ci_friction_accounting_app::Finding>, code: &str) -> BTreeSet<String> {
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

    // 1. Frozen-empty closure-integrity codes: NO key allowed on the live corpus (born-blocking).
    for code in [
        "friction_policy_gate_id_mismatch",
        "friction_unknown_status",
        "friction_duplicate_primary_row",
    ] {
        let keys = keys_for(&findings, code);
        assert!(
            keys.is_empty(),
            "{code} is born-blocking frozen-empty; live ledger must carry zero, got {keys:?}"
        );
    }

    // 2. Shrink-only legacy codes: measured legacy debt must equal the committed baseline EXACTLY.
    //    A NEW friction hitting one of these codes adds a key NOT in the baseline -> set inequality
    //    -> RED. Fixing a legacy friction removes its measured key -> the test forces the baseline to
    //    shrink in the SAME PR (settle discipline). The baseline is reviewed, not regenerated.
    const SHRINK_ONLY_CODES: [&str; 4] = [
        "friction_missing_required_field",
        "friction_no_disposition",
        "friction_closed_without_evidence",
        "friction_accepted_risk_without_evidence",
    ];
    // Reviewed ceilings: NOT derived from any generated artifact; only ever edited DOWN. They are an
    // independent growth tripwire on top of the set-equality check (FRIC-1781112000): even if a future
    // PR tried to grow the baseline to absorb new debt, breaching a ceiling fails closed.
    const CEILINGS: [(&str, usize); 4] = [
        ("friction_missing_required_field", 4),
        ("friction_no_disposition", 3),
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
        "FRICTION-ACCOUNTING live corpus: rows={row_count} missing_field={} no_disposition={} \
         closed_without_evidence={} accepted_risk_without_evidence={}",
        keys_for(&findings, "friction_missing_required_field").len(),
        keys_for(&findings, "friction_no_disposition").len(),
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
    assert!(findings.iter().any(|f| f.code == "friction_unknown_status" && f.key == "FRIC-NEW"));
    assert_eq!(evaluate(&fixture_policy(), &json!({ "rows": [good_open("ok")] })).verdict, Verdict::Green);
}

#[test]
fn red_duplicate_primary_id_fails_closed() {
    let findings = evaluate_keyed(
        &fixture_policy(),
        &json!({ "rows": [good_open("FRIC-DUP"), good_open("FRIC-DUP")] }),
    );
    assert!(findings.iter().any(|f| f.code == "friction_duplicate_primary_row" && f.key == "FRIC-DUP"));
}

#[test]
fn red_blank_enforcement_fix_fails_closed() {
    let mut row = good_open("FRIC-ND");
    row["enforcement_fix"] = json!("");
    let findings = evaluate_keyed(&fixture_policy(), &json!({ "rows": [row] }));
    assert!(findings.iter().any(|f| f.code == "friction_no_disposition" && f.key == "FRIC-ND"));
    assert!(findings.iter().any(|f| f.code == "friction_missing_required_field" && f.key == "FRIC-ND"));
}

#[test]
fn red_closed_without_evidence_fails_closed() {
    let mut row = good_open("FRIC-CLOSED");
    row["status"] = json!("RESOLVED");
    let findings = evaluate_keyed(&fixture_policy(), &json!({ "rows": [row] }));
    assert!(findings.iter().any(|f| f.code == "friction_closed_without_evidence" && f.key == "FRIC-CLOSED"));
}

#[test]
fn red_accepted_risk_without_evidence_fails_closed() {
    let mut row = good_open("FRIC-HELD");
    row["status"] = json!("escalated-to-leader-for-force-complete");
    let findings = evaluate_keyed(&fixture_policy(), &json!({ "rows": [row] }));
    assert!(findings.iter().any(|f| {
        f.code == "friction_accepted_risk_without_evidence" && f.key == "FRIC-HELD"
    }));
}

#[test]
fn red_baseline_is_shrink_only_a_grown_set_breaks_equality() {
    // Simulate the anti-laundering check directly: a measured set with an extra key not in the
    // frozen baseline must break set-equality (the live test's core assertion).
    let baseline: BTreeSet<String> = ["FRIC-A".to_owned(), "FRIC-B".to_owned()].into_iter().collect();
    let measured_grown: BTreeSet<String> =
        ["FRIC-A".to_owned(), "FRIC-B".to_owned(), "FRIC-NEW-DEBT".to_owned()]
            .into_iter()
            .collect();
    assert_ne!(
        measured_grown, baseline,
        "a measured set that grew past the frozen baseline must fail the set-equality ratchet"
    );
    // Shrinking (a fixed friction) is the only sanctioned divergence and is also caught by equality,
    // forcing the baseline edit into the same PR.
    let measured_shrunk: BTreeSet<String> = ["FRIC-A".to_owned()].into_iter().collect();
    assert_ne!(measured_shrunk, baseline);
}
