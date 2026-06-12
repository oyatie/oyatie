//! scm-facts emitter — the SINGLE out-of-graph git boundary (ADR-0515 D3 narrow exception).
//!
//! This is the ONE place in the oya-ci pipeline that is allowed to shell out to `git`. It runs
//! OUTSIDE the buck2 action graph (a CI pre-step + a local regen hook), snapshots the four
//! ambient git outputs the accounting producer used to derive itself, and writes them as the
//! committed, content-addressed, registry-drift-protected `scm-facts.generated.json` face. The
//! producer and every gate `rust_test` then consume that committed face as a DECLARED input, so
//! no buck2 action ever calls git (OYA-CI-HERMETIC-EXECUTION-DESIGN §1.5, Option C).
//!
//! The git calls below are moved VERBATIM out of the producer's old `git_*` helpers so the
//! facts are bit-for-bit what the live git calls produced — the make-or-break byte-parity
//! guarantee: a producer reading this face regenerates the six faces byte-identically.
//!
//! STABLE vs VOLATILE facts (ADR-0552, fixes FRIC-1781234047). The COMMITTED face
//! (`scm-facts.generated.json`, schema v2) carries ONLY tree-derived stable facts
//! (`tracked_paths`): a pure function of the committed TREE, so a squash-merge (which
//! rewrites commit ids but preserves the tree) can never un-settle it. The HISTORY-derived
//! volatile facts (per-path `last_touch_commit`, `commit_author_ts_secs`, the deterministic
//! `head_time_secs` aging anchor) move to the UNTRACKED, gitignored, CI-rematerialized
//! `scm-volatile-facts.generated.json` beside this crate — the same materialized-snapshot
//! pattern as the ADR-0551 merge-base baseline. Precedent: Bazel splits
//! `volatile-status.txt` from `stable-status.txt` so stamp data never invalidates hermetic
//! action keys. Volatile facts are NEVER a merge surface and NEVER byte-compared.
//!
//! Usage:
//!   oya-cloud-ci-scm-facts-emitter-app [--repo-root <path>] [--out <path>]
//!       [--volatile-out <path>] [--merge-base-baseline] [--frozen-base-ref <ref>]
//!
//! Default `--repo-root` is discovered up-tree (the dir holding `specs/root-hub-pointers.json`),
//! default `--out` is `<repo-root>/cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json`,
//! default `--volatile-out` is `<repo-root>/`[`VOLATILE_FACTS_PATH`].
//!
//! With `--merge-base-baseline` the emitter ALSO materializes the firewall's frozen
//! reference (ADR-0551, fixes FRIC-1781112000): it computes `git merge-base <bootstrap> HEAD`,
//! reads `ratchet-policy.json` AS COMMITTED AT THAT MERGE-BASE (frozen-policy-wins,
//! FRIC-1781280000), extracts the gate-baseline face at the same revision, and writes the
//! provenance-wrapped snapshot to the candidate policy's `out_path` (untracked + gitignored).
//! This lives HERE because the emitter is the single out-of-graph git boundary — the
//! firewall gate itself never calls git.
//!
//! FROZEN-POLICY-WINS (FRIC-1781280000): every policy fact that SELECTS the frozen
//! reference (`base_ref`, `face_path`) is read from the merge-base tree, never the
//! candidate tree. The bootstrap ref that locates the merge-base is OUT-OF-BAND
//! (`--frozen-base-ref` from the CI invocation, default [`DEFAULT_FROZEN_BOOTSTRAP_REF`]) —
//! it must NOT come from the candidate tree, because any candidate-supplied hint converges
//! to an attacker-chosen fixpoint: a same-PR `"base_ref": "HEAD"` edit makes
//! merge-base(HEAD, HEAD) = HEAD, the "frozen" policy/face become the PR's own settled
//! copies, and frozen == proposed (complete self-laundering — the PR #698 review MED
//! finding). The merge-base policy's `base_ref` must AGREE with the bootstrap (fail-closed
//! cross-check); repointing therefore changes only FUTURE behavior post-merge and requires
//! touching the out-of-band invocation too. Prow precedent: OWNERS are read from the base
//! branch, never the PR head. FAIL-CLOSED: an unresolvable bootstrap ref or merge-base is a
//! hard error; only a policy/face genuinely absent at the merge-base (repo bootstrap — the
//! PR introducing the ratchet) falls back to DECLARED candidate facts
//! (`frozen_policy_source: "candidate-bootstrap"` / `missing_at_merge_base: true`).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_accounting_registry_app::to_canonical_json;
use serde_json::json;

