// ADR-0565 cloud-ci-no-graphql-without-adr: born-blocking self-test over TODAY's real candidate
// tree plus hermetic RED/GREEN fixtures. It asserts:
//   * the LIVE post-drop worktree is born-blocking GREEN — the real candidate-tree collector finds
//     zero GraphQL library (in ANY Cargo.toml), zero forbidden crate in Cargo.lock, and zero
//     .graphql/.graphqls/.gql/.gqls/.sdl schema file or build-graph glob, so the frozen baseline is
//     EMPTY and any NEW GraphQL artifact fails closed on arrival.
//   * a RED fixture (a synthetic candidate tree whose member Cargo.toml adds `async-graphql`) makes
//     the gate FAIL — proving it genuinely catches a reintroduced lib, not an always-pass stub.
//   * a RED fixture (a synthetic candidate tree with a new `.graphql` schema file, no ADR ref)
//     makes the gate FAIL.
//   * a RED fixture (a synthetic candidate tree with a BUCK `**/*.graphql` srcs glob) makes the gate
//     FAIL before a schema can become a normal build input.
//   * a RED fixture (`async-graphql` smuggled via a `[workspace.dependencies]` rename + a member
//     `{ workspace = true }` inheritance) makes the gate FAIL — the workspace-rename backdoor.
//   * a RED fixture (`async-graphql` in a NON-member `services/gw/Cargo.toml`) makes the gate FAIL —
//     the LIB leg now scans EVERY Cargo.toml in the tree, not only resolved members.
//   * a RED fixture (a `.graphqls` SDL file) makes the gate FAIL — the canonical SDL extension.
//   * a RED fixture (a forbidden crate present ONLY in Cargo.lock) makes the gate FAIL — the
//     transitive-reintroduction catch.
//   * a RED fixture (citing a fabricated `# ADR-9999` token, no allowlist/validation) makes the gate
//     FAIL — the escape-hatch is NOT a bare-token backdoor.
//   * a GREEN fixture (forbidden artifacts citing an ALLOWLISTED id backed by a real Accepted
//     reversing ADR in the temp tree's docs/decisions) makes the gate PASS — proving the escape-hatch
//     is live AND that it requires real authorization, not a no-op nor a backdoor.
//   * the committed policy gate_id matches the crate contract.
// The fixtures drive the REAL collector (the only I/O) end-to-end over a temp workspace, so the
// collector's hermetic fs scan + the pure evaluator are both exercised.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ci_graphql_usage_policy::{
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
    root.join("ci/facade/graphql-usage-policy")
}

fn load_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
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
    let members = observed["workspace_members_found"]
        .as_u64()
        .expect("member census");
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
fn live_owned_stack_has_no_active_graphql_layer_vocabulary() {
    // The policy evaluator already blocks real GraphQL libraries and schema files. This live
    // regression scan closes the drift hole that let owned-stack scaffolds/docs keep the retired
    // layer vocabulary even while the policy gate was green.
    let root = repo_root();
    let scan_roots = [
        "cell/cell-rebalancer/ARCH.md",
        "comms/facade/contact-center-voice-routing",
        "data/facade/pipeline-lineage-replay-service",
        "docs/architecture/product-graph.md",
        "docs/automation/service-map-spec.md",
        "docs/GLOSSARY.md",
        "docs/standards",
        "docs/templates/microservice-template.md",
        "governance/check/statelessness",
        "marketplace/core/doc-set-scaffold",
        "marketplace/facade/dev-cli/src",
        "marketplace/observability/slos",
        "marketplace",
        "data/ontology/decisions/ADR-ONT-001-rdf-shape-vs-property-graph-storage.md",
        "oya/workplace-integration",
    ];
    let forbidden_needles = [
        "Layer::Graphql",
        "Self::Graphql",
        "ArchitectureLayer::Graphql",
        "Graphql,",
        "\"graphql\"",
        "`graphql`",
        "-graphql",
        ".graphql",
        "rest, grpc, graphql",
        "cli/rest/grpc/graphql",
    ];

    let mut hits = Vec::new();
    for rel in scan_roots {
        collect_active_vocabulary_hits(&root.join(rel), rel, &forbidden_needles, &mut hits);
    }

    assert!(
        hits.is_empty(),
        "active owned-stack paths must not carry retired GraphQL layer vocabulary:\n{}",
        hits.join("\n")
    );
}

