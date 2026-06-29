//! cloud-ci-affected-set composition root (ADR-0554, FRIC-1781310000).
//!
//! Orchestrates: merge-base diff (git) -> pure kernel classification -> per-file `owner()`
//! + `rdeps()` closure (buck2 uquery) -> `buck2 build` + `buck2 test` of the decided set.
//!
//! FAIL-CLOSED SEAMS (the escalation IS the automation — zero manual escape hatches):
//! - any git/uquery/rdeps derivation failure escalates to the FULL workspace run, never skips;
//! - an owner-required file with no owning target FAILS the lane (graph-invisible code cannot
//!   be made safe by running more targets);
//! - `--mode full` (merge-queue admission / post-merge on the integration branch) bypasses
//!   derivation entirely and runs the policy's full-run target patterns.
//!
//! Transparency contract: every changed file is printed with its classification, the decision
//! is printed with its reasons, and on FAILURE the exact reproduction command is printed.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

use oya_cloud_ci_affected_set_app::{
    Change, Decision, GATE_ID, GatePhaseOutcome, PathClass, Plan, Policy,
    affected_set_operator_artifact, build_health_verdict, failing_targets, parse_build_report,
    plan_changes, resolve,
};

const LOG: &str = "affected-set";

struct Args {
    policy_path: String,
    base: Option<String>,
    head: String,
    mode: Mode,
    derive_only: bool,
    /// Optional path to the merge-base build-health baseline report (ADR-0554 round-3). When set,
    /// a FULL decision runs the BUILD-HEALTH RATCHET (`buck2 build //... --keep-going
    /// --build-report` at head, compared against this baseline) instead of a hard `buck2 build
    /// //...` — blocking only build REGRESSIONS while grandfathering pre-existing build debt.
    /// The baseline MUST be produced from the merge-base checkout out-of-band (never the
    /// candidate tree); see the build-health binary's soundness note.
    baseline_report: Option<String>,
    /// Optional path for the durable machine-readable operator artifact that records the selected
    /// affected-set tier, refs, baseline requirement, and long-running phase signals.
    decision_artifact_out: Option<String>,
}

