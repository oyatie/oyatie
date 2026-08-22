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

use ci_endpoint_authorization_coverage::{
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
    root.join("ci/facade/endpoint-authorization-coverage/authz-coverage-policy.json")
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
        s.get("file")
            .and_then(Value::as_str)
            .map(|f| f.contains("intelligence/adapters/rest/src/lib.rs"))
            == Some(true)
    });
    assert!(
        rest_observed,
        "intelligence/adapters/rest router surface must be observed"
    );

    // The tenancy lifecycle surface MUST be observed (its authorize() per route covers it).
    let tenancy_observed = surfaces.iter().any(|s| {
        s.get("file")
            .and_then(Value::as_str)
            .map(|f| f.contains("tenancy/facade/tenant-lifecycle-app/src/lib.rs"))
            == Some(true)
    });
    assert!(
        tenancy_observed,
        "tenancy tenant-lifecycle router surface must be observed"
    );

    // Neither reference surface may be in the frozen baseline (they are GREEN by authz, not by
    // exemption) nor produce a live finding.
    let baseline: Vec<String> = policy
        .get("frozen_unauthenticated_surfaces")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
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
    // Routes carry the post-review observation shape (`method_class` + `path_raw`).
    let observed = json!({
        "surfaces_found": 1,
        "surfaces": [{
            "file": "synthetic/unauth.rs",
            "scope": "build_router",
            "router_line": 1,
            "routes": [
                { "path": "/things", "path_raw": "/things", "method": "post", "method_class": "mutating", "handler": "create_thing", "surface_unclassified": false },
                { "path": "/things/{id}", "path_raw": "/things/{id}", "method": "delete", "method_class": "mutating", "handler": "delete_thing", "surface_unclassified": false },
                { "path": "/healthz", "path_raw": "/healthz", "method": "get", "method_class": "non-mutating", "handler": "healthz", "surface_unclassified": false }
            ],
            "has_auth_layer": false,
            "handler_authz": { "create_thing": false, "delete_thing": false, "healthz": false },
            "unresolved_subrouters": []
        }]
    });
    let findings = evaluate_keyed(&policy, &observed);
    let hit = findings
        .iter()
        .find(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE")
        .expect("a synthetic unauthenticated mutating router must be RED");
    // The key is now a stable scope+signature (M2): file#scope + ALL sorted (method, path) tuples
    // (the whole router's identity, including the non-mutating /healthz), not `router@<line>`.
    assert_eq!(
        hit.key,
        "synthetic/unauth.rs#build_router::router[delete /things/{id}; get /healthz; post /things]"
    );
    assert!(
        hit.detail.contains("intelligence/adapters/rest"),
        "remediation points at the doctrine"
    );
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
                { "path": "/healthz", "path_raw": "/healthz", "method": "get", "method_class": "non-mutating", "handler": "healthz" },
                { "path": "/metrics", "path_raw": "/metrics", "method": "get", "method_class": "non-mutating", "handler": "metrics" }
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

/// END-TO-END (PR #780 review B1/B2/B3): write the four reproduced bypass fixtures into an isolated
/// temp tree and run the REAL filesystem collector + evaluator over them; each must produce a finding
/// (RED). This is the load-bearing proof that the collect->parse->evaluate path — not just the pure
/// evaluator on hand-built JSON — fails closed on the idiomatic-axum bypasses. The temp tree is
/// outside the repo scan roots, so it never pollutes the live gate.
#[test]
fn bypass_fixtures_fail_closed_end_to_end() {
    use std::fs;

    let base = std::env::temp_dir().join(format!(
        "authz-coverage-fixtures-{}-{}",
        std::process::id(),
        // a per-test discriminator so parallel test bins don't collide
        line!()
    ));
    let src = base.join("src");
    fs::create_dir_all(&src).expect("create temp fixture dir");

    // B1: const-path unauthenticated mutating route.
    fs::write(
        src.join("b1_const.rs"),
        r#"const NUKE: &str = "/tenants/{id}";
           async fn nuke() -> StatusCode { StatusCode::NO_CONTENT }
           pub fn r() -> Router { Router::new().route(NUKE, delete(nuke)).with_state(()) }"#,
    )
    .unwrap();
    // B2a: MethodRouter bound to a variable, unauthenticated.
    fs::write(
        src.join("b2_var.rs"),
        r#"async fn del() -> StatusCode { StatusCode::NO_CONTENT }
           pub fn r() -> Router { let m = delete(del); Router::new().route("/x/{id}", m).with_state(()) }"#,
    )
    .unwrap();
    // B2b: on(MethodFilter::DELETE, h), unauthenticated.
    fs::write(
        src.join("b2_on.rs"),
        r#"async fn del() -> StatusCode { StatusCode::NO_CONTENT }
           pub fn r() -> Router { Router::new().route("/y/{id}", on(MethodFilter::DELETE, del)).with_state(()) }"#,
    )
    .unwrap();
    // B3: comment-only "guard" handler.
    fs::write(
        src.join("b3_comment.rs"),
        r#"async fn del() -> StatusCode {
               // TODO: authorize() this later
               StatusCode::NO_CONTENT
           }
           pub fn r() -> Router { Router::new().route("/z/{id}", delete(del)).with_state(()) }"#,
    )
    .unwrap();

    // A policy whose scan root is the temp tree, with NO baseline + NO floor so each fixture is RED.
    let root = repo_root();
    let mut policy = load_policy(&root);
    policy["scan_roots"] = json!(["src"]);
    policy["frozen_unauthenticated_surfaces"] = json!([]);
    policy["min_expected_surfaces"] = json!(0);

    let observed = collect_surfaces(&base, &policy).expect("collect temp fixtures");
    let findings = evaluate_keyed(&policy, &observed);

    let by_file = |needle: &str, code: &str| {
        findings
            .iter()
            .any(|f| f.code == code && f.key.contains(needle))
    };
    assert!(
        by_file("b1_const.rs", "AC-UNAUTHENTICATED-CONTROL-PLANE"),
        "B1 const-path bypass must be RED end-to-end: {}",
        render_findings(&findings)
    );
    assert!(
        by_file("b2_var.rs", "AC-UNAUTHENTICATED-CONTROL-PLANE"),
        "B2 MethodRouter-variable bypass must be RED end-to-end: {}",
        render_findings(&findings)
    );
    assert!(
        by_file("b2_on.rs", "AC-UNAUTHENTICATED-CONTROL-PLANE"),
        "B2 on(MethodFilter::DELETE) bypass must be RED end-to-end: {}",
        render_findings(&findings)
    );
    assert!(
        by_file("b3_comment.rs", "AC-UNAUTHENTICATED-CONTROL-PLANE"),
        "B3 comment-only-guard bypass must be RED end-to-end: {}",
        render_findings(&findings)
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    let _ = fs::remove_dir_all(&base);
}

/// END-TO-END (PR #780 SECOND-PASS): the five reproduced surface-DISCOVERY bypasses + two false-cover
/// CONTROLS + GREEN counterparts, run through the REAL filesystem collector + evaluator. The first
/// fix closed within-surface gaps but surface DISCOVERY was still fail-OPEN: anchored on the literal
/// `Router::new()` with a chain truncated at `.with_state(`. Each RED fixture below was empirically
/// shown to PASS the gate on the pre-fix binary; each must now be RED. The CONTROLS prove the
/// whole-token auth-layer/guard match has no false-cover; the GREEN fixtures prove legit covered
/// routers still pass.
#[test]
fn second_pass_discovery_bypasses_fail_closed_end_to_end() {
    use std::fs;

    let base = std::env::temp_dir().join(format!(
        "authz-coverage-s2-fixtures-{}-{}",
        std::process::id(),
        line!()
    ));
    let src = base.join("src");
    fs::create_dir_all(&src).expect("create temp fixture dir");

    // BLOCKER-1a: Router::default().route(...) — never produced by Router::new().
    fs::write(
        src.join("s1_default.rs"),
        r#"async fn h() -> StatusCode { StatusCode::NO_CONTENT }
           pub fn r() -> Router { Router::default().route("/v1/tenants/{id}", delete(h)).with_state(()) }"#,
    )
    .unwrap();
    // BLOCKER-1b: route on a Router PARAMETER in a helper fn (no constructor here at all).
    fs::write(
        src.join("s1_param.rs"),
        r#"async fn h() -> StatusCode { StatusCode::OK }
           pub fn add(r: Router) -> Router { r.route("/admin/v1/create", post(h)) }"#,
    )
    .unwrap();
    // BLOCKER-3: a route declared AFTER the `.with_state(` chain on a bound variable.
    fs::write(
        src.join("s3_after_state.rs"),
        r#"async fn hz() -> StatusCode { StatusCode::OK }
           async fn nuke() -> StatusCode { StatusCode::NO_CONTENT }
           pub fn r() -> Router {
               let b = Router::new().route("/healthz", get(hz)).with_state(());
               b.route("/tenants/{id}", delete(nuke))
           }"#,
    )
    .unwrap();
    // BLOCKER-4 CONTROL: auth-layer substring (RequireAuthMetricsRecorder) must NOT false-cover.
    fs::write(
        src.join("s4_layer_substring.rs"),
        r#"async fn create_thing() -> StatusCode { StatusCode::OK }
           pub fn r() -> Router {
               Router::new().route("/v1/things", post(create_thing)).layer(RequireAuthMetricsRecorder::new()).with_state(())
           }"#,
    )
    .unwrap();
    // BLOCKER-5 CONTROL: guard substring (unauthorized_response) must NOT false-cover.
    fs::write(
        src.join("s5_guard_substring.rs"),
        r#"fn unauthorized_response() -> StatusCode { StatusCode::UNAUTHORIZED }
           async fn nuke() -> StatusCode { let _ = unauthorized_response(); StatusCode::NO_CONTENT }
           pub fn r() -> Router { Router::new().route("/v1/tenants/{id}", delete(nuke)).with_state(()) }"#,
    )
    .unwrap();
    // Composition fail-closed: an unresolved .merge(subrouter()).
    fs::write(
        src.join("comp_merge.rs"),
        r#"async fn hz() -> StatusCode { StatusCode::OK }
           pub fn r() -> Router { Router::new().route("/healthz", get(hz)).merge(admin_subrouter()).with_state(()) }"#,
    )
    .unwrap();
    // Owned-kernel POST (oya-http-router-kernel grammar) with no guard.
    fs::write(
        src.join("owned_post.rs"),
        r#"fn build(router: &mut Router<SyncHandler>) -> Result<(), RouterError> {
               router.route(HttpMethod::Post, "/admin/v1/provision", provision_handler)?;
               Ok(())
           }"#,
    )
    .unwrap();
    // GREEN: a real RequireAuth layer + a real authorize() guard handler must PASS.
    fs::write(
        src.join("green_covered.rs"),
        r#"async fn create_thing() -> StatusCode { StatusCode::OK }
           async fn retire(headers: HeaderMap) -> StatusCode { authorize(&state, &headers, Action::Retire)?; StatusCode::NO_CONTENT }
           pub fn layered() -> Router {
               Router::new().route("/v1/things", post(create_thing)).layer(RequireAuth::new(v)).with_state(())
           }
           pub fn guarded() -> Router {
               Router::new().route("/v1/tenants/{id}", delete(retire)).with_state(())
           }"#,
    )
    .unwrap();

    // MAJOR fix RED fixture: owned-kernel-SHAPED 3-arg call where arg2 is a field access
    // (`route.path` contains `.` → not a literal/ident path). Must fail-CLOSED as
    // AC-UNCLASSIFIED-SURFACE instead of silently dropping (fail-open). Reproduces the exact
    // shape of libs/oya-shared-backbone-rest-runtime-adapter/src/lib.rs:503.
    fs::write(
        src.join("major_field_path.rs"),
        r#"fn register(router: &mut Router<SyncHandler>, route: &RouteSpec, handler: SyncHandler) {
               router.route(method, route.path, handler).expect("route");
           }"#,
    )
    .unwrap();

    // MINOR fix GREEN fixture: a router declared inside a `tests/` subdirectory must NOT be
    // scanned (excluded_dir_names now includes "tests"). Create the dir one level under `src`
    // — the walk will skip it entirely.
    let tests_dir = src.join("tests");
    fs::create_dir_all(&tests_dir).expect("create tests subdir");
    fs::write(
        tests_dir.join("integ.rs"),
        r#"async fn h() -> StatusCode { StatusCode::NO_CONTENT }
           pub fn test_router() -> Router { Router::new().route("/admin/v1/test", post(h)).with_state(()) }"#,
    )
    .unwrap();

    let root = repo_root();
    let mut policy = load_policy(&root);
    policy["scan_roots"] = json!(["src"]);
    policy["frozen_unauthenticated_surfaces"] = json!([]);
    policy["min_expected_surfaces"] = json!(0);

    let observed = collect_surfaces(&base, &policy).expect("collect temp fixtures");
    let findings = evaluate_keyed(&policy, &observed);
    let red = |needle: &str, code: &str| {
        findings
            .iter()
            .any(|f| f.code == code && f.key.contains(needle))
    };

    assert!(
        red("s1_default.rs", "AC-UNAUTHENTICATED-CONTROL-PLANE"),
        "BLOCKER-1 Router::default() bypass must be RED: {}",
        render_findings(&findings)
    );
    assert!(
        red("s1_param.rs", "AC-UNAUTHENTICATED-CONTROL-PLANE"),
        "BLOCKER-2 Router-parameter helper bypass must be RED: {}",
        render_findings(&findings)
    );
    assert!(
        red("s3_after_state.rs", "AC-UNAUTHENTICATED-CONTROL-PLANE"),
        "BLOCKER-3 route-after-with_state bypass must be RED: {}",
        render_findings(&findings)
    );
    assert!(
        red("s4_layer_substring.rs", "AC-UNAUTHENTICATED-CONTROL-PLANE"),
        "BLOCKER-4 auth-layer substring must NOT false-cover (must be RED): {}",
        render_findings(&findings)
    );
    assert!(
        red("s5_guard_substring.rs", "AC-UNAUTHENTICATED-CONTROL-PLANE"),
        "BLOCKER-5 guard substring must NOT false-cover (must be RED): {}",
        render_findings(&findings)
    );
    assert!(
        red("comp_merge.rs", "AC-UNRESOLVED-SUBROUTER"),
        "unresolved .merge() composition must fail closed: {}",
        render_findings(&findings)
    );
    assert!(
        red("owned_post.rs", "AC-UNAUTHENTICATED-CONTROL-PLANE"),
        "owned-kernel POST with no guard must be RED: {}",
        render_findings(&findings)
    );

    // MAJOR fix: 3-arg field-path call must fail-CLOSED as AC-UNCLASSIFIED-SURFACE.
    assert!(
        red("major_field_path.rs", "AC-UNCLASSIFIED-SURFACE"),
        "MAJOR fix: .route(method_var, field.path, handler) must fail-CLOSED AC-UNCLASSIFIED-SURFACE \
         not silently drop: {}",
        render_findings(&findings)
    );

    // GREEN: neither covered router in green_covered.rs may produce any finding.
    assert!(
        !findings.iter().any(|f| f.key.contains("green_covered.rs")),
        "a real RequireAuth layer + a real authorize() guard must PASS (no finding): {}",
        render_findings(&findings)
    );

    // MINOR fix: routes inside a `tests/` subdirectory must NOT be scanned.
    assert!(
        !findings.iter().any(|f| f.key.contains("integ.rs")),
        "MINOR fix: a router inside tests/ subdir must be excluded from scan (not flagged): {}",
        render_findings(&findings)
    );

    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    let _ = fs::remove_dir_all(&base);
}

