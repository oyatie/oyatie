// cloud-ci-tier-field-coverage: born-blocking self-test over TODAY's real service manifests
// (Phase-0 capability-first reorg; ADR-0562/0536/0245/0280/0348). The test collects every governed
// service manifest from the live tree and asserts:
//   * the live corpus is born-blocking GREEN — every service manifest under cloud/ + oya/ carries a
//     valid tier/tier_subtype/dr_tier triple with no `tier` type-overload, every substrate carries a
//     valid substrate_dag_position, every top-level service declares ADR-0348 sharding automation,
//     and every SLO entry resolves to OpenSLO or carries a live exemption;
//   * the scan actually found the manifest census (TFC-EMPTY-SCAN floor is met), so a broken
//     glob/CWD/collect cannot pass as a silent false-green;
//   * the committed policy gate_id matches the crate contract, and the policy enums match the
//     authoritative specs/platform-architecture.json microservice_taxonomy enums (no policy drift).
// Filesystem RED/GREEN fixtures (materialized under the OS temp dir at runtime) prove the collector
// resolves real manifest.json files from disk and that a missing-tier / overloaded-tier / non-enum
// manifest fails the gate. Pure-unit RED fixtures live in src/tests.rs.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use ci_service_tier_metadata::{Verdict, collect_manifests, evaluate, evaluate_keyed};
use serde_json::Value;

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Mirrors the sibling gates so collection runs from the resolved
/// repo root, not the `cargo test` CWD.
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
    root.join("ci/facade/service-tier-metadata")
}

fn load_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn policy(root: &Path) -> Value {
    load_json(&gate_dir(root).join("tier-field-coverage-policy.json"))
}

/// Every `exclude_path_prefixes` entry must still shadow at least one REAL
/// `manifest.json`, and must never shadow a top-level service manifest. Without
/// this, an exclusion added for a test fixture rots into a permanent unaudited
/// blind spot the moment the fixture moves — the same failure mode
/// scan-root-liveness prevents for forward declarations.
#[test]
fn every_manifest_exclusion_is_live_and_shadows_no_service_manifest() {
    let root = repo_root();
    let policy = policy(&root);
    let roots: Vec<String> = policy["governed_service_roots"]
        .as_array()
        .expect("governed_service_roots")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    let prefixes: Vec<String> = policy["exclude_path_prefixes"]
        .as_array()
        .expect("exclude_path_prefixes must be declared (may be empty)")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();

    // Re-walk WITHOUT the exclusions so the shadowed set is observable.
    let unfiltered = {
        let mut open = policy.clone();
        open["exclude_path_prefixes"] = Value::Array(Vec::new());
        collect_manifests(&root, &open).expect("unfiltered collection")
    };
    let all: Vec<String> = unfiltered["manifests"]
        .as_array()
        .expect("manifests")
        .iter()
        .filter_map(|m| m["path"].as_str().map(str::to_owned))
        .collect();

    for prefix in &prefixes {
        let shadowed: Vec<&String> = all.iter().filter(|p| p.starts_with(prefix)).collect();
        assert!(
            !shadowed.is_empty(),
            "exclusion `{prefix}` shadows no manifest.json — remove it; a stale exclusion is a \
             permanent unaudited blind spot"
        );
        for path in shadowed {
            let depth = path.split('/').count();
            let is_service_manifest = depth == 3
                && roots
                    .iter()
                    .any(|r| path.split('/').next() == Some(r.as_str()));
            assert!(
                !is_service_manifest,
                "exclusion `{prefix}` shadows TOP-LEVEL service manifest `{path}` — exclusions \
                 exist for test fixtures, never for a governed service"
            );
        }
    }
}