struct ArtifactContext {
    path: String,
    mode: &'static str,
    resolved_base_ref: String,
    resolved_head_ref: String,
    baseline_report_present: bool,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Mode {
    Auto,
    Full,
}

fn parse_args(mut argv: std::env::Args) -> Result<Args, String> {
    let _bin = argv.next();
    let mut policy_path = None;
    let mut base = None;
    let mut head = "HEAD".to_owned();
    let mut mode = Mode::Auto;
    let mut derive_only = false;
    let mut baseline_report = None;
    let mut decision_artifact_out = None;
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--policy" => policy_path = Some(argv.next().ok_or("--policy needs a value")?),
            "--base" => base = Some(argv.next().ok_or("--base needs a value")?),
            "--head" => head = argv.next().ok_or("--head needs a value")?,
            "--mode" => {
                mode = match argv.next().as_deref() {
                    Some("auto") => Mode::Auto,
                    Some("full") => Mode::Full,
                    other => return Err(format!("--mode must be auto|full, got {other:?}")),
                }
            }
            "--derive-only" => derive_only = true,
            "--baseline-report" => {
                baseline_report = Some(argv.next().ok_or("--baseline-report needs a value")?)
            }
            "--decision-artifact-out" => {
                decision_artifact_out =
                    Some(argv.next().ok_or("--decision-artifact-out needs a value")?)
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(Args {
        policy_path: policy_path.ok_or("--policy <path> is required")?,
        base,
        head,
        mode,
        derive_only,
        baseline_report,
        decision_artifact_out,
    })
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{LOG}: ARGS ERROR: {e}");
            eprintln!(
                "{LOG}: usage: oya-cloud-ci-affected-set --policy <pack.json> [--base <ref>] [--head <ref>] [--mode auto|full] [--derive-only] [--baseline-report <merge-base-build-report.json>] [--decision-artifact-out <path>]"
            );
            return ExitCode::from(2);
        }
    };
    // The policy pack is the ONLY hard input: without it not even the full-run target set is
    // known, so a missing/invalid pack cannot escalate — it fails.
    let policy_bytes = match fs::read_to_string(&args.policy_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!(
                "{LOG}: POLICY ERROR: cannot read `{}`: {e}",
                args.policy_path
            );
            return ExitCode::from(2);
        }
    };
    let policy = match Policy::from_json(&policy_bytes) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{LOG}: POLICY ERROR ({GATE_ID}): {e}");
            return ExitCode::from(2);
        }
    };
    let buck2 = std::env::var("BUCK2").unwrap_or_else(|_| "buck2".to_owned());
    let base = args
        .base
        .clone()
        .unwrap_or_else(|| policy.default_base_ref.clone());
    let artifact_context = match build_artifact_context(&args, &base) {
        Ok(context) => context,
        Err(e) => {
            eprintln!("{LOG}: FAIL — cannot resolve refs for affected-set operator artifact: {e}");
            return ExitCode::from(2);
        }
    };

    let decision = match args.mode {
        Mode::Full => Decision::Full {
            reasons: vec!["--mode full (admission/integration tier)".to_owned()],
        },
        Mode::Auto => derive(&args, &base, &policy, &buck2),
    };

    match decision {
        Decision::RefuseUnowned { paths } => {
            let final_decision = Decision::RefuseUnowned {
                paths: paths.clone(),
            };
            let phases = vec![
                phase("derive-affected-set-tier", "completed", "decision.tier"),
                phase(
                    "binding-build-test",
                    "not-run",
                    "owner-required file refused before build",
                ),
            ];
            if let Err(e) =
                maybe_write_decision_artifact(&artifact_context, &final_decision, &phases)
            {
                return artifact_failure(e);
            }
            eprintln!("{LOG}: FAIL — owner-required file(s) with NO owning buck2 target:");
            for p in &paths {
                eprintln!("{LOG}:   {p}");
            }
            eprintln!(
                "{LOG}: graph-invisible code cannot be made safe by running more targets — even a \
                 full-workspace run would not compile these files. Wire them into a BUCK target \
                 (or delete them); refusing to false-green."
            );
            ExitCode::from(2)
        }
        Decision::NoGraphTargets => {
            let final_decision = Decision::NoGraphTargets;
            let phases = vec![
                phase("derive-affected-set-tier", "completed", "decision.tier"),
                phase("binding-build-test", "not-run", "no graph targets"),
            ];
            if let Err(e) =
                maybe_write_decision_artifact(&artifact_context, &final_decision, &phases)
            {
                return artifact_failure(e);
            }
            println!(
                "{LOG}: decision=NO-GRAPH-TARGETS — every changed file is unowned and not in any \
                 owner-required class (docs/config-text outside the buildfile/escape classes) -> PASS"
            );
            ExitCode::SUCCESS
        }
        Decision::Full { reasons } => {
            let final_decision = Decision::Full {
                reasons: reasons.clone(),
            };
            println!("{LOG}: decision=FULL — running the complete workspace, because:");
            for r in &reasons {
                println!("{LOG}:   - {r}");
            }
            if args.derive_only {
                let phases = vec![
                    phase("derive-affected-set-tier", "completed", "decision.tier"),
                    phase(
                        "materialize-merge-base-build-health-baseline",
                        if args.baseline_report.is_some() {
                            "present"
                        } else if args.mode == Mode::Auto {
                            "absent"
                        } else {
                            "not-required"
                        },
                        "merge_base_build_health_baseline.report_present",
                    ),
                    phase("binding-build-test", "not-run", "--derive-only"),
                ];
                if let Err(e) =
                    maybe_write_decision_artifact(&artifact_context, &final_decision, &phases)
                {
                    return artifact_failure(e);
                }
                println!(
                    "{LOG}: --derive-only: would run `{buck2} build` + `{buck2} test` on: {}",
                    policy.full_run_targets.join(" ")
                );
                return ExitCode::SUCCESS;
            }
            let code = run_full(&buck2, &policy, args.baseline_report.as_deref());
            let phases = vec![
                phase(
                    "derive-affected-set-tier",
                    if args.mode == Mode::Full {
                        "bypassed-mode-full"
                    } else {
                        "completed"
                    },
                    "decision.tier",
                ),
                phase(
                    "materialize-merge-base-build-health-baseline",
                    if args.baseline_report.is_some() {
                        "present"
                    } else if args.mode == Mode::Auto {
                        "absent"
                    } else {
                        "not-required"
                    },
                    "merge_base_build_health_baseline.report_present",
                ),
                phase(
                    "binding-build-test",
                    "completed-check-exit-code",
                    "FULL workspace run completed; verdict is process exit code",
                ),
            ];
            if let Err(e) =
                maybe_write_decision_artifact(&artifact_context, &final_decision, &phases)
            {
                return artifact_failure(e);
            }
            code
        }
        Decision::Affected { seeds } => {
            println!(
                "{LOG}: decision=AFFECTED — {} seed target(s) from owners + package patterns",
                seeds.len()
            );
            match rdeps_closure(&buck2, &policy, &seeds) {
                Ok(targets) => {
                    println!(
                        "{LOG}: {} affected target(s) (seeds + reverse-dependency closure):",
                        targets.len()
                    );
                    for t in &targets {
                        println!("{LOG}:   {t}");
                    }
                    let final_decision = Decision::Affected {
                        seeds: seeds.clone(),
                    };
                    if args.derive_only {
                        let phases = vec![
                            phase("derive-affected-set-tier", "completed", "decision.tier"),
                            phase(
                                "rdeps-closure",
                                "completed",
                                format!("{} affected target(s)", targets.len()),
                            ),
                            phase("binding-build-test", "not-run", "--derive-only"),
                        ];
                        if let Err(e) = maybe_write_decision_artifact(
                            &artifact_context,
                            &final_decision,
                            &phases,
                        ) {
                            return artifact_failure(e);
                        }
                        println!("{LOG}: --derive-only: stopping before build/test.");
                        return ExitCode::SUCCESS;
                    }
                    match write_argfile("targets", &targets) {
                        Ok(path) => {
                            let code = run_buck(&buck2, &[], Some(&path));
                            let phases = vec![
                                phase("derive-affected-set-tier", "completed", "decision.tier"),
                                phase(
                                    "rdeps-closure",
                                    "completed",
                                    format!("{} affected target(s)", targets.len()),
                                ),
                                phase(
                                    "target-argfile",
                                    "completed",
                                    format!("target list preserved at {}", path.display()),
                                ),
                                phase(
                                    "binding-build-test",
                                    "completed-check-exit-code",
                                    "affected target build/test completed; verdict is process exit code",
                                ),
                            ];
                            if let Err(e) = maybe_write_decision_artifact(
                                &artifact_context,
                                &final_decision,
                                &phases,
                            ) {
                                return artifact_failure(e);
                            }
                            code
                        }
                        Err(e) => {
                            // Cannot even materialize the argfile: escalate, never skip.
                            println!("{LOG}: ESCALATE to FULL — argfile write failed: {e}");
                            let final_decision = Decision::Full {
                                reasons: vec![format!(
                                    "argfile write failed after AFFECTED decision: {e}"
                                )],
                            };
                            let code = run_full(&buck2, &policy, args.baseline_report.as_deref());
                            let phases = vec![
                                phase("derive-affected-set-tier", "completed", "decision.tier"),
                                phase(
                                    "rdeps-closure",
                                    "completed",
                                    format!("{} affected target(s)", targets.len()),
                                ),
                                phase("target-argfile", "failed-escalated", e.to_string()),
                                phase(
                                    "binding-build-test",
                                    "completed-check-exit-code",
                                    "FULL escalation executed after argfile failure",
                                ),
                            ];
                            if let Err(e) = maybe_write_decision_artifact(
                                &artifact_context,
                                &final_decision,
                                &phases,
                            ) {
                                return artifact_failure(e);
                            }
                            code
                        }
                    }
                }
                Err(reason) => {
                    println!("{LOG}: ESCALATE to FULL — {reason}");
                    let final_decision = Decision::Full {
                        reasons: vec![format!(
                            "rdeps closure failed after AFFECTED decision: {reason}"
                        )],
                    };
                    if args.derive_only {
                        let phases = vec![
                            phase("derive-affected-set-tier", "completed", "decision.tier"),
                            phase("rdeps-closure", "failed-escalated", reason.clone()),
                            phase("binding-build-test", "not-run", "--derive-only"),
                        ];
                        if let Err(e) = maybe_write_decision_artifact(
                            &artifact_context,
                            &final_decision,
                            &phases,
                        ) {
                            return artifact_failure(e);
                        }
                        println!(
                            "{LOG}: --derive-only: would run the full workspace: {}",
                            policy.full_run_targets.join(" ")
                        );
                        return ExitCode::SUCCESS;
                    }
                    let code = run_full(&buck2, &policy, args.baseline_report.as_deref());
                    let phases = vec![
                        phase("derive-affected-set-tier", "completed", "decision.tier"),
                        phase("rdeps-closure", "failed-escalated", reason),
                        phase(
                            "binding-build-test",
                            "completed-check-exit-code",
                            "FULL escalation executed after rdeps failure",
                        ),
                    ];
                    if let Err(e) =
                        maybe_write_decision_artifact(&artifact_context, &final_decision, &phases)
                    {
                        return artifact_failure(e);
                    }
                    code
                }
            }
        }
    }
}

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Auto => "auto",
        Mode::Full => "full",
    }
}

