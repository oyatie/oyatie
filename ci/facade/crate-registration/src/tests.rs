//! Hermetic tmp-repo tests for the register-crate orchestrator (G011 slice 3b). Each test builds
//! a minimal on-disk repo (a real `git init` so `git ls-files` works for the producer bridges)
//! and exercises one path through [`register_crate`]. std-only — no buck2/shell beyond the `git`
//! plumbing the orchestrator itself shells to.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_crate_registrar_kernel::{CatalogSpec, CrateRole, RegisterCrateRequest};

use super::*;

/// A throwaway tmp repo dir, removed on drop. Uses a unique nonce so concurrent tests never clash.
struct TmpRepo {
    root: PathBuf,
}

impl Drop for TmpRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl TmpRepo {
    fn write(&self, rel: &str, content: &str) {
        let abs = self.root.join(rel);
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(abs, content).unwrap();
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.root.join(rel)).unwrap()
    }

    fn exists(&self, rel: &str) -> bool {
        self.root.join(rel).exists()
    }

    /// `git init` + `git add -A` so `git ls-files` enumerates the fixture (the producer bridges'
    /// self-validation universe). Identity is set locally so commits/adds never depend on a global
    /// git config.
    fn git_add_all(&self) {
        run_git(&self.root, &["init", "-q"]);
        run_git(&self.root, &["config", "user.email", "test@example.com"]);
        run_git(&self.root, &["config", "user.name", "Test"]);
        run_git(&self.root, &["add", "-A"]);
    }
}

fn run_git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

/// A unique tmp dir under the system temp root.
fn unique_root(tag: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "register-crate-{tag}-{nonce}-{:?}",
        std::thread::current().id()
    ))
}

/// The new crate dir under test: a ci gate-tool crate (absorbed by the `ci` capability dir).
const NEW_DIR: &str = "ci/facade/oya-cloud-ci-example-app";

/// A minimal capability-registry mirroring the REAL schema the membership-lint gate's
/// `parse_mapping` consumes (`capabilities[].name` + the full `membership_lint_coverage` block):
///   - a crate-glob group (`build/`) with a `*`-suffix glob (`libs/oya-some-*`) for the closed
///     CapabilitySet + the glob-membership path;
///   - a capability (`ci`, by `name`) that absorbs `ci` so the new gate crate is already
///     capability-mapped by its dir — exactly the producer's situation post-keystone-move;
///   - `app_products` (→ `meta:app/`) absorbing `oya/application`;
///   - `meta_directory_absorbs` (→ `meta:kernel/`/`meta:os/`) absorbing `cloud/cloud-kernel` +
///     `os`.
///
/// The orchestrator REUSES the gate's `parse_mapping`/`homes_for`, so this fixture must be the same
/// shape the gate enforces (the drift the fix removes).
fn capability_registry() -> &'static str {
    r#"{
  "capabilities": [
    {
      "name": "ci",
      "absorbs_current_dirs": ["ci"]
    },
    {
      "name": "data",
      "absorbs_current_dirs": ["cloud/cloud-data"]
    }
  ],
  "membership_lint_coverage": {
    "app_products": {
      "meta_dir": "app/",
      "current_dirs": ["oya/application"]
    },
    "meta_directory_absorbs": [
      { "meta_dir": "kernel/", "current_dirs": ["cloud/cloud-kernel"] },
      { "meta_dir": "os/", "current_dirs": ["os"] }
    ],
    "absorbs_current_crate_globs": [
      {
        "meta_dir": "build/",
        "globs": ["libs/oya-some-*"]
      },
      {
        "capability": "data",
        "globs": []
      }
    ]
  }
}
"#
}

/// A root Cargo.toml whose members glob `ci/facade/*` covers the new crate dir.
fn root_cargo_toml() -> &'static str {
    r#"[workspace]
resolver = "2"
members = [
    "libs/oya-*",
    "ci/facade/*",
]

[workspace.package]
edition = "2024"
version = "0.1.0"
"#
}

/// A stub ADR with an EMPTY `## Governed surfaces` block (the writer upserts paths into it).
fn stub_adr() -> &'static str {
    "# ADR-0568: Born-accounting register_crate\n\n\
     Status: Accepted\n\n\
     ## Governed surfaces\n\n\
     ```\n\
     ```\n"
}

/// An empty (but well-formed) reachability registry.
fn reachability_registry() -> &'static str {
    "{\n  \"registered\": []\n}\n"
}

/// Build a fully-wired fixture repo for born-accounting `NEW_DIR`, with the new crate's source
/// files present (so member-glob coverage + git-tracking are realistic) but NONE of its
/// born-accounting SSOTs seeded yet. The `tag` namespaces the tmp dir so concurrent tests never
/// clash. Returns the repo handle (removed on drop).
fn fixture_tagged(tag: &str) -> TmpRepo {
    let repo = TmpRepo {
        root: unique_root(tag),
    };
    repo.write("governance/capability-registry.json", capability_registry());
    repo.write("Cargo.toml", root_cargo_toml());
    repo.write("docs/adr-archive/ADR-0568-born-accounting-register-crate-registrar-kernel.md", stub_adr());
    repo.write("specs/reachability-registry.json", reachability_registry());

    // The new crate's intrinsic source (Cargo.toml + lib.rs) so it is a real dir + git-tracked.
    repo.write(
        &format!("{NEW_DIR}/Cargo.toml"),
        "[package]\nname = \"oya-cloud-ci-example-app\"\n",
    );
    repo.write(&format!("{NEW_DIR}/src/lib.rs"), "//! example\n");

    // An existing sibling so the gate tree is non-trivial.
    repo.write(
        "ci/facade/oya-cloud-ci-some-app/Cargo.toml",
        "[package]\nname=\"x\"\n",
    );

    repo.git_add_all();
    repo
}

/// The base request: a cloud-ci gate-tool crate (App role, has-lib, has-test, no catalog — the
/// documented default for a cloud-ci gate crate, matching the producer + every gate sibling).
fn base_request() -> RegisterCrateRequest {
    RegisterCrateRequest {
        crate_dir: NEW_DIR.to_owned(),
        capability: "build/".to_owned(),
        owning_adr: "ADR-0568".to_owned(),
        owner: "cloud-ci-platform".to_owned(),
        role: CrateRole::App,
        has_lib: true,
        has_test_code: true,
        catalog: None,
        extra_governed_paths: Vec::new(),
    }
}

