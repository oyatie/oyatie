//! CLI entrypoint for Rust/Buck2 ADR index regeneration.

use std::path::PathBuf;

use oya_adr_index_regenerator_app::{RunMode, repo_root_from_env_or_cwd, run};

fn main() {
    let mut mode = RunMode::Check;
    let mut json = false;
    let mut repo_root = repo_root_from_env_or_cwd();
    let mut args = std::env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--check" => mode = RunMode::Check,
            "--write" => mode = RunMode::Write,
            "--json" => json = true,
            "--repo-root" => {
                let Some(value) = args.next() else {
                    eprintln!("--repo-root requires a path");
                    std::process::exit(64);
                };
                repo_root = PathBuf::from(value);
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            other => {
                eprintln!("unknown argument: {other}");
                print_help();
                std::process::exit(64);
            }
        }
    }

    match run(&repo_root, mode) {
        Ok(report) => {
            if json {
                print!("{}", report.to_json());
            } else {
                print!("{}", report.to_text());
            }
            if mode == RunMode::Check && !report.clean() {
                std::process::exit(2);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        "Rust/Buck2 ADR index regenerator\n\nUsage: oya-adr-index-regenerator-app [--check|--write] [--json] [--repo-root PATH]\n\n--check      verify docs/ADR-INDEX.md and docs/machine-readable/decisions.json match docs/decisions/ (default)\n--write      regenerate the committed artifacts\n--json       emit a machine-readable run report\n--repo-root  repository root, defaults to OYA_REPO_ROOT or current directory"
    );
}
