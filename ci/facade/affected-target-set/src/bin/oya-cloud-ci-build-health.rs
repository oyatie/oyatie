//! cloud-ci build-health ratchet (ADR-0554 round-3; reuses the ADR-0551/#698 merge-base
//! frozen-baseline pattern).
//!
//! Reads two buck2 `--build-report` JSON files — the BASELINE (a full `buck2 build //...
//! --keep-going` at the MERGE-BASE checkout, materialized out-of-band so it is NEVER
//! candidate-controlled) and the HEAD (the same build at the PR head) — and compares their
//! failure sets:
//!
//!   - a head failure NOT in the baseline failure set is a REGRESSION (it built at the
//!     merge-base, or the target is brand-new) -> BLOCK (exit 1);
//!   - a head failure ALSO in the baseline is GRANDFATHERED (shrink-only burn-down) -> allowed;
//!   - a baseline failure that now builds is FIXED (informational).
//!
//! The required context is green IFF there are no regressions. This turns the FULL tier from a
//! flag-day requirement (the whole workspace must compile) into a true ratchet: block NEW build
//! debt, grandfather pre-existing (the founder doctrine, FRIC-1781112000 / #698).
//!
//! SOUNDNESS (the #698 F1 lesson, re-checked here): the baseline failure set comes ENTIRELY from
//! the `--baseline-report` file, which the workflow produces by building the MERGE-BASE checkout.
//! Nothing in the candidate tree feeds the baseline, so a PR cannot launder a regression by
//! growing its own baseline. The binary refuses to run if the baseline report is missing/empty in
//! a way that would silently turn every head failure into "grandfathered" — see the
//! `--require-baseline` guard.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use ci_affected_target_set::{
    BASELINE_PROVENANCE_FILENAME, BaselineKind, BuildHealthVerdict,
    REQUIRED_CONTEXT_WORKFLOW_PATH, baseline_artifact_name,
    build_health_verdict, failing_targets, parse_build_report, parse_test_verdicts,
    test_verdicts_to_report_value, trusted_baseline_artifact_id, trusted_dev_push_run_id,
    validate_trusted_baseline_artifact, validated_merge_base_sha,
};

const LOG: &str = "build-health";

/// Exit code meaning "no trusted baseline is available — fall back to the cold merge-base
/// rebuild". Distinct from 2 (bad arguments) purely for operator legibility; the workflow falls
/// back on ANY non-zero, which is the fail-closed default.
const NO_TRUSTED_BASELINE: u8 = 3;

struct Args {
    baseline_report: String,
    head_report: String,
    /// Fail-closed guard: if set, the baseline report MUST parse to a non-empty `results` map
    /// (a real build happened at the merge-base). Without it, a truncated/empty baseline would
    /// make every head failure look pre-existing — the laundering hole. CI always passes it.
    require_baseline_results: bool,
}

fn parse_args(mut argv: std::env::Args) -> Result<Args, String> {
    let _bin = argv.next();
    let mut baseline_report = None;
    let mut head_report = None;
    let mut require_baseline_results = false;
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--baseline-report" => {
                baseline_report = Some(argv.next().ok_or("--baseline-report needs a value")?)
            }
            "--head-report" => {
                head_report = Some(argv.next().ok_or("--head-report needs a value")?)
            }
            "--require-baseline-results" => require_baseline_results = true,
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(Args {
        baseline_report: baseline_report.ok_or("--baseline-report <path> is required")?,
        head_report: head_report.ok_or("--head-report <path> is required")?,
        require_baseline_results,
    })
}