// (1) HAPPY PATH: a new crate dir → all expected edits applied, requires_faces_settle = true.
#[test]
fn happy_path_applies_all_edits_and_requires_settle() {
    let repo = fixture_tagged("happy");
    let req = base_request();
    let outcome = register_crate(&repo.root, &req).unwrap();

    // FacesSettle obligation recorded (something changed).
    assert!(
        outcome.requires_faces_settle,
        "faces must need a settle after a real registration"
    );

    let kinds: Vec<_> = outcome.applied.iter().map(|a| a.kind).collect();
    // The kernel's plan is the DIFF vs the live snapshot: it emits an edit ONLY for an SSOT not
    // already satisfied. In this fixture:
    //   - OWNERS is absent          → OwnersWrite IS emitted + dispatched.
    //   - the ADR block is empty    → AdrGovernedPathAppend IS emitted + dispatched.
    //   - the members glob ALREADY covers the dir → NO WorkspaceMemberGlob edit (already covered).
    //   - the dir is absorbed by the `ci` capability → NO CapabilityMapping edit.
    //   - no catalog requested      → NO CatalogYaml edit.
    assert!(kinds.contains(&AppliedEditKind::OwnersWrite), "{kinds:?}");
    assert!(
        kinds.contains(&AppliedEditKind::AdrGovernedPathAppend),
        "{kinds:?}"
    );
    assert!(
        !kinds.contains(&AppliedEditKind::WorkspaceMemberGlob),
        "members glob already covers the dir — no WorkspaceMemberGlob edit expected: {kinds:?}"
    );
    assert!(
        !kinds.contains(&AppliedEditKind::CapabilityMapping),
        "dir is absorbed by the ci capability — no CapabilityMapping edit expected: {kinds:?}"
    );
    assert!(
        !kinds.contains(&AppliedEditKind::CatalogYaml),
        "no catalog requested — no CatalogYaml edit expected: {kinds:?}"
    );

    // OWNERS file now exists with the owner.
    assert!(repo.exists(&format!("{NEW_DIR}/OWNERS")));
    assert_eq!(
        repo.read(&format!("{NEW_DIR}/OWNERS")),
        "cloud-ci-platform\n"
    );

    // The ADR now enumerates the crate's conventional governed paths verbatim.
    let adr = repo.read("docs/adr-archive/ADR-0568-born-accounting-register-crate-registrar-kernel.md");
    assert!(adr.contains(&format!("{NEW_DIR}/Cargo.toml")), "{adr}");
    assert!(adr.contains(&format!("{NEW_DIR}/BUCK")), "{adr}");
    assert!(adr.contains(&format!("{NEW_DIR}/OWNERS")), "{adr}");
    assert!(adr.contains(&format!("{NEW_DIR}/src/lib.rs")), "{adr}");
}

// CapabilityMapping dispatch: a libs/ crate NOT absorbed by any capability dir → the kernel emits
// a CapabilityMapping edit and the writer upserts the dir into the `build/` group's globs.
#[test]
fn libs_crate_maps_capability_via_writer() {
    let repo = fixture_tagged("cap-map");
    let libs_dir = "libs/oya-new-thing-kernel";
    repo.write(
        &format!("{libs_dir}/Cargo.toml"),
        "[package]\nname=\"oya-new-thing-kernel\"\n",
    );
    repo.write(&format!("{libs_dir}/src/lib.rs"), "//! new thing\n");
    run_git(&repo.root, &["add", "-A"]);

    let req = RegisterCrateRequest {
        crate_dir: libs_dir.to_owned(),
        capability: "build/".to_owned(),
        owning_adr: "ADR-0568".to_owned(),
        owner: "cloud-ci-platform".to_owned(),
        role: CrateRole::Kernel,
        has_lib: true,
        has_test_code: true,
        catalog: None,
        extra_governed_paths: Vec::new(),
    };
    let outcome = register_crate(&repo.root, &req).unwrap();

    let cap = outcome
        .applied
        .iter()
        .find(|a| a.kind == AppliedEditKind::CapabilityMapping)
        .expect("a libs/ crate not absorbed by a capability dir must get a CapabilityMapping edit");
    assert!(
        cap.changed,
        "the registry must be rewritten with the new mapping"
    );
    assert_eq!(cap.path, "governance/capability-registry.json");

    // The dir is now in the `build/` group's globs.
    let registry = repo.read("governance/capability-registry.json");
    assert!(
        registry.contains(libs_dir),
        "registry must list the new dir: {registry}"
    );
}

// CatalogYaml + ReachabilityEntry dispatch: a catalog-bearing crate with a non-crate extra
// governed path drives both the catalog writer and the producer's fix_reachability bridge.
#[test]
fn catalog_and_reachability_dispatch() {
    let repo = fixture_tagged("cat-reach");
    let mut req = base_request();
    req.catalog = Some(CatalogSpec {
        plane: "run".to_owned(),
        slo: "ga-control-plane".to_owned(),
    });
    // A non-crate governed path (outside the crate dir) → a ReachabilityEntry edit.
    let extra = "specs/fixtures/register-crate/example-case.json";
    repo.write(extra, "{}\n");
    req.extra_governed_paths = vec![extra.to_owned()];
    run_git(&repo.root, &["add", "-A"]);

    let outcome = register_crate(&repo.root, &req).unwrap();
    let kinds: Vec<_> = outcome.applied.iter().map(|a| a.kind).collect();
    assert!(kinds.contains(&AppliedEditKind::CatalogYaml), "{kinds:?}");
    assert!(
        kinds.contains(&AppliedEditKind::ReachabilityEntry),
        "{kinds:?}"
    );

    // The catalog file was rendered with the human-supplied plane + slo.
    let leaf = NEW_DIR.rsplit('/').next().unwrap();
    let catalog = repo.read(&format!("registry/catalog/{leaf}.yaml"));
    assert!(catalog.contains("plane: run"), "{catalog}");
    assert!(catalog.contains("slo: ga-control-plane"), "{catalog}");

    // The reachability registry now carries the non-crate path as a registered prefix.
    let reach = repo.read("specs/reachability-registry.json");
    assert!(reach.contains(extra), "{reach}");
}

// (2) IDEMPOTENT RE-RUN: after born-accounting, a second run applies no changes. (Re-run uses a
//     fresh CurrentState read from disk, so the seeded OWNERS/ADR make the plan empty.)
#[test]
fn idempotent_rerun_applies_nothing() {
    let repo = fixture_tagged("idem");
    let req = base_request();

    // First run: registers everything.
    let first = register_crate(&repo.root, &req).unwrap();
    assert!(first.requires_faces_settle);

    // Re-stage so the just-written OWNERS file is tracked (the producer's owners-resolution and
    // git ls-files both read the tracked universe).
    run_git(&repo.root, &["add", "-A"]);

    // Second run: the plan is empty → no edits, no settle.
    let second = register_crate(&repo.root, &req).unwrap();
    assert!(
        second.applied.is_empty(),
        "re-running an already-registered crate must apply nothing, got {:?}",
        second.applied
    );
    assert!(
        !second.requires_faces_settle,
        "a no-op re-run needs no faces settle"
    );
}

// (3) FAIL-CLOSED: an unknown capability is refused by the kernel → RegisterError::Plan.
#[test]
fn unknown_capability_fails_closed() {
    let repo = fixture_tagged("unknown-cap");
    let mut req = base_request();
    req.capability = "totally-made-up".to_owned();
    let err = register_crate(&repo.root, &req).unwrap_err();
    match err {
        RegisterError::Plan(ValidationError::UnknownCapability { capability }) => {
            assert_eq!(capability, "totally-made-up");
        }
        other => panic!("expected Plan(UnknownCapability), got {other:?}"),
    }
    // Fail-closed: no OWNERS file was written (the refusal precedes any dispatch).
    assert!(!repo.exists(&format!("{NEW_DIR}/OWNERS")));
}

