//! cloud-ci-dto-authz-trust gate — live-corpus self-test + RED/GREEN fixtures (the CLASS-FIX for
//! caller-supplied-authorization-trust; sibling of cloud-ci-authz-coverage / issue #770 / AUTH-005).
//!
//! The LIVE leg walks the real candidate tree with the committed policy and asserts the gate is
//! born-blocking GREEN today: every pre-existing caller-supplied-authz-trust instance is frozen in
//! `frozen_dto_authz_trust_instances`, so the verdict is GREEN and no NEW instance is present. It
//! also asserts the FIXED iam/ports/policy-cedar-api/src/authz.rs PDP-port pattern is NOT flagged
//! (the GREEN reference) and a representative known instance (secrets/ports/kms-api) IS in the
//! baseline (matched by file+fn prefix since v2 keys include a body-hash suffix). The RED fixture
//! proves the gate genuinely FAILS on a NEW function that trusts a forged authorization DTO with no
//! PDP call; the GREEN fixture proves an ensure_authorized PDP handler is tolerated even when it
//! also reads the DTO. NOTE: verify_principal is AUTHN not AUTHZ and is absent from
//! pdp_decision_idents since v2; GREEN_PDP_BACKED uses ensure_authorized as the PDP call.
//!
//! ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ci_caller_supplied_authorization::{
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

const POLICY_REL: &str = "ci/facade/caller-supplied-authorization/dto-authz-trust-policy.json";

