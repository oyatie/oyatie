// cloud-ci-capability-membership: born self-test over TODAY's real crate corpus (Phase-0
// capability-first reorg; ADR-0562 §6, the anti-junk-drawer MEMBERSHIP lint). The test collects
// every crate from the live tree and asserts:
//   * the live corpus is born-ADVISORY GREEN — every crate maps to exactly one registered
//     capability/meta home OR is in the registry's frozen unmapped baseline; there is NO NEW
//     unmapped crate and NO NEW top-level dir outside the closed set, so any regression fails;
//   * the scan actually found the crate census (MEM-EMPTY-SCAN floor met), so a broken
//     glob/CWD/collect cannot pass as a silent false-green;
//   * the committed policy gate_id matches the crate contract.
// Filesystem RED/GREEN fixtures (materialized under the OS temp dir at runtime) prove the collector
// resolves real Cargo.toml crates from disk and that the three mandated regressions fail: a crate
// in NO capability (new, unmapped), a NEW top-level dir common/, and a crate in TWO capabilities.
// Pure-unit RED fixtures live in src/tests.rs.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ci_module_membership::{Verdict, collect, evaluate, evaluate_keyed};
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
    root.join("ci/facade/module-membership")
}

fn load_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn policy(root: &Path) -> Value {
    load_json(&gate_dir(root).join("capability-membership-policy.json"))
}

#[test]
fn live_crate_corpus_is_born_advisory_green() {
    let root = repo_root();
    let policy = policy(&root);

    let observed =
        collect(&root, &policy).expect("read-only crate collection should not need temp files");
    let report = evaluate(&policy, &observed);

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "capability-membership is born-advisory green on the live corpus; got {} finding(s):\n{}",
        findings.len(),
        findings
            .iter()
            .map(|f| format!("  {} {}: {}", f.code, f.key, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(report.verdict, Verdict::Green);
    assert!(
        report.crates_checked >= 780,
        "the live tree should carry at least ~783 crates; got {}",
        report.crates_checked
    );
    eprintln!(
        "CAPABILITY-MEMBERSHIP live corpus: crates={} mapped={} frozen-unmapped={} findings=0 (born-advisory green)",
        report.crates_checked, report.mapped_to_home, report.frozen_unmapped
    );
}

#[test]
fn policy_gate_id_matches_the_crate_contract() {
    let root = repo_root();
    assert_eq!(
        policy(&root)["gate_id"].as_str(),
        Some("cloud-ci-capability-membership")
    );
}

#[test]
fn registry_membership_coverage_block_is_present() {
    // The registry extension MUST be present or the gate cannot evaluate.
    let root = repo_root();
    let registry = load_json(&root.join("governance/capability-registry.json"));
    assert!(
        registry.get("membership_lint_coverage").is_some(),
        "governance/capability-registry.json must carry the membership_lint_coverage block"
    );
}

#[test]
fn d41_office_suite_and_translate_are_not_app_products() {
    // Interview D41/D42: notes/slides/sites/office/translate must not be absorb-into-app
    // destinations. sheets stays (Foundry-like; D41 did not name it).
    let root = repo_root();
    let registry = load_json(&root.join("governance/capability-registry.json"));
    let coverage = registry
        .get("membership_lint_coverage")
        .expect("membership_lint_coverage");
    let app_dirs: Vec<&str> = coverage["app_products"]["current_dirs"]
        .as_array()
        .expect("app_products.current_dirs")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    let retired_dirs: Vec<&str> = coverage["retired_v1_products"]["current_dirs"]
        .as_array()
        .expect("retired_v1_products.current_dirs")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    for retired in [
        "oya/notes",
        "oya/slides",
        "oya/sites",
        "oya/office",
        "oya/translate",
    ] {
        assert!(
            !app_dirs.contains(&retired),
            "{retired} must not be an app_products absorb destination: {app_dirs:?}"
        );
    }
    assert!(
        app_dirs.contains(&"oya/sheets"),
        "sheets stays in app_products (D41 did not name it): {app_dirs:?}"
    );
    for still_on_disk in ["oya/notes", "oya/slides", "oya/sites", "oya/translate"] {
        assert!(
            retired_dirs.contains(&still_on_disk),
            "{still_on_disk} must be retired_v1_products (still on disk): {retired_dirs:?}"
        );
        assert!(
            root.join(still_on_disk).is_dir(),
            "{still_on_disk} must still exist (this PR does not delete product trees)"
        );
    }
    assert!(
        !retired_dirs.contains(&"oya/office"),
        "oya/office is already gone from the tree; current_dirs lists only live paths: {retired_dirs:?}"
    );
    assert_eq!(
        coverage["retired_v1_products"]["disposition"].as_str(),
        Some("retire-in-place; do not absorb into app/; D41/D42")
    );
    assert!(
        coverage["retired_v1_products"]
            .get("meta_dir")
            .and_then(|v| v.as_str())
            .is_none(),
        "retired_v1_products must not carry meta_dir (that would absorb into app/)"
    );
}

// ---------------------------------------------------------------------------
// Filesystem fixture repos: prove collect resolves real Cargo.toml crates from disk and that the
// three mandated regressions fail. Materialized under the OS temp dir (self-cleaning).
// ---------------------------------------------------------------------------

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
        "oya-mem-fixture-{}-{}",
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

fn package_manifest(name: &str) -> String {
    format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n")
}

/// A small registry fixture mapping cloud/cloud-iam + cloud/cloud-data, with no overlaps.
fn fixture_registry() -> &'static str {
    r#"{
  "capabilities": [
    { "name": "iam", "absorbs_current_dirs": ["cloud/cloud-iam"] },
    { "name": "data", "absorbs_current_dirs": ["cloud/cloud-data"] }
  ],
  "membership_lint_coverage": {
    "app_products": { "meta_dir": "app/", "current_dirs": ["oya/crm"] },
    "meta_directory_absorbs": [],
    "absorbs_current_crate_globs": [],
    "frozen_unmapped_baseline": { "burn_down_target": 0, "crates": [] }
  }
}
"#
}