// (4) FAIL-CLOSED: a member-glob-uncovered dir → RegisterError::MemberGlobUncovered (never
//     synthesizes a glob — that is a human ADR decision).
#[test]
fn uncovered_member_glob_fails_closed() {
    let repo = fixture_tagged("uncovered");
    // Overwrite the root Cargo.toml with a members glob that does NOT cover the new dir.
    repo.write(
        "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\n    \"libs/oya-*\",\n]\n\n\
         [workspace.package]\nedition = \"2024\"\nversion = \"0.1.0\"\n",
    );
    run_git(&repo.root, &["add", "-A"]);

    let req = base_request();
    let err = register_crate(&repo.root, &req).unwrap_err();
    match err {
        RegisterError::MemberGlobUncovered { dir } => {
            assert_eq!(
                dir, NEW_DIR,
                "the uncovered dir must be named so the human can add a glob"
            );
        }
        other => panic!("expected MemberGlobUncovered, got {other:?}"),
    }
}

// FAIL-CLOSED bonus: a missing owning-ADR file → RegisterError::AdrFileNotFound (the ADR must
// exist before its governed surfaces can be appended).
#[test]
fn missing_adr_file_fails_closed() {
    let repo = fixture_tagged("no-adr");
    fs::remove_file(repo.root.join("docs/adr-archive/ADR-0568-born-accounting-register-crate-registrar-kernel.md")).unwrap();
    run_git(&repo.root, &["add", "-A"]);

    let req = base_request();
    let err = register_crate(&repo.root, &req).unwrap_err();
    match err {
        RegisterError::AdrFileNotFound { adr, adr_dir } => {
            assert_eq!(adr, "ADR-0568");
            assert_eq!(adr_dir, "docs/decisions");
        }
        other => panic!("expected AdrFileNotFound, got {other:?}"),
    }
}

// register_crate_detailed surfaces the edits applied BEFORE a mid-dispatch failure (recovery aid).
#[test]
fn detailed_reports_partial_application_on_dispatch_failure() {
    let repo = fixture_tagged("partial");
    // Remove the ADR file: its governed-paths snapshot reads empty, so the kernel STILL plans the
    // AdrGovernedPathAppend edit. The plan order (with the members glob already covering the dir)
    // is OwnersWrite → AdrGovernedPathAppend → FacesSettle, so OwnersWrite applies first, then the
    // ADR step fails closed in resolve_adr_path (the file is gone).
    fs::remove_file(repo.root.join("docs/adr-archive/ADR-0568-born-accounting-register-crate-registrar-kernel.md")).unwrap();
    run_git(&repo.root, &["add", "-A"]);

    let req = base_request();
    match register_crate_detailed(&repo.root, &req) {
        RegisterOutcome::Failed { error, applied } => {
            assert!(
                matches!(error, RegisterError::AdrFileNotFound { .. }),
                "{error}"
            );
            // OWNERS was applied before the ADR step aborted (the recovery-aid record).
            let kinds: Vec<_> = applied.iter().map(|a| a.kind).collect();
            assert!(kinds.contains(&AppliedEditKind::OwnersWrite), "{kinds:?}");
            assert!(
                !kinds.contains(&AppliedEditKind::AdrGovernedPathAppend),
                "{kinds:?}"
            );
        }
        RegisterOutcome::Done(_) => panic!("expected a dispatch failure"),
    }
}

// ─────────────────────────── Issue B: membership-gate parity ───────────────────────────
// `capability_already_mapped` REUSES the membership gate's `homes_for`, so it sees every home
// source the gate enforces. A crate already mapped by a meta home (app_products → meta:app/,
// meta_directory_absorbs → meta:kernel//os/) or by a `*`-suffix glob is correctly detected as
// MAPPED — no spurious CapabilityMapping edit (which would DOUBLE-MAP it and turn the gate RED).
#[test]
fn meta_and_glob_homes_are_detected_as_already_mapped() {
    let repo = fixture_tagged("meta-homes");

    // app_products → meta:app/ : a crate under oya/application is already mapped.
    assert!(
        capability_already_mapped(&repo.root, "oya/application/some-crate").unwrap(),
        "a crate under an app_products dir (→ meta:app/) must read as already mapped"
    );
    // meta_directory_absorbs → meta:kernel/ : a crate under cloud/cloud-kernel is already mapped.
    assert!(
        capability_already_mapped(&repo.root, "cloud/cloud-kernel/sub").unwrap(),
        "a crate under a meta_directory_absorbs kernel/ dir must read as already mapped"
    );
    // meta_directory_absorbs → meta:os/ : a crate under os/ is already mapped.
    assert!(
        capability_already_mapped(&repo.root, "os/sub").unwrap(),
        "a crate under a meta_directory_absorbs os/ dir must read as already mapped"
    );
    // capabilities[].absorbs_current_dirs : a crate under ci/ (the `ci` capability dir).
    assert!(
        capability_already_mapped(&repo.root, "ci/facade/x").unwrap(),
        "a crate under a capability absorbs_current_dirs dir must read as already mapped"
    );
    // `*`-suffix glob membership : libs/oya-some-* is mapped via the build/ group's glob.
    assert!(
        capability_already_mapped(&repo.root, "libs/oya-some-widget").unwrap(),
        "a crate matching a `*`-suffix crate-glob must read as already mapped (glob_match, not \
         exact-string compare)"
    );
    // A genuinely-unmapped crate (no dir-prefix, no glob match) reads as NOT mapped.
    assert!(
        !capability_already_mapped(&repo.root, "libs/oya-brand-new-thing").unwrap(),
        "a crate in no home must read as NOT mapped"
    );
}

// The EXPRESSIBLE meta homes (app_products/meta_directory_absorbs slugs) join the CapabilitySet so
// a genuinely-unmapped meta crate has a valid capability CHOICE (it is not forced into a wrong
// group). The writer-appliable crate-glob slugs are still present too.
#[test]
fn capability_set_includes_expressible_meta_homes() {
    let repo = fixture_tagged("cap-set");
    let set = load_capability_set(&repo.root).unwrap();
    // Writer-appliable crate-glob slugs.
    assert!(
        set.contains("build/"),
        "build/ (crate-glob group) must be a valid capability: {set:?}"
    );
    assert!(
        set.contains("data"),
        "data (crate-glob group) must be a valid capability: {set:?}"
    );
    // Expressible meta homes.
    assert!(
        set.contains("app/"),
        "app/ (app_products meta) must be expressible: {set:?}"
    );
    assert!(
        set.contains("kernel/"),
        "kernel/ (meta_directory_absorbs) must be expressible: {set:?}"
    );
    assert!(
        set.contains("os/"),
        "os/ (meta_directory_absorbs) must be expressible: {set:?}"
    );
}

