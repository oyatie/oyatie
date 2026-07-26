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
use std::process::{Command, ExitCode, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use ci_affected_target_set::{
    Decision, GATE_ID, GatePhaseOutcome, PathClass, Plan, Policy, affected_set_operator_artifact,
    build_health_verdict, failing_targets, failing_test_targets, long_step_telemetry_line,
    parse_build_report, parse_name_status_z, parse_test_verdicts, plan_changes, resolve,
    test_verdicts_to_report_value,
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
    /// Optional path to the merge-base TEST baseline report (ADR-0554 round-6, defect 3). A
    /// build-report-shaped `{results: {label: {success}}}` JSON of the merge-base `buck2 test //...`
    /// per-target verdicts (produced out-of-band from the merge-base checkout, like the build
    /// baseline). When set, a FULL decision runs the TEST-HEALTH RATCHET after the build-health
    /// ratchet: `buck2 test //... --keep-going` at head, whose per-target verdicts are diffed
    /// against this baseline — blocking only test REGRESSIONS while grandfathering pre-existing
    /// test debt. When ABSENT, the FULL tier still RUNS the tests but hard-fails on ANY test
    /// failure (no grandfathering) — FULL must build AND test, never build-only.
    test_baseline_report: Option<String>,
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
    let mut test_baseline_report = None;
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
            "--test-baseline-report" => {
                test_baseline_report =
                    Some(argv.next().ok_or("--test-baseline-report needs a value")?)
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
        test_baseline_report,
        decision_artifact_out,
    })
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{LOG}: ARGS ERROR: {e}");
            eprintln!(
                "{LOG}: usage: oya-cloud-ci-affected-set --policy <pack.json> [--base <ref>] [--head <ref>] [--mode auto|full] [--derive-only] [--baseline-report <merge-base-build-report.json>] [--test-baseline-report <merge-base-test-report.json>] [--decision-artifact-out <path>]"
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
            let code = run_full(
                &buck2,
                &policy,
                args.baseline_report.as_deref(),
                args.test_baseline_report.as_deref(),
            );
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
                            let code = run_full(
                                &buck2,
                                &policy,
                                args.baseline_report.as_deref(),
                                args.test_baseline_report.as_deref(),
                            );
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
                    let code = run_full(
                        &buck2,
                        &policy,
                        args.baseline_report.as_deref(),
                        args.test_baseline_report.as_deref(),
                    );
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
            PathClass::Deleted => {
                println!("{LOG}:   FULL-TRIGGER {path} (deleted — cone uncomputable at HEAD)")
            }
            PathClass::Structural(kind) => {
                println!(
                    "{LOG}:   FULL-TRIGGER {path} ({} — structural change the cone cannot bound)",
                    kind.describe()
                )
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
fn run_full(
    buck2: &str,
    policy: &Policy,
    baseline_report: Option<&str>,
    test_baseline_report: Option<&str>,
) -> ExitCode {
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
    let mut command = Command::new(buck2);
    command.args([
        "build",
        "//...",
        "--keep-going",
        "--build-report",
        &head_report,
    ]);
    if let Err(e) = run_command_with_progress(
        "build-health-ratchet-head-build",
        &mut command,
        &format!("{buck2} build //... --keep-going --build-report {head_report}"),
    ) {
        eprintln!("{LOG}: WARN — could not execute head build-health command: {e}");
    }

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

    // No build regressions. The build side is GREEN, but a FULL fallback that only BUILDS is
    // "checking less": a target can BUILD and still FAIL its tests at runtime (ADR-0554 round-6,
    // defect 3). Run the TEST-HEALTH RATCHET so FULL builds AND tests.
    println!(
        "{LOG}: build-health PASS — no build regressions vs the merge-base ({} pre-existing build \
         failure(s) grandfathered). Proceeding to the FULL-tier test-health ratchet.",
        verdict.grandfathered.len()
    );
    run_full_test_health(buck2, test_baseline_report)
}

/// FULL-tier TEST-HEALTH RATCHET (ADR-0554 round-6, defect 3). Runs `buck2 test //... --keep-going`
/// (builds are cache hits from the preceding build-health `buck2 build //...`), captures buck2's
/// per-target verdict console, and diffs the HEAD test-failure set against the merge-base TEST
/// baseline: only test REGRESSIONS block, pre-existing test debt is grandfathered — exactly the
/// build-health ratchet, one layer up. buck2's `--build-report` marks a build-OK-but-runtime-failed
/// target `SUCCESS` (verified live), so the console verdict lines are the ONLY per-target test
/// signal; [`parse_test_verdicts`] reconciles them against the `Tests finished:` summary and
/// fail-closes on any mismatch. When no `--test-baseline-report` is supplied the ratchet degrades
/// to HARD test-health (block ANY test failure, no grandfathering) — still fail-closed, never
/// build-only.
fn run_full_test_health(buck2: &str, test_baseline_report: Option<&str>) -> ExitCode {
    let console_path = match std::env::temp_dir()
        .join(format!(
            "{GATE_ID}-head-test-console-{}.log",
            std::process::id()
        ))
        .into_os_string()
        .into_string()
    {
        Ok(p) => p,
        Err(_) => {
            eprintln!("{LOG}: FAIL — could not form a temp path for the head test console");
            return ExitCode::from(2);
        }
    };
    let console_file = match fs::File::create(&console_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{LOG}: FAIL — could not create head test console `{console_path}`: {e}");
            return ExitCode::from(2);
        }
    };
    let console_err = match console_file.try_clone() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{LOG}: FAIL — could not clone head test console handle: {e}");
            return ExitCode::from(2);
        }
    };
    println!(
        "{LOG}: FULL tier test-health: {buck2} test //... --keep-going (per-target verdicts \
         captured to {console_path})"
    );
    // We intentionally do NOT propagate this run's exit code: --keep-going exits non-zero on ANY
    // pre-existing test/build failure, but only test REGRESSIONS must block. The verdict comes from
    // the reconciled per-target verdict diff below (a genuine infra/parse failure fails closed).
    let mut command = Command::new(buck2);
    command
        .args(["test", "//...", "--keep-going"])
        .stdout(Stdio::from(console_file))
        .stderr(Stdio::from(console_err));
    if let Err(e) = run_command_with_progress(
        "full-tier-test-health",
        &mut command,
        &format!("{buck2} test //... --keep-going"),
    ) {
        eprintln!("{LOG}: WARN — could not execute head test-health command: {e}");
    }

    let console = match fs::read_to_string(&console_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{LOG}: FAIL — cannot read head test console `{console_path}`: {e}");
            return ExitCode::from(2);
        }
    };
    // Surface buck2's own test output in the CI log (it was captured, not streamed).
    print!("{console}");

    let head = match parse_test_verdicts(&console) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "{LOG}: FAIL — could not derive a trustworthy per-target test-verdict set: {e}"
            );
            return ExitCode::from(2);
        }
    };
    let head_failures = failing_test_targets(&head);

    // Baseline test-failure set: the merge-base `buck2 test //...` verdicts, normalized to the
    // build-report shape. ABSENT baseline => EMPTY set => every head test failure is a regression
    // (HARD test-health, the strict end — no laundering risk).
    let baseline_failures = match test_baseline_report {
        Some(path) => {
            let json = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{LOG}: FAIL — cannot read merge-base test baseline `{path}`: {e}");
                    return ExitCode::from(2);
                }
            };
            match parse_build_report(&json) {
                Ok(r) => failing_targets(&r),
                Err(e) => {
                    eprintln!("{LOG}: FAIL — merge-base test baseline parse error: {e}");
                    return ExitCode::from(2);
                }
            }
        }
        None => {
            println!(
                "{LOG}: no --test-baseline-report supplied — HARD test-health (any test failure \
                 blocks; no pre-existing-debt grandfathering)"
            );
            std::collections::BTreeSet::new()
        }
    };

    let verdict = build_health_verdict(&baseline_failures, &head_failures);
    println!(
        "{LOG}: test-health — head test failures={}, baseline test failures={}, regressions={}, \
         grandfathered={}, fixed={}",
        head_failures.len(),
        baseline_failures.len(),
        verdict.regressions.len(),
        verdict.grandfathered.len(),
        verdict.fixed.len()
    );
    for t in &verdict.grandfathered {
        println!("{LOG}:   pre-existing test failure (grandfathered) {t}");
    }
    if !verdict.is_green() {
        eprintln!(
            "{LOG}: RED — {} TEST REGRESSION(S) vs the merge-base (passed at origin/dev, FAIL at \
             head — or a brand-new failing test target):",
            verdict.regressions.len()
        );
        for t in &verdict.regressions {
            eprintln!("{LOG}:   TEST-REGRESSION {t}");
        }
        eprintln!(
            "{LOG}: REMEDIATION: fix these tests or revert the change that broke them; pre-existing \
             test failures are grandfathered, only NEW test debt blocks. REPRODUCE: {buck2} test {}",
            verdict.regressions.join(" ")
        );
        return ExitCode::from(1);
    }
    println!(
        "{LOG}: PASS — FULL tier build+test green: no build regressions and no test regressions vs \
         the merge-base ({} pre-existing test failure(s) grandfathered).",
        verdict.grandfathered.len()
    );
    ExitCode::SUCCESS
}

