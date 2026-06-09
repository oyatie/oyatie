// cloud-ci-firewall — the single required GO-LIVE status check. Regenerates the gate
// baseline over the LIVE tree, loads the committed baseline + the sign-off door, and runs
// both pure predicates (compare-mode + ratchet-invariant). This is the proof that, with the
// baseline frozen at today, the firewall is GREEN on the current corpus (no NEW debt) yet
// the per-code RED/GREEN unit fixtures prove it still blocks any NEW finite violation.
// ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_firewall_app::{
    baseline_keys_map, evaluate_firewall, Baseline, SignOff,
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

fn signoff_path(root: &Path) -> PathBuf {
    root.join("cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json")
}

fn load_json(path: &Path) -> Value {
    let text =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Regenerate the gate-baseline face from the LIVE tree (in --stdout sandbox mode),
/// HERMETICALLY (no `env!("CARGO")`, the compile-time cargo-only macro that breaks the buck2
/// build). The producer binary is resolved at RUNTIME: under buck2 from `OYA_CI_PRODUCER_BIN`
/// (the `$(exe ...)`-substituted built binary), else under cargo via the runtime `CARGO` env
/// var. The producer reads the committed git-facts face (a declared input); it never calls git.
fn regenerate_baseline(root: &Path) -> Value {
    let git_facts = faces_dir(root).join("git-facts.generated.json");
    let output = if let Ok(bin) = std::env::var("OYA_CI_PRODUCER_BIN") {
        let bin = if Path::new(&bin).is_absolute() {
            PathBuf::from(bin)
        } else {
            root.join(bin)
        };
        Command::new(bin)
            .arg("--repo-root")
            .arg(root)
            .arg("--git-facts")
            .arg(&git_facts)
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
            .arg("--git-facts")
            .arg(&git_facts)
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
                        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
                        .unwrap_or_default();
                    code_map.insert(code.clone(), set);
                }
            }
            out.insert(gate.clone(), code_map);
        }
    }
    out
}

