//! CI pipeline entrypoint for the architecture-graph dashboard generator.
//!
//! Usage:
//!   architecture-graph-generator [--write|--check]
//!     [--ssot <docs/machine-readable/architecture-graph.json>]
//!     [--masterplan <docs/machine-readable/masterplan.generated.json>]
//!     [--template <docs/architecture/product-graph.template.html>]
//!     [--output <docs/architecture/product-graph.html>]
//!
//! Default (no flag) and `--write` regenerate the dashboard HTML in place.
//! `--check` regenerates in memory and byte-compares against the committed file,
//! exiting non-zero on drift (the drift gate the CI pipeline runs).

use std::path::PathBuf;
use std::process::ExitCode;

use architecture_graph_generator_app::{
    DEFAULT_GRAPH_SSOT, DEFAULT_MASTERPLAN, DEFAULT_OUTPUT, DEFAULT_TEMPLATE, RunOutcome, run,
};

const USAGE: &str = "usage: architecture-graph-generator [--write|--check] \
[--ssot <path>] [--masterplan <path>] [--template <path>] [--output <path>]";

struct Args {
    ssot: PathBuf,
    masterplan: PathBuf,
    template: PathBuf,
    output: PathBuf,
    check: bool,
}

fn parse_args(mut argv: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut args = Args {
        ssot: PathBuf::from(DEFAULT_GRAPH_SSOT),
        masterplan: PathBuf::from(DEFAULT_MASTERPLAN),
        template: PathBuf::from(DEFAULT_TEMPLATE),
        output: PathBuf::from(DEFAULT_OUTPUT),
        check: false,
    };
    let mut write = false;
    while let Some(flag) = argv.next() {
        match flag.as_str() {
            "--write" => write = true,
            "--check" => args.check = true,
            "--ssot" => args.ssot = PathBuf::from(argv.next().ok_or(USAGE)?),
            "--masterplan" => args.masterplan = PathBuf::from(argv.next().ok_or(USAGE)?),
            "--template" => args.template = PathBuf::from(argv.next().ok_or(USAGE)?),
            "--output" => args.output = PathBuf::from(argv.next().ok_or(USAGE)?),
            "-h" | "--help" => return Err(USAGE.to_string()),
            _ => return Err(USAGE.to_string()),
        }
    }
    if write && args.check {
        return Err("--write and --check are mutually exclusive".to_string());
    }
    Ok(args)
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    match run(
        &args.ssot,
        &args.masterplan,
        &args.template,
        &args.output,
        args.check,
    ) {
        Ok(RunOutcome::Wrote) => {
            println!(
                "architecture-graph: wrote {} from {} + {}",
                args.output.display(),
                args.ssot.display(),
                args.masterplan.display()
            );
            ExitCode::SUCCESS
        }
        Ok(RunOutcome::Clean) => {
            println!(
                "architecture-graph --check passed: {} matches the regenerated dashboard",
                args.output.display()
            );
            ExitCode::SUCCESS
        }
        Ok(RunOutcome::Drifted { committed_path }) => {
            eprintln!(
                "architecture-graph --check failed: {committed_path} drifted from the regenerated dashboard"
            );
            eprintln!("  run `architecture-graph-generator --write` to regenerate it");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("architecture-graph: {error}");
            ExitCode::FAILURE
        }
    }
}
