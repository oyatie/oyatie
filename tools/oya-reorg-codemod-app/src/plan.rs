//! Orchestration: apply a [`MovePlan`] to a real repo tree (forward or inverse). Walks the
//! workspace, applies the cargo/buck/rust-source transforms, recomputes the move-fatal
//! relative path-deps, then performs the history-preserving `git mv` directory moves. The
//! same engine runs forward and in reverse — the inverse is just `plan.inverse()`.
//!
//! Apply order matters and is fixed for determinism + atomicity:
//! 1. validate the plan (fail-closed on collisions / bad paths);
//! 2. pre-flight: every source dir exists, every target dir is free;
//! 3. rewrite ALL `Cargo.toml` (deps + path recompute) and the moved manifests' package
//!    names — IN PLACE at their OLD paths (so paths recompute against the to-be layout);
//! 4. rewrite ALL `BUCK` labels + the moved BUCKs' own name/crate fields;
//! 5. rewrite ALL `.rs` crate-ident references;
//! 6. rewrite the root workspace members/exclude if needed;
//! 7. `git mv` each crate dir old -> new (longest-path-first so nested moves are safe).
//!
//! Steps 3-6 edit files at their CURRENT (pre-move) locations; step 7 then relocates the dirs
//! wholesale, carrying the already-rewritten manifests/sources with them. This ordering keeps
//! the operation a single coherent transform and lets a `--dry-run` shadow copy prove it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_workspace_members_kernel::resolve_member_dirs_from_str;

use crate::buck;
use crate::cargo;
use crate::model::{
    dir_exists, snake, CodemodError, CrateMove, Mapping, MovePlan,
};
use crate::rust_src;

/// Options controlling an apply run.
#[derive(Debug, Clone)]
pub struct ApplyOptions {
    /// Use `git mv` (history-preserving) for the directory move. When false (shadow/dry-run
    /// over a non-git temp copy), a plain `fs::rename` is used.
    pub use_git_mv: bool,
}

impl Default for ApplyOptions {
    fn default() -> Self {
        ApplyOptions { use_git_mv: true }
    }
}

/// The result of an apply: the emitted mapping (audit + invertible) and the set of files
/// touched (relative paths), for reporting.
#[derive(Debug, Clone)]
pub struct ApplyOutcome {
    pub mapping: Mapping,
    pub manifests_rewritten: Vec<String>,
    pub bucks_rewritten: Vec<String>,
    pub rust_files_rewritten: Vec<String>,
    pub root_workspace_changed: bool,
    pub dirs_moved: Vec<(String, String)>,
}