/// The scm-facts face schema id — bumped only on a breaking shape change.
/// v2 (ADR-0552): history-volatile fields (`last_touch_commit`, `commit_author_ts_secs`,
/// `head_time_secs`) left the committed face for the volatile-facts snapshot.
const SCHEMA: &str = "oya-ci/scm-facts/v2";

/// The volatile-facts snapshot schema id (history-derived facts; never committed).
const VOLATILE_SCHEMA: &str = "oya-ci/scm-volatile-facts/v1";

/// The canonical repo-relative path of the UNTRACKED, gitignored volatile-facts snapshot.
/// Consumers (the staleness gate) read it from here; CI rematerializes it before gates run.
const VOLATILE_FACTS_PATH: &str =
    "cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app/scm-volatile-facts.generated.json";

fn main() {
    if let Err(error) = run() {
        eprintln!("oya-cloud-ci-scm-facts-emitter-app: {error}");
        std::process::exit(1);
    }
}

/// The committed merge-base ratchet policy (DATA) the `--merge-base-baseline` mode reads —
/// the FROZEN copy at the merge-base selects the frozen reference (frozen-policy-wins,
/// FRIC-1781280000); the candidate copy supplies only the local `out_path`.
const RATCHET_POLICY_PATH: &str = "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/ratchet-policy.json";

/// The OUT-OF-BAND bootstrap ref that locates the merge-base (overridable per invocation
/// via `--frozen-base-ref`, the adopter's CI-config surface). Deliberately a compiled-in
/// constant, NEVER read from the candidate tree: a candidate-supplied hint converges to an
/// attacker-chosen fixpoint (`base_ref: "HEAD"` ⇒ merge-base = HEAD ⇒ frozen == proposed).
/// Changing it is a code/invocation edit — the same review class as editing the workflow.
const DEFAULT_FROZEN_BOOTSTRAP_REF: &str = "origin/dev";

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repo_root: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut volatile_out: Option<PathBuf> = None;
    let mut merge_base_baseline = false;
    let mut frozen_base_ref: Option<String> = None;

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
            "--volatile-out" => {
                i += 1;
                volatile_out = args.get(i).map(PathBuf::from);
            }
            "--merge-base-baseline" => {
                merge_base_baseline = true;
            }
            "--frozen-base-ref" => {
                i += 1;
                frozen_base_ref = args.get(i).cloned();
                if frozen_base_ref.as_deref().is_none_or(str::is_empty) {
                    return Err("--frozen-base-ref requires a ref".to_owned());
                }
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
    let volatile_out = volatile_out.unwrap_or_else(|| repo_root.join(VOLATILE_FACTS_PATH));

    let source = GitCliScmFactsSource::new(repo_root.clone());
    let emission = emit_scm_facts(&source)?;

    // Build the faces as serde_json Values with BTreeMap-backed maps so the on-disk key order
    // is the canonical sorted order, then serialize through the producer's exact canonicalizer
    // (to_string_pretty + trailing newline). The stable face is the committed merge surface;
    // the volatile snapshot is untracked + gitignored (ADR-0552) and never byte-compared.
    let text = to_canonical_json(&emission.value).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(&out, &text).map_err(|e| format!("{}: {e}", out.display()))?;
    let volatile_text =
        to_canonical_json(&emission.volatile).map_err(|e| format!("serialize volatile: {e}"))?;
    std::fs::write(&volatile_out, &volatile_text)
        .map_err(|e| format!("{}: {e}", volatile_out.display()))?;
    eprintln!(
        "oya-cloud-ci-scm-facts-emitter-app: {} tracked paths -> {} (volatile facts -> {})",
        emission.tracked_paths_len,
        out.display(),
        volatile_out.display()
    );

    if merge_base_baseline {
        let bootstrap_ref =
            frozen_base_ref.unwrap_or_else(|| DEFAULT_FROZEN_BOOTSTRAP_REF.to_owned());
        emit_merge_base_baseline(&repo_root, &bootstrap_ref)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Merge-base frozen-baseline snapshot (ADR-0551, FRIC-1781112000)
// ---------------------------------------------------------------------------

/// The parsed `ratchet-policy.json` (the configurable comparison root, R0 policy-as-data).
#[derive(Clone)]
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

/// The narrow git seam the frozen-reference resolution needs. Implemented by the live git
/// CLI ([`GitCliFrozenRefSource`]) and by a fake in tests — so the frozen-policy-wins
/// property (a candidate-tree policy edit can never select the PR's own frozen reference)
/// is pinned by an executable reproduction of the PR #698 review attack recipe.
trait FrozenRefSource {
    /// `git merge-base <base_ref> HEAD` (full hex revision id; hard error on failure).
    fn merge_base(&self, base_ref: &str) -> Result<String, String>;
    /// File content at `<revision>:<path>`; `Ok(None)` iff the path is absent there.
    fn show_file(&self, revision: &str, path: &str) -> Result<Option<String>, String>;
}

struct GitCliFrozenRefSource<'a> {
    repo_root: &'a Path,
}

impl FrozenRefSource for GitCliFrozenRefSource<'_> {
    fn merge_base(&self, base_ref: &str) -> Result<String, String> {
        git_merge_base(self.repo_root, base_ref)
    }

    fn show_file(&self, revision: &str, path: &str) -> Result<Option<String>, String> {
        git_show_file(self.repo_root, revision, path)
    }
}