#[test]
fn live_service_corpus_is_born_blocking_green() {
    let root = repo_root();
    let policy = policy(&root);

    let observed = collect_manifests(&root, &policy)
        .expect("read-only manifest collection should not need temp files or cleanup");
    let report = evaluate(&policy, &observed);

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "tier-field-coverage is born-blocking green on the live corpus; got {} finding(s):\n{}",
        findings.len(),
        findings
            .iter()
            .map(|f| format!("  {} {}: {}", f.code, f.key, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(report.verdict, Verdict::Green);
    assert!(
        report.manifests_checked >= 95,
        "the live tree should carry at least the ~101 service manifests; got {}",
        report.manifests_checked
    );
    eprintln!(
        "TIER-FIELD-COVERAGE live corpus: manifests={} findings=0 (born-blocking green)",
        report.manifests_checked
    );
}

#[test]
fn policy_gate_id_matches_the_crate_contract() {
    let root = repo_root();
    assert_eq!(
        policy(&root)["gate_id"].as_str(),
        Some("cloud-ci-tier-field-coverage")
    );
}

#[test]
fn policy_enums_match_platform_architecture_taxonomy() {
    // No policy drift: the gate's tier_enum + tier_subtype_enum must equal the authoritative
    // specs/platform-architecture.json microservice_taxonomy enums verbatim.
    let root = repo_root();
    let policy = policy(&root);
    let pa = load_json(&root.join("specs/platform-architecture.json"));
    // The taxonomy block is nested under microservice_taxonomy; locate the enums by key search
    // rather than a brittle fixed path.
    let pa_tier_enum = find_array(&pa, "tier_enum").expect("platform-arch tier_enum");
    let pa_subtype_enum =
        find_array(&pa, "tier_subtype_enum").expect("platform-arch tier_subtype_enum");
    assert_eq!(
        policy["tier_enum"], pa_tier_enum,
        "gate tier_enum must equal platform-architecture tier_enum verbatim"
    );
    assert_eq!(
        policy["tier_subtype_enum"], pa_subtype_enum,
        "gate tier_subtype_enum must equal platform-architecture tier_subtype_enum verbatim"
    );
}

/// Recursively find the first array value for `key` anywhere in `value`.
fn find_array(value: &Value, key: &str) -> Option<Value> {
    match value {
        Value::Object(map) => {
            if let Some(v) = map.get(key) {
                if v.is_array() {
                    return Some(v.clone());
                }
            }
            for v in map.values() {
                if let Some(found) = find_array(v, key) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(arr) => {
            for v in arr {
                if let Some(found) = find_array(v, key) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Filesystem fixture repos: prove collect_manifests resolves real manifest.json files from disk
// and that missing/overloaded/non-enum manifests fail the gate. Materialized under the OS temp dir
// at runtime (self-cleaning), exercising the exact collect_manifests filesystem path.
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
        "oya-tfc-fixture-{}-{}",
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

/// The committed policy with the scan floor lowered so the small fixture repos do not trip
/// TFC-EMPTY-SCAN (the floor guards the LIVE corpus, not fixtures).
fn fixture_policy() -> Value {
    let live = repo_root();
    let mut policy = policy(&live);
    policy["min_expected_service_manifests"] = Value::from(1u64);
    policy
}

#[test]
fn green_repo_fixture_passes_from_disk() {
    let repo = new_temp_repo();
    let root = &repo.root;
    write_file(
        root,
        "cloud/cloud-iam/manifest.json",
        r#"{
  "microservice": "cloud-iam",
  "tier": "substrate",
  "tier_subtype": "substrate-identity",
  "dr_tier": "T1",
  "substrate_dag_position": { "stratum": "S1", "depends_on": ["cell"], "consumed_by_substrates": [] },
  "sharding_automation": {
    "autosharding": { "enabled": false, "mode": "not_claimed_runtime", "intended_control_plane": "control_plane_driven" },
    "auto_rebalance": { "enabled": false },
    "dynamic_sharding": { "enabled": false }
  },
  "slos": [],
  "slo_exemption": {
    "status": "live_exempted_fixture",
    "owner": "axis-identity",
    "rationale": "The fixture intentionally models a non-runtime service whose OpenSLO coverage lands through a later cloud-ci admitted artifact.",
    "cutover_on": "2026-12-31",
    "evidence": "cloud-ci-tier-field-coverage fixture"
  }
}
"#,
    );
    write_file(
        root,
        "oya/crm/manifest.json",
        r#"{
  "microservice": "crm",
  "tier": "product",
  "tier_subtype": "product-consumer",
  "dr_tier": "T3",
  "sharding_automation": {
    "autosharding": "control_plane_driven",
    "auto_rebalance": { "enabled": false },
    "dynamic_sharding": { "enabled": false }
  },
  "slos": [],
  "slo_exemption": {
    "status": "live_exempted_fixture",
    "owner": "axis-crm",
    "rationale": "The fixture intentionally models a non-runtime service whose OpenSLO coverage lands through a later cloud-ci admitted artifact.",
    "cutover_on": "2026-12-31",
    "evidence": "cloud-ci-tier-field-coverage fixture"
  }
}
"#,
    );
    let policy = fixture_policy();
    let observed = collect_manifests(root, &policy).expect("collect green fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "green fixture must pass: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);
    assert_eq!(evaluate(&policy, &observed).manifests_checked, 2);
}

#[test]
fn red_repo_fixture_missing_tier_fails_from_disk() {
    let repo = new_temp_repo();
    let root = &repo.root;
    // No tier field at all.
    write_file(
        root,
        "oya/widget/manifest.json",
        "{\n  \"microservice\": \"widget\",\n  \"tier_subtype\": \"product-consumer\",\n  \"dr_tier\": \"T2\"\n}\n",
    );
    let policy = fixture_policy();
    let observed = collect_manifests(root, &policy).expect("collect red fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "TFC-MISSING-TIER" && f.key == "oya/widget/manifest.json"),
        "a manifest missing tier must fail from disk: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

#[test]
fn red_repo_fixture_overloaded_tier_fails_from_disk() {
    let repo = new_temp_repo();
    let root = &repo.root;
    // `tier: T1` overloads the dependency class with a DR/reliability value (the V3 anti-pattern).
    write_file(
        root,
        "cloud/legacy-svc/manifest.json",
        "{\n  \"microservice\": \"legacy-svc\",\n  \"tier\": \"T1\",\n  \"tier_subtype\": \"substrate-infra\",\n  \"dr_tier\": \"T1\"\n}\n",
    );
    let policy = fixture_policy();
    let observed = collect_manifests(root, &policy).expect("collect red fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.iter().any(
            |f| f.code == "TFC-TIER-TYPE-OVERLOAD" && f.key == "cloud/legacy-svc/manifest.json"
        ),
        "an overloaded tier (DR value) must fail from disk: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

#[test]
fn red_repo_fixture_non_enum_subtype_fails_from_disk() {
    let repo = new_temp_repo();
    let root = &repo.root;
    write_file(
        root,
        "oya/erp/manifest.json",
        "{\n  \"microservice\": \"erp\",\n  \"tier\": \"product\",\n  \"tier_subtype\": \"erp-parity-single-concern\",\n  \"dr_tier\": \"T2\"\n}\n",
    );
    let policy = fixture_policy();
    let observed = collect_manifests(root, &policy).expect("collect red fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "TFC-TIER-SUBTYPE-NOT-IN-ENUM" && f.key == "oya/erp/manifest.json"),
        "a non-enum tier_subtype must fail from disk: {findings:#?}"
    );
}

#[test]
fn red_repo_fixture_substrate_without_dag_position_fails_from_disk() {
    let repo = new_temp_repo();
    let root = &repo.root;
    write_file(
        root,
        "cloud/cell/manifest.json",
        "{\n  \"microservice\": \"cell\",\n  \"tier\": \"substrate\",\n  \"tier_subtype\": \"substrate-infra\",\n  \"dr_tier\": \"T1\"\n}\n",
    );
    let policy = fixture_policy();
    let observed = collect_manifests(root, &policy).expect("collect red fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "TFC-SUBSTRATE-MISSING-DAG-POSITION"),
        "a substrate without substrate_dag_position must fail: {findings:#?}"
    );
}
