//! CLI for the reversible capability-move codemod + pre-move green-snapshot oracle
//! (ADR-0562 Phase-0 reorg machinery). Local bridge tool ONLY — it ships UNUSED until the
//! strangler invokes it, and merge authority stays in the cloud-ci/oya-ci required contexts
//! (cli_surface_policy). NO real capability is moved by this lane.
//!
//! Modes:
//!   apply    --plan <plan.json> [--repo-root <p>] [--revert]
//!              Apply (or, with --revert, inverse-apply) a move plan to the tree. Fail-closed.
//!   dry-run  --plan <plan.json> [--repo-root <p>] [--revert] [--with-buck] [--keep-shadow]
//!              Shadow-apply + prove resolution WITHOUT landing. Exit 0 = clean, 2 = unclean.
//!   snapshot [--repo-root <p>] [--with-buck]
//!              Capture the green snapshot (cargo metadata + buck2 targets) as the rollback
//!              oracle; printed to stdout as canonical JSON faces.
//!
//! The plan JSON shape:
//!   { "capability": "iam",
//!     "moves": [ { "old_path": "...", "new_path": "...",
//!                  "old_cargo_name": "...", "new_cargo_name": "..." }, ... ],
//!     "artifacts": [ { "old_path": "...", "new_path": "..." }, ... ] }   // OPTIONAL
//! `artifacts` (optional; absent => empty) are NON-crate co-moves — SLOs, catalog records —
//! moved content-preserving (no in-file rewrite) alongside the capability's crate moves.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use oya_reorg_codemod_app::model::{
    ArtifactMove, CodemodError, CrateMove, MovePlan, move_manifest_value,
};
use oya_reorg_codemod_app::oracle;
use oya_reorg_codemod_app::plan::{ApplyOptions, apply_plan};
use serde_json::{Value, json};

/// The de-committed move-manifest's canonical repo-relative materialization path (task #64).
/// Regenerated each run under `specs/reorg/` (DECIDED) and declared `not-tracked-in-git` in
/// `registry/generated-artifact-control-plane.json`. ADR-0614's regenerate-twice canary checks
/// it only at a git-bearing Cargo or CI regeneration boundary; hermetic Buck actions skip that
/// source canary by design because they have no `.git` directory.
const DEFAULT_MANIFEST_OUT: &str = "specs/reorg/move-manifest.generated.json";

/// The out-of-band bootstrap ref the landed-plan probe anchors on (the emitter's own base ref).
/// Named once so the probe and the [`CodemodError::MergeBaseUnresolved`] report cannot drift.
const MERGE_BASE_REF: &str = "origin/dev";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("oya-reorg-codemod: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("");
    let rest = &args[args.len().min(1)..];

    match mode {
        "apply" => cmd_apply(rest),
        "dry-run" => cmd_dry_run(rest),
        "snapshot" => cmd_snapshot(rest),
        "manifest" => cmd_manifest(rest),
        "" | "-h" | "--help" => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown mode {other:?}; try --help")),
    }
}

