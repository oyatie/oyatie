//! Pure-unit RED/GREEN fixtures for the tier-field-coverage evaluator (no filesystem). The
//! live-corpus self-test + on-disk fixtures live in tests/tier_field_coverage.rs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use serde_json::json;

fn policy() -> Value {
    json!({
        "gate_id": GATE_ID,
        "governed_service_roots": ["cloud", "oya"],
        "tier_enum": ["substrate", "product", "service-cell", "reserved"],
        "tier_subtype_enum": ["substrate-identity", "substrate-infra", "substrate-api-gateway", "product-consumer", "product-developer-sdk"],
        "dr_tier_enum": ["T0", "T1", "T2", "T3"],
        "substrate_dag_stratum_enum": ["S0", "S1", "S2", "S3", "S4", "S5", "forward-declared"],
        "de_overload_denied_tier_values": ["T0", "T1", "T2", "T3", "saas", "surface", "external-facing"],
        "substrate_requires_dag_position": true,
        "min_expected_service_manifests": 1,
        "require_sharding_automation": false,
        "require_openslo_manifest_refs": false
    })
}

fn observed(manifests: Vec<(&str, Value)>) -> Value {
    let arr: Vec<Value> = manifests
        .into_iter()
        .map(|(path, m)| json!({ "path": path, "manifest": m }))
        .collect();
    json!({ "manifest_count": arr.len(), "manifests": arr })
}

fn codes(findings: &BTreeSet<Finding>) -> BTreeSet<String> {
    findings.iter().map(|f| f.code.clone()).collect()
}

#[test]
fn green_substrate_manifest_passes() {
    let m = json!({
        "microservice": "cloud-iam",
        "tier": "substrate",
        "tier_subtype": "substrate-identity",
        "dr_tier": "T1",
        "substrate_dag_position": { "stratum": "S1", "depends_on": ["cell"], "consumed_by_substrates": [] }
    });
    let obs = observed(vec![("cloud/cloud-iam/manifest.json", m)]);
    let findings = evaluate_keyed(&policy(), &obs);
    assert!(
        findings.is_empty(),
        "green substrate must pass: {findings:?}"
    );
    assert_eq!(evaluate(&policy(), &obs).verdict, Verdict::Green);
}

#[test]
fn green_product_manifest_passes() {
    let m = json!({
        "microservice": "crm",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T3"
    });
    let obs = observed(vec![("oya/crm/manifest.json", m)]);
    assert!(evaluate_keyed(&policy(), &obs).is_empty());
}

#[test]
fn missing_tier_fails() {
    let m = json!({ "microservice": "x", "tier_subtype": "product-consumer", "dr_tier": "T2" });
    let obs = observed(vec![("oya/x/manifest.json", m)]);
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("TFC-MISSING-TIER"), "{c:?}");
}

#[test]
fn missing_tier_subtype_and_dr_tier_fail() {
    let m = json!({ "microservice": "x", "tier": "product" });
    let obs = observed(vec![("oya/x/manifest.json", m)]);
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("TFC-MISSING-TIER-SUBTYPE"), "{c:?}");
    assert!(c.contains("TFC-MISSING-DR-TIER"), "{c:?}");
}

#[test]
fn tier_overloaded_with_dr_value_fails() {
    // The V3 de-overload guard: `tier: T1` is a DR/reliability value, not a dependency class.
    let m = json!({ "microservice": "x", "tier": "T1", "tier_subtype": "product-consumer", "dr_tier": "T1" });
    let obs = observed(vec![("oya/x/manifest.json", m)]);
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("TFC-TIER-TYPE-OVERLOAD"), "{c:?}");
    // It must NOT be reported as merely not-in-enum — the overload class is the precise diagnosis.
    assert!(
        !c.contains("TFC-TIER-NOT-IN-ENUM"),
        "overload should pre-empt not-in-enum: {c:?}"
    );
}

