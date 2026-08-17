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
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use ci_affected_target_set::{
    BASELINE_PROVENANCE_FILENAME, BASELINE_REUSE_OUTCOME_FILENAME, BaselineKind,
    BaselineReuseState, BuildHealthVerdict, REQUIRED_CONTEXT_WORKFLOW_PATH,
    TrustedBaselineArtifact, TrustedProducerJob, TrustedWorkflowRun, baseline_artifact_name,
    build_health_verdict, classify_api_status, failing_targets, parse_build_report,
    parse_test_verdicts, test_verdicts_to_report_value, trusted_affected_set_producer_job,
    trusted_baseline_artifact, trusted_dev_push_run, validate_trusted_baseline_artifact,
    validated_merge_base_sha,
};
use sha2::{Digest, Sha256};

const LOG: &str = "build-health";

/// GitHub REST base. Fixed rather than configurable: the trusted baseline is only meaningful for
/// the repository whose `oya-ci-required` runs produced it, and a settable API host would be a way
/// to point provenance validation at a server the candidate controls.
const GITHUB_API: &str = "https://api.github.com";
/// GitHub rejects API requests without a User-Agent.
const HTTP_USER_AGENT: &str = "oya-cloud-ci-build-health";
/// GitHub's pinned REST version; unversioned requests can change shape under us.
const GITHUB_API_VERSION: &str = "2026-03-10";
/// Per-request ceiling. The fast path exists to save ~12 minutes; a request that hangs longer than
/// this has already lost to the cold rebuild it was avoiding.
const HTTP_TIMEOUT: Duration = Duration::from_secs(60);

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
/// cold in a clean worktree is the FULL tier's wall-clock bottleneck. When the published pair
/// validates, we download it and the cold rebuild is skipped entirely.
///
/// WHAT IT IS WORTH, measured rather than asserted. Across the last 60 `oya-ci-required` PR runs on
/// the owned arm64 fleet, the merge-base materialization step is sharply bimodal: 13 non-FULL runs
/// exit the step in 1-2s, and 12 successful FULL runs spend 672s..1068s in it (mean 12m50s, median
/// 12m33s) — a cold `buck2 build //...` plus `buck2 test //...` of the whole workspace. The fast
/// path replaces all of it with four REST calls and two zip inflations: measured END TO END at 2s
/// against a real merge-base, yielding a validated 2199-target build baseline and a 1117-target
/// test baseline. FULL was derived on 14 of 27 PR runs that reached this step, so this is not a
/// rare path. The saving is only realised when the merge-base dev tip is green and its artifacts
/// are unexpired; a red dev tip legitimately reports `Unavailable` and pays the rebuild.
///
/// OWNED-RUST, NOT WORKFLOW SHELL: the whole sequence — list runs, select the trusted one, select
/// each artifact, download, unzip, validate — lives here rather than in the workflow's `run:`
/// block. Keeping it here also keeps the rust-first inline-shell ratchet honest: the workflow grows
/// a handful of lines instead of ~35.
///
/// NOT `gh` (the fix for GH job 91383250718). This previously drove `gh api` as a subprocess on the
/// premise that `gh` was "already installed". It is not: the owned arm64 runner image
/// (infra/ci/runner-image/Dockerfile) never installs it, and a probe of the live digest-pinned
/// image finds no file named `gh` anywhere on the rootfs while curl/unzip/git/buck2/rustc/pwsh/jq
/// are all present. Every FULL-tier run on the owned fleet therefore failed at the first API call
/// with `No such file or directory (os error 2)` and paid the cold merge-base rebuild. The REST
/// calls now go over `reqwest::blocking` — the same choice, for the same reason, as the sibling
/// `oya-cloud-ci-run-terminal-state` consumer. `curl` was rejected despite being present: it would
/// keep a CLI dependency the repo is retiring, and it puts the bearer token in the process table.
///
/// `unzip` REMAINS a subprocess and is deliberately untouched: it IS present in the image (probed,
/// not assumed), the artifact API serves only a zip, and inflating one in-process would mean a new
/// dependency for something already installed.
///
/// ANTI-LAUNDERING (the D6 guarantees, preserved and tightened): a baseline is accepted ONLY from
/// a GitHub Actions run whose PROVENANCE proves the candidate could not have produced it —
/// `event=push`, `head_branch=dev`, `status=completed`, `path` = the single required-context
/// workflow file, and `head_sha` EXACTLY the merge-base. The run's paginated jobs must contain
/// exactly one exact-name affected-set producer with `status=completed` and
/// `conclusion=success`; aggregate run failure is allowed because unrelated lanes do not produce
/// these artifacts. The artifact NAME is never sufficient on its own; it is additionally checked
/// against the name the API reports for the id actually downloaded, and the payload must parse to
/// a non-empty build-report `results` map. ANY doubt at ANY step exits non-zero with the partial
/// outputs deleted, and the workflow then runs the untouched cold rebuild.
///
/// Usage: `--trusted-baseline --merge-base <sha> --repo <owner/name> --out-dir <dir>`.
fn trusted_baseline_exit(raw: &[String]) -> u8 {
    let (mut state, mut why) = match reuse_trusted_baselines(raw) {
        Ok(()) => (
            BaselineReuseState::Reused,
            "validated merge-base baseline pair downloaded".to_owned(),
        ),
        Err(outcome) => (outcome.state(), outcome.into_reason()),
    };

    // REPORT BEFORE RETURNING, on every path. The stderr line below always existed; what did not
    // was any record a machine could query, which is why a dead capability survived unnoticed for
    // the whole life of the owned fleet. `--out-dir` is validated inside `reuse_trusted_baselines`,
    // so re-reading it here without validation is only ever used as a directory to write into.
    if let Some(out_dir) = flag_value(raw, "--out-dir")
        && let Err(error) = report_reuse_outcome(
            Path::new(out_dir),
            flag_value(raw, "--merge-base").unwrap_or("<unparsed>"),
            state,
            &why,
        )
    {
        cleanup_baseline_outputs(Path::new(out_dir));
        state = BaselineReuseState::Refused;
        why = format!("outcome telemetry could not be persisted: {error}");
        println!("::error title=Trusted-baseline telemetry failed::{why}");
    }

    match state {
        BaselineReuseState::Reused => {}
        BaselineReuseState::Unavailable => eprintln!(
            "{LOG}: NO TRUSTED BASELINE — {why}; the cold merge-base rebuild runs instead"
        ),
        BaselineReuseState::Refused => eprintln!(
            "{LOG}: TRUSTED BASELINE REFUSED — {why}; the cold merge-base rebuild runs instead"
        ),
        BaselineReuseState::CapabilityFault => eprintln!(
            "{LOG}: TRUSTED BASELINE CAPABILITY FAULT — {why}. This is an INFRASTRUCTURE defect, \
             not a property of this PR: the fast path is dark and every FULL-tier run pays the \
             cold merge-base rebuild until it is fixed."
        ),
    }
    state.exit_code()
}

