//! cloud-ci-canonical-json gate + fixer binary (ADR-0546).
//!
//! Default mode CHECKS the governed corpus and fails (exit 1) on any non-canonical / unparseable /
//! duplicate-key file. `--fix` rewrites non-canonical files to canonical form (refusing parse/dup
//! defects). This is a LOCAL BRIDGE feedback tool (founder CLI-retirement directive): merge authority
//! lives in the buck2 gate test behind oya-ci-required, never in this binary.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use ci_canonical_json::{
    POLICY_PATH, Verdict, collect_observed, evaluate, fix_observed, load_policy, render_findings,
};

struct Args {
    repo_root: PathBuf,
    policy_path: String,
    fix: bool,
    dry_run: bool,
}

const USAGE: &str =
    "usage: oya-cloud-ci-canonical-json [--repo-root <path>] [--policy <path>] [--fix [--dry-run]]";

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1).collect()) {
        // `--help` is a successful request, not a usage error: print to stdout and exit 0.
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

    let policy = match load_policy(&args.repo_root, &args.policy_path) {
        Ok(policy) => policy,
        Err(error) => {
            eprintln!("canonical-json gate failed to load policy: {error}");
            return ExitCode::FAILURE;
        }
    };

    let observed = match collect_observed(&args.repo_root, &policy) {
        Ok(observed) => observed,
        Err(error) => {
            eprintln!("canonical-json gate failed to collect corpus: {error}");
            return ExitCode::FAILURE;
        }
    };

    if args.fix {
        match fix_observed(&args.repo_root, &policy, &observed, args.dry_run) {
            Ok(report) => {
                let verb = if args.dry_run {
                    "would rewrite"
                } else {
                    "rewrote"
                };
                if report.fixed.is_empty() {
                    println!("canonical-json fixer: no files needed rewriting");
                } else {
                    println!(
                        "canonical-json fixer {verb} {} file(s):",
                        report.fixed.len()
                    );
                    for path in &report.fixed {
                        println!("  {path}");
                    }
                }
                for (path, reason) in &report.refused {
                    eprintln!("REFUSED {path}: {reason} (fix this by hand, not with the fixer)");
                }
                if report.is_clean() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(error) => {
                eprintln!("canonical-json fixer failed: {error}");
                ExitCode::FAILURE
            }
        }
    } else {
        let report = evaluate(&policy, &observed);
        println!("{}", render_findings(&report.findings));
        match report.verdict {
            Verdict::Green => ExitCode::SUCCESS,
            Verdict::Red => ExitCode::FAILURE,
        }
    }
}

/// Parse argv. `Ok(None)` means `--help` was requested (print usage, exit 0); `Ok(Some(_))` is a
/// runnable invocation; `Err` is a usage error (exit 2).
fn parse_args(args: Vec<String>) -> Result<Option<Args>, String> {
    let mut repo_root = PathBuf::from(".");
    let mut policy_path = POLICY_PATH.to_owned();
    let mut fix = false;
    let mut dry_run = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| format!("--repo-root requires a path; {USAGE}"))?,
                );
            }
            "--policy" => {
                policy_path = iter
                    .next()
                    .ok_or_else(|| format!("--policy requires a path; {USAGE}"))?;
            }
            "--fix" => fix = true,
            "--dry-run" => dry_run = true,
            "--help" | "-h" => return Ok(None),
            other => return Err(format!("unknown argument {other:?}; {USAGE}")),
        }
    }
    if dry_run && !fix {
        return Err(format!("--dry-run requires --fix; {USAGE}"));
    }
    Ok(Some(Args {
        repo_root,
        policy_path,
        fix,
        dry_run,
    }))
}