// Integration: registering a crate ALREADY home'd by a meta dir (oya/application → meta:app/) must
// NOT emit a CapabilityMapping edit (the drift fix: the old exact-string/2-source check would have
// missed the meta home and double-mapped it).
#[test]
fn meta_homed_crate_emits_no_capability_mapping_edit() {
    let repo = fixture_tagged("meta-no-edit");
    let app_dir = "oya/application/widget-app";
    repo.write(
        &format!("{app_dir}/Cargo.toml"),
        "[package]\nname=\"widget-app\"\n",
    );
    repo.write(&format!("{app_dir}/src/lib.rs"), "//! widget\n");
    // The members glob must cover the new dir so no WorkspaceMemberGlob edit/abort interferes.
    repo.write(
        "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\n    \"libs/oya-*\",\n    \"ci/facade/*\",\n    \"oya/application/*\",\n]\n\n\
         [workspace.package]\nedition = \"2024\"\nversion = \"0.1.0\"\n",
    );
    run_git(&repo.root, &["add", "-A"]);

    let req = RegisterCrateRequest {
        crate_dir: app_dir.to_owned(),
        // The human names the meta home; it is in the CapabilitySet so the kernel accepts it.
        capability: "app/".to_owned(),
        owning_adr: "ADR-0568".to_owned(),
        owner: "cloud-ci-platform".to_owned(),
        role: CrateRole::App,
        has_lib: true,
        has_test_code: true,
        catalog: None,
        extra_governed_paths: Vec::new(),
    };
    let outcome = register_crate(&repo.root, &req).unwrap();
    let kinds: Vec<_> = outcome.applied.iter().map(|a| a.kind).collect();
    assert!(
        !kinds.contains(&AppliedEditKind::CapabilityMapping),
        "a crate already home'd by a meta dir (oya/application → meta:app/) must NOT get a \
         CapabilityMapping edit — that would DOUBLE-MAP it: {kinds:?}"
    );
}

// ─────────────────────────── Issue C: non-oyatie oya-ci.toml ───────────────────────────
// The orchestrator loads the repo's oya-ci.toml (not the compiled-in oyatie default). A non-oyatie
// repo (neutral profile + custom reachability.registry / justification.adr_dir / owners.file_name)
// has its loaders + bridges honour those custom paths, proving universality.
#[test]
fn honours_non_oyatie_oya_ci_toml() {
    let repo = TmpRepo {
        root: unique_root("neutral-cfg"),
    };
    // A non-oyatie config: neutral profile, with custom non-oyatie SSOT paths.
    repo.write(
        "oya-ci.toml",
        "profile = \"neutral\"\n\n\
         [reachability]\nregistry = \"governance/reach.json\"\n\n\
         [justification]\nadr_dir = \"governance/decisions\"\n\n\
         [owners]\nfile_name = \"OWNERS\"\nmax_paths_per_owners_file = 4096\n",
    );
    // Registry + root manifest (oyatie-neutral paths still used for the capability registry, which
    // is not config-driven — only reachability/justification/owners are exercised here).
    repo.write("governance/capability-registry.json", capability_registry());
    repo.write("Cargo.toml", root_cargo_toml());
    // The ADR corpus + reachability registry live at the CUSTOM config paths, NOT the oyatie ones.
    repo.write(
        "governance/decisions/ADR-0568-born-accounting.md",
        stub_adr(),
    );
    repo.write("governance/reach.json", reachability_registry());

    let new_dir = "ci/facade/oya-cloud-ci-neutral-app";
    repo.write(
        &format!("{new_dir}/Cargo.toml"),
        "[package]\nname=\"oya-cloud-ci-neutral-app\"\n",
    );
    repo.write(&format!("{new_dir}/src/lib.rs"), "//! neutral\n");
    repo.git_add_all();

    // A non-crate extra governed path → a ReachabilityEntry edit that must write the CUSTOM registry.
    let extra = "specs/fixtures/neutral/case.json";
    repo.write(extra, "{}\n");
    run_git(&repo.root, &["add", "-A"]);

    let req = RegisterCrateRequest {
        crate_dir: new_dir.to_owned(),
        capability: "build/".to_owned(),
        owning_adr: "ADR-0568".to_owned(),
        owner: "cloud-ci-platform".to_owned(),
        role: CrateRole::App,
        has_lib: true,
        has_test_code: true,
        catalog: None,
        extra_governed_paths: vec![extra.to_owned()],
    };
    let outcome = register_crate(&repo.root, &req).unwrap();
    let kinds: Vec<_> = outcome.applied.iter().map(|a| a.kind).collect();

    // The ADR append resolved the owning ADR at the CUSTOM adr_dir (governance/decisions), proving
    // justification.adr_dir was honoured (else AdrFileNotFound would have aborted).
    assert!(
        kinds.contains(&AppliedEditKind::AdrGovernedPathAppend),
        "{kinds:?}"
    );
    let adr = repo.read("governance/decisions/ADR-0568-born-accounting.md");
    assert!(adr.contains(&format!("{new_dir}/Cargo.toml")), "{adr}");

    // The reachability entry was written to the CUSTOM registry path, proving reachability.registry
    // was honoured (the oyatie default specs/reachability-registry.json was never created).
    assert!(
        kinds.contains(&AppliedEditKind::ReachabilityEntry),
        "{kinds:?}"
    );
    let reach = repo.read("governance/reach.json");
    assert!(
        reach.contains(extra),
        "custom reachability registry must carry the path: {reach}"
    );
    assert!(
        !repo.exists("specs/reachability-registry.json"),
        "the oyatie-default reachability registry must NOT be touched on a neutral-profile repo"
    );
}

// A malformed oya-ci.toml fails CLOSED (loud), never silently reverting to the oyatie default.
#[test]
fn malformed_oya_ci_toml_fails_closed() {
    let repo = fixture_tagged("malformed-cfg");
    repo.write("oya-ci.toml", "this is = = not valid toml [[[\n");
    run_git(&repo.root, &["add", "-A"]);

    let req = base_request();
    let err = register_crate(&repo.root, &req).unwrap_err();
    match err {
        RegisterError::Io(msg) => assert!(
            msg.contains("oya-ci.toml"),
            "a malformed config must fail closed naming oya-ci.toml, got {msg:?}"
        ),
        other => panic!("expected Io(malformed oya-ci.toml), got {other:?}"),
    }
}

// ─────────────────────────── slice 3c: register_crate_and_settle ───────────────────────────
// These exercise the FacesSettle "close the loop" via the injectable RegenPort + a FakeRegenPort,
// so they run with NO buck2 (the Buck2RegenAdapter path is covered by the buck2-gated integration
// test below).