/// The committed policy with the floor lowered + registry repointed at the fixture registry so the
/// small fixture repos do not trip MEM-EMPTY-SCAN (the floor guards the LIVE corpus, not fixtures).
/// The `legacy_root_freeze` block is dropped for the same reason: its census names the 445 real
/// legacy-root crate dirs, none of which exist in a temp fixture, so every fixture would drown in
/// MEM-STALE-LEGACY-ROOT-BASELINE. The freeze is exercised against its own small fixture below, and
/// the COMMITTED block's liveness is asserted by `committed_policy_freezes_the_legacy_roots`.
fn fixture_policy(repo: &Path) -> Value {
    let live = repo_root();
    let mut policy = policy(&live);
    policy["min_expected_crates"] = Value::from(1u64);
    if let Some(object) = policy.as_object_mut() {
        object.remove("legacy_root_freeze");
    }
    // The fixture registry lives at the fixture repo root.
    write_file(
        repo,
        "governance/capability-registry.json",
        fixture_registry(),
    );
    // Also stamp root-hub-pointers so any walk-up logic that targets the fixture is satisfied.
    write_file(repo, "specs/root-hub-pointers.json", "{}\n");
    policy
}

#[test]
fn green_repo_fixture_passes_from_disk() {
    let repo = new_temp_repo();
    let root = &repo.root;
    write_file(
        root,
        "cloud/cloud-iam/crates/iam-kernel/Cargo.toml",
        &package_manifest("iam-kernel"),
    );
    write_file(
        root,
        "cloud/cloud-data/crates/data-kernel/Cargo.toml",
        &package_manifest("data-kernel"),
    );
    write_file(
        root,
        "oya/crm/crates/crm-kernel/Cargo.toml",
        &package_manifest("crm-kernel"),
    );
    let policy = fixture_policy(root);
    let observed = collect(root, &policy).expect("collect green fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "green fixture must pass: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);
    assert_eq!(evaluate(&policy, &observed).crates_checked, 3);
}

