//! scm-facts emitter — the SINGLE out-of-graph git boundary (ADR-0515 D3 narrow exception).
//!
//! This is the ONE place in the oya-ci pipeline that is allowed to shell out to `git`. It runs
//! OUTSIDE the buck2 action graph (a CI pre-step + a local regen hook), snapshots the four
//! ambient git outputs the accounting producer used to derive itself, and writes them as the
//! committed, content-addressed, registry-drift-protected `scm-facts.generated.json` face. The
//! producer and every gate `rust_test` then consume that committed face as a DECLARED input, so
//! no buck2 action ever calls git (OYA-CI-HERMETIC-EXECUTION-DESIGN §1.5, Option C).
//!
//! The four git calls below are moved VERBATIM out of the producer's old `git_*` helpers so the
//! facts are bit-for-bit what the live git calls produced — the make-or-break byte-parity
//! guarantee: a producer reading this face regenerates the six faces byte-identically.
//!
//! Usage:
//!   oya-cloud-ci-scm-facts-emitter-app [--repo-root <path>] [--out <path>]
//!
//! Default `--repo-root` is discovered up-tree (the dir holding `specs/root-hub-pointers.json`),
//! default `--out` is `<repo-root>/cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_accounting_registry_app::to_canonical_json;
use serde_json::json;

/// The scm-facts face schema id — bumped only on a breaking shape change.
const SCHEMA: &str = "oya-ci/scm-facts/v1";

fn main() {
    if let Err(error) = run() {
        eprintln!("oya-cloud-ci-scm-facts-emitter-app: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repo_root: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                i += 1;
                repo_root = args.get(i).map(PathBuf::from);
            }
            "--out" => {
                i += 1;
                out = args.get(i).map(PathBuf::from);
            }
            other => return Err(format!("unknown argument {other}")),
        }
        i += 1;
    }

    let repo_root = match repo_root {
        Some(root) => root,
        None => discover_repo_root()?,
    };
    let out = out.unwrap_or_else(|| {
        repo_root.join(
            "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json",
        )
    });

    let source = GitCliScmFactsSource::new(repo_root.clone());
    let emission = emit_scm_facts(&source)?;

    // Build the face as a serde_json Value with BTreeMap-backed maps so the on-disk key order
    // is the canonical sorted order, then serialize through the producer's exact canonicalizer
    // (to_string_pretty + trailing newline).
    let text = to_canonical_json(&emission.value).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(&out, &text).map_err(|e| format!("{}: {e}", out.display()))?;
    eprintln!(
        "oya-cloud-ci-scm-facts-emitter-app: {} tracked paths -> {}",
        emission.tracked_paths_len,
        out.display()
    );
    Ok(())
}

/// Stable seam for the scm-facts source. Git CLI is transitional implementation #1;
/// a future bespoke SCM source should implement these same three primitives without
/// changing the emitted v1 facts shape or producer/gate consumers.
trait ScmFactsSource {
    /// The tracked path universe, sorted and deduplicated by the implementation.
    fn tracked_paths(&self) -> Result<Vec<String>, String>;

    /// Path -> last-touch revision id, with generated-class paths excluded.
    fn last_touch(&self) -> Result<BTreeMap<String, String>, String>;

    /// Revision id -> author timestamp (epoch secs).
    fn revision_author_timestamps(&self) -> Result<BTreeMap<String, u64>, String>;
}

struct GitCliScmFactsSource {
    repo_root: PathBuf,
}

impl GitCliScmFactsSource {
    fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

impl ScmFactsSource for GitCliScmFactsSource {
    fn tracked_paths(&self) -> Result<Vec<String>, String> {
        git_ls_files(&self.repo_root)
    }

    fn last_touch(&self) -> Result<BTreeMap<String, String>, String> {
        git_last_touch(&self.repo_root)
    }

