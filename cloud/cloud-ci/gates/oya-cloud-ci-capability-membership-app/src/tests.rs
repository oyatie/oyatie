//! Pure-unit RED/GREEN fixtures for the capability-membership evaluator (no filesystem). The
//! live-corpus self-test + on-disk fixtures live in tests/capability_membership.rs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use serde_json::json;

fn policy() -> Value {
    json!({
        "gate_id": GATE_ID,
        "registry_path": "specs/capability-registry.json",
        "scan_roots": ["cloud", "oya", "libs", "tools"],
        "meta_directories": ["kernel", "os", "base", "governance", "build", "app"],
        "allowed_top_level_dirs": [
            "kernel", "os", "base", "governance", "build", "app",
            "cloud", "oya", "libs", "tools", "specs", "docs"
        ],
        "min_expected_crates": 1
    })
}

/// A minimal registry exercising one capability, one app product, one meta absorb, one glob, and a
/// frozen baseline entry.
fn registry() -> Value {
    json!({
        "capabilities": [
            { "name": "iam", "absorbs_current_dirs": ["cloud/cloud-iam", "oya/identity"] },
            { "name": "data", "absorbs_current_dirs": ["cloud/cloud-data"] }
        ],
        "membership_lint_coverage": {
            "app_products": { "meta_dir": "app/", "current_dirs": ["oya/crm"] },
            "meta_directory_absorbs": [
                { "meta_dir": "kernel/", "current_dirs": ["cloud/cloud-kernel"] }
            ],
            "absorbs_current_crate_globs": [
                { "meta_dir": "governance/", "globs": ["libs/oya-check-*"] },
                { "capability": "data", "globs": ["libs/oya-data-*"] }
            ],
            "frozen_unmapped_baseline": {
                "burn_down_target": 0,
                "crates": ["libs/oya-shared-idempotency-key-kernel"]
            }
        }
    })
}

fn observed(crates: Vec<&str>, top_level: Vec<&str>) -> Value {
    json!({
        "crate_count": crates.len(),
        "crates": crates,
        "top_level_dirs": top_level,
        "registry": registry(),
    })
}

fn codes(findings: &BTreeSet<Finding>) -> BTreeSet<String> {
    findings.iter().map(|f| f.code.clone()).collect()
}

#[test]
fn every_crate_mapped_passes() {
    let obs = observed(
        vec![
            "cloud/cloud-iam/crates/oya-cloud-iam-kernel",
            "oya/identity/crates/oya-identity-kernel",
            "oya/crm/crates/oya-crm-kernel",
            "cloud/cloud-kernel/crates/oya-cloud-kernel-app",
            "libs/oya-check-cohesion",
            "libs/oya-data-sql-kernel",
            "libs/oya-shared-idempotency-key-kernel",
        ],
        vec!["cloud", "oya", "libs", "tools", "specs", "docs"],
    );
    let findings = evaluate_keyed(&policy(), &obs);
    assert!(findings.is_empty(), "all-mapped corpus must pass: {findings:#?}");
    let report = evaluate(&policy(), &obs);
    assert_eq!(report.verdict, Verdict::Green);
    assert_eq!(report.mapped_to_home, 6);
    assert_eq!(report.frozen_unmapped, 1);
}

// --- The three mandated RED fixtures ---

#[test]
fn red_crate_in_no_capability_unmapped_new_fails() {
    // A NEW crate under a scan root that maps to no capability and is NOT in the frozen baseline.
    let obs = observed(
        vec!["oya/widget/crates/oya-widget-kernel"],
        vec!["cloud", "oya", "libs", "tools", "specs", "docs"],
    );
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("MEM-NEW-UNMAPPED-CRATE"), "{c:?}");
    assert_eq!(evaluate(&policy(), &obs).verdict, Verdict::Red);
}

#[test]
fn red_new_top_level_dir_common_fails() {
    // A NEW top-level dir `common/` (the canonical junk-drawer) outside the closed set.
    let obs = observed(
        vec!["cloud/cloud-iam/crates/oya-cloud-iam-kernel"],
        vec!["cloud", "oya", "libs", "tools", "specs", "docs", "common"],
    );
    let findings = evaluate_keyed(&policy(), &obs);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "MEM-NEW-TOP-LEVEL-DIR" && f.key == "common"),
        "a NEW top-level dir common/ must fail: {findings:#?}"
    );
    assert_eq!(evaluate(&policy(), &obs).verdict, Verdict::Red);
}

#[test]
fn red_crate_in_two_capabilities_fails() {
    // A crate that two absorbs entries claim → double-mapped. We craft a registry where two
    // capabilities both absorb an overlapping dir prefix.
    let mut reg = registry();
    reg["capabilities"][1]["absorbs_current_dirs"] = json!(["cloud/cloud-data", "cloud/cloud-iam"]);
    let obs = json!({
        "crate_count": 1,
        "crates": ["cloud/cloud-iam/crates/oya-cloud-iam-kernel"],
        "top_level_dirs": ["cloud", "oya", "libs", "tools", "specs", "docs"],
        "registry": reg,
    });
    let findings = evaluate_keyed(&policy(), &obs);
    assert!(
        findings.iter().any(|f| f.code == "MEM-DOUBLE-MAPPED-CRATE"),
        "a crate claimed by two capabilities must fail: {findings:#?}"
    );
    assert_eq!(evaluate(&policy(), &obs).verdict, Verdict::Red);
}

// --- frozen-baseline advisory + drift ---