/// NORMALIZE mode (ADR-0554 round-6, defect 3/4): distill a captured `buck2 test //...` console
/// stream into a build-report-shaped `{results: {label: {success}}}` JSON, so a merge-base TEST
/// baseline flows through the exact same `parse_build_report`/`failing_targets`/ratchet machinery
/// as the build baseline. Fail-closed: an unreconcilable console (see [`parse_test_verdicts`])
/// errors rather than emitting an under-counted baseline that could launder a test regression.
fn run_normalize(console_path: &str, out_path: &str) -> ExitCode {
    let console = match fs::read_to_string(console_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{LOG}: NORMALIZE ERROR: cannot read console `{console_path}`: {e}");
            return ExitCode::from(2);
        }
    };
    let verdicts = match parse_test_verdicts(&console) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{LOG}: NORMALIZE ERROR: {e}");
            return ExitCode::from(2);
        }
    };
    let value = test_verdicts_to_report_value(&verdicts);
    let bytes = match serde_json::to_vec_pretty(&value) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{LOG}: NORMALIZE ERROR: serialize test-verdict report: {e}");
            return ExitCode::from(2);
        }
    };
    if let Err(e) = fs::write(out_path, bytes) {
        eprintln!("{LOG}: NORMALIZE ERROR: write `{out_path}`: {e}");
        return ExitCode::from(2);
    }
    println!(
        "{LOG}: normalized {} test target verdict(s) -> {out_path}",
        verdicts.len()
    );
    ExitCode::SUCCESS
}

/// The value that follows `flag` in `raw`, if present.
fn flag_value<'a>(raw: &'a [String], flag: &str) -> Option<&'a str> {
    raw.iter()
        .position(|a| a == flag)
        .and_then(|i| raw.get(i + 1))
        .map(String::as_str)
}

fn require<'a>(raw: &'a [String], flag: &str) -> Result<&'a str, String> {
    flag_value(raw, flag).ok_or_else(|| format!("{flag} <value> is required"))
}

