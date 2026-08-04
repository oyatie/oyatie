//! cloud-ci-supply-chain-audit gate binary (owned RustSec advisory scan; replaces reverted #974).
<<<<<<< HEAD
//! The default invocation matches the workspace `Cargo.lock` against the vendored advisory mirror
//! and reports any affected, un-ignored crate. The blocking buck2 `rust_test` gate is the backstop.
=======
//! The default invocation first proves the policy-declared workspace lockfile corpus exactly against
//! the materialized SCM tracked-path boundary, then matches strict package rows against the vendored
//! advisory mirror and reports any affected, un-ignored crate. Paths are repo-relative, validated,
//! and symlink-free; collection never recursively walks mutable checkout state. The blocking buck2
//! `rust_test` gate is the backstop.
>>>>>>> 6d564e888 (fix(supply-chain): prove lockfile corpus totality)
//!
//! `--write` rewrites the policy's `ignore[]` SHRINK-ONLY: it drops entries that suppress no live
//! affected advisory (`SCA-STALE-IGNORE`) and never adds one (a new vuln must be fixed, not
//! baselined). After a successful `--write` the gate is GREEN against the cleaned ignore list;
//! review the diff before committing.
//!
//! Usage:
//!   oya-cloud-ci-supply-chain-audit-app-bin [--repo-root <path>] [--policy <path>] [--write]
//!
//! Exit codes: 0 = green; 1 = red findings remain; 2 = argument/collection error.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_supply_chain_audit::{
    Verdict, collect, evaluate, evaluate_keyed, render_findings, shrink_only_ignore,
};
use serde_json::Value;

const DEFAULT_POLICY: &str = "ci/facade/supply-chain-audit/supply-chain-audit-policy.json";

struct Args {
    repo_root: PathBuf,
    policy: Option<PathBuf>,
    write: bool,
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
            eprintln!("supply-chain-audit gate failed to run: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
    let observed = collect(&args.repo_root, &policy).map_err(|e| e.to_string())?;

    if args.write {
        let path = policy_path(&args.repo_root, args.policy.as_deref());
        let (kept, dropped) = shrink_only_ignore(&policy, &observed);
        write_ignore(&path, &kept)?;
        println!(
            "supply-chain-audit --write: ignore[] cleaned to {} entry(ies) in {} (shrink-only; dropped {} stale)",
            kept.len(),
            path.display(),
            dropped.len()
        );
        for id in &dropped {
            println!("  - dropped stale ignore {id}");
        }
        println!(
            "Review the diff and commit; the gate is now GREEN against the cleaned ignore list."
        );
        let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
        return Ok(report(&policy, &observed));
    }

    Ok(report(&policy, &observed))
}

/// Rewrite `ignore[]` in the policy JSON to keep only `keep_ids`, preserving the per-entry metadata
/// (reason/pull_chain/remove_by) of the kept entries and every other top-level field. Read-modify-write.
fn write_ignore(path: &Path, keep_ids: &[String]) -> Result<(), String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("read policy {}: {e}", path.display()))?;
    let mut policy: Value =
        serde_json::from_str(&text).map_err(|e| format!("parse policy {}: {e}", path.display()))?;
    let kept: Vec<Value> = policy
        .get("ignore")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter(|e| {
                    e.get("id")
                        .and_then(Value::as_str)
                        .is_some_and(|id| keep_ids.contains(&id.to_owned()))
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let Some(obj) = policy.as_object_mut() else {
        return Err(format!("policy {} is not a JSON object", path.display()));
    };
    obj.insert("ignore".to_owned(), Value::Array(kept));
    let mut serialized = serde_json::to_string_pretty(&policy)
        .map_err(|e| format!("serialize policy {}: {e}", path.display()))?;
    serialized.push('\n');
    std::fs::write(path, serialized).map_err(|e| format!("write policy {}: {e}", path.display()))
}

fn policy_path(repo_root: &Path, policy: Option<&Path>) -> PathBuf {
    match policy {
        Some(path) => path.to_path_buf(),
        None => repo_root.join(DEFAULT_POLICY),
    }
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
    let path = policy_path(repo_root, policy);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("read policy {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse policy {}: {e}", path.display()))
}

fn parse_args(args: Vec<String>) -> ParseOutcome {
    let mut repo_root = PathBuf::from(".");
    let mut policy = None;
    let mut write = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "supply-chain-audit: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--policy" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "supply-chain-audit: --policy requires a path".to_owned(),
                    );
                };
                policy = Some(PathBuf::from(value));
            }
            "--write" => write = true,
            "--help" | "-h" => return ParseOutcome::Help,
            other => {
                return ParseOutcome::Error(format!(
                    "supply-chain-audit: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    ParseOutcome::Run(Args {
        repo_root,
        policy,
        write,
    })
}

fn usage() -> String {
    "usage: oya-cloud-ci-supply-chain-audit-app-bin [--repo-root <path>] [--policy <path>] [--write]"
        .to_owned()
}