    fn revision_author_timestamps(&self) -> Result<BTreeMap<String, u64>, String> {
        git_commit_timestamps(&self.repo_root)
    }
}

struct ScmFactsEmission {
    value: serde_json::Value,
    tracked_paths_len: usize,
}

fn emit_scm_facts(source: &impl ScmFactsSource) -> Result<ScmFactsEmission, String> {
    let tracked_paths = source.tracked_paths()?;
    let tracked_paths_len = tracked_paths.len();
    let tracked_path_set: std::collections::BTreeSet<&String> = tracked_paths.iter().collect();
    let last_touch_commit: BTreeMap<String, String> = source
        .last_touch()?
        .into_iter()
        .filter(|(path, _)| tracked_path_set.contains(path))
        .collect();
    let all_commit_ts = source.revision_author_timestamps()?;

    // CONVERGENCE: scm-facts must be a pure function of the COMMITTED TREE STATE, never of which
    // commit is currently HEAD — otherwise every commit (which advances HEAD and appends a new
    // commit timestamp) would churn scm-facts forever (no fixpoint). So:
    //   - `commit_author_ts_secs` keeps ONLY the timestamps of the commits that are some path's
    //     last-touch (the only ones the producer's staleness aging ever looks up). The unused
    //     ~half of full history is dropped — it served no face and only tracked HEAD growth.
    //   - `head_time_secs` (the deterministic "now" for aging) is the MAX last-touch timestamp,
    //     a tree-content function equal to the live HEAD time whenever HEAD is itself a
    //     last-touch, and STABLE across HEAD-only-advancing commits (e.g. a faces settle that
    //     touches only generated-class files). This reproduces today's faces byte-for-byte.
    //   - `head_commit` is dropped: the producer never reads it and it tracks the moving HEAD.
    // last_touch is constrained to the current tracked-path universe and excludes generated-class
    // paths (see `git_last_touch`), so deleted/renamed historical paths cannot leak through the
    // stable scm-facts contract and the re-run is byte-identical.
    let last_touch_shas: std::collections::BTreeSet<&String> = last_touch_commit.values().collect();
    let commit_author_ts_secs: BTreeMap<String, u64> = all_commit_ts
        .iter()
        .filter(|(sha, _)| last_touch_shas.contains(sha))
        .map(|(sha, ts)| (sha.clone(), *ts))
        .collect();
    let head_time_secs = commit_author_ts_secs.values().copied().max().unwrap_or(0);

    let value = json!({
        "schema": SCHEMA,
        "head_time_secs": head_time_secs,
        "tracked_paths": tracked_paths,
        "last_touch_commit": last_touch_commit,
        "commit_author_ts_secs": commit_author_ts_secs,
    });
    Ok(ScmFactsEmission {
        value,
        tracked_paths_len,
    })
}

/// Walk up from cwd to the repo root (the dir holding `specs/root-hub-pointers.json`).
fn discover_repo_root() -> Result<PathBuf, String> {
    let mut dir = std::env::current_dir().map_err(|e| e.to_string())?;
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err("failed to locate repo root (no specs/root-hub-pointers.json up-tree)".to_owned())
}

/// One `git log --format` pass builds the commit-sha -> author-timestamp map (epoch secs).
/// Moved VERBATIM from the producer's old `git_commit_timestamps`. The caller filters this to
/// the last-touch commits and derives the deterministic "now" (max last-touch ts) from it, so
/// scm-facts never depends on the moving HEAD (the producer's old `git_head_secs` HEAD-time read
/// is replaced by that tree-content max — it equals the HEAD time whenever HEAD is a last-touch
/// and stays stable across HEAD-only-advancing commits, preserving the faces byte-for-byte).
fn git_commit_timestamps(repo_root: &Path) -> Result<BTreeMap<String, u64>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["log", "--format=%H %ct"])
        .output()
        .map_err(|e| format!("log timestamps: {e}"))?;
    if !output.status.success() {
        return Err(format!("log timestamps exit {:?}", output.status.code()));
    }
    let mut map = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some((sha, ts)) = line.split_once(' ')
            && let Ok(ts) = ts.trim().parse::<u64>()
        {
            map.insert(sha.to_owned(), ts);
        }
    }
    Ok(map)
}

