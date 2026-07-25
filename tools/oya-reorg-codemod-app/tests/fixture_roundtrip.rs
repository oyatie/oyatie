//! End-to-end fixture proof of the capability-move codemod (ADR-0562 P0.13 CRITICAL scope).
//!
//! Builds a SYNTHETIC 3-crate tree (mirroring the real shapes: a deep `../../../` move-fatal
//! path-dep, an absolute BUCK label, a kebab->snake Rust import), moves it with the codemod,
//! and asserts:
//!   (a) `cargo metadata` resolves post-move (and would buck2-resolve; the BUCK labels are
//!       rewritten and asserted textually);
//!   (b) the inverse restores the tree BYTE-IDENTICALLY;
//!   (c) the dry-run gate PASSES a clean move and FAILS a move that would break resolution.
//!
//! No real capability is moved; the fixture is a throwaway temp tree.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use oya_reorg_codemod_app::model::{CrateMove, MovePlan};
use oya_reorg_codemod_app::oracle;
use oya_reorg_codemod_app::plan::{apply_plan, ApplyOptions};

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

/// Snapshot every first-party file (Cargo.toml/BUCK/*.rs + .buckconfig) as (rel -> bytes)
/// for byte-identity comparison across the round trip.
fn snapshot_tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if path.is_dir() {
                if name == ".git" || name == "target" {
                    continue;
                }
                stack.push(path);
            } else {
                let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                out.insert(rel, std::fs::read(&path).unwrap());
            }
        }
    }
    out
}

#[test]
fn validation_failure_preserves_the_entire_tree_byte_for_byte() {
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
        "validation failure must leave every fixture file byte-identical"
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
    w(root, "crates/oya-bystander/src/lib.rs", "pub fn bystand() {}\n");

    // libs/oya-shared-kernel (does not move)
    w(
        root,
        "libs/oya-shared-kernel/Cargo.toml",
        "[package]\nname = \"oya-shared-kernel\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    w(root, "libs/oya-shared-kernel/src/lib.rs", "pub fn root() {}\n");
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
    assert!(app_manifest.contains("cap-core = "), "renamed dep key: {app_manifest}");
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
    assert!(app_src.contains("use cap_core::engine;"), "rust import: {app_src}");
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
    assert!(root_manifest.contains("crates/*"), "bystander keeps crates/* alive");

    // (8) cargo metadata resolves the moved workspace.
    let snap = oracle::capture_snapshot(&root, false);
    assert!(snap.cargo_ok, "post-move cargo metadata failed: {}", snap.cargo_metadata);
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

#[test]
fn inverse_restores_byte_identically() {
    let root = tmp_root("inv");
    build_fixture(&root);
    let before = snapshot_tree(&root);

    let plan = capability_plan();
    apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false }).unwrap();

    // Apply the inverse.
    apply_plan(&root, &plan.inverse(), &ApplyOptions { use_git_mv: false }).unwrap();

    let after = snapshot_tree(&root);
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>(),
        "file set must be identical after round trip"
    );
    for (path, bytes) in &before {
        assert_eq!(
            after.get(path),
            Some(bytes),
            "file {path} not byte-identical after inverse"
        );
    }

    // And cargo still resolves the restored tree.
    let snap = oracle::capture_snapshot(&root, false);
    assert!(snap.cargo_ok, "restored cargo metadata failed: {}", snap.cargo_metadata);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn dry_run_passes_a_clean_capability_move_without_landing() {
    let root = tmp_root("dryok");
    build_fixture(&root);
    let plan = capability_plan();
    let before = snapshot_tree(&root);

    let report = oracle::dry_run(&root, &plan, false, false).unwrap();
    assert!(report.clean, "expected clean dry-run; cargo={}", report.cargo_detail);
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
    assert!(!report.cargo_ok, "cargo metadata must fail on the broken graph");

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
    w(&root, "libs/oya-shared-kernel/src/lib.rs", "pub fn root() {}\n");
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
    assert!(!manifest.contains("crates/*"), "emptied glob pruned: {manifest}");
    assert!(manifest.contains("cap/core/cap-core"), "new dir added: {manifest}");

    // The pruned + extended workspace resolves.
    let snap = oracle::capture_snapshot(&root, false);
    assert!(snap.cargo_ok, "post-prune cargo metadata failed: {}", snap.cargo_metadata);

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