fn build_artifact_context(args: &Args, base: &str) -> Result<Option<ArtifactContext>, String> {
    let Some(path) = &args.decision_artifact_out else {
        return Ok(None);
    };
    Ok(Some(ArtifactContext {
        path: path.clone(),
        mode: mode_name(args.mode),
        resolved_base_ref: resolve_git_ref(base)?,
        resolved_head_ref: resolve_git_ref(&args.head)?,
        baseline_report_present: args.baseline_report.is_some(),
    }))
}

fn resolve_git_ref(reference: &str) -> Result<String, String> {
    let resolved = capture("git", &["rev-parse", "--verify", reference])?;
    let resolved = resolved.trim();
    if resolved.is_empty() {
        return Err(format!(
            "git rev-parse --verify {reference} produced an empty ref"
        ));
    }
    Ok(resolved.to_owned())
}

fn maybe_write_decision_artifact(
    context: &Option<ArtifactContext>,
    decision: &Decision,
    phases: &[GatePhaseOutcome],
) -> Result<(), String> {
    let Some(context) = context else {
        return Ok(());
    };
    let artifact = affected_set_operator_artifact(
        context.mode,
        &context.resolved_base_ref,
        &context.resolved_head_ref,
        context.baseline_report_present,
        decision,
        phases,
    );
    let bytes = serde_json::to_vec_pretty(&artifact)
        .map_err(|e| format!("serialize affected-set operator artifact: {e}"))?;
    fs::write(&context.path, bytes).map_err(|e| e.to_string())?;
    println!("{LOG}: operator-artifact: {}", context.path);
    Ok(())
}

