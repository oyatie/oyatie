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
            dir.join("ci/facade/layer-dependency-acyclicity/tests/fixtures"),
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
    assert_eq!(
        report.regressions, 0,
        "born-advisory: zero regressions at birth"
    );
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
        34,
        "the frozen baseline holds 34 rows: 8 SUBSTRATE-UPWARD + 9 S-RANK-INVERSION edges, plus 17 \
         UNCLASSIFIED-ROOT-NOT-META roots.\n\
         \n\
         The root rows dropped 21 -> 17 when messaging/ci/storage were tier-declared (ADR-0631 \
         floor test) and `policy` was removed as a ROOT. Three of those four are real burn-down; \
         `policy` is a bookkeeping deletion, not a fix — the capability keeps its live \
         policy-engine DAG node, but it owns ZERO crates (the nine Cedar/PDP crates stay \
         iam-mapped per ADR-0615 to avoid a membership double-map), so `policy/*/*` matched \
         nothing and the exemption governed nothing. Re-homing those crates into `policy/` is a \
         MOVE, filed separately; it will need this root back.\n\
         \n\
         The S-RANK-INVERSION count did NOT move (9, unchanged): declaring the three surfaced \
         ZERO new inversions. That is a fact about the neighbourhood, not a clean bill of health \
         — messaging's and storage's ranked neighbours (audit/cell/network/secrets) are already \
         classified and they sit legally, while every one of ci's neighbours (libs, intelligence, \
         governance) is STILL unclassified, so ci's S5 constrains no live edge yet. Verified by \
         perturbation: forcing messaging to S1 or storage to S1 each REDs one inversion, but \
         forcing ci to S1 REDs nothing. ci's S4 floor becomes real when `intelligence` moves to \
         capability_roots.\n\
         \n\
         The 9 S-RANK-INVERSIONs are the point of the capability_roots change. This assertion \
         previously read 8 and explained the drop from 12 as 'burned down by ADR-0562 move-19: \
         oya-residency-domain left the cloud/ tier'd substrate root for the UNCLASSIFIED network/ \
         capability home'. That was not burn-down — the inverting edges still exist; relocating one \
         endpoint into an unenforced root removed them from the comparison. Tier-classifying \
         network/ (with cell/observability/secrets/audit) brings them back, which is why this \
         number went UP: the gate now sees inversions it had been structurally blind to.\n\
         \n\
         The 21 root rows are the capability roots still exempt; each burns down as its root moves \
         to capability_roots. --emit-baseline never MINTS one (see the baseline _comment), so a \
         structural exemption cannot be laundered by re-running the tool; it does carry the rows \
         already committed here forward, filtered to those still live, so a re-emit does not \
         silently delete 21 advisory rows and turn them into 21 regressions."
    );
}

/// A fixture-scoped policy: the live policy's rules + S-rank order, but a zero crate floor (the
/// synthetic fixtures hold only a couple of crates, far below the live false-green floor), and the
/// ROOT-scoped rules (R6b/R6c) narrowed to the synthetic tree. The fixtures exist to exercise the
/// EDGE rules over two synthetic services; carrying the live root lists would make every fixture
/// verdict a function of the live repo's 21 outstanding capability exemptions instead of the edge
/// under test.
fn fixture_policy() -> serde_json::Value {
    let root = repo_root();
    let mut policy = load_json(&root, POLICY_PATH).expect("load policy");
    policy["min_expected_crates"] = serde_json::json!(0);
    policy["capability_roots"] = serde_json::json!([]);
    // Matches the fixtures' `registry_meta_dirs`, so R6b is quiet.
    policy["unclassified_roots"] = serde_json::json!(["os"]);
    policy
}

#[test]
fn red_fixture_substrate_to_product_fails_closed() {
    let policy = fixture_policy();
    // Evaluate the synthetic RED corpus against an EMPTY baseline → the wrong-tier edge is a
    // regression and the gate fails closed.
    let empty_baseline = serde_json::json!({ "gate_id": GATE_ID, "violations": [] });
    let observed =
        load_json(&fixtures_dir_root(), "red-substrate-to-product.json").expect("load RED fixture");
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
    let observed =
        load_json(&fixtures_dir_root(), "burn-down.json").expect("load burn-down fixture");
    let report = evaluate(&policy, &baseline, &observed);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "burning down a baselined violation is allowed"
    );
    assert_eq!(report.regressions, 0);
    assert_eq!(
        report.burned_down, 1,
        "the fixed baselined edge counts as burned down"
    );
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
        !report
            .findings
            .iter()
            .any(|f| f.code == "TDA-STALE-BASELINE"),
        "all baseline subjects present -> no phantom rows: {:?}",
        report.findings
    );
    assert_eq!(report.verdict, Verdict::Green);
}

/// The fixtures dir as a `root` argument for `load_json` (which joins `root`/`path`).
fn fixtures_dir_root() -> PathBuf {
    fixtures_dir()
}

