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
//!    5b. rewrite ADR doc-anchor path citations (`docs/decisions/*.md`) old -> new for every moved
//!    crate — a load-bearing `justification_ref` anchor for the total-accounting gate
//!    (`resolve_justifications` token-walks the ADR corpus); left un-rewritten, a moved file an
//!    ADR cites by exact path silently loses its justification (distinct from the ADR-0563
//!    baseline-relabel class, which only covers PRE-EXISTING baselined debt, not a live citation);
//! 6. rewrite the root workspace members/exclude if needed;
//!    then relocate the moved crates' `Cargo.lock` entries (rename + re-canonicalize) via the
//!    owned pure transform — byte-identical, no cargo, no-op without a root lockfile;
//! 7. `git mv` each crate dir old -> new (longest-path-first so nested moves are safe);
//! 8. `git mv` each NON-crate artifact (SLOs, catalog records) old -> new, content-preserving
//!    (no in-file rewrite — these carry no cargo/buck/rust idents).
//!
//! Steps 3-6 edit files at their CURRENT (pre-move) locations; step 7 then relocates the dirs
//! wholesale, carrying the already-rewritten manifests/sources with them; step 8 co-moves the
//! capability's non-crate artifacts wholesale. This ordering keeps the operation a single
//! coherent transform and lets a `--dry-run` shadow copy prove it.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_workspace_members_kernel::{ResolveError, resolve_member_dirs_from_str};

