//! End-to-end fixture proof of the capability-move codemod (ADR-0562 P0.13 CRITICAL scope).
//!
//! Builds a SYNTHETIC 3-crate tree (mirroring the real shapes: a deep `../../../` move-fatal
//! path-dep, an absolute BUCK label, a kebab->snake Rust import), moves it with the codemod,
//! and asserts:
//!   (a) `cargo metadata` resolves post-move (and would buck2-resolve; the BUCK labels are
//!       rewritten and asserted textually);
//!   (b) the inverse restores every file and symlink BYTE-IDENTICALLY; empty-directory
//!       provenance across independent `apply_plan` calls is intentionally out of scope;
//!   (c) the dry-run gate PASSES a clean move and FAILS a move that would break resolution.
//!
//! No real capability is moved; the fixture is a throwaway temp tree.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use oya_reorg_codemod_app::model::{CrateMove, MovePlan};
use oya_reorg_codemod_app::oracle;
use oya_reorg_codemod_app::plan::{ApplyOptions, apply_plan};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp_root(tag: &str) -> PathBuf {
    let unique = format!(
        "oya-reorg-fixture-{}-{}-{}",
        tag,
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let root = std::env::temp_dir().join(unique);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn w(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn r(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

/// Snapshot entry type and content. Symlinks retain their raw link target and are never followed.
#[derive(Clone, Debug, PartialEq, Eq)]
enum SnapshotEntry {
    Directory,
    File(Vec<u8>),
    Symlink(PathBuf),
}

/// Snapshot the full fixture tree for byte-and-type comparison without following symlinks.
fn snapshot_tree(root: &Path) -> BTreeMap<String, SnapshotEntry> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if metadata.file_type().is_symlink() {
                out.insert(
                    rel,
                    SnapshotEntry::Symlink(std::fs::read_link(&path).unwrap()),
                );
            } else if metadata.is_dir() {
                if name == ".git" || name == "target" {
                    continue;
                }
                out.insert(rel, SnapshotEntry::Directory);
                stack.push(path);
            } else {
                out.insert(rel, SnapshotEntry::File(std::fs::read(&path).unwrap()));
            }
        }
    }
    out
}

fn non_directory_entries(
    snapshot: &BTreeMap<String, SnapshotEntry>,
) -> BTreeMap<String, SnapshotEntry> {
    snapshot
        .iter()
        .filter(|(_, entry)| !matches!(entry, SnapshotEntry::Directory))
        .map(|(path, entry)| (path.clone(), entry.clone()))
        .collect()
}

#[cfg(unix)]
#[test]
fn snapshot_tree_models_files_directories_and_dangling_symlinks_without_following_them() {
    use std::os::unix::fs::symlink;

    let root = tmp_root("snapshot-model");
    w(&root, "file.txt", "file bytes\n");
    w(&root, "target-dir/inside.txt", "must not be followed\n");
    std::fs::create_dir_all(root.join("empty-dir")).unwrap();
    symlink("file.txt", root.join("file-link")).unwrap();
    symlink("target-dir", root.join("directory-link")).unwrap();
    symlink("missing-target", root.join("dangling-link")).unwrap();

    let snapshot = snapshot_tree(&root);
    assert_eq!(
        snapshot.get("file.txt"),
        Some(&SnapshotEntry::File(b"file bytes\n".to_vec()))
    );
    assert_eq!(snapshot.get("empty-dir"), Some(&SnapshotEntry::Directory));
    assert_eq!(
        snapshot.get("file-link"),
        Some(&SnapshotEntry::Symlink(PathBuf::from("file.txt")))
    );
    assert_eq!(
        snapshot.get("directory-link"),
        Some(&SnapshotEntry::Symlink(PathBuf::from("target-dir")))
    );
    assert_eq!(
        snapshot.get("dangling-link"),
        Some(&SnapshotEntry::Symlink(PathBuf::from("missing-target")))
    );
    assert!(
        !snapshot.contains_key("directory-link/inside.txt"),
        "snapshot must not follow a symlinked directory"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn validation_failure_preserves_the_entire_tree_snapshot() {
    let root = tmp_root("validation-no-mutation");
    w(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/*\", \"libs/*\"]\nresolver = \"2\"\n",
    );
    w(
        &root,
        "crates/oya-widget/Cargo.toml",
        "[package]\nname = \"oya-widget\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    w(
        &root,
        "crates/oya-widget/src/lib.rs",
        "pub fn widget() {}\n",
    );
    w(
        &root,
        "docs/unchanged.md",
        "unrelated file stays byte-identical\n",
    );
    std::fs::create_dir_all(root.join("libs/missing-manifest"))
        .expect("create invalid workspace member");
    std::fs::create_dir_all(root.join("preexisting-empty-dir"))
        .expect("create pre-existing empty directory");
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        w(&root, "symlink-target/file.txt", "target bytes\n");
        symlink("docs/unchanged.md", root.join("file-link")).expect("create file symlink");
        symlink("symlink-target", root.join("directory-link")).expect("create directory symlink");
        symlink("missing-target", root.join("dangling-link")).expect("create dangling symlink");
    }

    let plan = MovePlan {
        capability: "widget".to_owned(),
        moves: vec![CrateMove {
            old_path: "crates/oya-widget".to_owned(),
            new_path: "widget/core/widget".to_owned(),
            old_cargo_name: "oya-widget".to_owned(),
            new_cargo_name: "widget-domain".to_owned(),
        }],
        artifacts: vec![],
    };
    let before = snapshot_tree(&root);

    let error = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false })
        .expect_err("invalid workspace membership must fail before any codemod write");

    assert!(
        error.to_string().contains("libs/missing-manifest"),
        "the resolver failure must retain its matched path: {error}"
    );
    assert_eq!(
        snapshot_tree(&root),
        before,
        "validation failure must preserve files, empty directories, and symlinks exactly"
    );

    let _ = std::fs::remove_dir_all(root);
}

/// The synthetic capability tree:
///   crates/oya-cap-core/      (engine)   depends on libs/oya-shared-kernel via ../../libs/...
///   crates/oya-cap-app/       (facade)   depends on oya-cap-core (sibling, ../oya-cap-core)
///                                          AND on libs/oya-shared-kernel via ../../libs/...
///   crates/oya-bystander/     (stays put — keeps crates/* non-empty so no members rewrite)
///   libs/oya-shared-kernel/   (stays put)
///
/// The move re-homes the two cap crates into cap/{core,facade}/ with de-branded names; the
/// shared kernel + bystander do NOT move (the cap crates' deep path-deps must be RECOMPUTED,
/// and the bystander's manifest/source must stay byte-identical).
fn build_fixture(root: &Path) {
    // The bystander keeps crates/* non-empty (no glob prune fires); cap/ has no covering glob,
    // so the forward move ADDS the cap dirs as literal members and the inverse REMOVES exactly
    // those literals -> the root manifest round-trips byte-identically. (The empty-glob PRUNE
    // behavior is exercised separately by `forward_move_prunes_emptied_members_glob`.)
    w(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/*\", \"libs/*\"]\nresolver = \"2\"\n",
    );
    // .buckconfig + root BUCK so buck2 targets //... has a graph (used only when --with-buck).
    w(root, ".buckconfig", "[cells]\n  root = .\n");

    // crates/oya-bystander (does not move; proves untouched crates stay byte-identical).
    w(
        root,
        "crates/oya-bystander/Cargo.toml",
        "[package]\nname = \"oya-bystander\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    w(
        root,
        "crates/oya-bystander/src/lib.rs",
        "pub fn bystand() {}\n",
    );

    // libs/oya-shared-kernel (does not move)
    w(
        root,
        "libs/oya-shared-kernel/Cargo.toml",
        "[package]\nname = \"oya-shared-kernel\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    w(
        root,
        "libs/oya-shared-kernel/src/lib.rs",
        "pub fn root() {}\n",
    );
    w(
        root,
        "libs/oya-shared-kernel/BUCK",
        "rust_library(\n    name = \"oya-shared-kernel\",\n    crate = \"oya_shared_kernel\",\n    crate_root = \"src/lib.rs\",\n    srcs = [\"src/lib.rs\"],\n)\n",
    );

    // crates/oya-cap-core (engine) -> moves to cap/core/cap-core
    w(
        root,
        "crates/oya-cap-core/Cargo.toml",
        "[package]\nname = \"oya-cap-core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"oya_cap_core\"\npath = \"src/lib.rs\"\n\n[dependencies]\noya-shared-kernel = { path = \"../../libs/oya-shared-kernel\" }\n",
    );
    w(
        root,
        "crates/oya-cap-core/src/lib.rs",
        "use oya_shared_kernel::root;\npub fn engine() { root(); }\n",
    );
    w(
        root,
        "crates/oya-cap-core/BUCK",
        "rust_library(\n    name = \"oya-cap-core\",\n    crate = \"oya_cap_core\",\n    crate_root = \"src/lib.rs\",\n    srcs = [\"src/lib.rs\"],\n    deps = [\"//libs/oya-shared-kernel:oya-shared-kernel\"],\n)\n",
    );

    // crates/oya-cap-app (facade) -> moves to cap/facade/cap-app
    w(
        root,
        "crates/oya-cap-app/Cargo.toml",
        "[package]\nname = \"oya-cap-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"oya_cap_app\"\npath = \"src/lib.rs\"\n\n[dependencies]\noya-cap-core = { path = \"../oya-cap-core\" }\noya-shared-kernel = { path = \"../../libs/oya-shared-kernel\" }\n",
    );
    w(
        root,
        "crates/oya-cap-app/src/lib.rs",
        "use oya_cap_core::engine;\nuse oya_shared_kernel::root;\npub fn facade() { engine(); root(); }\n",
    );
    w(
        root,
        "crates/oya-cap-app/BUCK",
        "rust_library(\n    name = \"oya-cap-app\",\n    crate = \"oya_cap_app\",\n    crate_root = \"src/lib.rs\",\n    srcs = [\"src/lib.rs\"],\n    deps = [\n        \"//crates/oya-cap-core:oya-cap-core\",\n        \"//libs/oya-shared-kernel:oya-shared-kernel\",\n    ],\n)\n",
    );
}

fn capability_plan() -> MovePlan {
    MovePlan {
        capability: "cap".to_string(),
        moves: vec![
            CrateMove {
                old_path: "crates/oya-cap-core".to_string(),
                new_path: "cap/core/cap-core".to_string(),
                old_cargo_name: "oya-cap-core".to_string(),
                new_cargo_name: "cap-core".to_string(),
            },
            CrateMove {
                old_path: "crates/oya-cap-app".to_string(),
                new_path: "cap/facade/cap-app".to_string(),
                old_cargo_name: "oya-cap-app".to_string(),
                new_cargo_name: "cap-app".to_string(),
            },
        ],
        artifacts: vec![],
    }
}

#[test]
fn forward_move_recomputes_paths_and_resolves() {
    let root = tmp_root("fwd");
    build_fixture(&root);
    let plan = capability_plan();

    let outcome = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false }).unwrap();

    // (1) directories moved.
    assert!(root.join("cap/core/cap-core/Cargo.toml").is_file());
    assert!(root.join("cap/facade/cap-app/Cargo.toml").is_file());
    assert!(!root.join("crates/oya-cap-core").exists());
    assert!(!root.join("crates/oya-cap-app").exists());

    // (2) package names rewritten (kebab) + lib snake mirror.
    let core_manifest = r(&root, "cap/core/cap-core/Cargo.toml");
    assert!(core_manifest.contains("name = \"cap-core\""));
    assert!(core_manifest.contains("name = \"cap_core\"")); // [lib].name snake mirror

    // (3) the deep move-fatal path-dep RECOMPUTED: cap/core/cap-core -> libs/oya-shared-kernel
    //     = ../../../libs/oya-shared-kernel
    assert!(
        core_manifest.contains("path = \"../../../libs/oya-shared-kernel\""),
        "recomputed deep path-dep: {core_manifest}"
    );

    // (4) sibling dep recomputed + key renamed: cap-app depends on cap-core; both moved into
    //     cap/{core,facade} so the relative path is ../../core/cap-core.
    let app_manifest = r(&root, "cap/facade/cap-app/Cargo.toml");
    assert!(
        app_manifest.contains("cap-core = "),
        "renamed dep key: {app_manifest}"
    );
    assert!(!app_manifest.contains("oya-cap-core ="));
    assert!(
        app_manifest.contains("path = \"../../core/cap-core\""),
        "recomputed sibling path: {app_manifest}"
    );
    assert!(
        app_manifest.contains("path = \"../../../libs/oya-shared-kernel\""),
        "recomputed deep path-dep in facade: {app_manifest}"
    );

    // (5) Rust import idents rewritten kebab->snake.
    let app_src = r(&root, "cap/facade/cap-app/src/lib.rs");
    assert!(
        app_src.contains("use cap_core::engine;"),
        "rust import: {app_src}"
    );
    assert!(!app_src.contains("oya_cap_core"));

    // (6) BUCK labels rewritten (absolute path + target) + own name/crate.
    let app_buck = r(&root, "cap/facade/cap-app/BUCK");
    assert!(app_buck.contains("name = \"cap-app\""));
    assert!(app_buck.contains("crate = \"cap_app\""));
    assert!(
        app_buck.contains("//cap/core/cap-core:cap-core"),
        "rewritten absolute label: {app_buck}"
    );
    // the unmoved shared-kernel label is untouched.
    assert!(app_buck.contains("//libs/oya-shared-kernel:oya-shared-kernel"));

    // (7) root workspace gets the new cap dirs as literal members (no covering glob existed);
    //     crates/* stays (bystander keeps it non-empty), so the post-move set resolves.
    let root_manifest = r(&root, "Cargo.toml");
    assert!(outcome.root_workspace_changed);
    assert!(root_manifest.contains("cap/core/cap-core"));
    assert!(root_manifest.contains("cap/facade/cap-app"));
    assert!(
        root_manifest.contains("crates/*"),
        "bystander keeps crates/* alive"
    );

    // (8) cargo metadata resolves the moved workspace.
    let snap = oracle::capture_snapshot(&root, false);
    assert!(
        snap.cargo_ok,
        "post-move cargo metadata failed: {}",
        snap.cargo_metadata
    );
    assert!(snap.cargo_metadata.contains("cap-core"));
    assert!(snap.cargo_metadata.contains("cap-app"));

    // (9) emitted mapping carries the 5-tuple including buck_label.
    let core_row = outcome
        .mapping
        .rows
        .iter()
        .find(|row| row.new_cargo_name == "cap-core")
        .unwrap();
    assert_eq!(core_row.buck_label, "//cap/core/cap-core:cap-core");

    let _ = std::fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn successful_apply_does_not_follow_an_outside_root_directory_symlink() {
    use std::os::unix::fs::symlink;

    let root = tmp_root("outside-root-symlink");
    let outside = tmp_root("outside-root-sentinel");
    build_fixture(&root);
    w(
        &outside,
        "sentinel.rs",
        "use oya_cap_core::engine;\npub fn sentinel() { engine(); }\n",
    );
    let sentinel_before = r(&outside, "sentinel.rs");
    symlink(&outside, root.join("outside-root")).expect("create outside-root symlink");

    apply_plan(
        &root,
        &capability_plan(),
        &ApplyOptions { use_git_mv: false },
    )
    .expect("ordinary fixture move must succeed");

    assert_eq!(
        r(&outside, "sentinel.rs"),
        sentinel_before,
        "successful apply must not rewrite files reached only through an outside-root symlink"
    );

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn inverse_restores_file_and_symlink_content_but_not_empty_directory_provenance() {
    let root = tmp_root("inv");
    build_fixture(&root);
    let before = snapshot_tree(&root);

    let plan = capability_plan();
    apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false }).unwrap();

    // Apply the inverse.
    apply_plan(&root, &plan.inverse(), &ApplyOptions { use_git_mv: false }).unwrap();

    let after = snapshot_tree(&root);
    assert_eq!(
        non_directory_entries(&before),
        non_directory_entries(&after),
        "files and symlink targets must be identical after inverse; empty-directory provenance is not tracked across independent apply calls"
    );

    // And cargo still resolves the restored tree.
    let snap = oracle::capture_snapshot(&root, false);
    assert!(
        snap.cargo_ok,
        "restored cargo metadata failed: {}",
        snap.cargo_metadata
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn dry_run_passes_a_clean_capability_move_without_landing() {
    let root = tmp_root("dryok");
    build_fixture(&root);
    let plan = capability_plan();
    let before = snapshot_tree(&root);

    let report = oracle::dry_run(&root, &plan, false, false).unwrap();
    assert!(
        report.clean,
        "expected clean dry-run; cargo={}",
        report.cargo_detail
    );
    assert!(report.cargo_ok);

    // The real tree was NOT modified (dry-run is shadow-only).
    let after = snapshot_tree(&root);
    assert_eq!(before, after, "dry-run must not modify the real tree");
    assert!(root.join("crates/oya-cap-core").is_dir());
    assert!(!root.join("cap").exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn dry_run_fails_a_move_that_would_break_resolution() {
    let root = tmp_root("dryfail");
    build_fixture(&root);

    // Introduce an UNFIXABLE break: cap-app depends on a crate OUTSIDE the plan via a path
    // that becomes invalid post-move and that the engine will not recompute (the dep target
    // is not in the move plan, and the manifest itself moves, so the now-relative path no
    // longer resolves). We point cap-app at a sibling crate `ghost` that we then make the
    // move strand by NOT including ghost in the plan AND placing ghost where the recomputed
    // relative path cannot reach a real crate.
    //
    // Simplest deterministic break: add a dependency on a non-existent path. Post-move, the
    // recompute still yields a path to a directory with no Cargo.toml -> cargo metadata fails.
    w(
        &root,
        "crates/oya-cap-app/Cargo.toml",
        "[package]\nname = \"oya-cap-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"oya_cap_app\"\npath = \"src/lib.rs\"\n\n[dependencies]\noya-cap-core = { path = \"../oya-cap-core\" }\nphantom = { path = \"../oya-phantom-missing\" }\n",
    );

    let plan = capability_plan();
    let report = oracle::dry_run(&root, &plan, false, false).unwrap();
    assert!(
        !report.clean,
        "expected dry-run to FAIL: the phantom path-dep makes the workspace non-resolving"
    );
    assert!(
        !report.cargo_ok,
        "cargo metadata must fail on the broken graph"
    );

    // Critically, the failing dry-run did NOT land anything (fail-closed safety).
    assert!(root.join("crates/oya-cap-app").is_dir());
    assert!(!root.join("cap").exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn validate_rejects_target_collision_before_any_move() {
    let root = tmp_root("collide");
    build_fixture(&root);

    // A plan that maps two crates onto the SAME new cargo name -> fail-closed, no move.
    let bad = MovePlan {
        capability: "cap".to_string(),
        moves: vec![
            CrateMove {
                old_path: "crates/oya-cap-core".to_string(),
                new_path: "cap/core/x".to_string(),
                old_cargo_name: "oya-cap-core".to_string(),
                new_cargo_name: "dupe".to_string(),
            },
            CrateMove {
                old_path: "crates/oya-cap-app".to_string(),
                new_path: "cap/facade/y".to_string(),
                old_cargo_name: "oya-cap-app".to_string(),
                new_cargo_name: "dupe".to_string(),
            },
        ],
        artifacts: vec![],
    };
    let err = apply_plan(&root, &bad, &ApplyOptions { use_git_mv: false }).unwrap_err();
    assert!(err.to_string().contains("duplicate"), "fail-closed: {err}");
    // nothing moved.
    assert!(root.join("crates/oya-cap-core").is_dir());
    assert!(!root.join("cap").exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn forward_move_prunes_emptied_members_glob_and_still_resolves() {
    // A tree where the moved crates are the ONLY members of crates/*; moving them empties the
    // glob. The codemod must PRUNE crates/* (a stale glob makes cargo error) and add the cap
    // dirs, keeping the workspace resolvable.
    let root = tmp_root("prune");
    // members WITHOUT a cap/*/* glob and WITHOUT a bystander -> crates/* will empty.
    w(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/*\", \"libs/*\"]\nresolver = \"2\"\n",
    );
    w(
        &root,
        "libs/oya-shared-kernel/Cargo.toml",
        "[package]\nname = \"oya-shared-kernel\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    w(
        &root,
        "libs/oya-shared-kernel/src/lib.rs",
        "pub fn root() {}\n",
    );
    w(
        &root,
        "crates/oya-cap-core/Cargo.toml",
        "[package]\nname = \"oya-cap-core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"oya_cap_core\"\npath = \"src/lib.rs\"\n\n[dependencies]\noya-shared-kernel = { path = \"../../libs/oya-shared-kernel\" }\n",
    );
    w(
        &root,
        "crates/oya-cap-core/src/lib.rs",
        "use oya_shared_kernel::root;\npub fn engine() { root(); }\n",
    );

    let plan = MovePlan {
        capability: "cap".to_string(),
        moves: vec![CrateMove {
            old_path: "crates/oya-cap-core".to_string(),
            new_path: "cap/core/cap-core".to_string(),
            old_cargo_name: "oya-cap-core".to_string(),
            new_cargo_name: "cap-core".to_string(),
        }],
        artifacts: vec![],
    };
    let outcome = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false }).unwrap();
    assert!(outcome.root_workspace_changed);

    let manifest = r(&root, "Cargo.toml");
    assert!(
        !manifest.contains("crates/*"),
        "emptied glob pruned: {manifest}"
    );
    assert!(
        manifest.contains("cap/core/cap-core"),
        "new dir added: {manifest}"
    );

    // The pruned + extended workspace resolves.
    let snap = oracle::capture_snapshot(&root, false);
    assert!(
        snap.cargo_ok,
        "post-prune cargo metadata failed: {}",
        snap.cargo_metadata
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn git_mv_path_preserves_history_in_a_real_git_repo() {
    // Prove the use_git_mv=true path works in an actual git repo (history preservation is the
    // point of git mv; we assert the move lands + git tracks it as a rename).
    let root = tmp_root("gitmv");
    build_fixture(&root);

    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(args)
            .output()
            .expect("git")
    };
    assert!(git(&["init", "-q"]).status.success());
    let _ = git(&["config", "user.email", "fixture@example.com"]);
    let _ = git(&["config", "user.name", "Fixture"]);
    assert!(git(&["add", "-A"]).status.success());
    assert!(git(&["commit", "-q", "-m", "seed"]).status.success());

    let plan = capability_plan();
    apply_plan(&root, &plan, &ApplyOptions { use_git_mv: true }).unwrap();

    // git status should show renames (R) for the moved files, not delete+add.
    let status = git(&["status", "--porcelain"]);
    let out = String::from_utf8_lossy(&status.stdout);
    assert!(root.join("cap/core/cap-core/Cargo.toml").is_file());
    // At least one rename detected (git -M defaults to rename detection in status).
    let renamed = git(&["diff", "--cached", "--name-status", "-M"]);
    let diff = String::from_utf8_lossy(&renamed.stdout);
    // After git mv + the in-place edits, git tracks the moves; we assert the new path is staged.
    assert!(
        out.contains("cap/core/cap-core") || diff.contains("cap/core/cap-core"),
        "git should track the move; status={out} diff={diff}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// Build a kernel-shaped fixture: one crate embedding a workspace-root sibling binary via a
/// hop-count-fixed literal, plus that binary as an artifact co-move (the kuberos `out/*.elf`
/// shape). The move preserves the crate's workspace-root depth (`crates/<c> -> kern/adapters/<c>`),
/// so `../../out/x.bin` stays the correct literal when `out/` rides the move.
fn build_elf_embed_fixture(root: &Path) {
    w(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/*\", \"kern/*\", \"kern/*/*\"]\nresolver = \"2\"\n",
    );
    w(
        root,
        "crates/oya-arch/Cargo.toml",
        "[package]\nname = \"oya-arch\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    w(
        root,
        "crates/oya-arch/src/lib.rs",
        "pub const IMAGE: &[u8] = include_bytes!(\"../../../out/x.bin\");\n",
    );
    w(root, "out/x.bin", "ELF-BYTES");
    w(root, "out/stays.keep", "untouched");
}

fn elf_embed_plan(_with_artifact: bool) -> MovePlan {
    // Depth-preserving move: crates/oya-arch/src -> kern/arch/src keeps ../../../ == repo
    // root (both sit 3 segments below the root), so the untouched target out/x.bin is still
    // hit by the same literal.
    MovePlan {
        capability: "kernel".to_owned(),
        moves: vec![CrateMove {
            old_path: "crates/oya-arch".to_owned(),
            new_path: "kern/arch".to_owned(),
            old_cargo_name: "oya-arch".to_owned(),
            new_cargo_name: "kernel-arch".to_owned(),
        }],
        artifacts: vec![],
    }
}

#[test]
fn app_product_move_keeps_the_oya_brand_while_capability_roots_stay_debranded() {
    // app/<product>/ is the brand-preserving composition ring: oya-hr-* names move in unchanged.
    let app_plan = MovePlan {
        capability: "app-hr".to_owned(),
        moves: vec![CrateMove {
            old_path: "oya/hr/crates/oya-hr-employment-api".to_owned(),
            new_path: "app/hr/crates/oya-hr-employment-api".to_owned(),
            old_cargo_name: "oya-hr-employment-api".to_owned(),
            new_cargo_name: "oya-hr-employment-api".to_owned(),
        }],
        artifacts: vec![],
    };
    oya_reorg_codemod_app::model::MovePlan::validate(&app_plan)
        .expect("app/ destinations keep the product brand");
    app_plan
        .validate_debrand_targets()
        .expect("app/ destinations keep the product brand in the forward gate too");
    // A capability-root destination keeping the brand still refuses.
    let bad_plan = MovePlan {
        capability: "iam".to_owned(),
        moves: vec![CrateMove {
            old_path: "oya/identity/crates/oya-identity-app".to_owned(),
            new_path: "iam/core/oya-identity-app".to_owned(),
            old_cargo_name: "oya-identity-app".to_owned(),
            new_cargo_name: "oya-identity-app".to_owned(),
        }],
        artifacts: vec![],
    };
    assert!(
        bad_plan.validate_debrand_targets().is_err(),
        "capability-root destinations must de-brand"
    );
}

#[test]
fn app_product_move_and_its_revert_round_trip_byte_identically() {
    // REGRESSION (PR #1965 wave-2, comment 3783872051): `apply --revert` swaps the tuple
    // (`MovePlan::inverse`) before apply_plan validates it, so the inverse of the sanctioned
    // app/<product>/ move (app/hr/... -> oya/hr/... keeping oya-hr-*) previously tripped the
    // de-brand refusal — rollback could never run. The inverse must validate and restore the
    // legacy branded home byte-identically.
    let root = tmp_root("app-revert");
    w(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"oya/hr/crates/*\"]\nresolver = \"2\"\n",
    );
    w(
        &root,
        "oya/hr/crates/oya-hr-employment-api/Cargo.toml",
        "[package]\nname = \"oya-hr-employment-api\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    w(
        &root,
        "oya/hr/crates/oya-hr-employment-api/src/lib.rs",
        "pub fn employment() {}\n",
    );
    // A bystander keeps the `oya/hr/crates/*` glob non-empty (no glob prune on either leg),
    // so the root manifest round-trips byte-identically through forward + revert.
    w(
        &root,
        "oya/hr/crates/oya-hr-bystander/Cargo.toml",
        "[package]\nname = \"oya-hr-bystander\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    w(
        &root,
        "oya/hr/crates/oya-hr-bystander/src/lib.rs",
        "pub fn b() {}\n",
    );
    let before = non_directory_entries(&snapshot_tree(&root));

    let forward = MovePlan {
        capability: "app-hr".to_owned(),
        moves: vec![CrateMove {
            old_path: "oya/hr/crates/oya-hr-employment-api".to_owned(),
            new_path: "app/hr/crates/oya-hr-employment-api".to_owned(),
            old_cargo_name: "oya-hr-employment-api".to_owned(),
            new_cargo_name: "oya-hr-employment-api".to_owned(),
        }],
        artifacts: vec![],
    };
    apply_plan(&root, &forward, &ApplyOptions { use_git_mv: false })
        .expect("the sanctioned app-product move applies");
    assert!(
        root.join("app/hr/crates/oya-hr-employment-api/Cargo.toml")
            .is_file()
    );

    // The REVERT direction: plan.inverse() swaps the tuple -> app/hr -> oya/hr with the same
    // branded name. validate() must accept it (recognized inverse) so rollback actually runs.
    apply_plan(
        &root,
        &forward.inverse(),
        &ApplyOptions { use_git_mv: false },
    )
    .expect("the inverse of a sanctioned app-product move must apply (revertability)");

    let after = non_directory_entries(&snapshot_tree(&root));
    assert_eq!(
        after, before,
        "revert must restore the legacy branded home byte-identically"
    );
    assert!(
        root.join("oya/hr/crates/oya-hr-employment-api/Cargo.toml")
            .is_file()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn escaping_literal_with_depth_preserved_and_untouched_target_is_accepted() {
    let root = tmp_root("elf-embed-accepted");
    build_elf_embed_fixture(&root);
    apply_plan(
        &root,
        &elf_embed_plan(true),
        &ApplyOptions { use_git_mv: false },
    )
    .expect("a depth-preserving move with an untouched target must be move-invariant");
    let moved = r(&root, "kern/arch/src/lib.rs");
    assert!(
        moved.contains("include_bytes!(\"../../../out/x.bin\")"),
        "the literal is hop-count-preserved: {moved}"
    );
    assert!(root.join("out/x.bin").is_file(), "target untouched");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn escaping_literal_with_shallower_depth_is_refused() {
    let root = tmp_root("elf-embed-refused");
    build_elf_embed_fixture(&root);
    // Same crate content but moved one level DEEPER (kern/adapters/arch): ../../../ from
    // kern/adapters/arch/src now resolves to kern/out/x.bin, not root/out/x.bin, so the
    // untouched target is no longer hit -> refused.
    let plan = MovePlan {
        capability: "kernel".to_owned(),
        moves: vec![CrateMove {
            old_path: "crates/oya-arch".to_owned(),
            new_path: "kern/adapters/arch".to_owned(),
            old_cargo_name: "oya-arch".to_owned(),
            new_cargo_name: "kernel-arch".to_owned(),
        }],
        artifacts: vec![],
    };
    let error = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false })
        .expect_err("a depth change that breaks the literal must stay fail-closed");
    let message = error.to_string();
    assert!(
        message.contains("resolve OUTSIDE that crate"),
        "the refusal must name the escaping-literal class: {message}"
    );
    assert!(
        message.contains("out/x.bin"),
        "the refusal must name the unresolved target: {message}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
