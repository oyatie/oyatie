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
//! 3. **Graph equivalence** ([`prove_graph_equivalence`]): diff a BEFORE snapshot against an
//!    AFTER snapshot under the plan's bijection. A pure relocation must yield the same targets
//!    and the same dependency edges, only renamed. When that holds, the move needs no
//!    full-workspace rebuild to be trusted — which is the difference between a set comparison
//!    and building 900+ crates twice. Capturing snapshots without ever diffing them proves
//!    nothing, so this is what makes (1) load-bearing rather than decorative.
//!
//! The dry-run is the safety gate. The engine refuses to land a move whose dry-run fails.

use std::collections::BTreeSet;
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

    let (cargo_ok, cargo_detail) = run_cargo_metadata(&shadow);
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

/// The verdict of a graph-equivalence proof across a move.
///
/// A pure relocation must not change the build graph: the same targets with the same dependency
/// edges, renamed by the plan's bijection. When that holds, a move PR needs no full-workspace
/// rebuild — the equivalence IS the proof, and it costs a set comparison instead of ~60 minutes
/// of building and testing 900+ crates twice.
#[derive(Debug, Clone, Default)]
pub struct GraphEquivalence {
    /// True iff every checked graph is isomorphic under the plan's bijection.
    pub equivalent: bool,
    /// False when the before/after snapshots could not both be read (fail-closed: never
    /// `equivalent` without evidence).
    pub cargo_checked: bool,
    pub buck_checked: bool,
    /// Present in the relabelled BEFORE graph, absent from AFTER (a target/package the move lost).
    pub only_before: Vec<String>,
    /// Present in AFTER, absent from the relabelled BEFORE (a target/package the move invented).
    pub only_after: Vec<String>,
    /// Human-readable account of what was and was not proven.
    pub detail: String,
}