/// TRUSTED-BASELINE CONSUMER (GH #1323/#899; the ADR-0554 D8 cross-run consumer).
///
/// The merge-base of a PR IS a `dev` commit, and its push-to-dev `oya-ci-required` run already
/// built AND tested the whole workspace and published both baselines as artifacts. Rebuilding them
/// cold in a clean worktree is the FULL tier's wall-clock bottleneck — 31m16s of a measured 68m57s
/// job on PR #1376. When the published pair validates, we download it and the cold rebuild is
/// skipped entirely.
///
/// OWNED-RUST, NOT WORKFLOW SHELL: the whole sequence — list runs, select the trusted one, select
/// each artifact, download, unzip, validate — lives here rather than in the workflow's `run:`
/// block. The crate gains no HTTP client and no new dependency; it drives the already-installed
/// `gh` and `unzip` as subprocesses, exactly the composition-root pattern the sibling affected-set
/// binary uses for `git` and `buck2`. Keeping it here also keeps the rust-first inline-shell
/// ratchet honest: the workflow grows a handful of lines instead of ~35.
///
/// ANTI-LAUNDERING (the D6 guarantees, preserved and tightened): a baseline is accepted ONLY from
/// a GitHub Actions run whose PROVENANCE proves the candidate could not have produced it —
/// `event=push`, `head_branch=dev`, `conclusion=success`, `path` = the single required-context
/// workflow file, and `head_sha` EXACTLY the merge-base. The artifact NAME is never sufficient on
/// its own; it is additionally checked against the name the API reports for the id actually
/// downloaded, and the payload must parse to a non-empty build-report `results` map. ANY doubt at
/// ANY step exits non-zero with the partial outputs deleted, and the workflow then runs the
/// untouched cold rebuild.
///
/// Usage: `--trusted-baseline --merge-base <sha> --repo <owner/name> --out-dir <dir>`.
fn trusted_baseline_exit(raw: &[String]) -> u8 {
    match reuse_trusted_baselines(raw) {
        Ok(()) => 0,
        Err(TrustedBaselineOutcome::Unavailable(why)) => {
            eprintln!(
                "{LOG}: NO TRUSTED BASELINE — {why}; the cold merge-base rebuild runs instead"
            );
            NO_TRUSTED_BASELINE
        }
        Err(TrustedBaselineOutcome::Refused(why)) => {
            eprintln!(
                "{LOG}: TRUSTED BASELINE REFUSED — {why}; the cold merge-base rebuild runs instead"
            );
            2
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TrustedBaselineOutcome {
    /// No baseline exists for this merge-base (normal: first push, expired retention, red dev).
    Unavailable(String),
    /// A baseline exists but failed a provenance/shape check, or an input/subprocess was bad.
    Refused(String),
}

use TrustedBaselineOutcome::{Refused, Unavailable};

/// `owner/name`, restricted to the characters GitHub actually allows, so a hostile value can never
/// smuggle extra path or query segments into the API routes built below.
fn validated_repo(repo: &str) -> Result<&str, TrustedBaselineOutcome> {
    let mut parts = repo.split('/');
    let ok = matches!((parts.next(), parts.next(), parts.next()), (Some(o), Some(n), None)
        if !o.is_empty()
            && !n.is_empty()
            && repo
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/')));
    if ok {
        Ok(repo)
    } else {
        Err(Refused(format!("`{repo}` is not a valid <owner>/<name> repository")))
    }
}

/// PURE: which workflow run may be trusted for `merge_base`, given the runs payload.
fn select_trusted_run(runs_json: &str, merge_base: &str) -> Result<u64, TrustedBaselineOutcome> {
    match trusted_dev_push_run_id(runs_json, merge_base, REQUIRED_CONTEXT_WORKFLOW_PATH)
        .map_err(Refused)?
    {
        Some(id) => Ok(id),
        None => Err(Unavailable(format!(
            "no successful push-to-dev `{REQUIRED_CONTEXT_WORKFLOW_PATH}` run at merge-base {merge_base}"
        ))),
    }
}

/// PURE: which artifact of a trusted run carries the `kind` baseline for `merge_base`.
fn select_trusted_artifact(
    artifacts_json: &str,
    kind: BaselineKind,
    merge_base: &str,
) -> Result<(u64, String), TrustedBaselineOutcome> {
    let name = baseline_artifact_name(kind, merge_base).map_err(Refused)?;
    match trusted_baseline_artifact_id(artifacts_json, &name).map_err(Refused)? {
        Some(id) => Ok((id, name)),
        None => Err(Unavailable(format!(
            "the trusted run published no unexpired `{name}` artifact"
        ))),
    }
}

/// PURE: may the downloaded payload be used as the `kind` baseline? `artifact_name` is what the
/// API reports for the id that was actually fetched, not a locally-recomputed string.
fn accept_downloaded_baseline(
    kind: BaselineKind,
    artifact_name: &str,
    merge_base: &str,
    report_json: &str,
) -> Result<usize, TrustedBaselineOutcome> {
    validate_trusted_baseline_artifact(kind, artifact_name, merge_base, report_json)
        .map_err(Refused)
}

/// Run `gh api <route>` and return stdout. A non-zero status (404, no token, rate limit) is a
/// refusal, never a silent empty payload.
fn gh_api(route: &str) -> Result<String, TrustedBaselineOutcome> {
    let out = Command::new("gh")
        .args(["api", route])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Refused(format!("could not execute `gh api {route}`: {e}")))?;
    if !out.status.success() {
        return Err(Refused(format!(
            "`gh api {route}` exited with {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    String::from_utf8(out.stdout).map_err(|e| Refused(format!("`gh api {route}` output is not UTF-8: {e}")))
}

/// Stream a subprocess's stdout straight to `dest` (artifact zips are binary, and `unzip -p`
/// output can be large — neither belongs in memory).
fn capture_to_file(
    program: &str,
    args: &[&str],
    dest: &Path,
) -> Result<(), TrustedBaselineOutcome> {
    let file = fs::File::create(dest)
        .map_err(|e| Refused(format!("could not create `{}`: {e}", dest.display())))?;
    let status = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(file))
        .status()
        .map_err(|e| Refused(format!("could not execute `{program}`: {e}")))?;
    if !status.success() {
        return Err(Refused(format!(
            "`{program} {}` exited with {status}",
            args.join(" ")
        )));
    }
    Ok(())
}

/// Download and validate ONE kind of baseline into `out_dir/<kind>-health-baseline.json`.
fn fetch_one_baseline(
    repo: &str,
    merge_base: &str,
    run_id: u64,
    artifacts_json: &str,
    kind: BaselineKind,
    out_dir: &Path,
) -> Result<(), TrustedBaselineOutcome> {
    let (artifact_id, expected_name) = select_trusted_artifact(artifacts_json, kind, merge_base)?;
    // The name the API reports for the id we are about to download closes the "selected one
    // artifact, fetched another" loop against a live server response.
    let meta = gh_api(&format!("repos/{repo}/actions/artifacts/{artifact_id}"))?;
    let reported_name = serde_json::from_str::<serde_json::Value>(&meta)
        .ok()
        .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(str::to_owned))
        .ok_or_else(|| {
            Refused(format!("artifact {artifact_id} metadata carries no string `name`"))
        })?;

    let zip = out_dir.join(format!("trusted-{}-health-baseline.zip", kind.prefix()));
    capture_to_file(
        "gh",
        &["api", &format!("repos/{repo}/actions/artifacts/{artifact_id}/zip")],
        &zip,
    )?;
    // ADR-0523 irreducible glue: the Actions artifact API serves ONLY a zip, and this crate takes
    // no new dependency, so extraction is delegated to `unzip` (present on ubuntu-latest). It is
    // the one external tool here without prior repo use — deliberately kept to a NON-CRITICAL
    // position: if it is missing or fails, `capture_to_file` refuses, the pair is abandoned, and
    // the cold rebuild runs. Absence therefore costs wall-clock, never correctness. Retire it when
    // an owned inflate lands; do NOT swap in `gh run download`, which selects by NAME and would
    // discard the artifact-ID binding established above.
    let report = out_dir.join(format!("{}-health-baseline.json", kind.prefix()));
    capture_to_file("unzip", &["-p", &zip.display().to_string()], &report)?;
    let _ = fs::remove_file(&zip);

    let payload = fs::read_to_string(&report)
        .map_err(|e| Refused(format!("cannot read `{}`: {e}", report.display())))?;
    let count = accept_downloaded_baseline(kind, &reported_name, merge_base, &payload)?;
    println!(
        "{LOG}: trusted {} baseline VALID — {count} target result(s) from `{expected_name}` \
         (artifact {artifact_id} of run {run_id}) -> {}",
        kind.prefix(),
        report.display()
    );
    Ok(())
}

fn reuse_trusted_baselines(raw: &[String]) -> Result<(), TrustedBaselineOutcome> {
    let out_dir = PathBuf::from(require(raw, "--out-dir").map_err(Refused)?);
    let outcome = reuse_trusted_baselines_inner(raw, &out_dir);
    if outcome.is_err() {
        // ONE fail-closed cleanup for EVERY failure mode, including a late one after both halves
        // validated: never leave a partial baseline the cold path's `test -s` guard could mistake
        // for a freshly rebuilt one, and never leave a sidecar that would mislabel a cold rebuild
        // as a reused artifact.
        for kind in [BaselineKind::Build, BaselineKind::Test] {
            let _ = fs::remove_file(out_dir.join(format!("{}-health-baseline.json", kind.prefix())));
        }
        let _ = fs::remove_file(out_dir.join(BASELINE_PROVENANCE_FILENAME));
    }
    outcome
}

fn reuse_trusted_baselines_inner(
    raw: &[String],
    out_dir: &Path,
) -> Result<(), TrustedBaselineOutcome> {
    // BOTH interpolated values are shape-checked BEFORE they reach an API route, not merely
    // before they are compared. `select_trusted_run` re-checks the SHA downstream, but that is
    // too late to keep a malformed value out of the URL.
    let merge_base =
        validated_merge_base_sha(require(raw, "--merge-base").map_err(Refused)?).map_err(Refused)?;
    let repo = validated_repo(require(raw, "--repo").map_err(Refused)?)?;

    let runs = gh_api(&format!(
        "repos/{repo}/actions/workflows/oya-ci-required.yml/runs?event=push&branch=dev&status=success&head_sha={merge_base}&per_page=20"
    ))?;
    let run_id = select_trusted_run(&runs, merge_base)?;
    let artifacts = gh_api(&format!(
        "repos/{repo}/actions/runs/{run_id}/artifacts?per_page=100"
    ))?;

    // BOTH halves must validate. A build-only pair is worthless: the FULL tier's merge-base
    // `buck2 test //...` would rebuild the workspace anyway, so a partial reuse saves nothing and
    // would leave the test ratchet without its baseline.
    //
    // STRICTNESS NOTE: a dev tip only publishes these after passing admission, where ANY failure
    // hard-fails — so a reused baseline's failure set is empty and the FULL tier grandfathers
    // nothing. That is the SAFE direction (a reused baseline can never be laxer than a rebuilt
    // one), but it does differ from the cold path, which can observe env-dependent merge-base
    // failures and grandfather them. A PR blocked here is being told the truth: the merge-base
    // was proven green, so the failure is new.
    for kind in [BaselineKind::Build, BaselineKind::Test] {
        // Partial-state cleanup is the caller's single handler — nothing to unwind here.
        fetch_one_baseline(repo, merge_base, run_id, &artifacts, kind, out_dir)?;
    }
    // Record WHICH baseline the FULL tier is about to grandfather against. The affected-set gate
    // reads this sidecar beside the baseline it was handed and stamps it into the operator
    // decision artifact, so "reused vs rebuilt" is a recorded decision, not an inheritance.
    let provenance = serde_json::json!({
        "schema_version": 1,
        "source": "trusted-artifact",
        "merge_base": merge_base,
        "workflow_run_id": run_id,
        "workflow_path": REQUIRED_CONTEXT_WORKFLOW_PATH,
        "grandfathering": "none — the source dev tip passed admission, so its failure set is empty",
    });
    let sidecar = out_dir.join(BASELINE_PROVENANCE_FILENAME);
    let bytes = serde_json::to_vec_pretty(&provenance)
        .map_err(|e| Refused(format!("could not serialize baseline provenance: {e}")))?;
    fs::write(&sidecar, bytes)
        .map_err(|e| Refused(format!("could not write `{}`: {e}", sidecar.display())))?;
    println!(
        "{LOG}: trusted merge-base baseline pair REUSED from run {run_id} at {merge_base} — the \
         cold merge-base rebuild is skipped (provenance: {})",
        sidecar.display()
    );
    Ok(())
}

fn main() -> ExitCode {
    // Early NORMALIZE mode: `--normalize-test-console <console> --normalize-out <report.json>`.
    // Handled before the ratchet arg parse because it needs neither baseline nor head report.
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.iter().any(|a| a == "--trusted-baseline") {
        return ExitCode::from(trusted_baseline_exit(&raw));
    }
    if let Some(idx) = raw.iter().position(|a| a == "--normalize-test-console") {
        let console = raw.get(idx + 1).cloned();
        let out = raw
            .iter()
            .position(|a| a == "--normalize-out")
            .and_then(|i| raw.get(i + 1).cloned());
        match (console, out) {
            (Some(console), Some(out)) => return run_normalize(&console, &out),
            _ => {
                eprintln!(
                    "{LOG}: ARGS ERROR: --normalize-test-console <console> requires --normalize-out <path>"
                );
                return ExitCode::from(2);
            }
        }
    }

    let args = match parse_args(std::env::args()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{LOG}: ARGS ERROR: {e}");
            eprintln!(
                "{LOG}: usage: oya-cloud-ci-build-health --baseline-report <merge-base.json> \
                 --head-report <head.json> [--require-baseline-results]"
            );
            return ExitCode::from(2);
        }
    };

    let baseline_json = match fs::read_to_string(&args.baseline_report) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{LOG}: BASELINE ERROR: cannot read `{}`: {e}",
                args.baseline_report
            );
            return ExitCode::from(2);
        }
    };
    let head_json = match fs::read_to_string(&args.head_report) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{LOG}: HEAD ERROR: cannot read `{}`: {e}", args.head_report);
            return ExitCode::from(2);
        }
    };

    let baseline = match parse_build_report(&baseline_json) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{LOG}: BASELINE PARSE ERROR: {e}");
            return ExitCode::from(2);
        }
    };
    let head = match parse_build_report(&head_json) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{LOG}: HEAD PARSE ERROR: {e}");
            return ExitCode::from(2);
        }
    };

    // Fail-closed laundering guard: an empty baseline `results` would make every head failure
    // look pre-existing. CI builds the whole merge-base workspace, so the baseline is never
    // legitimately empty; refuse rather than silently grandfather everything.
    if args.require_baseline_results && baseline.is_empty() {
        eprintln!(
            "{LOG}: BASELINE EMPTY — the merge-base build-report has no `results`. Refusing to \
             grandfather every head failure against an empty baseline (the laundering hole). \
             Re-run the merge-base `buck2 build //... --keep-going --build-report`."
        );
        return ExitCode::from(2);
    }

    let baseline_failures = failing_targets(&baseline);
    let head_failures = failing_targets(&head);
    let verdict = build_health_verdict(&baseline_failures, &head_failures);

    report(&verdict, baseline.len(), head.len());

    if verdict.is_green() {
        println!(
            "{LOG}: GREEN — no build regressions vs the merge-base ({} pre-existing failure(s) \
             grandfathered).",
            verdict.grandfathered.len()
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "{LOG}: RED — {} build REGRESSION(S) vs the merge-base (target(s) that build at \
             origin/dev but FAIL at this head, or brand-new failing target(s)):",
            verdict.regressions.len()
        );
        for t in &verdict.regressions {
            eprintln!("{LOG}:   REGRESSION {t}");
        }
        eprintln!(
            "{LOG}: REMEDIATION: fix these targets (they compiled at the merge-base), or revert \
             the change that broke them. Pre-existing failures are grandfathered; only NEW build \
             debt blocks."
        );
        ExitCode::from(1)
    }
}