/// Fixture-driven RED/GREEN corpus: each tc-*.json carries a committed_baseline + current +
/// proposed_baseline + signoff and the expected firewall verdict / failing codes / ratchet
/// growth count. The compare-mode + ratchet-invariant predicates are pure, so the fixtures
/// drive them with zero scanner special-cases (the per-code behaviour is DATA: mode +
/// frozen_empty). This is the data-under-test contract, mirroring the four gate corpora.
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
    assert!(!tc_paths.is_empty(), "firewall fixture corpus must not be empty");

    let mut seen_green = false;
    let mut seen_red = false;

    for path in &tc_paths {
        let fixture = load_json(path);
        let label = path.file_name().unwrap().to_string_lossy().to_string();

        let committed = Baseline::from_value(&fixture["committed_baseline"]);
        let proposed = Baseline::from_value(&fixture["proposed_baseline"]);
        let signoff = SignOff::from_value(&fixture["signoff"]);
        let current = current_from_value(&fixture["current"]);

        let report = evaluate_firewall(&committed, &proposed, &current, &signoff);

        let expected_growth = fixture["expected_ratchet_growth"].as_u64().unwrap_or(0) as usize;
        assert_eq!(
            report.ratchet_growth.len(),
            expected_growth,
            "{label}: ratchet_growth count mismatch (growth = {:?})",
            report.ratchet_growth
        );

        let expected_failing: BTreeSet<String> = fixture["expected_failing_codes"]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
            .unwrap_or_default();
        let actual_failing: BTreeSet<String> = report
            .codes
            .iter()
            .filter(|r| r.fails())
            .map(|r| r.code.clone())
            .collect();
        assert_eq!(actual_failing, expected_failing, "{label}: failing-code set mismatch");

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

/// THE GO-LIVE PROOF: with the baseline frozen at today, the firewall is GREEN on the live
/// corpus. The committed baseline == the regenerated (proposed) baseline (registry-drift
/// also enforces this byte-exact), so:
///   - compare-mode: current == baseline => zero regressions => no baseline-block-on-new
///     code fails (advisory codes report their counts but never fail);
///   - ratchet-invariant: proposed == committed => zero growth => no ratchet_regression.
#[test]
fn firewall_is_green_on_the_live_corpus_with_the_baseline() {
    let root = repo_root();

    // The committed baseline (byte-diff-protected by registry-drift).
    let committed_value = load_json(&faces_dir(&root).join("gate-baseline.generated.json"));
    let committed = Baseline::from_value(&committed_value);

    // The proposed baseline = what TODAY's corpus would freeze.
    let proposed_value = regenerate_baseline(&root);
    let proposed = Baseline::from_value(&proposed_value);

    // The sign-off door (the one-way exemption; empty = ratchet fully closed).
    let signoff = SignOff::from_value(&load_json(&signoff_path(&root)));

    // The live "current" keyed violations == the proposed baseline's keys (the producer
    // captured them via evaluate_keyed over the live faces).
    let current = baseline_keys_map(&proposed);

    let report = evaluate_firewall(&committed, &proposed, &current, &signoff);

    // Evidence digest: per-code current/baseline/regressions/fixed/tolerated.
    eprintln!("FIREWALL GO-LIVE report (live corpus, baseline frozen at today):");
    for r in &report.codes {
        eprintln!(
            "  [{}] {:48} mode={:22} current={:6} baseline={:6} regressions={:4} fixed={:4} tolerated={:6}{}",
            r.gate,
            r.code,
            r.mode,
            r.current,
            r.baseline,
            r.regressions.len(),
            r.fixed.len(),
            r.tolerated.len(),
            if r.fails() { "  <-- FAIL" } else { "" }
        );
    }
    eprintln!(
        "  ratchet_growth (un-signed-off baseline additions): {}",
        report.ratchet_growth.len()
    );

    let failing: Vec<&str> = report
        .codes
        .iter()
        .filter(|r| r.fails())
        .map(|r| r.code.as_str())
        .collect();
    assert!(
        failing.is_empty(),
        "GO-LIVE: firewall must be GREEN on today's corpus (no NEW debt), but these codes FAIL: {failing:?}"
    );
    assert!(
        report.ratchet_growth.is_empty(),
        "GO-LIVE: committed baseline == regenerated, so the ratchet must show zero growth, got {:?}",
        report.ratchet_growth
    );
    assert!(report.is_green(), "firewall must be GREEN with the baseline frozen at today");

    // Sanity: the baseline is NON-trivial (the frozen pre-existing corpus debt is real).
    let total_baselined: usize = report.codes.iter().map(|r| r.baseline).sum();
    assert!(
        total_baselined > 0,
        "the baseline must freeze the real pre-existing corpus debt"
    );
}

/// RED-on-NEW proof against the LIVE corpus: inject ONE synthetic NEW key into the live
/// "current" set for a baseline-block-on-new code and assert the firewall FAILS — proving
/// the gate still blocks any new finite violation that is not in the frozen baseline.
#[test]
fn firewall_goes_red_on_a_synthetic_new_violation() {
    let root = repo_root();
    let committed_value = load_json(&faces_dir(&root).join("gate-baseline.generated.json"));
    let committed = Baseline::from_value(&committed_value);
    let proposed = Baseline::from_value(&regenerate_baseline(&root));
    let signoff = SignOff::from_value(&load_json(&signoff_path(&root)));

    let mut current = baseline_keys_map(&proposed);
    // Add a NEW unjustified path that is NOT in the committed baseline.
    current
        .entry("cloud-ci-total-accounting".to_owned())
        .or_default()
        .entry("unjustified".to_owned())
        .or_default()
        .insert("SYNTHETIC/new-unjustified-file.rs".to_owned());

    let report = evaluate_firewall(&committed, &proposed, &current, &signoff);
    let unjust = report
        .codes
        .iter()
        .find(|r| r.gate == "cloud-ci-total-accounting" && r.code == "unjustified")
        .expect("unjustified code present");
    assert!(
        unjust.regressions.contains("SYNTHETIC/new-unjustified-file.rs"),
        "the synthetic NEW file must show up as a regression"
    );
    assert!(unjust.fails(), "a NEW unjustified file must FAIL the firewall");
    assert!(!report.is_green(), "firewall must be RED on a NEW finite violation");
}

/// RATCHET proof against the LIVE corpus: a regen that GROWS the baseline (without sign-off)
/// is a ratchet_regression — debt cannot be laundered into the baseline by re-running the
/// producer.
#[test]
fn firewall_blocks_baseline_growth_without_signoff() {
    let root = repo_root();
    let committed = Baseline::from_value(&load_json(
        &faces_dir(&root).join("gate-baseline.generated.json"),
    ));
    // A proposed baseline that ADDS a key beyond the committed set.
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
    let report = evaluate_firewall(&committed, &proposed, &current, &SignOff::default());
    assert!(
        report
            .ratchet_growth
            .iter()
            .any(|(_, code, key)| code == "unjustified" && key == "SYNTHETIC/laundered-debt.rs"),
        "growing the baseline without sign-off must be a ratchet_regression"
    );
    assert!(!report.is_green());
}