fn artifact_failure(error: String) -> ExitCode {
    eprintln!("{LOG}: FAIL — cannot write affected-set operator artifact: {error}");
    ExitCode::from(2)
}

fn phase(
    phase: impl Into<String>,
    status: impl Into<String>,
    operator_signal: impl Into<String>,
) -> GatePhaseOutcome {
    GatePhaseOutcome::new(phase, status, operator_signal)
}

/// Auto-mode derivation. Any uncertainty returns `Decision::Full` with the reason (fail-closed
/// escalation); only determined graph-invisibility returns `RefuseUnowned`.
fn derive(args: &Args, base: &str, policy: &Policy, buck2: &str) -> Decision {
    let merge_base = match capture("git", &["merge-base", &args.head, base]) {
        Ok(out) => out.trim().to_owned(),
        Err(e) => {
            return Decision::Full {
                reasons: vec![format!(
                    "derivation uncertainty: git merge-base {} {base} failed: {e}",
                    args.head
                )],
            };
        }
    };
    println!(
        "{LOG}: base={base} head={} merge-base={merge_base}",
        args.head
    );
    let diff = match capture(
        "git",
        &["diff", "--name-status", "-z", &merge_base, &args.head],
    ) {
        Ok(out) => out,
        Err(e) => {
            return Decision::Full {
                reasons: vec![format!("derivation uncertainty: git diff failed: {e}")],
            };
        }
    };
    let changes = match parse_name_status_z(&diff) {
        Ok(c) => c,
        Err(e) => {
            return Decision::Full {
                reasons: vec![format!(
                    "derivation uncertainty: unparseable git diff entry: {e}"
                )],
            };
        }
    };
    if changes.is_empty() {
        println!("{LOG}: no changed files vs merge-base — nothing to derive");
        return Decision::NoGraphTargets;
    }
    println!("{LOG}: {} changed file(s) vs {merge_base}", changes.len());
    let plan = plan_changes(&changes, policy);
    let owner_results = match query_owners(buck2, &plan) {
        Ok(map) => map,
        Err(e) => {
            return Decision::Full {
                reasons: vec![format!(
                    "derivation uncertainty: buck2 owner() query failed: {e}"
                )],
            };
        }
    };
    print_classification(&plan, &owner_results);
    resolve(&plan, &owner_results, policy)
}