fn collect_active_vocabulary_hits(
    path: &Path,
    rel: &str,
    forbidden_needles: &[&str],
    hits: &mut Vec<String>,
) {
    if path.is_dir() {
        let mut entries = fs::read_dir(path)
            .unwrap_or_else(|error| {
                panic!("read active-vocabulary dir {}: {error}", path.display())
            })
            .map(|entry| entry.expect("read active-vocabulary dir entry").path())
            .collect::<Vec<_>>();
        entries.sort();
        for entry in entries {
            let name = entry
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if matches!(name, ".git" | "target" | "buck-out" | "BUCK") {
                continue;
            }
            let child_rel = format!("{rel}/{name}");
            collect_active_vocabulary_hits(&entry, &child_rel, forbidden_needles, hits);
        }
        return;
    }

    // A scan root that no longer resolves is NOT "clean" — it is a root that scans nothing and
    // reports GREEN over an empty set. `scan_roots` here is a Rust array literal, so the
    // //ci/facade/scan-root-liveness gate (which reads JSON policy files) structurally cannot see
    // it; this assertion is that gate's stand-in for this declaration site. A reorg move or a
    // retirement that empties a root must delete the root in the same PR.
    assert!(
        path.exists(),
        "dead active-vocabulary scan root {rel}: declared but resolves to no path, so it scans \
         nothing and reports green over an empty set — delete the root in the PR that removes it"
    );
    if !path.is_file() {
        return;
    }
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    for (line_idx, line) in text.lines().enumerate() {
        if let Some(needle) = forbidden_needles
            .iter()
            .find(|needle| line.contains(**needle))
        {
            hits.push(format!("{rel}:{} contains {needle}: {line}", line_idx + 1));
        }
    }
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

/// Write a file at `rel` (repo-relative to `root`), creating parent dirs.
fn write_file(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(&path, body).expect("write file");
}

/// Overwrite the temp root `Cargo.toml` with a custom body (e.g. to add `[workspace.dependencies]`).
fn write_root_manifest(root: &Path, body: &str) {
    fs::write(root.join("Cargo.toml"), body).expect("write root manifest");
}

/// Write a real Accepted ADR that reverses the forbidding ADR into the temp tree's `docs/decisions`,
/// so the gate's defense-in-depth validation resolves the cited id. The id `ADR-0800` is the test-only
/// authorizing decision.
fn write_authorizing_adr(root: &Path) {
    write_file(
        root,
        "docs/decisions/ADR-0800-reintroduce-graphql.md",
        "---\nid: ADR-0800\nstatus: Accepted\nsupersedes:\n  - ADR-0565\n---\n\n# ADR-0800: Reintroduce GraphQL\n\nThis decision reverses ADR-0565 and readmits a single generated GraphQL surface.\n",
    );
}

/// A policy with the member floor lowered to 1 so the synthetic single-member fixtures meet it
/// (the committed policy's 100 floor is a live-tree guard, not a fixture constraint). The policy is
/// loaded from the REAL repo root (the committed file lives there, not in the temp fixture tree);
/// everything else mirrors the committed policy. `authorizing_adrs` stays EMPTY by default (matching
/// the committed policy) — the GREEN escape-hatch fixture allowlists `ADR-0800` explicitly.
fn fixture_policy() -> Value {
    let mut p = committed_policy(&repo_root());
    p["min_expected_workspace_members"] = Value::from(1u64);
    p
}

/// As [`fixture_policy`] but allowlisting the given authorizing ADR ids — the policy a repo ships
/// after an Accepted reversing ADR is enumerated.
fn fixture_policy_allowing(ids: &[&str]) -> Value {
    let mut p = fixture_policy();
    p["authorizing_adrs"] = serde_json::json!(ids);
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
    assert!(
        f.detail.contains("ADR-0565"),
        "remediation must name the forbidding ADR: {f:?}"
    );
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
    assert!(
        f.key.ends_with("schema.graphql"),
        "the finding key must name the schema path: {f:?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_buck_graphql_schema_glob() {
    // RED, hermetic: a synthetic candidate tree with no schema file yet, but a BUCK source glob that
    // would keep admitting GraphQL schemas as normal build inputs.
    let root = temp_repo();
    write_member(
        &root,
        "analytics-api",
        "[package]\nname = \"analytics-api\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n",
        None,
    );
    write_file(
        &root,
        "crates/analytics-api/BUCK",
        "rust_library(\n    name = \"analytics-api\",\n    srcs = glob([\"src/**/*.rs\", \"**/*.graphql\"]),\n)\n",
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    let f = findings
        .iter()
        .find(|f| f.code == "NGQL-BUILD-GRAPH-SCHEMA-GLOB")
        .unwrap_or_else(|| panic!("a BUCK GraphQL schema glob must be RED: {findings:#?}"));
    assert!(
        f.key.contains("crates/analytics-api/BUCK:3:**/*.graphql"),
        "the finding key must name the BUCK path, line, and glob: {f:?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn green_fixture_allowlisted_and_validated_adr() {
    // GREEN, hermetic: the SAME forbidden artifacts (async-graphql dep + a .graphql file) citing an
    // authorizing id (ADR-0800) that is BOTH (1) in the policy `authorizing_adrs` allowlist AND
    // (2) backed by a real Accepted ADR in the temp tree's docs/decisions that reverses ADR-0565. The
    // escape-hatch admits them; the gate PASSES. Proves the escape is live AND requires REAL
    // authorization — not a no-op, and not a bare-token backdoor.
    let root = temp_repo();
    write_authorizing_adr(&root);
    write_member(
        &root,
        "studio-graphql",
        "# Reintroduced per ADR-0800 (reverses ADR-0565).\n[package]\nname = \"studio-graphql\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nasync-graphql = \"7\"\n",
        Some("# Authorized by ADR-0800 (reverses ADR-0565).\ntype Query { ok: Boolean }\n"),
    );

    let policy = fixture_policy_allowing(&["ADR-0800"]);
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    assert!(
        !findings
            .iter()
            .any(|f| f.code == "NGQL-FORBIDDEN-LIB" || f.code == "NGQL-SCHEMA-FILE"),
        "allowlisted+validated-ADR GraphQL artifacts must be allowed: {findings:#?}"
    );
    assert_eq!(
        evaluate(&policy, &observed).verdict,
        Verdict::Green,
        "an authorized (allowlisted + validated ADR) change must be GREEN"
    );

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_fabricated_adr_token_does_not_launder() {
    // RED, hermetic (CRITICAL backdoor): an async-graphql dep with a bare fabricated `# ADR-9999`
    // comment and a `.graphql` file with a sibling `.adr` marker citing `ADR-1234` — NEITHER id is
    // allowlisted nor backed by a real Accepted reversing ADR. The gate must be RED.
    let root = temp_repo();
    write_member(
        &root,
        "studio-graphql",
        "# ADR-9999\n[package]\nname = \"studio-graphql\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nasync-graphql = \"7\"\n",
        None,
    );
    // A real api.graphql + a sibling api.graphql.adr citing ADR-1234 — still not authorized.
    write_file(
        &root,
        "crates/studio-graphql/api.graphql",
        "type Query { ok: Boolean }\n",
    );
    write_file(&root, "crates/studio-graphql/api.graphql.adr", "ADR-1234\n");

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    assert!(
        findings.iter().any(|f| f.code == "NGQL-FORBIDDEN-LIB"),
        "a fabricated ADR token must NOT launder a forbidden lib: {findings:#?}"
    );
    assert!(
        findings.iter().any(|f| f.code == "NGQL-SCHEMA-FILE"),
        "a sibling .adr marker citing an unvalidated id must NOT launder a schema file: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_workspace_dependencies_rename_smuggle() {
    // RED, hermetic (CRITICAL backdoor): async-graphql smuggled via a root [workspace.dependencies]
    // rename + a member `{ workspace = true }` inheritance. The collector must resolve the rename back
    // to the real name and the gate must be RED.
    let root = temp_repo();
    write_root_manifest(
        &root,
        "[workspace]\nresolver = \"2\"\nmembers = [\"crates/*\"]\n\n[workspace.dependencies]\ngqlrt = { package = \"async-graphql\", version = \"7\" }\n",
    );
    write_member(
        &root,
        "gw",
        "[package]\nname = \"gw\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ngqlrt = { workspace = true }\n",
        None,
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    assert!(
        findings
            .iter()
            .any(|f| f.code == "NGQL-FORBIDDEN-LIB" && f.key.ends_with("async-graphql")),
        "a [workspace.dependencies] rename must be denied on the real name: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_non_member_cargo_toml() {
    // RED, hermetic (MAJOR scope): async-graphql in a NON-member `services/gw/Cargo.toml` (the
    // workspace globs `crates/*`, so `services/gw` is NOT a resolved member). The LIB leg now scans
    // EVERY Cargo.toml in the tree, so the gate must be RED.
    let root = temp_repo();
    // A clean member so the census floor is met.
    write_member(
        &root,
        "core",
        "[package]\nname = \"core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n",
        None,
    );
    // The forbidden dep in a NON-member dir.
    write_file(
        &root,
        "services/gw/Cargo.toml",
        "[package]\nname = \"gw\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nasync-graphql = \"7\"\n",
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    assert!(
        findings
            .iter()
            .any(|f| f.code == "NGQL-FORBIDDEN-LIB" && f.key.starts_with("services/gw/Cargo.toml")),
        "a forbidden dep in a NON-member Cargo.toml must be RED: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_graphqls_sdl_extension() {
    // RED, hermetic (MAJOR ext): a `schema.graphqls` (the canonical GraphQL SDL extension) file.
    let root = temp_repo();
    write_member(
        &root,
        "analytics-api",
        "[package]\nname = \"analytics-api\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n",
        None,
    );
    write_file(
        &root,
        "crates/analytics-api/schema.graphqls",
        "type Query { ok: Boolean }\n",
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    assert!(
        findings
            .iter()
            .any(|f| f.code == "NGQL-SCHEMA-FILE" && f.key.ends_with("schema.graphqls")),
        "a .graphqls SDL file must be RED: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_transitive_crate_in_cargo_lock() {
    // RED, hermetic (MAJOR transitive): a forbidden crate present ONLY in Cargo.lock (no manifest
    // names it directly) must be RED — the transitive-reintroduction catch.
    let root = temp_repo();
    write_member(
        &root,
        "core",
        "[package]\nname = \"core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n",
        None,
    );
    write_file(
        &root,
        "Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"serde\"\nversion = \"1.0.150\"\n\n[[package]]\nname = \"async-graphql\"\nversion = \"7.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    let f = findings
        .iter()
        .find(|f| f.code == "NGQL-LOCK-FORBIDDEN")
        .unwrap_or_else(|| {
            panic!("a transitive forbidden crate in Cargo.lock must be RED: {findings:#?}")
        });
    assert_eq!(f.key, "Cargo.lock:async-graphql");
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

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
        findings
            .iter()
            .any(|f| f.code == "NGQL-FORBIDDEN-LIB" && f.key.ends_with("juniper")),
        "citing only the forbidding ADR must NOT self-launder: {findings:#?}"
    );

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_malformed_manifest_with_forbidden_crate() {
    // Fix 1 RED, hermetic: a Cargo.toml that FAILS to parse as valid TOML but contains the text
    // "async-graphql" must be caught by the raw-text fallback and the gate must be RED. This
    // prevents the fails-open vulnerability where a forbidden crate in an unparseable manifest
    // passes GREEN because the TOML parser returns an empty set.
    let root = temp_repo();
    // Write a clean member so the census floor is met.
    write_member(
        &root,
        "core",
        "[package]\nname = \"core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n",
        None,
    );
    // Write a malformed Cargo.toml containing `async-graphql` in a non-member path.
    write_file(
        &root,
        "services/broken/Cargo.toml",
        "THIS IS NOT VALID TOML [[[\nasync-graphql = \"broken\"\n",
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    assert!(
        findings.iter().any(|f| f.code == "NGQL-FORBIDDEN-LIB"),
        "malformed Cargo.toml with async-graphql must be RED (fail-closed raw fallback): {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_malformed_cargo_lock_with_forbidden_crate() {
    // Fix 1 RED, hermetic: a Cargo.lock that FAILS to parse as valid TOML but contains a
    // `name = "async-graphql"` line must be caught by the raw lock fallback and the gate must be RED.
    let root = temp_repo();
    write_member(
        &root,
        "core",
        "[package]\nname = \"core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n",
        None,
    );
    // Write a malformed Cargo.lock that contains the forbidden crate name.
    write_file(
        &root,
        "Cargo.lock",
        "version = BROKEN_NOT_TOML\n\n[[package]]\nname = \"async-graphql\"\nversion = \"7.0.0\"\n[[BROKEN\n",
    );

    let policy = fixture_policy();
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    assert!(
        findings.iter().any(|f| f.code == "NGQL-LOCK-FORBIDDEN"),
        "malformed Cargo.lock with async-graphql must be RED (fail-closed raw fallback): {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn red_fixture_adr_only_in_related_field_does_not_reverse() {
    // Fix 4 RED, hermetic: an escape-hatch ADR that lists the forbidding id only under `related:`
    // (not `supersedes:`/`amends:`/`reverses:`) must NOT validate as a reversal. The gate must
    // remain RED even if the ADR id is in the policy allowlist.
    let root = temp_repo();
    // Write an "authorizing" ADR that only lists ADR-0565 in `related:` + has body prose
    // containing "has not been superseded" — the structural supersedes check must reject this.
    write_file(
        &root,
        "docs/decisions/ADR-0800-not-a-reversal.md",
        "---\nid: ADR-0800\nstatus: Accepted\nrelated:\n  - ADR-0565\n---\n\n# ADR-0800\n\nADR-0565 has not been superseded by this decision.\n",
    );
    write_member(
        &root,
        "studio-graphql",
        "# Reintroduced per ADR-0800 (cites ADR-0565).\n[package]\nname = \"studio-graphql\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nasync-graphql = \"7\"\n",
        None,
    );

    // Even with ADR-0800 in the allowlist, it does not validate as a reversal.
    let policy = fixture_policy_allowing(&["ADR-0800"]);
    let observed = collect_graphql_artifacts(&root, &policy).expect("collect on temp tree");
    let findings = evaluate_keyed(&policy, &observed);

    assert!(
        findings.iter().any(|f| f.code == "NGQL-FORBIDDEN-LIB"),
        "ADR listed only in related: must NOT validate as reversal — gate must be RED: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);

    fs::remove_dir_all(&root).expect("remove temp repo");
}