/// SCAN-ROOTS WIDENING (this change): `scan_roots` was widened from 10 dirs to the 20-root superset
/// the sibling `cloud-ci-dto-authz-trust` gate already scans, closing a FAIL-OPEN where a NEW
/// unauthenticated HTTP control plane in one of the 10 previously-unscanned roots (audit, cell,
/// compliance, compute, data, network, observability, secrets, storage, workflow) passed the gate.
/// This end-to-end proof writes an unauthenticated mutating axum router into a `secrets/` fixture (a
/// NEWLY-added root) and asserts: (a) under the COMMITTED (widened) scan_roots it is RED — the
/// widening BITES; (b) under the PRIOR 10 roots the IDENTICAL surface is invisible and the gate is
/// GREEN — the exact fail-open this change closes.
#[test]
fn widened_scan_root_bites_new_unauth_surface_end_to_end() {
    use std::fs;

    let base = std::env::temp_dir().join(format!(
        "authz-coverage-widen-fixtures-{}-{}",
        std::process::id(),
        line!()
    ));
    // A NEW unauthenticated mutating control plane under `secrets/` (one of the widened roots): a
    // POST to a per-resource path with no auth layer and no per-handler guard.
    let secrets_src = base.join("secrets/rotation-preview/src");
    fs::create_dir_all(&secrets_src).expect("create temp secrets fixture dir");
    fs::write(
        secrets_src.join("lib.rs"),
        r#"async fn rotate() -> StatusCode { StatusCode::OK }
           pub fn r() -> Router { Router::new().route("/v1/secrets/{id}/rotate", post(rotate)).with_state(()) }"#,
    )
    .unwrap();

    let root = repo_root();
    let mut policy = load_policy(&root);
    policy["frozen_unauthenticated_surfaces"] = json!([]);
    policy["min_expected_surfaces"] = json!(0);

    // Guard the widening from regression: the committed policy must scan every one of the 10
    // newly-added roots (sibling dto-authz-trust 20-root parity).
    let widened: Vec<String> = policy["scan_roots"]
        .as_array()
        .expect("scan_roots array")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    for added in [
        "audit",
        "cell",
        "compliance",
        "compute",
        "data",
        "network",
        "observability",
        "secrets",
        "storage",
        "workflow",
    ] {
        assert!(
            widened.iter().any(|r| r == added),
            "scan_roots must include the widened root `{added}` (dto-authz-trust parity)"
        );
    }

    // (a) POST-WIDEN: the committed scan_roots include `secrets/`, so the new unauthenticated
    // control plane is caught → RED.
    let observed = collect_surfaces(&base, &policy).expect("collect widened fixtures");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "AC-UNAUTHENTICATED-CONTROL-PLANE" && f.key.contains("secrets/")),
        "the widened scan must FLAG the unauthenticated secrets/ control plane: {}",
        render_findings(&findings)
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    // (b) PRE-WIDEN: the prior 10 roots did NOT include `secrets/`, so the IDENTICAL surface was
    // unscanned — the gate was GREEN. This is the fail-open the widening closes.
    let mut prewiden = policy.clone();
    prewiden["scan_roots"] = json!([
        "billing",
        "cloud",
        "console",
        "iac",
        "iam",
        "intelligence",
        "k8s",
        "libs",
        "oya",
        "tenancy"
    ]);
    let observed_pre = collect_surfaces(&base, &prewiden).expect("collect pre-widen fixtures");
    let findings_pre = evaluate_keyed(&prewiden, &observed_pre);
    assert!(
        findings_pre.is_empty(),
        "pre-widen the secrets/ control plane was UNSCANNED (fail-open) — it must produce no \
         finding under the prior 10 roots: {}",
        render_findings(&findings_pre)
    );

    let _ = fs::remove_dir_all(&base);
}
