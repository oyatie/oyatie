//! The pre-move green-snapshot oracle.
//!
//! Two responsibilities:
//!
//! 1. **Green snapshot** ([`capture_snapshot`]): capture `cargo metadata` + `buck2 targets
//!    //...` clean output BEFORE a move, committed as the rollback oracle. Post-move green is
//!    necessary but not sufficient; the snapshot is the byte-baseline the rollback restores.
//!
//! 2. **Dry-run / shadow-apply** ([`dry_run`]): copy the workspace into a throwaway dir,
//!    apply the plan there (no `git mv` — a plain rename, since the shadow is not a git repo),
//!    then PROVE resolution: `cargo metadata` resolves AND `buck2 targets //...` resolves
//!    (the rust-analyzer-equivalent). PASS proves the move is clean WITHOUT landing it; FAIL
//!    is fail-closed (the move would break resolution).
//!
//! The dry-run is the safety gate. The engine refuses to land a move whose dry-run fails.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::model::{CodemodError, MovePlan};
use crate::plan::{apply_plan, ApplyOptions};

/// A captured green snapshot: the stdout of `cargo metadata` and `buck2 targets //...`. These
/// are the rollback oracle (the resolvable-baseline the tree returns to).
#[derive(Debug, Clone)]
pub struct GreenSnapshot {
    pub cargo_metadata: String,
    pub buck_targets: String,
    pub cargo_ok: bool,
    pub buck_ok: bool,
    pub buck_available: bool,
}

/// The outcome of a dry-run resolution check.
#[derive(Debug, Clone)]
pub struct DryRunReport {
    /// True iff the shadow tree resolved under every available checker.
    pub clean: bool,
    pub cargo_ok: bool,
    pub cargo_detail: String,
    /// `None` when buck2 is not on PATH (cargo-only verification; the caller is told).
    pub buck_ok: Option<bool>,
    pub buck_detail: String,
    /// The shadow dir where the move was applied (kept for inspection unless `keep` is false).
    pub shadow_dir: Option<PathBuf>,
}

/// Capture the green snapshot at `repo_root`. `run_buck` lets the caller skip buck2 when it is
/// unavailable (cargo-only baseline, reported honestly).
pub fn capture_snapshot(repo_root: &Path, run_buck: bool) -> GreenSnapshot {
    let (cargo_ok, cargo_metadata) = run_cargo_metadata(repo_root);
    let buck_available = run_buck && which("buck2");
    let (buck_ok, buck_targets) = if buck_available {
        run_buck_targets(repo_root)
    } else {
        (false, String::new())
    };
    GreenSnapshot {
        cargo_metadata,
        buck_targets,
        cargo_ok,
        buck_ok,
        buck_available,
    }
}

/// Shadow-apply `plan` into a throwaway copy of `repo_root` and prove resolution WITHOUT
/// landing it. `run_buck` requests the buck2 check (skipped if buck2 is absent). `keep_shadow`
/// keeps the shadow dir for inspection; otherwise it is removed before returning.
pub fn dry_run(
    repo_root: &Path,
    plan: &MovePlan,
    run_buck: bool,
    keep_shadow: bool,
) -> Result<DryRunReport, CodemodError> {
    plan.validate()?;
    let shadow = make_shadow(repo_root)?;

    // Apply WITHOUT git mv (the shadow is a plain copy, not a git repo).
    let apply_res = apply_plan(&shadow, plan, &ApplyOptions { use_git_mv: false });
    if let Err(error) = apply_res {
        // A fail-closed apply error IS a dry-run failure (the move is not clean).
        let report = DryRunReport {
            clean: false,
            cargo_ok: false,
            cargo_detail: format!("apply failed: {error}"),
            buck_ok: None,
            buck_detail: String::new(),
            shadow_dir: if keep_shadow { Some(shadow.clone()) } else { None },
        };
        if !keep_shadow {
            let _ = std::fs::remove_dir_all(&shadow);
        }
        return Ok(report);
    }

    let (cargo_ok, cargo_detail) = verify_owning_workspaces(&shadow, plan);
    let buck_available = run_buck && which("buck2");
    let (buck_ok, buck_detail) = if buck_available {
        let (ok, detail) = run_buck_targets(&shadow);
        (Some(ok), detail)
    } else {
        (None, "buck2 not on PATH; cargo-only dry-run".to_string())
    };

    let clean = cargo_ok && buck_ok.unwrap_or(true);
    let report = DryRunReport {
        clean,
        cargo_ok,
        cargo_detail,
        buck_ok,
        buck_detail,
        shadow_dir: if keep_shadow { Some(shadow.clone()) } else { None },
    };
    if !keep_shadow {
        let _ = std::fs::remove_dir_all(&shadow);
    }
    Ok(report)
}