/// Prove that `after` is the same build graph as `before` modulo the plan's bijection.
///
/// FAIL-CLOSED: any snapshot that did not resolve, or a buck2 that was unavailable on either
/// side, yields `equivalent: false` with the reason in `detail`. An unproven graph is never
/// reported as equivalent — that is the whole point, and it is the failure mode that makes the
/// codemod's own dry-run oracle untrustworthy when buck2 is off PATH.
/// `declared` names differences the author has justified in the plan/ADR (e.g. build targets the
/// codemod deliberately does not rename). A declared entry only ever REMOVES a reported
/// difference — it can never make an unchecked graph look checked, and an undeclared difference
/// still fails. Declaring is therefore auditable: the list is the complete set of ways this move
/// changed the graph.
pub fn prove_graph_equivalence(
    before: &GreenSnapshot,
    after: &GreenSnapshot,
    plan: &MovePlan,
    declared: &[String],
) -> GraphEquivalence {
    let dirs: Vec<(String, String)> = plan
        .moves
        .iter()
        .map(|m| (m.old_path.clone(), m.new_path.clone()))
        .collect();
    let names: Vec<(String, String)> = plan
        .moves
        .iter()
        .map(|m| (m.old_cargo_name.clone(), m.new_cargo_name.clone()))
        .collect();

    let mut out = GraphEquivalence::default();
    let mut notes: Vec<String> = Vec::new();
    let mut only_before: BTreeSet<String> = BTreeSet::new();
    let mut only_after: BTreeSet<String> = BTreeSet::new();

    // ---- cargo: package name -> its first-party dependency names, bijection applied ----
    if before.cargo_ok && after.cargo_ok {
        match (
            cargo_package_edges(&before.cargo_metadata),
            cargo_package_edges(&after.cargo_metadata),
        ) {
            (Some(b), Some(a)) if !b.is_empty() && !a.is_empty() => {
                let relabelled: BTreeSet<String> = b
                    .iter()
                    .map(|edge| relabel_cargo_edge(edge, &names))
                    .collect();
                only_before.extend(relabelled.difference(&a).map(|s| format!("cargo:{s}")));
                only_after.extend(a.difference(&relabelled).map(|s| format!("cargo:{s}")));
                // PLAN COVERAGE: every crate the plan claims to move must be VISIBLE at its
                // destination. Without this the proof is vacuous on a truncated or filtered
                // graph — two sets that both omit the moved crates agree trivially.
                let after_names: BTreeSet<&str> =
                    a.iter().filter_map(|e| e.split('|').next()).collect();
                let missing: Vec<&str> = plan
                    .moves
                    .iter()
                    .map(|m| m.new_cargo_name.as_str())
                    .filter(|n| !after_names.contains(n))
                    .collect();
                if missing.is_empty() {
                    out.cargo_checked = true;
                } else {
                    notes.push(format!(
                        "cargo: {} planned crate(s) absent from the post-move graph (e.g. {}); \
                         equivalence NOT proven",
                        missing.len(),
                        missing[0]
                    ));
                }
                notes.push(format!(
                    "cargo: {} packages before, {} after",
                    b.len(),
                    a.len()
                ));
            }
            (Some(_), Some(_)) => notes
                .push("cargo: an EMPTY package graph proves nothing; equivalence NOT proven".to_owned()),
            _ => notes.push("cargo: metadata did not parse; equivalence NOT proven".to_owned()),
        }
    } else {
        notes.push("cargo: a snapshot did not resolve; equivalence NOT proven".to_owned());
    }

    // ---- buck2: the target label set, bijection applied ----
    if before.buck_available && after.buck_available && before.buck_ok && after.buck_ok {
        let b: BTreeSet<String> = buck_labels(&before.buck_targets)
            .into_iter()
            .map(|l| relabel_buck_label(&l, &dirs, &names))
            .collect();
        let a: BTreeSet<String> = buck_labels(&after.buck_targets).into_iter().collect();
        only_before.extend(b.difference(&a).map(|s| format!("buck:{s}")));
        only_after.extend(a.difference(&b).map(|s| format!("buck:{s}")));
        if b.is_empty() || a.is_empty() {
            notes.push(
                "buck2: an EMPTY target graph proves nothing; equivalence NOT proven".to_owned(),
            );
        } else {
            // PLAN COVERAGE, buck2 side: each destination package must own at least one target.
            let missing: Vec<&str> = plan
                .moves
                .iter()
                .map(|m| m.new_path.as_str())
                .filter(|dest| {
                    let needle = format!("//{dest}:");
                    !a.iter().any(|l| l.contains(&needle))
                })
                .collect();
            if missing.is_empty() {
                out.buck_checked = true;
            } else {
                notes.push(format!(
                    "buck2: {} destination package(s) own no target (e.g. {}); equivalence NOT \
                     proven",
                    missing.len(),
                    missing[0]
                ));
            }
        }
        notes.push(format!("buck2: {} targets before, {} after", b.len(), a.len()));
    } else {
        notes.push(
            "buck2: unavailable or a snapshot did not resolve; equivalence NOT proven".to_owned(),
        );
    }

    let is_declared = |s: &String| declared.iter().any(|d| s.contains(d.as_str()));
    let declared_hits = only_before.iter().filter(|s| is_declared(s)).count()
        + only_after.iter().filter(|s| is_declared(s)).count();
    if declared_hits > 0 {
        notes.push(format!("{declared_hits} declared difference(s) excluded"));
    }
    out.only_before = only_before.into_iter().filter(|s| !is_declared(s)).collect();
    out.only_after = only_after.into_iter().filter(|s| !is_declared(s)).collect();
    // Both graphs must have been checked AND agree. A cargo-only proof is not a graph proof:
    // cargo cannot see buck2-only targets (tests, bins, genrules), which is exactly where a
    // relocation breaks.
    out.equivalent = out.cargo_checked
        && out.buck_checked
        && out.only_before.is_empty()
        && out.only_after.is_empty();
    if !out.equivalent && out.only_before.is_empty() && out.only_after.is_empty() {
        notes.push("no differences found, but not every graph was checked".to_owned());
    }
    out.detail = notes.join("; ");
    out
}

/// Prove a LANDED move preserved the build graph, with no saved snapshot required.
///
/// Captures AFTER at `repo_root`, then reconstructs BEFORE by INVERSE-applying the plan into a
/// throwaway shadow, and diffs the two under the bijection. The reconstruction is what makes
/// this usable in CI on a candidate tree: the pre-move graph is derived from the post-move tree
/// plus the plan, so nothing has to be carried across runs.
///
/// A `true` verdict means a full-workspace rebuild proves nothing this comparison has not
/// already proven — the targets and edges are identical modulo renaming.
pub fn prove_move(
    repo_root: &Path,
    plan: &MovePlan,
    run_buck: bool,
    declared: &[String],
) -> Result<GraphEquivalence, CodemodError> {
    plan.validate()?;
    let after = capture_snapshot(repo_root, run_buck);
    let shadow = make_shadow(repo_root)?;
    let reconstructed = apply_plan(&shadow, &plan.inverse(), &ApplyOptions { use_git_mv: false });
    let verdict = match reconstructed {
        Ok(_) => {
            let before = capture_snapshot(&shadow, run_buck);
            prove_graph_equivalence(&before, &after, plan, declared)
        }
        Err(error) => GraphEquivalence {
            detail: format!("could not reconstruct the pre-move tree: {error}"),
            ..GraphEquivalence::default()
        },
    };
    let _ = std::fs::remove_dir_all(&shadow);
    Ok(verdict)
}

