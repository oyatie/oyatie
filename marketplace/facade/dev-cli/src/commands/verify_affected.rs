//! Affected-target scope computation for `oya verify --affected` (ADR-0360 O1).
//!
//! Presubmit affected-target selection: classify a changed-file set into the
//! minimal cargo scope that must be re-checked, mirroring Google TAP / Bazel
//! `rdeps` but sourced from `cargo metadata`. `--ci-required` remains the
//! authoritative whole-workspace mirror (the trunk backstop), so this mode can
//! only ever NARROW presubmit work — never under-test trunk.
//!
//! Correctness rules (ADR-0360 O1, grounded in best-practice research):
//! - **Full** triggers force the whole workspace: `Cargo.lock`, root/workspace
//!   `Cargo.toml`, a `workspace-hack` manifest, `rust-toolchain*`, `.cargo/`
//!   config, CI config, the `oya-dev-cli` gate engine itself, and any `build.rs`
//!   (a lockfile/feature/proc-macro/build-script change can silently alter what
//!   compiles, so file-path mapping is unsafe — run everything).
//! - **NoRust**: no Rust-relevant file changed (docs/`.md`/YAML/JSON/evidence/
//!   specs) ⇒ skip the cargo mirror; gates still run.
//! - **Crates**: otherwise, the changed crates ∪ their transitive reverse-
//!   dependency closure (dev + build edges included).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;
use std::process::Command;

/// The cargo scope a presubmit run must cover.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BuildScope {
    /// Run the whole-workspace cargo mirror (a full-build trigger fired).
    Full,
    /// No Rust-relevant change — skip cargo; run gates only.
    NoRust,
    /// Run cargo scoped to these crates (already reverse-dependency-closed).
    Crates(Vec<String>),
}

/// Reverse-dependency graph: `Rdeps[c]` = the crates that depend on `c`
/// (including dev + build edges).
pub(crate) type Rdeps = BTreeMap<String, BTreeSet<String>>;

/// A workspace member: package name + its manifest directory (repo-relative,
/// forward-slashed, no trailing slash), e.g. `("oya-foo", "crates/oya-foo")`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Member {
    pub name: String,
    pub dir: String,
}

/// True if `file` (repo-relative, forward-slashed) forces a whole-workspace run.
pub(crate) fn is_full_build_trigger(file: &str) -> bool {
    let base = file.rsplit('/').next().unwrap_or(file);
    file == "Cargo.lock"
        || file == "Cargo.toml" // workspace root manifest
        || base == "build.rs"
        || base == "rust-toolchain"
        || base == "rust-toolchain.toml"
        || file.starts_with(".cargo/")
        || file.contains("workspace-hack")
        // CI config + the gate engine itself: changing how we build/verify must
        // re-run everything.
        || file.starts_with(".github/workflows/")
        || base == "Jenkinsfile"
        || file.starts_with("infra/ci/jenkins/")
        || file.starts_with("crates/oya-dev-cli/")
        || file == "bin/oya"
        || file.starts_with("scripts/")
}

/// True if `file` is Rust-relevant (could change compiled/tested output).
fn is_rust_relevant(file: &str) -> bool {
    file.ends_with(".rs") || file.ends_with("/Cargo.toml")
}

/// Find the workspace member that owns `file` (the member whose `dir` is the
/// longest path-prefix of `file`). Returns the member name, if any.
fn owning_member<'a>(file: &str, members: &'a [Member]) -> Option<&'a str> {
    members
        .iter()
        .filter(|m| {
            file == m.dir
                || file
                    .strip_prefix(&m.dir)
                    .is_some_and(|rest| rest.starts_with('/'))
        })
        .max_by_key(|m| m.dir.len())
        .map(|m| m.name.as_str())
}

/// Transitive reverse-dependency closure of `seed` over `rdeps`
/// (`rdeps[c]` = crates that depend on `c`). Includes the seed.
fn rdeps_closure(seed: &BTreeSet<String>, rdeps: &Rdeps) -> BTreeSet<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut queue: VecDeque<String> = seed.iter().cloned().collect();
    while let Some(c) = queue.pop_front() {
        if !seen.insert(c.clone()) {
            continue;
        }
        if let Some(dependents) = rdeps.get(&c) {
            for d in dependents {
                if !seen.contains(d) {
                    queue.push_back(d.clone());
                }
            }
        }
    }
    seen
}

