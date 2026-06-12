// cloud-ci-firewall — the single required GO-LIVE status check. Regenerates the gate
// baseline over the LIVE tree, loads the FROZEN merge-base reference (the gate-baseline
// face at `git merge-base <base_ref> HEAD`, materialized out-of-graph by the scm-facts
// emitter — ADR-0551, fixes FRIC-1781112000) + the sign-off door, and runs both pure
// predicates (compare-mode + ratchet-invariant). The reference is NEVER the PR-local face:
// the settle protocol mandates regeneration and registry-drift mandates
// committed==regenerated, so a PR-local reference is grown by the very regen the protocol
// requires (the PR #670 laundering exhibit — pinned below as a foil). This is the proof
// that, with the baseline frozen at the merge-base, the firewall is GREEN on the current
// corpus (no NEW debt) yet still blocks any NEW finite violation.
// ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_firewall_app::{
    Baseline, FrozenBaseline, SignOff, baseline_keys_map, evaluate_firewall,
    FROZEN_SNAPSHOT_PATH, RATCHET_POLICY_PATH, SIGNOFF_FIXER_COMMAND, SIGNOFF_PATH,
};
use serde_json::Value;

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

fn faces_dir(root: &Path) -> PathBuf {
    root.join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app")
}

// The firewall-side file paths are lib constants (single owner) shared with the signoff
// fixer — gate and fixer can never disagree about which files they police.
fn signoff_path(root: &Path) -> PathBuf {
    root.join(SIGNOFF_PATH)
}

fn frozen_snapshot_path(root: &Path) -> PathBuf {
    root.join(FROZEN_SNAPSHOT_PATH)
}

fn ratchet_policy_path(root: &Path) -> PathBuf {
    root.join(RATCHET_POLICY_PATH)
}

/// Load the FROZEN merge-base reference. FAIL-CLOSED: a missing or invalid snapshot is a
/// hard failure with the exact remediation, never a silent fall-back to the PR-local face
/// (the FRIC-1781112000 laundering hole this gate exists to close).
fn load_frozen_baseline(root: &Path) -> FrozenBaseline {
    let path = frozen_snapshot_path(root);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "FAIL-CLOSED: merge-base frozen baseline snapshot missing at {} ({e}). The \
             firewall compares against the gate-baseline face at `git merge-base <base_ref> \
             HEAD` (ADR-0551, FRIC-1781112000), never the PR-local copy. Materialize it: \
             infra/ci/materialize-cloud-ci-generated-faces.sh . (CI runs this before every \
             gate lane).",
            path.display()
        )
    });
    let value: Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
    FrozenBaseline::from_value(&value)
        .unwrap_or_else(|e| panic!("invalid frozen baseline snapshot {}: {e}", path.display()))
}

fn load_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Regenerate the gate-baseline face from the LIVE tree (in --stdout sandbox mode),
/// HERMETICALLY (no `env!("CARGO")`, the compile-time cargo-only macro that breaks the buck2
/// build). The producer binary is resolved at RUNTIME: under buck2 from `OYA_CI_PRODUCER_BIN`
/// (the `$(exe ...)`-substituted built binary), else under cargo via the runtime `CARGO` env
/// var. The producer reads the committed scm-facts face (a declared input); it never calls git.
fn regenerate_baseline(root: &Path) -> Value {
    let scm_facts = faces_dir(root).join("scm-facts.generated.json");
    let output = if let Ok(bin) = std::env::var("OYA_CI_PRODUCER_BIN") {
        let bin = if Path::new(&bin).is_absolute() {
            PathBuf::from(bin)
        } else {
            root.join(bin)
        };
        Command::new(bin)
            .arg("--repo-root")
            .arg(root)
            .arg("--scm-facts")
            .arg(&scm_facts)
            .arg("--stdout")
            .arg("--face")
            .arg("baseline")
            .current_dir(root)
            .output()
            .expect("run producer binary")
    } else {
        Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()))
            .arg("run")
            .arg("--quiet")
            .arg("-p")
            .arg("oya-cloud-ci-accounting-registry-app")
            .arg("--")
            .arg("--repo-root")
            .arg(root)
            .arg("--scm-facts")
            .arg(&scm_facts)
            .arg("--stdout")
            .arg("--face")
            .arg("baseline")
            .current_dir(root)
            .output()
            .expect("cargo run oya-cloud-ci-accounting-registry-app --face baseline")
    };
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("baseline stdout is valid JSON")
}