/// Split `buck2 targets //...` stdout into target labels, ignoring blank/non-label lines.
fn buck_labels(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && l.contains("//") && l.contains(':'))
        .map(str::to_owned)
        .collect()
}

/// Apply the move bijection to a buck2 label `cell//package/path:target`.
fn relabel_buck_label(
    label: &str,
    dirs: &[(String, String)],
    names: &[(String, String)],
) -> String {
    let Some((cell_pkg, target)) = label.rsplit_once(':') else {
        return label.to_owned();
    };
    let (cell, pkg) = match cell_pkg.split_once("//") {
        Some((c, p)) => (c, p),
        None => ("", cell_pkg),
    };
    let new_pkg = remap_prefix(pkg, dirs, '/');
    let new_target = remap_prefix(target, names, '-');
    format!("{cell}//{new_pkg}:{new_target}")
}

/// Longest-match-first prefix remap. `value` maps when it equals `old` or begins with
/// `old` followed by `sep` — so `a/b` and `a/b/c` both remap under `a/b`, while `a/bc` does not.
/// Longest-first matters: a plan may move both `x` and `x/y`, and the wrong order silently
/// produces a path that never existed.
fn remap_prefix(value: &str, pairs: &[(String, String)], sep: char) -> String {
    let mut best: Option<(&str, &str)> = None;
    for (old, new) in pairs {
        let matches = value == old.as_str()
            || (value.len() > old.len()
                && value.starts_with(old.as_str())
                && value[old.len()..].starts_with(sep));
        if matches && best.is_none_or(|(b, _)| old.len() > b.len()) {
            best = Some((old.as_str(), new.as_str()));
        }
    }
    match best {
        Some((old, new)) => format!("{new}{}", &value[old.len()..]),
        None => value.to_owned(),
    }
}

/// Reduce `cargo metadata --no-deps` stdout to `name|dep,dep,...` rows: the package set plus its
/// first-party dependency edges. Paths are deliberately excluded — relocating IS the change, so
/// comparing paths would always differ; names carry the bijection and the edges carry the graph.
fn cargo_package_edges(metadata: &str) -> Option<BTreeSet<String>> {
    let value: serde_json::Value = serde_json::from_str(metadata).ok()?;
    let packages = value.get("packages")?.as_array()?;
    let mut out = BTreeSet::new();
    for pkg in packages {
        let name = pkg.get("name")?.as_str()?;
        let mut deps: Vec<&str> = pkg
            .get("dependencies")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| d.get("name").and_then(|n| n.as_str()))
                    .collect()
            })
            .unwrap_or_default();
        deps.sort_unstable();
        deps.dedup();
        out.insert(format!("{name}|{}", deps.join(",")));
    }
    Some(out)
}

