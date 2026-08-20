//! cloud-ci-dto-authz-trust gate binary (the CLASS-FIX for caller-supplied-authorization-trust;
//! sibling of cloud-ci-authz-coverage / issue #770 / AUTH-005). The default invocation DETECTS NEW
//! functions that trust a caller-supplied authorization decision (an `*Authorization` DTO / an
//! `x-authorization-*` header) self-compared against the same request with NO server-side PDP
//! decision-port call, and reports them with a remediation pointer to the fail-closed PDP doctrine.
//! The blocking buck2 `rust_test` gate is the backstop.
//!
//! `--write` (alias `--update-baseline`) regenerates the policy's `frozen_dto_authz_trust_instances`
//! signature keys from the live tree (the AUTOMATED property). The re-baseline is SHRINK-ONLY — it
//! drops keys for fixed/removed instances but REFUSES to absorb any key absent from the prior
//! committed baseline (a NEW instance), exiting 2 and printing each new key. Growing the baseline
//! requires the explicit `--allow-new` flag (an intentional, reviewed grandfather).
//!
//! Usage:
//!   oya-cloud-ci-dto-authz-trust-app-bin [--repo-root <path>] [--policy <path>] [--write [--allow-new]]
//!
//! Exit codes: 0 = green (no new caller-supplied-authz-trust, or baseline regenerated); 1 = red
//! findings remain; 2 = argument/collection error, or a `--write` blocked by new keys (fail-closed).
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_caller_supplied_authorization::{
    Verdict, collect_instances, evaluate, evaluate_keyed, render_findings, shrink_only_baseline,
};
use serde_json::Value;

const DEFAULT_POLICY: &str = "ci/facade/caller-supplied-authorization/dto-authz-trust-policy.json";

struct Args {
    repo_root: PathBuf,
    policy: Option<PathBuf>,
    write: bool,
    allow_new: bool,
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
            eprintln!("dto-authz-trust gate failed to run: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
    let observed = collect_instances(&args.repo_root, &policy).map_err(|e| e.to_string())?;

    if args.write {
        let path = policy_path(&args.repo_root, args.policy.as_deref());
        let (keys, new_keys) = shrink_only_baseline(&policy, &observed, args.allow_new);
        if !new_keys.is_empty() && !args.allow_new {
            eprintln!(
                "dto-authz-trust --write: REFUSING to grow the baseline. {} NEW caller-supplied-authz-trust instance key(s) are present that are NOT in the prior committed baseline:",
                new_keys.len()
            );
            for key in &new_keys {
                eprintln!("  + {key}");
            }
            eprintln!(
                "The baseline is SHRINK-ONLY. A new instance must be fixed (call a server-side PDP, fail closed) before merge, not baselined. To intentionally grandfather these (founder-signed exception), re-run with --allow-new. No file was written."
            );
            return Ok(ExitCode::from(2));
        }
        write_baseline(&path, &keys)?;
        println!(
            "dto-authz-trust --write: regenerated frozen_dto_authz_trust_instances with {} key(s) in {} (shrink-only{})",
            keys.len(),
            path.display(),
            if args.allow_new {
                "; --allow-new: NEW keys absorbed"
            } else {
                ""
            }
        );
        for key in &keys {
            println!("  - {key}");
        }
        if args.allow_new && !new_keys.is_empty() {
            println!("ABSORBED {} NEW key(s) (--allow-new):", new_keys.len());
            for key in &new_keys {
                println!("  + {key}");
            }
        }
        println!(
            "Review the diff and commit; the gate is now GREEN against the regenerated baseline."
        );
        let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
        return Ok(report(&policy, &observed));
    }

    Ok(report(&policy, &observed))
}

/// Rewrite `frozen_dto_authz_trust_instances` in the policy JSON to `keys`, preserving every other
/// field and a stable, pretty-printed shape.
fn write_baseline(path: &Path, keys: &[String]) -> Result<(), String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("read policy {}: {e}", path.display()))?;
    let mut policy: Value =
        serde_json::from_str(&text).map_err(|e| format!("parse policy {}: {e}", path.display()))?;
    let Some(obj) = policy.as_object_mut() else {
        return Err(format!("policy {} is not a JSON object", path.display()));
    };
    obj.insert(
        "frozen_dto_authz_trust_instances".to_owned(),
        Value::from(keys.to_vec()),
    );
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
    let mut allow_new = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "dto-authz-trust: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--policy" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "dto-authz-trust: --policy requires a path".to_owned(),
                    );
                };
                policy = Some(PathBuf::from(value));
            }
            "--write" | "--update-baseline" => write = true,
            "--allow-new" => allow_new = true,
            "--help" | "-h" => {
                return ParseOutcome::Help;
            }
            other => {
                return ParseOutcome::Error(format!(
                    "dto-authz-trust: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    ParseOutcome::Run(Args {
        repo_root,
        policy,
        write,
        allow_new,
    })
}

fn usage() -> String {
    "usage: oya-cloud-ci-dto-authz-trust-app-bin [--repo-root <path>] [--policy <path>] [--write [--allow-new]]"
        .to_owned()
}
