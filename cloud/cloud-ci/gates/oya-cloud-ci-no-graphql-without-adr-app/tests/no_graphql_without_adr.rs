// ADR-0565 cloud-ci-no-graphql-without-adr: born-blocking self-test over TODAY's real candidate
// tree plus hermetic RED/GREEN fixtures. It asserts:
//   * the LIVE post-drop worktree is born-blocking GREEN — the real candidate-tree collector finds
//     zero GraphQL library (in any member Cargo.toml) and zero .graphql/.gql/.sdl schema file, so
//     the frozen baseline is EMPTY and any NEW GraphQL artifact fails closed on arrival.
//   * a RED fixture (a synthetic candidate tree whose member Cargo.toml adds `async-graphql`) makes
//     the gate FAIL — proving it genuinely catches a reintroduced lib, not an always-pass stub.
//   * a RED fixture (a synthetic candidate tree with a new `.graphql` schema file, no ADR ref)
//     makes the gate FAIL.
//   * a GREEN fixture (the same forbidden artifacts but citing an authorizing/reversing ADR) makes
//     the gate PASS — proving the ADR escape-hatch is live, not a no-op.
//   * the committed policy gate_id matches the crate contract.
// The fixtures drive the REAL collector (the only I/O) end-to-end over a temp workspace, so the
// collector's hermetic fs scan + the pure evaluator are both exercised.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use oya_cloud_ci_no_graphql_without_adr_app::{
    Verdict, collect_graphql_artifacts, evaluate, evaluate_keyed,
};
use serde_json::Value;

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Verbatim from the sibling gates so collection runs from the
/// resolved repo root, not the `cargo test` CWD.
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

fn gate_dir(root: &Path) -> PathBuf {
    root.join("cloud/cloud-ci/gates/oya-cloud-ci-no-graphql-without-adr-app")
}

fn load_json(path: &Path) -> Value {
    let text =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn committed_policy(root: &Path) -> Value {
    load_json(&gate_dir(root).join("no-graphql-without-adr-policy.json"))
}

#[test]
fn live_workspace_is_born_blocking_green_zero_graphql() {
    // GREEN, end-to-end against the LIVE post-drop worktree: run the real hermetic candidate-tree
    // collector and assert zero GraphQL artifact. This is the live-tree PASS the gate must produce
    // and proves the frozen baseline is EMPTY.
    let root = repo_root();
    let policy = committed_policy(&root);

    let observed = collect_graphql_artifacts(&root, &policy)
        .expect("read-only candidate-tree collection should succeed on the live tree");

    // The member census floor must be met (a broken CWD / member glob would otherwise be a silent
    // false-green).
    let members = observed["workspace_members_found"].as_u64().expect("member census");
    let floor = policy["min_expected_workspace_members"]
        .as_u64()
        .expect("policy member floor");
    assert!(
        members >= floor,
        "the live workspace should carry at least the policy member floor ({floor}); got {members}"
    );

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "no-graphql-without-adr is born-blocking GREEN on the live tree (zero GraphQL); got {} finding(s):\n{}",
        findings.len(),
        findings
            .iter()
            .map(|f| format!("  {} {}: {}", f.code, f.key, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);

    let schema_files = observed["schema_files"].as_array().expect("schema_files");
    eprintln!(
        "NO-GRAPHQL-WITHOUT-ADR live: members={members} graphql_schema_files={} (born-blocking green; EMPTY frozen baseline — ADR-0565)",
        schema_files.len()
    );
}

#[test]
fn policy_gate_id_matches_the_crate_contract() {
    let root = repo_root();
    let policy = committed_policy(&root);
    assert_eq!(
        policy["gate_id"].as_str(),
        Some("cloud-ci-no-graphql-without-adr")
    );
}

// --- Temp candidate-tree fixtures (drive the REAL collector hermetically) ---

static TEMP_REPO_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Build a minimal candidate-tree repo with a virtual workspace root + one member crate. The
/// caller passes the member's `Cargo.toml` body and an optional `.graphql` file body so each
/// fixture exercises exactly one reintroduction vector.
fn temp_repo() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let counter = TEMP_REPO_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "oya-no-graphql-gate-test-{}-{nanos}-{counter}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create temp repo");
    // A glob-only virtual workspace root (ADR-0538 shape) the member resolver understands.
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = [\"crates/*\"]\n",
    )
    .expect("write root manifest");
    root
}

/// Write a member crate at `crates/<name>/Cargo.toml` with `manifest_body` and, if some, a
/// `.graphql` schema file at `crates/<name>/schema.graphql` with `schema_body`.
fn write_member(root: &Path, name: &str, manifest_body: &str, schema_body: Option<&str>) {
    let dir = root.join("crates").join(name);
    fs::create_dir_all(dir.join("src")).expect("create member dir");
    fs::write(dir.join("Cargo.toml"), manifest_body).expect("write member manifest");
    fs::write(dir.join("src").join("lib.rs"), "// member\n").expect("write member src");
    if let Some(body) = schema_body {
        fs::write(dir.join("schema.graphql"), body).expect("write schema file");
    }
}