/// Verify the workspaces that ACTUALLY OWN the moved paths, not just the repo root.
///
/// This is the D1 fix. The old oracle ran `cargo metadata` at the repo root only — but the root
/// `Cargo.toml` EXCLUDES the ADR-0512 nested carve-outs (`kernel`, `cloud/cloud-kernel`), so for
/// any move inside one of them the oracle validated a workspace that did not contain the crates
/// it had just moved, and reported `cargo_ok: true` over dangling path deps. A migration tool
/// whose oracle cannot see the tree it is migrating converts every nested-workspace move into a
/// silent corruption.
///
/// Two assertions per owning workspace:
/// 1. `cargo metadata` RESOLVES there (catches dangling `path=` deps — the 5-edge class), and
/// 2. every moved crate is actually PRESENT in that workspace's member set (catches a move that
///    lands outside every `members` glob, which resolution alone would not report).
///
/// The repo root is always included, so unrelated root breakage is still caught.
fn verify_owning_workspaces(shadow: &Path, plan: &MovePlan) -> (bool, String) {
    let mut workspaces: BTreeMap<String, Vec<String>> = BTreeMap::new();
    workspaces.entry(String::new()).or_default();
    for m in &plan.moves {
        match crate::plan::owning_workspace_root(shadow, &m.new_path) {
            Ok(Some(workspace)) => workspaces
                .entry(workspace)
                .or_default()
                .push(m.new_path.clone()),
            Ok(None) => {}
            Err(error) => return (false, format!("owning-workspace resolution failed: {error}")),
        }
    }

    let mut details = Vec::new();
    let mut ok = true;
    for (workspace, moved_paths) in &workspaces {
        let label = if workspace.is_empty() {
            "<repo root>"
        } else {
            workspace.as_str()
        };
        let workspace_abs = shadow.join(workspace);
        if !workspace_abs.join("Cargo.toml").is_file() {
            continue; // fixture tree without a manifest here
        }
        let (resolved, detail) = run_cargo_metadata(&workspace_abs);
        if !resolved {
            ok = false;
            details.push(format!("[{label}] cargo metadata FAILED: {detail}"));
            continue;
        }
        // Membership: the moved crate's manifest must appear in ITS OWN workspace's metadata.
        for moved in moved_paths {
            let manifest_abs = shadow.join(moved).join("Cargo.toml");
            let needle = manifest_abs.to_string_lossy().to_string();
            if !detail.contains(&needle) {
                ok = false;
                details.push(format!(
                    "[{label}] moved crate {moved:?} resolved by NO workspace member entry \
                     (cargo metadata at {label} does not list {needle})"
                ));
            }
        }
        details.push(format!("[{label}] ok"));
    }
    (ok, details.join("\n"))
}

/// Run `cargo metadata` at `root`. Returns `(resolved_ok, stdout_or_stderr)`. `--no-deps` +
/// `--format-version 1` keeps it a pure workspace resolution check (no registry fetch of
/// transitive deps), which is exactly the "does the workspace graph resolve" question.
fn run_cargo_metadata(root: &Path) -> (bool, String) {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version")
        .arg("1")
        .arg("--offline")
        .current_dir(root)
        .output();
    match output {
        Ok(out) if out.status.success() => (true, String::from_utf8_lossy(&out.stdout).to_string()),
        Ok(out) => (false, String::from_utf8_lossy(&out.stderr).to_string()),
        Err(e) => (false, format!("cargo metadata spawn failed: {e}")),
    }
}

/// Run `buck2 targets //...` at `root`. Returns `(resolved_ok, stdout_or_stderr)`.
fn run_buck_targets(root: &Path) -> (bool, String) {
    let output = Command::new("buck2")
        .arg("targets")
        .arg("//...")
        .current_dir(root)
        .output();
    match output {
        Ok(out) if out.status.success() => (true, String::from_utf8_lossy(&out.stdout).to_string()),
        Ok(out) => (false, String::from_utf8_lossy(&out.stderr).to_string()),
        Err(e) => (false, format!("buck2 targets spawn failed: {e}")),
    }
}

