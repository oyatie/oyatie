//! Adapter binary for the ADR-0535 owned Rust toolchain reconciler.
//!
//! The capability's primary surface is the `reconcile` library API (desired state in, reconciled
//! tree + report out). This binary is a thin adapter a scheduled workflow or the future typed
//! cloud-ci runner invokes — the same shape as the other `oya-cloud-ci-*` automation binaries.
//! It performs NO network I/O, NO subprocesses, NO clock, NO randomness: the latest stable
//! version is supplied by the caller (`--latest-stable <v>` or `OYA_LATEST_STABLE_RUST`), so the
//! scheduled fetch of `https://static.rust-lang.org/dist/channel-rust-stable.toml` lives in the
//! workflow step and the parsed version is handed in.
//!
//! Exit codes:
//! - `0` — up to date (and the tree is drift-clean), dry-run plan produced, or reconcile applied
//!   and verified clean;
//! - `1` — `--check` found a bump available (the scheduled guard's "act now" signal);
//! - `2` — usage/validation error, stale (older) latest supplied, or residual drift after an
//!   equal-pin apply (fail closed).

#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_rust_toolchain_bump_proposer::{
    BumpPlan, ReconcileOutcome, ResidualDrift, current_pin, latest_is_newer, parse_stable_version,
    plan_bump, reconcile, verify_clean,
};

const LATEST_STABLE_ENV: &str = "OYA_LATEST_STABLE_RUST";

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    DryRun,
    Apply,
    Check,
}