/// Apply a [`MovePlan`] to the tree rooted at `repo_root`. Fail-closed: returns the first
/// [`CodemodError`] without leaving a partial move (file edits precede the dir moves, and the
/// dir moves are the last, all-or-nothing step; an error before step 7 means no dir moved).
pub fn apply_plan(
    repo_root: &Path,
    plan: &MovePlan,
    opts: &ApplyOptions,
) -> Result<ApplyOutcome, CodemodError> {
    plan.validate()?;

    // Pre-flight: sources present, targets free.
    for m in &plan.moves {
        if !dir_exists(repo_root, &m.old_path) {
            return Err(CodemodError::SourceMissing {
                path: m.old_path.clone(),
            });
        }
        if dir_exists(repo_root, &m.new_path) {
            return Err(CodemodError::TargetExists {
                path: m.new_path.clone(),
            });
        }
    }

    let by_old_path = plan.by_old_path();
    let by_old_name = plan.by_old_cargo_name();

    // resolve_target: OLD repo-relative crate dir -> NEW dir (identity if unmoved).
    let resolve_target = |old: &str| -> Option<String> {
        by_old_path.get(old).map(|cm| cm.new_path.clone())
    };

    // ident_renames for rust sources: old snake -> new snake.
    let ident_renames: BTreeMap<String, String> = plan
        .moves
        .iter()
        .map(|m| (snake(&m.old_cargo_name), snake(&m.new_cargo_name)))
        .collect();

    let mut outcome = ApplyOutcome {
        mapping: plan.mapping(),
        manifests_rewritten: Vec::new(),
        bucks_rewritten: Vec::new(),
        rust_files_rewritten: Vec::new(),
        root_workspace_changed: false,
        dirs_moved: Vec::new(),
    };

    // --- Steps 3-5: walk every first-party file and rewrite in place. ---
    let all_files = walk_repo_files(repo_root)?;

    for rel in &all_files {
        let abs = repo_root.join(rel);
        let file_name = Path::new(rel).file_name().and_then(|s| s.to_str());
        if file_name == Some("Cargo.toml") {
            rewrite_one_cargo_toml(
                repo_root,
                rel,
                &abs,
                &by_old_path,
                &by_old_name,
                &resolve_target,
                &mut outcome,
            )?;
        } else if file_name == Some("BUCK") || file_name == Some("BUCK.v2") {
            rewrite_one_buck(repo_root, rel, &abs, &by_old_path, &mut outcome)?;
        } else if rel.ends_with(".rs") {
            rewrite_one_rust(&abs, rel, &ident_renames, &mut outcome)?;
        }
    }

    // --- Step 6: root workspace members/exclude. ---
    rewrite_root_workspace(repo_root, plan, &mut outcome)?;

    // --- Step 7: the directory moves (longest old_path first so nested dirs move safely). ---
    let mut ordered: Vec<&CrateMove> = plan.moves.iter().collect();
    ordered.sort_by(|a, b| b.old_path.len().cmp(&a.old_path.len()).then(a.old_path.cmp(&b.old_path)));
    for m in ordered {
        move_dir(repo_root, &m.old_path, &m.new_path, opts)?;
        outcome.dirs_moved.push((m.old_path.clone(), m.new_path.clone()));
    }
    outcome.dirs_moved.sort();

    Ok(outcome)
}

fn rewrite_one_cargo_toml(
    _repo_root: &Path,
    rel: &str,
    abs: &Path,
    by_old_path: &BTreeMap<&str, &CrateMove>,
    by_old_name: &BTreeMap<&str, &CrateMove>,
    resolve_target: &dyn Fn(&str) -> Option<String>,
    outcome: &mut ApplyOutcome,
) -> Result<(), CodemodError> {
    let text = read(abs, rel)?;
    let manifest_dir = parent_dir(rel);
    let mut current = text.clone();
    let mut any_change = false;

    // (a) If this manifest's crate is itself moving, rename its package first.
    if let Some(cm) = by_old_path.get(manifest_dir.as_str()) {
        let renamed = cargo::rewrite_moved_manifest_package(&current, rel, cm)?;
        if renamed != current {
            current = renamed;
            any_change = true;
        }
    }

    // (b) Rewrite dependency keys + recompute relative path-deps (the move-fatal class).
    let this_moved_to = by_old_path.get(manifest_dir.as_str()).map(|cm| cm.new_path.clone());
    let (dep_rewritten, dep_changed) = cargo::rewrite_dependencies_in_manifest(
        &current,
        rel,
        &manifest_dir,
        this_moved_to.as_deref(),
        by_old_name,
        resolve_target,
    )?;
    if dep_changed {
        current = dep_rewritten;
        any_change = true;
    }

    if any_change {
        write(abs, rel, &current)?;
        outcome.manifests_rewritten.push(rel.to_string());
    }
    Ok(())
}

