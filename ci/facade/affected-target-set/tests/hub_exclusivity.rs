// cloud-ci-hub-exclusivity gate (ADR-0711 / Swarm Delivery Law).
//
// 1. Policy pack cites specs/integ-branch-envelopes.json#hubs.paths — never re-lists hubs.
// 2. RED fixture: multi-owned hub among open integ PRs MUST Refuse.
// 3. GREEN fixture: sole owner is Green.
// 4. When envelopes exist on tip: load live #hubs.paths and prove extract + non-empty authority.
//    When absent (parked integ/ci before #1644 land): skip live load; fixture proofs still bind.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use ci_affected_target_set::hub_exclusivity::{
    CODE_MULTI_OWN_HUB, GATE_ID, HUBS_PATHS_POINTER, HubAuthority, HubExclusivityPolicy,
    OpenPrFact, Verdict, evaluate, evaluate_from_producer_docs, hubs_paths_from_envelopes,
    open_pr_facts_from_json,
};
use serde_json::Value;

const POLICY_PATH: &str = "ci/facade/affected-target-set/hub-exclusivity-policy.json";
const ENVELOPES_PATH: &str = "specs/integ-branch-envelopes.json";

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(POLICY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (dir holding {POLICY_PATH})");
}

fn load_policy(root: &Path) -> HubExclusivityPolicy {
    let raw = fs::read_to_string(root.join(POLICY_PATH)).expect("read hub-exclusivity policy");
    let doc: Value = serde_json::from_str(&raw).expect("policy JSON");
    HubExclusivityPolicy::from_json(&doc)
}

fn pr(number: u64, head: &str, files: &[&str]) -> OpenPrFact {
    OpenPrFact {
        number,
        head_ref_name: head.to_owned(),
        files: files.iter().map(|f| (*f).to_owned()).collect(),
    }
}

#[test]
fn shipped_policy_cites_hubs_paths_pointer_and_gate_id() {
    let root = repo_root();
    let policy = load_policy(&root);
    assert_eq!(policy.gate_id, GATE_ID);
    assert_eq!(policy.hubs_paths_authority, HUBS_PATHS_POINTER);
    assert!(policy.sole_owner_per_wave);
    assert_eq!(policy.integ_head_ref_prefix, "integ/");

    // Anti-drift: policy file body must not embed a hubs.paths array of real hubs.
    let raw = fs::read_to_string(root.join(POLICY_PATH)).expect("read");
    assert!(
        !raw.contains("\"paths\""),
        "hub-exclusivity policy must cite {HUBS_PATHS_POINTER}, not re-list hubs.paths"
    );
}

#[test]
fn red_fixture_multi_own_hub_refuses() {
    let root = repo_root();
    let policy = load_policy(&root);
    // Synthetic hubs — not a re-list of live #hubs.paths; prove the refuse mechanism.
    let authority = HubAuthority {
        paths: ["fixture/hub-a".to_owned(), "fixture/hub-b".to_owned()]
            .into_iter()
            .collect(),
    };
    let open = [
        pr(100, "integ/a", &["fixture/hub-a", "a/x.rs"]),
        pr(200, "integ/b", &["fixture/hub-a", "b/y.rs"]),
    ];
    let report = evaluate(&policy, &authority, &open);
    assert_eq!(report.verdict, Verdict::Refuse);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == CODE_MULTI_OWN_HUB && f.key == "fixture/hub-a"),
        "expected hub_multi_owned for fixture/hub-a, got {:?}",
        report.findings
    );
}

#[test]
fn green_fixture_sole_owner_passes() {
    let root = repo_root();
    let policy = load_policy(&root);
    let authority = HubAuthority {
        paths: BTreeSet::from(["fixture/hub-a".to_owned()]),
    };
    let open = [
        pr(100, "integ/a", &["fixture/hub-a"]),
        pr(200, "integ/b", &["other/file.rs"]),
    ];
    let report = evaluate(&policy, &authority, &open);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
}

#[test]
fn live_envelopes_hubs_paths_bind_when_present() {
    let root = repo_root();
    let envelopes = root.join(ENVELOPES_PATH);
    if !envelopes.is_file() {
        // Parked integ/ci tip may lack envelopes until #1644 lands — fixture proofs above bind.
        eprintln!(
            "skip live envelopes bind: {ENVELOPES_PATH} absent on tip (expected until integ/specs lands)"
        );
        return;
    }
    let raw = fs::read_to_string(&envelopes).expect("read envelopes");
    let doc: Value = serde_json::from_str(&raw).expect("envelopes JSON");
    let authority = hubs_paths_from_envelopes(&doc).expect("hubs.paths parse");
    assert!(
        !authority.paths.is_empty(),
        "live {HUBS_PATHS_POINTER} must be non-empty"
    );

    let policy = load_policy(&root);
    // No open PRs → no multi-own; authority non-empty → Green.
    let report = evaluate(&policy, &authority, &[]);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "live authority with empty open set must be Green, got {:?}",
        report.findings
    );
}

#[test]
fn producer_fixture_multi_own_refuses_via_evaluate_from_producer_docs() {
    let root = repo_root();
    let policy_doc: Value =
        serde_json::from_str(&fs::read_to_string(root.join(POLICY_PATH)).expect("policy"))
            .expect("policy json");
    let envelopes_doc: Value = serde_json::from_str(
        &fs::read_to_string(root.join(
            "ci/facade/affected-target-set/tests/fixtures/hub_exclusivity/envelopes-synthetic.json",
        ))
        .expect("synthetic envelopes"),
    )
    .expect("envelopes json");
    let open_prs_doc: Value = serde_json::from_str(
        &fs::read_to_string(root.join(
            "ci/facade/affected-target-set/tests/fixtures/hub_exclusivity/open-prs-multi-own.json",
        ))
        .expect("open prs fixture"),
    )
    .expect("open prs json");
    let facts = open_pr_facts_from_json(&open_prs_doc).expect("facts");
    assert_eq!(facts.len(), 2);
    let report = evaluate_from_producer_docs(&policy_doc, &envelopes_doc, &open_prs_doc);
    assert_eq!(report.verdict, Verdict::Refuse);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == CODE_MULTI_OWN_HUB && f.key == "fixture/hub-a"),
        "expected hub_multi_owned for fixture/hub-a, got {:?}",
        report.findings
    );
}

#[test]
fn producer_fixture_sole_owner_is_green() {
    let root = repo_root();
    let policy_doc: Value =
        serde_json::from_str(&fs::read_to_string(root.join(POLICY_PATH)).expect("policy"))
            .expect("policy json");
    let envelopes_doc: Value = serde_json::from_str(
        &fs::read_to_string(root.join(
            "ci/facade/affected-target-set/tests/fixtures/hub_exclusivity/envelopes-synthetic.json",
        ))
        .expect("synthetic envelopes"),
    )
    .expect("envelopes json");
    let open_prs_doc: Value = serde_json::from_str(
        &fs::read_to_string(root.join(
            "ci/facade/affected-target-set/tests/fixtures/hub_exclusivity/open-prs-sole-owner.json",
        ))
        .expect("open prs fixture"),
    )
    .expect("open prs json");
    let report = evaluate_from_producer_docs(&policy_doc, &envelopes_doc, &open_prs_doc);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
}