/// `manifest [--plan <plan.json>] [--repo-root <p>] [--out <p>]`: materialize the de-committed
/// canonical-JSON move-manifest (schema `oya-ci/reorg-move-manifest/v1`) the rename-aware
/// path-keyed CI baseline relabel consumes (task #64). It loads the plan (if any), VALIDATES
/// it, enumerates the candidate tracked tree (`git ls-files` — the emitter's exact universe),
/// derives the file-level + crate-ident pairs deterministically, and writes the canonical
/// face. With NO `--plan` it discovers the single ACTIVE committed plan itself after excluding
/// already-landed plans; multiple active plans fail closed, while zero active plans write the
/// canonical EMPTY manifest (`files: []`, `crate_idents: []`) — the strict no-op the emitter reads
/// as "no renames" (identity relabel), so a no-move PR is gate-green and byte-stable. Determinism:
/// sorted pairs + canonical JSON keep emissions byte-stable. ADR-0614's regenerate-twice source
/// canary verifies that property only at a git-bearing Cargo or CI regeneration boundary; hermetic
/// Buck actions skip it by design.
fn cmd_manifest(args: &[String]) -> Result<ExitCode, String> {
    let mut plan_path: Option<PathBuf> = None;
    let mut repo_root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut out: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--plan" => plan_path = Some(PathBuf::from(next(args, &mut i, "--plan")?)),
            "--repo-root" => repo_root = PathBuf::from(next(args, &mut i, "--repo-root")?),
            "--out" => out = Some(PathBuf::from(next(args, &mut i, "--out")?)),
            other => return Err(format!("unknown flag {other:?}")),
        }
        i += 1;
    }
    let out = out.unwrap_or_else(|| repo_root.join(DEFAULT_MANIFEST_OUT));

    // FAIL-CLOSED on a >1-committed-plan candidate tree (#65). A MOVE PR commits exactly one plan
    // at `specs/reorg/<capability>-move-plan.json`; more than one is a contributor error the
    // materialization must NOT silently first-win on. This guard runs REGARDLESS of `--plan` (the
    // candidate tree is ambiguous full stop), and when no `--plan` is named the codemod itself
    // SELECTS the single committed plan — so the materialization is the authority and a no-move PR
    // (zero plans) still emits the canonical empty manifest.
    // MUST-PASS #5 (straddle DoS): exclude ALREADY-LANDED committed plans (whose every move old
    // crate-dir is absent from the merge-base tree) BEFORE the single-plan count guard, so a merged
    // move-plan a prior PR never cleaned up cannot hard-error every subsequent materialization. The
    // merge-base is the emitter's out-of-band bootstrap (`origin/dev`). A per-path git failure at a
    // RESOLVED merge-base fails closed to PRESENT, so that plan stays ACTIVE and the guard stays
    // sharp. A merge-base that does not resolve AT ALL is a different animal: it is an INPUT failure
    // that leaves EVERY plan unclassifiable, so it is reported as itself rather than coerced.
    //
    // It used to return `false` ("not absent" => present => pending). That made all N committed
    // plans read ACTIVE on any checkout where `origin/dev` was missing (shallow clone, force-pushed
    // base, rewritten history, a fetch that never brought the ref), and the materializer then died
    // on `MultipleMovePlans { count: N }` from step 1 — every CI leg, every local gate lane,
    // repo-wide, pointing remediation at deleting move plans that were never the problem.
    let merge_base = git_merge_base(&repo_root, MERGE_BASE_REF);
    let old_dir_absent_at_merge_base = |dir: &str| -> Result<bool, CodemodError> {
        let merge_base =
            merge_base
                .as_deref()
                .ok_or_else(|| CodemodError::MergeBaseUnresolved {
                    base_ref: MERGE_BASE_REF.to_owned(),
                })?;
        Ok(!git_dir_present_at(&repo_root, merge_base, dir))
    };
    let load_old_crate_dirs = |p: &Path| -> Result<Vec<String>, CodemodError> {
        let plan = load_plan(p, false).map_err(|message| CodemodError::Io {
            context: format!("load committed plan {}", p.display()),
            message,
        })?;
        // Artifact old_paths participate in the landed probe alongside crate moves. Without them an
        // ARTIFACT-ONLY plan (`moves: []`, the PR-B backfill shape that model.rs:119-128 explicitly
        // blesses) yields an EMPTY probe input, and `plan_is_landed` returns false for empty input —
        // so such a plan is ACTIVE FOREVER and can never self-heal. Two of them committed would make
        // `select_move_plan` raise MultipleMovePlans, and that error is raised from step 1 of the
        // UNIVERSAL materializer, fail-closed, on every CI leg and every local gate lane — wedging
        // every subsequent PR in the repo. The validator already accepted this plan shape; only the
        // landed probe was never extended to it.
        //
        // `old_dir_absent_at_merge_base` is path-agnostic (`git ls-tree -r --name-only <rev> -- <p>`
        // matches a file pathspec as well as a directory), so an artifact old_path that names a FILE
        // probes correctly and needs no separate handling.
        Ok(oya_reorg_codemod_app::plan_probe_paths(&plan))
    };
    let plan_path = oya_reorg_codemod_app::resolve_effective_active_move_plan(
        plan_path,
        &repo_root,
        load_old_crate_dirs,
        old_dir_absent_at_merge_base,
    )
    .map_err(|e: CodemodError| e.to_string())?;

    // The plan is OPTIONAL: a no-move PR has no plan and emits the canonical empty manifest.
    // When a plan IS supplied, validate fail-closed (its bijection back-guarantees the relabel
    // determinism) before deriving any pair.
    let (capability, file_pairs, crate_dir_pairs, crate_ident_pairs) = match plan_path {
        Some(path) => {
            let plan = load_plan(&path, false)?;
            plan.validate().map_err(|e: CodemodError| e.to_string())?;
            plan.validate_debrand_targets()
                .map_err(|e: CodemodError| e.to_string())?;
            let tracked = git_ls_files(&repo_root)?;
            // MERGE the NON-crate artifact file pairs into the `files` list (sorted + deduped via
            // a BTreeMap keyed by old_path, the same canonical ordering the crate pairs carry) so
            // ADR-0563's path-keyed relabel + the total-accounting follow co-moved artifacts.
            // EMPTY artifacts => no extra pairs => byte-identical `files` to today (back-compat).
            let mut merged: std::collections::BTreeMap<String, String> =
                plan.file_level_manifest(&tracked).into_iter().collect();
            for (old, new) in plan.artifact_file_pairs(&tracked) {
                merged.insert(old, new);
            }
            (
                plan.capability.clone(),
                merged.into_iter().collect::<Vec<_>>(),
                plan.crate_dir_pairs(&tracked),
                plan.crate_ident_pairs(),
            )
        }
        None => (String::new(), Vec::new(), Vec::new(), Vec::new()),
    };

    let manifest = move_manifest_value(
        &capability,
        &file_pairs,
        &crate_dir_pairs,
        &crate_ident_pairs,
    );
    let text = to_canonical_json(&manifest);
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    std::fs::write(&out, &text).map_err(|e| format!("write {}: {e}", out.display()))?;
    eprintln!(
        "oya-reorg-codemod: move-manifest ({} file pairs, {} crate-dir pairs, {} crate-ident pairs) -> {}",
        file_pairs.len(),
        crate_dir_pairs.len(),
        crate_ident_pairs.len(),
        out.display()
    );
    Ok(ExitCode::SUCCESS)
}

