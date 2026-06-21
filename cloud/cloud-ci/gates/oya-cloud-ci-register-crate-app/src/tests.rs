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
    std::env::temp_dir().join(format!("register-crate-{tag}-{nonce}-{:?}", std::thread::current().id()))
}

/// The new crate dir under test: a cloud-ci gate-tool crate (absorbed by the `cloud/cloud-ci` dir).
const NEW_DIR: &str = "cloud/cloud-ci/gates/oya-cloud-ci-example-app";

/// A minimal capability-registry: one crate-glob group (`build/`) for the closed CapabilitySet,
/// plus a capability that absorbs `cloud/cloud-ci` (so the new gate crate is already
/// capability-mapped by its dir — exactly the producer's situation).
fn capability_registry() -> &'static str {
    r#"{
  "capabilities": [
    {
      "id": "ci",
      "absorbs_current_dirs": ["cloud/cloud-ci"]
    },
    {
      "id": "data",
      "absorbs_current_dirs": ["cloud/cloud-data"]
    }
  ],
  "membership_lint_coverage": {
    "absorbs_current_crate_globs": [
      {
        "meta_dir": "build/",
        "globs": ["libs/oya-some-kernel"]
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

/// A root Cargo.toml whose members glob `cloud/cloud-ci/gates/*` covers the new crate dir.
fn root_cargo_toml() -> &'static str {
    r#"[workspace]
resolver = "2"
members = [
    "libs/oya-*",
    "cloud/cloud-ci/gates/*",
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
    let repo = TmpRepo { root: unique_root(tag) };
    repo.write("specs/capability-registry.json", capability_registry());
    repo.write("Cargo.toml", root_cargo_toml());
    repo.write("docs/decisions/ADR-0568-born-accounting.md", stub_adr());
    repo.write("specs/reachability-registry.json", reachability_registry());

    // The new crate's intrinsic source (Cargo.toml + lib.rs) so it is a real dir + git-tracked.
    repo.write(&format!("{NEW_DIR}/Cargo.toml"), "[package]\nname = \"oya-cloud-ci-example-app\"\n");
    repo.write(&format!("{NEW_DIR}/src/lib.rs"), "//! example\n");

    // An existing sibling so the gate tree is non-trivial.
    repo.write("cloud/cloud-ci/gates/oya-cloud-ci-some-app/Cargo.toml", "[package]\nname=\"x\"\n");

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
    assert!(outcome.requires_faces_settle, "faces must need a settle after a real registration");

    let kinds: Vec<_> = outcome.applied.iter().map(|a| a.kind).collect();
    // The kernel's plan is the DIFF vs the live snapshot: it emits an edit ONLY for an SSOT not
    // already satisfied. In this fixture:
    //   - OWNERS is absent          → OwnersWrite IS emitted + dispatched.
    //   - the ADR block is empty    → AdrGovernedPathAppend IS emitted + dispatched.
    //   - the members glob ALREADY covers the dir → NO WorkspaceMemberGlob edit (already covered).
    //   - the dir is absorbed by the `cloud/cloud-ci` capability → NO CapabilityMapping edit.
    //   - no catalog requested      → NO CatalogYaml edit.
    assert!(kinds.contains(&AppliedEditKind::OwnersWrite), "{kinds:?}");
    assert!(kinds.contains(&AppliedEditKind::AdrGovernedPathAppend), "{kinds:?}");
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
    assert_eq!(repo.read(&format!("{NEW_DIR}/OWNERS")), "cloud-ci-platform\n");

    // The ADR now enumerates the crate's conventional governed paths verbatim.
    let adr = repo.read("docs/decisions/ADR-0568-born-accounting.md");
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
    repo.write(&format!("{libs_dir}/Cargo.toml"), "[package]\nname=\"oya-new-thing-kernel\"\n");
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
    assert!(cap.changed, "the registry must be rewritten with the new mapping");
    assert_eq!(cap.path, "specs/capability-registry.json");

    // The dir is now in the `build/` group's globs.
    let registry = repo.read("specs/capability-registry.json");
    assert!(registry.contains(libs_dir), "registry must list the new dir: {registry}");
}

// CatalogYaml + ReachabilityEntry dispatch: a catalog-bearing crate with a non-crate extra
// governed path drives both the catalog writer and the producer's fix_reachability bridge.
#[test]
fn catalog_and_reachability_dispatch() {
    let repo = fixture_tagged("cat-reach");
    let mut req = base_request();
    req.catalog = Some(CatalogSpec { plane: "run".to_owned(), slo: "ga-control-plane".to_owned() });
    // A non-crate governed path (outside the crate dir) → a ReachabilityEntry edit.
    let extra = "specs/fixtures/register-crate/example-case.json";
    repo.write(extra, "{}\n");
    req.extra_governed_paths = vec![extra.to_owned()];
    run_git(&repo.root, &["add", "-A"]);

    let outcome = register_crate(&repo.root, &req).unwrap();
    let kinds: Vec<_> = outcome.applied.iter().map(|a| a.kind).collect();
    assert!(kinds.contains(&AppliedEditKind::CatalogYaml), "{kinds:?}");
    assert!(kinds.contains(&AppliedEditKind::ReachabilityEntry), "{kinds:?}");

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
            assert_eq!(dir, NEW_DIR, "the uncovered dir must be named so the human can add a glob");
        }
        other => panic!("expected MemberGlobUncovered, got {other:?}"),
    }
}

// FAIL-CLOSED bonus: a missing owning-ADR file → RegisterError::AdrFileNotFound (the ADR must
// exist before its governed surfaces can be appended).
#[test]
fn missing_adr_file_fails_closed() {
    let repo = fixture_tagged("no-adr");
    fs::remove_file(repo.root.join("docs/decisions/ADR-0568-born-accounting.md")).unwrap();
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
    fs::remove_file(repo.root.join("docs/decisions/ADR-0568-born-accounting.md")).unwrap();
    run_git(&repo.root, &["add", "-A"]);

    let req = base_request();
    match register_crate_detailed(&repo.root, &req) {
        RegisterOutcome::Failed { error, applied } => {
            assert!(matches!(error, RegisterError::AdrFileNotFound { .. }), "{error}");
            // OWNERS was applied before the ADR step aborted (the recovery-aid record).
            let kinds: Vec<_> = applied.iter().map(|a| a.kind).collect();
            assert!(kinds.contains(&AppliedEditKind::OwnersWrite), "{kinds:?}");
            assert!(!kinds.contains(&AppliedEditKind::AdrGovernedPathAppend), "{kinds:?}");
        }
        RegisterOutcome::Done(_) => panic!("expected a dispatch failure"),
    }
}
