//! CLI wrapper for the ADR-0535 owned Rust toolchain bump proposer.
//!
//! Pure planner by design: no network I/O, no subprocesses, no clock, no randomness. The latest
//! stable version is supplied by the caller (`--latest-stable <v>` or `OYA_LATEST_STABLE_RUST`),
//! so a scheduled workflow (or an operator) owns the network fetch to
//! `https://static.rust-lang.org/dist/channel-rust-stable.toml` and hands the parsed version in.
//!
//! Exit codes:
//! - `0` — up to date, or dry-run plan produced, or bump applied and verified clean;
//! - `1` — `--check` found a bump available (the scheduled guard's "act now" signal);
//! - `2` — usage/validation error, or the applied bump left residual drift (fail closed).

#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_rust_toolchain_bump_proposer::{
    BumpPlan, apply_plan, current_pin, latest_is_newer, parse_stable_version, plan_bump,
    verify_clean,
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

    if args.mode == Mode::Check {
        let newer = latest_is_newer(&current, &latest)
            .map_err(|error| format!("compare versions: {error}"))?;
        if newer {
            println!("bump available: {current} -> {latest}");
            return Ok(1);
        }
        println!("up to date: pinned {current} is the latest stable {latest}");
        return Ok(0);
    }

    if current == latest {
        println!("no bump needed: pinned {current} already equals latest stable {latest}");
        return Ok(0);
    }

    let plan = plan_bump(repo_root, &current, &latest)
        .map_err(|error| format!("plan bump {current} -> {latest}: {error}"))?;
    print_plan(&plan);

    if args.mode == Mode::DryRun {
        println!(
            "dry-run: {} file(s) would change; rerun with --apply to mutate the tree",
            plan.changed_count()
        );
        return Ok(0);
    }

    apply_plan(repo_root, &plan).map_err(|error| format!("apply bump: {error}"))?;
    println!(
        "applied {current} -> {latest} across {} file(s)",
        plan.changed_count()
    );

    let residual = verify_clean(repo_root).map_err(|error| format!("verify bump: {error}"))?;
    if residual.is_clean() {
        println!(
            "verified: freshness rust-toolchain drift evaluator GREEN; ADR-0535 dependency-automation gate GREEN"
        );
        Ok(0)
    } else {
        eprintln!("residual drift after bump (fail closed, refusing to call it done):");
        for finding in &residual.drift_findings {
            eprintln!("  drift: {finding}");
        }
        for finding in &residual.gate_findings {
            eprintln!("  gate: {finding}");
        }
        Ok(2)
    }
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
    })
}

fn usage() -> String {
    format!(
        "usage:\n  oya-cloud-ci-rust-toolchain-bump-proposer [--repo-root <path>] [--latest-stable <v>] [--dry-run|--apply|--check]\n\n\
         --latest-stable <v>   latest stable Rust release (or set {LATEST_STABLE_ENV}); the caller owns the network fetch\n\
         --dry-run             print the bump plan without touching disk (default)\n\
         --apply               apply the plan, then verify against the drift evaluator and the ADR-0535 gate\n\
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
            }
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn apply_and_check_modes_parse() {
        match parse(&["--repo-root", "x", "--latest-stable", "1.98.0", "--apply"]) {
            ParseOutcome::Run(args) => {
                assert_eq!(args.mode, Mode::Apply);
                assert_eq!(args.repo_root, Path::new("x"));
                assert_eq!(args.latest_stable.as_deref(), Some("1.98.0"));
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
}
