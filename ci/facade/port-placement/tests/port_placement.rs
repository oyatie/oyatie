// ADR-0570 cloud-ci-port-placement: born-advisory self-test over TODAY's real */adapters/* crates.
// The test collects each adapter crate's `pub trait` definitions from the live tree and asserts:
//   * the live corpus is born-advisory GREEN against the frozen baseline — every storage-port trait
//     defined in an adapter today is either captured in port-placement-baseline.json or
//     allowlisted, so any NEW port-in-adapter (beyond the baseline) fails closed on arrival;
//   * the scan actually found the workspace member census (PP-EMPTY-SCAN floor is met), so a broken
//     glob/collect cannot pass as a silent false-green;
//   * the committed policy gate_id matches the crate contract;
//   * the frozen baseline is not stale (no PP-STALE-BASELINE) — it captures exactly the live set.
// Filesystem RED/GREEN fixtures (materialized under the OS temp dir at runtime) prove the collector
// resolves real manifests + src trees and that the suffix heuristic + baseline subtraction behave
// end-to-end on disk. Pure-unit RED fixtures live in src/lib.rs.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use ci_port_placement::{Verdict, collect_port_traits, evaluate, evaluate_keyed};
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
    root.join("ci/facade/port-placement")
}

fn load_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

#[test]
fn live_corpus_is_born_advisory_green_against_frozen_baseline() {
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("port-placement-policy.json"));
    let baseline = load_json(&gate_dir(&root).join("port-placement-baseline.json"));

    let observed = collect_port_traits(&root, &policy)
        .expect("read-only port-trait collection should not need temp files or cleanup");
    let member_count = observed["member_crates_found"].as_u64().expect("count");
    let floor = policy["min_expected_member_crates"]
        .as_u64()
        .expect("policy floor");
    assert!(
        member_count >= floor,
        "the live tree should carry at least the policy member-census floor ({floor}); got {member_count}"
    );

    let findings = evaluate_keyed(&policy, &baseline, &observed);

    // Born-advisory: NO finding allowed on the live corpus (the frozen baseline absorbs the
    // pre-existing violations; a NEW one beyond it would surface here as PP-PORT-IN-ADAPTER, and a
    // relocated one as PP-STALE-BASELINE). Surface the full set if any appears.
    assert!(
        findings.is_empty(),
        "port-placement is born-advisory green on the live corpus (baseline absorbs pre-existing); got {} finding(s):\n{}",
        findings.len(),
        findings
            .iter()
            .map(|f| format!("  {} {}: {}", f.code, f.key, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(
        evaluate(&policy, &baseline, &observed).verdict,
        Verdict::Green
    );

    eprintln!(
        "PORT-PLACEMENT live corpus: members={member_count} findings=0 (born-advisory green vs frozen baseline)"
    );
}

#[test]
fn policy_gate_id_matches_the_crate_contract() {
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("port-placement-policy.json"));
    assert_eq!(policy["gate_id"].as_str(), Some("cloud-ci-port-placement"));
}

#[test]
fn frozen_baseline_captures_a_nonempty_set_of_existing_violations() {
    // After #116, billing is clean but the live corpus still carries pre-existing storage-port
    // traits in adapter crates (tenant-rbac / session / secret-provider / kms-domain-repo /
    // payroll / hr). The frozen baseline MUST be non-empty (do NOT assume zero), and every entry
    // must have {member_path, trait} keys.
    let root = repo_root();
    let baseline = load_json(&gate_dir(&root).join("port-placement-baseline.json"));
    let entries = baseline["baseline"].as_array().expect("baseline array");
    assert!(
        !entries.is_empty(),
        "the frozen baseline must capture the pre-existing port-in-adapter violations (not zero)"
    );
    for entry in entries {
        assert!(
            entry["member_path"].as_str().is_some(),
            "each baseline entry must have a `member_path` key: {entry}"
        );
        assert!(
            entry["trait"].as_str().is_some(),
            "each baseline entry must have a `trait` key: {entry}"
        );
    }
}

// ---------------------------------------------------------------------------
// Filesystem fixture repos: prove collect_port_traits resolves real manifests + src trees from
// disk and that the suffix heuristic + baseline subtraction behave end-to-end. Materialized under
// the OS temp dir at runtime (mirroring the kernel-purity test pattern) — a committed nested
// [package] Cargo.toml would confuse cargo workspace resolution; a runtime temp tree is hermetic
// and self-cleaning.
// ---------------------------------------------------------------------------

use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempRepo {
    root: PathBuf,
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn new_temp_repo() -> TempRepo {
    let unique = format!(
        "oya-pp-fixture-{}-{}",
        std::process::id(),
        FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let root = std::env::temp_dir().join(unique);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture root");
    TempRepo { root }
}

fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create fixture parent dir");
    }
    std::fs::write(&path, contents).expect("write fixture file");
}

/// Root workspace manifest covering a capability tree with both adapters/ and core/ layers.
fn write_root_manifest(root: &Path) {
    write_file(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"cap/adapters/*\", \"cap/core/*\"]\nresolver = \"2\"\n",
    );
}

/// The committed policy with the member-count floor lowered so the small fixture repos do not trip
/// PP-EMPTY-SCAN (the floor guards the LIVE corpus, not fixtures).
fn fixture_policy() -> Value {
    let live = repo_root();
    let mut policy = load_json(&gate_dir(&live).join("port-placement-policy.json"));
    policy["min_expected_member_crates"] = Value::from(1u64);
    policy
}

fn member(root: &Path, member_path: &str, src_rel: &str, src: &str) {
    write_file(
        root,
        &format!("{member_path}/Cargo.toml"),
        &format!(
            "[package]\nname = \"{}\"\nversion = \"0.0.0\"\n",
            member_path.rsplit('/').next().unwrap()
        ),
    );
    write_file(root, &format!("{member_path}/{src_rel}"), src);
}

#[test]
fn red_repo_fixture_surfaces_port_in_adapter_from_disk() {
    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);

    // An adapter crate that WRONGLY defines a storage-port trait (the #116 class).
    member(
        root,
        "cap/adapters/foo-inmemory",
        "src/lib.rs",
        "pub trait FooStoragePort {\n    fn put(&mut self);\n}\n\
         // a non-port behavioral trait in the same file must NOT be flagged\n\
         pub trait FooSpawner { fn spawn(&self); }\n",
    );
    // A core crate that correctly defines a port trait — must NOT be flagged (not in adapters/).
    member(
        root,
        "cap/core/foo-port",
        "src/lib.rs",
        "pub trait BarStore { fn get(&self); }\n",
    );

    let policy = fixture_policy();
    let observed = collect_port_traits(root, &policy).expect("collect red-repo fixture");
    let findings = evaluate_keyed(&policy, &serde_json::json!([]), &observed);

    let keys: Vec<String> = findings
        .iter()
        .filter(|f| f.code == "PP-PORT-IN-ADAPTER")
        .map(|f| f.key.clone())
        .collect();
    assert!(
        keys.contains(&"cap/adapters/foo-inmemory:FooStoragePort".to_owned()),
        "a storage-port trait defined in an adapter must be caught from disk: {keys:?}"
    );
    assert!(
        !keys.iter().any(|k| k.contains("FooSpawner")),
        "a behavioral adapter trait must NOT be flagged: {keys:?}"
    );
    assert!(
        !keys.iter().any(|k| k.contains("BarStore")),
        "a port trait in a CORE crate must NOT be flagged: {keys:?}"
    );
    assert_eq!(
        evaluate(&policy, &serde_json::json!([]), &observed).verdict,
        Verdict::Red
    );
}

