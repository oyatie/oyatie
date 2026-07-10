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
use std::process::ExitCode;

use ci_affected_target_set::{
    BuildHealthVerdict, build_health_verdict, failing_targets, parse_build_report,
    parse_test_verdicts, test_verdicts_to_report_value,
};

const LOG: &str = "build-health";

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

fn main() -> ExitCode {
    // Early NORMALIZE mode: `--normalize-test-console <console> --normalize-out <report.json>`.
    // Handled before the ratchet arg parse because it needs neither baseline nor head report.
    let raw: Vec<String> = std::env::args().skip(1).collect();
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
