//! Live-corpus + fixture gate for the facade->core layering rule.
//!
//! The live leg is the one that matters: it runs the COMMITTED policy against the real tree and
//! asserts born-blocking green. A fixture-only suite would pass while the real scan enumerated
//! nothing, which is why `the_live_scan_is_not_empty` is a separate assertion rather than an
//! implicit consequence of "no findings".

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use ci_facade_core_layering::{CODE_DIRECT_DEP, CODE_NO_PORTS, collect, evaluate_keyed};
use serde_json::Value;

/// Walk up to the repo root. `env!("CARGO_MANIFEST_DIR")` is deliberately NOT used — it does not
/// exist under buck2, which is the binding build here (same approach as the sibling gates).
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

const POLICY_REL: &str = "ci/facade/facade-core-layering/facade-core-layering-policy.json";

/// Loads the COMMITTED policy from the candidate tree — not an inlined copy — so the live legs
/// assert against exactly what ships.
fn policy() -> Value {
    let p = repo_root().join(POLICY_REL);
    let text = fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", p.display()))
}

#[test]
fn the_live_tree_is_green_against_the_frozen_baseline() {
    let observed = collect(&repo_root(), &policy()).expect("scan");
    let findings = evaluate_keyed(&policy(), &observed);
    assert!(
        findings.is_empty(),
        "live facade->core layering findings (baseline is shrink-only — repair the edge, do not \
         add a row):\n{findings:#?}"
    );
}

#[test]
fn the_live_scan_is_not_empty() {
    let observed = collect(&repo_root(), &policy()).expect("scan");
    let caps = observed["capabilities_scanned"].as_u64().unwrap_or(0);
    let pkgs = observed["facade_packages_scanned"].as_u64().unwrap_or(0);
    assert!(
        caps >= 10 && pkgs >= 50,
        "scan enumerated {caps} capabilities / {pkgs} facade packages — a shrinking scan set is \
         how this gate silently stops enforcing"
    );
}

#[test]
fn the_baseline_matches_the_live_violation_set_exactly() {
    // Set EQUALITY, not containment. Containment would let the baseline carry phantom rows that
    // pre-authorize a future violation under a name nobody is watching.
    let observed = collect(&repo_root(), &policy()).expect("scan");
    let pol = policy();
    for code in [CODE_DIRECT_DEP, CODE_NO_PORTS] {
        let live: BTreeSet<&str> = observed["violations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["code"].as_str() == Some(code))
            .filter_map(|r| r["key"].as_str())
            .collect();
        let baseline: BTreeSet<&str> = pol["frozen_baseline"][code]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert_eq!(live, baseline, "code {code}: live set != frozen baseline");
    }
}

#[test]
fn red_fixture_a_new_violator_fails_closed() {
    let dir = std::env::temp_dir().join("facade-core-layering-red-fixture");
    let _ = fs::remove_dir_all(&dir);
    let pkg = dir.join("iam/facade/brand-new-service");
    fs::create_dir_all(&pkg).unwrap();
    fs::create_dir_all(dir.join("iam/ports/policy-api")).unwrap();
    fs::write(
        pkg.join("BUCK"),
        "rust_library(name = \"x\", deps = [\"//iam/core/policy-kernel:policy-kernel\"])",
    )
    .unwrap();
    fs::write(
        pkg.join("Cargo.toml"),
        "[package]\nname = \"iam-brand-new-service\"\n",
    )
    .unwrap();

    let observed = collect(&dir, &policy()).expect("scan");
    let findings = evaluate_keyed(&policy(), &observed);
    let keys: BTreeSet<&str> = findings.iter().map(|f| f.key.as_str()).collect();
    assert!(
        keys.contains("iam-brand-new-service"),
        "a NEW facade->core violator must be born-blocking; got {findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|f| f.key == "iam-brand-new-service" && f.code == CODE_DIRECT_DEP),
        "iam has a ports layer in this fixture, so the violation is a direct-dep, not no-ports"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn green_fixture_reaching_core_through_ports_is_allowed() {
    let dir = std::env::temp_dir().join("facade-core-layering-green-fixture");
    let _ = fs::remove_dir_all(&dir);
    let pkg = dir.join("iam/facade/well-layered-service");
    fs::create_dir_all(&pkg).unwrap();
    fs::create_dir_all(dir.join("iam/ports/policy-api")).unwrap();
    // Depends on ports only. ports itself depending on core is the sanctioned path.
    fs::write(
        pkg.join("BUCK"),
        "rust_library(name = \"x\", deps = [\"//iam/ports/policy-api:policy-api\"])",
    )
    .unwrap();
    fs::write(
        pkg.join("Cargo.toml"),
        "[package]\nname = \"iam-well-layered-service\"\n",
    )
    .unwrap();

    let observed = collect(&dir, &policy()).expect("scan");
    assert_eq!(
        observed["violations"].as_array().unwrap().len(),
        0,
        "facade -> ports is the sanctioned path and must not be flagged: {observed:#?}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_capability_without_ports_is_classified_under_the_adr_sanctioned_code() {
    // ADR-0562 §10.6: where no ports layer exists, the facade->core edge is dependency-legal.
    // It gets its OWN code so that introducing a ports layer closes that code cleanly instead of
    // reading as a regression against the other one.
    let dir = std::env::temp_dir().join("facade-core-layering-noports-fixture");
    let _ = fs::remove_dir_all(&dir);
    let pkg = dir.join("compute/facade/vm-api");
    fs::create_dir_all(&pkg).unwrap();
    fs::write(
        pkg.join("BUCK"),
        "rust_library(name = \"x\", deps = [\"//compute/core/vm-kernel:vm-kernel\"])",
    )
    .unwrap();
    fs::write(
        pkg.join("Cargo.toml"),
        "[package]\nname = \"compute-vm-api\"\n",
    )
    .unwrap();

    let observed = collect(&dir, &policy()).expect("scan");
    let rows = observed["violations"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{observed:#?}");
    assert_eq!(rows[0]["code"].as_str(), Some(CODE_NO_PORTS));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn declared_codes_and_policy_codes_agree() {
    let pol = policy();
    let declared: BTreeSet<&str> = pol["codes"]
        .as_object()
        .expect("codes object")
        .keys()
        .map(String::as_str)
        .collect();
    let emitted: BTreeSet<&str> = [CODE_DIRECT_DEP, CODE_NO_PORTS].into_iter().collect();
    assert_eq!(
        declared, emitted,
        "a code emitted by the engine but not declared in policy data (or vice versa) is a \
         silent enforcement gap"
    );
    for code in emitted {
        assert!(
            pol["frozen_baseline"].get(code).is_some(),
            "code {code} has no frozen_baseline array"
        );
    }
}
