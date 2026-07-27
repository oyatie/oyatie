// Phase-0 capability-first reorg tier-dependency acyclicity lane (ADR-0245/0280/0562): the live-corpus
// + RED-fixture + burn-down-fixture gate.
//
// 1. LIVE: collect the REAL crate/dependency/tier corpus from the live tree (cargo path-deps + buck
//    deps, projected through the per-service manifest tier facets) and assert the gate is GREEN —
//    zero REGRESSIONS vs the freshly-frozen baseline. This is the born-ADVISORY proof: the live tree
//    == the baseline, so there are zero regressions.
// 2. BASELINE FIDELITY: every entry in the frozen baseline is STILL observed on the live tree (the
//    baseline is neither stale — listing a fixed edge — nor over-broad). Burn-down would shrink it.
// 3. RED FIXTURE: a synthetic substrate->product edge MUST fail the gate (a wrong-tier edge that
//    passes is a false-green).
// 4. BURN-DOWN FIXTURE: removing a baselined inverting edge keeps the gate GREEN (improving is always
//    allowed) and reports the burn-down.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::Value;

use ci_layer_dependency_acyclicity::{
    BASELINE_PATH, GATE_ID, POLICY_PATH, Status, Verdict, collect_corpus, evaluate, load_json,
    parse_baseline,
};

/// Walk up from the test's working directory to the repo root (the dir holding the gate policy).
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
    panic!("failed to locate repo root (the dir holding {POLICY_PATH}) from the test current_dir");
}