/// Parse `git diff --name-status -z` output: NUL-separated records, `R`/`C` carry two paths.
fn parse_name_status_z(raw: &str) -> Result<Vec<Change>, String> {
    let mut fields = raw.split('\0').filter(|s| !s.is_empty());
    let mut changes = Vec::new();
    while let Some(status) = fields.next() {
        let kind = status.chars().next().ok_or("empty status field")?;
        match kind {
            'A' | 'M' | 'T' => {
                let p = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without a path"))?;
                changes.push(Change::Present(p.to_owned()));
            }
            'D' => {
                let p = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without a path"))?;
                changes.push(Change::Deleted(p.to_owned()));
            }
            'R' => {
                let old = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without source path"))?;
                let new = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without destination path"))?;
                changes.push(Change::Deleted(old.to_owned()));
                changes.push(Change::Present(new.to_owned()));
            }
            'C' => {
                let _src = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without source path"))?;
                let dst = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without destination path"))?;
                changes.push(Change::Present(dst.to_owned()));
            }
            // U (unmerged), X (unknown), B (broken pair): states a clean CI checkout cannot
            // produce — surface as uncertainty rather than guessing.
            other => return Err(format!("unsupported diff status `{other}`")),
        }
    }
    Ok(changes)
}

/// Batched per-file owner resolution: `buck2 uquery --json "owner(%s)" @argfile` returns a
/// JSON object keyed by each path. A query ERROR is uncertainty (caller escalates) — it is
/// NEVER treated as "no owner" (the historic false-pass bug class).
fn query_owners(buck2: &str, plan: &Plan) -> Result<BTreeMap<String, Vec<String>>, String> {
    if plan.owner_paths.is_empty() {
        return Ok(BTreeMap::new());
    }
    let argfile = write_argfile("owner-paths", &plan.owner_paths).map_err(|e| e.to_string())?;
    let out = capture(
        buck2,
        &[
            "uquery",
            "--json",
            "owner(%s)",
            &format!("@{}", argfile.display()),
        ],
    )?;
    let v: serde_json::Value =
        serde_json::from_str(&out).map_err(|e| format!("owner() output is not JSON: {e}"))?;
    let obj = v.as_object().ok_or("owner() JSON is not an object")?;
    let mut map = BTreeMap::new();
    for (path, owners) in obj {
        let list = owners.as_array().ok_or("owner() entry is not an array")?;
        let mut targets = Vec::with_capacity(list.len());
        for t in list {
            targets.push(
                t.as_str()
                    .ok_or("owner() target is not a string")?
                    .to_owned(),
            );
        }
        map.insert(path.clone(), targets);
    }
    Ok(map)
}

/// Seeds -> reverse-dependency closure within the policy universe, via @argfile + `%Ss`
/// (arbitrary set size; an inline set overflows on large packages).
fn rdeps_closure(buck2: &str, policy: &Policy, seeds: &[String]) -> Result<Vec<String>, String> {
    let argfile = write_argfile("seeds", seeds).map_err(|e| e.to_string())?;
    let query = format!("rdeps({}, %Ss)", policy.universe);
    let out = capture(
        buck2,
        &["uquery", &query, &format!("@{}", argfile.display())],
    )?;
    let targets: Vec<String> = out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    if targets.is_empty() {
        return Err(
            "rdeps returned an empty closure for non-empty seeds (query problem)".to_owned(),
        );
    }
    Ok(targets)
}

fn print_classification(plan: &Plan, owners: &BTreeMap<String, Vec<String>>) {
    println!("{LOG}: classification (every changed file, mechanically derived):");
    for (path, class) in &plan.classified {
        match class {
            PathClass::FullTrigger(pat) => {
                println!("{LOG}:   FULL-TRIGGER {path} (matches `{pat}`)")
            }
            PathClass::DeletedGraphFile => {
                println!("{LOG}:   FULL-TRIGGER {path} (graph file deleted/unmappable)")
            }
            PathClass::Buildfile => {
                println!(
                    "{LOG}:   FULL-TRIGGER {path} (buildfile — blast radius exceeds its package)"
                )
            }
            PathClass::PackagePattern(pat) => println!("{LOG}:   PACKAGE      {path} -> {pat}"),
            PathClass::OwnerQuery => {
                let n = owners.get(path).map(Vec::len).unwrap_or(0);
                println!("{LOG}:   OWNER        {path} -> {n} target(s)");
            }
            PathClass::DeletedIrrelevant => {
                println!("{LOG}:   NO-GRAPH     {path} (deleted, outside graph classes)")
            }
        }
    }
}

