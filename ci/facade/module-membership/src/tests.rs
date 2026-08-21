//! Pure-unit RED/GREEN fixtures for the capability-membership evaluator (no filesystem). The
//! live-corpus self-test + on-disk fixtures live in tests/capability_membership.rs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use serde_json::json;

fn policy() -> Value {
    json!({
        "gate_id": GATE_ID,
        "registry_path": "governance/capability-registry.json",
        "scan_roots": ["cloud", "oya", "libs", "tools"],
        "meta_directories": ["kernel", "os", "base", "governance", "build", "app"],
        "allowed_top_level_dirs": [
            "kernel", "os", "base", "governance", "build", "app",
            "cloud", "oya", "libs", "tools", "specs", "docs", "flags"
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
    assert!(
        findings.is_empty(),
        "all-mapped corpus must pass: {findings:#?}"
    );
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
fn retired_v1_products_map_in_place_not_to_app() {
    let mut reg = registry();
    reg["membership_lint_coverage"]["retired_v1_products"] = json!({
        "disposition": "retire-in-place; do not absorb into app/; D41/D42",
        "current_dirs": ["oya/notes"]
    });
    let mapping = parse_mapping(&reg).expect("retired_v1_products is a recognized coverage key");
    let homes = homes_for(&mapping, "oya/notes/crates/oya-notes-domain");
    assert_eq!(homes, vec![RETIRE_IN_PLACE_HOME.to_owned()]);
    assert!(
        !homes.iter().any(|home| home.contains("app")),
        "D41/D42 retired dirs must not home under app/: {homes:?}"
    );

    let obs = json!({
        "crate_count": 2,
        "crates": [
            "oya/notes/crates/oya-notes-domain",
            "libs/oya-shared-idempotency-key-kernel"
        ],
        "top_level_dirs": ["cloud", "oya", "libs", "tools", "specs", "docs"],
        "registry": reg,
    });
    let findings = evaluate_keyed(&policy(), &obs);
    assert!(
        findings.is_empty(),
        "a crate under retired_v1_products must stay mapped (not MEM-NEW-UNMAPPED): {findings:#?}"
    );
}

#[test]
fn retired_v1_products_meta_dir_app_is_malformed() {
    let mut reg = registry();
    reg["membership_lint_coverage"]["retired_v1_products"] = json!({
        "meta_dir": "app/",
        "current_dirs": ["oya/notes"]
    });
    let err = match parse_mapping(&reg) {
        Ok(_) => panic!("meta_dir: app/ must fail closed"),
        Err(message) => message,
    };
    assert!(
        err.contains("retired_v1_products.meta_dir"),
        "malformed meta_dir must name the field: {err}"
    );
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
fn empty_scan_fails_closed() {
    let obs = observed(
        vec![],
        vec!["cloud", "oya", "libs", "tools", "specs", "docs"],
    );
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("MEM-EMPTY-SCAN"), "{c:?}");
}

#[test]
fn wrong_gate_id_fails_closed() {
    let mut p = policy();
    p["gate_id"] = json!("not-the-gate");
    let obs = observed(
        vec![
            "cloud/cloud-iam/crates/oya-cloud-iam-kernel",
            "libs/oya-shared-idempotency-key-kernel",
        ],
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

// ---------------------------------------------------------------------------
// STOP ACCRUAL — the legacy-root freeze (pure fixtures).
// ---------------------------------------------------------------------------

/// The base policy plus a freeze over `cloud`/`oya` with a two-entry census.
fn freeze_policy(census: Vec<&str>) -> Value {
    let mut p = policy();
    p["legacy_root_freeze"] = json!({
        "frozen_roots": ["cloud", "oya"],
        "crates": census,
    });
    p
}

#[test]
fn frozen_census_tolerates_every_crate_that_exists_today() {
    let p = freeze_policy(vec![
        "cloud/cloud-iam/crates/oya-cloud-iam-kernel",
        "oya/identity/crates/oya-identity-kernel",
    ]);
    let obs = observed(
        vec![
            "cloud/cloud-iam/crates/oya-cloud-iam-kernel",
            "oya/identity/crates/oya-identity-kernel",
            // The shared registry fixture's own unmapped-baseline entry; present so this test
            // isolates the freeze rather than tripping MEM-STALE-FROZEN-BASELINE.
            "libs/oya-shared-idempotency-key-kernel",
        ],
        vec!["cloud", "oya", "libs"],
    );
    let findings = evaluate_keyed(&p, &obs);
    assert!(findings.is_empty(), "{findings:#?}");
    assert_eq!(evaluate(&p, &obs).legacy_root_crates, 2);
}

#[test]
fn a_crate_born_under_a_frozen_root_is_a_regression() {
    // THE WHOLE POINT: the new crate maps cleanly to a registered capability, so membership alone
    // is green. Only the freeze catches it.
    let p = freeze_policy(vec!["cloud/cloud-iam/crates/oya-cloud-iam-kernel"]);
    let obs = observed(
        vec![
            "cloud/cloud-iam/crates/oya-cloud-iam-kernel",
            "cloud/cloud-iam/crates/oya-cloud-iam-brand-new",
        ],
        vec!["cloud"],
    );
    let findings = evaluate_keyed(&p, &obs);
    let hit = findings
        .iter()
        .find(|f| f.code == "MEM-NEW-LEGACY-ROOT-CRATE")
        .unwrap_or_else(|| panic!("expected a freeze finding: {findings:#?}"));
    assert_eq!(hit.key, "cloud/cloud-iam/crates/oya-cloud-iam-brand-new");
    // The remedy must name where the crate SHOULD go, not merely that it is unwelcome.
    assert!(hit.detail.contains("capability root"), "{}", hit.detail);
    assert_eq!(evaluate(&p, &obs).verdict, Verdict::Red);
}

#[test]
fn a_crate_born_outside_every_frozen_root_is_not_a_freeze_finding() {
    // The freeze must not become a repo-wide "no new crates" rule: a crate in a capability root is
    // exactly the destination the reorg wants, and must land freely.
    let p = freeze_policy(vec!["cloud/cloud-iam/crates/oya-cloud-iam-kernel"]);
    let obs = observed(
        vec![
            "cloud/cloud-iam/crates/oya-cloud-iam-kernel",
            "iam/core/pdp-kernel",
        ],
        vec!["cloud", "iam"],
    );
    let c = codes(&evaluate_keyed(&p, &obs));
    assert!(!c.contains("MEM-NEW-LEGACY-ROOT-CRATE"), "{c:?}");
    assert_eq!(evaluate(&p, &obs).legacy_root_crates, 1);
}

#[test]
fn a_moved_crate_must_shrink_the_census_in_the_same_change() {
    // Burn-down that is not recorded leaves slack behind: a crate later re-created at that exact
    // path would land pre-forgiven. The move is only complete when the census entry is gone.
    let p = freeze_policy(vec![
        "cloud/cloud-iam/crates/oya-cloud-iam-kernel",
        "oya/identity/crates/oya-identity-kernel",
    ]);
    let obs = observed(
        vec!["cloud/cloud-iam/crates/oya-cloud-iam-kernel"],
        vec!["cloud"],
    );
    let findings = evaluate_keyed(&p, &obs);
    let hit = findings
        .iter()
        .find(|f| f.code == "MEM-STALE-LEGACY-ROOT-BASELINE")
        .unwrap_or_else(|| panic!("expected a stale-census finding: {findings:#?}"));
    assert_eq!(hit.key, "oya/identity/crates/oya-identity-kernel");
    assert_eq!(evaluate(&p, &obs).verdict, Verdict::Red);
}

#[test]
fn a_policy_without_the_freeze_block_is_inert() {
    // Fixture / adopting-repo contract: no block ⇒ no freeze findings at all, in either direction.
    // The live gate's non-inertness is asserted separately over the COMMITTED policy.
    let obs = observed(
        vec!["cloud/cloud-iam/crates/oya-cloud-iam-brand-new"],
        vec!["cloud"],
    );
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(!c.contains("MEM-NEW-LEGACY-ROOT-CRATE"), "{c:?}");
    assert!(!c.contains("MEM-STALE-LEGACY-ROOT-BASELINE"), "{c:?}");
    assert_eq!(evaluate(&policy(), &obs).legacy_root_crates, 0);
}

#[test]
fn every_freeze_code_is_registered() {
    let p = freeze_policy(vec!["oya/identity/crates/gone"]);
    let obs = observed(
        vec!["cloud/cloud-iam/crates/oya-cloud-iam-brand-new"],
        vec!["cloud"],
    );
    let findings = evaluate_keyed(&p, &obs);
    for f in &findings {
        assert!(
            VIOLATION_CODES.contains(&f.code.as_str()),
            "unregistered {}",
            f.code
        );
    }
    let c = codes(&findings);
    assert!(c.contains("MEM-NEW-LEGACY-ROOT-CRATE"), "{c:?}");
    assert!(c.contains("MEM-STALE-LEGACY-ROOT-BASELINE"), "{c:?}");
}

fn layout_policy() -> Value {
    let mut p = policy();
    p["capability_root_allowed_children"] = json!([
        "core",
        "ports",
        "adapters",
        "facade",
        "contracts",
        "cedar",
        "policy",
        "observability",
        "iac",
        "OWNERS",
        "README.md",
        "PRD.md",
        "BUCK",
        "catalog.yaml"
    ]);
    p
}

fn observed_layout(children: Value, protected: Option<Value>) -> Value {
    let mut obs = observed(
        vec![
            "cloud/cloud-iam/crates/oya-cloud-iam-kernel",
            "libs/oya-shared-idempotency-key-kernel",
        ],
        vec!["cloud", "flags", "app", "libs"],
    );
    obs["capability_root_children"] = children;
    if let Some(protected) = protected {
        obs["protected_capability_root_children"] = protected;
    }
    obs
}

#[test]
fn grandfathered_flags_ips_is_green() {
    let obs = observed_layout(
        json!({"flags": ["core", "IPs"]}),
        Some(json!({"flags": ["IPs"]})),
    );
    let findings = evaluate_keyed(&layout_policy(), &obs);
    assert!(
        !findings
            .iter()
            .any(|f| f.code == "MEM-NEW-CAPABILITY-ROOT-CHILD"),
        "existing IPs must stay grandfathered: {findings:#?}"
    );
    assert_eq!(
        evaluate(&layout_policy(), &obs).verdict,
        Verdict::Green,
        "grandfathered IPs must be green: {findings:#?}"
    );
}

#[test]
fn new_synthetic_flags_ips2_is_red() {
    let obs = observed_layout(json!({"flags": ["IPs2"]}), Some(json!({"flags": ["IPs"]})));
    let findings = evaluate_keyed(&layout_policy(), &obs);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "MEM-NEW-CAPABILITY-ROOT-CHILD" && f.key == "flags/IPs2"),
        "a NEW flags/IPs2 child must fail: {findings:#?}"
    );
    assert_eq!(evaluate(&layout_policy(), &obs).verdict, Verdict::Red);
}

#[test]
fn grandfathered_app_product_child_is_green() {
    let obs = observed_layout(
        json!({"app/calendar": ["OWNERS", "IPs"]}),
        Some(json!({"app/calendar": ["IPs"]})),
    );
    let findings = evaluate_keyed(&layout_policy(), &obs);
    assert!(
        !findings
            .iter()
            .any(|f| f.code == "MEM-NEW-CAPABILITY-ROOT-CHILD"),
        "existing app/<product>/ extras must stay grandfathered: {findings:#?}"
    );
}

#[test]
fn new_app_product_child_is_red() {
    let obs = observed_layout(
        json!({"app/calendar": ["IPs2"]}),
        Some(json!({"app/calendar": ["IPs"]})),
    );
    let findings = evaluate_keyed(&layout_policy(), &obs);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "MEM-NEW-CAPABILITY-ROOT-CHILD" && f.key == "app/calendar/IPs2"),
        "a NEW app/<product>/ child must fail: {findings:#?}"
    );
}

#[test]
fn live_tree_extras_are_advisory_without_protected_children() {
    let obs = observed_layout(
        json!({"flags": ["IPs", "scorecards", "AUDIT-FINDINGS-2026-05-18.json"]}),
        None,
    );
    let findings = evaluate_keyed(&layout_policy(), &obs);
    assert!(
        !findings
            .iter()
            .any(|f| f.code == "MEM-NEW-CAPABILITY-ROOT-CHILD"),
        "without protected children the live extras stay advisory: {findings:#?}"
    );
    assert_eq!(evaluate(&layout_policy(), &obs).verdict, Verdict::Green);
}