#[test]
fn tier_overloaded_with_deployment_mode_fails() {
    let m = json!({ "microservice": "x", "tier": "saas", "tier_subtype": "product-consumer", "dr_tier": "T2" });
    let c = codes(&evaluate_keyed(
        &policy(),
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-TIER-TYPE-OVERLOAD"), "{c:?}");
}

#[test]
fn legacy_external_facing_tier_fails() {
    let m = json!({ "microservice": "developer-sdk", "tier": "external-facing", "tier_subtype": "product-developer-sdk", "dr_tier": "T1" });
    let c = codes(&evaluate_keyed(
        &policy(),
        &observed(vec![("oya/developer-sdk/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-TIER-TYPE-OVERLOAD"), "{c:?}");
}

#[test]
fn tier_not_in_enum_fails() {
    let m = json!({ "microservice": "x", "tier": "gizmo", "tier_subtype": "product-consumer", "dr_tier": "T2" });
    let c = codes(&evaluate_keyed(
        &policy(),
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-TIER-NOT-IN-ENUM"), "{c:?}");
}

#[test]
fn tier_subtype_not_in_enum_fails() {
    // erp-parity-single-concern is NOT in the canonical platform-architecture enum.
    let m = json!({ "microservice": "x", "tier": "product", "tier_subtype": "erp-parity-single-concern", "dr_tier": "T2" });
    let c = codes(&evaluate_keyed(
        &policy(),
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-TIER-SUBTYPE-NOT-IN-ENUM"), "{c:?}");
}

#[test]
fn dr_tier_not_in_enum_fails() {
    let m = json!({ "microservice": "x", "tier": "product", "tier_subtype": "product-consumer", "dr_tier": "T9" });
    let c = codes(&evaluate_keyed(
        &policy(),
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-DR-TIER-NOT-IN-ENUM"), "{c:?}");
}

#[test]
fn substrate_missing_dag_position_fails() {
    let m = json!({ "microservice": "x", "tier": "substrate", "tier_subtype": "substrate-infra", "dr_tier": "T1" });
    let c = codes(&evaluate_keyed(
        &policy(),
        &observed(vec![("cloud/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-SUBSTRATE-MISSING-DAG-POSITION"), "{c:?}");
}

#[test]
fn substrate_invalid_stratum_fails() {
    let m = json!({
        "microservice": "x", "tier": "substrate", "tier_subtype": "substrate-infra", "dr_tier": "T1",
        "substrate_dag_position": { "stratum": "S9", "depends_on": [], "consumed_by_substrates": [] }
    });
    let c = codes(&evaluate_keyed(
        &policy(),
        &observed(vec![("cloud/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-SUBSTRATE-DAG-STRATUM-INVALID"), "{c:?}");
}

#[test]
fn product_does_not_require_dag_position() {
    let m = json!({ "microservice": "x", "tier": "product", "tier_subtype": "product-consumer", "dr_tier": "T2" });
    assert!(evaluate_keyed(&policy(), &observed(vec![("oya/x/manifest.json", m)])).is_empty());
}

#[test]
fn empty_scan_below_floor_fails() {
    let obs = json!({ "manifest_count": 0, "manifests": [] });
    let c = codes(&evaluate_keyed(&policy(), &obs));
    assert!(c.contains("TFC-EMPTY-SCAN"), "{c:?}");
}

#[test]
fn wrong_gate_id_fails_closed() {
    let mut p = policy();
    p["gate_id"] = json!("not-the-gate");
    let m = json!({ "microservice": "x", "tier": "product", "tier_subtype": "product-consumer", "dr_tier": "T2" });
    let c = codes(&evaluate_keyed(
        &p,
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-POLICY-GATE-ID-MISMATCH"), "{c:?}");
}

#[test]
fn malformed_policy_fails_closed() {
    let mut p = policy();
    p["tier_enum"] = json!("not-an-array");
    let c = codes(&evaluate_keyed(&p, &observed(vec![])));
    assert!(c.contains("TFC-POLICY-MALFORMED"), "{c:?}");
}

fn strict_policy() -> Value {
    let mut p = policy();
    p["require_sharding_automation"] = json!(true);
    p["require_openslo_manifest_refs"] = json!(true);
    p["canonical_autosharding_mode"] = json!("control_plane_driven");
    p["allowed_disabled_autosharding_modes"] = json!(["not_claimed_runtime"]);
    p
}

fn observed_with_openslo(manifests: Vec<(&str, Value)>, openslo_files: &[(&str, bool)]) -> Value {
    let mut obs = observed(manifests);
    obs["available_openslo_files"] = json!(
        openslo_files
            .iter()
            .map(|(path, non_empty)| json!({ "path": path, "non_empty": non_empty }))
            .collect::<Vec<Value>>()
    );
    obs
}

fn green_sharding() -> Value {
    json!({
        "autosharding": { "enabled": false, "mode": "not_claimed_runtime", "intended_control_plane": "control_plane_driven" },
        "auto_rebalance": { "enabled": false },
        "dynamic_sharding": { "enabled": false }
    })
}

fn slo_exemption() -> Value {
    json!({
        "status": "live_exempted_no_runtime_sli",
        "owner": "axis-test",
        "rationale": "Unit fixture service has no measured runtime SLI until the gate fixture service exists.",
        "cutover_on": "2026-09-30",
        "ticket": "LANE-D-TEST-SLO-CUTOVER"
    })
}

#[test]
fn strict_sharding_block_is_required_for_top_level_service_manifest() {
    let m = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "slos": [],
        "slo_exemption": slo_exemption()
    });
    let c = codes(&evaluate_keyed(
        &strict_policy(),
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-SHARDING-MISSING-BLOCK"), "{c:?}");
}

#[test]
fn strict_manual_autosharding_mode_fails() {
    let m = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "sharding_automation": {
            "autosharding": "manual",
            "auto_rebalance": { "enabled": false },
            "dynamic_sharding": { "enabled": false }
        },
        "slos": [],
        "slo_exemption": slo_exemption()
    });
    let c = codes(&evaluate_keyed(
        &strict_policy(),
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-AUTOSHARDING-MANUAL-MODE"), "{c:?}");
}

#[test]
fn strict_enabled_rebalance_requires_residency_and_audit_emit() {
    let m = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "sharding_automation": {
            "autosharding": "control_plane_driven",
            "auto_rebalance": { "enabled": true, "trigger_load_skew_threshold_percent": 30 },
            "dynamic_sharding": { "enabled": false }
        },
        "slos": [],
        "slo_exemption": slo_exemption()
    });
    let c = codes(&evaluate_keyed(
        &strict_policy(),
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-AUTOREBALANCE-RESIDENCY-MISSING"), "{c:?}");
    assert!(
        c.contains("TFC-AUTOMATION-AUDIT-CHAIN-EMIT-MISSING"),
        "{c:?}"
    );
}

#[test]
fn strict_enabled_dynamic_sharding_requires_thresholds() {
    let m = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "sharding_automation": {
            "autosharding": "control_plane_driven",
            "auto_rebalance": { "enabled": false },
            "dynamic_sharding": { "enabled": true, "audit_chain_emit": true }
        },
        "slos": [],
        "slo_exemption": slo_exemption()
    });
    let c = codes(&evaluate_keyed(
        &strict_policy(),
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(
        c.contains("TFC-DYNAMIC-SHARDING-THRESHOLD-MISSING"),
        "{c:?}"
    );
}

#[test]
fn strict_slo_file_must_resolve_or_be_exempted() {
    let m = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "sharding_automation": green_sharding(),
        "slos": [{ "name": "availability", "file": "microservices/x/slos/availability.openslo.yaml" }]
    });
    let c = codes(&evaluate_keyed(
        &strict_policy(),
        &observed_with_openslo(vec![("oya/x/manifest.json", m)], &[]),
    ));
    assert!(c.contains("TFC-SLO-REFERENCE-UNRESOLVED"), "{c:?}");
}

#[test]
fn strict_existing_openslo_file_passes() {
    let m = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "sharding_automation": green_sharding(),
        "slos": [{ "name": "availability", "file": "oya/x/slos/availability.openslo.yaml" }]
    });
    let findings = evaluate_keyed(
        &strict_policy(),
        &observed_with_openslo(
            vec![("oya/x/manifest.json", m)],
            &[("oya/x/slos/availability.openslo.yaml", true)],
        ),
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn strict_empty_slos_require_explicit_exemption() {
    let m = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "sharding_automation": green_sharding(),
        "slos": []
    });
    let c = codes(&evaluate_keyed(
        &strict_policy(),
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-SLO-MISSING-OR-UNEXEMPT"), "{c:?}");
}

#[test]
fn strict_empty_openslo_file_does_not_count_as_coverage() {
    let m = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "sharding_automation": green_sharding(),
        "slos": [{ "name": "availability", "file": "oya/x/slos/availability.openslo.yaml" }]
    });
    let c = codes(&evaluate_keyed(
        &strict_policy(),
        &observed_with_openslo(
            vec![("oya/x/manifest.json", m)],
            &[("oya/x/slos/availability.openslo.yaml", false)],
        ),
    ));
    assert!(c.contains("TFC-SLO-REFERENCE-UNRESOLVED"), "{c:?}");
}

#[test]
fn strict_policy_roots_drive_top_level_service_checks() {
    let mut p = strict_policy();
    p["governed_service_roots"] = json!(["services"]);

    let governed = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "slos": [],
        "slo_exemption": slo_exemption()
    });
    let c = codes(&evaluate_keyed(
        &p,
        &observed(vec![("services/x/manifest.json", governed)]),
    ));
    assert!(c.contains("TFC-SHARDING-MISSING-BLOCK"), "{c:?}");

    let legacy_root = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2"
    });
    let findings = evaluate_keyed(&p, &observed(vec![("oya/x/manifest.json", legacy_root)]));
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn malformed_required_policy_booleans_fail_closed() {
    let mut p = policy();
    p["require_sharding_automation"] = json!("true");
    let c = codes(&evaluate_keyed(&p, &observed(vec![])));
    assert!(c.contains("TFC-POLICY-MALFORMED"), "{c:?}");

    let mut p = policy();
    p["require_openslo_manifest_refs"] = json!(null);
    let c = codes(&evaluate_keyed(&p, &observed(vec![])));
    assert!(c.contains("TFC-POLICY-MALFORMED"), "{c:?}");
}

#[test]
fn malformed_or_placeholder_slo_exemption_fails() {
    let m = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "sharding_automation": green_sharding(),
        "slos": [],
        "slo_exemption": {
            "status": "todo",
            "owner": "axis-test",
            "rationale": "Replace this exemption before production or hyperscaler-ready promotion.",
            "cutover_on": "2026-09-30",
            "ticket": "TODO"
        }
    });
    let c = codes(&evaluate_keyed(
        &strict_policy(),
        &observed(vec![("oya/x/manifest.json", m)]),
    ));
    assert!(c.contains("TFC-SLO-EXEMPTION-MALFORMED"), "{c:?}");
    assert!(c.contains("TFC-SLO-MISSING-OR-UNEXEMPT"), "{c:?}");
}

#[test]
fn scalar_slo_name_resolves_to_non_empty_local_openslo_file() {
    let m = json!({
        "microservice": "x",
        "tier": "product",
        "tier_subtype": "product-consumer",
        "dr_tier": "T2",
        "sharding_automation": green_sharding(),
        "slos": ["availability"]
    });
    let findings = evaluate_keyed(
        &strict_policy(),
        &observed_with_openslo(
            vec![("oya/x/manifest.json", m)],
            &[("oya/x/slos/availability.openslo.yaml", true)],
        ),
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn structured_named_slo_exemption_covers_planned_entry_without_stale_file() {
    let m = json!({
        "microservice": "connector",
        "tier": "substrate",
        "tier_subtype": "substrate-api-gateway",
        "dr_tier": "T2",
        "substrate_dag_position": { "stratum": "S0" },
        "sharding_automation": green_sharding(),
        "slos": [{ "name": "connect-retirement-evidence-lag", "target": "0.99", "sli": "retirement evidence lag" }],
        "slo_exemptions": [{
            "name": "connect-retirement-evidence-lag",
            "status": "live_exempted_retiring_service",
            "owner": "council-architecture",
            "rationale": "Connector retirement remains an audit-readiness workflow while runtime traffic migrates to replacement service surfaces.",
            "cutover_on": "2026-09-30",
            "ticket": "LANE-D-CONNECTOR-OPENSLO-CUTOVER"
        }]
    });
    let findings = evaluate_keyed(
        &strict_policy(),
        &observed(vec![("oya/connector/manifest.json", m)]),
    );
    assert!(findings.is_empty(), "{findings:?}");
}

/// REGRESSION PIN, 2026-08-08 (PR #1620 review, finding 1 of 10).
///
/// The capability-first rehome turned `cloud/cloud-billing/manifest.json` into
/// `billing/manifest.json` — ONE path component after the governed root, not two. The
/// predicate required exactly two, so fourteen substrate services silently left
/// `evaluate_sharding_automation` and `evaluate_openslo_manifest_refs` while the gate
/// still reported green: billing, compliance, console, flags, gateway, iac,
/// intelligence, k8s, marketplace, network, observability, secrets, storage, tenancy.
///
/// All fourteen satisfied the predicate before the move. A gate that stops looking at a
/// T0/T1 substrate service and stays green is the false-green class this gate exists to
/// prevent, which is why this is pinned rather than left to the corpus test.
#[test]
fn both_top_level_service_manifest_shapes_are_governed_and_deeper_paths_are_not() {
    let roots: BTreeSet<String> = ["billing", "cell", "cloud", "oya"]
        .into_iter()
        .map(String::from)
        .collect();

    // <root>/manifest.json — the capability's OWN manifest. This is the shape the rehome
    // produces and the one whose absence was the defect.
    assert!(
        is_top_level_service_manifest("billing/manifest.json", &roots),
        "a capability's own manifest must be governed; requiring two components dropped 14 services"
    );

    // <root>/<service>/manifest.json — a sub-service. Must keep working.
    assert!(
        is_top_level_service_manifest("oya/connector/manifest.json", &roots),
        "a sub-service manifest must stay governed"
    );

    // Deeper paths are not services. A test fixture that demands tier metadata it will
    // never legitimately carry is a false positive, so the depth limit is load-bearing.
    assert!(
        !is_top_level_service_manifest(
            "cell/core/regional-pack/tests/fixtures/kr/manifest.json",
            &roots
        ),
        "a tests/fixtures manifest is not a service and must not be pulled into tier coverage"
    );
    assert!(
        !is_top_level_service_manifest("oya/a/b/manifest.json", &roots),
        "nested non-service manifests stay out"
    );

    // Not a manifest, and not under a governed root.
    assert!(!is_top_level_service_manifest("billing/README.md", &roots));
    assert!(!is_top_level_service_manifest(
        "ungoverned/manifest.json",
        &roots
    ));
}
