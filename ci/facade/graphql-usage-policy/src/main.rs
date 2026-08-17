//! cloud-ci-no-graphql-without-adr gate binary (ADR-0565). It scans the CANDIDATE tree — EVERY
//! `Cargo.toml` in the tree (members AND non-members; resolving `[workspace.dependencies]` renames
//! and `{ workspace = true }` inheritance), the resolved `Cargo.lock` graph (the transitive catch),
//! plus a read-only walk for `.graphql`/`.graphqls`/`.gql`/`.gqls`/`.sdl` schema files — and FAILS iff
//! a forbidden GraphQL library or schema file is present WITHOUT the artifact citing an ALLOWLISTED +
//! VALIDATED authorizing (reversing) ADR (a `policy.authorizing_adrs` id backed by a real Accepted
//! `docs/decisions` ADR that reverses the forbidding one). It deliberately does NOT diff a frozen
//! merge-base baseline (that is the PR/push baseline-asymmetry false-green); the candidate-tree
//! verdict is identical at PR-tier and push-tier. All I/O is hermetic read-only fs (no cargo/buck
//! shell-out, no network, no VCS).
//!
//! Usage:
//!   oya-cloud-ci-no-graphql-without-adr-app-bin [--repo-root <path>] [--policy <path>]
//!
//! Exit codes: 0 = green (no forbidden GraphQL artifact); 1 = red findings; 2 = argument or
//! collection error (fail-closed).
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_graphql_usage_policy::{
    Verdict, collect_graphql_artifacts, evaluate, evaluate_keyed, render_findings,
};
use serde_json::Value;

const DEFAULT_POLICY: &str = "ci/facade/graphql-usage-policy/no-graphql-without-adr-policy.json";

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
            eprintln!("no-graphql-without-adr gate failed to run: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
    let observed =
        collect_graphql_artifacts(&args.repo_root, &policy).map_err(|e| e.to_string())?;
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
                        "no-graphql-without-adr: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--policy" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "no-graphql-without-adr: --policy requires a path".to_owned(),
                    );
                };
                policy = Some(PathBuf::from(value));
            }
            "--help" | "-h" => return ParseOutcome::Help,
            other => {
                return ParseOutcome::Error(format!(
                    "no-graphql-without-adr: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    ParseOutcome::Run(Args { repo_root, policy })
}

fn usage() -> String {
    "usage: oya-cloud-ci-no-graphql-without-adr-app-bin [--repo-root <path>] [--policy <path>]"
        .to_owned()
}