fn fixture_dir(root: &Path) -> PathBuf {
    root.join("specs/fixtures/cloud-ci-firewall")
}

fn current_from_value(value: &Value) -> BTreeMap<String, BTreeMap<String, BTreeSet<String>>> {
    let mut out: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
    if let Some(gates) = value.as_object() {
        for (gate, codes) in gates {
            let mut code_map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            if let Some(codes_obj) = codes.as_object() {
                for (code, keys) in codes_obj {
                    let set: BTreeSet<String> = keys
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_owned)
                                .collect()
                        })
                        .unwrap_or_default();
                    code_map.insert(code.clone(), set);
                }
            }
            out.insert(gate.clone(), code_map);
        }
    }
    out
}

/// Fixture-driven RED/GREEN corpus: each tc-*.json carries a merge_base_baseline (the
/// FROZEN reference) + current + proposed_baseline + signoff and the expected firewall
/// verdict / failing codes / ratchet growth count. The compare-mode + ratchet-invariant
/// predicates are pure, so the fixtures drive them with zero scanner special-cases (the
/// per-code behaviour is DATA: mode + frozen_empty). This is the data-under-test contract,
/// mirroring the four gate corpora.
#[test]
fn firewall_fixtures_execute_red_green_cases() {
    let dir = fixture_dir(&repo_root());
    let mut tc_paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("tc-") && n.ends_with(".json"))
        })
        .collect();
    tc_paths.sort();
    assert!(
        !tc_paths.is_empty(),
        "firewall fixture corpus must not be empty"
    );

    let mut seen_green = false;
    let mut seen_red = false;

    for path in &tc_paths {
        let fixture = load_json(path);
        let label = path.file_name().unwrap().to_string_lossy().to_string();

        assert!(
            fixture.get("committed_baseline").is_none(),
            "{label}: stale fixture field committed_baseline — the frozen reference is the \
             merge-base baseline (merge_base_baseline) per ADR-0551/FRIC-1781112000"
        );
        let frozen = Baseline::from_value(&fixture["merge_base_baseline"]);
        let proposed = Baseline::from_value(&fixture["proposed_baseline"]);
        let signoff = SignOff::from_value(&fixture["signoff"]);
        let current = current_from_value(&fixture["current"]);

        let report = evaluate_firewall(&frozen, &proposed, &current, &signoff);

        let expected_growth = fixture["expected_ratchet_growth"].as_u64().unwrap_or(0) as usize;
        assert_eq!(
            report.ratchet_growth.len(),
            expected_growth,
            "{label}: ratchet_growth count mismatch (growth = {:?})",
            report.ratchet_growth
        );

        let expected_inert = fixture["expected_inert_signoff"].as_u64().unwrap_or(0) as usize;
        assert_eq!(
            report.inert_signoff.len(),
            expected_inert,
            "{label}: inert_signoff count mismatch (inert = {:?})",
            report.inert_signoff
        );

        let expected_failing: BTreeSet<String> = fixture["expected_failing_codes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let actual_failing: BTreeSet<String> = report
            .codes
            .iter()
            .filter(|r| r.fails())
            .map(|r| r.code.clone())
            .collect();
        assert_eq!(
            actual_failing, expected_failing,
            "{label}: failing-code set mismatch"
        );

        match fixture["expected_firewall"].as_str() {
            Some("GREEN") => {
                seen_green = true;
                assert!(report.is_green(), "{label} must be GREEN");
            }
            Some("RED") => {
                seen_red = true;
                assert!(!report.is_green(), "{label} must be RED");
            }
            other => panic!("{label} has unsupported expected_firewall {other:?}"),
        }
    }

    assert!(
        seen_green && seen_red,
        "firewall fixtures must include BOTH RED and GREEN cases"
    );
}

