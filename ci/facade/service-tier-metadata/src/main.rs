//! cloud-ci-tier-field-coverage gate binary (Phase-0 capability-first reorg; ADR-0562/0536/0245).
//! Detects (and reports, surface-all) any governed service manifest missing the dependency-class
//! triple (tier/tier_subtype/dr_tier), carrying an out-of-enum value, overloading `tier` with a
//! DR/deployment-mode value, or (for substrates) lacking a valid `substrate_dag_position`. The
//! blocking buck2 `rust_test` gate is the backstop; this binary is the runnable detector.
//!
//! Usage:
//!   oya-cloud-ci-tier-field-coverage-app-bin [--repo-root <path>] [--policy <path>]
//!
//! Exit codes: 0 = green (no findings); 1 = red findings remain; 2 = argument or collection error
//! (fail-closed).
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_service_tier_metadata::{
    Verdict, collect_manifests, evaluate, evaluate_keyed, render_findings,
};
use serde_json::Value;

const DEFAULT_POLICY: &str = "ci/facade/service-tier-metadata/tier-field-coverage-policy.json";

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
            eprintln!("tier-field-coverage gate failed to run: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
    let observed = collect_manifests(&args.repo_root, &policy).map_err(|e| e.to_string())?;
    let report = evaluate(&policy, &observed);
    let findings = evaluate_keyed(&policy, &observed);
    println!("{}", render_findings(&findings));
    println!(
        "tier-field-coverage: {} service manifest(s) checked",
        report.manifests_checked
    );
    if report.verdict == Verdict::Green {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

fn load_policy(repo_root: &Path, policy: Option<&Path>) -> Result<Value, String> {
    let path = match policy {
        Some(path) => path.to_path_buf(),
        None => repo_root.join(DEFAULT_POLICY),
    };
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("read policy {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse policy {}: {e}", path.display()))
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
                        "tier-field-coverage: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--policy" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "tier-field-coverage: --policy requires a path".to_owned(),
                    );
                };
                policy = Some(PathBuf::from(value));
            }
            "--help" | "-h" => return ParseOutcome::Help,
            other => {
                return ParseOutcome::Error(format!(
                    "tier-field-coverage: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    ParseOutcome::Run(Args { repo_root, policy })
}

fn usage() -> String {
    "usage: oya-cloud-ci-tier-field-coverage-app-bin [--repo-root <path>] [--policy <path>]"
        .to_owned()
}