/// Write the typed fast-path outcome everywhere it can be noticed: a machine-readable sidecar the
/// affected-set gate folds into its operator artifact, the job step summary, and — for a capability
/// fault/refusal — a GitHub annotation. The machine sidecar is mandatory: reuse without a durable
/// outcome is refused and the cold fallback runs.
fn report_reuse_outcome(
    out_dir: &Path,
    merge_base: &str,
    state: BaselineReuseState,
    why: &str,
) -> Result<(), String> {
    let outcome = serde_json::json!({
        "schema_version": 2,
        "state": state.as_str(),
        "capability_fault": state.is_capability_fault(),
        "reason": why,
        "merge_base": merge_base,
        "cold_fallback_required": state != BaselineReuseState::Reused,
        "exit_code": state.exit_code(),
        "telemetry_persisted": true,
    });
    let bytes =
        serde_json::to_vec_pretty(&outcome).map_err(|e| format!("serialize outcome: {e}"))?;
    let path = out_dir.join(BASELINE_REUSE_OUTCOME_FILENAME);
    let temporary = out_dir.join(format!(".{BASELINE_REUSE_OUTCOME_FILENAME}.tmp"));
    fs::write(&temporary, bytes).map_err(|e| format!("write `{}`: {e}", temporary.display()))?;
    fs::rename(&temporary, &path).map_err(|e| format!("publish `{}`: {e}", path.display()))?;

    match state {
        BaselineReuseState::CapabilityFault => {
            println!("::warning title=Trusted-baseline fast path is dark::{why}");
        }
        BaselineReuseState::Refused => {
            println!("::warning title=Trusted-baseline provenance refused::{why}");
        }
        BaselineReuseState::Unavailable => {
            println!("::notice title=Trusted baseline unavailable::{why}");
        }
        BaselineReuseState::Reused => {}
    }
    if let Ok(summary) = std::env::var("GITHUB_STEP_SUMMARY")
        && let Ok(mut file) = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(summary)
    {
        let _ = writeln!(
            file,
            "- merge-base baseline reuse: **{}** — {why}",
            state.as_str()
        );
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum TrustedBaselineOutcome {
    /// No baseline exists for this merge-base (normal: first push, expired retention, red dev).
    Unavailable(String),
    /// A baseline exists but failed a provenance/shape check, or an input was bad.
    Refused(String),
    /// The runner could not ASK: a required executable is absent, no token is present, or the API
    /// was unreachable. Split out of `Refused` because folding the two together is exactly what
    /// hid `gh`'s absence — a broken environment reported identically to a working one that had
    /// nothing to offer.
    CapabilityFault(String),
}

impl TrustedBaselineOutcome {
    fn state(&self) -> BaselineReuseState {
        match self {
            Self::Unavailable(_) => BaselineReuseState::Unavailable,
            Self::Refused(_) => BaselineReuseState::Refused,
            Self::CapabilityFault(_) => BaselineReuseState::CapabilityFault,
        }
    }

    fn into_reason(self) -> String {
        match self {
            Self::Unavailable(why) | Self::Refused(why) | Self::CapabilityFault(why) => why,
        }
    }
}

use TrustedBaselineOutcome::{CapabilityFault, Refused, Unavailable};

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
        Err(Refused(format!(
            "`{repo}` is not a valid <owner>/<name> repository"
        )))
    }
}