/// A throwaway repo tree for the collection-side tests. Returns `(root, policy)`.
fn scratch_repo(label: &str, registry: &str) -> (PathBuf, Value) {
    let root = std::env::temp_dir().join(format!(
        "tda-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    ));
    std::fs::create_dir_all(root.join("capx/core/domain")).expect("scratch tree");
    std::fs::write(
        root.join("capx/core/domain/Cargo.toml"),
        "[package]\nname = \"x\"\n",
    )
    .expect("crate");
    std::fs::write(root.join("registry.json"), registry).expect("registry");
    let policy = serde_json::json!({
        "gate_id": GATE_ID,
        "enforcement": "advisory-baseline",
        "crate_root_globs": ["capx/*/*"],
        "service_roots": ["cloud", "oya"],
        "capability_roots": ["capx"],
        "capability_registry_path": "registry.json",
        "unclassified_roots": ["os"],
        "stratum_rank_order": ["S0", "S1", "S2", "S3", "S4", "S5"],
        "min_expected_crates": 0
    });
    (root, policy)
}

/// Write a service `manifest.json` at `<root>/<rel>`.
fn write_service(root: &std::path::Path, rel: &str, stratum: &str) {
    std::fs::create_dir_all(root.join(rel)).expect("service dir");
    std::fs::write(
        root.join(rel).join("manifest.json"),
        format!(r#"{{"tier":"substrate","substrate_dag_position":{{"stratum":"{stratum}"}}}}"#),
    )
    .expect("service manifest");
}

#[test]
fn a_capability_tier_is_declared_not_derived_from_the_dirs_it_absorbs() {
    // HIGH-2 + HIGH-3, which the same change deletes rather than guards.
    //
    // The tier used to be PROJECTED from the services in `absorbs_current_dirs`, resolved through
    // `service_tiers` — built only from `<service_root>/<name>/manifest.json`. Two consequences, both
    // fatal, neither patchable:
    //   HIGH-2: a COMPLETED capability move DELETES those dirs, so the terminal state of a successful
    //           migration was an unresolvable tier. TDA-CAPABILITY-TIER-UNRESOLVED is hardcoded
    //           Status::Regression, so the baseline could not absorb it and --emit-baseline refused
    //           it: the gate failed permanently exactly when the reorg SUCCEEDED.
    //   HIGH-3: unanimity was computed over whichever services had not moved YET, so MIGRATION ORDER
    //           decided the answer — `capx` below spans S1+S3 and has no defensible projected tier,
    //           but move the S3 service first and the survivor projects a confident S1.
    //
    // A DECLARED tier is invariant under both. This test moves the services one at a time and
    // asserts the tier never changes — which is the property, not a symptom of it.
    let (root, policy) = scratch_repo(
        "declared-tier",
        r#"{"capabilities":[{"name":"capx","tier":"substrate",
             "substrate_dag_position":{"stratum":"S2"},
             "absorbs_current_dirs":["capx","cloud/a","cloud/b"]}],
            "meta_directories":[{"dir":"os/"}]}"#,
    );
    let empty_baseline = serde_json::json!({ "gate_id": GATE_ID, "violations": [] });
    let declared = serde_json::json!({"tier": "substrate", "stratum": "S2"});

    // Both absorbed services present, and they DISAGREE (S1 vs S3) — the projection's unresolvable
    // case, which the declaration is unaffected by.
    write_service(&root, "cloud/a", "S1");
    write_service(&root, "cloud/b", "S3");
    let observed = collect_corpus(&root, &policy).expect("collect");
    assert_eq!(
        observed["service_tiers"]["capx"], declared,
        "the registry declaration is the tier, not the absorbed services' (dis)agreement"
    );
    assert_eq!(
        evaluate(&policy, &empty_baseline, &observed).verdict,
        Verdict::Green
    );

    // The S3 service moves in — the projection would now read a unanimous S1.
    std::fs::remove_dir_all(root.join("cloud/b")).expect("complete the S3 move");
    let observed = collect_corpus(&root, &policy).expect("collect");
    assert_eq!(
        observed["service_tiers"]["capx"], declared,
        "migration ORDER must not change the tier"
    );

    // The migration COMPLETES — every absorbed dir is gone, the projection has nothing left at all.
    std::fs::remove_dir_all(root.join("cloud/a")).expect("complete the S1 move");
    let observed = collect_corpus(&root, &policy).expect("collect");
    assert_eq!(
        observed["service_tiers"]["capx"], declared,
        "a COMPLETED move must not orphan the tier"
    );
    assert_eq!(
        evaluate(&policy, &empty_baseline, &observed).verdict,
        Verdict::Green,
        "the gate must not fail exactly when the reorg succeeds"
    );

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn an_undeclared_capability_is_red_and_never_falls_back_to_a_projection() {
    // The other half of "one mechanism": removing the derivation only helps if nothing quietly
    // restores it. A capability with perfectly unanimous absorbed services and NO declared tier must
    // still RED — otherwise the projection is back as a fallback, and a fallback that covers for a
    // missing declaration is exactly how `unclassified_roots` became a silent exemption.
    let (root, policy) = scratch_repo(
        "undeclared",
        r#"{"capabilities":[{"name":"capx",
             "absorbs_current_dirs":["capx","cloud/a","cloud/b"]}],
            "meta_directories":[{"dir":"os/"}]}"#,
    );
    write_service(&root, "cloud/a", "S1");
    write_service(&root, "cloud/b", "S1");

    let observed = collect_corpus(&root, &policy).expect("collect");
    assert!(
        observed["service_tiers"].get("capx").is_none(),
        "unanimous absorbed services must NOT resurrect a projected tier: {}",
        observed["service_tiers"]
    );
    let report = evaluate(
        &policy,
        &serde_json::json!({ "gate_id": GATE_ID, "violations": [] }),
        &observed,
    );
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-CAPABILITY-TIER-UNRESOLVED")
        .expect("an undeclared capability must RED");
    assert_eq!(f.subject, "capx");
    assert_eq!(f.status, Status::Regression);
    assert!(
        f.detail.contains("registry entry"),
        "the remedy must point at the registry; got {}",
        f.detail
    );

    std::fs::remove_dir_all(&root).ok();
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
        .chain(
            policy["capability_roots"]
                .as_array()
                .expect("capability_roots"),
        )
        .chain(
            policy["unclassified_roots"]
                .as_array()
                .expect("unclassified_roots"),
        )
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