/// Copy `repo_root` into a fresh temp dir, skipping VCS/build-output/vendored trees so the
/// shadow is cheap and the resolution checks run on first-party graph shape only.
fn make_shadow(repo_root: &Path) -> Result<PathBuf, CodemodError> {
    const SKIP: [&str; 6] = [".git", "target", "node_modules", "buck-out", ".buckd", "vendor"];
    let unique = format!(
        "oya-reorg-shadow-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let dest = std::env::temp_dir().join(unique);
    copy_tree(repo_root, &dest, &SKIP)?;
    Ok(dest)
}

fn copy_tree(src: &Path, dest: &Path, skip: &[&str]) -> Result<(), CodemodError> {
    std::fs::create_dir_all(dest).map_err(|e| CodemodError::Io {
        context: format!("mkdir {}", dest.display()),
        message: e.to_string(),
    })?;
    let entries = std::fs::read_dir(src).map_err(|e| CodemodError::Io {
        context: format!("read_dir {}", src.display()),
        message: e.to_string(),
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let target = dest.join(name);
        let file_type = entry.file_type().map_err(|e| CodemodError::Io {
            context: format!("file_type {}", path.display()),
            message: e.to_string(),
        })?;
        if file_type.is_dir() {
            if skip.contains(&name) {
                continue;
            }
            copy_tree(&path, &target, skip)?;
        } else if file_type.is_symlink() {
            // Preserve symlinks as-is (buckconfig/prelude often symlinked).
            if let Ok(link) = std::fs::read_link(&path) {
                #[cfg(unix)]
                {
                    let _ = std::os::unix::fs::symlink(&link, &target);
                }
                #[cfg(not(unix))]
                {
                    let _ = std::fs::copy(&path, &target);
                }
            }
        } else {
            std::fs::copy(&path, &target).map_err(|e| CodemodError::Io {
                context: format!("copy {}", path.display()),
                message: e.to_string(),
            })?;
        }
    }
    Ok(())
}

/// True if `bin` is resolvable on PATH.
fn which(bin: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            if dir.join(bin).is_file() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CrateMove, MovePlan};
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_root(tag: &str) -> PathBuf {
        let unique = format!(
            "oya-oracle-test-{}-{}-{}",
            tag,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let root = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    /// A minimal 2-crate cargo workspace (no buck) that cargo metadata can resolve.
    fn make_cargo_fixture(root: &Path) {
        write(
            root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"2\"\n",
        );
        write(
            root,
            "crates/oya-a/Cargo.toml",
            "[package]\nname = \"oya-a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\noya-b = { path = \"../oya-b\" }\n",
        );
        write(root, "crates/oya-a/src/lib.rs", "pub use oya_b::hello;\n");
        write(
            root,
            "crates/oya-b/Cargo.toml",
            "[package]\nname = \"oya-b\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(root, "crates/oya-b/src/lib.rs", "pub fn hello() {}\n");
    }

    #[test]
    fn snapshot_captures_cargo_metadata_when_cargo_present() {
        let root = tmp_root("snap");
        make_cargo_fixture(&root);
        let snap = capture_snapshot(&root, false);
        // cargo is present in this environment; metadata must resolve the 2-crate workspace.
        assert!(snap.cargo_ok, "cargo metadata failed: {}", snap.cargo_metadata);
        assert!(snap.cargo_metadata.contains("oya-a"));
        assert!(snap.cargo_metadata.contains("oya-b"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn dry_run_passes_a_clean_move() {
        let root = tmp_root("clean");
        make_cargo_fixture(&root);
        // Move oya-b to a capability home; recompute the path-dep in oya-a.
        let plan = MovePlan {
            capability: "demo".to_string(),
            moves: vec![CrateMove {
                old_path: "crates/oya-b".to_string(),
                new_path: "demo/core/b".to_string(),
                old_cargo_name: "oya-b".to_string(),
                new_cargo_name: "demo-b".to_string(),
            }],
            artifacts: vec![],
        };
        // members glob is crates/* which will NOT cover demo/core/b -> the engine must add it.
        let report = dry_run(&root, &plan, false, false).unwrap();
        assert!(
            report.clean,
            "expected clean dry-run; cargo={}",
            report.cargo_detail
        );
        assert!(report.cargo_ok);
        // The real tree was NOT modified (dry-run is shadow-only).
        assert!(root.join("crates/oya-b/Cargo.toml").is_file());
        assert!(!root.join("demo/core/b").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A root workspace that EXCLUDES a nested workspace (the ADR-0512 carve-out shape:
    /// root `Cargo.toml` excludes `nested`, which is its own `[workspace]` root). `libs/keep`
    /// exists so the ROOT workspace itself is non-empty and resolves on its own.
    fn make_excluded_nested_workspace_fixture(root: &Path, break_nested: bool) {
        write(
            root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"libs/*\"]\nexclude = [\"nested\"]\nresolver = \"2\"\n",
        );
        write(
            root,
            "libs/keep/Cargo.toml",
            "[package]\nname = \"keep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(root, "libs/keep/src/lib.rs", "pub fn keep() {}\n");

        // The NESTED workspace — invisible to `cargo metadata` at the ROOT.
        write(
            root,
            "nested/Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"2\"\n",
        );
        // `a` depends on `b`. When `break_nested`, it ALSO carries a dangling path dep that
        // no plan can repair — the deliberate post-move break the oracle must catch.
        let a_manifest = if break_nested {
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nb = { path = \"../b\" }\nghost = { path = \"../oya-ghost\" }\n"
        } else {
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nb = { path = \"../b\" }\n"
        };
        write(root, "nested/crates/a/Cargo.toml", a_manifest);
        write(root, "nested/crates/a/src/lib.rs", "pub use b::hello;\n");
        write(
            root,
            "nested/crates/b/Cargo.toml",
            "[package]\nname = \"b\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(root, "nested/crates/b/src/lib.rs", "pub fn hello() {}\n");
    }

    fn nested_move_plan() -> MovePlan {
        MovePlan {
            capability: "nested".to_string(),
            moves: vec![CrateMove {
                old_path: "nested/crates/b".to_string(),
                new_path: "nested/core/b".to_string(),
                old_cargo_name: "b".to_string(),
                new_cargo_name: "nested-b".to_string(),
            }],
            artifacts: vec![],
        }
    }

    /// D1 (RED before the owning-workspace oracle): the moved crates live inside a workspace
    /// the ROOT `Cargo.toml` EXCLUDES, so `cargo metadata` at the ROOT can never observe them.
    /// A root-only oracle reports `clean: true` over an arbitrarily broken nested workspace.
    #[test]
    fn dry_run_fails_when_an_excluded_nested_workspace_is_left_broken() {
        let root = tmp_root("nested-broken");
        make_excluded_nested_workspace_fixture(&root, true);
        let report = dry_run(&root, &nested_move_plan(), false, false).unwrap();
        assert!(
            !report.clean,
            "oracle must FAIL on a broken EXCLUDED nested workspace; \
             a root-only `cargo metadata` is structurally blind to it. cargo={}",
            report.cargo_detail
        );
        assert!(!report.cargo_ok, "cargo leg must be the failing one");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The GREEN counterpart: the SAME nested/excluded geometry with a correct result must
    /// still pass. Together with the RED test above this proves the oracle actually descended
    /// into the nested workspace rather than skipping it (a skip would pass BOTH).
    #[test]
    fn dry_run_passes_a_correct_move_inside_an_excluded_nested_workspace() {
        let root = tmp_root("nested-clean");
        make_excluded_nested_workspace_fixture(&root, false);
        let report = dry_run(&root, &nested_move_plan(), false, false).unwrap();
        assert!(
            report.clean,
            "a correct nested move must pass; cargo={}",
            report.cargo_detail
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn dry_run_fails_a_move_that_breaks_resolution() {
        let root = tmp_root("broken");
        make_cargo_fixture(&root);
        // A DELIBERATELY broken plan: rename oya-b's package but craft a fixture where the
        // dependent references the OLD name through a hand-broken manifest. We simulate the
        // break by moving oya-b out from under the members glob WITHOUT the engine being able
        // to fix the dependent — achieved by making oya-a depend on a name the move renames,
        // while a stale second manifest hardcodes a now-wrong absolute path.
        // Simplest deterministic break: point oya-a at a path that will not exist post-move
        // and is OUTSIDE the plan (so the engine won't recompute it).
        write(
            &root,
            "crates/oya-a/Cargo.toml",
            "[package]\nname = \"oya-a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nghost = { path = \"../oya-ghost\" }\n",
        );
        let plan = MovePlan {
            capability: "demo".to_string(),
            moves: vec![CrateMove {
                old_path: "crates/oya-b".to_string(),
                new_path: "demo/core/b".to_string(),
                old_cargo_name: "oya-b".to_string(),
                new_cargo_name: "demo-b".to_string(),
            }],
            artifacts: vec![],
        };
        let report = dry_run(&root, &plan, false, false).unwrap();
        assert!(
            !report.clean,
            "expected dry-run to FAIL on unresolvable workspace"
        );
        assert!(!report.cargo_ok);
        let _ = std::fs::remove_dir_all(&root);
    }
}