/// Apply the name bijection to a `name|dep,dep` row (both sides of the edge).
fn relabel_cargo_edge(edge: &str, names: &[(String, String)]) -> String {
    let (name, deps) = edge.split_once('|').unwrap_or((edge, ""));
    let new_name = remap_prefix(name, names, '-');
    let new_deps: Vec<String> = deps
        .split(',')
        .filter(|d| !d.is_empty())
        .map(|d| remap_prefix(d, names, '-'))
        .collect();
    let mut sorted = new_deps;
    sorted.sort();
    format!("{new_name}|{}", sorted.join(","))
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
    use crate::model::CrateMove;
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

    // ---- graph-equivalence proof ----------------------------------------------------------

    fn os_plan() -> MovePlan {
        MovePlan {
            capability: "os".to_string(),
            moves: vec![CrateMove {
                old_path: "cloud/cloud-os/crates/oya-cloud-os-apid-domain".to_string(),
                new_path: "os/core/apid-domain".to_string(),
                old_cargo_name: "oya-cloud-os-apid-domain".to_string(),
                new_cargo_name: "os-apid-domain".to_string(),
            }],
            artifacts: vec![],
        }
    }

    fn snapshot(cargo: &str, buck: &str) -> GreenSnapshot {
        GreenSnapshot {
            cargo_metadata: cargo.to_string(),
            buck_targets: buck.to_string(),
            cargo_ok: true,
            buck_ok: true,
            buck_available: true,
        }
    }

    const BEFORE_CARGO: &str = r#"{"packages":[
        {"name":"oya-cloud-os-apid-domain","dependencies":[{"name":"serde"}]},
        {"name":"iam-pdp","dependencies":[{"name":"oya-cloud-os-apid-domain"}]}]}"#;
    const AFTER_CARGO: &str = r#"{"packages":[
        {"name":"os-apid-domain","dependencies":[{"name":"serde"}]},
        {"name":"iam-pdp","dependencies":[{"name":"os-apid-domain"}]}]}"#;
    const BEFORE_BUCK: &str = "\
root//cloud/cloud-os/crates/oya-cloud-os-apid-domain:oya-cloud-os-apid-domain
root//cloud/cloud-os/crates/oya-cloud-os-apid-domain:oya-cloud-os-apid-domain-unittest
root//iam/facade/pdp:iam-pdp";
    const AFTER_BUCK: &str = "\
root//os/core/apid-domain:os-apid-domain
root//os/core/apid-domain:os-apid-domain-unittest
root//iam/facade/pdp:iam-pdp";

    #[test]
    fn pure_relocation_is_graph_equivalent_under_the_bijection() {
        let out = prove_graph_equivalence(
            &snapshot(BEFORE_CARGO, BEFORE_BUCK),
            &snapshot(AFTER_CARGO, AFTER_BUCK),
            &os_plan(),
            &[],
        );
        assert!(out.equivalent, "expected equivalence; detail={}", out.detail);
        assert!(out.cargo_checked && out.buck_checked);
        assert!(out.only_before.is_empty() && out.only_after.is_empty());
    }

    #[test]
    fn a_dropped_target_breaks_equivalence() {
        // The unittest target vanished — exactly the class a relocation silently loses.
        let after_buck = "root//os/core/apid-domain:os-apid-domain\nroot//iam/facade/pdp:iam-pdp";
        let out = prove_graph_equivalence(
            &snapshot(BEFORE_CARGO, BEFORE_BUCK),
            &snapshot(AFTER_CARGO, after_buck),
            &os_plan(),
            &[],
        );
        assert!(!out.equivalent);
        assert!(
            out.only_before
                .iter()
                .any(|s| s.contains("os-apid-domain-unittest")),
            "expected the dropped unittest target to be named; got {:?}",
            out.only_before
        );
    }

    #[test]
    fn an_invented_target_breaks_equivalence() {
        let after_buck = format!("{AFTER_BUCK}\nroot//os/core/apid-domain:sneaky-extra");
        let out = prove_graph_equivalence(
            &snapshot(BEFORE_CARGO, BEFORE_BUCK),
            &snapshot(AFTER_CARGO, &after_buck),
            &os_plan(),
            &[],
        );
        assert!(!out.equivalent);
        assert!(out.only_after.iter().any(|s| s.contains("sneaky-extra")));
    }

    #[test]
    fn a_rewired_dependency_edge_breaks_equivalence() {
        // Same target set, but a referrer lost its edge — invisible to a label-only comparison.
        let after_cargo = r#"{"packages":[
            {"name":"os-apid-domain","dependencies":[{"name":"serde"}]},
            {"name":"iam-pdp","dependencies":[]}]}"#;
        let out = prove_graph_equivalence(
            &snapshot(BEFORE_CARGO, BEFORE_BUCK),
            &snapshot(after_cargo, AFTER_BUCK),
            &os_plan(),
            &[],
        );
        assert!(!out.equivalent, "a dropped dep edge must break equivalence");
        assert!(out.only_before.iter().any(|s| s.starts_with("cargo:iam-pdp")));
    }

    #[test]
    fn missing_buck2_fails_closed_and_is_never_equivalent() {
        let mut before = snapshot(BEFORE_CARGO, BEFORE_BUCK);
        let mut after = snapshot(AFTER_CARGO, AFTER_BUCK);
        before.buck_available = false;
        after.buck_available = false;
        let out = prove_graph_equivalence(&before, &after, &os_plan(), &[]);
        assert!(
            !out.equivalent,
            "cargo-only agreement must NOT be reported as a graph proof"
        );
        assert!(!out.buck_checked);
        assert!(out.detail.contains("buck2"));
    }

    #[test]
    fn a_declared_difference_is_excluded_but_an_undeclared_one_still_fails() {
        // The codemod deliberately never renames a `-bin` sibling, so that label legitimately
        // differs across a move. Declaring it must clear it — and must NOT clear anything else.
        let after_buck = "\
root//os/core/apid-domain:os-apid-domain
root//os/core/apid-domain:os-apid-domain-unittest
root//os/core/apid-domain:oya-cloud-os-apid-domain-bin
root//iam/facade/pdp:iam-pdp";
        let before_buck = format!("{BEFORE_BUCK}\nroot//cloud/cloud-os/crates/oya-cloud-os-apid-domain:oya-cloud-os-apid-domain-bin");

        let undeclared = prove_graph_equivalence(
            &snapshot(BEFORE_CARGO, &before_buck),
            &snapshot(AFTER_CARGO, after_buck),
            &os_plan(),
            &[],
        );
        assert!(!undeclared.equivalent, "undeclared bin rename must fail");

        let declared = prove_graph_equivalence(
            &snapshot(BEFORE_CARGO, &before_buck),
            &snapshot(AFTER_CARGO, after_buck),
            &os_plan(),
            // BOTH sides must be declared: the expected (relabelled) label AND the actual one.
            // Declaring only one leaves the other reported, which is the honest behaviour — a
            // retained name is two facts, not one.
            &[
                "os-apid-domain-bin".to_string(),
                "oya-cloud-os-apid-domain-bin".to_string(),
            ],
        );
        assert!(
            declared.equivalent,
            "declaring the retained bin name should clear it; detail={}",
            declared.detail
        );
        assert!(declared.detail.contains("declared difference"));

        // A declaration must not launder an unrelated difference.
        let with_extra = format!("{after_buck}\nroot//os/core/apid-domain:sneaky");
        let still_fails = prove_graph_equivalence(
            &snapshot(BEFORE_CARGO, &before_buck),
            &snapshot(AFTER_CARGO, &with_extra),
            &os_plan(),
            &[
                "os-apid-domain-bin".to_string(),
                "oya-cloud-os-apid-domain-bin".to_string(),
            ],
        );
        assert!(!still_fails.equivalent);
        assert!(still_fails.only_after.iter().any(|s| s.contains("sneaky")));
    }

    #[test]
    fn two_empty_graphs_are_not_a_proof() {
        // Two empty sets are equal. Without a guard that reads as "equivalent" while having
        // learned nothing — the exact false green this tool exists to avoid, because a buck2
        // that exits 0 on a broken cell emits no targets at all.
        let empty_cargo = r#"{"packages":[]}"#;
        let out = prove_graph_equivalence(
            &snapshot(empty_cargo, ""),
            &snapshot(empty_cargo, ""),
            &os_plan(),
            &[],
        );
        assert!(
            !out.equivalent,
            "an empty graph must never prove equivalence; detail={}",
            out.detail
        );
        assert!(!out.cargo_checked && !out.buck_checked);
        assert!(out.detail.contains("EMPTY"));
    }

    #[test]
    fn a_truncated_graph_that_omits_the_moved_crates_is_not_a_proof() {
        // Both sides agree, both are non-empty — but neither mentions the crate the plan claims
        // to have moved. Agreement about unrelated targets is not evidence about this move.
        let unrelated_cargo = r#"{"packages":[{"name":"iam-pdp","dependencies":[]}]}"#;
        let unrelated_buck = "root//iam/facade/pdp:iam-pdp";
        let out = prove_graph_equivalence(
            &snapshot(unrelated_cargo, unrelated_buck),
            &snapshot(unrelated_cargo, unrelated_buck),
            &os_plan(),
            &[],
        );
        assert!(
            !out.equivalent,
            "a graph that cannot see the moved crate proves nothing; detail={}",
            out.detail
        );
        assert!(!out.cargo_checked, "cargo leg must reject: destination crate absent");
        assert!(!out.buck_checked, "buck leg must reject: destination package owns no target");
        assert!(out.detail.contains("absent") || out.detail.contains("no target"));
    }

    #[test]
    fn remap_is_longest_match_first_and_respects_separators() {
        let pairs = vec![
            ("a".to_string(), "X".to_string()),
            ("a/b".to_string(), "Y".to_string()),
        ];
        // longest wins, so a/b/c must not become X/b/c
        assert_eq!(remap_prefix("a/b/c", &pairs, '/'), "Y/c");
        assert_eq!(remap_prefix("a/z", &pairs, '/'), "X/z");
        // a separator boundary is required: `ab` is not under `a`
        assert_eq!(remap_prefix("ab", &pairs, '/'), "ab");
        assert_eq!(remap_prefix("a", &pairs, '/'), "X");
    }
}