/// `frozen_policy_source` value: policy facts read from the merge-base tree (normal path).
const FROZEN_POLICY_SOURCE_MERGE_BASE: &str = "merge-base";
/// `frozen_policy_source` value: policy absent at the merge-base (the PR introducing the
/// ratchet) — candidate facts used, DECLARED in the provenance.
const FROZEN_POLICY_SOURCE_CANDIDATE_BOOTSTRAP: &str = "candidate-bootstrap";

/// Assemble the provenance-wrapped snapshot the firewall parses (`FrozenBaseline`).
/// `face` is the gate-baseline face content at the merge-base, or `None` when the face
/// does not exist there (repo bootstrap): the frozen reference is then EMPTY and declared
/// as such, so every proposed key is growth until signed off — fail-closed, never
/// fail-open.
fn build_merge_base_baseline_snapshot(
    frozen_policy: &RatchetPolicy,
    frozen_policy_source: &str,
    merge_base: &str,
    face: Option<serde_json::Value>,
) -> serde_json::Value {
    let missing = face.is_none();
    json!({
        "schema": "oya-ci/merge-base-baseline/v2",
        "_comment": "GENERATED out-of-graph by oya-cloud-ci-scm-facts-emitter-app --merge-base-baseline (ADR-0551). The firewall's FROZEN reference: the gate-baseline face exactly as committed at `git merge-base <bootstrap> HEAD`, selected by the ratchet policy AS COMMITTED AT THAT MERGE-BASE (frozen-policy-wins, FRIC-1781280000 — a same-PR base_ref repoint cannot select this PR's own frozen reference). Untracked + gitignored — it varies with the base branch position and is rematerialized by CI before gates consume it; it is NEVER a merge surface.",
        "base_ref": frozen_policy.base_ref,
        "merge_base": merge_base,
        "face_path": frozen_policy.face_path,
        "frozen_policy_source": frozen_policy_source,
        "missing_at_merge_base": missing,
        "baseline": face.unwrap_or_else(|| json!({"gates": {}})),
    })
}