/// The FULL-tier runner (ADR-0554 round-3; D7 round-4 producer). Two modes:
///
/// - WITHOUT a baseline report (`--mode full` at admission, or any caller that does not pass
///   `--baseline-report`): a hard `buck2 build //... --keep-going --build-report` + `buck2 test
///   //...` — EVERY build failure blocks (non-empty failure set = hard fail; no grandfathering:
///   the integration tip MUST be green). D7 (round-4): the admission build now captures a
///   `--build-report` as a PURE BYPRODUCT and derives the same hard verdict from the report's
///   failure set being EMPTY. The report is written to a stable path (`admission_report_path`)
///   so the trusted push-to-dev workflow can publish it as the `build-health-baseline-<sha>`
///   artifact (the merge-base-to-be baseline for the DEFERRED D8 cross-run consumer + ADR-0560
///   warm-CAS). Merge authority is UNCHANGED — the verdict is identical to the prior hard build,
///   nothing consumes the artifact yet, so there is zero laundering surface.
/// - WITH a baseline report (the PR `pull_request` FULL tier): the BUILD-HEALTH RATCHET. It builds
///   `//... --keep-going --build-report` at HEAD and tests them, then compares the HEAD build
///   FAILURE set against the merge-base baseline failure set: only REGRESSIONS (targets that build
///   at the merge-base but fail at head, or brand-new failing targets) block; pre-existing build
///   debt is grandfathered. This turns the FULL tier from a flag-day requirement into a true
///   ratchet (block new debt, grandfather pre-existing — FRIC-1781112000 / #698). Tests are still
///   run and a TEST regression in a buildable target blocks via the test exit (the ratchet governs
///   BUILD failures; a build that succeeds then test-fails is a normal hard failure).
fn run_full(buck2: &str, policy: &Policy, baseline_report: Option<&str>) -> ExitCode {
    let Some(baseline_path) = baseline_report else {
        // Admission/integration tier: hard full build+test, every failure blocks. D7: emit the
        // build-report as a byproduct and derive the hard verdict from an EMPTY failure set.
        return run_full_admission_producer(buck2, policy);
    };

    // PR FULL tier: build-health ratchet. Build the whole workspace with --keep-going so every
    // target's status is recorded even past the first failure, into a build-report.
    let head_report = match std::env::temp_dir()
        .join(format!(
            "{GATE_ID}-head-build-report-{}.json",
            std::process::id()
        ))
        .into_os_string()
        .into_string()
    {
        Ok(p) => p,
        Err(_) => {
            eprintln!("{LOG}: FAIL — could not form a temp path for the head build-report");
            return ExitCode::from(2);
        }
    };
    println!(
        "{LOG}: FULL tier (build-health ratchet vs merge-base baseline {baseline_path}): \
         {buck2} build //... --keep-going --build-report {head_report}"
    );
    // We intentionally do NOT propagate this build's exit code: --keep-going still exits non-zero
    // if ANY target failed, but pre-existing failures must NOT block. The verdict comes from the
    // build-report diff below. (A genuine infra failure surfaces as an unparseable/empty report,
    // which the ratchet then refuses on — fail-closed.)
    let _ = Command::new(buck2)
        .args([
            "build",
            "//...",
            "--keep-going",
            "--build-report",
            &head_report,
        ])
        .stdin(Stdio::null())
        .status();

    let baseline_json = match fs::read_to_string(baseline_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{LOG}: FAIL — cannot read merge-base baseline report `{baseline_path}`: {e}"
            );
            return ExitCode::from(2);
        }
    };
    let head_json = match fs::read_to_string(&head_report) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{LOG}: FAIL — cannot read head build-report `{head_report}`: {e}");
            return ExitCode::from(2);
        }
    };
    let baseline = match parse_build_report(&baseline_json) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{LOG}: FAIL — merge-base baseline report parse error: {e}");
            return ExitCode::from(2);
        }
    };
    let head = match parse_build_report(&head_json) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{LOG}: FAIL — head build-report parse error: {e}");
            return ExitCode::from(2);
        }
    };
    // Fail-closed laundering guard: an empty merge-base baseline would grandfather every head
    // failure. CI builds the whole merge-base workspace, so the baseline is never legitimately
    // empty — refuse rather than silently pass.
    if baseline.is_empty() {
        eprintln!(
            "{LOG}: FAIL — merge-base baseline build-report has no `results`. Refusing to \
             grandfather every head failure against an empty baseline (the laundering hole)."
        );
        return ExitCode::from(2);
    }

    let baseline_failures = failing_targets(&baseline);
    let head_failures = failing_targets(&head);
    let verdict = build_health_verdict(&baseline_failures, &head_failures);
    println!(
        "{LOG}: build-health — head build failures={}, baseline failures={}, regressions={}, \
         grandfathered={}, fixed={}",
        head_failures.len(),
        baseline_failures.len(),
        verdict.regressions.len(),
        verdict.grandfathered.len(),
        verdict.fixed.len()
    );
    for t in &verdict.grandfathered {
        println!("{LOG}:   pre-existing-red (grandfathered) {t}");
    }
    if !verdict.is_green() {
        eprintln!(
            "{LOG}: RED — {} build REGRESSION(S) vs the merge-base (built at origin/dev, FAIL at \
             head — or brand-new failing target):",
            verdict.regressions.len()
        );
        for t in &verdict.regressions {
            eprintln!("{LOG}:   REGRESSION {t}");
        }
        eprintln!(
            "{LOG}: REMEDIATION: fix these targets or revert the change that broke them; \
             pre-existing failures are grandfathered, only NEW build debt blocks. REPRODUCE: \
             {buck2} build {} --keep-going",
            verdict.regressions.join(" ")
        );
        return ExitCode::from(1);
    }

    // No build regressions -> GREEN. SCOPE (ADR-0554 round-3): the FULL tier governs BUILD health
    // (the cf16525 class is a COMPILE break). It deliberately does NOT run a workspace-wide
    // `buck2 test //...`: that would reintroduce a flag-day on PRE-EXISTING test failures (the
    // exact debt-grandfathering problem this round fixes, one layer up). Test coverage of the
    // ACTUAL changed code is the cone path's job (auto mode, hard-fail, unchanged — the cf16525
    // fixture); a FULL-tier TEST-health ratchet (same baseline-diff over a test report) is the
    // declared next IP. Conservative and sound: never false-green on a build regression, never
    // flag-day on pre-existing debt.
    println!(
        "{LOG}: PASS — no build regressions vs the merge-base ({} pre-existing build failure(s) \
         grandfathered).",
        verdict.grandfathered.len()
    );
    ExitCode::SUCCESS
}