/// A policy with the member floor lowered to 1 so the synthetic single-member fixtures meet it
/// (the committed policy's 100 floor is a live-tree guard, not a fixture constraint). The policy is
/// loaded from the REAL repo root (the committed file lives there, not in the temp fixture tree);
/// everything else mirrors the committed policy.
fn fixture_policy() -> Value {
    let mut p = committed_policy(&repo_root());
    p["min_expected_workspace_members"] = Value::from(1u64);
    p
}

#[test]
fn red_fixture_cargo_toml_adds_async_graphql() {
    // RED, hermetic: a synthetic candidate tree whose member Cargo.toml adds `async-graphql` with
    // NO ADR reference. The real collector resolves the member + parses its deps; the gate FAILS.
    let root = temp_repo();
    write_member(
        &root,
        "studio-graphql",
        "[package]\nname = \"studio-graphql\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nasync-graphql = \"7\"\nserde = \"1\"\n",
        None,
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    let f = findings
        .iter()
        .find(|f| f.code == "NGQL-FORBIDDEN-LIB")
        .unwrap_or_else(|| panic!("a Cargo.toml adding async-graphql must be RED: {findings:#?}"));
    assert!(
        f.key.ends_with("Cargo.toml:async-graphql"),
        "the finding key must name the manifest + crate: {f:?}"
    );
    assert!(f.detail.contains("ADR-0565"), "remediation must name the forbidding ADR: {f:?}");
    assert_eq!(
        evaluate(&policy, &observed).verdict,
        Verdict::Red,
        "a reintroduced GraphQL lib must be RED"
    );

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_new_graphql_schema_file() {
    // RED, hermetic: a synthetic candidate tree with a new `.graphql` schema file, NO ADR reference.
    let root = temp_repo();
    write_member(
        &root,
        "analytics-api",
        "[package]\nname = \"analytics-api\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n",
        Some("type Query { workflowExecutionDashboard: String }\n"),
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    let f = findings
        .iter()
        .find(|f| f.code == "NGQL-SCHEMA-FILE")
        .unwrap_or_else(|| panic!("a new .graphql file must be RED: {findings:#?}"));
    assert!(f.key.ends_with("schema.graphql"), "the finding key must name the schema path: {f:?}");
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn green_fixture_adr_referenced_lib_and_schema() {
    // GREEN, hermetic: the SAME forbidden artifacts (async-graphql dep + a .graphql file) but each
    // citing an authorizing/reversing ADR (a DIFFERENT ADR than the forbidding ADR-0565). The
    // escape-hatch admits them; the gate PASSES. Proves the escape is live, not a no-op, and that
    // the gate genuinely distinguishes an authorized change from a bare reintroduction.
    let root = temp_repo();
    write_member(
        &root,
        "studio-graphql",
        "# Reintroduced per ADR-0700 (reverses ADR-0565).\n[package]\nname = \"studio-graphql\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nasync-graphql = \"7\"\n",
        Some("# Authorized by ADR-0700 (reverses ADR-0565).\ntype Query { ok: Boolean }\n"),
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    assert!(
        !findings.iter().any(|f| f.code == "NGQL-FORBIDDEN-LIB" || f.code == "NGQL-SCHEMA-FILE"),
        "ADR-referenced GraphQL artifacts must be allowed: {findings:#?}"
    );
    assert_eq!(
        evaluate(&policy, &observed).verdict,
        Verdict::Green,
        "an authorized (ADR-referenced) change must be GREEN"
    );

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn green_fixture_clean_member_tree() {
    // GREEN, hermetic: a member with no GraphQL lib and no schema file — the post-drop clean shape.
    let root = temp_repo();
    write_member(
        &root,
        "iam-pdp",
        "[package]\nname = \"iam-pdp\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = \"1\"\ntokio = \"1\"\n",
        None,
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    assert_eq!(
        evaluate(&policy, &observed).verdict,
        Verdict::Green,
        "a clean member tree must be GREEN: {:#?}",
        evaluate_keyed(&policy, &observed)
    );

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_mentioning_only_forbidding_adr_does_not_self_launder() {
    // RED, hermetic: a manifest that cites ONLY the forbidding ADR (ADR-0565 — the rule it would be
    // violating) must NOT escape. The escape-hatch requires citing a DIFFERENT (reversing) ADR.
    let root = temp_repo();
    write_member(
        &root,
        "sneaky-graphql",
        "# This file mentions ADR-0565 but does not reverse it.\n[package]\nname = \"sneaky-graphql\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\njuniper = \"0.16\"\n",
        None,
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.iter().any(|f| f.code == "NGQL-FORBIDDEN-LIB" && f.key.ends_with("juniper")),
        "citing only the forbidding ADR must NOT self-launder: {findings:#?}"
    );

    fs::remove_dir_all(&root).expect("remove temp repo");
}