#[test]
fn frozen_baseline_crate_is_advisory_not_a_failure() {
    let obs = observed(
        vec!["libs/oya-shared-idempotency-key-kernel"],
        vec!["cloud", "oya", "libs", "tools", "specs", "docs"],
    );
    let findings = evaluate_keyed(&policy(), &obs);
    assert!(
        findings.is_empty(),
        "a frozen-baseline crate is advisory (no regression), not a failure: {findings:#?}"
    );
    assert_eq!(evaluate(&policy(), &obs).frozen_unmapped, 1);
}

#[test]
fn stale_frozen_entry_now_mapped_fails() {
    // The frozen entry is now also matched by a glob → it must be removed from the baseline.
    let mut reg = registry();
    reg["membership_lint_coverage"]["absorbs_current_crate_globs"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "capability": "iam", "globs": ["libs/oya-shared-idempotency-*"] }));
    let obs = json!({
        "crate_count": 1,
        "crates": ["libs/oya-shared-idempotency-key-kernel"],
        "top_level_dirs": ["cloud", "oya", "libs", "tools", "specs", "docs"],
        "registry": reg,
    });
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("MEM-STALE-FROZEN-BASELINE"), "{c:?}");
}

#[test]
fn stale_frozen_entry_not_in_tree_fails() {
    // The frozen entry no longer exists in the tree → drift.
    let obs = observed(
        vec!["cloud/cloud-iam/crates/oya-cloud-iam-kernel"],
        vec!["cloud", "oya", "libs", "tools", "specs", "docs"],
    );
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("MEM-STALE-FROZEN-BASELINE"), "{c:?}");
}

// --- base/-admission rule ---

#[test]
fn base_crate_with_fewer_than_three_consumers_fails() {
    let obs = json!({
        "crate_count": 1,
        "crates": ["base/oya-base-clock"],
        "top_level_dirs": ["base", "cloud", "oya", "libs", "tools", "specs", "docs"],
        "registry": registry(),
        "base_admission_facts": {
            "base/oya-base-clock": { "capability_consumers": ["iam", "data"], "strictly_below_all_consumers": true }
        }
    });
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("MEM-BASE-ADMISSION-CONSUMERS"), "{c:?}");
}

#[test]
fn base_crate_not_strictly_below_consumers_fails() {
    let obs = json!({
        "crate_count": 1,
        "crates": ["base/oya-base-clock"],
        "top_level_dirs": ["base", "cloud", "oya", "libs", "tools", "specs", "docs"],
        "registry": registry(),
        "base_admission_facts": {
            "base/oya-base-clock": { "capability_consumers": ["iam", "data", "cell"], "strictly_below_all_consumers": false }
        }
    });
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("MEM-BASE-ADMISSION-DAG"), "{c:?}");
}

#[test]
fn base_crate_with_three_consumers_below_all_passes() {
    let obs = json!({
        "crate_count": 1,
        "crates": ["base/oya-base-clock"],
        "top_level_dirs": ["base", "cloud", "oya", "libs", "tools", "specs", "docs"],
        "registry": registry(),
        "base_admission_facts": {
            "base/oya-base-clock": { "capability_consumers": ["iam", "data", "cell"], "strictly_below_all_consumers": true }
        }
    });
    // The base/ crate itself maps to the base/ meta dir via the closed top-level set; it is not in
    // the absorbs registry, so it would be MEM-NEW-UNMAPPED unless registered. Here we only assert
    // the base-admission checks pass — strip the membership finding by registering base/ as a glob.
    let findings = evaluate_keyed(&policy(), &obs);
    assert!(
        !findings.iter().any(|f| f.code == "MEM-BASE-ADMISSION-CONSUMERS"
            || f.code == "MEM-BASE-ADMISSION-DAG"),
        "an admitted base/ crate must pass the admission checks: {findings:#?}"
    );
}

// --- empty-scan + fail-closed ---

#[test]
fn empty_scan_below_floor_fails() {
    let obs = observed(vec![], vec!["cloud", "oya", "libs", "tools", "specs", "docs"]);
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("MEM-EMPTY-SCAN"), "{c:?}");
}

#[test]
fn wrong_gate_id_fails_closed() {
    let mut p = policy();
    p["gate_id"] = json!("not-the-gate");
    let obs = observed(
        vec!["cloud/cloud-iam/crates/oya-cloud-iam-kernel", "libs/oya-shared-idempotency-key-kernel"],
        vec!["cloud", "oya", "libs", "tools", "specs", "docs"],
    );
    let c = codes(&evaluate_keyed(&p, &obs));
    assert!(c.contains("MEM-POLICY-GATE-ID-MISMATCH"), "{c:?}");
}

#[test]
fn malformed_registry_fails_closed() {
    let obs = json!({
        "crate_count": 1,
        "crates": ["cloud/cloud-iam/crates/x"],
        "top_level_dirs": ["cloud"],
        "registry": json!({ "capabilities": "not-an-array" }),
    });
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("MEM-POLICY-MALFORMED"), "{c:?}");
}

#[test]
fn registry_without_coverage_block_fails_closed() {
    let obs = json!({
        "crate_count": 1,
        "crates": ["cloud/cloud-iam/crates/x"],
        "top_level_dirs": ["cloud"],
        "registry": json!({ "capabilities": [{ "name": "iam", "absorbs_current_dirs": ["cloud/cloud-iam"] }] }),
    });
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("MEM-POLICY-MALFORMED"), "{c:?}");
}