/// The candidate tracked-path universe (`git ls-files`), sorted + deduplicated — the SAME
/// universe the scm-facts emitter censuses (so the file-level derivation the relabel later
/// re-verifies against `git ls-files` agrees by construction). The git call here is local
/// bridge tooling (the codemod ships unused until the strangler invokes it); it never feeds a
/// transform decision, only the manifest enumeration.
fn git_ls_files(repo_root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("ls-files")
        .output()
        .map_err(|e| format!("git ls-files: {e}"))?;
    if !output.status.success() {
        return Err(format!("git ls-files exit {:?}", output.status.code()));
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

/// `git merge-base <base_ref> HEAD` (full hex sha), or `None` when the ref does not resolve.
/// `None` is NOT an answer the landed-plan exclusion can use — it means the exclusion cannot run at
/// all — so the caller turns it into [`CodemodError::MergeBaseUnresolved`]. Disabling the exclusion
/// instead (the old behaviour) silently reclassified every landed plan as ACTIVE.
fn git_merge_base(repo_root: &Path, base_ref: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["merge-base", base_ref, "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (sha.len() >= 40 && sha.chars().all(|c| c.is_ascii_hexdigit())).then_some(sha)
}

/// True iff any tracked file exists under `dir` at `rev`. FAIL-CLOSED to PRESENT (`true`) on any git
/// error, so uncertainty never marks a plan landed (never excludes it): the exclusion can only ever
/// REMOVE a false single-plan-guard trip for a PROVABLY-landed move, never hide a pending one.
fn git_dir_present_at(repo_root: &Path, rev: &str, dir: &str) -> bool {
    let dir = dir.trim_end_matches('/');
    let out = match Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["ls-tree", "-r", "--name-only", rev, "--", dir])
        .output()
    {
        Ok(out) => out,
        Err(_) => return true,
    };
    if !out.status.success() {
        return true;
    }
    !String::from_utf8_lossy(&out.stdout).trim().is_empty()
}

fn cmd_apply(args: &[String]) -> Result<ExitCode, String> {
    let opts = parse_common(args)?;
    let plan = load_plan(&opts.plan, opts.revert)?;
    let outcome = apply_plan(&opts.repo_root, &plan, &ApplyOptions { use_git_mv: true })
        .map_err(|e: CodemodError| e.to_string())?;
    println!("{}", to_canonical_json(&apply_outcome_json(&outcome)));
    Ok(ExitCode::SUCCESS)
}

fn cmd_dry_run(args: &[String]) -> Result<ExitCode, String> {
    let opts = parse_common(args)?;
    let plan = load_plan(&opts.plan, opts.revert)?;
    let report = oracle::dry_run(&opts.repo_root, &plan, opts.with_buck, opts.keep_shadow)
        .map_err(|e: CodemodError| e.to_string())?;
    println!("{}", to_canonical_json(&dry_run_json(&report)));
    if report.clean {
        Ok(ExitCode::SUCCESS)
    } else {
        // Distinct unclean exit code (2) so a gate can tell "tool error" from "move unclean".
        Ok(ExitCode::from(2))
    }
}

fn cmd_snapshot(args: &[String]) -> Result<ExitCode, String> {
    let opts = parse_common_lenient(args)?;
    let snap = oracle::capture_snapshot(&opts.repo_root, opts.with_buck);
    println!("{}", to_canonical_json(&snapshot_json(&snap)));
    Ok(ExitCode::SUCCESS)
}

struct Opts {
    plan: PathBuf,
    repo_root: PathBuf,
    revert: bool,
    with_buck: bool,
    keep_shadow: bool,
}

fn parse_common(args: &[String]) -> Result<Opts, String> {
    let opts = parse_common_lenient(args)?;
    if opts.plan.as_os_str().is_empty() {
        return Err("--plan <path> is required".to_string());
    }
    Ok(opts)
}

fn parse_common_lenient(args: &[String]) -> Result<Opts, String> {
    let mut plan = PathBuf::new();
    let mut repo_root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut revert = false;
    let mut with_buck = false;
    let mut keep_shadow = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--plan" => {
                plan = PathBuf::from(next(args, &mut i, "--plan")?);
            }
            "--repo-root" => {
                repo_root = PathBuf::from(next(args, &mut i, "--repo-root")?);
            }
            "--revert" => revert = true,
            "--with-buck" => with_buck = true,
            "--keep-shadow" => keep_shadow = true,
            other => return Err(format!("unknown flag {other:?}")),
        }
        i += 1;
    }
    Ok(Opts {
        plan,
        repo_root,
        revert,
        with_buck,
        keep_shadow,
    })
}