/// THE GO-LIVE PROOF: with the FROZEN merge-base reference, the firewall is GREEN on the
/// live corpus. A settled change only ever SHRINKS the baseline relative to the merge-base
/// (or grows it through the sign-off door), so:
///   - compare-mode: current keys ⊆ frozen ∪ signed-off => zero regressions => no
///     baseline-block-on-new code fails (advisory codes report their counts but never fail);
///   - ratchet-invariant: blocking proposed keys ⊆ frozen ∪ signed-off => zero growth.
#[test]
fn firewall_is_green_on_the_live_corpus_with_the_baseline() {
    let root = repo_root();

    // The FROZEN reference: the gate-baseline face at `git merge-base <base_ref> HEAD`,
    // materialized out-of-graph by the scm-facts emitter. NEVER the PR-local face.
    let frozen = load_frozen_baseline(&root);

    // The proposed baseline = what TODAY's corpus would freeze.
    let proposed_value = regenerate_baseline(&root);
    let proposed = Baseline::from_value(&proposed_value);

    // The sign-off door (the one-way exemption; empty = ratchet fully closed).
    let signoff = SignOff::from_value(&load_json(&signoff_path(&root)));

    // The live "current" keyed violations == the proposed baseline's keys (the producer
    // captured them via evaluate_keyed over the live faces).
    let current = baseline_keys_map(&proposed);

    let report = evaluate_firewall(&frozen.baseline, &proposed, &current, &signoff);

    // Evidence digest: per-code current/baseline/regressions/fixed/tolerated/signed-off.
    eprintln!(
        "FIREWALL GO-LIVE report (live corpus vs frozen {} @ merge-base {}):",
        frozen.base_ref, frozen.merge_base
    );
    for r in &report.codes {
        eprintln!(
            "  [{}] {:48} mode={:22} current={:6} baseline={:6} regressions={:4} fixed={:4} tolerated={:6} signed_off={:4}{}",
            r.gate,
            r.code,
            r.mode,
            r.current,
            r.baseline,
            r.regressions.len(),
            r.fixed.len(),
            r.tolerated.len(),
            r.signed_off.len(),
            if r.fails() { "  <-- FAIL" } else { "" }
        );
    }
    eprintln!(
        "  ratchet_growth (un-signed-off blocking baseline additions vs merge-base): {}",
        report.ratchet_growth.len()
    );
    eprintln!(
        "  inert_signoff (door entries exempting nothing — retire them): {}",
        report.inert_signoff.len()
    );

    let failing: Vec<&str> = report
        .codes
        .iter()
        .filter(|r| r.fails())
        .map(|r| r.code.as_str())
        .collect();
    let regression_detail: Vec<(&str, &str, Vec<&str>)> = report
        .codes
        .iter()
        .filter(|r| r.fails())
        .map(|r| {
            (
                r.gate.as_str(),
                r.code.as_str(),
                r.regressions.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert!(
        failing.is_empty(),
        "GO-LIVE: firewall must be GREEN on today's corpus (no NEW debt vs the merge-base), \
         but these codes FAIL: {failing:?}; regressions: {regression_detail:?}"
    );
    assert!(
        report.ratchet_growth.is_empty(),
        "GO-LIVE: blocking baseline keys must shrink (or pass the sign-off door) relative \
         to the merge-base, got growth {:?}",
        report.ratchet_growth
    );
    assert!(
        report.inert_signoff.is_empty(),
        "GO-LIVE: every sign-off door entry must exempt something that exists (frozen, \
         current, or proposed) — an inert entry is a standing re-introduction ticket. \
         Remediation (auto-derives + applies the retirement): {SIGNOFF_FIXER_COMMAND} \
         Inert: {:?}",
        report.inert_signoff
    );
    assert!(
        report.is_green(),
        "firewall must be GREEN with the merge-base frozen reference"
    );

    // Sanity: the baseline is NON-trivial (the frozen pre-existing corpus debt is real).
    let total_baselined: usize = report.codes.iter().map(|r| r.baseline).sum();
    assert!(
        total_baselined > 0,
        "the baseline must freeze the real pre-existing corpus debt"
    );
}

/// The frozen snapshot's provenance must agree with the committed ratchet policy: same
/// configurable base_ref (R0 policy-as-data), a full revision id, and the exact face path —
/// the audit trail naming WHICH frozen point the firewall compared against. Under
/// frozen-policy-wins (FRIC-1781280000) the snapshot's facts come from the MERGE-BASE
/// policy, so this doubles as the live pin that the candidate and frozen policies agree —
/// any divergence (e.g. a same-PR repoint) goes RED here.
#[test]
fn frozen_snapshot_provenance_matches_ratchet_policy() {
    let root = repo_root();
    let frozen = load_frozen_baseline(&root);
    let policy = load_json(&ratchet_policy_path(&root));
    assert_eq!(
        frozen.base_ref,
        policy["base_ref"].as_str().expect("policy base_ref"),
        "snapshot base_ref (the FROZEN merge-base policy's) must agree with the candidate \
         ratchet-policy.json"
    );
    assert_eq!(frozen.merge_base.len(), 40, "full hex revision id");
    assert_eq!(
        frozen.frozen_policy_source, "merge-base",
        "this repo's ratchet policy exists at the merge-base (merged in PR #698): the \
         frozen-policy-wins path must be the one actually exercised, never the \
         candidate-bootstrap fallback"
    );
    let snapshot = load_json(&frozen_snapshot_path(&root));
    assert_eq!(
        snapshot["face_path"],
        policy["frozen_reference"]["face_path"],
        "snapshot must record the (frozen) policy face path"
    );
    assert!(
        !frozen.missing_at_merge_base,
        "this repo's gate-baseline face exists at the merge-base; a missing-face snapshot \
         here means the emitter extracted the wrong path"
    );
}

/// THE F1 PIN (defense-in-depth on top of frozen-policy-wins, never instead of it): the
/// committed comparison root is `origin/dev`. A same-PR `base_ref` repoint can no longer
/// select the PR's own frozen reference (the emitter reads the frozen-side policy from the
/// merge-base tree), but it could still change post-merge behavior silently — this pin
/// makes any repoint require a visible edit to THIS test as well.
#[test]
fn ratchet_policy_base_ref_is_pinned_to_origin_dev() {
    let root = repo_root();
    let policy = load_json(&ratchet_policy_path(&root));
    assert_eq!(
        policy["base_ref"], "origin/dev",
        "ratchet-policy.json base_ref repointed: the frozen comparison root for this \
         repository is origin/dev (ADR-0551). If this repoint is intentional it requires \
         founder sign-off, an update to the out-of-band bootstrap (--frozen-base-ref / \
         DEFAULT_FROZEN_BOOTSTRAP_REF in the scm-facts emitter), and an edit to this pin \
         (FRIC-1781280000)."
    );
}

/// THE FRIC-1781112000 PIN. A PR adds new debt AND regenerates the baseline face in the
/// same change — exactly what the settle protocol mandates and registry-drift enforces, so
/// the PR-local face always equals the regenerated face. FOIL: against the PR-local
/// reference the laundering is structurally invisible (GREEN). GATE: against the FROZEN
/// merge-base reference the same state is RED — both as compare-mode regressions and as
/// ratchet growth. This is the misattribution shape from PR #670 (a new
/// manifest-hygiene debt key absorbed by a same-PR baseline regen, 21/21 checks green).
#[test]
fn firewall_blocks_same_pr_baseline_regen_laundering() {
    let root = repo_root();
    let frozen = load_frozen_baseline(&root);
    let signoff = SignOff::from_value(&load_json(&signoff_path(&root)));

    // The laundering PR: new debt appears in the regenerated (and therefore committed)
    // baseline face and in the live current set simultaneously.
    let mut proposed_value = regenerate_baseline(&root);
    if let Some(keys) = proposed_value
        .get_mut("gates")
        .and_then(|g| g.get_mut("cloud-ci-total-accounting"))
        .and_then(|g| g.get_mut("unjustified"))
        .and_then(|c| c.get_mut("keys"))
        .and_then(Value::as_array_mut)
    {
        keys.push(Value::String("SYNTHETIC/laundered-in-same-pr.rs".to_owned()));
    }
    let proposed = Baseline::from_value(&proposed_value);
    let pr_local_reference = proposed.clone(); // settle protocol: committed == regenerated
    let current = baseline_keys_map(&proposed);

    // FOIL — the historical hole: the PR-local reference cannot see its own laundering.
    let laundered = evaluate_firewall(&pr_local_reference, &proposed, &current, &signoff);
    assert!(
        laundered.is_green(),
        "FOIL: against the PR-local reference the laundering must be invisible — if this \
         fails, the foil no longer demonstrates the hole and the pin needs re-derivation"
    );

    // THE GATE: the frozen merge-base reference blocks it, on BOTH predicates.
    let report = evaluate_firewall(&frozen.baseline, &proposed, &current, &signoff);
    assert!(
        report.ratchet_growth.iter().any(|(_, code, key)| {
            code == "unjustified" && key == "SYNTHETIC/laundered-in-same-pr.rs"
        }),
        "same-PR baseline regen must be ratchet growth vs the merge-base: {:?}",
        report.ratchet_growth
    );
    let unjust = report
        .codes
        .iter()
        .find(|r| r.gate == "cloud-ci-total-accounting" && r.code == "unjustified")
        .expect("unjustified code present");
    assert!(
        unjust
            .regressions
            .contains("SYNTHETIC/laundered-in-same-pr.rs"),
        "the laundered key must also be a compare-mode regression vs the merge-base"
    );
    assert!(!report.is_green(), "firewall must be RED on same-PR laundering");
}

/// RED-on-NEW proof against the LIVE corpus: inject ONE synthetic NEW key into the live
/// "current" set for a baseline-block-on-new code and assert the firewall FAILS — proving
/// the gate still blocks any new finite violation that is not in the frozen baseline.
#[test]
fn firewall_goes_red_on_a_synthetic_new_violation() {
    let root = repo_root();
    let frozen = load_frozen_baseline(&root);
    let proposed = Baseline::from_value(&regenerate_baseline(&root));
    let signoff = SignOff::from_value(&load_json(&signoff_path(&root)));

    let mut current = baseline_keys_map(&proposed);
    // Add a NEW unjustified path that is NOT in the frozen merge-base baseline.
    current
        .entry("cloud-ci-total-accounting".to_owned())
        .or_default()
        .entry("unjustified".to_owned())
        .or_default()
        .insert("SYNTHETIC/new-unjustified-file.rs".to_owned());

    let report = evaluate_firewall(&frozen.baseline, &proposed, &current, &signoff);
    let unjust = report
        .codes
        .iter()
        .find(|r| r.gate == "cloud-ci-total-accounting" && r.code == "unjustified")
        .expect("unjustified code present");
    assert!(
        unjust
            .regressions
            .contains("SYNTHETIC/new-unjustified-file.rs"),
        "the synthetic NEW file must show up as a regression"
    );
    assert!(
        unjust.fails(),
        "a NEW unjustified file must FAIL the firewall"
    );
    assert!(
        !report.is_green(),
        "firewall must be RED on a NEW finite violation"
    );
}

/// RATCHET proof against the LIVE corpus: a regen that GROWS the baseline (without sign-off)
/// relative to the FROZEN merge-base reference is a ratchet_regression — debt cannot be
/// laundered into the baseline by re-running the producer.
#[test]
fn firewall_blocks_baseline_growth_without_signoff() {
    let root = repo_root();
    let frozen = load_frozen_baseline(&root);
    // A proposed baseline that ADDS a key beyond the frozen set.
    let mut proposed_value = regenerate_baseline(&root);
    if let Some(keys) = proposed_value
        .get_mut("gates")
        .and_then(|g| g.get_mut("cloud-ci-total-accounting"))
        .and_then(|g| g.get_mut("unjustified"))
        .and_then(|c| c.get_mut("keys"))
        .and_then(Value::as_array_mut)
    {
        keys.push(Value::String("SYNTHETIC/laundered-debt.rs".to_owned()));
    }
    let proposed = Baseline::from_value(&proposed_value);
    let current: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();

    // Empty sign-off => the grown key is NOT exempt.
    let report = evaluate_firewall(&frozen.baseline, &proposed, &current, &SignOff::default());
    assert!(
        report
            .ratchet_growth
            .iter()
            .any(|(_, code, key)| code == "unjustified" && key == "SYNTHETIC/laundered-debt.rs"),
        "growing the baseline without sign-off must be a ratchet_regression"
    );
    assert!(!report.is_green());
}