// (3c-1) A real registration (FacesSettle obligation recorded) → register_crate_and_settle EXECUTES
//        the RegenPort (regen + verify_drift both called) and records faces_settled with the written
//        faces + a clean drift verdict.
#[test]
fn settle_runs_regen_and_marks_faces_settled() {
    let repo = fixture_tagged("settle-runs");
    let req = base_request();
    let regen = FakeRegenPort::default();

    let outcome =
        register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::Skip).unwrap();

    // The plan recorded a settle obligation AND the settle ran.
    assert!(
        outcome.requires_faces_settle,
        "a real registration records a FacesSettle obligation"
    );
    let settled = outcome
        .faces_settled
        .expect("faces_settled must be Some after a settle run");
    assert!(settled.drift_clean, "the fake reported no drift");
    // The 6 producer faces + the scm-facts snapshot were recorded as written (sorted).
    assert!(
        settled
            .faces_written
            .contains(&"accounting-registry.generated.json".to_owned()),
        "{:?}",
        settled.faces_written
    );
    assert!(
        settled
            .faces_written
            .contains(&"scm-facts.generated.json".to_owned()),
        "{:?}",
        settled.faces_written
    );
    assert_eq!(
        settled.faces_written.len(),
        7,
        "{:?}",
        settled.faces_written
    );

    // The RegenPort was actually driven: regenerate + verify_drift each ran exactly once, with the
    // repo root.
    assert_eq!(
        regen.regen_calls.borrow().as_slice(),
        std::slice::from_ref(&repo.root)
    );
    assert_eq!(
        regen.verify_calls.borrow().as_slice(),
        std::slice::from_ref(&repo.root)
    );
}

// (3c-1b) Auto-on-birth means Cargo.lock registration is part of the same birth primitive: refresh
//         Cargo.lock BEFORE settling faces so the scm-facts snapshot/producer see the lockfile that
//         `cargo metadata` would commit. This prevents the #833-class skip where a new crate is
//         otherwise born-accounted in SSOTs but still absent from Cargo.lock.
#[test]
fn auto_on_birth_refreshes_cargo_lock_before_settling_faces() {
    let repo = fixture_tagged("settle-lock");
    let req = base_request();
    let regen = FakeRegenPort::default();

    let outcome =
        register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::Skip).unwrap();

    assert!(
        outcome.cargo_lock_refreshed,
        "birth registration must refresh Cargo.lock as part of the same primitive"
    );
    assert_eq!(
        regen.lock_refresh_calls.borrow().as_slice(),
        std::slice::from_ref(&repo.root),
        "cargo metadata must run once for the birth"
    );
    let events = regen.events.borrow().clone();
    assert_eq!(
        events,
        vec![
            "cargo-lock-refresh".to_owned(),
            "faces-regenerate".to_owned(),
            "faces-verify".to_owned(),
        ],
        "Cargo.lock must be refreshed before scm-facts/face regeneration"
    );
}

#[test]
fn cargo_lock_refresh_failure_aborts_before_face_regen() {
    let repo = fixture_tagged("settle-lock-fail");
    let req = base_request();
    let regen = FakeRegenPort {
        fail_lock_refresh: true,
        ..FakeRegenPort::default()
    };

    let err =
        register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::Skip).unwrap_err();
    match err {
        RegisterError::CargoLockRefreshFailed(msg) => {
            assert!(msg.contains("fake cargo metadata failure"), "{msg}");
        }
        other => panic!("expected CargoLockRefreshFailed, got {other:?}"),
    }
    assert_eq!(
        regen.events.borrow().as_slice(),
        &["cargo-lock-refresh".to_owned()],
        "a failed Cargo.lock refresh must stop before any generated face is rewritten"
    );
    assert!(
        regen.regen_calls.borrow().is_empty(),
        "face regeneration must not run after Cargo.lock refresh fails"
    );
    assert!(
        regen.verify_calls.borrow().is_empty(),
        "byte-rediff must not run after Cargo.lock refresh fails"
    );
}

#[test]
fn cargo_lock_refresh_failure_retry_preserves_settle_obligation() {
    let repo = fixture_tagged("settle-lock-retry");
    let req = base_request();
    let failing_regen = FakeRegenPort {
        fail_lock_refresh: true,
        ..FakeRegenPort::default()
    };

    let err = register_crate_and_settle(&repo.root, &req, &failing_regen, ValidationMode::Skip)
        .unwrap_err();
    assert!(
        matches!(err, RegisterError::CargoLockRefreshFailed(_)),
        "expected CargoLockRefreshFailed, got {err:?}"
    );

    // The SSOT edits were applied before the lock refresh failed. Simulate the real recovery hazard:
    // a caller stages those edits, then retries after fixing the manifest/lock cause. The pure plan
    // is now empty, so the missing Cargo.lock entry must carry the settle obligation.
    run_git(&repo.root, &["add", "-A"]);

    let retry_regen = FakeRegenPort::default();
    let outcome =
        register_crate_and_settle(&repo.root, &req, &retry_regen, ValidationMode::Skip).unwrap();

    assert!(
        outcome.requires_faces_settle,
        "a retry after lock-refresh failure must still settle faces even when SSOT edits are already present"
    );
    assert!(
        outcome.cargo_lock_refreshed,
        "retry must rerun Cargo.lock refresh instead of reporting no-op success"
    );
    assert_eq!(
        retry_regen.events.borrow().as_slice(),
        &[
            "cargo-lock-refresh".to_owned(),
            "faces-regenerate".to_owned(),
            "faces-verify".to_owned(),
        ],
    );
}

#[test]
fn cargo_lock_retry_without_parseable_package_name_fails_closed() {
    let repo = fixture_tagged("settle-lock-no-name");
    let req = base_request();
    let failing_regen = FakeRegenPort {
        fail_lock_refresh: true,
        ..FakeRegenPort::default()
    };

    let err = register_crate_and_settle(&repo.root, &req, &failing_regen, ValidationMode::Skip)
        .unwrap_err();
    assert!(
        matches!(err, RegisterError::CargoLockRefreshFailed(_)),
        "expected initial lock refresh failure, got {err:?}"
    );

    repo.write(
        &format!("{NEW_DIR}/Cargo.toml"),
        "[package]\nversion = \"0.1.0\"\n",
    );
    run_git(&repo.root, &["add", "-A"]);

    let retry_regen = FakeRegenPort::default();
    let err = register_crate_and_settle(&repo.root, &req, &retry_regen, ValidationMode::Skip)
        .unwrap_err();

    match err {
        RegisterError::Io(msg) => assert!(
            msg.contains("missing parseable [package] name"),
            "retry must fail closed on unparseable package name, got {msg}"
        ),
        other => panic!("expected Io(missing package name), got {other:?}"),
    }
    assert!(
        retry_regen.lock_refresh_calls.borrow().is_empty(),
        "missing package name must fail before Cargo.lock refresh"
    );
}

