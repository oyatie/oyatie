//! Runnable producer for `registry/graph/architecture-map.json`.
//!
//! Usage:
//!   oya-intelligence-architecture-map [--write|--check]
//!     [--repo-root <.>] [--output <registry/graph/architecture-map.json>]
//!
//! Default (no flag) and `--write` regenerate the face in place; the
//! materializer invokes it that way. `--check` regenerates in memory and
//! byte-compares against the committed file, exiting non-zero on drift.
//! Not a human-typed surface (cli_surface_policy): the binary exists so the
//! generated face has a producer in the buck2 graph.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use oya_intelligence_architecture_map_app::{DEFAULT_OUTPUT, RunOutcome, run};

const USAGE: &str = "usage: oya-intelligence-architecture-map [--write|--check] \
[--repo-root <path>] [--output <path>]";

fn main() -> ExitCode {
    let mut repo_root = PathBuf::from(".");
    let mut output: Option<PathBuf> = None;
    let mut check = false;
    let mut write = false;
    let mut argv = std::env::args().skip(1);
    while let Some(flag) = argv.next() {
        let value = match flag.as_str() {
            "--write" => {
                write = true;
                continue;
            }
            "--check" => {
                check = true;
                continue;
            }
            "--repo-root" | "--output" => match argv.next() {
                Some(value) if !value.is_empty() => value,
                _ => {
                    eprintln!("{USAGE}");
                    return ExitCode::from(2);
                }
            },
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        };
        match flag.as_str() {
            "--repo-root" => repo_root = PathBuf::from(value),
            _ => output = Some(PathBuf::from(value)),
        }
    }
    if write && check {
        eprintln!("--write and --check are mutually exclusive");
        return ExitCode::from(2);
    }
    let output = output.unwrap_or_else(|| repo_root.join(DEFAULT_OUTPUT));

    match run(&repo_root, &output, check) {
        Ok(RunOutcome::Wrote) => {
            println!("architecture-map: wrote {}", output.display());
            ExitCode::SUCCESS
        }
        Ok(RunOutcome::Clean) => {
            println!("architecture-map --check passed: {}", output.display());
            ExitCode::SUCCESS
        }
        Ok(RunOutcome::Drifted { committed_path }) => {
            eprintln!(
                "architecture-map --check failed: {committed_path} drifted from the regenerated map"
            );
            eprintln!("  run `oya-intelligence-architecture-map --write` to regenerate it");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("architecture-map: {error}");
            ExitCode::FAILURE
        }
    }
}
