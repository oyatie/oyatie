//! cloud-ci-crypto-backend-purity gate binary (ADR-0506). It reads first-party BUCK files plus
//! generated `third-party/BUCK` and FAILS iff a policy-forbidden crypto backend is reachable from
//! the local Buck graph. It deliberately does NOT inspect Cargo.lock text nor invoke Cargo, because
//! lock/cargo-metadata supersets retain the documented unactivated optional-dep `ring` phantom
//! (ADR-0506) and would false-RED.
//!
//! Usage:
//!   cloud-ci-crypto-backend-purity-app-bin [--repo-root <path>] [--policy <path>]
//!
//! Exit codes: 0 = green (no forbidden backend activated); 1 = red findings; 2 = argument or
//! collection error (fail-closed).
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_crypto_backend_policy::{
    Verdict, collect_activated_backends, evaluate, evaluate_keyed, render_findings,
};
use serde_json::Value;

const DEFAULT_POLICY: &str = "ci/facade/crypto-backend-policy/crypto-backend-purity-policy.json";

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
            eprintln!("crypto-backend-purity gate failed to run: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
    let observed =
        collect_activated_backends(&args.repo_root, &policy).map_err(|e| e.to_string())?;
    Ok(report(&policy, &observed))
}

fn report(policy: &Value, observed: &Value) -> ExitCode {
    let findings = evaluate_keyed(policy, observed);
    println!("{}", render_findings(&findings));
    if evaluate(policy, observed).verdict == Verdict::Green {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
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
                        "crypto-backend-purity: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--policy" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "crypto-backend-purity: --policy requires a path".to_owned(),
                    );
                };
                policy = Some(PathBuf::from(value));
            }
            "--help" | "-h" => return ParseOutcome::Help,
            other => {
                return ParseOutcome::Error(format!(
                    "crypto-backend-purity: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    ParseOutcome::Run(Args { repo_root, policy })
}

fn usage() -> String {
    "usage: cloud-ci-crypto-backend-purity-app-bin [--repo-root <path>] [--policy <path>]"
        .to_owned()
}
