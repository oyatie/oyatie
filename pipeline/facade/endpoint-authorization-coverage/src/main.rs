//! cloud-ci-authz-coverage gate binary (issue #770; AUTH-005 pipeline-as-product backstop).
//! The default invocation DETECTS NEW unauthenticated HTTP control-plane surfaces over the
//! candidate tree and reports them with a remediation pointer to the fail-closed doctrine. The
//! blocking buck2 `rust_test` gate is the backstop.
//!
//! `--write` (alias `--update-baseline`) regenerates the policy's `frozen_unauthenticated_surfaces`
//! signature keys from the live tree (the AUTOMATED property: re-baselining is mechanical, not
//! hand-edited — mirrors the kernel-purity `--fix` / arch-graph `--write` pattern). MAJOR-1: the
//! re-baseline is SHRINK-ONLY — it drops keys for fixed/removed surfaces but REFUSES to absorb any
//! key absent from the prior committed baseline (a NEW unauthenticated control plane), exiting 2 and
//! printing each new key. Growing the baseline requires the explicit `--allow-new` flag (an
//! intentional, reviewed grandfather). After a successful `--write` the gate is GREEN against the
//! regenerated baseline; review the diff before committing.
//!
//! Usage:
//!   cloud-ci-authz-coverage-app-bin [--repo-root <path>] [--policy <path>] [--write [--allow-new]]
//!
//! Exit codes: 0 = green (no new unauthenticated control plane, or baseline regenerated); 1 = red
//! findings remain; 2 = argument/collection error, or a `--write` blocked by new keys (fail-closed).
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_endpoint_authorization_coverage::{
    Verdict, collect_surfaces, evaluate, evaluate_keyed, render_findings, shrink_only_baseline,
};
use serde_json::Value;

const DEFAULT_POLICY: &str = "ci/facade/endpoint-authorization-coverage/authz-coverage-policy.json";

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
            eprintln!("authz-coverage gate failed to run: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
    let observed = collect_surfaces(&args.repo_root, &policy).map_err(|e| e.to_string())?;

    if args.write {
        let path = policy_path(&args.repo_root, args.policy.as_deref());
        // MAJOR-1: SHRINK-ONLY re-baseline. Without --allow-new the baseline can only shrink (drop
        // fixed/removed surfaces); a NEW unauthenticated control plane is NOT silently absorbed. The
        // new keys are always printed; --allow-new is required (and prints them as ABSORBED) to grow.
        let (keys, new_keys) = shrink_only_baseline(&policy, &observed, args.allow_new);
        if !new_keys.is_empty() && !args.allow_new {
            eprintln!(
                "authz-coverage --write: REFUSING to grow the baseline. {} NEW unauthenticated control-plane surface key(s) are present that are NOT in the prior committed baseline:",
                new_keys.len()
            );
            for key in &new_keys {
                eprintln!("  + {key}");
            }
            eprintln!(
                "The baseline is SHRINK-ONLY. A new unauthenticated control plane must be fixed (add fail-closed authz) before merge, not baselined. To intentionally grandfather these (founder-signed exception), re-run with --allow-new. No file was written."
            );
            return Ok(ExitCode::from(2));
        }
        write_baseline(&path, &keys)?;
        println!(
            "authz-coverage --write: regenerated frozen_unauthenticated_surfaces with {} key(s) in {} (shrink-only{})",
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
        // Re-load + re-evaluate so the printed verdict reflects the freshly written baseline.
        let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
        return Ok(report(&policy, &observed));
    }

    Ok(report(&policy, &observed))
}

/// Rewrite `frozen_unauthenticated_surfaces` in the policy JSON to `keys`, preserving every other
/// field and a stable, pretty-printed shape. Read-modify-write of the on-disk policy DATA.
fn write_baseline(path: &Path, keys: &[String]) -> Result<(), String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("read policy {}: {e}", path.display()))?;
    let mut policy: Value =
        serde_json::from_str(&text).map_err(|e| format!("parse policy {}: {e}", path.display()))?;
    let Some(obj) = policy.as_object_mut() else {
        return Err(format!("policy {} is not a JSON object", path.display()));
    };
    obj.insert(
        "frozen_unauthenticated_surfaces".to_owned(),
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
                        "authz-coverage: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--policy" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "authz-coverage: --policy requires a path".to_owned(),
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
                    "authz-coverage: unknown argument {other:?}; {}",
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
    "usage: cloud-ci-authz-coverage-app-bin [--repo-root <path>] [--policy <path>] [--write [--allow-new]]"
        .to_owned()
}
