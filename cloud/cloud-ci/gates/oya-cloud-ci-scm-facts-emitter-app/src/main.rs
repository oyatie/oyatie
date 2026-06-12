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
//!   oya-cloud-ci-scm-facts-emitter-app [--repo-root <path>] [--out <path>] [--merge-base-baseline]
//!
//! Default `--repo-root` is discovered up-tree (the dir holding `specs/root-hub-pointers.json`),
//! default `--out` is `<repo-root>/cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json`.
//!
//! With `--merge-base-baseline` the emitter ALSO materializes the firewall's frozen
//! reference (ADR-0551, fixes FRIC-1781112000): it reads `ratchet-policy.json` (DATA: the
//! configurable `base_ref` + face/out paths), computes `git merge-base <base_ref> HEAD`,
//! extracts the gate-baseline face as committed at that revision, and writes the
//! provenance-wrapped snapshot to the policy `out_path` (untracked + gitignored). This
//! lives HERE because the emitter is the single out-of-graph git boundary — the firewall
//! gate itself never calls git. FAIL-CLOSED: an unresolvable base_ref or merge-base is a
//! hard error; only a face genuinely absent at the merge-base (repo bootstrap) produces a
//! `missing_at_merge_base` snapshot with an EMPTY frozen reference (everything is growth
//! until signed off).
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

/// The committed merge-base ratchet policy (DATA) the `--merge-base-baseline` mode reads.
const RATCHET_POLICY_PATH: &str = "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/ratchet-policy.json";

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repo_root: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut merge_base_baseline = false;

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
            "--merge-base-baseline" => {
                merge_base_baseline = true;
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

    if merge_base_baseline {
        emit_merge_base_baseline(&repo_root)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Merge-base frozen-baseline snapshot (ADR-0551, FRIC-1781112000)
// ---------------------------------------------------------------------------

/// The parsed `ratchet-policy.json` (the configurable comparison root, R0 policy-as-data).
struct RatchetPolicy {
    base_ref: String,
    face_path: String,
    out_path: String,
}

/// Parse + validate the ratchet policy. Fail-closed: every field is required and non-empty
/// — a missing/garbled policy must never silently disable the frozen reference.
fn parse_ratchet_policy(text: &str) -> Result<RatchetPolicy, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("ratchet-policy parse: {e}"))?;
    let field = |path: &[&str]| -> Result<String, String> {
        let mut cursor = &value;
        for key in path {
            cursor = cursor
                .get(key)
                .ok_or_else(|| format!("ratchet-policy missing {}", path.join(".")))?;
        }
        let s = cursor
            .as_str()
            .ok_or_else(|| format!("ratchet-policy {} is not a string", path.join(".")))?;
        if s.trim().is_empty() {
            return Err(format!("ratchet-policy {} is empty", path.join(".")));
        }
        Ok(s.to_owned())
    };
    Ok(RatchetPolicy {
        base_ref: field(&["base_ref"])?,
        face_path: field(&["frozen_reference", "face_path"])?,
        out_path: field(&["frozen_reference", "out_path"])?,
    })
}

/// Assemble the provenance-wrapped snapshot the firewall parses (`FrozenBaseline`).
/// `face` is the gate-baseline face content at the merge-base, or `None` when the face
/// does not exist there (repo bootstrap): the frozen reference is then EMPTY and declared
/// as such, so every proposed key is growth until signed off — fail-closed, never
/// fail-open.
fn build_merge_base_baseline_snapshot(
    policy: &RatchetPolicy,
    merge_base: &str,
    face: Option<serde_json::Value>,
) -> serde_json::Value {
    let missing = face.is_none();
    json!({
        "schema": "oya-ci/merge-base-baseline/v1",
        "_comment": "GENERATED out-of-graph by oya-cloud-ci-scm-facts-emitter-app --merge-base-baseline (ADR-0551). The firewall's FROZEN reference: the gate-baseline face exactly as committed at `git merge-base <base_ref> HEAD`. Untracked + gitignored — it varies with the base branch position and is rematerialized by CI before gates consume it; it is NEVER a merge surface.",
        "base_ref": policy.base_ref,
        "merge_base": merge_base,
        "face_path": policy.face_path,
        "missing_at_merge_base": missing,
        "baseline": face.unwrap_or_else(|| json!({"gates": {}})),
    })
}