// (3c-2) A RegenPort failure → RegenFailed (fail LOUD); verify_drift is NEVER reached.
#[test]
fn settle_propagates_regen_failure() {
    let repo = fixture_tagged("settle-fail");
    let req = base_request();
    let regen = FakeRegenPort {
        fail: true,
        ..FakeRegenPort::default()
    };

    let err =
        register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::Skip).unwrap_err();
    match err {
        RegisterError::RegenFailed(msg) => {
            assert!(msg.contains("fake regen failure"), "{msg}");
        }
        other => panic!("expected RegenFailed, got {other:?}"),
    }
    // regenerate ran (and failed); verify_drift was never reached (fail-closed short-circuit).
    assert_eq!(regen.regen_calls.borrow().len(), 1);
    assert!(
        regen.verify_calls.borrow().is_empty(),
        "verify_drift must not run after a regen failure"
    );
}

// (3c-3) A drift mismatch → DriftDetected naming the drifting face (fail closed before recording a
//        face the registry-drift gate would flag RED).
#[test]
fn settle_drift_mismatch_fails_closed() {
    let repo = fixture_tagged("settle-drift");
    let req = base_request();
    let regen = FakeRegenPort {
        drift_face: Some("ttl-policy.generated.json".to_owned()),
        ..FakeRegenPort::default()
    };

    let err =
        register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::Skip).unwrap_err();
    match err {
        RegisterError::DriftDetected { face } => {
            assert_eq!(face, "ttl-policy.generated.json");
        }
        other => panic!("expected DriftDetected, got {other:?}"),
    }
    // regen ran, then verify_drift ran and reported drift — the obligation was NOT marked settled.
    assert_eq!(regen.regen_calls.borrow().len(), 1);
    assert_eq!(regen.verify_calls.borrow().len(), 1);
}

// (3c-4) A no-op re-run (requires_faces_settle == false) does NOT call the RegenPort and leaves
//        faces_settled None — settling is only triggered by a recorded obligation.
#[test]
fn settle_skips_regen_when_no_obligation() {
    let repo = fixture_tagged("settle-noop");
    let req = base_request();

    // First settle: a real registration that runs the (fake) regen.
    let first_regen = FakeRegenPort::default();
    let first =
        register_crate_and_settle(&repo.root, &req, &first_regen, ValidationMode::Skip).unwrap();
    assert!(
        first.faces_settled.is_some(),
        "the first registration settles the faces"
    );
    assert_eq!(first_regen.regen_calls.borrow().len(), 1);

    // Re-stage the just-written SSOTs so the re-run reads them as already-registered. The fake
    // RegenPort does not execute `cargo metadata`, so mirror the successful production side effect:
    // the package is now represented in Cargo.lock and there is no remaining settle obligation.
    repo.write(
        "Cargo.lock",
        r#"[[package]]
name = "oya-cloud-ci-example-app"
version = "0.1.0"
"#,
    );
    run_git(&repo.root, &["add", "-A"]);

    // Second settle: the plan is empty (idempotent) → NO obligation → regen never runs.
    let second_regen = FakeRegenPort::default();
    let second =
        register_crate_and_settle(&repo.root, &req, &second_regen, ValidationMode::Skip).unwrap();
    assert!(
        !second.requires_faces_settle,
        "a no-op re-run records no FacesSettle obligation"
    );
    assert!(
        !second.cargo_lock_refreshed,
        "a no-op re-run must not refresh Cargo.lock"
    );
    assert!(
        second.faces_settled.is_none(),
        "no obligation ⇒ faces_settled stays None"
    );
    assert!(
        second_regen.lock_refresh_calls.borrow().is_empty(),
        "Cargo.lock refresh must NOT run when there is no settle obligation"
    );
    assert!(
        second_regen.regen_calls.borrow().is_empty(),
        "the RegenPort must NOT be called when there is no settle obligation"
    );
    assert!(second_regen.verify_calls.borrow().is_empty());
}

// ─────────────────────────── slice 3d: fail-closed self-validation ───────────────────────────
// MinimalSubset runs a crate-scoped SUBSET of gates' evaluate_keyed over the POST-settle faces and
// REFUSES success if the just-registered crate would fail any of them. total-accounting +
// capability-membership always; slo-coverage + catalog-liveness only when a CatalogYaml edit applied.
// The subset is driven over the REAL gate evaluate_keyed: total-accounting reads an on-disk face,
// capability-membership re-runs the gate's own collect, slo/catalog faces are injected via the
// FakeRegenPort's gate_faces map (stdout-only producer faces — no buck2).

/// The membership-gate policy the self-validation re-runs collect()+evaluate_keyed over. Mirrors the
/// committed `capability-membership-policy.json` shape (gate_id + registry_path + scan_roots +
/// allowed_top_level_dirs), with a `min_expected_crates` of 0 so the tiny fixture corpus does not
/// trip MEM-EMPTY-SCAN (that finding is keyed `<policy>`, not the crate, so it would be filtered out
/// anyway — but 0 keeps the fixture's intent crisp).
fn membership_policy() -> &'static str {
    r#"{
  "gate_id": "cloud-ci-capability-membership",
  "registry_path": "governance/capability-registry.json",
  "scan_roots": ["cloud", "libs", "oya"],
  "allowed_top_level_dirs": ["cloud", "libs", "oya", "specs", "docs", "registry"],
  "min_expected_crates": 0
}
"#
}

/// Seed the on-disk faces the slice-3d self-validation reads directly (the committed total-accounting
/// `registry` face + the membership-gate policy). `ta_rows` is the JSON body of the total-accounting
/// face (`{"rows":[...], ...}`); pass `{}` / `{"rows":[]}` for a clean face. The slo/catalog faces are
/// NOT seeded here — they are stdout-only producer faces injected via the FakeRegenPort.gate_faces map.
fn seed_self_validation_faces(repo: &TmpRepo, ta_face: &str) {
    repo.write(
        "ci/facade/artifact-inventory-registry/accounting-registry.generated.json",
        ta_face,
    );
    repo.write(
        "ci/facade/module-membership/capability-membership-policy.json",
        membership_policy(),
    );
}

// (3d-1) A registration whose POST-settle faces are CLEAN (no crate-keyed finding) → MinimalSubset
//        returns validation: Some with an EMPTY new_findings set (success). total-accounting +
//        capability-membership run; slo/catalog do NOT (no CatalogYaml edit on the base request).
#[test]
fn self_validation_clean_crate_succeeds_with_empty_findings() {
    let repo = fixture_tagged("sv-clean");
    // A clean total-accounting face (no rows ⇒ no crate-keyed finding). The crate dir maps to the
    // `ci` capability (ci/ absorbs it), so capability-membership emits no crate finding.
    seed_self_validation_faces(&repo, "{\"rows\":[]}\n");
    run_git(&repo.root, &["add", "-A"]);

    let req = base_request();
    let regen = FakeRegenPort::default();

    let outcome =
        register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::MinimalSubset).unwrap();

    assert!(
        outcome.faces_settled.is_some(),
        "MinimalSubset still settles the faces"
    );
    let validation = outcome
        .validation
        .expect("MinimalSubset records a SelfValidation");
    assert!(
        validation.new_findings.is_empty(),
        "a clean crate must produce NO crate-scoped findings: {:?}",
        validation.new_findings
    );
    // The faces-settle path still ran (regen + verify_drift each once).
    assert_eq!(regen.regen_calls.borrow().len(), 1);
    assert_eq!(regen.verify_calls.borrow().len(), 1);
    // No CatalogYaml edit ⇒ slo/catalog gate-input faces were NOT rendered.
    assert!(
        regen.gate_face_calls.borrow().is_empty(),
        "slo/catalog faces must NOT be rendered without a CatalogYaml edit: {:?}",
        regen.gate_face_calls.borrow()
    );
}