fn load_committed_policy(root: &Path) -> Value {
    let path = root.join(POLICY_REL);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read policy {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse policy {}: {e}", path.display()))
}

/// Test-only policy whose scan_roots point at a fixture directory written under a temp dir, so the
/// RED/GREEN exhibits are evaluated in isolation from the live tree.
/// NOTE: `verify_principal` is deliberately ABSENT from `pdp_decision_idents` since v2 — it is
/// AUTHN not AUTHZ and must not satisfy the PDP-call check.
fn fixture_policy(scan_root: &str) -> Value {
    json!({
        "gate_id": GATE_ID,
        "min_expected_functions": 0,
        "scan_roots": [scan_root],
        "excluded_dir_names": ["target", "third-party"],
        "authorization_dto_type_suffixes": ["Authorization"],
        "trigger_decision_field_idents": ["allowed_surfaces", "permitted_scopes", "caller_roles", "granted", "allowed_actions"],
        "decision_field_idents": ["allowed_surfaces", "decision_id"],
        "authorization_header_idents": [
            "x-authorization-decision-id",
            "x-authorization-surfaces",
            "x-authorization-principal-id",
            "x-authorization-tenant-id"
        ],
        "pdp_decision_idents": [".decide(", "ensure_authorized", "check_authz", "ensure_authz", "authorize_decision", "pdp_decide"],
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
        ci_caller_supplied_authorization::render_findings(&findings)
    );
}

// ---------------------------------------------------------------------------
// Scan-root derivation: the coverage the gate CLAIMS must be the coverage it HAS.
// ---------------------------------------------------------------------------

/// Recompute the expected scan-root set straight from the capability registry, independently of
/// `ci-scan-root-derivation-adapters`. An independent reading here is the review-visible contract:
/// if the resolver and this test ever disagree about what the registry means, that disagreement is
/// the finding.
fn expected_roots_from_registry(root: &Path) -> (BTreeSet<String>, BTreeSet<String>) {
    let text = std::fs::read_to_string(root.join("governance/capability-registry.json"))
        .expect("read capability registry");
    let registry: Value = serde_json::from_str(&text).expect("parse capability registry");
    let mut scanned: BTreeSet<String> = BTreeSet::new();
    let mut pending: BTreeSet<String> = BTreeSet::new();

    for capability in registry["capabilities"].as_array().expect("capabilities") {
        let name = capability["name"]
            .as_str()
            .expect("capability name")
            .to_owned();
        let materialized = capability["absorbs_current_dirs"]
            .as_array()
            .is_some_and(|dirs| !dirs.is_empty());
        let on_disk = root.join(&name).is_dir();
        assert_eq!(
            materialized, on_disk,
            "capability `{name}`: the registry's materialization record and the tree disagree — \
             either the registry row is stale or the directory was deleted without retiring it"
        );
        if on_disk {
            scanned.insert(name);
        } else {
            pending.insert(name);
        }
    }

    for meta in registry["meta_directories"]
        .as_array()
        .expect("meta_directories")
    {
        let dir = meta["dir"]
            .as_str()
            .expect("meta dir")
            .trim_end_matches('/');
        // third-party/ holds reindeer-vendored UPSTREAM sources; a first-party authorization gate
        // must not report findings nobody in this repository can fix.
        if dir == "third-party" {
            continue;
        }
        if root.join(dir).is_dir() {
            scanned.insert(dir.to_owned());
        } else {
            pending.insert(dir.to_owned());
        }
    }

    // Legacy roots are NOT in the closed registry and cannot be derived. They are enumerated once
    // for the whole fleet in ci/adapters/scan-root-derivation, each with a written deletion
    // condition, and shrink to nothing as the ADR-0562 moves drain them.
    for legacy in ["oya", "libs", "tools", "infra"] {
        assert!(
            root.join(legacy).is_dir(),
            "legacy root `{legacy}/` has drained — delete its LEGACY_ROOTS entry in \
             ci/adapters/scan-root-derivation (the resolver already fails closed on this)"
        );
        scanned.insert(legacy.to_owned());
    }

    (scanned, pending)
}

/// The structural broken-scan guard, and a SET rather than a count. `min_expected_functions` cannot
/// tell a collapsed scan from a legitimately smaller corpus; this can, and it names the root.
#[test]
fn walked_roots_are_exactly_the_registry_derived_set() {
    let root = repo_root();
    let policy = load_committed_policy(&root);
    let observed = collect_instances(&root, &policy).expect("collect over live tree");

    let walked: BTreeSet<String> = observed["scan_roots"]
        .as_array()
        .expect("observed scan_roots")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    eprintln!(
        "{GATE_ID}: walked {} roots, {} functions scanned",
        walked.len(),
        observed["functions_scanned"].as_u64().unwrap_or_default()
    );
    let (expected, _) = expected_roots_from_registry(&root);

    assert_eq!(
        walked, expected,
        "the roots this gate WALKED must equal the roots the capability registry says exist. A \
         capability missing from the left side is a blind spot the gate would report GREEN over; a \
         root missing from the right side means the derivation drifted from the registry."
    );
    assert_eq!(
        observed["scan_root_source"].as_str(),
        Some("capability-registry"),
        "the live gate must run in derived mode; the explicit-array mode exists only for adopting \
         repositories with no capability registry"
    );
}

/// EVERY registered capability is either scanned or frozen as pending. Nothing in the closed
/// registry may be silently outside this gate's reach.
#[test]
fn no_registered_capability_is_unaccounted_for() {
    let root = repo_root();
    let policy = load_committed_policy(&root);
    let observed = collect_instances(&root, &policy).expect("collect over live tree");
    let text = std::fs::read_to_string(root.join("governance/capability-registry.json"))
        .expect("read capability registry");
    let registry: Value = serde_json::from_str(&text).expect("parse capability registry");

    let covered: BTreeSet<String> = observed["scan_roots"]
        .as_array()
        .into_iter()
        .chain(observed["pending_roots"].as_array())
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();

    let unaccounted: Vec<String> = registry["capabilities"]
        .as_array()
        .expect("capabilities")
        .iter()
        .filter_map(|c| c["name"].as_str())
        .filter(|name| !covered.contains(*name))
        .map(str::to_owned)
        .collect();

    assert!(
        unaccounted.is_empty(),
        "registered capabilities neither scanned nor frozen pending: {unaccounted:?}"
    );
}

/// Pending roots are frozen TWO-SIDED. A newly registered capability with no directory is
/// born-blocking (it is not in the frozen set); a pending root that LANDS is blocking until it is
/// struck from the frozen set in the same change that makes it real.
#[test]
fn pending_roots_equal_the_frozen_set_exactly() {
    let root = repo_root();
    let policy = load_committed_policy(&root);
    let observed = collect_instances(&root, &policy).expect("collect over live tree");

    let measured: BTreeSet<String> = observed["pending_roots"]
        .as_array()
        .expect("observed pending_roots")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    let frozen: BTreeSet<String> = policy["_pending_scan_roots"]
        .as_array()
        .expect("policy _pending_scan_roots")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();

    assert_eq!(
        measured, frozen,
        "pending scan roots must equal the frozen set EXACTLY. A key only on the left is a newly \
         registered capability with no directory — freeze it deliberately. A key only on the right \
         has LANDED and is now scanned — strike it from _pending_scan_roots in this change."
    );

    let (_, expected_pending) = expected_roots_from_registry(&root);
    assert_eq!(
        measured, expected_pending,
        "the resolver and this test's independent registry reading must agree on what is pending"
    );
}

/// The rule, on the real tree: a DECLARED root that is ABSENT is an error naming the path, never
/// the `Err(NotFound) => Ok(())` that used to sit in the directory walk.
#[test]
fn red_declared_but_absent_root_fails_closed_naming_the_path() {
    let root = repo_root();
    let mut policy = fixture_policy("ci");
    policy["scan_roots"] = json!(["ci", "a-root-that-does-not-exist"]);
    let error = collect_instances(&root, &policy).expect_err("an absent declared root must fail");
    assert!(
        error.to_string().contains("a-root-that-does-not-exist"),
        "the error must NAME the path: {error}"
    );
}

/// The pre-change policy declared `base`, `cloud` and `policy`, none of which exist, and omitted the
/// legacy root `infra/` entirely. Pin that this shape is rejected today so the drift cannot return.
#[test]
fn red_the_pre_change_declared_root_list_is_rejected_today() {
    let root = repo_root();
    let mut policy = fixture_policy("ci");
    policy["scan_roots"] = json!([
        "app",
        "audit",
        "base",
        "billing",
        "build",
        "cell",
        "ci",
        "cloud",
        "comms",
        "compliance",
        "compute",
        "console",
        "data",
        "flags",
        "gateway",
        "governance",
        "iac",
        "iam",
        "intelligence",
        "k8s",
        "kernel",
        "libs",
        "marketplace",
        "messaging",
        "network",
        "observability",
        "os",
        "oya",
        "policy",
        "secrets",
        "storage",
        "tenancy",
        "tools",
        "workflow"
    ]);
    let error = collect_instances(&root, &policy).expect_err("three of these roots are dead");
    let message = error.to_string();
    assert!(
        ["base", "cloud", "policy"]
            .iter()
            .any(|dead| message.contains(&format!("`{dead}`"))),
        "expected one of the three dead roots to be named; got {message}"
    );
}

/// A policy declaring NEITHER form is malformed — the two forms are exclusive alternatives, not an
/// optional field that can quietly resolve to an empty scan.
#[test]
fn a_policy_declaring_no_scan_roots_at_all_is_malformed() {
    let mut policy = fixture_policy("ci");
    policy.as_object_mut().expect("object").remove("scan_roots");
    let findings = evaluate_keyed(&policy, &json!({ "functions_scanned": 0, "instances": [] }));
    assert!(
        findings.iter().any(|f| f.code == "DAT-POLICY-MALFORMED"),
        "a policy with no scan-root declaration must fail closed"
    );
}

#[test]
fn baseline_is_nonempty_and_covers_a_known_instance() {
    let root = repo_root();
    let policy = load_committed_policy(&root);
    let baseline: Vec<String> = policy
        .get("frozen_dto_authz_trust_instances")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    assert!(
        baseline.len() >= 40,
        "expected the frozen baseline to enumerate the known instances the review found; got {}",
        baseline.len()
    );
    // v2 keys have the format `<file>#<fn>:<body_hash>` — match by `<file>#<fn>:` prefix so the
    // check is stable across body edits that change the hash but not the identity. These are
    // confirmed caller-supplied-authz-trust instances that REMAIN un-remediated on the dev tip the
    // gate was re-baselined against (the compute VM/K8s/Functions instances plus the secrets/kms-api,
    // tenancy/api, network/lb, audit, observability and finops `validate_authorization` instances were
    // remediated by the AUTH-005 and G2 PRs — they legitimately dropped from the shrink-only baseline
    // and are no longer expected here).
    for expected_prefix in [
        "cell/ports/region/src/lib.rs#validate_authorization:",
        "iam/ports/cloud-api/src/lib.rs#validate_authorization:",
        "storage/ports/object-api/src/lib.rs#validate_authorization:",
        "data/ports/ontology-api/src/lib.rs#validate_authorization:",
        "app/application/ports/workspace-chat-api/src/lib.rs#validate_authorization:",
    ] {
        assert!(
            baseline.iter().any(|k| k.starts_with(expected_prefix)),
            "expected confirmed instance with prefix `{expected_prefix}` in the frozen baseline"
        );
    }
}

/// `publish_handler` is a split-decision false positive, not frozen authz debt: its private
/// router-only path reaches the required PDP through `enforce_publish_authz` before any mutation.
/// Keep the exact audited body in the curated allowlist so a later `--allow-new` run cannot quietly
/// re-grandfather it as unresolved caller-supplied authorization trust.
#[test]
fn pdp_dominated_publish_handler_is_curated_not_frozen() {
    const PREFIX: &str = "iam/ports/policy-cedar-api/src/rest/mod.rs#publish_handler:";
    const AUDITED_KEY: &str = "iam/ports/policy-cedar-api/src/rest/mod.rs#publish_handler:5bc02232";

    let root = repo_root();
    let policy = load_committed_policy(&root);
    let frozen = policy
        .get("frozen_dto_authz_trust_instances")
        .and_then(Value::as_array)
        .expect("frozen baseline array");
    assert!(
        frozen
            .iter()
            .filter_map(Value::as_str)
            .all(|key| !key.starts_with(PREFIX)),
        "the PDP-dominated publish handler must never return to the frozen debt baseline"
    );

    let curated = policy
        .get("split_decision_allowlist")
        .and_then(Value::as_array)
        .expect("curated split-decision array");
    let matching = curated
        .iter()
        .filter(|entry| {
            entry
                .get("key")
                .and_then(Value::as_str)
                .is_some_and(|key| key.starts_with(PREFIX))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matching.len(),
        1,
        "publish_handler must have one exact, non-launderable split-decision audit entry"
    );
    assert_eq!(
        matching[0].get("key").and_then(Value::as_str),
        Some(AUDITED_KEY),
        "a body-hash change requires a fresh call-graph audit"
    );

    let justification = matching[0]
        .get("justification")
        .and_then(Value::as_str)
        .expect("split-decision justification");
    for required_proof in [
        "enforce_publish_authz",
        "ensure_authorized",
        "before mutation",
        "cannot convert a PDP deny into allow",
    ] {
        assert!(
            justification.contains(required_proof),
            "publish_handler justification must preserve proof token {required_proof:?}"
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
        i.get("file").and_then(Value::as_str) == Some("iam/ports/policy-cedar-api/src/authz.rs")
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
    let dir = std::env::temp_dir().join(format!("dto-authz-trust-{test}-{}", std::process::id()));
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
    assert!(
        report
            .violations
            .contains("DAT-CALLER-SUPPLIED-AUTHZ-TRUST")
    );
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

    // First collect with empty baseline to learn the actual v2 key (file#fn:<body_hash>).
    let observed_initial = collect_instances(&dir, &policy).expect("collect initial");
    let actual_key = observed_initial
        .get("instances")
        .and_then(Value::as_array)
        .and_then(|arr| {
            arr.iter()
                .find(|i| i.get("fn").and_then(Value::as_str) == Some("validate_authorization"))
        })
        .and_then(|i| i.get("key").and_then(Value::as_str))
        .expect("expected validate_authorization instance with key")
        .to_owned();

    // Baseline using the actual v2 key (includes body-hash suffix).
    policy["frozen_dto_authz_trust_instances"] = json!([actual_key]);
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