/// The stable path the admission build-report is written to (D7). GitHub Actions sets
/// `RUNNER_TEMP`; we anchor the report there so the workflow's upload step references the SAME
/// path without guessing a PID. Off-CI (or if `RUNNER_TEMP` is unset) it falls back to the OS
/// temp dir with the identical basename — deterministic either way.
fn admission_report_path() -> PathBuf {
    let dir = std::env::var_os("RUNNER_TEMP")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    dir.join("build-health-admission-report.json")
}

/// The admission/integration FULL tier (D7 producer). Runs `buck2 build //... --keep-going
/// --build-report <stable path>` so the WHOLE workspace builds and every target's status is
/// captured into a report (a pure byproduct the trusted push-to-dev workflow publishes), then
/// derives the HARD verdict from the report's FAILURE SET being EMPTY — non-empty = hard fail,
/// NO grandfathering (the integration tip MUST be green, preserving `run_buck`'s admission
/// semantics). Finally runs `buck2 test //...` exactly as before. The verdict is identical to the
/// prior hard `buck2 build //...`; emitting the report does not change merge authority.
fn run_full_admission_producer(buck2: &str, policy: &Policy) -> ExitCode {
    let report_path = admission_report_path();
    let report_str = report_path.display().to_string();
    println!(
        "{LOG}: admission FULL tier (D7 producer): {buck2} build //... --keep-going \
         --build-report {report_str}"
    );
    // --keep-going still exits non-zero if ANY target failed; we do NOT read that exit code as the
    // verdict — the verdict comes from the build-report failure set below, so the report (the
    // published byproduct) and the pass/fail decision are derived from the SAME source of truth. A
    // genuine infra failure (buck2 could not run, no report) surfaces as an unparseable/empty
    // report, which is refused fail-closed.
    let _ = Command::new(buck2)
        .args([
            "build",
            "//...",
            "--keep-going",
            "--build-report",
            &report_str,
        ])
        .stdin(Stdio::null())
        .status();

    let report_json = match fs::read_to_string(&report_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{LOG}: FAIL — cannot read admission build-report `{report_str}`: {e}");
            return ExitCode::from(2);
        }
    };
    let report = match parse_build_report(&report_json) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{LOG}: FAIL — admission build-report parse error: {e}");
            return ExitCode::from(2);
        }
    };
    // Fail-closed: an admission build with no `results` is an infra failure, not a clean
    // workspace — refuse rather than false-green on an empty report.
    if report.is_empty() {
        eprintln!(
            "{LOG}: FAIL — admission build-report has no `results` (buck2 produced no targets). \
             Refusing to PASS the integration tip on an empty report."
        );
        return ExitCode::from(2);
    }

    let failures = failing_targets(&report);
    if !failures.is_empty() {
        // No grandfathering at admission: the integration tip MUST be green (the `run_buck`
        // hard-build semantics, now derived from the report's failure set).
        eprintln!(
            "{LOG}: RED — admission FULL build failed on {} target(s) (integration tip must be \
             green, no grandfathering):",
            failures.len()
        );
        for t in &failures {
            eprintln!("{LOG}:   BUILD-FAIL {t}");
        }
        eprintln!(
            "{LOG}: REPRODUCE: {buck2} build {} --keep-going",
            failures.iter().cloned().collect::<Vec<_>>().join(" ")
        );
        return ExitCode::from(1);
    }
    println!(
        "{LOG}: admission build GREEN — all {} workspace target(s) built; running {buck2} test \
         //... (report byproduct at {report_str})",
        report.len()
    );
    // Build is green -> run the full test suite exactly as the prior admission path did.
    run_buck(buck2, &policy.full_run_targets, None)
}