/// PURE: which workflow run may be trusted for `merge_base`, given the runs payload.
fn select_trusted_run(
    runs_json: &str,
    merge_base: &str,
) -> Result<TrustedWorkflowRun, TrustedBaselineOutcome> {
    match trusted_dev_push_run(runs_json, merge_base, REQUIRED_CONTEXT_WORKFLOW_PATH)
        .map_err(Refused)?
    {
        Some(id) => Ok(id),
        None => Err(Unavailable(format!(
            "no completed push-to-dev `{REQUIRED_CONTEXT_WORKFLOW_PATH}` run at merge-base {merge_base}"
        ))),
    }
}

/// PURE: require the unique exact producer job to be terminal green.
fn select_trusted_producer(
    jobs_json: &str,
    run: &TrustedWorkflowRun,
) -> Result<TrustedProducerJob, TrustedBaselineOutcome> {
    match trusted_affected_set_producer_job(jobs_json, run.id, &run.head_sha).map_err(Refused)? {
        Some(job) => Ok(job),
        None => Err(Unavailable(
            "the completed run has no unique exact affected-set producer job with conclusion=success"
                .to_owned(),
        )),
    }
}

/// PURE: which artifact of a trusted run carries the `kind` baseline for `merge_base`.
fn select_trusted_artifact(
    artifacts_json: &str,
    kind: BaselineKind,
    run: &TrustedWorkflowRun,
) -> Result<TrustedBaselineArtifact, TrustedBaselineOutcome> {
    let name = baseline_artifact_name(kind, &run.head_sha, run.id, run.attempt).map_err(Refused)?;
    match trusted_baseline_artifact(artifacts_json, &name, run.id, &run.head_sha)
        .map_err(Refused)?
    {
        Some(artifact) => Ok(artifact),
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
    run: &TrustedWorkflowRun,
    report_json: &str,
) -> Result<usize, TrustedBaselineOutcome> {
    validate_trusted_baseline_artifact(
        kind,
        artifact_name,
        &run.head_sha,
        run.id,
        run.attempt,
        report_json,
    )
    .map_err(Refused)
}

/// Owned GitHub REST adapter. Carries no verdict of its own: it returns bytes, and the pure
/// selector/validator functions decide whether those bytes may become a baseline.
struct GitHubApi {
    client: reqwest::blocking::Client,
    token: String,
}

impl GitHubApi {
    /// `GH_TOKEN` first (what the workflow sets and what `gh` read), then `GITHUB_TOKEN`.
    ///
    /// An absent or blank token is a CAPABILITY FAULT, not a refusal: it means this runner cannot
    /// ask the question at all, which is the same class of defect as the missing binary this
    /// adapter replaces.
    fn from_env() -> Result<Self, TrustedBaselineOutcome> {
        let token = ["GH_TOKEN", "GITHUB_TOKEN"]
            .iter()
            .find_map(|name| std::env::var(name).ok())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                CapabilityFault(
                    "neither GH_TOKEN nor GITHUB_TOKEN is set to a non-empty value, so the \
                     Actions API cannot be queried"
                        .to_owned(),
                )
            })?;
        let client = reqwest::blocking::Client::builder()
            .user_agent(HTTP_USER_AGENT)
            .timeout(HTTP_TIMEOUT)
            .build()
            .map_err(|e| CapabilityFault(format!("could not build the GitHub HTTP client: {e}")))?;
        Ok(Self { client, token })
    }

    /// GET `<api>/<route>` with the status classified into a typed degrade.
    ///
    /// Redirects are followed by the default policy, which is load-bearing for the artifact `zip`
    /// route: it 302s to a signed blob host, and reqwest strips `Authorization` on a cross-host
    /// redirect — required, because that host rejects a request carrying two auth mechanisms.
    fn get(&self, route: &str) -> Result<reqwest::blocking::Response, TrustedBaselineOutcome> {
        let response = self
            .client
            .get(format!("{GITHUB_API}/{route}"))
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .map_err(|e| {
                CapabilityFault(format!("GET {route}: the GitHub API is unreachable: {e}"))
            })?;
        let status = response.status().as_u16();
        match classify_api_status(status) {
            None => Ok(response),
            Some(BaselineReuseState::Unavailable) => {
                Err(Unavailable(format!("GET {route}: HTTP {status}")))
            }
            Some(BaselineReuseState::CapabilityFault) => Err(CapabilityFault(format!(
                "GET {route}: HTTP {status} — the runner cannot authenticate to or reach the \
                 GitHub Actions API"
            ))),
            Some(_) => Err(Refused(format!("GET {route}: unexpected HTTP {status}"))),
        }
    }

    /// GET `route` and return the body as text. Never a silent empty payload: a body that cannot
    /// be read is a fault, and the callers' pure validators reject anything that is not the
    /// expected JSON shape.
    fn get_text(&self, route: &str) -> Result<String, TrustedBaselineOutcome> {
        self.get(route)?
            .text()
            .map_err(|e| CapabilityFault(format!("GET {route}: could not read the body: {e}")))
    }

    /// Fetch every page from the Actions jobs endpoint and return one selector payload.
    ///
    /// The exact producer can appear after page one on a wide workflow. Stopping at the first
    /// hundred jobs would turn ordering into provenance, so pagination is mandatory. The hard
    /// ceiling prevents a broken server from making the consumer loop forever; reaching it while
    /// pages remain full is a refusal and therefore falls back to the cold rebuild.
    fn get_paginated_jobs(
        &self,
        repo: &str,
        run: &TrustedWorkflowRun,
    ) -> Result<String, TrustedBaselineOutcome> {
        let mut jobs = Vec::new();
        for page in 1..=100 {
            let route = format!(
                "repos/{repo}/actions/runs/{}/attempts/{}/jobs?per_page=100&page={page}",
                run.id, run.attempt
            );
            let body = self.get_text(&route)?;
            let payload: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
                Refused(format!("workflow-jobs page {page} is not valid JSON: {e}"))
            })?;
            let batch = payload
                .get("jobs")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| Refused(format!("workflow-jobs page {page} has no `jobs` array")))?;
            let count = batch.len();
            jobs.extend(batch.iter().cloned());
            if count < 100 {
                return serde_json::to_string(&serde_json::json!({ "jobs": jobs })).map_err(|e| {
                    Refused(format!("could not assemble workflow-jobs payload: {e}"))
                });
            }
        }
        Err(Refused(
            "workflow-jobs pagination exceeded 100 full pages; refusing a potentially truncated producer set"
                .to_owned(),
        ))
    }

    /// Stream `route` straight to `dest` — artifact zips are binary and do not belong in memory.
    fn download(&self, route: &str, dest: &Path) -> Result<(), TrustedBaselineOutcome> {
        let mut response = self.get(route)?;
        let mut file = fs::File::create(dest)
            .map_err(|e| Refused(format!("could not create `{}`: {e}", dest.display())))?;
        std::io::copy(&mut response, &mut file)
            .map_err(|e| CapabilityFault(format!("GET {route}: download interrupted: {e}")))?;
        Ok(())
    }
}