fn report(verdict: &BuildHealthVerdict, baseline_total: usize, head_total: usize) {
    println!(
        "{LOG}: build-health ratchet vs merge-base — baseline targets={baseline_total}, head \
         targets={head_total}"
    );
    println!(
        "{LOG}:   regressions (BLOCK)     = {}",
        verdict.regressions.len()
    );
    println!(
        "{LOG}:   pre-existing (grandfathered, shrink-only) = {}",
        verdict.grandfathered.len()
    );
    for t in &verdict.grandfathered {
        println!("{LOG}:     pre-existing-red {t}");
    }
    if !verdict.fixed.is_empty() {
        println!(
            "{LOG}:   fixed (burned down vs merge-base) = {}",
            verdict.fixed.len()
        );
        for t in &verdict.fixed {
            println!("{LOG}:     fixed {t}");
        }
    }
}

/// Fail-closed seams of the trusted-baseline consumer. These cover the DECISION layer, which is
/// where every acceptance is made; the `gh`/`unzip` subprocess plumbing around it carries no
/// verdict of its own — a non-zero status from either is an unconditional refusal.
#[cfg(test)]
mod trusted_baseline_tests {
    use super::*;

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
    const OTHER_SHA: &str = "fedcba9876543210fedcba9876543210fedcba98";

