//! ci-reorg-target-debt gate binary (bootstrap step T3b). LOCAL BRIDGE feedback plus the
//! interval-audit engine entry point: merge authority is the Buck2 gate test behind
//! `oya-ci-required`, never this binary.
//!
//! Modes:
//! - default (`--mode check`): run Arms A–D over the live tree; JSON report to stdout;
//!   non-zero on red. Emits the liveness signal on every run.
//! - `--mode audit --commit-set <file> [--out <file>]`: deterministic interval audit over
//!   a captured commit-set (git I/O belongs to the caller; see the policy's
//!   `audit_mode.commit_set_materialization` recipe). Fails closed on missing, empty, or
//!   incomplete input.
//! - `--regen-baseline`: snapshot the current target-prefix estate into the committed
//!   shrink-only baseline file (the one-command regeneration the gate's failure messages
//!   name).
#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use ci_reorg_target_debt::{
    POLICY_PATH, Verdict, audit_interval, check_live_tree, load_baseline, load_json, load_policy,
    regenerate_baseline,
};

const USAGE: &str = "usage: ci-reorg-target-debt-bin [--repo-root <path>] [--policy <path>] \
                     [--mode check|audit] [--commit-set <path>] [--out <path>] [--regen-baseline]";

struct Args {
    repo_root: PathBuf,
    policy_path: String,
    mode: String,
    commit_set: Option<PathBuf>,
    out: Option<PathBuf>,
    regen_baseline: bool,
}

fn parse_args(args: Vec<String>) -> Result<Option<Args>, String> {
    let mut repo_root = PathBuf::from(".");
    let mut policy_path = POLICY_PATH.to_owned();
    let mut mode = "check".to_owned();
    let mut commit_set = None;
    let mut out = None;
    let mut regen_baseline = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(
                    iter.next().ok_or_else(|| format!("--repo-root requires a path; {USAGE}"))?,
                );
            }
            "--policy" => {
                policy_path =
                    iter.next().ok_or_else(|| format!("--policy requires a path; {USAGE}"))?;
            }
            "--mode" => {
                mode = iter.next().ok_or_else(|| format!("--mode requires a value; {USAGE}"))?;
            }
            "--commit-set" => {
                commit_set = Some(PathBuf::from(
                    iter.next().ok_or_else(|| format!("--commit-set requires a path; {USAGE}"))?,
                ));
            }
            "--out" => {
                out = Some(PathBuf::from(
                    iter.next().ok_or_else(|| format!("--out requires a path; {USAGE}"))?,
                ));
            }
            "--regen-baseline" => regen_baseline = true,
            "--help" | "-h" => return Ok(None),
            other => return Err(format!("unknown argument {other:?}; {USAGE}")),
        }
    }
    if mode != "check" && mode != "audit" {
        return Err(format!("--mode must be check or audit; {USAGE}"));
    }
    if mode == "audit" && commit_set.is_none() {
        return Err(format!("--mode audit requires --commit-set <path>; {USAGE}"));
    }
    Ok(Some(Args {
        repo_root,
        policy_path,
        mode,
        commit_set,
        out,
        regen_baseline,
    }))
}

fn render(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1).collect()) {
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Ok(Some(args)) => args,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    let (policy, _policy_value) = match load_policy(&args.repo_root, &args.policy_path) {
        Ok(loaded) => loaded,
        Err(error) => {
            eprintln!("reorg-target-debt gate failed to load policy: {error}");
            return ExitCode::FAILURE;
        }
    };

    if args.regen_baseline {
        return match regenerate_baseline(&args.repo_root, &policy) {
            Ok(baseline) => {
                let target = args.repo_root.join(&policy.baseline_file);
                let bytes = format!("{}\n", render(&baseline.to_json()));
                match fs::write(&target, bytes) {
                    Ok(()) => {
                        println!(
                            "reorg-target-debt baseline regenerated: {} ({} path digest(s), {} dep path digest(s), {} dep name digest(s), {} anchor(s))",
                            target.display(),
                            baseline.path_hashes.len(),
                            baseline.workspace_path_dep_hashes.len(),
                            baseline.dep_name_hashes.len(),
                            baseline.anchors.len(),
                        );
                        ExitCode::SUCCESS
                    }
                    Err(error) => {
                        eprintln!("failed to write baseline {}: {error}", target.display());
                        ExitCode::FAILURE
                    }
                }
            }
            Err(error) => {
                eprintln!("reorg-target-debt baseline regeneration failed: {error}");
                ExitCode::FAILURE
            }
        };
    }

    if args.mode == "audit" {
        let Some(commit_set_path) = args.commit_set else {
            eprintln!("--mode audit requires --commit-set; {USAGE}");
            return ExitCode::from(2);
        };
        let input = match load_json(&commit_set_path) {
            Ok(input) => input,
            Err(error) => {
                // FAIL-CLOSED: an unreadable capture is an explicit finding, never a pass.
                eprintln!("RTD_AUDIT_INPUT_INVALID: cannot load commit-set: {error}");
                return ExitCode::FAILURE;
            }
        };
        return match audit_interval(&policy, &input) {
            Ok(report) => {
                let rendered = render(&report.to_json());
                println!("{rendered}");
                if let Some(out) = &args.out
                    && let Err(error) = fs::write(out, format!("{rendered}\n"))
                {
                    eprintln!("failed to write audit report {}: {error}", out.display());
                    return ExitCode::FAILURE;
                }
                match report.verdict() {
                    Verdict::Green => ExitCode::SUCCESS,
                    Verdict::Red => ExitCode::FAILURE,
                }
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        };
    }

    let baseline = match load_baseline(&args.repo_root, &policy) {
        Ok(baseline) => baseline,
        Err(error) => {
            eprintln!("reorg-target-debt gate failed to load baseline: {error}");
            return ExitCode::FAILURE;
        }
    };
    match check_live_tree(&args.repo_root, &policy, &baseline) {
        Ok(report) => {
            println!("{}", render(&report.to_json()));
            match report.verdict() {
                Verdict::Green => ExitCode::SUCCESS,
                Verdict::Red => ExitCode::FAILURE,
            }
        }
        Err(error) => {
            eprintln!("reorg-target-debt gate failed: {error}");
            ExitCode::FAILURE
        }
    }
}