#[derive(Debug, PartialEq, Eq)]
struct Args {
    repo_root: PathBuf,
    latest_stable: Option<String>,
    mode: Mode,
    json: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum ParseOutcome {
    Run(Args),
    Help,
    Error(String),
}

fn main() -> ExitCode {
    let args = match parse_args(env::args().skip(1).collect()) {
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
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("rust-toolchain-bump-proposer: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<u8, String> {
    let repo_root = args.repo_root.as_path();
    let current = current_pin(repo_root).map_err(|error| format!("read current pin: {error}"))?;

    let latest_raw = args
        .latest_stable
        .clone()
        .or_else(|| env::var(LATEST_STABLE_ENV).ok())
        .ok_or_else(|| {
            format!(
                "latest stable version required: pass --latest-stable <v> or set {LATEST_STABLE_ENV}"
            )
        })?;
    let latest = parse_stable_version(&latest_raw)
        .map_err(|error| format!("validate latest stable: {error}"))?;

    match args.mode {
        Mode::Check => {
            let newer = latest_is_newer(&current, &latest)
                .map_err(|error| format!("compare versions: {error}"))?;
            if newer {
                println!("bump available: {current} -> {latest}");
                return Ok(1);
            }
            println!("up to date: pinned {current} is the latest stable {latest}");
            Ok(0)
        }
        Mode::DryRun => {
            if current == latest {
                // An equal pin does not make a stale tree aligned: verify both validators before
                // claiming no bump is needed.
                let residual = verify_clean(repo_root)
                    .map_err(|error| format!("verify tree alignment: {error}"))?;
                if !residual.is_clean() {
                    eprintln!(
                        "pinned {current} equals latest stable {latest}, but the tree has residual drift:{}",
                        render_residual(&residual)
                    );
                    return Ok(2);
                }
                println!(
                    "no bump needed: pinned {current} already equals latest stable {latest} and the tree is drift-clean"
                );
                return Ok(0);
            }
            if !latest_is_newer(&current, &latest)
                .map_err(|error| format!("compare versions: {error}"))?
            {
                return Err(format!(
                    "supplied latest stable {latest} is not newer than the pinned {current}; \
                     refusing to plan a backward rewrite"
                ));
            }
            let plan = plan_bump(repo_root, &current, &latest)
                .map_err(|error| format!("plan bump {current} -> {latest}: {error}"))?;
            print_plan(&plan);
            println!(
                "dry-run: {} file(s) would change; rerun with --apply to reconcile the tree",
                plan.changed_count()
            );
            Ok(0)
        }
        Mode::Apply => {
            let report = reconcile(repo_root, &latest)
                .map_err(|error| format!("reconcile {current} -> {latest}: {error}"))?;
            match report.outcome {
                ReconcileOutcome::Bumped => {
                    println!(
                        "reconciled {} -> {} across {} file(s)",
                        report.current,
                        report.latest,
                        report.changed_files.len()
                    );
                    println!(
                        "verified: freshness rust-toolchain drift evaluator GREEN; ADR-0535 dependency-automation gate GREEN"
                    );
                }
                ReconcileOutcome::UpToDate => {
                    println!(
                        "up to date: pinned {} already equals latest stable {} and the tree is drift-clean",
                        report.current, report.latest
                    );
                }
            }
            if args.json {
                println!("{}", render_report_json(&report));
            }
            Ok(0)
        }
    }
}

fn render_residual(residual: &ResidualDrift) -> String {
    let mut out = String::new();
    for finding in &residual.drift_findings {
        out.push_str(&format!("\n  drift: {finding}"));
    }
    for finding in &residual.gate_findings {
        out.push_str(&format!("\n  gate: {finding}"));
    }
    out
}

fn render_report_json(report: &ci_rust_toolchain_bump_proposer::ReconcileReport) -> String {
    let outcome = match report.outcome {
        ReconcileOutcome::UpToDate => "up-to-date",
        ReconcileOutcome::Bumped => "bumped",
    };
    let changed: Vec<String> = report
        .changed_files
        .iter()
        .map(|path| format!("\"{}\"", escape_json(path)))
        .collect();
    format!(
        "{{\n  \"current\": \"{}\",\n  \"latest\": \"{}\",\n  \"outcome\": \"{outcome}\",\n  \"changed_files\": [{}]\n}}",
        escape_json(&report.current),
        escape_json(&report.latest),
        changed.join(", ")
    )
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn print_plan(plan: &BumpPlan) {
    println!("bump plan: {} -> {}", plan.old, plan.new);
    for file in &plan.files {
        let marker = if file.changed { "CHANGE" } else { "same  " };
        println!("  [{marker}] {}", file.path);
    }
}

fn parse_args(args: Vec<String>) -> ParseOutcome {
    let mut repo_root = PathBuf::from(".");
    let mut latest_stable = None;
    let mut mode = Mode::DryRun;
    let mut json = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "rust-toolchain-bump-proposer: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--latest-stable" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "rust-toolchain-bump-proposer: --latest-stable requires a version"
                            .to_owned(),
                    );
                };
                latest_stable = Some(value);
            }
            "--apply" => mode = Mode::Apply,
            "--check" => mode = Mode::Check,
            "--dry-run" => mode = Mode::DryRun,
            "--json" => json = true,
            "--help" | "-h" => return ParseOutcome::Help,
            other => {
                return ParseOutcome::Error(format!(
                    "rust-toolchain-bump-proposer: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    ParseOutcome::Run(Args {
        repo_root,
        latest_stable,
        mode,
        json,
    })
}

fn usage() -> String {
    format!(
        "usage:\n  oya-cloud-ci-rust-toolchain-bump-proposer [--repo-root <path>] [--latest-stable <v>] [--dry-run|--apply|--check] [--json]\n\n\
         --latest-stable <v>   latest stable Rust release (or set {LATEST_STABLE_ENV}); the caller owns the network fetch\n\
         --dry-run             print the bump plan without touching disk (default)\n\
         --apply               reconcile the tree to <v> (plan + apply + verify); with --json, emit the machine-readable report\n\
         --check               exit 1 when a bump is available, 0 when up to date (scheduled-guard signal)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(parts: &[&str]) -> ParseOutcome {
        parse_args(parts.iter().map(|part| part.to_string()).collect())
    }

    #[test]
    fn defaults_to_dry_run_with_current_dir() {
        match parse(&[]) {
            ParseOutcome::Run(args) => {
                assert_eq!(args.mode, Mode::DryRun);
                assert_eq!(args.repo_root, Path::new("."));
                assert_eq!(args.latest_stable, None);
                assert!(!args.json);
            }
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn apply_check_and_json_modes_parse() {
        match parse(&[
            "--repo-root",
            "x",
            "--latest-stable",
            "1.98.0",
            "--apply",
            "--json",
        ]) {
            ParseOutcome::Run(args) => {
                assert_eq!(args.mode, Mode::Apply);
                assert_eq!(args.repo_root, Path::new("x"));
                assert_eq!(args.latest_stable.as_deref(), Some("1.98.0"));
                assert!(args.json);
            }
            other => panic!("expected Run, got {other:?}"),
        }
        match parse(&["--check"]) {
            ParseOutcome::Run(args) => assert_eq!(args.mode, Mode::Check),
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn unknown_flag_fails_closed() {
        assert!(matches!(parse(&["--bogus"]), ParseOutcome::Error(_)));
    }

    #[test]
    fn help_parses() {
        assert!(matches!(parse(&["--help"]), ParseOutcome::Help));
    }

    #[test]
    fn json_escaping_handles_quotes_and_backslashes() {
        assert_eq!(escape_json("a\"b\\c"), "a\\\"b\\\\c");
    }
}