use crate::buck;
use crate::cargo;
use crate::model::{
    dir_exists, rewrite_path_token, snake, CodemodError, CrateMove, Mapping, MovePlan,
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
    /// `docs/decisions/*.md` files whose ADR body cited a moved crate's OLD exact path and were
    /// rewritten to the NEW path (Step 5b). Empty when no ADR anchors a moved crate's path.
    pub docs_rewritten: Vec<String>,
    pub root_workspace_changed: bool,
    /// True iff the root `Cargo.lock` package entries were relocated (renamed + re-canonicalized)
    /// by this move. False when the tree has no lockfile or no crate's cargo name changed.
    pub cargo_lock_changed: bool,
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

    // Build lookup maps up-front — needed by both the pre-flight artifact check and the
    // rewrite/move steps below.
    let by_old_path = plan.by_old_path();
    let by_old_name = plan.by_old_cargo_name();

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
    // Pre-flight for NON-crate artifacts (a file OR a dir): source present, target free. Uses a
    // path-exists check (not dir-only) because an artifact may be a single SLO/catalog file.
    //
    // Pre-flight runs BEFORE any moves, so we check old_path directly — even for artifacts
    // nested inside a moving crate dir the file exists at old_path at this point.  The
    // post-crate-move location is only needed in step 8 (after step 7 has relocated the crate).
    for a in &plan.artifacts {
        if !path_exists(repo_root, &a.old_path) {
            return Err(CodemodError::SourceMissing {
                path: a.old_path.clone(),
            });
        }
        if path_exists(repo_root, &a.new_path) {
            return Err(CodemodError::TargetExists {
                path: a.new_path.clone(),
            });
        }
    }

    // Resolve the current root workspace before any transform writes. A missing-manifest or
    // filesystem inspection failure must not become an empty member set after steps 3-5 have
    // already edited files: that could prune unrelated globs and leave a partial move.
    validate_root_workspace_members(repo_root)?;

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
        docs_rewritten: Vec::new(),
        root_workspace_changed: false,
        cargo_lock_changed: false,
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

    // --- Step 5b: rewrite ADR doc-anchor path citations. ---
    rewrite_doc_anchors(repo_root, plan, &mut outcome)?;

    // --- Step 6: root workspace members/exclude. ---
    rewrite_root_workspace(repo_root, plan, &mut outcome)?;

    // --- Step 6b: relocate the moved crates' Cargo.lock package entries (rename + re-canonicalize)
    // via the owned pure transform — byte-identically, WITHOUT invoking cargo. Keyed on the SAME
    // name map the crate moves carry, so it needs no parallel config; a no-op when the tree has no
    // root Cargo.lock (fixtures / sub-workspaces). ---
    rewrite_cargo_lock(repo_root, plan, &mut outcome)?;

    // --- Step 7: the directory moves (longest old_path first so nested dirs move safely). ---
    let mut ordered: Vec<&CrateMove> = plan.moves.iter().collect();
    ordered.sort_by(|a, b| b.old_path.len().cmp(&a.old_path.len()).then(a.old_path.cmp(&b.old_path)));
    for m in ordered {
        move_dir(repo_root, &m.old_path, &m.new_path, opts)?;
        outcome.dirs_moved.push((m.old_path.clone(), m.new_path.clone()));
    }

    // --- Step 8: NON-crate artifact moves (SLOs, catalog records). Content-preserving wholesale
    // `git mv` of each file/dir; `move_dir` mkdir -p's the parent + handles file AND dir sources.
    // Longest old_path first so a nested artifact under another artifact moves safely. Appended to
    // dirs_moved so total-accounting / the manifest see them. EMPTY artifacts => zero iterations
    // => byte-identical to a pre-ArtifactMove apply (back-compat no-op).
    //
    // NOTE: an artifact whose old_path is nested under a crate that step 7 already moved must be
    // sourced from its post-crate-move location (`artifact_effective_source`). The dirs_moved
    // record always uses a.old_path (the canonical plan-declared source) so the manifest relabel
    // engine's old→new mapping stays stable regardless of where the artifact physically lived. ---
    let mut ordered_artifacts: Vec<&crate::model::ArtifactMove> = plan.artifacts.iter().collect();
    ordered_artifacts.sort_by(|a, b| {
        b.old_path
            .len()
            .cmp(&a.old_path.len())
            .then(a.old_path.cmp(&b.old_path))
    });
    for a in ordered_artifacts {
        let effective_src = artifact_effective_source(a, &by_old_path);
        move_dir(repo_root, &effective_src, &a.new_path, opts)?;
        outcome.dirs_moved.push((a.old_path.clone(), a.new_path.clone()));
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
    let excludes_to_remove: Vec<String> = current_excludes(&text)?
        .into_iter()
        .filter(|e| moved_old.contains(e.as_str()))
        .collect();

    // Prune any members glob that matches ZERO crates post-move (a move emptied a globbed
    // dir; the stale glob would make cargo error `failed to read <glob>/Cargo.toml`). We
    // resolve the CURRENT member dirs from the live tree, simulate the move (remove old_paths,
    // add new_paths), and drop a glob whose post-move match set is empty.
    let mut globs_to_prune = compute_globs_to_prune(repo_root, &text, plan)?;

    // Drop LITERAL members entries that point at a now-moved old_path (the symmetric inverse of
    // the literal new-dir additions, so a forward-add/inverse-remove round-trips byte-clean).
    let current_members = current_members(&text)?;
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

/// Relocate the moved crates' `Cargo.lock` package entries using the proven owned lock transform
/// ([`oya_cargo_lock_transform_kernel::move_lockfile`]) — rename the `[[package]]` block + every
/// dependency reference and re-canonicalize into Cargo's package order, byte-identically and
/// WITHOUT invoking cargo (no version resolution: a crate relocation preserves the version graph).
///
/// The rename map is derived from the plan's own name map (`old_cargo_name -> new_cargo_name`), so
/// this is data-driven off the existing move-plan with no parallel config; `plan.validate()` has
/// already guaranteed both sides are injective, so the map cannot collapse two crates. A pure
/// relocation adds no new members or edges, hence [`GraphAdditions::empty`].
///
/// No-op (returns `Ok`) when the tree has no root `Cargo.lock` (fixtures, sub-workspaces) or when
/// no crate's cargo name actually changes — mirroring [`rewrite_root_workspace`]'s tolerance for a
/// missing root manifest. Runs forward AND inverse: the inverse plan swaps the name map, so
/// `--revert` restores the lockfile byte-identically.
fn rewrite_cargo_lock(
    repo_root: &Path,
    plan: &MovePlan,
    outcome: &mut ApplyOutcome,
) -> Result<(), CodemodError> {
    let lock_path = repo_root.join("Cargo.lock");
    if !lock_path.is_file() {
        return Ok(());
    }
    let rename_map: HashMap<String, String> = plan
        .moves
        .iter()
        .filter(|m| m.old_cargo_name != m.new_cargo_name)
        .map(|m| (m.old_cargo_name.clone(), m.new_cargo_name.clone()))
        .collect();
    if rename_map.is_empty() {
        return Ok(());
    }
    let content = read(&lock_path, "Cargo.lock")?;
    let rewritten = oya_cargo_lock_transform_kernel::move_lockfile(
        &content,
        &rename_map,
        &oya_cargo_lock_transform_kernel::GraphAdditions::empty(),
    )
    .map_err(|e| CodemodError::LockfileTransform {
        message: e.to_string(),
    })?;
    if rewritten != content {
        write(&lock_path, "Cargo.lock", &rewritten)?;
        outcome.cargo_lock_changed = true;
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
) -> Result<Vec<String>, CodemodError> {
    let entries =
        oya_workspace_members_kernel::workspace_manifest_entries_from_str(root_manifest_text)
            .map_err(workspace_member_resolution_error)?;
    // Current resolved member dirs from the live (pre-move) tree.
    let current = resolve_member_dirs_from_str(root_manifest_text, repo_root)
        .map_err(workspace_member_resolution_error)?;
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
    Ok(prune)
}

/// Validate root workspace membership before the codemod performs any writes.
fn validate_root_workspace_members(repo_root: &Path) -> Result<(), CodemodError> {
    let root_manifest = repo_root.join("Cargo.toml");
    if !root_manifest.is_file() {
        return Ok(());
    }
    let text = read(&root_manifest, "Cargo.toml")?;
    resolve_member_dirs_from_str(&text, repo_root)
        .map(|_| ())
        .map_err(workspace_member_resolution_error)
}

fn workspace_member_resolution_error(error: ResolveError) -> CodemodError {
    let message = error.to_string();
    match error {
        ResolveError::Read(_) | ResolveError::InspectMemberPath { .. } => CodemodError::Io {
            context: "resolve root workspace members".to_string(),
            message,
        },
        ResolveError::Parse(_) | ResolveError::Shape(_) | ResolveError::MissingManifests(_) => {
            CodemodError::Parse {
                path: "Cargo.toml".to_string(),
                message,
            }
        }
    }
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

/// FAIL-CLOSED: a malformed root manifest (or one lacking the `[workspace]` shape) must
/// propagate, not silently read as "no excludes today" — that would let `rewrite_root_workspace`
/// proceed on wrong data and could re-add or fail to prune entries incorrectly instead of
/// erroring loud on a genuinely corrupt root manifest.
fn current_excludes(root_manifest_text: &str) -> Result<Vec<String>, CodemodError> {
    oya_workspace_members_kernel::workspace_manifest_entries_from_str(root_manifest_text)
        .map(|e| e.exclude)
        .map_err(|e| CodemodError::Parse {
            path: "Cargo.toml".to_string(),
            message: e.to_string(),
        })
}

/// FAIL-CLOSED: see [`current_excludes`] — the same malformed-manifest risk applies to the
/// members list.
fn current_members(root_manifest_text: &str) -> Result<Vec<String>, CodemodError> {
    oya_workspace_members_kernel::workspace_manifest_entries_from_str(root_manifest_text)
        .map(|e| e.members)
        .map_err(|e| CodemodError::Parse {
            path: "Cargo.toml".to_string(),
            message: e.to_string(),
        })
}

/// Resolve the concrete member dirs from a manifest text against a tree (used by the oracle
/// to assert the post-move member set is non-empty / well-formed).
pub fn resolve_members(
    root_manifest_text: &str,
    repo_root: &Path,
) -> Result<Vec<String>, ResolveError> {
    resolve_member_dirs_from_str(root_manifest_text, repo_root)
}

/// True if `rel` exists on disk (file OR directory) under `repo_root`. Used for the artifact
/// pre-flight, where an artifact may be a single SLO/catalog file rather than a directory.
fn path_exists(repo_root: &Path, rel: &str) -> bool {
    repo_root.join(rel).exists()
}

/// Resolve the effective step-8 source path for an artifact, accounting for the case where the
/// artifact's `old_path` is nested inside a crate directory that step 7 has already relocated.
///
/// If `old_path` starts with `<crate.old_path>/` (i.e. is nested under a moving crate dir),
/// the artifact will have been carried along to `<crate.new_path>/<suffix>` by step 7.
/// In that case we return the post-crate-move location so step 8's `git mv` finds the file.
///
/// If `old_path` is NOT nested under any moving crate dir it is returned unchanged (the
/// canonical case: stand-alone SLO/catalog files that live outside any crate tree).
///
/// When moving crates nest (e.g. both `oya/a` and `oya/a/b` move), step 7 relocates dirs
/// LONGEST-old_path-first, so the inner crate's files end up under the INNER crate's new path.
/// We must therefore select the LONGEST matching crate prefix here to mirror step 7 — picking
/// the outer match would point at a path step 7 never created and abort step 8 mid-move.
fn artifact_effective_source(
    a: &crate::model::ArtifactMove,
    by_old_path: &BTreeMap<&str, &CrateMove>,
) -> String {
    // Find the LONGEST moving-crate old_path that is a proper directory prefix of this
    // artifact's old_path. The trailing '/' anchors a directory boundary (avoids e.g.
    // "oya/foo" matching "oya/foobar/...").
    let mut best: Option<(&str, &CrateMove)> = None;
    for (crate_old, crate_move) in by_old_path {
        let prefix_with_slash = format!("{}/", crate_old);
        if a.old_path.starts_with(prefix_with_slash.as_str())
            && best.is_none_or(|(b, _)| crate_old.len() > b.len())
        {
            best = Some((crate_old, crate_move));
        }
    }
    match best {
        Some((crate_old, crate_move)) => {
            let suffix = &a.old_path[crate_old.len() + 1..];
            format!("{}/{}", crate_move.new_path, suffix)
        }
        None => a.old_path.clone(),
    }
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
/// VCS, build-output, and symlinked trees. Symlinks are never followed: a repository-local
/// link can otherwise escape the repository root or form a traversal cycle. Returns sorted
/// repo-relative forward-slash paths.
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
            let metadata = std::fs::symlink_metadata(&path).map_err(|e| CodemodError::Io {
                context: format!("symlink_metadata {}", path.display()),
                message: e.to_string(),
            })?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
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

/// Repo-relative directory the codemod scans for ADR doc-anchor rewrites. Mirrors
/// `oya-ci.toml`'s `[justification].adr_dir` default (`docs/decisions`) — the two tools are
/// independent (this codemod carries no `oya_ci_config_kernel` dependency) but conventionally
/// agree on the path, since both read the SAME repo's ADR corpus.
const ADR_DIR: &str = "docs/decisions";

/// Step 5b: rewrite ADR doc-anchor path citations old -> new for every crate move. A doc citing
/// a moved crate's exact path (e.g. `oya/intelligence/crates/oya-intelligence-catalog-domain/
/// src/lib.rs`) is a load-bearing `justification_ref` anchor for the cloud-ci-total-accounting
/// gate: `resolve_justifications` token-walks every `docs/decisions/*.md` body for tracked-path
/// substrings, and a citation of a currently-live file (not pre-existing baselined debt) is that
/// file's ONLY justification source. Leaving the anchor at the OLD path after a move silently
/// drops the moved file's justification — a genuine `unjustified` regression the ADR-0563
/// rename-aware baseline relabel does NOT cover (relabel only shifts PRE-EXISTING accepted-debt
/// baseline keys; a live, currently-justified citation was never baselined, so there is no key to
/// relabel). Content-preserving: only exact, boundary-safe path-token occurrences are substituted
/// (see [`rewrite_path_token`]); every other byte in the file is untouched. A tree with no ADR
/// dir (e.g. a codemod fixture) is a silent no-op.
fn rewrite_doc_anchors(
    repo_root: &Path,
    plan: &MovePlan,
    outcome: &mut ApplyOutcome,
) -> Result<(), CodemodError> {
    if plan.moves.is_empty() {
        return Ok(());
    }
    let dir = repo_root.join(ADR_DIR);
    // Only a MISSING ADR dir is a legitimate silent no-op (e.g. a codemod fixture with no
    // docs/decisions tree). Any OTHER read_dir failure (permission denied, a non-directory at
    // that path, ...) must propagate — the old `let Ok(..) else` swallowed EVERY error class,
    // which could silently skip a real ADR corpus and drop an anchor rewrite with no signal.
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(CodemodError::Io {
                context: format!("read_dir {}", dir.display()),
                message: e.to_string(),
            });
        }
    };
    let mut doc_paths: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.ends_with(".md") {
            doc_paths.push(format!("{ADR_DIR}/{name}"));
        }
    }
    doc_paths.sort();

    // Longest old_path FIRST — the SAME ordering step 7's directory moves use, and for the
    // identical reason: when one move's old_path is a PREFIX of another's (e.g. `a/b` and
    // `a/b/c` in the same plan), processing the SHORTER one first would match its boundary-safe
    // token at the START of a citation of the LONGER one (`a/b/c/src/lib.rs` starts with the
    // boundary-safe substring `a/b` followed by `/`), corrupting a citation that was actually
    // about the longer, more specific crate. Longest-first guarantees the more specific move
    // consumes its citations before a shorter, coarser old_path ever gets a chance to match.
    let mut ordered_moves: Vec<&CrateMove> = plan.moves.iter().collect();
    ordered_moves.sort_by(|a, b| {
        b.old_path
            .len()
            .cmp(&a.old_path.len())
            .then(a.old_path.cmp(&b.old_path))
    });

    for rel in &doc_paths {
        let abs = repo_root.join(rel);
        let mut text = read(&abs, rel)?;
        let mut file_changed = false;
        for m in &ordered_moves {
            if let Some(rewritten) = rewrite_path_token(&text, &m.old_path, &m.new_path) {
                text = rewritten;
                file_changed = true;
            }
        }
        if file_changed {
            write(&abs, rel, &text)?;
            outcome.docs_rewritten.push(rel.clone());
        }
    }
    Ok(())
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

    // --- rewrite_path_token pure unit tests moved to model.rs (the function's new home). ---
    // --- rewrite_doc_anchors: the full apply_plan integration RED/GREEN proof ---

    fn adr_tmp_root(tag: &str) -> PathBuf {
        artifact_tmp_root(&format!("adr-{tag}"))
    }

    fn build_adr_anchor_fixture(root: &Path) {
        wf(
            root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\", \"intelligence/*/*\"]\nresolver = \"2\"\n",
        );
        wf(
            root,
            "crates/oya-intelligence-catalog-domain/Cargo.toml",
            "[package]\nname = \"oya-intelligence-catalog-domain\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(
            root,
            "crates/oya-intelligence-catalog-domain/src/lib.rs",
            "pub fn d() {}\n",
        );
        // The load-bearing exhibit: an ADR that cites the crate's exact OLD file path as a
        // justification anchor (mirrors ADR-0565's real citation of catalog-domain/src/lib.rs).
        wf(
            root,
            "docs/decisions/ADR-0001-graphql.md",
            "In `crates/oya-intelligence-catalog-domain/src/lib.rs`, the \"graphql\" value is banned.\n",
        );
        // A SIBLING doc that must be left byte-identical (no citation of the moved crate).
        wf(
            root,
            "docs/decisions/ADR-0002-unrelated.md",
            "This ADR mentions nothing about intelligence crates.\n",
        );
    }

    fn adr_anchor_plan() -> MovePlan {
        MovePlan {
            capability: "intelligence".to_string(),
            moves: vec![CrateMove {
                old_path: "crates/oya-intelligence-catalog-domain".to_string(),
                new_path: "intelligence/core/catalog-domain".to_string(),
                old_cargo_name: "oya-intelligence-catalog-domain".to_string(),
                new_cargo_name: "intelligence-catalog-domain".to_string(),
            }],
            artifacts: vec![],
        }
    }

    #[test]
    fn apply_rewrites_an_adr_doc_anchor_old_to_new() {
        let root = adr_tmp_root("rewrite");
        build_adr_anchor_fixture(&root);
        let plan = adr_anchor_plan();

        let outcome = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false }).unwrap();

        let adr = std::fs::read_to_string(root.join("docs/decisions/ADR-0001-graphql.md")).unwrap();
        assert_eq!(
            adr,
            "In `intelligence/core/catalog-domain/src/lib.rs`, the \"graphql\" value is banned.\n",
            "the ADR anchor must be rewritten old -> new, content otherwise byte-identical"
        );
        assert!(
            !adr.contains("crates/oya-intelligence-catalog-domain"),
            "no residual old-path token may remain: {adr}"
        );

        // CANARY: this fails if Step 5b is removed (docs_rewritten would be empty).
        assert_eq!(
            outcome.docs_rewritten,
            vec!["docs/decisions/ADR-0001-graphql.md".to_string()],
            "only the citing doc is touched, and it must be recorded in the outcome"
        );

        // The sibling doc with no citation must be byte-identical (not merely unlisted).
        let unrelated =
            std::fs::read_to_string(root.join("docs/decisions/ADR-0002-unrelated.md")).unwrap();
        assert_eq!(
            unrelated,
            "This ADR mentions nothing about intelligence crates.\n"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_leaves_docs_rewritten_empty_when_no_adr_cites_the_moved_crate() {
        let root = adr_tmp_root("no-citation");
        wf(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\", \"intelligence/*/*\"]\nresolver = \"2\"\n",
        );
        wf(
            &root,
            "crates/oya-intelligence-catalog-domain/Cargo.toml",
            "[package]\nname = \"oya-intelligence-catalog-domain\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(
            &root,
            "crates/oya-intelligence-catalog-domain/src/lib.rs",
            "pub fn d() {}\n",
        );
        // No docs/decisions dir at all — the RED case this must NOT crash on.
        let plan = adr_anchor_plan();

        let outcome = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false }).unwrap();
        assert!(
            outcome.docs_rewritten.is_empty(),
            "no ADR dir/citation ⇒ nothing rewritten: {:?}",
            outcome.docs_rewritten
        );
        assert!(root.join("intelligence/core/catalog-domain/Cargo.toml").is_file());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_rewrites_doc_anchors_correctly_when_one_old_path_is_a_prefix_of_another() {
        // ORDERING PROOF: two moves in the SAME plan where one's old_path (`crates/a`) is a
        // PREFIX of the other's (`crates/a/nested`) — a real, anticipated shape (step 7's
        // directory-move already sorts longest-first for exactly this reason: a nested crate
        // dir must move before/independent of its outer sibling). Processing `plan.moves`
        // UNSORTED would let the SHORTER old_path's boundary-safe match fire at the START of a
        // citation of the LONGER, more specific crate (`crates/a` matches the first segment of
        // `crates/a/nested/src/lib.rs`, followed by `/` — a valid trailing boundary), corrupting
        // it to `capA/nested/src/lib.rs` instead of the correct `capB/src/lib.rs`.
        let root = adr_tmp_root("nested-ordering");
        wf(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/a\", \"crates/a/nested\"]\nresolver = \"2\"\n",
        );
        wf(
            &root,
            "crates/a/Cargo.toml",
            "[package]\nname = \"oya-a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(&root, "crates/a/src/lib.rs", "pub fn a() {}\n");
        wf(
            &root,
            "crates/a/nested/Cargo.toml",
            "[package]\nname = \"oya-a-nested\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(&root, "crates/a/nested/src/lib.rs", "pub fn nested() {}\n");
        wf(
            &root,
            "docs/decisions/ADR-0001-both.md",
            "See `crates/a/nested/src/lib.rs` and also `crates/a/src/lib.rs`.\n",
        );

        let plan = MovePlan {
            capability: "test".to_string(),
            moves: vec![
                CrateMove {
                    old_path: "crates/a".to_string(),
                    new_path: "capA".to_string(),
                    old_cargo_name: "oya-a".to_string(),
                    new_cargo_name: "cap-a".to_string(),
                },
                CrateMove {
                    old_path: "crates/a/nested".to_string(),
                    new_path: "capB".to_string(),
                    old_cargo_name: "oya-a-nested".to_string(),
                    new_cargo_name: "cap-b".to_string(),
                },
            ],
            artifacts: vec![],
        };

        apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false }).unwrap();

        let adr = std::fs::read_to_string(root.join("docs/decisions/ADR-0001-both.md")).unwrap();
        assert_eq!(
            adr,
            "See `capB/src/lib.rs` and also `capA/src/lib.rs`.\n",
            "the nested crate's citation must resolve to its OWN new_path, not get swallowed by \
             the shorter sibling old_path processed out of order: {adr}"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    // --- ArtifactMove (PR-A) apply tests: NON-crate co-move (SLOs, catalog records) ---

    use crate::model::{ArtifactMove, CrateMove, MovePlan};
    use std::sync::atomic::{AtomicU64, Ordering};

    static ARTIFACT_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn artifact_tmp_root(tag: &str) -> PathBuf {
        let unique = format!(
            "oya-reorg-artifact-{}-{}-{}",
            tag,
            std::process::id(),
            ARTIFACT_TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let root = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn wf(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    #[test]
    fn apply_fails_before_mutation_when_workspace_member_resolution_fails() {
        let root = artifact_tmp_root("workspace-resolve-fail");
        wf(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\", \"libs/*\"]\nresolver = \"2\"\n",
        );
        wf(
            &root,
            "crates/oya-widget/Cargo.toml",
            "[package]\nname = \"oya-widget\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(
            &root,
            "crates/oya-widget/src/lib.rs",
            "pub fn widget() {}\n",
        );
        wf(
            &root,
            "libs/keep/Cargo.toml",
            "[package]\nname = \"keep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        std::fs::create_dir_all(root.join("libs/missing-manifest")).unwrap();

        let plan = MovePlan {
            capability: "widget".to_string(),
            moves: vec![CrateMove {
                old_path: "crates/oya-widget".to_string(),
                new_path: "widget/core/widget".to_string(),
                old_cargo_name: "oya-widget".to_string(),
                new_cargo_name: "widget-domain".to_string(),
            }],
            artifacts: vec![],
        };
        let root_manifest_before = std::fs::read(root.join("Cargo.toml")).unwrap();
        let crate_manifest_before =
            std::fs::read(root.join("crates/oya-widget/Cargo.toml")).unwrap();

        let error = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false })
            .expect_err("invalid current workspace membership must abort the codemod");

        assert!(
            error.to_string().contains("libs/missing-manifest"),
            "the typed resolver error must remain visible: {error}"
        );
        assert_eq!(
            std::fs::read(root.join("Cargo.toml")).unwrap(),
            root_manifest_before,
            "a failed resolver preflight must not prune unrelated workspace globs"
        );
        assert_eq!(
            std::fs::read(root.join("crates/oya-widget/Cargo.toml")).unwrap(),
            crate_manifest_before,
            "a failed resolver preflight must precede package rewrites"
        );
        assert!(root.join("crates/oya-widget").is_dir());
        assert!(!root.join("widget/core/widget").exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    /// A minimal one-crate workspace plus a NON-crate SLO dir + a catalog file, so an artifact
    /// co-move has something to move. `use_git_mv: false` => plain fs::rename (no git needed).
    fn build_artifact_fixture(root: &Path) {
        wf(
            root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\", \"observability/*/*\"]\nresolver = \"2\"\n",
        );
        wf(
            root,
            "crates/oya-observability-domain/Cargo.toml",
            "[package]\nname = \"oya-observability-domain\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(
            root,
            "crates/oya-observability-domain/src/lib.rs",
            "pub fn d() {}\n",
        );
        // NON-crate artifacts: an SLO dir (2 files) and a single catalog file.
        wf(
            root,
            "oya/observability/slos/api-availability.openslo.yaml",
            "apiVersion: openslo/v1\nkind: SLO\n",
        );
        wf(
            root,
            "oya/observability/slos/api-latency.openslo.yaml",
            "apiVersion: openslo/v1\nkind: SLO\n",
        );
        wf(
            root,
            "registry/catalog/oya-observability-domain.yaml",
            "crate_id: oya-observability-domain\n",
        );
    }

    fn observability_plan_with_artifacts() -> MovePlan {
        MovePlan {
            capability: "observability".to_string(),
            moves: vec![CrateMove {
                old_path: "crates/oya-observability-domain".to_string(),
                new_path: "observability/core/domain".to_string(),
                old_cargo_name: "oya-observability-domain".to_string(),
                new_cargo_name: "observability-domain".to_string(),
            }],
            artifacts: vec![
                ArtifactMove {
                    old_path: "oya/observability/slos".to_string(),
                    new_path: "observability/observability/slos".to_string(),
                },
                ArtifactMove {
                    old_path: "registry/catalog/oya-observability-domain.yaml".to_string(),
                    new_path: "registry/catalog/observability-domain.yaml".to_string(),
                },
            ],
        }
    }

    #[test]
    fn apply_co_moves_artifacts_and_records_them_in_dirs_moved() {
        let root = artifact_tmp_root("apply");
        build_artifact_fixture(&root);
        let plan = observability_plan_with_artifacts();

        let outcome = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false }).unwrap();

        // (1) the DIR artifact moved wholesale: NEW descendants present, OLD dir gone.
        assert!(root
            .join("observability/observability/slos/api-availability.openslo.yaml")
            .is_file());
        assert!(root
            .join("observability/observability/slos/api-latency.openslo.yaml")
            .is_file());
        assert!(!root.join("oya/observability/slos").exists());
        // (2) the FILE artifact moved (content preserved, no in-file rewrite).
        let cat = root.join("registry/catalog/observability-domain.yaml");
        assert!(cat.is_file());
        assert!(!root
            .join("registry/catalog/oya-observability-domain.yaml")
            .exists());
        assert_eq!(
            std::fs::read_to_string(&cat).unwrap(),
            "crate_id: oya-observability-domain\n",
            "artifact move is content-preserving (no rewrite of SLO/catalog YAML)"
        );
        // (3) the crate move also landed.
        assert!(root.join("observability/core/domain/Cargo.toml").is_file());
        // (4) CANARY: artifacts appear in dirs_moved (so total-accounting / the manifest see them).
        //     This FAILS if step-8 is removed (artifacts would be absent from dirs_moved).
        assert!(
            outcome
                .dirs_moved
                .contains(&(
                    "oya/observability/slos".to_string(),
                    "observability/observability/slos".to_string()
                )),
            "step-8 must record the dir artifact in dirs_moved: {:?}",
            outcome.dirs_moved
        );
        assert!(
            outcome.dirs_moved.contains(&(
                "registry/catalog/oya-observability-domain.yaml".to_string(),
                "registry/catalog/observability-domain.yaml".to_string()
            )),
            "step-8 must record the file artifact in dirs_moved: {:?}",
            outcome.dirs_moved
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn manifest_files_list_carries_artifact_pairs_for_a_real_move() {
        // NON-VACUOUS CANARY: build the manifest `files` list EXACTLY as `cmd_manifest` does
        // (file_level_manifest merged with artifact_file_pairs over the candidate POST-move tree),
        // and assert the artifact pairs are PRESENT. This FAILS if the manifest-merge of
        // artifact_file_pairs is removed (the artifact pairs would vanish from `files`).
        let plan = observability_plan_with_artifacts();
        // Candidate POST-move tracked tree: NEW paths present, OLD paths gone.
        let tracked = vec![
            "observability/core/domain/Cargo.toml".to_string(),
            "observability/core/domain/src/lib.rs".to_string(),
            "observability/observability/slos/api-availability.openslo.yaml".to_string(),
            "observability/observability/slos/api-latency.openslo.yaml".to_string(),
            "registry/catalog/observability-domain.yaml".to_string(),
        ];
        let mut files: std::collections::BTreeMap<String, String> =
            plan.file_level_manifest(&tracked).into_iter().collect();
        for (old, new) in plan.artifact_file_pairs(&tracked) {
            files.insert(old, new);
        }
        // crate file pair is present (sanity)...
        assert_eq!(
            files.get("crates/oya-observability-domain/src/lib.rs"),
            Some(&"observability/core/domain/src/lib.rs".to_string())
        );
        // ...AND the artifact pairs are present (the canary).
        assert_eq!(
            files.get("oya/observability/slos/api-availability.openslo.yaml"),
            Some(&"observability/observability/slos/api-availability.openslo.yaml".to_string()),
            "dir-artifact descendant must be in the manifest files list"
        );
        assert_eq!(
            files.get("registry/catalog/oya-observability-domain.yaml"),
            Some(&"registry/catalog/observability-domain.yaml".to_string()),
            "file-artifact must be in the manifest files list"
        );
    }

    /// An artifact whose old_path is nested inside a moving crate dir is carried by step 7 and
    /// must be sourced from its post-crate-move location in step 8.  This test proves the codemod
    /// succeeds (no SourceMissing error) and that the artifact lands at its declared new_path.
    #[test]
    fn apply_co_moves_artifact_nested_inside_moving_crate_dir() {
        let root = artifact_tmp_root("nested");
        // Workspace with one crate that contains an slos/ subdir (the nested artifact).
        wf(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"2\"\n",
        );
        wf(
            &root,
            "crates/oya-svid-kernel/Cargo.toml",
            "[package]\nname = \"oya-svid-kernel\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(&root, "crates/oya-svid-kernel/src/lib.rs", "pub fn f() {}\n");
        // The SLO lives INSIDE the crate dir — exactly the iam svid-kernel scenario.
        wf(
            &root,
            "crates/oya-svid-kernel/slos/svid-availability.openslo.yaml",
            "apiVersion: openslo/v1\nkind: SLO\n",
        );

        let plan = MovePlan {
            capability: "iam".to_string(),
            moves: vec![CrateMove {
                old_path: "crates/oya-svid-kernel".to_string(),
                new_path: "iam/core/svid-kernel".to_string(),
                old_cargo_name: "oya-svid-kernel".to_string(),
                new_cargo_name: "iam-svid-kernel".to_string(),
            }],
            // old_path is nested under the crate dir — step 7 will carry it to the new crate
            // location before step 8 runs.  The codemod must resolve the effective source.
            artifacts: vec![ArtifactMove {
                old_path: "crates/oya-svid-kernel/slos/svid-availability.openslo.yaml"
                    .to_string(),
                new_path: "iam/observability/slos/svid-kernel/svid-availability.openslo.yaml"
                    .to_string(),
            }],
        };

        let outcome = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false })
            .expect("apply must succeed for artifact nested inside moving crate dir");

        // Crate landed at new location.
        assert!(root.join("iam/core/svid-kernel/Cargo.toml").is_file());
        // Artifact landed at its declared new_path (NOT still inside the new crate dir).
        assert!(
            root.join("iam/observability/slos/svid-kernel/svid-availability.openslo.yaml")
                .is_file(),
            "artifact must land at new_path"
        );
        // Old crate dir is gone.
        assert!(!root.join("crates/oya-svid-kernel").exists());
        // Artifact is NOT left behind in the new crate dir (step 8 moved it out).
        assert!(
            !root
                .join("iam/core/svid-kernel/slos/svid-availability.openslo.yaml")
                .exists(),
            "artifact must not remain inside the new crate dir after step 8"
        );
        // dirs_moved records the canonical old_path (not the effective source).
        assert!(
            outcome.dirs_moved.contains(&(
                "crates/oya-svid-kernel/slos/svid-availability.openslo.yaml".to_string(),
                "iam/observability/slos/svid-kernel/svid-availability.openslo.yaml".to_string(),
            )),
            "dirs_moved must use the canonical old_path for manifest relabel: {:?}",
            outcome.dirs_moved
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_resolves_artifact_under_inner_of_two_nested_moving_crates_by_longest_prefix() {
        // REGRESSION (PR #763 re-review): when two moving crates NEST (`oya/a` and `oya/a/b`)
        // and an artifact lives under the INNER crate, step 7 moves crates longest-old_path-first
        // so the file ends up under the INNER crate's new path. artifact_effective_source must
        // select the LONGEST matching crate prefix; picking the outer match points at a path that
        // never exists and aborts step 8 mid-move.
        let root = artifact_tmp_root("nested-pair");
        wf(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"oya/a\", \"oya/a/b\"]\nresolver = \"2\"\n",
        );
        wf(
            &root,
            "oya/a/Cargo.toml",
            "[package]\nname = \"oya-a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(&root, "oya/a/src/lib.rs", "pub fn a() {}\n");
        wf(
            &root,
            "oya/a/b/Cargo.toml",
            "[package]\nname = \"oya-a-b\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(&root, "oya/a/b/src/lib.rs", "pub fn b() {}\n");
        // Artifact nested under the INNER crate (oya/a/b), so both crate prefixes match.
        wf(
            &root,
            "oya/a/b/slos/x.openslo.yaml",
            "apiVersion: openslo/v1\nkind: SLO\n",
        );

        let plan = MovePlan {
            capability: "cap".to_string(),
            moves: vec![
                CrateMove {
                    old_path: "oya/a".to_string(),
                    new_path: "cap/core/a".to_string(),
                    old_cargo_name: "oya-a".to_string(),
                    new_cargo_name: "cap-a".to_string(),
                },
                CrateMove {
                    old_path: "oya/a/b".to_string(),
                    new_path: "cap/core/b".to_string(),
                    old_cargo_name: "oya-a-b".to_string(),
                    new_cargo_name: "cap-b".to_string(),
                },
            ],
            artifacts: vec![ArtifactMove {
                old_path: "oya/a/b/slos/x.openslo.yaml".to_string(),
                new_path: "cap/observability/slos/x.openslo.yaml".to_string(),
            }],
        };

        let outcome = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false })
            .expect("apply must succeed when an artifact nests under the inner of two moving crates");

        // Both crates landed.
        assert!(root.join("cap/core/a/Cargo.toml").is_file());
        assert!(root.join("cap/core/b/Cargo.toml").is_file());
        // Artifact reached its declared destination (longest-prefix resolution found it under b).
        assert!(
            root.join("cap/observability/slos/x.openslo.yaml").is_file(),
            "artifact must land at new_path via the INNER (longest) crate prefix"
        );
        // It was NOT left inside the inner crate's new dir, nor mis-sourced under the outer crate.
        assert!(!root.join("cap/core/b/slos/x.openslo.yaml").exists());
        assert!(!root.join("cap/core/a/b/slos/x.openslo.yaml").exists());
        // dirs_moved still records the canonical old_path for the relabel engine.
        assert!(outcome.dirs_moved.contains(&(
            "oya/a/b/slos/x.openslo.yaml".to_string(),
            "cap/observability/slos/x.openslo.yaml".to_string(),
        )));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_with_empty_artifacts_is_byte_identical_no_op_for_the_artifact_step() {
        // BACK-COMPAT: a plan with the SAME crate move but NO artifacts moves ONLY the crate and
        // records ONLY the crate in dirs_moved — provably identical to a pre-ArtifactMove apply.
        let root = artifact_tmp_root("noop");
        build_artifact_fixture(&root);
        let mut plan = observability_plan_with_artifacts();
        plan.artifacts.clear();

        let outcome = apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false }).unwrap();

        // the crate moved...
        assert!(root.join("observability/core/domain/Cargo.toml").is_file());
        // ...but the artifacts did NOT (no artifact step fired).
        assert!(root.join("oya/observability/slos").is_dir());
        assert!(root
            .join("registry/catalog/oya-observability-domain.yaml")
            .is_file());
        // dirs_moved carries the crate ONLY.
        assert_eq!(
            outcome.dirs_moved,
            vec![(
                "crates/oya-observability-domain".to_string(),
                "observability/core/domain".to_string()
            )],
            "empty artifacts => dirs_moved is the crate move alone (back-compat no-op)"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    // --- Step-6b Cargo.lock co-move: a crate relocation must ALSO relocate the crate's
    // Cargo.lock package entry byte-identically, via the owned pure transform (no cargo). ---

    /// The canonical (cargo-generated) lockfile the fixture starts from: `oya-widget` is a member,
    /// referenced by `alpha`, and sits between `alpha` and `zeta` in name order.
    const INPUT_LOCK: &str = "\
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = \"alpha\"
version = \"0.1.0\"
dependencies = [
 \"oya-widget\",
]

[[package]]
name = \"oya-widget\"
version = \"0.1.0\"

[[package]]
name = \"zeta\"
version = \"0.1.0\"
";

    /// After moving `oya-widget` -> `zzz-widget`: the package block is RENAMED, the `alpha`
    /// dependency reference is renamed, and the block RELOCATES to its new canonical position
    /// (past `zeta`, since `zeta` < `zzz-widget`). This is exactly `cargo`'s canonical output —
    /// the load-bearing byte-identity property.
    const EXPECTED_LOCK: &str = "\
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = \"alpha\"
version = \"0.1.0\"
dependencies = [
 \"zzz-widget\",
]

[[package]]
name = \"zeta\"
version = \"0.1.0\"

[[package]]
name = \"zzz-widget\"
version = \"0.1.0\"
";

    fn lock_move_fixture(root: &Path) {
        wf(
            root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"oya/*\", \"widget/*/*\"]\nresolver = \"2\"\n",
        );
        wf(
            root,
            "oya/widget/Cargo.toml",
            "[package]\nname = \"oya-widget\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(root, "oya/widget/src/lib.rs", "pub fn w() {}\n");
        wf(root, "Cargo.lock", INPUT_LOCK);
    }

    fn lock_move_plan() -> MovePlan {
        MovePlan {
            capability: "widget".to_string(),
            moves: vec![CrateMove {
                old_path: "oya/widget".to_string(),
                new_path: "widget/core/widget".to_string(),
                old_cargo_name: "oya-widget".to_string(),
                new_cargo_name: "zzz-widget".to_string(),
            }],
            artifacts: vec![],
        }
    }

    #[test]
    fn apply_relocates_cargo_lock_entry_byte_identically() {
        let root = artifact_tmp_root("lock");
        lock_move_fixture(&root);

        let outcome = apply_plan(&root, &lock_move_plan(), &ApplyOptions { use_git_mv: false })
            .expect("apply must succeed and relocate the lock entry");

        // The crate dir moved (sanity).
        assert!(root.join("widget/core/widget/Cargo.toml").is_file());
        // LOAD-BEARING: the Cargo.lock package entry was relocated BYTE-IDENTICALLY to cargo's
        // canonical output — renamed + re-sorted, with NO cargo invoked.
        let got = std::fs::read_to_string(root.join("Cargo.lock")).unwrap();
        assert_eq!(got, EXPECTED_LOCK, "lock entry must relocate byte-identically");
        assert!(
            outcome.cargo_lock_changed,
            "the outcome must report the lockfile change"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn inverse_apply_restores_cargo_lock_byte_identically() {
        let root = artifact_tmp_root("lock-inv");
        lock_move_fixture(&root);
        let plan = lock_move_plan();

        apply_plan(&root, &plan, &ApplyOptions { use_git_mv: false }).unwrap();
        let inverse = apply_plan(&root, &plan.inverse(), &ApplyOptions { use_git_mv: false })
            .expect("inverse apply must succeed");

        // Reversibility-by-construction: --revert restores the original lockfile byte-for-byte.
        let got = std::fs::read_to_string(root.join("Cargo.lock")).unwrap();
        assert_eq!(got, INPUT_LOCK, "inverse must restore the lock byte-identically");
        assert!(inverse.cargo_lock_changed);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_without_a_lockfile_is_a_no_op_for_the_lock_step() {
        // BACK-COMPAT: a fixture tree with NO Cargo.lock (the existing roundtrip fixtures) must
        // leave the lock step a no-op — no file created, cargo_lock_changed stays false.
        let root = artifact_tmp_root("lock-absent");
        wf(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"oya/*\", \"widget/*/*\"]\nresolver = \"2\"\n",
        );
        wf(
            &root,
            "oya/widget/Cargo.toml",
            "[package]\nname = \"oya-widget\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        wf(&root, "oya/widget/src/lib.rs", "pub fn w() {}\n");

        let outcome = apply_plan(&root, &lock_move_plan(), &ApplyOptions { use_git_mv: false }).unwrap();

        assert!(!root.join("Cargo.lock").exists(), "no lockfile must be created");
        assert!(!outcome.cargo_lock_changed);

        let _ = std::fs::remove_dir_all(&root);
    }
}