// (3d-2) A REAL total-accounting evaluate_keyed finding keyed to a path UNDER the crate dir →
//        SelfValidationFailed (fail-closed). The face is crafted so the gate's own evaluate_keyed
//        emits an `unaccounted` finding keyed under the crate — the wiring is genuinely exercised.
#[test]
fn self_validation_crate_keyed_finding_fails_closed() {
    let repo = fixture_tagged("sv-fail");
    // A tracked path under the crate dir with NO accounting row ⇒ the REAL total-accounting
    // evaluate_keyed emits `unaccounted` keyed to `<NEW_DIR>/orphan.rs` (a path under the crate).
    let ta_face = format!("{{\"rows\":[],\"unaccounted_paths\":[\"{NEW_DIR}/orphan.rs\"]}}\n");
    seed_self_validation_faces(&repo, &ta_face);
    run_git(&repo.root, &["add", "-A"]);

    let req = base_request();
    let regen = FakeRegenPort::default();

    let err = register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::MinimalSubset)
        .unwrap_err();
    match err {
        RegisterError::SelfValidationFailed { findings } => {
            assert!(
                findings.iter().any(|finding| finding.gate
                    == ci_artifact_accountability::GATE_ID
                    && finding.code == "unaccounted"
                    && finding.key == format!("{NEW_DIR}/orphan.rs")),
                "the crate-keyed total-accounting finding must be reported: {findings:?}"
            );
        }
        other => panic!("expected SelfValidationFailed, got {other:?}"),
    }
}

// (3d-3) Pre-existing corpus debt keyed to OTHER paths (outside the crate dir) → NOT flagged. Proves
//        the crate-scope filter: the same real evaluate_keyed emits an `unaccounted` finding, but it
//        is keyed to a foreign path and is correctly filtered out (not new for THIS crate).
#[test]
fn self_validation_ignores_pre_existing_debt_keyed_elsewhere() {
    let repo = fixture_tagged("sv-foreign");
    // An `unaccounted` finding keyed to a path OUTSIDE the crate dir (pre-existing frozen corpus
    // debt). The real total-accounting evaluate_keyed emits it, but the crate-scope filter drops it.
    let ta_face = "{\"rows\":[],\"unaccounted_paths\":[\"some/other/unrelated/path.rs\"]}\n";
    seed_self_validation_faces(&repo, ta_face);
    run_git(&repo.root, &["add", "-A"]);

    let req = base_request();
    let regen = FakeRegenPort::default();

    let outcome =
        register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::MinimalSubset).unwrap();
    let validation = outcome
        .validation
        .expect("MinimalSubset records a SelfValidation");
    assert!(
        validation.new_findings.is_empty(),
        "pre-existing debt keyed to OTHER paths must NOT be flagged for this crate: {:?}",
        validation.new_findings
    );
}

// (3d-4) CatalogYaml-conditional: the slo/catalog stdout-only faces are rendered (and their REAL
//        evaluate_keyed driven) ONLY when a CatalogYaml edit was applied. A crate-keyed slo finding
//        then fails closed; a foreign-keyed one is filtered out.
#[test]
fn self_validation_runs_slo_catalog_only_on_catalog_edit_and_scopes_them() {
    let repo = fixture_tagged("sv-catalog");
    seed_self_validation_faces(&repo, "{\"rows\":[]}\n");
    run_git(&repo.root, &["add", "-A"]);

    // A catalog-bearing request ⇒ a CatalogYaml edit IS applied ⇒ slo/catalog gates run.
    let mut req = base_request();
    req.catalog = Some(CatalogSpec {
        plane: "run".to_owned(),
        slo: "ga-control-plane".to_owned(),
    });

    let catalog_id = NEW_DIR.rsplit('/').next().unwrap();
    // Inject crafted slo + catalog stdout-only faces. The slo face carries a row for the crate's
    // catalog-id with a MISSING slo ⇒ the REAL slo-coverage evaluate_keyed emits
    // `slo_missing_or_blank_slo` keyed to the crate-id. A SECOND row keyed elsewhere is filtered out.
    let mut gate_faces = std::collections::BTreeMap::new();
    gate_faces.insert(
        "slo-coverage".to_owned(),
        serde_json::json!({
            "rows": [
                { "crate_id": catalog_id, "slo": serde_json::Value::Null },
                { "crate_id": "some-other-crate", "slo": serde_json::Value::Null }
            ]
        }),
    );
    // A clean catalog-liveness face for the crate (live row ⇒ no finding).
    gate_faces.insert(
        "catalog-liveness".to_owned(),
        serde_json::json!({
            "rows": [ { "crate_id": catalog_id, "is_live": true, "marker": serde_json::Value::Null } ]
        }),
    );
    let regen = FakeRegenPort {
        gate_faces,
        ..FakeRegenPort::default()
    };

    let err = register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::MinimalSubset)
        .unwrap_err();
    match err {
        RegisterError::SelfValidationFailed { findings } => {
            // The crate-keyed slo finding is reported.
            assert!(
                findings.iter().any(|finding| finding.gate
                    == ci_slo_coverage::GATE_ID
                    && finding.code == "slo_missing_or_blank_slo"
                    && finding.key == catalog_id),
                "the crate-keyed slo-coverage finding must be reported: {findings:?}"
            );
            // The foreign-keyed slo row was filtered out.
            assert!(
                !findings
                    .iter()
                    .any(|finding| finding.key == "some-other-crate"),
                "a slo finding keyed to another crate must NOT be reported: {findings:?}"
            );
        }
        other => panic!("expected SelfValidationFailed, got {other:?}"),
    }
    // Both stdout-only gate-input faces were rendered (slo + catalog), proving the CatalogYaml gate.
    let calls = regen.gate_face_calls.borrow();
    let faces: Vec<&str> = calls.iter().map(|(_, f)| f.as_str()).collect();
    assert!(
        faces.contains(&"slo-coverage"),
        "slo-coverage face must be rendered: {faces:?}"
    );
    assert!(
        faces.contains(&"catalog-liveness"),
        "catalog-liveness face must be rendered: {faces:?}"
    );
}

