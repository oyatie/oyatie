//! cloud-ci-kernel-purity gate binary (ADR-0547). Automation-default doctrine: the default
//! invocation DETECTS and reports (with the best next action per finding); `--fix` APPLIES the
//! derivable subset (dead transient deps removed from Cargo.toml AND their rust_library BUCK
//! edges, the latter via the oya-buck-syntax-kernel fixer harness — ADR-0549 closes the
//! FRIC-1781200001 refusal-only descope) and reports what is left for a human design decision.
//! The blocking buck2 `rust_test` gate is the backstop.
//!
//! Usage:
//!   oya-cloud-ci-kernel-purity-app-bin [--repo-root <path>] [--policy <path>] [--fix]
//!
//! Exit codes: 0 = green (no findings, or all findings auto-fixed under --fix); 1 = red findings
//! remain; 2 = argument or collection error (fail-closed).
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_core_dependency_isolation::{
    Verdict, apply_fixes, collect_kernel_deps, evaluate, evaluate_keyed, plan_fixes,
    render_findings,
};
use serde_json::Value;

const DEFAULT_POLICY: &str = "ci/facade/core-dependency-isolation/kernel-purity-policy.json";

struct Args {
    repo_root: PathBuf,
    policy: Option<PathBuf>,
    fix: bool,
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
            eprintln!("kernel-purity gate failed to run: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
    let observed = collect_kernel_deps(&args.repo_root, &policy).map_err(|e| e.to_string())?;

    if args.fix {
        let fixes = plan_fixes(&policy, &observed);
        if fixes.is_empty() {
            println!("kernel-purity --fix: no auto-fixable (dead transient dependency) findings");
        } else {
            let applied = apply_fixes(&args.repo_root, &fixes).map_err(|e| e.to_string())?;
            println!("kernel-purity --fix applied {} edit(s):", applied.len());
            for line in &applied {
                println!("  - {line}");
            }
            println!(
                "Re-run the gate and commit the diff. Cargo.lock may need `cargo metadata >/dev/null`."
            );
        }
        // Re-collect after the edits so the residual report reflects the new tree state.
        let observed = collect_kernel_deps(&args.repo_root, &policy).map_err(|e| e.to_string())?;
        return Ok(report(&policy, &observed));
    }

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
    let mut fix = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "kernel-purity: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--policy" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "kernel-purity: --policy requires a path".to_owned(),
                    );
                };
                policy = Some(PathBuf::from(value));
            }
            "--fix" => fix = true,
            "--help" | "-h" => {
                return ParseOutcome::Help;
            }
            other => {
                return ParseOutcome::Error(format!(
                    "kernel-purity: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    ParseOutcome::Run(Args {
        repo_root,
        policy,
        fix,
    })
}

fn usage() -> String {
    "usage: oya-cloud-ci-kernel-purity-app-bin [--repo-root <path>] [--policy <path>] [--fix]"
        .to_owned()
}
