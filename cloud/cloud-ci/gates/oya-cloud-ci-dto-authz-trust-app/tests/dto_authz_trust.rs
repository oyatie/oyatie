//! cloud-ci-dto-authz-trust gate — live-corpus self-test + RED/GREEN fixtures (the CLASS-FIX for
//! caller-supplied-authorization-trust; sibling of cloud-ci-authz-coverage / issue #770 / AUTH-005).
//!
//! The LIVE leg walks the real candidate tree with the committed policy and asserts the gate is
//! born-blocking GREEN today: every pre-existing caller-supplied-authz-trust instance is frozen in
//! `frozen_dto_authz_trust_instances`, so the verdict is GREEN and no NEW instance is present. It
//! also asserts the FIXED iam/ports/policy-cedar-api/src/authz.rs PDP-port pattern is NOT flagged
//! (the GREEN reference) and a representative known instance (secrets/ports/kms-api) IS in the
//! baseline. The RED fixture proves the gate genuinely FAILS on a NEW function that trusts a forged
//! authorization DTO with no PDP call; the GREEN fixture proves a verify_principal +
//! ensure_authorized PDP handler is tolerated even when it also reads the DTO.
//!
//! ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use oya_cloud_ci_dto_authz_trust_app::{
    GATE_ID, Verdict, collect_instances, evaluate, evaluate_keyed,
};
use serde_json::{Value, json};

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`).
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

const POLICY_REL: &str =
    "cloud/cloud-ci/gates/oya-cloud-ci-dto-authz-trust-app/dto-authz-trust-policy.json";

fn load_committed_policy(root: &Path) -> Value {
    let path = root.join(POLICY_REL);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read policy {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse policy {}: {e}", path.display()))
}

/// Test-only policy whose scan_roots point at a fixture directory written under a temp dir, so the
/// RED/GREEN exhibits are evaluated in isolation from the live tree.
fn fixture_policy(scan_root: &str) -> Value {
    json!({
        "gate_id": GATE_ID,
        "min_expected_functions": 0,
        "scan_roots": [scan_root],
        "excluded_dir_names": ["target", "third-party"],
        "authorization_dto_type_suffixes": ["Authorization"],
        "trigger_decision_field_idents": ["allowed_surfaces"],
        "decision_field_idents": ["allowed_surfaces", "decision_id"],
        "authorization_header_idents": ["x-authorization-decision"],
        "pdp_decision_idents": [".decide(", "ensure_authorized", "verify_principal", "check_authz", "ensure_authz"],
        "self_compare_tokens": ["==", "!=", ".iter().any(", ".contains("],
        "frozen_dto_authz_trust_instances": []
    })
}

// ---------------------------------------------------------------------------
// LIVE leg: born-blocking green over the real candidate tree.
// ---------------------------------------------------------------------------

#[test]
fn live_tree_is_green_against_the_frozen_baseline() {
    let root = repo_root();
    let policy = load_committed_policy(&root);
    assert_eq!(
        policy.get("gate_id").and_then(Value::as_str),
        Some(GATE_ID),
        "committed policy gate_id must be {GATE_ID}"
    );

    let observed = collect_instances(&root, &policy).expect("collect over live tree");
    let findings = evaluate_keyed(&policy, &observed);

    // The gate must be GREEN: no NEW caller-supplied-authz-trust instance, and no stale-baseline /
    // policy / empty-scan finding. If this fails, the failing finding details say exactly why.
    let report = evaluate(&policy, &observed);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "dto-authz-trust gate is not GREEN over the live tree. Findings:\n{}",
        oya_cloud_ci_dto_authz_trust_app::render_findings(&findings)
    );
}

#[test]
fn baseline_is_nonempty_and_covers_a_known_instance() {
    let root = repo_root();
    let policy = load_committed_policy(&root);
    let baseline: Vec<String> = policy
        .get("frozen_dto_authz_trust_instances")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default();

    assert!(
        baseline.len() >= 40,
        "expected the frozen baseline to enumerate the ~50 known instances the review found; got {}",
        baseline.len()
    );
    // A representative confirmed instance from the task must be in the baseline.
    for expected in [
        "secrets/ports/kms-api/src/lib.rs#validate_authorization",
        "tenancy/ports/api/src/lib.rs#validate_authorization",
        "network/ports/lb/src/lib.rs#validate_authorization",
        "audit/core/usecase/src/lib.rs#validate_authorization",
        "observability/core/api/src/lib.rs#validate_authorization",
        "billing/ports/finops-api/src/lib.rs#validate_authorization",
    ] {
        assert!(
            baseline.iter().any(|k| k == expected),
            "expected confirmed instance `{expected}` in the frozen baseline"
        );
    }
}

/// The FIXED IAM keystone pattern (iam/ports/policy-cedar-api/src/authz.rs) — verify_principal +
/// ensure_authorized PDP ports — must NOT be flagged as a caller-supplied-authz-trust instance. If
/// the engine ever flagged that module it would be a false positive against the GREEN reference.
#[test]
fn fixed_iam_authz_module_is_not_flagged() {
    let root = repo_root();
    let policy = load_committed_policy(&root);
    let observed = collect_instances(&root, &policy).expect("collect over live tree");
    let instances = observed
        .get("instances")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let flagged_authz_rs = instances.iter().any(|i| {
        i.get("file").and_then(Value::as_str)
            == Some("iam/ports/policy-cedar-api/src/authz.rs")
    });
    assert!(
        !flagged_authz_rs,
        "the FIXED iam/ports/policy-cedar-api/src/authz.rs PDP-port pattern (verify_principal + \
         ensure_authorized) must never be flagged — it is the GREEN reference"
    );
}

// ---------------------------------------------------------------------------
// RED / GREEN fixtures: prove the gate genuinely fails and genuinely passes.
// ---------------------------------------------------------------------------

/// Write a `src/<name>.rs` fixture under a unique temp dir and return the temp root path. The temp
/// dir name embeds the test name + pid so concurrent test runs do not collide.
fn write_fixture(test: &str, name: &str, src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "dto-authz-trust-{test}-{}",
        std::process::id()
    ));
    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("create fixture dir");
    std::fs::write(src_dir.join(format!("{name}.rs")), src).expect("write fixture");
    dir
}

const RED_FORGED_DTO: &str = r#"
    pub struct CloudNewApiAuthorization {
        pub tenant_id: String,
        pub principal_id: String,
        pub decision_id: String,
        pub allowed_surfaces: Vec<String>,
    }
    struct Principal { tenant_id: String, principal_id: String }

    // The antipattern: trusts the caller-supplied authorization blob, no PDP call.
    fn validate_authorization(
        principal: &Principal,
        authorization: &CloudNewApiAuthorization,
        surface: &str,
    ) -> Result<(), Error> {
        if authorization.decision_id.trim().is_empty() {
            return Err(Error::Empty);
        }
        if authorization.tenant_id != principal.tenant_id {
            return Err(Error::TenantMismatch);
        }
        if !authorization.allowed_surfaces.iter().any(|s| s == surface) {
            return Err(Error::Denied);
        }
        Ok(())
    }
"#;

const GREEN_PDP_BACKED: &str = r#"
    struct ReqAuthorization { allowed_surfaces: Vec<String>, decision_id: String }

    // The fixed pattern: verify a principal from a credential, call a PDP decide() port, fail closed.
    fn ensure_authorized_handler(
        credential: &CallerCredential,
        authorization: &ReqAuthorization,
        resource: &Resource,
    ) -> Result<(), Error> {
        let principal = verifier.verify_principal(credential)?;
        if authorization.decision_id.is_empty() { return Err(Error::Empty); }
        authorizer.ensure_authorized(&principal, resource)?;
        Ok(())
    }
"#;

#[test]
fn red_fixture_new_forged_dto_is_blocked() {
    let dir = write_fixture("red", "lib", RED_FORGED_DTO);
    let policy = fixture_policy("src");
    let observed = collect_instances(&dir, &policy).expect("collect red fixture");
    let report = evaluate(&policy, &observed);
    assert_eq!(
        report.verdict,
        Verdict::Red,
        "a NEW forged-DTO self-validating handler must be RED; observed={observed:#}"
    );
    assert!(report.violations.contains("DAT-CALLER-SUPPLIED-AUTHZ-TRUST"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn green_fixture_pdp_backed_handler_is_clean() {
    let dir = write_fixture("green", "lib", GREEN_PDP_BACKED);
    let policy = fixture_policy("src");
    let observed = collect_instances(&dir, &policy).expect("collect green fixture");
    let report = evaluate(&policy, &observed);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "a PDP-backed (verify_principal + ensure_authorized) handler must be GREEN; observed={observed:#}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn baselined_instance_is_tolerated_but_a_sibling_new_one_is_not() {
    // Two instances in one fixture: one baselined (tolerated), one NOT (must RED).
    let dir = write_fixture("baseline", "lib", RED_FORGED_DTO);
    let mut policy = fixture_policy("src");
    // Baseline the fixture's instance key.
    policy["frozen_dto_authz_trust_instances"] = json!(["src/lib.rs#validate_authorization"]);
    let observed = collect_instances(&dir, &policy).expect("collect");
    let report = evaluate(&policy, &observed);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "a baselined instance must be tolerated; observed={observed:#}"
    );

    // Now add a NEW sibling instance not in the baseline → RED.
    let new_src = format!(
        "{RED_FORGED_DTO}\n{}",
        RED_FORGED_DTO.replace("validate_authorization", "validate_authorization_v2")
    );
    std::fs::write(dir.join("src/lib.rs"), &new_src).expect("rewrite");
    let observed2 = collect_instances(&dir, &policy).expect("collect2");
    let report2 = evaluate(&policy, &observed2);
    assert_eq!(
        report2.verdict,
        Verdict::Red,
        "a NEW sibling instance not in the baseline must be RED; observed={observed2:#}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