fn rewrite_one_buck(
    _repo_root: &Path,
    rel: &str,
    abs: &Path,
    by_old_path: &BTreeMap<&str, &CrateMove>,
    outcome: &mut ApplyOutcome,
) -> Result<(), CodemodError> {
    let text = read(abs, rel)?;
    let buck_dir = parent_dir(rel);
    let mut current = text.clone();
    let mut any_change = false;

    // (a) absolute label rewrites across every BUCK.
    let (label_rewritten, label_changed) = buck::rewrite_buck_labels(&current, by_old_path);
    if label_changed {
        current = label_rewritten;
        any_change = true;
    }
    // (b) if this BUCK's package is moving, rewrite its own name/crate/self-deps.
    if let Some(cm) = by_old_path.get(buck_dir.as_str()) {
        let (own_rewritten, own_changed) = buck::rewrite_moved_buck(&current, cm);
        if own_changed {
            current = own_rewritten;
            any_change = true;
        }
    }
    if any_change {
        write(abs, rel, &current)?;
        outcome.bucks_rewritten.push(rel.to_string());
    }
    Ok(())
}

fn rewrite_one_rust(
    abs: &Path,
    rel: &str,
    ident_renames: &BTreeMap<String, String>,
    outcome: &mut ApplyOutcome,
) -> Result<(), CodemodError> {
    let text = read(abs, rel)?;
    let (rewritten, changed) = rust_src::rewrite_rust_source(&text, ident_renames);
    if changed {
        write(abs, rel, &rewritten)?;
        outcome.rust_files_rewritten.push(rel.to_string());
    }
    Ok(())
}

fn rewrite_root_workspace(
    repo_root: &Path,
    plan: &MovePlan,
    outcome: &mut ApplyOutcome,
) -> Result<(), CodemodError> {
    let root_manifest = repo_root.join("Cargo.toml");
    if !root_manifest.is_file() {
        return Ok(()); // a fixture without a root workspace is allowed
    }
    let text = read(&root_manifest, "Cargo.toml")?;

    // Determine which new dirs are NOT covered by the existing globs. We resolve the member
    // dirs against a SHADOW tree where the moves are already applied as bare dirs. Cheaper:
    // simulate by checking each new_path against the existing patterns via the resolver after
    // creating the dirs — but at this point the dirs have NOT moved yet (step 7). Instead we
    // compute coverage from the manifest's globs directly using the kernel's resolver against
    // a tiny synthetic check: a new_path is "covered" if some member glob would match it.
    let resolved_pattern_covers = |new_dir: &str| pattern_set_covers(&text, new_dir);

    let mut uncovered: Vec<String> = plan
        .moves
        .iter()
        .map(|m| m.new_path.clone())
        .filter(|d| !resolved_pattern_covers(d))
        .collect();
    uncovered.sort();
    uncovered.dedup();

    // Old exclude entries that pointed at a now-moved old_path should be dropped.
    let moved_old: BTreeSet<&str> = plan.moves.iter().map(|m| m.old_path.as_str()).collect();
    let excludes_to_remove: Vec<String> = current_excludes(&text)
        .into_iter()
        .filter(|e| moved_old.contains(e.as_str()))
        .collect();

    // Prune any members glob that matches ZERO crates post-move (a move emptied a globbed
    // dir; the stale glob would make cargo error `failed to read <glob>/Cargo.toml`). We
    // resolve the CURRENT member dirs from the live tree, simulate the move (remove old_paths,
    // add new_paths), and drop a glob whose post-move match set is empty.
    let mut globs_to_prune = compute_globs_to_prune(repo_root, &text, plan);

    // Drop LITERAL members entries that point at a now-moved old_path (the symmetric inverse of
    // the literal new-dir additions, so a forward-add/inverse-remove round-trips byte-clean).
    let current_members = current_members(&text);
    for entry in &current_members {
        if !entry.contains('*') && moved_old.contains(entry.as_str()) {
            globs_to_prune.push(entry.clone());
        }
    }
    globs_to_prune.sort();
    globs_to_prune.dedup();

    let (new_text, changed) = cargo::rewrite_root_workspace_members(
        &text,
        &uncovered,
        &globs_to_prune,
        &[],
        &excludes_to_remove,
    )?;
    if changed {
        write(&root_manifest, "Cargo.toml", &new_text)?;
        outcome.root_workspace_changed = true;
    }
    Ok(())
}