/// Pure classifier: compute the [`BuildScope`] for a changed-file set.
///
/// `changed` are repo-relative, forward-slashed paths. `members` is the
/// workspace member list. `rdeps` maps each crate to the crates that depend on
/// it (already including dev + build edges).
pub(crate) fn classify(changed: &[String], members: &[Member], rdeps: &Rdeps) -> BuildScope {
    if changed.iter().any(|f| is_full_build_trigger(f)) {
        return BuildScope::Full;
    }
    let mut seed: BTreeSet<String> = BTreeSet::new();
    for f in changed {
        if is_rust_relevant(f) {
            if let Some(name) = owning_member(f, members) {
                seed.insert(name.to_string());
            } else {
                // Rust-relevant change we cannot attribute to a member — be safe.
                return BuildScope::Full;
            }
        }
    }
    if seed.is_empty() {
        return BuildScope::NoRust;
    }
    let closure = rdeps_closure(&seed, rdeps);
    BuildScope::Crates(closure.into_iter().collect())
}

/// Repo-relative, forward-slashed paths changed vs `base` (tracked diff +
/// working tree + untracked), so local and CI runs agree.
pub(crate) fn changed_files(repo_root: &Path, base: &str) -> Result<Vec<String>, String> {
    let merge_base = run_git(repo_root, &["merge-base", "HEAD", base])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| base.to_string());
    let mut files: BTreeSet<String> = BTreeSet::new();
    // committed + uncommitted tracked changes vs the merge base
    for line in run_git(repo_root, &["diff", "--name-only", &merge_base])?.lines() {
        let f = line.trim();
        if !f.is_empty() {
            files.insert(f.replace('\\', "/"));
        }
    }
    // untracked files (new, not yet added)
    if let Ok(out) = run_git(repo_root, &["ls-files", "--others", "--exclude-standard"]) {
        for line in out.lines() {
            let f = line.trim();
            if !f.is_empty() {
                files.insert(f.replace('\\', "/"));
            }
        }
    }
    Ok(files.into_iter().collect())
}