fn next(args: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    *i += 1;
    args.get(*i)
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn load_plan(path: &Path, revert: bool) -> Result<MovePlan, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read plan {}: {e}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|e| format!("parse plan {}: {e}", path.display()))?;
    let capability = value
        .get("capability")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let moves_val = value
        .get("moves")
        .and_then(Value::as_array)
        .ok_or("plan missing `moves` array")?;
    let mut moves = Vec::with_capacity(moves_val.len());
    for (idx, m) in moves_val.iter().enumerate() {
        moves.push(CrateMove {
            old_path: field(m, "old_path", idx)?,
            new_path: field(m, "new_path", idx)?,
            old_cargo_name: field(m, "old_cargo_name", idx)?,
            new_cargo_name: field(m, "new_cargo_name", idx)?,
        });
    }
    // Optional top-level `artifacts: [{old_path, new_path}]` — NON-crate co-moves (SLOs, catalog
    // records). ABSENT => empty Vec (back-compatible with the committed 4-field marketplace plan
    // shape, so existing plans parse + behave identically).
    let artifacts = match value.get("artifacts") {
        None => Vec::new(),
        Some(arts_val) => {
            let arts = arts_val
                .as_array()
                .ok_or("plan `artifacts` must be an array")?;
            let mut out = Vec::with_capacity(arts.len());
            for (idx, a) in arts.iter().enumerate() {
                out.push(ArtifactMove {
                    old_path: artifact_field(a, "old_path", idx)?,
                    new_path: artifact_field(a, "new_path", idx)?,
                });
            }
            out
        }
    };
    let plan = MovePlan {
        capability,
        moves,
        artifacts,
    };
    Ok(if revert { plan.inverse() } else { plan })
}