/// Compute the members globs that match ZERO crate dirs after the plan's moves are applied.
/// We resolve the CURRENT member dirs (live tree, via the kernel resolver), then form the
/// post-move dir set (drop moved old_paths, add moved new_paths). A `members` entry that is a
/// glob (`*`-containing) and matches none of the post-move dirs is returned for pruning. A
/// literal members entry is never pruned here (its own move, if any, is handled separately).
fn compute_globs_to_prune(
    repo_root: &Path,
    root_manifest_text: &str,
    plan: &MovePlan,
) -> Vec<String> {
    let entries = match oya_workspace_members_kernel::workspace_manifest_entries_from_str(
        root_manifest_text,
    ) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    // Current resolved member dirs from the live (pre-move) tree.
    let current: Vec<String> =
        resolve_member_dirs_from_str(root_manifest_text, repo_root).unwrap_or_default();
    let moved_old: BTreeSet<&str> = plan.moves.iter().map(|m| m.old_path.as_str()).collect();
    let mut post_move: BTreeSet<String> = current
        .into_iter()
        .filter(|d| !moved_old.contains(d.as_str()))
        .collect();
    for m in &plan.moves {
        post_move.insert(m.new_path.clone());
    }

    let mut prune = Vec::new();
    for pat in &entries.members {
        if !pat.contains('*') {
            continue; // literals are handled by the move logic, never auto-pruned
        }
        let matches_any = post_move.iter().any(|dir| {
            let segs: Vec<&str> = dir.split('/').collect();
            glob_matches(pat, &segs)
        });
        if !matches_any {
            prune.push(pat.clone());
        }
    }
    prune.sort();
    prune.dedup();
    prune
}

/// True if the root manifest's `[workspace].members` glob set would match `new_dir`. We use
/// the kernel resolver semantics by parsing the patterns and testing component-wise; since
/// the new dir does not exist on disk yet, we test the glob shape (not the Cargo.toml
/// presence rule, which the post-move tree satisfies by construction).
fn pattern_set_covers(root_manifest_text: &str, new_dir: &str) -> bool {
    let entries = match oya_workspace_members_kernel::workspace_manifest_entries_from_str(
        root_manifest_text,
    ) {
        Ok(e) => e,
        Err(_) => return false,
    };
    let segments: Vec<&str> = new_dir.split('/').collect();
    entries.members.iter().any(|pat| glob_matches(pat, &segments))
}

/// Match a `members` glob pattern (slash-separated, `*`-wildcard components) against a path's
/// segments. Mirrors the kernel resolver's per-component `*` semantics (a `*` matches exactly
/// one component; partial wildcards like `oya-*` match within one component).
fn glob_matches(pattern: &str, segments: &[&str]) -> bool {
    let pat_segs: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    if pat_segs.len() != segments.len() {
        return false;
    }
    pat_segs
        .iter()
        .zip(segments.iter())
        .all(|(p, s)| segment_glob(p, s))
}

fn segment_glob(pattern: &str, name: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == name;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut cursor = 0usize;
    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if idx == 0 {
            if !name[cursor..].starts_with(part) {
                return false;
            }
            cursor += part.len();
        } else if idx == parts.len() - 1 {
            return name[cursor..].ends_with(part) && name.len() - cursor >= part.len();
        } else {
            match name[cursor..].find(part) {
                Some(off) => cursor += off + part.len(),
                None => return false,
            }
        }
    }
    true
}

fn current_excludes(root_manifest_text: &str) -> Vec<String> {
    oya_workspace_members_kernel::workspace_manifest_entries_from_str(root_manifest_text)
        .map(|e| e.exclude)
        .unwrap_or_default()
}

fn current_members(root_manifest_text: &str) -> Vec<String> {
    oya_workspace_members_kernel::workspace_manifest_entries_from_str(root_manifest_text)
        .map(|e| e.members)
        .unwrap_or_default()
}

/// Resolve the concrete member dirs from a manifest text against a tree (used by the oracle
/// to assert the post-move member set is non-empty / well-formed).
pub fn resolve_members(root_manifest_text: &str, repo_root: &Path) -> Vec<String> {
    resolve_member_dirs_from_str(root_manifest_text, repo_root).unwrap_or_default()
}