/// Materialize the frozen reference: policy -> merge-base -> face-at-revision -> snapshot.
fn emit_merge_base_baseline(repo_root: &Path) -> Result<(), String> {
    let policy_path = repo_root.join(RATCHET_POLICY_PATH);
    let policy_text = std::fs::read_to_string(&policy_path)
        .map_err(|e| format!("{}: {e}", policy_path.display()))?;
    let policy = parse_ratchet_policy(&policy_text)?;

    let merge_base = git_merge_base(repo_root, &policy.base_ref)?;
    let face = match git_show_file(repo_root, &merge_base, &policy.face_path)? {
        Some(text) => Some(
            serde_json::from_str(&text)
                .map_err(|e| format!("{}@{merge_base} parse: {e}", policy.face_path))?,
        ),
        None => None,
    };
    let missing = face.is_none();
    let snapshot = build_merge_base_baseline_snapshot(&policy, &merge_base, face);

    let out = repo_root.join(&policy.out_path);
    let text = to_canonical_json(&snapshot).map_err(|e| format!("serialize snapshot: {e}"))?;
    std::fs::write(&out, &text).map_err(|e| format!("{}: {e}", out.display()))?;
    eprintln!(
        "oya-cloud-ci-scm-facts-emitter-app: frozen baseline {} @ merge-base {merge_base}{} -> {}",
        policy.base_ref,
        if missing { " (face missing at merge-base: EMPTY frozen reference)" } else { "" },
        out.display()
    );
    Ok(())
}

/// `git merge-base <base_ref> HEAD` — the frozen comparison root. A failure (unknown ref,
/// shallow history, detached state without HEAD) is a HARD error: the ratchet must never
/// silently fall back to a PR-controlled reference.
fn git_merge_base(repo_root: &Path, base_ref: &str) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["merge-base", base_ref, "HEAD"])
        .output()
        .map_err(|e| format!("merge-base: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git merge-base {base_ref} HEAD failed (exit {:?}): {} — the frozen ratchet \
             reference REQUIRES the base ref; fetch it or repoint ratchet-policy.json base_ref",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if sha.len() < 40 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("git merge-base produced a non-revision: {sha:?}"));
    }
    Ok(sha)
}

/// `git show <revision>:<path>` with existence distinguished from failure: `Ok(None)` iff
/// the path does not exist at the revision (checked via `git cat-file -e`), `Err` for any
/// other git failure (fail-closed).
fn git_show_file(
    repo_root: &Path,
    revision: &str,
    path: &str,
) -> Result<Option<String>, String> {
    let spec = format!("{revision}:{path}");
    let exists = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["cat-file", "-e", &spec])
        .output()
        .map_err(|e| format!("cat-file: {e}"))?;
    if !exists.status.success() {
        return Ok(None);
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["show", &spec])
        .output()
        .map_err(|e| format!("show: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git show {spec} failed (exit {:?}): {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map(Some).map_err(|e| format!("show {spec}: {e}"))
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

    const POLICY_TEXT: &str = r#"{
        "base_ref": "origin/dev",
        "frozen_reference": {
            "face_path": "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/gate-baseline.generated.json",
            "out_path": "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.merge-base.generated.json"
        }
    }"#;

    #[test]
    fn ratchet_policy_parses_and_requires_every_field() {
        let policy = parse_ratchet_policy(POLICY_TEXT).unwrap();
        assert_eq!(policy.base_ref, "origin/dev");
        assert!(policy.face_path.ends_with("gate-baseline.generated.json"));
        assert!(policy.out_path.ends_with("gate-baseline.merge-base.generated.json"));

        // Fail-closed: a policy missing the comparison root must be a hard error, never a
        // silently-disabled frozen reference.
        assert!(parse_ratchet_policy("{}").is_err());
        assert!(parse_ratchet_policy(r#"{"base_ref": ""}"#).is_err());
        assert!(
            parse_ratchet_policy(r#"{"base_ref": "origin/dev", "frozen_reference": {}}"#)
                .is_err()
        );
    }

    #[test]
    fn merge_base_baseline_snapshot_wraps_face_with_provenance() {
        let policy = parse_ratchet_policy(POLICY_TEXT).unwrap();
        let face = json!({"gates": {"g": {"c": {"mode": "baseline-block-on-new", "keys": ["k"]}}}});
        let snapshot = build_merge_base_baseline_snapshot(
            &policy,
            "d5d8be5d4121e91655d7ba361f63271c98c57a68",
            Some(face.clone()),
        );
        assert_eq!(snapshot["schema"], "oya-ci/merge-base-baseline/v1");
        assert_eq!(snapshot["base_ref"], "origin/dev");
        assert_eq!(
            snapshot["merge_base"],
            "d5d8be5d4121e91655d7ba361f63271c98c57a68"
        );
        assert_eq!(snapshot["missing_at_merge_base"], false);
        assert_eq!(snapshot["baseline"], face);
    }

    #[test]
    fn merge_base_baseline_snapshot_declares_bootstrap_emptiness() {
        // A face absent at the merge-base (repo bootstrap) must yield a DECLARED-empty
        // frozen reference: everything is growth until signed off (fail-closed).
        let policy = parse_ratchet_policy(POLICY_TEXT).unwrap();
        let snapshot = build_merge_base_baseline_snapshot(
            &policy,
            "d5d8be5d4121e91655d7ba361f63271c98c57a68",
            None,
        );
        assert_eq!(snapshot["missing_at_merge_base"], true);
        assert_eq!(snapshot["baseline"], json!({"gates": {}}));
    }
}