#[test]
fn red_crate_in_no_capability_fails_from_disk() {
    // RED FIXTURE #1: a crate that maps to NO capability (new, unmapped).
    let repo = new_temp_repo();
    let root = &repo.root;
    write_file(
        root,
        "cloud/cloud-iam/crates/iam-kernel/Cargo.toml",
        &package_manifest("iam-kernel"),
    );
    write_file(
        root,
        "oya/widget/crates/widget-kernel/Cargo.toml",
        &package_manifest("widget-kernel"),
    );
    let policy = fixture_policy(root);
    let observed = collect(root, &policy).expect("collect red#1 fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "MEM-NEW-UNMAPPED-CRATE"
                && f.key == "oya/widget/crates/widget-kernel"),
        "an unmapped (new) crate must fail from disk: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

#[test]
fn red_new_top_level_dir_common_fails_from_disk() {
    // RED FIXTURE #2: a NEW top-level dir common/.
    let repo = new_temp_repo();
    let root = &repo.root;
    write_file(
        root,
        "cloud/cloud-iam/crates/iam-kernel/Cargo.toml",
        &package_manifest("iam-kernel"),
    );
    write_file(
        root,
        "common/oya-util/Cargo.toml",
        &package_manifest("oya-util"),
    );
    let policy = fixture_policy(root);
    let observed = collect(root, &policy).expect("collect red#2 fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "MEM-NEW-TOP-LEVEL-DIR" && f.key == "common"),
        "a NEW top-level dir common/ must fail from disk: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

#[test]
fn red_crate_in_two_capabilities_fails_from_disk() {
    // RED FIXTURE #3: a crate claimed by TWO capabilities (overlapping absorbs).
    let repo = new_temp_repo();
    let root = &repo.root;
    write_file(
        root,
        "cloud/cloud-iam/crates/iam-kernel/Cargo.toml",
        &package_manifest("iam-kernel"),
    );
    let mut policy = fixture_policy(root);
    // Repoint the fixture registry to one where two caps both absorb cloud/cloud-iam.
    write_file(
        root,
        "governance/capability-registry.json",
        r#"{
  "capabilities": [
    { "name": "iam", "absorbs_current_dirs": ["cloud/cloud-iam"] },
    { "name": "data", "absorbs_current_dirs": ["cloud/cloud-iam"] }
  ],
  "membership_lint_coverage": {
    "app_products": { "meta_dir": "app/", "current_dirs": [] },
    "meta_directory_absorbs": [],
    "absorbs_current_crate_globs": [],
    "frozen_unmapped_baseline": { "burn_down_target": 0, "crates": [] }
  }
}
"#,
    );
    policy["min_expected_crates"] = Value::from(1u64);
    let observed = collect(root, &policy).expect("collect red#3 fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "MEM-DOUBLE-MAPPED-CRATE"
                && f.key == "cloud/cloud-iam/crates/iam-kernel"),
        "a crate in two capabilities must fail from disk: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

#[test]
fn the_committed_policy_scans_every_crate_owning_destination_root() {
    // COVERAGE PROOF over the COMMITTED policy (not a fixture policy): a crate at
    // each crate-owning meta destination is actually WALKED. `app/` is the
    // destination for the 110 app-product crates; dropping any of these roots from
    // `scan_roots` would drop those crates out of the membership lint silently —
    // the min_expected_crates floor is a broken-scan guard, not a coverage guard,
    // and cannot see a partial-root loss. This test fails if a root is dropped.
    let repo = new_temp_repo();
    let root = &repo.root;
    let policy = fixture_policy(root);
    for meta in ["app", "base", "kernel", "os"] {
        write_file(
            root,
            &format!("{meta}/member/Cargo.toml"),
            &package_manifest(&format!("{meta}-member")),
        );
    }
    let observed = collect(root, &policy).expect("collect destination-root fixture");
    let collected: Vec<String> = observed["crates"]
        .as_array()
        .expect("crates array")
        .iter()
        .map(|c| c.as_str().unwrap_or_default().to_owned())
        .collect();
    for meta in ["app", "base", "kernel", "os"] {
        assert!(
            collected.contains(&format!("{meta}/member")),
            "the committed scan_roots must walk the `{meta}/` destination; collected {collected:?}"
        );
    }
}

#[test]
fn a_base_crate_without_admission_facts_is_red_now_that_base_is_scanned() {
    // The ADR-0562 §6 base/-admission rule was VACUOUS while `base` sat outside
    // `scan_roots`: no base/ crate ever reached the evaluator. With the root
    // scanned, a base/ crate that cannot prove >=3 capability consumers fails
    // CLOSED, which is the anti-junk-drawer backstop the ADR mandates.
    let repo = new_temp_repo();
    let root = &repo.root;
    write_file(
        root,
        "base/dumping-ground/Cargo.toml",
        &package_manifest("dumping-ground"),
    );
    let policy = fixture_policy(root);
    let observed = collect(root, &policy).expect("collect base fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "MEM-BASE-ADMISSION-CONSUMERS" && f.key == "base/dumping-ground"),
        "a base/ crate with no declared admission facts must fail closed: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

// ---------------------------------------------------------------------------
// STOP ACCRUAL — the legacy-root freeze, proven in BOTH directions on the COMMITTED policy.
// ---------------------------------------------------------------------------

#[test]
fn committed_policy_freezes_the_legacy_roots_with_a_producer_emitted_census() {
    // ANTI-VACUITY. The freeze is inert when the block is absent or declares no roots — which is
    // exactly what a fixture wants and exactly what would silently switch the live gate off. This
    // asserts the COMMITTED policy is not in that state: all four legacy source roots are frozen,
    // and the census is the real corpus, not a token entry.
    let root = repo_root();
    let policy = policy(&root);
    let freeze = &policy["legacy_root_freeze"];

    let roots: Vec<&str> = freeze["frozen_roots"]
        .as_array()
        .expect("legacy_root_freeze.frozen_roots must be an array")
        .iter()
        .map(|v| v.as_str().expect("root is a string"))
        .collect();
    assert_eq!(
        roots,
        vec!["cloud", "libs", "oya", "tools"],
        "every legacy source root ADR-0562 empties must be frozen; dropping one re-opens accrual there"
    );

    let census = freeze["crates"]
        .as_array()
        .expect("legacy_root_freeze.crates must be an array");
    // Anti-vacuity is asserted WITHOUT a constant floor. The census is shrink-only with a
    // burn-down target of ZERO, so any hardcoded minimum is really a ceiling on the reorg this
    // block exists to protect: this assertion read `census.len() > 400` and went RED the first
    // time a move batch legitimately burned 66 entries down to 350. Fidelity — that the census
    // IS the live legacy-root crate set, neither over-broad nor short — is proven exactly by
    // `live_corpus_census_matches_the_committed_freeze_exactly` below, which strictly subsumes
    // any count check (a token census fails the equality). All this one owes is that the block
    // is not the inert empty stub a fixture carries.
    assert!(
        !census.is_empty(),
        "the frozen census is EMPTY — the freeze is inert, which is exactly the state that \
         silently switches the live gate off; re-emit it with --emit-legacy-freeze"
    );
    for entry in census {
        let dir = entry.as_str().expect("census entry is a string");
        assert!(
            roots
                .iter()
                .any(|r| dir == *r || dir.starts_with(&format!("{r}/"))),
            "census entry {dir:?} is not under any frozen root — the census is keyed by crate DIR \
             under a frozen root, so an off-root entry can only ever be dead weight"
        );
    }
}

#[test]
fn live_corpus_census_matches_the_committed_freeze_exactly() {
    // BASELINE FIDELITY (direction (a) of the oracle, stated as an equality rather than an absence
    // of findings): the committed census IS the live legacy-root crate set. Neither over-broad (a
    // stale entry would pre-forgive a crate re-created at that path) nor short (a missing entry
    // would fail a crate that exists today).
    let root = repo_root();
    let policy = policy(&root);
    let observed = collect(&root, &policy).expect("collect the live corpus");
    let crates: Vec<String> = observed["crates"]
        .as_array()
        .expect("crates array")
        .iter()
        .map(|c| c.as_str().unwrap_or_default().to_owned())
        .collect();

    let live = ci_module_membership::legacy_root_census(&policy, &crates);
    let committed: Vec<String> = policy["legacy_root_freeze"]["crates"]
        .as_array()
        .expect("census array")
        .iter()
        .map(|c| c.as_str().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(
        live, committed,
        "the frozen census must equal the live legacy-root crate set exactly; re-run \
         `--emit-legacy-freeze` and commit the result"
    );
    eprintln!(
        "LEGACY-ROOT FREEZE: {} crate(s) frozen shrink-only across cloud/ libs/ oya/ tools/ \
         (burn-down target 0)",
        live.len()
    );
}

#[test]
fn red_new_crate_under_a_frozen_legacy_root_fails_from_disk() {
    // DIRECTION (b) OF THE ORACLE, from disk and against the COMMITTED freeze semantics: a crate
    // BORN under a frozen legacy root, mapping cleanly to a registered capability (so every other
    // membership check is happy), must still fail. Without this the gate is indistinguishable from
    // one that checks nothing — the live tree is green either way.
    let repo = new_temp_repo();
    let root = &repo.root;
    write_file(
        root,
        "cloud/cloud-iam/crates/iam-kernel/Cargo.toml",
        &package_manifest("iam-kernel"),
    );
    write_file(
        root,
        "cloud/cloud-iam/crates/iam-brand-new/Cargo.toml",
        &package_manifest("iam-brand-new"),
    );
    let mut policy = fixture_policy(root);
    // The freeze as it is committed, narrowed to this fixture's corpus: `cloud` frozen, and the
    // pre-existing crate — but NOT the new one — in the census.
    policy["legacy_root_freeze"] = serde_json::json!({
        "frozen_roots": ["cloud", "libs", "oya", "tools"],
        "crates": ["cloud/cloud-iam/crates/iam-kernel"]
    });

    let observed = collect(root, &policy).expect("collect legacy-freeze fixture");
    let findings = evaluate_keyed(&policy, &observed);

    // The new crate maps to exactly one capability, so it is NOT an unmapped-crate finding — the
    // freeze is the only thing that can catch it.
    assert!(
        !findings
            .iter()
            .any(|f| f.key == "cloud/cloud-iam/crates/iam-brand-new"
                && f.code == "MEM-NEW-UNMAPPED-CRATE"),
        "the fixture must exercise the freeze, not the membership map: {findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|f| f.code == "MEM-NEW-LEGACY-ROOT-CRATE"
                && f.key == "cloud/cloud-iam/crates/iam-brand-new"),
        "a crate born under a FROZEN legacy root must fail from disk: {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
    assert_eq!(evaluate(&policy, &observed).legacy_root_crates, 2);
}

#[test]
fn virtual_workspace_manifest_is_not_a_crate() {
    // A [workspace]-only Cargo.toml is NOT a crate; only [package] manifests count.
    let repo = new_temp_repo();
    let root = &repo.root;
    write_file(
        root,
        "cloud/cloud-iam/Cargo.toml",
        "[workspace]\nmembers = [\"crates/iam-kernel\"]\n",
    );
    write_file(
        root,
        "cloud/cloud-iam/crates/iam-kernel/Cargo.toml",
        &package_manifest("iam-kernel"),
    );
    let policy = fixture_policy(root);
    let observed = collect(root, &policy).expect("collect ws fixture");
    // Only the [package] crate is collected; the [workspace] manifest dir is not.
    assert_eq!(evaluate(&policy, &observed).crates_checked, 1);
    assert!(evaluate_keyed(&policy, &observed).is_empty());
}

/// LIVE CORPUS: every `absorbs_current_dirs` entry must name a directory that EXISTS.
///
/// The registry's whole job is to say which legacy dirs each capability is still absorbing — it is
/// the ledger the strangler burns down against. Nothing asserted that those paths resolve, so a
/// COMPLETED move left its entry behind forever and the ledger silently overstated the remaining
/// work. Measured at the time this landed: 10 of 73 legacy-prefixed entries named directories that
/// no longer existed, including `cloud/cloud-ci` — the same vacated tree that had the cold cache
/// canary building an empty target set.
///
/// This is the reorg co-move blind spot in its purest form: the tree moved, the ledger did not, and
/// every gate stayed green. Sibling detectors now exist for the canary's pinned targets and for the
/// disk-reclaim runner profile; this closes the third instance.
///
/// Direction matters. A stale entry is BURN-DOWN that was never recorded, so removing it is always
/// allowed and this test forces it to happen. The inverse — a live dir missing from the registry —
/// is what MEM-NEW-LEGACY-ROOT-CRATE and the frozen census already cover.
#[test]
fn live_registry_absorbs_dirs_all_resolve() {
    let root = repo_root();
    let registry = load_json(&root.join("governance/capability-registry.json"));

    let capabilities = registry
        .get("capabilities")
        .expect("capability-registry must carry `capabilities`");

    let mut stale: Vec<String> = Vec::new();
    let mut checked = 0usize;

    let mut check = |name: &str, value: &serde_json::Value| {
        for dir in value
            .get("absorbs_current_dirs")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or_default()
        {
            let Some(dir) = dir.as_str() else { continue };
            checked += 1;
            if !root.join(dir).is_dir() {
                stale.push(format!("{name} -> {dir}"));
            }
        }
    };

    match capabilities {
        serde_json::Value::Object(map) => {
            for (name, cap) in map {
                check(name, cap);
            }
        }
        serde_json::Value::Array(list) => {
            for cap in list {
                let name = cap
                    .get("name")
                    .or_else(|| cap.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unnamed>");
                check(name, cap);
            }
        }
        other => panic!("`capabilities` must be an object or array, got {other:?}"),
    }

    // MEM-EMPTY-SCAN floor: a registry that resolved zero entries would pass vacuously, which is the
    // exact false-green shape this test exists to prevent.
    assert!(
        checked > 0,
        "no absorbs_current_dirs entries were checked — the registry shape changed and this \
         detector is now vacuous, which reads as success while asserting nothing"
    );

    assert!(
        stale.is_empty(),
        "{} of {checked} `absorbs_current_dirs` entries name a directory that does NOT exist. A \
         completed move left its ledger entry behind, so the registry overstates the remaining \
         reorg work and nothing noticed. Removing a stale entry is burn-down and is always \
         allowed:\n  {}",
        stale.len(),
        stale.join("\n  ")
    );
}