fn sha256_file(path: &Path) -> Result<String, TrustedBaselineOutcome> {
    let bytes =
        fs::read(path).map_err(|e| Refused(format!("cannot hash `{}`: {e}", path.display())))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

/// Stream a subprocess's stdout straight to `dest` (`unzip -p` output can be large and does not
/// belong in memory).
///
/// THE TWO FAILURE SHAPES ARE NOT THE SAME DEFECT, and conflating them is the bug this change
/// exists to kill. "Could not SPAWN it" means the runner image lacks the tool — an environment
/// fault that will recur on every single run until someone changes the image. "It ran and exited
/// non-zero" is a statement about the DATA (a truncated or corrupt zip) and is a refusal. `gh`
/// was the site that got noticed; `unzip` reaches this same code path and had the same
/// mis-classification, so it is fixed here rather than at the one caller that was reported.
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
        .map_err(|e| {
            CapabilityFault(format!(
                "could not execute `{program}`: {e} — the runner image does not provide it"
            ))
        })?;
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
    api: &GitHubApi,
    repo: &str,
    run: &TrustedWorkflowRun,
    artifacts_json: &str,
    kind: BaselineKind,
    out_dir: &Path,
) -> Result<TrustedBaselineArtifact, TrustedBaselineOutcome> {
    let artifact = select_trusted_artifact(artifacts_json, kind, run)?;
    // The name the API reports for the id we are about to download closes the "selected one
    // artifact, fetched another" loop against a live server response.
    let meta = api.get_text(&format!("repos/{repo}/actions/artifacts/{}", artifact.id))?;
    let wrapped_meta = format!(r#"{{"artifacts":[{meta}]}}"#);
    let metadata_artifact =
        trusted_baseline_artifact(&wrapped_meta, &artifact.name, run.id, &run.head_sha)
            .map_err(Refused)?
            .ok_or_else(|| {
                Refused(format!(
                    "artifact {} metadata reports it unavailable",
                    artifact.id
                ))
            })?;
    if metadata_artifact != artifact {
        return Err(Refused(format!(
            "artifact {} list metadata does not match its by-id metadata",
            artifact.id
        )));
    }

    let zip = out_dir.join(format!("trusted-{}-health-baseline.zip", kind.prefix()));
    api.download(
        &format!("repos/{repo}/actions/artifacts/{}/zip", artifact.id),
        &zip,
    )?;
    let downloaded_digest = sha256_file(&zip)?;
    if downloaded_digest != artifact.digest {
        return Err(Refused(format!(
            "artifact {} digest mismatch: API={}, downloaded={downloaded_digest}",
            artifact.id, artifact.digest
        )));
    }
    // The artifact API serves ONLY a zip and this crate takes no new dependency, so extraction is
    // delegated to `unzip`. Declared, with the full justification and cutover, as a path- and
    // count-scoped hermetic exception in ci/facade/gate-self-conformance/gate-self-conformance-policy.json
    // (gate `affected-target-set`, token `Command::new`) — that row is the single source of truth.
    // Operationally: a missing or failing `unzip` refuses here, so the pair is abandoned and the
    // cold rebuild runs — absence costs wall-clock, never correctness.
    let report = out_dir.join(format!("{}-health-baseline.json", kind.prefix()));
    capture_to_file("unzip", &["-p", &zip.display().to_string()], &report)?;
    let _ = fs::remove_file(&zip);

    let payload = fs::read_to_string(&report)
        .map_err(|e| Refused(format!("cannot read `{}`: {e}", report.display())))?;
    let count = accept_downloaded_baseline(kind, &artifact.name, run, &payload)?;
    println!(
        "{LOG}: trusted {} baseline VALID — {count} target result(s) from `{}` \
         (artifact {} digest {} of run {} attempt {}) -> {}",
        kind.prefix(),
        artifact.name,
        artifact.id,
        artifact.digest,
        run.id,
        run.attempt,
        report.display()
    );
    Ok(artifact)
}

fn cleanup_baseline_outputs(out_dir: &Path) {
    for kind in [BaselineKind::Build, BaselineKind::Test] {
        let _ = fs::remove_file(out_dir.join(format!("{}-health-baseline.json", kind.prefix())));
    }
    let _ = fs::remove_file(out_dir.join(BASELINE_PROVENANCE_FILENAME));
}

fn reuse_trusted_baselines(raw: &[String]) -> Result<(), TrustedBaselineOutcome> {
    let out_dir = PathBuf::from(require(raw, "--out-dir").map_err(Refused)?);
    let outcome = reuse_trusted_baselines_inner(raw, &out_dir);
    if outcome.is_err() {
        cleanup_baseline_outputs(&out_dir);
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
    let merge_base = validated_merge_base_sha(require(raw, "--merge-base").map_err(Refused)?)
        .map_err(Refused)?;
    let repo = validated_repo(require(raw, "--repo").map_err(Refused)?)?;
    let api = GitHubApi::from_env()?;

    let runs = api.get_text(&format!(
        "repos/{repo}/actions/workflows/oya-ci-required.yml/runs?event=push&branch=dev&status=completed&head_sha={merge_base}&per_page=20"
    ))?;
    let run = select_trusted_run(&runs, merge_base)?;
    let jobs = api.get_paginated_jobs(repo, &run)?;
    let producer = select_trusted_producer(&jobs, &run)?;
    let artifacts = api.get_text(&format!(
        "repos/{repo}/actions/runs/{}/artifacts?per_page=100",
        run.id
    ))?;

    // BOTH halves must validate. A build-only pair is worthless: the FULL tier's merge-base
    // `buck2 test //...` would rebuild the workspace anyway, so a partial reuse saves nothing and
    // would leave the test ratchet without its baseline.
    //
    // STRICTNESS NOTE: the exact affected-set producer publishes these only after its own binding
    // build/test passes, so a reused baseline's failure set is empty and the FULL tier
    // grandfathers nothing. Other jobs may make the aggregate run red without changing that fact.
    // This is the SAFE direction (a reused baseline can never be laxer than a rebuilt one), but it
    // does differ from the cold path, which can observe env-dependent merge-base failures and
    // grandfather them. A PR blocked here is being told the truth: the producer proved this
    // merge-base green, so the failure is new.
    let mut receipts = Vec::new();
    for kind in [BaselineKind::Build, BaselineKind::Test] {
        // Partial-state cleanup is the caller's single handler — nothing to unwind here.
        let artifact = fetch_one_baseline(&api, repo, &run, &artifacts, kind, out_dir)?;
        receipts.push(serde_json::json!({
            "kind": kind.prefix(),
            "artifact_id": artifact.id,
            "artifact_name": artifact.name,
            "artifact_digest": artifact.digest,
            "size_in_bytes": artifact.size_in_bytes,
        }));
    }
    // Record WHICH baseline the FULL tier is about to grandfather against. The affected-set gate
    // reads this sidecar beside the baseline it was handed and stamps it into the operator
    // decision artifact, so "reused vs rebuilt" is a recorded decision, not an inheritance.
    let provenance = serde_json::json!({
        "schema_version": 3,
        "source": "trusted-artifact",
        "merge_base": merge_base,
        "workflow_run_id": run.id,
        "workflow_run_attempt": run.attempt,
        "workflow_path": REQUIRED_CONTEXT_WORKFLOW_PATH,
        "producer_job_id": producer.id,
        "producer_run_id": producer.run_id,
        "producer_head_sha": producer.head_sha,
        "producer_job_conclusion": producer.conclusion,
        "artifacts": receipts,
        "grandfathering": "none — the exact affected-set producer passed, so its failure set is empty",
    });
    let sidecar = out_dir.join(BASELINE_PROVENANCE_FILENAME);
    let bytes = serde_json::to_vec_pretty(&provenance)
        .map_err(|e| Refused(format!("could not serialize baseline provenance: {e}")))?;
    fs::write(&sidecar, bytes)
        .map_err(|e| Refused(format!("could not write `{}`: {e}", sidecar.display())))?;
    println!(
        "{LOG}: trusted merge-base baseline pair REUSED from run {} attempt {}, producer job {} at \
         {merge_base} — the \
         cold merge-base rebuild is skipped (provenance: {})",
        run.id,
        run.attempt,
        producer.id,
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
    const RUN_ID: u64 = 13;
    const RUN_ATTEMPT: u64 = 2;
    const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn run_entry(id: u64, sha: &str, event: &str, branch: &str, conclusion: &str) -> String {
        format!(
            r#"{{"id":{id},"run_attempt":{RUN_ATTEMPT},"head_sha":"{sha}","event":"{event}","head_branch":"{branch}","status":"completed","conclusion":"{conclusion}","path":".github/workflows/oya-ci-required.yml"}}"#
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

    fn trusted_run() -> TrustedWorkflowRun {
        TrustedWorkflowRun {
            id: RUN_ID,
            attempt: RUN_ATTEMPT,
            head_sha: SHA.to_owned(),
        }
    }

    fn artifact_payload(kind: BaselineKind, expired: bool) -> String {
        let name = baseline_artifact_name(kind, SHA, RUN_ID, RUN_ATTEMPT).unwrap();
        format!(
            r#"{{"artifacts":[{{
            "id":22,"name":"{name}","expired":{expired},"digest":"{DIGEST}",
            "size_in_bytes":42,"workflow_run":{{"id":{RUN_ID},"head_sha":"{SHA}"}}
        }}]}}"#
        )
    }

    #[test]
    fn selects_the_exact_merge_base_dev_push_run() {
        let payload = runs(&[
            run_entry(11, OTHER_SHA, "push", "dev", "success"),
            run_entry(RUN_ID, SHA, "push", "dev", "failure"),
        ]);
        assert_eq!(select_trusted_run(&payload, SHA), Ok(trusted_run()));
    }

    #[test]
    fn wrong_sha_event_branch_status_or_foreign_workflow_is_unavailable() {
        let payload = runs(&[
            run_entry(11, OTHER_SHA, "push", "dev", "success"),
            run_entry(12, SHA, "pull_request", "dev", "success"),
            run_entry(13, SHA, "push", "feature", "success"),
            format!(
                r#"{{"id":14,"run_attempt":1,"head_sha":"{SHA}","event":"push","head_branch":"dev","status":"in_progress","conclusion":null,"path":".github/workflows/oya-ci-required.yml"}}"#
            ),
            format!(
                r#"{{"id":15,"run_attempt":1,"head_sha":"{SHA}","event":"push","head_branch":"dev","status":"completed","conclusion":"success","path":".github/workflows/some-other-lane.yml"}}"#
            ),
        ]);
        let err = select_trusted_run(&payload, SHA).unwrap_err();
        assert!(is_unavailable(&err), "{err:?}");
    }

    #[test]
    fn duplicate_or_malformed_runs_are_refused() {
        let duplicate = runs(&[
            run_entry(11, SHA, "push", "dev", "success"),
            run_entry(12, SHA, "push", "dev", "failure"),
        ]);
        assert!(is_refused(
            &select_trusted_run(&duplicate, SHA).unwrap_err()
        ));
        let malformed = format!(
            r#"{{"workflow_runs":[{{"id":14,"head_sha":"{SHA}","event":"push","head_branch":"dev","status":"completed","conclusion":"success","path":".github/workflows/oya-ci-required.yml"}}]}}"#
        );
        assert!(is_refused(
            &select_trusted_run(&malformed, SHA).unwrap_err()
        ));
        for payload in ["not json at all", r#"{"ok":true}"#, ""] {
            assert!(is_refused(&select_trusted_run(payload, SHA).unwrap_err()));
        }
    }

    #[test]
    fn producer_selection_requires_unique_exact_terminal_green_bound_job() {
        let exact = ci_affected_target_set::AFFECTED_SET_PRODUCER_JOB_NAME;
        let valid = format!(
            r#"{{"jobs":[{{"id":81,"run_id":{RUN_ID},"head_sha":"{SHA}","name":"{exact}","status":"completed","conclusion":"success"}}]}}"#
        );
        assert_eq!(
            select_trusted_producer(&valid, &trusted_run()),
            Ok(TrustedProducerJob {
                id: 81,
                run_id: RUN_ID,
                head_sha: SHA.to_owned(),
                conclusion: "success".to_owned(),
            })
        );
        let duplicate = format!(
            r#"{{"jobs":[
            {{"id":81,"run_id":{RUN_ID},"head_sha":"{SHA}","name":"{exact}","status":"completed","conclusion":"success"}},
            {{"id":82,"run_id":{RUN_ID},"head_sha":"{SHA}","name":"{exact}","status":"completed","conclusion":"success"}}
        ]}}"#
        );
        assert!(is_refused(
            &select_trusted_producer(&duplicate, &trusted_run()).unwrap_err()
        ));
        let wrong_run = valid.replace(&format!(r#""run_id":{RUN_ID}"#), r#""run_id":99"#);
        assert!(is_refused(
            &select_trusted_producer(&wrong_run, &trusted_run()).unwrap_err()
        ));
    }

    #[test]
    fn artifact_selection_is_unique_kind_and_run_bound() {
        let build = artifact_payload(BaselineKind::Build, false);
        assert_eq!(
            select_trusted_artifact(&build, BaselineKind::Build, &trusted_run()),
            Ok(TrustedBaselineArtifact {
                id: 22,
                name: baseline_artifact_name(BaselineKind::Build, SHA, RUN_ID, RUN_ATTEMPT)
                    .unwrap(),
                digest: DIGEST.to_owned(),
                size_in_bytes: 42,
                workflow_run_id: RUN_ID,
                head_sha: SHA.to_owned(),
            })
        );
        assert!(is_unavailable(
            &select_trusted_artifact(&build, BaselineKind::Test, &trusted_run()).unwrap_err()
        ));
        assert!(is_unavailable(
            &select_trusted_artifact(
                &artifact_payload(BaselineKind::Build, true),
                BaselineKind::Build,
                &trusted_run()
            )
            .unwrap_err()
        ));
        for payload in ["not json", r#"{"ok":true}"#] {
            assert!(is_refused(
                &select_trusted_artifact(payload, BaselineKind::Build, &trusted_run()).unwrap_err()
            ));
        }
    }

    #[test]
    fn downloaded_reports_are_exactly_name_and_payload_bound() {
        let report =
            r#"{"results":{"root//a:a":{"success":"SUCCESS"},"root//b:b":{"success":"FAIL"}}}"#;
        let name = baseline_artifact_name(BaselineKind::Build, SHA, RUN_ID, RUN_ATTEMPT).unwrap();
        assert_eq!(
            accept_downloaded_baseline(BaselineKind::Build, &name, &trusted_run(), report),
            Ok(2)
        );
        let stale =
            baseline_artifact_name(BaselineKind::Build, OTHER_SHA, RUN_ID, RUN_ATTEMPT).unwrap();
        assert!(is_refused(
            &accept_downloaded_baseline(BaselineKind::Build, &stale, &trusted_run(), report)
                .unwrap_err()
        ));
        for payload in [
            "<html>404</html>",
            "",
            r#"{"ok":true}"#,
            r#"{"results":{}}"#,
        ] {
            assert!(is_refused(
                &accept_downloaded_baseline(BaselineKind::Build, &name, &trusted_run(), payload)
                    .unwrap_err()
            ));
        }
    }

    #[test]
    fn downloaded_archive_digest_is_stable() {
        let path = std::env::temp_dir().join("oya-build-health-digest-test");
        fs::write(&path, b"trusted archive bytes").unwrap();
        assert_eq!(
            sha256_file(&path).unwrap(),
            "sha256:5a76aff2b5371b6e71c59005abec7618f3a4dd88244f6cfc6cd65f93e68caa8d"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn repository_slugs_are_validated_before_they_reach_an_api_route() {
        assert_eq!(validated_repo("oyatie/oyatie"), Ok("oyatie/oyatie"));
        assert_eq!(
            validated_repo("owner-1/repo_name.rs"),
            Ok("owner-1/repo_name.rs")
        );
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

    /// THE REGRESSION TEST FOR CI JOB 91383250718.
    ///
    /// `gh` was absent from the owned arm64 runner image, so the first API call failed with
    /// `No such file or directory (os error 2)` and the consumer reported `Refused` — the same
    /// state it reports when a baseline exists but fails a provenance check. A missing tool and a
    /// rejected artifact are not the same defect: one recurs on every run until the image changes,
    /// the other is the gate doing its job. Because they shared a state, the fast path was dark on
    /// the whole fleet for its entire life and the only symptom was wall-clock.
    ///
    /// The `gh` call site is gone, but the CLASS is reachable through `capture_to_file`, which
    /// still spawns `unzip`. That is deliberately the assertion: this pins the classification of
    /// an unspawnable program, not the identity of one particular tool.
    #[test]
    fn an_unspawnable_program_is_a_capability_fault_not_a_refusal() {
        let dest = std::env::temp_dir().join("oya-build-health-capability-fault-probe");
        let err = capture_to_file(
            "oya-no-such-program-on-any-runner-image",
            &["--version"],
            &dest,
        )
        .unwrap_err();
        assert!(
            matches!(err, CapabilityFault(_)),
            "a missing executable is an environment fault, not a refusal: {err:?}"
        );
        assert_eq!(err.state(), BaselineReuseState::CapabilityFault);
        assert_ne!(
            err.state().exit_code(),
            BaselineReuseState::Refused.exit_code(),
            "the workflow must be able to tell a dead capability from a rejected baseline"
        );
        let _ = fs::remove_file(&dest);

        // CONTROL: a program that RUNS and exits non-zero is a statement about the data, so it
        // stays a refusal. Without this the test above would pass for a version that classified
        // every subprocess failure as a fault — the mirror image of the original bug.
        let control = std::env::temp_dir().join("oya-build-health-capability-fault-control");
        let err = capture_to_file("false", &[], &control).unwrap_err();
        assert!(
            matches!(err, Refused(_)),
            "a tool that ran and failed is a data refusal: {err:?}"
        );
        let _ = fs::remove_file(&control);
    }

    /// Every outcome — including the degrades — must leave a machine-readable record. Before this,
    /// the workflow's `if <bin>; then exit 0; fi` swallowed the exit code and the only trace was
    /// one stderr line in a 68-minute log.
    #[test]
    fn every_outcome_is_reported_where_a_machine_can_read_it() {
        let dir = std::env::temp_dir().join("oya-build-health-reuse-outcome-report");
        let _ = fs::create_dir_all(&dir);
        report_reuse_outcome(
            &dir,
            SHA,
            BaselineReuseState::CapabilityFault,
            "could not execute `gh`",
        )
        .unwrap();
        let written = fs::read_to_string(dir.join(BASELINE_REUSE_OUTCOME_FILENAME))
            .expect("a degrade must write the outcome sidecar");
        let value: serde_json::Value = serde_json::from_str(&written).expect("valid JSON");
        assert_eq!(value["state"], "capability-fault");
        assert_eq!(value["capability_fault"], true);
        assert_eq!(value["cold_fallback_required"], true);
        assert_eq!(value["telemetry_persisted"], true);
        assert_eq!(value["exit_code"], 4);
        assert_eq!(value["merge_base"], SHA);

        report_reuse_outcome(&dir, SHA, BaselineReuseState::Reused, "pair downloaded").unwrap();
        let written = fs::read_to_string(dir.join(BASELINE_REUSE_OUTCOME_FILENAME))
            .expect("success must be recorded too");
        let value: serde_json::Value = serde_json::from_str(&written).expect("valid JSON");
        assert_eq!(value["state"], "reused");
        assert_eq!(value["capability_fault"], false);
        assert_eq!(value["cold_fallback_required"], false);
        let _ = fs::remove_dir_all(&dir);
    }
}
