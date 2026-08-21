//! cloud-ci-gitops-chart-license gate binary (ADR-0706 D-5 / bead oyatie-f2fg). Reports every
//! GitOps-declared Helm chart pull that is either undeclared in the policy or carries a licence
//! forbidden for its plane. v1 is flag-only: adding a chart to the policy is a reviewed, one-line
//! data edit, not something this binary writes. The blocking `rust_test` gate is the enforcement
//! backstop.
//!
//! Usage:
//!   gitops-chart-license-gate [--repo-root <path>] [--policy <path>]
//!
//! Exit codes: 0 = green; 1 = red findings remain; 2 = argument or collection error (fail-closed).
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_gitops_chart_license::{
    Verdict, collect_chart_rows, evaluate, evaluate_keyed, render_findings,
};
use serde_json::Value;

const DEFAULT_POLICY: &str = "ci/facade/gitops-chart-license/gitops-chart-license-policy.json";

struct Args {
    repo_root: PathBuf,
    policy: Option<PathBuf>,
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
            eprintln!("gitops-chart-license gate failed to run: {message}");
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
    let observed = collect_chart_rows(&args.repo_root).map_err(|e| e.to_string())?;

    let findings = evaluate_keyed(&policy, &observed);
    println!("{}", render_findings(&findings));
    if evaluate(&policy, &observed).verdict == Verdict::Green {
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
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "gitops-chart-license: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--policy" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "gitops-chart-license: --policy requires a path".to_owned(),
                    );
                };
                policy = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                return ParseOutcome::Help;
            }
            other => {
                return ParseOutcome::Error(format!(
                    "gitops-chart-license: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    ParseOutcome::Run(Args { repo_root, policy })
}

fn usage() -> String {
    "usage: gitops-chart-license-gate [--repo-root <path>] [--policy <path>]".to_owned()
}