/// Resolve the FROZEN reference under frozen-policy-wins (FRIC-1781280000):
///
/// 1. `merge_base = git merge-base <bootstrap_ref> HEAD` — the bootstrap is OUT-OF-BAND
///    (CLI flag / compiled default), never a candidate-tree fact.
/// 2. The ratchet policy is read AT the merge-base; the candidate copy is used only when
///    the policy does not exist there (the PR introducing the ratchet — declared as
///    `frozen_policy_source: "candidate-bootstrap"`).
/// 3. The frozen policy's `base_ref` must AGREE with the bootstrap (fail-closed): a
///    divergence means the merged policy and the CI invocation no longer name the same
///    comparison root — repointing requires changing both, visibly.
/// 4. The frozen face is `face_path`-at-merge-base, with `face_path` taken from the FROZEN
///    policy.
fn resolve_merge_base_baseline_snapshot(
    source: &impl FrozenRefSource,
    candidate_policy: &RatchetPolicy,
    bootstrap_ref: &str,
) -> Result<serde_json::Value, String> {
    let merge_base = source.merge_base(bootstrap_ref)?;

    let (frozen_policy, frozen_policy_source) =
        match source.show_file(&merge_base, RATCHET_POLICY_PATH)? {
            Some(text) => {
                let policy = parse_ratchet_policy(&text)
                    .map_err(|e| format!("{RATCHET_POLICY_PATH}@{merge_base}: {e}"))?;
                (policy, FROZEN_POLICY_SOURCE_MERGE_BASE)
            }
            // Declared bootstrap path: the ratchet policy does not exist at the merge-base
            // (the PR that introduces the ratchet). The candidate policy is all there is;
            // the provenance DECLARES the fallback so it is auditable, and the bootstrap
            // cross-check below still binds the comparison root out-of-band.
            None => (
                candidate_policy.clone(),
                FROZEN_POLICY_SOURCE_CANDIDATE_BOOTSTRAP,
            ),
        };

    if frozen_policy.base_ref != bootstrap_ref {
        return Err(format!(
            "frozen ratchet policy base_ref {:?} (source: {frozen_policy_source}) disagrees \
             with the out-of-band bootstrap ref {bootstrap_ref:?} — fail-closed. Repointing \
             the comparison root requires updating BOTH the merged ratchet-policy.json and \
             the CI invocation (--frozen-base-ref / DEFAULT_FROZEN_BOOTSTRAP_REF), never a \
             same-PR policy edit (FRIC-1781280000 frozen-policy-wins).",
            frozen_policy.base_ref
        ));
    }

    let face = match source.show_file(&merge_base, &frozen_policy.face_path)? {
        Some(text) => Some(
            serde_json::from_str(&text)
                .map_err(|e| format!("{}@{merge_base} parse: {e}", frozen_policy.face_path))?,
        ),
        None => None,
    };
    Ok(build_merge_base_baseline_snapshot(
        &frozen_policy,
        frozen_policy_source,
        &merge_base,
        face,
    ))
}