#[test]
fn green_when_existing_violation_is_baselined_and_red_when_new_one_appears() {
    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);
    member(
        root,
        "cap/adapters/foo-inmemory",
        "src/lib.rs",
        "pub trait FooStore { fn get(&self); }\n",
    );

    let policy = fixture_policy();
    let observed = collect_port_traits(root, &policy).expect("collect");

    // Baselining the existing violation -> GREEN (born-advisory).
    let baseline = serde_json::json!({
        "baseline": [{ "member_path": "cap/adapters/foo-inmemory", "trait": "FooStore" }]
    });
    assert_eq!(
        evaluate(&policy, &baseline, &observed).verdict,
        Verdict::Green,
        "a baselined existing violation is not RED"
    );

    // Now plant a NEW port trait beyond the baseline -> RED.
    write_file(
        root,
        "cap/adapters/foo-inmemory/src/extra.rs",
        "pub trait FooRepository { fn all(&self); }\n",
    );
    let observed2 = collect_port_traits(root, &policy).expect("collect after planting");
    let findings = evaluate_keyed(&policy, &baseline, &observed2);
    assert!(
        findings.iter().any(|f| f.code == "PP-PORT-IN-ADAPTER"
            && f.key == "cap/adapters/foo-inmemory:FooRepository"),
        "a NEW port trait beyond the baseline must be RED: {findings:#?}"
    );
    assert_eq!(
        evaluate(&policy, &baseline, &observed2).verdict,
        Verdict::Red
    );
}