fn field(m: &Value, key: &str, idx: usize) -> Result<String, String> {
    m.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("move[{idx}] missing string field {key:?}"))
}

fn artifact_field(a: &Value, key: &str, idx: usize) -> Result<String, String> {
    a.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("artifact[{idx}] missing string field {key:?}"))
}

fn apply_outcome_json(o: &oya_reorg_codemod_app::plan::ApplyOutcome) -> Value {
    json!({
        "capability": o.mapping.capability,
        "mapping": o.mapping.rows.iter().map(|r| json!({
            "old_path": r.old_path,
            "new_path": r.new_path,
            "old_cargo_name": r.old_cargo_name,
            "new_cargo_name": r.new_cargo_name,
            "buck_label": r.buck_label,
        })).collect::<Vec<_>>(),
        "manifests_rewritten": o.manifests_rewritten,
        "bucks_rewritten": o.bucks_rewritten,
        "rust_files_rewritten": o.rust_files_rewritten,
        "docs_rewritten": o.docs_rewritten,
        "root_workspace_changed": o.root_workspace_changed,
        "cargo_lock_changed": o.cargo_lock_changed,
        "dirs_moved": o.dirs_moved.iter().map(|(a, b)| json!([a, b])).collect::<Vec<_>>(),
    })
}

fn dry_run_json(r: &oracle::DryRunReport) -> Value {
    json!({
        "clean": r.clean,
        "cargo_ok": r.cargo_ok,
        "cargo_detail": truncate(&r.cargo_detail, 4000),
        "buck_ok": r.buck_ok,
        "buck_detail": truncate(&r.buck_detail, 4000),
    })
}

fn snapshot_json(s: &oracle::GreenSnapshot) -> Value {
    json!({
        "cargo_ok": s.cargo_ok,
        "buck_available": s.buck_available,
        "buck_ok": s.buck_ok,
        "cargo_metadata_len": s.cargo_metadata.len(),
        "buck_targets_len": s.buck_targets.len(),
    })
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

/// Canonical JSON: sorted keys, 2-space indent, trailing newline. Mirrors the repo's
/// canonical-json discipline for any emitted JSON.
fn to_canonical_json(value: &Value) -> String {
    let canon = canonicalize(value);
    let mut out = serde_json::to_string_pretty(&canon).unwrap_or_default();
    out.push('\n');
    out
}

fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = serde_json::Map::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for k in keys {
                sorted.insert(k.clone(), canonicalize(&map[k]));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize).collect()),
        other => other.clone(),
    }
}

fn print_usage() {
    eprintln!(
        "oya-reorg-codemod — reversible capability-move codemod + green-snapshot oracle (ADR-0562)\n\
         \n\
         USAGE:\n\
         \x20 oya-reorg-codemod apply    --plan <plan.json> [--repo-root <p>] [--revert]\n\
         \x20 oya-reorg-codemod dry-run  --plan <plan.json> [--repo-root <p>] [--revert] [--with-buck] [--keep-shadow]\n\
         \x20 oya-reorg-codemod snapshot [--repo-root <p>] [--with-buck]\n\
         \x20 oya-reorg-codemod manifest [--plan <plan.json>] [--repo-root <p>] [--out <p>]\n\
         \n\
         Local bridge ONLY; merge authority stays in cloud-ci/oya-ci. Ships UNUSED until the\n\
         strangler invokes it. dry-run exits 0=clean, 2=unclean (fail-closed)."
    );
}