// (3d-5) ValidationMode::Skip continues to skip self-validation entirely (slice-3c backward compat):
//        validation stays None and the slo/catalog gate-input faces are NEVER rendered.
#[test]
fn self_validation_skip_mode_records_none_and_makes_no_gate_calls() {
    let repo = fixture_tagged("sv-skip");
    // Seed a DIRTY total-accounting face: under MinimalSubset this would fail closed, but Skip must
    // never even look at it (proving Skip makes no self-validation calls).
    let ta_face = format!("{{\"rows\":[],\"unaccounted_paths\":[\"{NEW_DIR}/orphan.rs\"]}}\n");
    seed_self_validation_faces(&repo, &ta_face);
    run_git(&repo.root, &["add", "-A"]);

    let req = base_request();
    let regen = FakeRegenPort::default();

    let outcome =
        register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::Skip).unwrap();
    assert!(
        outcome.faces_settled.is_some(),
        "Skip still settles the faces (3c behavior)"
    );
    assert!(
        outcome.validation.is_none(),
        "Skip records NO SelfValidation (backward compat): {:?}",
        outcome.validation
    );
    assert!(
        regen.gate_face_calls.borrow().is_empty(),
        "Skip must make NO gate-input-face calls: {:?}",
        regen.gate_face_calls.borrow()
    );
}

// (3d-6) MinimalSubset still settles the faces (regen + verify_drift) before self-validating — the
//        self-validation runs AFTER the settle so each gate evaluate_keyed sees the POST-settle faces.
#[test]
fn self_validation_runs_after_faces_settle() {
    let repo = fixture_tagged("sv-after-settle");
    seed_self_validation_faces(&repo, "{\"rows\":[]}\n");
    run_git(&repo.root, &["add", "-A"]);

    let req = base_request();
    let regen = FakeRegenPort::default();

    let outcome =
        register_crate_and_settle(&repo.root, &req, &regen, ValidationMode::MinimalSubset).unwrap();
    // The settle ran (regen + verify_drift each once) AND self-validation recorded a clean result.
    assert_eq!(
        regen.regen_calls.borrow().len(),
        1,
        "regen must run before self-validation"
    );
    assert_eq!(
        regen.verify_calls.borrow().len(),
        1,
        "verify_drift must run before self-validation"
    );
    assert!(
        outcome.validation.is_some(),
        "self-validation ran after the settle"
    );
}

#[test]
fn buck2_regen_adapter_parses_declared_enforcement_liveness_corpus_targets() {
    let output = "\
root//ci/facade/scm-facts-snapshot:oya-cloud-ci-scm-facts-emitter-app buck-out/v2/gen/emitter\n\
root//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin buck-out/v2/gen/producer\n\
root//tools/oya-reorg-codemod-app:oya-reorg-codemod buck-out/v2/gen/codemod\n\
root//.claude:settings-json buck-out/v2/gen/.claude/__settings-json__/settings-json\n\
root//.codex:hooks-json buck-out/v2/gen/.codex/__hooks-json__/hooks-json\n\
root//tools/hooks:top-level-hook-scripts buck-out/v2/gen/tools/hooks/__top-level-hook-scripts__/top-level-hook-scripts\n\
";

    let corpus =
        parse_enforcement_liveness_corpus_paths(Path::new("/repo"), output).expect("corpus paths");

    assert_eq!(
        corpus.claude_settings,
        PathBuf::from(
            "/repo/buck-out/v2/gen/.claude/__settings-json__/settings-json/settings.json"
        )
    );
    assert_eq!(
        corpus.codex_hooks,
        PathBuf::from("/repo/buck-out/v2/gen/.codex/__hooks-json__/hooks-json/hooks.json")
    );
    assert_eq!(
        corpus.hooks_dir,
        PathBuf::from(
            "/repo/buck-out/v2/gen/tools/hooks/__top-level-hook-scripts__/top-level-hook-scripts"
        )
    );

    let mut command = Command::new("/tmp/producer");
    append_enforcement_liveness_corpus_args(&mut command, &corpus);
    let args: Vec<String> = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    assert!(args.contains(&corpus.claude_settings.display().to_string()));
    assert!(args.contains(&corpus.codex_hooks.display().to_string()));
    assert!(args.contains(&corpus.hooks_dir.display().to_string()));
    assert!(!args.contains(&"/repo/.claude/settings.json".to_owned()));
    assert!(!args.contains(&"/repo/.codex/hooks.json".to_owned()));
    assert!(!args.contains(&"/repo/tools/hooks".to_owned()));
}

#[test]
fn crate_registration_delegates_move_plan_selection_to_codemod() {
    let mut command = Command::new("/tmp/codemod");
    append_manifest_args(
        &mut command,
        Path::new("/repo"),
        Path::new("/repo/specs/reorg/move-manifest.generated.json"),
    );

    let args: Vec<String> = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        args,
        [
            "manifest",
            "--repo-root",
            "/repo",
            "--out",
            "/repo/specs/reorg/move-manifest.generated.json",
        ]
    );
    assert!(!args.iter().any(|arg| arg == "--plan"));
}

// (3c-6, buck2-gated) The REAL Buck2RegenAdapter against the live checkout. Ignored by default so
// the std-only unit CI never needs buck2; run explicitly with `--ignored` (buck2 pre-approved).
//
// NON-MUTATING by design: it exercises ONLY `Buck2RegenAdapter::verify_drift` — the byte-rediff —
// which re-renders each producer face from the candidate tree and byte-compares to the COMMITTED
// face WITHOUT writing anything. On a settled tree (origin/dev) the rediff is clean. This proves
// the real `build_face_tools` (`buck2 build --show-output` parse) + the producer `--stdout --face`
// path + the byte-compare all work against actual built binaries — the path FakeRegenPort stands in
// for above — without seeding OWNERS/ADR into or otherwise mutating the live checkout (`regenerate`,
// which DOES write the faces, is covered by the FakeRegenPort + materialize.sh verification instead).
#[test]
#[ignore = "requires buck2 + the full candidate tree; run explicitly when buck2 is available"]
fn buck2_regen_adapter_byte_rediff_is_clean_on_settled_tree() {
    // The byte-rediff runs against THIS repo checkout (a full tree with the buck2 targets + the
    // committed settled faces) — the emitter/producer need the whole tracked tree.
    let repo_root = repo_root_for_integration();
    let regen = Buck2RegenAdapter;
    match regen.verify_drift(&repo_root) {
        Ok(()) => {}
        Err(RegisterError::DriftDetected { face }) => {
            panic!("committed face {face} drifted from a fresh re-render on a settled tree")
        }
        Err(e) => panic!("byte-rediff failed: {e}"),
    }
}

#[cfg(test)]
fn repo_root_for_integration() -> PathBuf {
    // Discover the repo root by walking up from CWD until the FACES_DIR + buck2 root markers exist.
    // `option_env!("CARGO_MANIFEST_DIR")` (set under cargo, absent under buck2) is a HINT only — we
    // never `env!` it, so the buck2 unittest target compiles regardless. Used only by the
    // #[ignore]d integration test, so a best-effort discovery is sufficient.
    let start = option_env!("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    for dir in start.ancestors() {
        if dir.join(super::FACES_DIR).is_dir() && dir.join(".buckconfig").exists() {
            return dir.to_path_buf();
        }
    }
    start
}