/// Materialize the frozen reference: bootstrap -> merge-base -> frozen policy -> frozen
/// face -> snapshot. The CANDIDATE policy contributes only the local `out_path` (where the
/// untracked snapshot is written) — never any fact that selects the frozen reference.
fn emit_merge_base_baseline(repo_root: &Path, bootstrap_ref: &str) -> Result<(), String> {
    let policy_path = repo_root.join(RATCHET_POLICY_PATH);
    let policy_text = std::fs::read_to_string(&policy_path)
        .map_err(|e| format!("{}: {e}", policy_path.display()))?;
    let candidate_policy = parse_ratchet_policy(&policy_text)?;

    let source = GitCliFrozenRefSource { repo_root };
    let snapshot =
        resolve_merge_base_baseline_snapshot(&source, &candidate_policy, bootstrap_ref)?;

    let out = repo_root.join(&candidate_policy.out_path);
    let text = to_canonical_json(&snapshot).map_err(|e| format!("serialize snapshot: {e}"))?;
    std::fs::write(&out, &text).map_err(|e| format!("{}: {e}", out.display()))?;
    eprintln!(
        "oya-cloud-ci-scm-facts-emitter-app: frozen baseline {} @ merge-base {} (policy: {}{}) -> {}",
        snapshot["base_ref"].as_str().unwrap_or("?"),
        snapshot["merge_base"].as_str().unwrap_or("?"),
        snapshot["frozen_policy_source"].as_str().unwrap_or("?"),
        if snapshot["missing_at_merge_base"] == json!(true) {
            "; face missing at merge-base: EMPTY frozen reference"
        } else {
            ""
        },
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
    volatile: serde_json::Value,
    tracked_paths_len: usize,
}

fn emit_scm_facts(source: &impl ScmFactsSource) -> Result<ScmFactsEmission, String> {
    let tracked_paths = source.tracked_paths()?;
    let tracked_paths_len = tracked_paths.len();
    let tracked_path_set: std::collections::BTreeSet<&String> = tracked_paths.iter().collect();
    // CONVERGENCE PIN (FRIC-1781234047 charter): generated-class paths are excluded from
    // last_touch AT THE EMISSION SEAM, regardless of which ScmFactsSource implementation
    // supplied the map (the git walk already filters; this makes the invariant hold for any
    // future bespoke SCM source too). A generated face's "last touch" is self-referential —
    // the settle commit that writes it — so admitting it would make the volatile snapshot
    // churn on every settle and the faces-only settle commit would never be a fixpoint.
    let last_touch_commit: BTreeMap<String, String> = source
        .last_touch()?
        .into_iter()
        .filter(|(path, _)| tracked_path_set.contains(path) && !is_generated_class(path))
        .collect();
    let all_commit_ts = source.revision_author_timestamps()?;

    // STABLE vs VOLATILE (ADR-0552). The COMMITTED face is a pure function of the committed
    // TREE STATE — `tracked_paths` only — so neither HEAD advancement, nor a faces-only
    // settle commit, nor a squash-merge (which rewrites every lane commit id but preserves
    // the tree) can change its bytes. Everything HISTORY-derived lives in the volatile
    // snapshot instead:
    //   - `last_touch_commit` — per-path last-touch revision ids (rewritten by squash-merge);
    //   - `commit_author_ts_secs` — ONLY the timestamps of commits that are some path's
    //     last-touch (the only ones staleness aging ever looks up);
    //   - `head_time_secs` — the deterministic "now" for aging: the MAX last-touch timestamp,
    //     never a wall clock, so aging is reproducible at a given history.
    let last_touch_shas: std::collections::BTreeSet<&String> = last_touch_commit.values().collect();
    let commit_author_ts_secs: BTreeMap<String, u64> = all_commit_ts
        .iter()
        .filter(|(sha, _)| last_touch_shas.contains(sha))
        .map(|(sha, ts)| (sha.clone(), *ts))
        .collect();
    let head_time_secs = commit_author_ts_secs.values().copied().max().unwrap_or(0);

    let value = json!({
        "schema": SCHEMA,
        "tracked_paths": tracked_paths,
    });
    let volatile = json!({
        "schema": VOLATILE_SCHEMA,
        "_comment": "GENERATED out-of-graph by oya-cloud-ci-scm-facts-emitter-app (ADR-0552, FRIC-1781234047). HISTORY-derived volatile facts: rewritten by squash-merges, so NEVER a committed merge surface and NEVER byte-compared. Untracked + gitignored; CI rematerializes it before gates consume it (infra/ci/materialize-cloud-ci-generated-faces.sh).",
        "head_time_secs": head_time_secs,
        "last_touch_commit": last_touch_commit,
        "commit_author_ts_secs": commit_author_ts_secs,
    });
    Ok(ScmFactsEmission {
        value,
        volatile,
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
        // The COMMITTED face carries ONLY tree-derived stable facts (ADR-0552): no
        // last_touch, no timestamps, no aging anchor — nothing a squash-merge can rewrite.
        assert_eq!(
            emission.value,
            json!({
                "schema": SCHEMA,
                "tracked_paths": ["a.txt", "b.txt"],
            })
        );
        // The history-derived facts live in the volatile snapshot, dropped paths excluded.
        assert_eq!(emission.volatile["schema"], VOLATILE_SCHEMA);
        assert_eq!(emission.volatile["head_time_secs"], 20);
        assert_eq!(
            emission.volatile["last_touch_commit"],
            json!({"a.txt": "rev-a", "b.txt": "rev-b"})
        );
        assert_eq!(
            emission.volatile["commit_author_ts_secs"],
            json!({"rev-a": 10, "rev-b": 20})
        );
    }

    #[test]
    fn generated_class_paths_never_enter_volatile_last_touch() {
        // CONVERGENCE PIN (FRIC-1781234047): a generated-class path's last-touch is the
        // settle commit that wrote it — self-referential. The emission seam must exclude it
        // for ANY ScmFactsSource implementation (not just the git walk), so a faces-only
        // settle commit is a fixpoint: it can never re-grow the volatile snapshot.
        let source = FakeScmFactsSource {
            tracked_paths: vec![
                "Cargo.lock".to_owned(),
                "a/face.generated.json".to_owned(),
                "src/real.rs".to_owned(),
            ],
            last_touch: BTreeMap::from([
                ("Cargo.lock".to_owned(), "rev-lock".to_owned()),
                ("a/face.generated.json".to_owned(), "rev-face".to_owned()),
                ("src/real.rs".to_owned(), "rev-src".to_owned()),
            ]),
            revision_author_timestamps: BTreeMap::from([
                ("rev-lock".to_owned(), 50),
                ("rev-face".to_owned(), 60),
                ("rev-src".to_owned(), 40),
            ]),
        };

        let emission = emit_scm_facts(&source).unwrap();

        assert_eq!(
            emission.volatile["last_touch_commit"],
            json!({"src/real.rs": "rev-src"}),
            "generated-class paths (settle-commit-touched) must be excluded at the seam"
        );
        // The aging anchor follows: only non-generated last-touch timestamps survive, so a
        // settle commit cannot advance head_time_secs either.
        assert_eq!(emission.volatile["head_time_secs"], 40);
        assert_eq!(
            emission.volatile["commit_author_ts_secs"],
            json!({"rev-src": 40})
        );
        // And the committed face is untouched by any of it.
        assert_eq!(
            emission.value,
            json!({
                "schema": SCHEMA,
                "tracked_paths": ["Cargo.lock", "a/face.generated.json", "src/real.rs"],
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
            FROZEN_POLICY_SOURCE_MERGE_BASE,
            "d5d8be5d4121e91655d7ba361f63271c98c57a68",
            Some(face.clone()),
        );
        assert_eq!(snapshot["schema"], "oya-ci/merge-base-baseline/v2");
        assert_eq!(snapshot["base_ref"], "origin/dev");
        assert_eq!(
            snapshot["merge_base"],
            "d5d8be5d4121e91655d7ba361f63271c98c57a68"
        );
        assert_eq!(snapshot["frozen_policy_source"], "merge-base");
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
            FROZEN_POLICY_SOURCE_MERGE_BASE,
            "d5d8be5d4121e91655d7ba361f63271c98c57a68",
            None,
        );
        assert_eq!(snapshot["missing_at_merge_base"], true);
        assert_eq!(snapshot["baseline"], json!({"gates": {}}));
    }

    // -----------------------------------------------------------------------
    // Frozen-policy-wins (FRIC-1781280000): the PR #698 review attack recipe,
    // reproduced over the git seam.
    // -----------------------------------------------------------------------

    const BASE_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HEAD_SHA: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const FACE_PATH: &str =
        "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/gate-baseline.generated.json";

    /// The repoint-attack repository shape: the base branch carries the honest policy +
    /// face; HEAD carries the SAME-PR attack — `base_ref` repointed to `"HEAD"` AND the
    /// settled (regenerated) face that absorbed a planted blocking-debt key.
    struct RepointAttackRepo;

    impl RepointAttackRepo {
        fn base_policy() -> String {
            POLICY_TEXT.to_owned()
        }

        fn attacker_policy() -> String {
            POLICY_TEXT.replace("origin/dev", "HEAD")
        }

        fn base_face() -> serde_json::Value {
            json!({"gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "baseline-block-on-new", "keys": ["pre-existing.rs"]}
            }}})
        }

        fn attacked_face() -> serde_json::Value {
            json!({"gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "baseline-block-on-new",
                                 "keys": ["PLANTED-debt.rs", "pre-existing.rs"]}
            }}})
        }
    }

    impl FrozenRefSource for RepointAttackRepo {
        fn merge_base(&self, base_ref: &str) -> Result<String, String> {
            match base_ref {
                "origin/dev" => Ok(BASE_SHA.to_owned()),
                // merge-base(HEAD, HEAD) = HEAD — the attacker's fixpoint.
                "HEAD" => Ok(HEAD_SHA.to_owned()),
                other => Err(format!("unknown ref {other}")),
            }
        }

        fn show_file(&self, revision: &str, path: &str) -> Result<Option<String>, String> {
            let value = match (revision, path) {
                (BASE_SHA, RATCHET_POLICY_PATH) => Some(Self::base_policy()),
                (BASE_SHA, FACE_PATH) => Some(Self::base_face().to_string()),
                (HEAD_SHA, RATCHET_POLICY_PATH) => Some(Self::attacker_policy()),
                (HEAD_SHA, FACE_PATH) => Some(Self::attacked_face().to_string()),
                _ => None,
            };
            Ok(value)
        }
    }

    /// THE F1 RED PIN — the exact #698 review recipe: a same-PR `"base_ref": "HEAD"`
    /// repoint + a planted blocking-debt key + the mandated settle regen. Under
    /// frozen-policy-wins the candidate policy CANNOT select the frozen reference: the
    /// out-of-band bootstrap locates the merge-base, the policy + face are read THERE, and
    /// the firewall goes RED on both predicates.
    #[test]
    fn frozen_policy_wins_defeats_same_pr_base_ref_repoint() {
        let candidate = parse_ratchet_policy(&RepointAttackRepo::attacker_policy()).unwrap();
        assert_eq!(candidate.base_ref, "HEAD", "the attack edit is in the candidate tree");

        let snapshot = resolve_merge_base_baseline_snapshot(
            &RepointAttackRepo,
            &candidate,
            DEFAULT_FROZEN_BOOTSTRAP_REF,
        )
        .unwrap();

        // The frozen point is the REAL merge-base, selected by the FROZEN policy — the
        // candidate repoint changed nothing about this PR's own reference.
        assert_eq!(snapshot["merge_base"], BASE_SHA);
        assert_eq!(snapshot["base_ref"], "origin/dev");
        assert_eq!(snapshot["frozen_policy_source"], "merge-base");
        assert_eq!(snapshot["baseline"], RepointAttackRepo::base_face());

        // End-to-end: the firewall over (frozen snapshot, attacked proposed/current) is
        // RED on BOTH predicates — the planted key is growth AND a compare regression.
        let frozen =
            oya_cloud_ci_firewall_app::FrozenBaseline::from_value(&snapshot).unwrap();
        let proposed = oya_cloud_ci_firewall_app::Baseline::from_value(
            &RepointAttackRepo::attacked_face(),
        );
        let current = oya_cloud_ci_firewall_app::baseline_keys_map(&proposed);
        let report = oya_cloud_ci_firewall_app::evaluate_firewall(
            &frozen.baseline,
            &proposed,
            &current,
            &oya_cloud_ci_firewall_app::SignOff::default(),
        );
        assert!(
            report
                .ratchet_growth
                .iter()
                .any(|(_, code, key)| code == "unjustified" && key == "PLANTED-debt.rs"),
            "the planted key must be ratchet growth vs the frozen merge-base: {:?}",
            report.ratchet_growth
        );
        assert!(
            report.codes.iter().any(|r| r.code == "unjustified"
                && r.regressions.contains("PLANTED-debt.rs")
                && r.fails()),
            "the planted key must be a failing compare-mode regression"
        );
        assert!(!report.is_green(), "the repoint attack must go RED at head");

        // THE FOIL — why the bootstrap must be OUT-OF-BAND: trusting the candidate
        // policy's base_ref (the pre-hardening behavior) converges to the attacker's
        // fixpoint (merge-base(HEAD, HEAD) = HEAD), the "frozen" face is the PR's own
        // settled copy, and the laundering is structurally invisible (GREEN).
        let foil_snapshot =
            resolve_merge_base_baseline_snapshot(&RepointAttackRepo, &candidate, "HEAD")
                .unwrap();
        assert_eq!(foil_snapshot["merge_base"], HEAD_SHA);
        assert_eq!(foil_snapshot["baseline"], RepointAttackRepo::attacked_face());
        let foil_frozen =
            oya_cloud_ci_firewall_app::FrozenBaseline::from_value(&foil_snapshot).unwrap();
        let foil_report = oya_cloud_ci_firewall_app::evaluate_firewall(
            &foil_frozen.baseline,
            &proposed,
            &current,
            &oya_cloud_ci_firewall_app::SignOff::default(),
        );
        assert!(
            foil_report.is_green(),
            "FOIL: a candidate-selected reference cannot see its own laundering — if this \
             fails, the foil no longer demonstrates the hole and the pin needs re-derivation"
        );
    }

    /// A FRESH attack variant: the policy at the merge-base is honest, but the bootstrap
    /// invocation and the merged policy disagree (e.g. a half-landed repoint, or an
    /// attacker-controlled invocation naming a ref whose merge-base policy says
    /// otherwise). Fail-closed: never proceed with an ambiguous comparison root.
    #[test]
    fn frozen_policy_base_ref_must_agree_with_bootstrap() {
        struct DivergentRepo;
        impl FrozenRefSource for DivergentRepo {
            fn merge_base(&self, _base_ref: &str) -> Result<String, String> {
                Ok(BASE_SHA.to_owned())
            }
            fn show_file(&self, _revision: &str, path: &str) -> Result<Option<String>, String> {
                Ok((path == RATCHET_POLICY_PATH).then(RepointAttackRepo::base_policy))
            }
        }
        let candidate = parse_ratchet_policy(POLICY_TEXT).unwrap();
        let err =
            resolve_merge_base_baseline_snapshot(&DivergentRepo, &candidate, "origin/main")
                .unwrap_err();
        assert!(err.contains("disagrees"), "{err}");
        assert!(err.contains("FRIC-1781280000"), "{err}");
    }

    /// The DECLARED bootstrap path: the policy does not exist at the merge-base (the PR
    /// introducing the ratchet). The candidate policy is used, the provenance declares it,
    /// and the bootstrap cross-check still binds the comparison root out-of-band.
    #[test]
    fn policy_missing_at_merge_base_falls_back_to_declared_candidate_bootstrap() {
        struct PreRatchetRepo;
        impl FrozenRefSource for PreRatchetRepo {
            fn merge_base(&self, base_ref: &str) -> Result<String, String> {
                if base_ref == "origin/dev" {
                    Ok(BASE_SHA.to_owned())
                } else {
                    Err(format!("unknown ref {base_ref}"))
                }
            }
            fn show_file(&self, _revision: &str, _path: &str) -> Result<Option<String>, String> {
                Ok(None) // neither the policy nor the face exists at the merge-base
            }
        }
        let candidate = parse_ratchet_policy(POLICY_TEXT).unwrap();
        let snapshot = resolve_merge_base_baseline_snapshot(
            &PreRatchetRepo,
            &candidate,
            DEFAULT_FROZEN_BOOTSTRAP_REF,
        )
        .unwrap();
        assert_eq!(snapshot["frozen_policy_source"], "candidate-bootstrap");
        assert_eq!(snapshot["missing_at_merge_base"], true);
        assert_eq!(snapshot["baseline"], json!({"gates": {}}));

        // The fallback still refuses a candidate policy that disagrees with the bootstrap:
        // an attacker cannot combine "delete the policy from history" with a repointed
        // candidate copy.
        let attacker = parse_ratchet_policy(&RepointAttackRepo::attacker_policy()).unwrap();
        assert!(
            resolve_merge_base_baseline_snapshot(
                &PreRatchetRepo,
                &attacker,
                DEFAULT_FROZEN_BOOTSTRAP_REF,
            )
            .is_err(),
            "candidate-bootstrap fallback must still bind base_ref to the bootstrap"
        );
    }
}