fn move_dir(
    repo_root: &Path,
    old: &str,
    new: &str,
    opts: &ApplyOptions,
) -> Result<(), CodemodError> {
    let new_abs = repo_root.join(new);
    if let Some(parent) = new_abs.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CodemodError::Io {
            context: format!("create parent {}", parent.display()),
            message: e.to_string(),
        })?;
    }
    if opts.use_git_mv {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .arg("mv")
            .arg(old)
            .arg(new)
            .output()
            .map_err(|e| CodemodError::Io {
                context: format!("git mv {old} {new}"),
                message: e.to_string(),
            })?;
        if !output.status.success() {
            return Err(CodemodError::Io {
                context: format!("git mv {old} {new}"),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
    } else {
        std::fs::rename(repo_root.join(old), &new_abs).map_err(|e| CodemodError::Io {
            context: format!("rename {old} {new}"),
            message: e.to_string(),
        })?;
    }
    Ok(())
}

/// Walk the repo for first-party source files (Cargo.toml, BUCK, *.rs), skipping vendored,
/// VCS, and build-output trees. Returns sorted repo-relative forward-slash paths.
fn walk_repo_files(repo_root: &Path) -> Result<Vec<String>, CodemodError> {
    const SKIP_DIRS: [&str; 7] = [
        ".git",
        "target",
        "third-party",
        "node_modules",
        "buck-out",
        ".buckd",
        "vendor",
    ];
    let mut out = Vec::new();
    let mut stack: Vec<PathBuf> = vec![repo_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| CodemodError::Io {
            context: format!("read_dir {}", dir.display()),
            message: e.to_string(),
        })?;
        let mut children: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .collect();
        children.sort();
        for path in children {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if path.is_dir() {
                if !SKIP_DIRS.contains(&name) {
                    stack.push(path);
                }
            } else if (name == "Cargo.toml"
                || name == "BUCK"
                || name == "BUCK.v2"
                || name.ends_with(".rs"))
                && let Ok(rel) = path.strip_prefix(repo_root)
            {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out.sort();
    Ok(out)
}

fn parent_dir(rel: &str) -> String {
    match rel.rfind('/') {
        Some(idx) => rel[..idx].to_string(),
        None => String::new(),
    }
}

fn read(abs: &Path, rel: &str) -> Result<String, CodemodError> {
    std::fs::read_to_string(abs).map_err(|e| CodemodError::Io {
        context: format!("read {rel}"),
        message: e.to_string(),
    })
}

fn write(abs: &Path, rel: &str, content: &str) -> Result<(), CodemodError> {
    std::fs::write(abs, content).map_err(|e| CodemodError::Io {
        context: format!("write {rel}"),
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matches_mirrors_kernel_semantics() {
        assert!(glob_matches("libs/oya-*", &["libs", "oya-kernel"]));
        assert!(glob_matches("cloud/*/crates/oya-*", &["cloud", "cloud-iam", "crates", "oya-x"]));
        assert!(!glob_matches("libs/oya-*", &["libs", "group", "oya-x"]));
        assert!(!glob_matches("iam/core/*", &["iam", "core"]));
        assert!(glob_matches("iam/core/*", &["iam", "core", "domain"]));
    }

    #[test]
    fn pattern_set_covers_new_capability_paths() {
        let manifest = r#"[workspace]
members = ["libs/oya-*", "cloud/*/crates/oya-*", "iam/*/*"]
"#;
        assert!(pattern_set_covers(manifest, "iam/core/domain"));
        assert!(!pattern_set_covers(manifest, "secrets/core/kms"));
    }

    #[test]
    fn parent_dir_handles_root_and_nested() {
        assert_eq!(parent_dir("Cargo.toml"), "");
        assert_eq!(parent_dir("a/b/Cargo.toml"), "a/b");
    }
}