/// Locate the on-disk fixtures directory (cargo: `$CARGO_MANIFEST_DIR/tests/fixtures`; buck: walk up).
fn fixtures_dir() -> PathBuf {
    if let Some(manifest) = std::env::var_os("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(manifest).join("tests/fixtures");
        if p.is_dir() {
            return p;
        }
    }
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        for cand in [
            dir.join("tests/fixtures"),
            dir.join(
                "ci/facade/layer-dependency-acyclicity/tests/fixtures",
            ),
        ] {
            if cand.is_dir() {
                return cand;
            }
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate the tier-dependency-acyclicity fixtures directory");
}

#[test]
fn live_tree_is_green_zero_regressions() {
    let root = repo_root();
    let policy = load_json(&root, POLICY_PATH).expect("load policy");
    let baseline = load_json(&root, BASELINE_PATH).expect("load baseline");
    let observed = collect_corpus(&root, &policy).expect("collect live corpus");
    let report = evaluate(&policy, &baseline, &observed);

    assert_eq!(
        report.verdict,
        Verdict::Green,
        "the live tree MUST be GREEN (zero regressions vs the frozen baseline). regressions:\n{}",
        report
            .findings
            .iter()
            .filter(|f| f.status == Status::Regression)
            .map(|f| format!("  [{}] {}: {}", f.code, f.subject, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(report.regressions, 0, "born-advisory: zero regressions at birth");
    assert!(
        report.crates_checked > 700,
        "the scan must cover the real corpus (got {})",
        report.crates_checked
    );
    eprintln!(
        "{GATE_ID} live corpus: GREEN — {} crates / {} edges / {} baselined advisory / 0 regressions",
        report.crates_checked, report.edges_checked, report.baselined
    );
}

#[test]
fn frozen_baseline_is_exactly_the_live_violation_set() {
    // The baseline must equal the CURRENT live violation set: every baselined entry is still observed
    // (not stale) AND there are no live violations missing from the baseline (no regression). This is
    // the "live tree == baseline" born invariant.
    let root = repo_root();
    let policy = load_json(&root, POLICY_PATH).expect("load policy");
    let baseline_doc = load_json(&root, BASELINE_PATH).expect("load baseline");
    let baseline = parse_baseline(&baseline_doc).expect("parse baseline");
    let observed = collect_corpus(&root, &policy).expect("collect live corpus");
    let report = evaluate(&policy, &baseline_doc, &observed);

    // No baselined entry has been silently fixed (burn-down would surface here, requiring a re-freeze).
    assert_eq!(
        report.burned_down, 0,
        "a baselined entry is no longer present — re-freeze the baseline (it lists a fixed edge)"
    );
    // Every live finding is baselined (none is a regression).
    let live_keys: std::collections::BTreeSet<String> = report
        .findings
        .iter()
        .map(|f| format!("{}|{}", f.code, f.subject))
        .collect();
    assert_eq!(
        live_keys, baseline.keys,
        "the live violation set must equal the frozen baseline exactly"
    );
    assert_eq!(
        baseline.keys.len(),
        8,
        "the frozen baseline holds 8 tier inversions (12 at birth, less the 3 cloud-kms -> residency \
         S-RANK-INVERSIONs burned down by ADR-0562 move-19: oya-residency-domain left the cloud/ \
         tier'd substrate root for the unclassified network/ capability home; less the 1 \
         oya-saas-bench-app -> oya-saas-plugin-app SUBSTRATE-UPWARD edge burned down by ADR-0562 \
         move-21: oya-saas-bench-app left the cloud/ tier'd substrate root for the unclassified \
         billing/ capability home, so that inversion left the classified graph)"
    );
}

/// A fixture-scoped policy: the live policy's rules + S-rank order, but a zero crate floor (the
/// synthetic fixtures hold only a couple of crates, far below the live false-green floor).
fn fixture_policy() -> serde_json::Value {
    let root = repo_root();
    let mut policy = load_json(&root, POLICY_PATH).expect("load policy");
    policy["min_expected_crates"] = serde_json::json!(0);
    policy
}

#[test]
fn red_fixture_substrate_to_product_fails_closed() {
    let policy = fixture_policy();
    // Evaluate the synthetic RED corpus against an EMPTY baseline → the wrong-tier edge is a
    // regression and the gate fails closed.
    let empty_baseline = serde_json::json!({ "gate_id": GATE_ID, "violations": [] });
    let observed = load_json(&fixtures_dir_root(), "red-substrate-to-product.json")
        .expect("load RED fixture");
    let report = evaluate(&policy, &empty_baseline, &observed);
    assert_eq!(
        report.verdict,
        Verdict::Red,
        "a synthetic substrate->product edge MUST fail the gate (false-green guard); findings: {:?}",
        report.findings
    );
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-SUBSTRATE-UPWARD")
        .expect("a substrate-upward finding");
    assert_eq!(f.status, Status::Regression);
}

#[test]
fn burn_down_fixture_stays_green() {
    let policy = fixture_policy();
    // The baseline still lists the (now-removed) inverting edge; the corpus no longer has it.
    let baseline = serde_json::json!({
        "gate_id": GATE_ID,
        "violations": [
            {
                "code": "TDA-SUBSTRATE-UPWARD",
                "subject": "cloud/synth-substrate/crates/oya-synth-substrate-app -> oya/synth-product/crates/oya-synth-product-app"
            }
        ]
    });
    let observed = load_json(&fixtures_dir_root(), "burn-down.json").expect("load burn-down fixture");
    let report = evaluate(&policy, &baseline, &observed);
    assert_eq!(report.verdict, Verdict::Green, "burning down a baselined violation is allowed");
    assert_eq!(report.regressions, 0);
    assert_eq!(report.burned_down, 1, "the fixed baselined edge counts as burned down");
}

#[test]
fn stale_baseline_phantom_row_fails_closed() {
    // B3 hardening — RED liveness fixture: a committed baseline subject whose `from` crate is ABSENT
    // from the live corpus is a phantom row a subset baseline can never RED on (an in-flight strangler
    // MOVE leaves the OLD-path edge behind). The liveness backstop must fail the gate closed.
    let policy = fixture_policy();
    let baseline = serde_json::json!({
        "gate_id": GATE_ID,
        "violations": [
            {
                "code": "TDA-SUBSTRATE-UPWARD",
                "subject": "cloud/moved-away/crates/oya-moved-away-app -> oya/synth-product/crates/oya-synth-product-app"
            }
        ]
    });
    // burn-down.json's corpus contains oya/synth-product/... but NOT cloud/moved-away/... .
    let observed = load_json(&fixtures_dir_root(), "burn-down.json").expect("load fixture corpus");
    let report = evaluate(&policy, &baseline, &observed);
    assert_eq!(
        report.verdict,
        Verdict::Red,
        "a phantom baseline row (crate absent from the live corpus) MUST fail the gate; findings: {:?}",
        report.findings
    );
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-STALE-BASELINE")
        .expect("a stale-baseline finding");
    assert_eq!(f.status, Status::Regression);
    assert_eq!(
        report.burned_down, 0,
        "a phantom baseline row is stale debt, not burn-down progress"
    );
}

#[test]
fn baseline_with_all_subjects_present_is_not_stale() {
    // B3 hardening — GREEN liveness fixture: every baseline subject's endpoints exist in the live
    // corpus, so there are no phantom rows. The liveness backstop stays quiet even though the inverting
    // EDGE was removed (a legitimate burn-down) and the gate stays GREEN.
    let policy = fixture_policy();
    let baseline = serde_json::json!({
        "gate_id": GATE_ID,
        "violations": [
            {
                "code": "TDA-SUBSTRATE-UPWARD",
                "subject": "cloud/synth-substrate/crates/oya-synth-substrate-app -> oya/synth-product/crates/oya-synth-product-app"
            }
        ]
    });
    let observed = load_json(&fixtures_dir_root(), "burn-down.json").expect("load fixture corpus");
    let report = evaluate(&policy, &baseline, &observed);
    assert!(
        !report.findings.iter().any(|f| f.code == "TDA-STALE-BASELINE"),
        "all baseline subjects present -> no phantom rows: {:?}",
        report.findings
    );
    assert_eq!(report.verdict, Verdict::Green);
}

/// The fixtures dir as a `root` argument for `load_json` (which joins `root`/`path`).
fn fixtures_dir_root() -> PathBuf {
    fixtures_dir()
}

#[test]
fn every_governed_glob_root_is_declared_in_the_policy() {
    // The invariant that MAKES the zero-debt property hold, checked over policy data alone: a
    // crate_root_glob whose first segment is declared in neither service_roots nor
    // unclassified_roots silently produces unenforced crates the moment one lands under it
    // (TDA-UNDECLARED-ROOT). Catching that at policy-edit time beats catching it on arrival.
    let policy: Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join(POLICY_PATH)).expect("read committed policy"),
    )
    .expect("policy parses");

    let declared: BTreeSet<&str> = policy["service_roots"]
        .as_array()
        .expect("service_roots")
        .iter()
        .chain(policy["unclassified_roots"].as_array().expect("unclassified_roots"))
        .map(|v| v.as_str().expect("root is a string"))
        .collect();

    let undeclared: Vec<&str> = policy["crate_root_globs"]
        .as_array()
        .expect("crate_root_globs")
        .iter()
        .filter_map(|g| g.as_str().and_then(|g| g.split('/').next()))
        .filter(|root| !declared.contains(root))
        .collect();

    assert!(
        undeclared.is_empty(),
        "crate_root_globs roots declared in neither service_roots nor unclassified_roots: \
         {undeclared:?}; crates landing under these are silently unenforced"
    );
}