/// The tracked-paths universe. Moved VERBATIM from the producer's old `git_ls_files`.
fn git_ls_files(repo_root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("ls-files")
        .output()
        .map_err(|e| format!("ls-files: {e}"))?;
    if !output.status.success() {
        return Err(format!("ls-files exit {:?}", output.status.code()));
    }
    let mut paths: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .filter(|p| !p.is_empty())
        .collect();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// True iff `path` is a `generated`-class file under the accounting producer's unit-class
/// policy (`unit-class-policy.json`: suffix `.generated.json`, suffix `Cargo.lock`, prefix
/// `docs/machine-readable/`). The producer ALWAYS sets `last_touch_commit = None` for these
/// (lib.rs: "generated class so the face is invariant to which commit holds it"), so their git
/// last-touch is dead data the producer never reads. Including it in scm-facts would make
/// scm-facts NON-CONVERGENT: every faces settle re-touches the `.generated.json` faces (and a
/// dependency bump re-touches `Cargo.lock`), churning their last-touch and forcing another
/// settle ad infinitum. Excluding them here mirrors the producer's null-out EXACTLY (a missing
/// key reads back as None, identical to the producer's explicit None) — the produced faces are
/// byte-identical, and scm-facts converges in the standard 2-commit settle.
fn is_generated_class(path: &str) -> bool {
    path.ends_with(".generated.json")
        || path.ends_with("Cargo.lock")
        || path.starts_with("docs/machine-readable/")
}

/// One `git log --name-only` pass builds the path -> last-touch-commit map for the whole tree.
/// The git walk is moved VERBATIM from the producer's old `git_last_touch`; the only addition is
/// skipping `generated`-class paths (see `is_generated_class`) so scm-facts is convergent without
/// altering any produced face.
fn git_last_touch(repo_root: &Path) -> Result<BTreeMap<String, String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["log", "--name-only", "--format=commit:%H"])
        .output()
        .map_err(|e| format!("log: {e}"))?;
    if !output.status.success() {
        return Err(format!("log exit {:?}", output.status.code()));
    }
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(sha) = line.strip_prefix("commit:") {
            current = Some(sha.to_owned());
        } else if !line.is_empty()
            && !is_generated_class(line)
            && let Some(sha) = &current
        {
            // first time we see a path (walking newest-first) is its last touch
            map.entry(line.to_owned()).or_insert_with(|| sha.clone());
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeScmFactsSource {
        tracked_paths: Vec<String>,
        last_touch: BTreeMap<String, String>,
        revision_author_timestamps: BTreeMap<String, u64>,
    }

    impl ScmFactsSource for FakeScmFactsSource {
        fn tracked_paths(&self) -> Result<Vec<String>, String> {
            Ok(self.tracked_paths.clone())
        }

        fn last_touch(&self) -> Result<BTreeMap<String, String>, String> {
            Ok(self.last_touch.clone())
        }

        fn revision_author_timestamps(&self) -> Result<BTreeMap<String, u64>, String> {
            Ok(self.revision_author_timestamps.clone())
        }
    }

    #[test]
    fn emit_scm_facts_uses_scm_source_primitives_without_behavior_change() {
        let source = FakeScmFactsSource {
            tracked_paths: vec!["a.txt".to_owned(), "b.txt".to_owned()],
            last_touch: BTreeMap::from([
                ("a.txt".to_owned(), "rev-a".to_owned()),
                ("b.txt".to_owned(), "rev-b".to_owned()),
                (
                    "deleted-old-boundary.txt".to_owned(),
                    "rev-deleted".to_owned(),
                ),
            ]),
            revision_author_timestamps: BTreeMap::from([
                ("rev-a".to_owned(), 10),
                ("rev-b".to_owned(), 20),
                ("unused-head".to_owned(), 30),
            ]),
        };

        let emission = emit_scm_facts(&source).unwrap();

        assert_eq!(emission.tracked_paths_len, 2);
        assert_eq!(
            emission.value,
            json!({
                "schema": SCHEMA,
                "head_time_secs": 20,
                "tracked_paths": ["a.txt", "b.txt"],
                "last_touch_commit": {
                    "a.txt": "rev-a",
                    "b.txt": "rev-b",
                },
                "commit_author_ts_secs": {
                    "rev-a": 10,
                    "rev-b": 20,
                },
            })
        );
    }

    #[test]
    fn generated_class_filter_matches_existing_policy() {
        assert!(is_generated_class(
            "cloud/cloud-ci/gates/app/foo.generated.json"
        ));
        assert!(is_generated_class("Cargo.lock"));
        assert!(is_generated_class("docs/machine-readable/catalog.json"));
        assert!(!is_generated_class("cloud/cloud-ci/gates/app/src/main.rs"));
    }
}