    fn run_entry(id: u64, sha: &str, event: &str, branch: &str, conclusion: &str) -> String {
        format!(
            r#"{{"id":{id},"head_sha":"{sha}","event":"{event}","head_branch":"{branch}","conclusion":"{conclusion}","path":".github/workflows/oya-ci-required.yml"}}"#
        )
    }

    fn runs(entries: &[String]) -> String {
        format!(r#"{{"workflow_runs":[{}]}}"#, entries.join(","))
    }

    fn is_unavailable(o: &TrustedBaselineOutcome) -> bool {
        matches!(o, Unavailable(_))
    }

    fn is_refused(o: &TrustedBaselineOutcome) -> bool {
        matches!(o, Refused(_))
    }

    #[test]
    fn selects_the_exact_merge_base_dev_push_run() {
        let payload = runs(&[
            run_entry(11, OTHER_SHA, "push", "dev", "success"),
            run_entry(13, SHA, "push", "dev", "success"),
        ]);
        assert_eq!(select_trusted_run(&payload, SHA), Ok(13));
    }

    #[test]
    fn wrong_sha_wrong_event_wrong_branch_and_red_runs_are_all_unavailable() {
        // Each entry is a distinct rejected provenance shape; none may be selected.
        let payload = runs(&[
            run_entry(11, OTHER_SHA, "push", "dev", "success"),
            run_entry(12, SHA, "pull_request", "dev", "success"),
            run_entry(13, SHA, "push", "feature", "success"),
            run_entry(14, SHA, "push", "dev", "failure"),
        ]);
        let err = select_trusted_run(&payload, SHA).unwrap_err();
        assert!(is_unavailable(&err), "{err:?}");
    }

    #[test]
    fn a_foreign_workflow_at_the_exact_merge_base_is_never_trusted() {
        // DEFENCE IN DEPTH: `reuse_trusted_baselines` queries the per-workflow runs route and
        // reads artifacts per-run, so a foreign workflow's run is not reachable today. Pinned
        // anyway so widening that route to the repo-wide run list cannot silently reduce
        // selection to the artifact name alone.
        let payload = format!(
            r#"{{"workflow_runs":[{{"id":14,"head_sha":"{SHA}","event":"push","head_branch":"dev","conclusion":"success","path":".github/workflows/some-other-lane.yml"}}]}}"#
        );
        let err = select_trusted_run(&payload, SHA).unwrap_err();
        assert!(is_unavailable(&err), "{err:?}");
    }

    #[test]
    fn malformed_run_payloads_and_bad_sha_shapes_are_refused() {
        for payload in ["not json at all", r#"{"ok":true}"#, ""] {
            let err = select_trusted_run(payload, SHA).unwrap_err();
            assert!(is_refused(&err), "payload {payload:?} -> {err:?}");
        }
        // An abbreviated SHA could resolve differently across runs — never a baseline key.
        let err = select_trusted_run(r#"{"workflow_runs":[]}"#, "0123456").unwrap_err();
        assert!(is_refused(&err), "{err:?}");
    }

    #[test]
    fn artifact_selection_is_kind_scoped_and_ignores_other_commits() {
        let artifacts = format!(
            r#"{{"artifacts":[
                {{"id":21,"name":"build-health-baseline-{OTHER_SHA}","expired":false}},
                {{"id":22,"name":"build-health-baseline-{SHA}","expired":false}},
                {{"id":23,"name":"test-health-baseline-{SHA}","expired":false}}
            ]}}"#
        );
        assert_eq!(
            select_trusted_artifact(&artifacts, BaselineKind::Build, SHA),
            Ok((22, format!("build-health-baseline-{SHA}")))
        );
        assert_eq!(
            select_trusted_artifact(&artifacts, BaselineKind::Test, SHA),
            Ok((23, format!("test-health-baseline-{SHA}")))
        );
    }

    #[test]
    fn expired_or_absent_artifacts_are_unavailable() {
        let expired = format!(
            r#"{{"artifacts":[{{"id":22,"name":"build-health-baseline-{SHA}","expired":true}}]}}"#
        );
        let err = select_trusted_artifact(&expired, BaselineKind::Build, SHA).unwrap_err();
        assert!(is_unavailable(&err), "expired must not be reused: {err:?}");

        // The BUILD half exists but the TEST half was never published (the pre-#1323 state): the
        // pair is incomplete, so the FULL tier must still rebuild cold.
        let build_only = format!(
            r#"{{"artifacts":[{{"id":22,"name":"build-health-baseline-{SHA}","expired":false}}]}}"#
        );
        let err = select_trusted_artifact(&build_only, BaselineKind::Test, SHA).unwrap_err();
        assert!(is_unavailable(&err), "missing test half: {err:?}");

        let err = select_trusted_artifact(r#"{"artifacts":[]}"#, BaselineKind::Build, SHA)
            .unwrap_err();
        assert!(is_unavailable(&err), "{err:?}");
    }

    #[test]
    fn malformed_artifact_payloads_are_refused() {
        for payload in ["not json", r#"{"ok":true}"#] {
            let err = select_trusted_artifact(payload, BaselineKind::Build, SHA).unwrap_err();
            assert!(is_refused(&err), "payload {payload:?} -> {err:?}");
        }
    }

    #[test]
    fn a_well_formed_exact_name_report_is_accepted() {
        let report = r#"{"results":{"root//a:a":{"success":"SUCCESS"},"root//b:b":{"success":"FAIL"}}}"#;
        let name = format!("build-health-baseline-{SHA}");
        assert_eq!(
            accept_downloaded_baseline(BaselineKind::Build, &name, SHA, report),
            Ok(2)
        );
    }

    #[test]
    fn downloads_that_do_not_match_the_selection_are_refused() {
        let report = r#"{"results":{"root//a:a":{"success":"SUCCESS"}}}"#;

        // A name for a DIFFERENT commit: what arrived is not what was selected.
        let stale = format!("build-health-baseline-{OTHER_SHA}");
        let err =
            accept_downloaded_baseline(BaselineKind::Build, &stale, SHA, report).unwrap_err();
        assert!(is_refused(&err), "stale name: {err:?}");

        // A build-named artifact must never satisfy the TEST baseline slot.
        let build_name = format!("build-health-baseline-{SHA}");
        let err =
            accept_downloaded_baseline(BaselineKind::Test, &build_name, SHA, report).unwrap_err();
        assert!(is_refused(&err), "kind confusion: {err:?}");
    }

    #[test]
    fn malformed_truncated_and_empty_reports_are_refused() {
        let name = format!("build-health-baseline-{SHA}");
        // `<html>` stands in for an error page landing where a zip entry was expected; the empty
        // string for a download that produced no bytes; `{"results":{}}` for the laundering hole
        // an empty baseline would open (every head failure would look pre-existing).
        for payload in [
            "<html>404</html>",
            "",
            r#"{"ok":true}"#,
            r#"{"results":{}}"#,
            r#"{"results":{"root//a:a":{"success":"SUCCESS"}"#,
        ] {
            let err =
                accept_downloaded_baseline(BaselineKind::Build, &name, SHA, payload).unwrap_err();
            assert!(is_refused(&err), "payload {payload:?} -> {err:?}");
        }
    }

    #[test]
    fn repository_slugs_are_validated_before_they_reach_an_api_route() {
        assert_eq!(validated_repo("oyatie/oyatie"), Ok("oyatie/oyatie"));
        assert_eq!(validated_repo("owner-1/repo_name.rs"), Ok("owner-1/repo_name.rs"));
        for bad in [
            "",
            "noslash",
            "owner/",
            "/repo",
            "owner/repo/extra",
            "owner/repo?per_page=1",
            "owner/repo#frag",
            "owner/re po",
        ] {
            assert!(validated_repo(bad).is_err(), "`{bad}` must be refused");
        }
    }

    #[test]
    fn missing_arguments_exit_non_zero_so_the_workflow_falls_back() {
        // The fail-closed contract the workflow depends on: anything short of a fully validated
        // pair exits non-zero, and the cold rebuild runs.
        let argv: Vec<String> = ["--trusted-baseline", "--merge-base", SHA]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        assert_eq!(trusted_baseline_exit(&argv), 2);
        assert_eq!(trusted_baseline_exit(&[]), 2);
    }

    #[test]
    fn unavailability_and_refusal_map_to_distinct_non_zero_exits() {
        assert_ne!(NO_TRUSTED_BASELINE, 0);
        assert_ne!(NO_TRUSTED_BASELINE, 2);
    }
}

