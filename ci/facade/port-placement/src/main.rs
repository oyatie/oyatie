//! cloud-ci-port-placement gate binary (ADR-0570). Detects + reports (with the best next action
//! per finding) storage-port traits DEFINED in adapter crates. v1 is flag-only: relocating a trait
//! is a design act, so there is no `--fix` — the auto-MOVE codemod is a noted follow-up. The
//! blocking buck2 `rust_test` gate is the enforcement backstop.
//!
//! Usage:
//!   oya-cloud-ci-port-placement-app-bin [--repo-root <path>] [--policy <path>] [--baseline <path>]
//!
//! Exit codes: 0 = green (no NEW violations beyond the frozen baseline); 1 = red findings remain;
//! 2 = argument or collection error (fail-closed).
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_port_placement::{Verdict, collect_port_traits, evaluate, evaluate_keyed, render_findings};
use serde_json::Value;

const DEFAULT_POLICY: &str = "ci/facade/port-placement/port-placement-policy.json";
const DEFAULT_BASELINE: &str = "ci/facade/port-placement/port-placement-baseline.json";

struct Args {
    repo_root: PathBuf,
    policy: Option<PathBuf>,
    baseline: Option<PathBuf>,
}

enum ParseOutcome {
    Run(Args),
    Help,
    Error(String),
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1).collect()) {
        ParseOutcome::Run(args) => args,
        ParseOutcome::Help => {
            println!("{}", usage());
            return ExitCode::SUCCESS;
        }
        ParseOutcome::Error(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("port-placement gate failed to run: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let policy = load_json(
        &args.repo_root,
        args.policy.as_deref(),
        DEFAULT_POLICY,
        "policy",
    )?;
    let baseline = load_json(
        &args.repo_root,
        args.baseline.as_deref(),
        DEFAULT_BASELINE,
        "baseline",
    )?;
    let observed = collect_port_traits(&args.repo_root, &policy).map_err(|e| e.to_string())?;

    let findings = evaluate_keyed(&policy, &baseline, &observed);
    println!("{}", render_findings(&findings));
    if evaluate(&policy, &baseline, &observed).verdict == Verdict::Green {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

fn load_json(
    repo_root: &Path,
    override_path: Option<&Path>,
    default_rel: &str,
    label: &str,
) -> Result<Value, String> {
    let path = match override_path {
        Some(path) => path.to_path_buf(),
        None => repo_root.join(default_rel),
    };
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("read {label} {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {label} {}: {e}", path.display()))
}

fn parse_args(args: Vec<String>) -> ParseOutcome {
    let mut repo_root = PathBuf::from(".");
    let mut policy = None;
    let mut baseline = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "port-placement: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--policy" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "port-placement: --policy requires a path".to_owned(),
                    );
                };
                policy = Some(PathBuf::from(value));
            }
            "--baseline" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "port-placement: --baseline requires a path".to_owned(),
                    );
                };
                baseline = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                return ParseOutcome::Help;
            }
            other => {
                return ParseOutcome::Error(format!(
                    "port-placement: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    ParseOutcome::Run(Args {
        repo_root,
        policy,
        baseline,
    })
}

fn usage() -> String {
    "usage: oya-cloud-ci-port-placement-app-bin [--repo-root <path>] [--policy <path>] [--baseline <path>]"
        .to_owned()
}