/// Workspace members + the reverse-dependency graph (`rdeps[c]` = crates that
/// depend on `c`, including dev + build edges), from `cargo metadata --no-deps`.
pub(crate) fn workspace_graph(repo_root: &Path) -> Result<(Vec<Member>, Rdeps), String> {
    let out = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("cargo metadata failed to launch: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let json: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("cargo metadata: invalid json: {e}"))?;
    let root = json
        .get("workspace_root")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .replace('\\', "/");
    let packages = json
        .get("packages")
        .and_then(|v| v.as_array())
        .ok_or("cargo metadata: no packages array")?;

    let member_names: BTreeSet<String> = packages
        .iter()
        .filter_map(|p| p.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect();

    let mut members: Vec<Member> = Vec::new();
    // forward[c] = workspace-member crates that c depends on
    let mut forward: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for p in packages {
        let name = match p.get("name").and_then(|n| n.as_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if let Some(mp) = p.get("manifest_path").and_then(|m| m.as_str()) {
            let mp = mp.replace('\\', "/");
            let dir = mp.strip_suffix("/Cargo.toml").unwrap_or(&mp);
            let rel = dir
                .strip_prefix(&format!("{root}/"))
                .unwrap_or(dir)
                .to_string();
            members.push(Member {
                name: name.clone(),
                dir: rel,
            });
        }
        let deps: BTreeSet<String> = p
            .get("dependencies")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| d.get("name").and_then(|n| n.as_str()))
                    .filter(|n| member_names.contains(*n))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        forward.insert(name, deps);
    }

    // invert forward -> rdeps
    let mut rdeps: Rdeps = BTreeMap::new();
    for (crate_name, deps) in &forward {
        for dep in deps {
            rdeps
                .entry(dep.clone())
                .or_default()
                .insert(crate_name.clone());
        }
    }
    Ok((members, rdeps))
}

fn run_git(repo_root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("git {args:?} failed to launch: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn members() -> Vec<Member> {
        vec![
            Member {
                name: "oya-foo".into(),
                dir: "crates/oya-foo".into(),
            },
            Member {
                name: "oya-bar".into(),
                dir: "crates/oya-bar".into(),
            },
            Member {
                name: "oya-baz".into(),
                dir: "crates/oya-baz".into(),
            },
        ]
    }

    // oya-bar depends on oya-foo; oya-baz depends on oya-bar.
    fn rdeps() -> BTreeMap<String, BTreeSet<String>> {
        let mut m: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        m.entry("oya-foo".into())
            .or_default()
            .insert("oya-bar".into());
        m.entry("oya-bar".into())
            .or_default()
            .insert("oya-baz".into());
        m
    }

    #[test]
    fn no_changes_is_no_rust() {
        assert_eq!(classify(&[], &members(), &rdeps()), BuildScope::NoRust);
    }

    #[test]
    fn docs_and_yaml_only_is_no_rust() {
        let changed = vec![
            "docs/adr-archive/ADR-0360-ci-pipeline-optimization-program.md".to_string(),
            "specs/masterplan.json".to_string(),
            "evidence/ci/x.txt".to_string(),
            "registry/catalog/oya-foo.yaml".to_string(),
        ];
        assert_eq!(classify(&changed, &members(), &rdeps()), BuildScope::NoRust);
    }

    #[test]
    fn cargo_lock_forces_full() {
        let changed = vec!["Cargo.lock".to_string(), "docs/x.md".to_string()];
        assert_eq!(classify(&changed, &members(), &rdeps()), BuildScope::Full);
    }

    #[test]
    fn root_manifest_and_build_script_force_full() {
        assert_eq!(
            classify(&["Cargo.toml".to_string()], &members(), &rdeps()),
            BuildScope::Full
        );
        assert_eq!(
            classify(
                &["crates/oya-foo/build.rs".to_string()],
                &members(),
                &rdeps()
            ),
            BuildScope::Full
        );
    }

    #[test]
    fn gate_engine_change_forces_full() {
        let changed = vec!["crates/oya-dev-cli/src/commands/verify.rs".to_string()];
        assert_eq!(classify(&changed, &members(), &rdeps()), BuildScope::Full);
    }

    #[test]
    fn ci_config_change_forces_full() {
        assert_eq!(
            classify(
                &["infra/ci/jenkins/values-local.yaml".to_string()],
                &members(),
                &rdeps()
            ),
            BuildScope::Full
        );
    }

    #[test]
    fn leaf_crate_change_selects_its_reverse_dep_closure() {
        // change oya-foo -> must also test oya-bar and oya-baz (transitive rdeps).
        let changed = vec!["crates/oya-foo/src/lib.rs".to_string()];
        assert_eq!(
            classify(&changed, &members(), &rdeps()),
            BuildScope::Crates(vec!["oya-bar".into(), "oya-baz".into(), "oya-foo".into()])
        );
    }

    #[test]
    fn top_crate_change_selects_only_itself() {
        let changed = vec!["crates/oya-baz/src/lib.rs".to_string()];
        assert_eq!(
            classify(&changed, &members(), &rdeps()),
            BuildScope::Crates(vec!["oya-baz".into()])
        );
    }

    #[test]
    fn crate_manifest_change_is_rust_relevant() {
        let changed = vec!["crates/oya-bar/Cargo.toml".to_string()];
        assert_eq!(
            classify(&changed, &members(), &rdeps()),
            BuildScope::Crates(vec!["oya-bar".into(), "oya-baz".into()])
        );
    }

    #[test]
    fn unattributable_rust_change_is_full() {
        // a .rs file not under any member dir — be safe, run full.
        let changed = vec!["tools/scratch/foo.rs".to_string()];
        assert_eq!(classify(&changed, &members(), &rdeps()), BuildScope::Full);
    }

    #[test]
    fn mixed_rust_and_docs_selects_crates() {
        let changed = vec![
            "crates/oya-baz/src/lib.rs".to_string(),
            "docs/x.md".to_string(),
        ];
        assert_eq!(
            classify(&changed, &members(), &rdeps()),
            BuildScope::Crates(vec!["oya-baz".into()])
        );
    }
}
