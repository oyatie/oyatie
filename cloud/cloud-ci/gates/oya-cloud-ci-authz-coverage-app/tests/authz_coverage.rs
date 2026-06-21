// cloud-ci-authz-coverage live-corpus self-test (issue #770; AUTH-005 backstop).
//
// Three legs:
//   1. LIVE: walk the real candidate tree under the committed policy and assert the gate is
//      born-blocking GREEN (every NEW control-plane surface authz-covered; pre-existing
//      unauthenticated surfaces frozen in the baseline). This is the load-bearing "lands green"
//      acceptance.
//   2. GREEN-REAL: assert the two reference surfaces — intelligence/adapters/rest and
//      tenancy/facade/tenant-lifecycle-app — are recognized as COVERED (no live finding keyed to
//      them), proving the detector recognizes the real admin_tenant_allowed / authorize() patterns.
//   3. RED-FIXTURE + EXEMPT: a synthetic unauthenticated mutating router fails closed; an
//      allowlisted health endpoint is exempt.
//
// Pure filesystem; no network, no git. ADR-0083 Tier-3: integration tests use unwrap/expect/panic.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use oya_cloud_ci_authz_coverage_app::{
    GATE_ID, Verdict, collect_surfaces, evaluate, evaluate_keyed, render_findings,
};
use serde_json::{Value, json};

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Mirrors the helper the firewall meta-gates use.
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

fn policy_path(root: &Path) -> PathBuf {
    root.join("cloud/cloud-ci/gates/oya-cloud-ci-authz-coverage-app/authz-coverage-policy.json")
}

fn load_policy(root: &Path) -> Value {
    let text = std::fs::read_to_string(policy_path(root)).expect("read committed policy");
    serde_json::from_str(&text).expect("parse committed policy")
}

/// A copy of the committed policy with the live-corpus DATA stripped — no frozen baseline (so it
/// does not go stale against a single synthetic observation) and no surface floor (so the
/// synthetic 1-surface observations do not trip AC-EMPTY-SCAN). The recognition vocabulary
/// (guard/auth-layer idents, exempt paths) is the REAL committed one, so the synthetic RED/exempt
/// cases exercise the committed gate-id + exempt allowlist exactly.
fn synthetic_policy(root: &Path) -> Value {
    let mut p = load_policy(root);
    p["frozen_unauthenticated_surfaces"] = serde_json::json!([]);
    p["min_expected_surfaces"] = serde_json::json!(0);
    p
}

#[test]
fn live_corpus_is_born_blocking_green() {
    let root = repo_root();
    let policy = load_policy(&root);
    assert_eq!(
        policy.get("gate_id").and_then(Value::as_str),
        Some(GATE_ID),
        "committed policy gate_id must be {GATE_ID}"
    );

    let observed = collect_surfaces(&root, &policy).expect("collect live surfaces");
    let surfaces_found = observed
        .get("surfaces_found")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    assert!(
        surfaces_found
            >= policy
                .get("min_expected_surfaces")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        "live scan found {surfaces_found} surfaces, below the policy floor — the scan roots or \
         excludes are likely broken"
    );

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "the authz-coverage gate must be born-blocking GREEN on the live corpus (NEW control \
         planes authz-covered; pre-existing surfaces frozen in the baseline). Live findings:\n{}",
        render_findings(&findings)
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);
}

#[test]
fn reference_surfaces_are_recognized_as_covered() {
    let root = repo_root();
    let policy = load_policy(&root);
    let observed = collect_surfaces(&root, &policy).expect("collect live surfaces");
    let surfaces = observed.get("surfaces").and_then(Value::as_array).unwrap();

    // The intelligence/adapters/rest surface MUST be observed and MUST NOT appear in any
    // unauthenticated finding (its admin handlers call admin_tenant_allowed; its data-plane POST
    // calls require_data_plane_bearer).
    let rest_observed = surfaces.iter().any(|s| {
        s.get("file").and_then(Value::as_str).map(|f| f.contains("intelligence/adapters/rest/src/lib.rs")) == Some(true)
    });
    assert!(rest_observed, "intelligence/adapters/rest router surface must be observed");

    // The tenancy lifecycle surface MUST be observed (its authorize() per route covers it).
    let tenancy_observed = surfaces.iter().any(|s| {
        s.get("file").and_then(Value::as_str).map(|f| f.contains("tenancy/facade/tenant-lifecycle-app/src/lib.rs")) == Some(true)
    });
    assert!(tenancy_observed, "tenancy tenant-lifecycle router surface must be observed");

    // Neither reference surface may be in the frozen baseline (they are GREEN by authz, not by
    // exemption) nor produce a live finding.
    let baseline: Vec<String> = policy
        .get("frozen_unauthenticated_surfaces")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default();
    for ref_file in [
        "intelligence/adapters/rest/src/lib.rs",
        "tenancy/facade/tenant-lifecycle-app/src/lib.rs",
    ] {
        assert!(
            !baseline.iter().any(|k| k.contains(ref_file)),
            "{ref_file} must be GREEN via authz detection, not frozen in the baseline"
        );
    }

    let findings = evaluate_keyed(&policy, &observed);
    for ref_file in [
        "intelligence/adapters/rest/src/lib.rs",
        "tenancy/facade/tenant-lifecycle-app/src/lib.rs",
    ] {
        assert!(
            !findings
                .iter()
                .any(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE" && f.key.contains(ref_file)),
            "{ref_file} must not be flagged unauthenticated (its authz pattern must be recognized)"
        );
    }
}

#[test]
fn red_on_synthetic_unauthenticated_router() {
    let policy = synthetic_policy(&repo_root());
    // A synthetic surface bypassing the filesystem: a POST + DELETE with no authz in either body.
    let observed = json!({
        "surfaces_found": 1,
        "surfaces": [{
            "file": "synthetic/unauth.rs",
            "router_line": 1,
            "routes": [
                { "path": "/things", "method": "post", "handler": "create_thing" },
                { "path": "/things/{id}", "method": "delete", "handler": "delete_thing" },
                { "path": "/healthz", "method": "get", "handler": "healthz" }
            ],
            "has_auth_layer": false,
            "handler_authz": { "create_thing": false, "delete_thing": false, "healthz": false }
        }]
    });
    let findings = evaluate_keyed(&policy, &observed);
    let hit = findings
        .iter()
        .find(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE")
        .expect("a synthetic unauthenticated mutating router must be RED");
    assert_eq!(hit.key, "synthetic/unauth.rs::router@1");
    assert!(hit.detail.contains("intelligence/adapters/rest"), "remediation points at the doctrine");
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

#[test]
fn exempt_health_endpoint_is_not_a_control_plane() {
    let policy = synthetic_policy(&repo_root());
    // A POST only to an exempt path (e.g. /metrics push) is not a control plane.
    let observed = json!({
        "surfaces_found": 1,
        "surfaces": [{
            "file": "synthetic/health.rs",
            "router_line": 1,
            "routes": [
                { "path": "/healthz", "method": "get", "handler": "healthz" },
                { "path": "/metrics", "method": "get", "handler": "metrics" }
            ],
            "has_auth_layer": false,
            "handler_authz": { "healthz": false, "metrics": false }
        }]
    });
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "a read-only health/metrics router must be exempt, not a control plane: {findings:?}"
    );
}