/// The directory the stable admission reports are written to. GitHub Actions sets `RUNNER_TEMP`;
/// we anchor the reports there so the workflow's upload steps reference the SAME paths without
/// guessing a PID. Off-CI (or if `RUNNER_TEMP` is unset) it falls back to the OS temp dir with
/// identical basenames — deterministic either way.
fn admission_report_dir() -> PathBuf {
    std::env::var_os("RUNNER_TEMP")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

/// The stable path the admission BUILD report is written to (D7).
fn admission_report_path() -> PathBuf {
    admission_report_dir().join("build-health-admission-report.json")
}

/// The stable path the admission TEST report is written to (GH #1323/#899, the test half of the
/// D8 baseline pair). Same shape as the build report — `parse_build_report`-readable — because it
/// is `test_verdicts_to_report_value` output, so the test-health ratchet consumes it identically.
fn admission_test_report_path() -> PathBuf {
    admission_report_dir().join("test-health-admission-report.json")
}

fn long_step_telemetry_interval() -> Duration {
    std::env::var("OYA_CI_LONG_STEP_TELEMETRY_INTERVAL_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(30))
}

fn run_command_with_progress(
    phase: &str,
    command: &mut Command,
    pretty: &str,
) -> std::io::Result<ExitStatus> {
    let started = Instant::now();
    println!(
        "{}",
        long_step_telemetry_line(LOG, phase, "started", 0, &format!("command={pretty}"))
    );

    let mut child = command.stdin(Stdio::null()).spawn()?;
    let interval = long_step_telemetry_interval();
    let poll_interval = if interval < Duration::from_millis(250) {
        interval
    } else {
        Duration::from_millis(250)
    };
    let mut last_running_emit = started;

    loop {
        if let Some(status) = child.try_wait()? {
            println!(
                "{}",
                long_step_telemetry_line(
                    LOG,
                    phase,
                    "completed",
                    started.elapsed().as_secs(),
                    &format!("exit_status={status}"),
                )
            );
            return Ok(status);
        }

        if last_running_emit.elapsed() >= interval {
            println!(
                "{}",
                long_step_telemetry_line(
                    LOG,
                    phase,
                    "running",
                    started.elapsed().as_secs(),
                    &format!("command={pretty}"),
                )
            );
            last_running_emit = Instant::now();
        }
        thread::sleep(poll_interval);
    }
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
    let mut command = Command::new(buck2);
    command.args([
        "build",
        "//...",
        "--keep-going",
        "--build-report",
        &report_str,
    ]);
    if let Err(e) = run_command_with_progress(
        "admission-full-build-health-baseline",
        &mut command,
        &format!("{buck2} build //... --keep-going --build-report {report_str}"),
    ) {
        eprintln!("{LOG}: WARN — could not execute admission build-health command: {e}");
    }

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
    // Build is green -> run the full test suite. The build above already proved every target
    // builds from the SAME report the verdict is derived from, so the redundant second
    // `buck2 build` the old `run_buck` tail performed is dropped (pure cache-hit re-walk).
    run_admission_test_producer(buck2, &policy.full_run_targets)
}

/// The admission/integration TEST tier + TEST-baseline producer (GH #1323/#899).
///
/// Runs the same `buck2 test <full_run_targets>` the admission path always ran and keeps the SAME
/// hard verdict (buck2's exit status — the integration tip must be green, no grandfathering), but
/// captures buck2's per-target verdict console and normalizes it into the build-report shape at
/// [`admission_test_report_path`]. That file is the TEST half of the merge-base baseline pair the
/// trusted push-to-dev workflow publishes as `test-health-baseline-<sha>`; without it the D8
/// consumer could only skip the merge-base BUILD, and `buck2 test //...` would rebuild the
/// workspace anyway — so the build-only baseline saves almost nothing.
///
/// Producer-only: nothing about merge authority changes here. A green tip is still required.
fn run_admission_test_producer(buck2: &str, patterns: &[String]) -> ExitCode {
    let console_path = admission_report_dir().join(format!("{GATE_ID}-admission-test-console.log"));
    let console_file = match fs::File::create(&console_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "{LOG}: FAIL — could not create admission test console `{}`: {e}",
                console_path.display()
            );
            return ExitCode::from(2);
        }
    };
    let console_err = match console_file.try_clone() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{LOG}: FAIL — could not clone admission test console handle: {e}");
            return ExitCode::from(2);
        }
    };
    let mut pretty = format!("{buck2} test");
    for p in patterns {
        pretty.push(' ');
        pretty.push_str(p);
    }
    println!("{LOG}: === {pretty} (per-target verdicts captured for the test baseline) ===");
    let mut command = Command::new(buck2);
    command
        .arg("test")
        .args(patterns)
        .stdout(Stdio::from(console_file))
        .stderr(Stdio::from(console_err));
    let status = run_command_with_progress("admission-test-health-baseline", &mut command, &pretty);

    let console = match fs::read_to_string(&console_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{LOG}: FAIL — cannot read admission test console `{}`: {e}",
                console_path.display()
            );
            return ExitCode::from(2);
        }
    };
    // The console was captured, not streamed — surface buck2's own output in the CI log.
    print!("{console}");

    // HARD admission verdict FIRST and unchanged: any test failure blocks the integration tip.
    // A red tip is never published as a baseline (the upload step never runs on a failed job), so
    // normalization is only attempted on green.
    match status {
        Ok(st) if st.success() => {}
        Ok(st) => {
            eprintln!("{LOG}: FAIL — `{pretty}` exited with {st}");
            eprintln!("{LOG}: REPRODUCE: {pretty}");
            return ExitCode::from(u8::try_from(st.code().unwrap_or(1)).unwrap_or(1));
        }
        Err(e) => {
            eprintln!("{LOG}: FAIL — could not execute `{pretty}`: {e}");
            return ExitCode::from(1);
        }
    }

    // Fail-closed normalization: `parse_test_verdicts` reconciles the per-target verdict lines
    // against buck2's own `Tests finished:` summary and errors on any mismatch. Refuse to publish
    // an under-enumerated baseline — a missing target reads as "not failing at the merge-base",
    // which is exactly how a future PR's test regression would get grandfathered away.
    let verdicts = match parse_test_verdicts(&console) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "{LOG}: FAIL — admission tests PASSED but the per-target verdict console could \
                 not be reconciled into a trustworthy test baseline: {e}"
            );
            return ExitCode::from(2);
        }
    };
    let test_report_path = admission_test_report_path();
    let bytes = match serde_json::to_vec_pretty(&test_verdicts_to_report_value(&verdicts)) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{LOG}: FAIL — could not serialize the admission test baseline: {e}");
            return ExitCode::from(2);
        }
    };
    if let Err(e) = fs::write(&test_report_path, bytes) {
        eprintln!(
            "{LOG}: FAIL — could not write the admission test baseline `{}`: {e}",
            test_report_path.display()
        );
        return ExitCode::from(2);
    }
    println!(
        "{LOG}: PASS — admission test tier green; test baseline byproduct: {} target verdict(s) \
         -> {}",
        verdicts.len(),
        test_report_path.display()
    );
    ExitCode::SUCCESS
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
        let phase = match verb {
            "build" => "binding-build",
            "test" => "binding-test",
            _ => "binding-build-test",
        };
        let mut command = Command::new(buck2);
        command.arg(verb).args(&spec);
        let status = run_command_with_progress(phase, &mut command, &pretty);
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