/// Run `buck2 build` then `buck2 test` on either explicit patterns or an @argfile, streaming
/// output. On failure prints the exact reproduction command and propagates the exit code.
fn run_buck(buck2: &str, patterns: &[String], argfile: Option<&PathBuf>) -> ExitCode {
    let spec: Vec<String> = match argfile {
        Some(path) => vec![format!("@{}", path.display())],
        None => patterns.to_vec(),
    };
    for verb in ["build", "test"] {
        let mut pretty = format!("{buck2} {verb}");
        for s in &spec {
            pretty.push(' ');
            pretty.push_str(s);
        }
        println!("{LOG}: === {pretty} ===");
        let status = Command::new(buck2)
            .arg(verb)
            .args(&spec)
            .stdin(Stdio::null())
            .status();
        match status {
            Ok(st) if st.success() => {}
            Ok(st) => {
                eprintln!("{LOG}: FAIL — `{pretty}` exited with {st}");
                eprintln!("{LOG}: ran on: {}", spec.join(" "));
                if let Some(path) = argfile {
                    eprintln!("{LOG}: target list preserved at {}", path.display());
                }
                eprintln!("{LOG}: REPRODUCE: {pretty}");
                return ExitCode::from(u8::try_from(st.code().unwrap_or(1)).unwrap_or(1));
            }
            Err(e) => {
                eprintln!("{LOG}: FAIL — could not execute `{pretty}`: {e}");
                return ExitCode::from(1);
            }
        }
    }
    println!("{LOG}: PASS");
    ExitCode::SUCCESS
}

fn capture(bin: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("could not execute `{bin} {}`: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "`{bin} {}` exited with {}: {}",
            args.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("`{bin}` output is not UTF-8: {e}"))
}

fn write_argfile(stem: &str, lines: &[String]) -> std::io::Result<PathBuf> {
    let path = std::env::temp_dir().join(format!("{GATE_ID}-{stem}-{}.txt", std::process::id()));
    let mut f = fs::File::create(&path)?;
    for line in lines {
        writeln!(f, "{line}")?;
    }
    Ok(path)
}
