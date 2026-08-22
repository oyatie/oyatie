//! Binary wrapper for the ADR-0535 gate and local Reindeer semantic-overlay bridge.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_dependency_automation::{
    Verdict, apply_third_party_buck_overlay_file, evaluate_repo, render_findings,
};

enum Command {
    Evaluate { repo_root: PathBuf },
    ApplyThirdPartyOverlay { buck_file: PathBuf },
}

enum ParseOutcome {
    Run(Command),
    Help,
    Error(String),
}

fn main() -> ExitCode {
    let command = match parse_args(std::env::args().skip(1).collect()) {
        ParseOutcome::Run(command) => command,
        ParseOutcome::Help => {
            println!("{}", usage());
            return ExitCode::SUCCESS;
        }
        ParseOutcome::Error(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    match command {
        Command::Evaluate { repo_root } => run_gate(&repo_root),
        Command::ApplyThirdPartyOverlay { buck_file } => {
            match apply_third_party_buck_overlay_file(&buck_file) {
                Ok(patches_applied) => {
                    println!(
                        "third-party Buck overlay: {patches_applied} patch(es) applied to {}",
                        buck_file.display()
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!(
                        "third-party Buck overlay failed for {}: {error}",
                        buck_file.display()
                    );
                    ExitCode::from(2)
                }
            }
        }
    }
}

fn run_gate(repo_root: &Path) -> ExitCode {
    match evaluate_repo(repo_root) {
        Ok(report) => {
            println!("{}", render_findings(&report));
            if report.verdict == Verdict::Green {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("dependency-automation gate failed to run: {error}");
            ExitCode::from(2)
        }
    }
}

fn parse_args(args: Vec<String>) -> ParseOutcome {
    if args.first().map(String::as_str) == Some("apply-third-party-overlay") {
        return parse_overlay_args(args.into_iter().skip(1));
    }

    let mut repo_root = PathBuf::from(".");
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "dependency-automation: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--help" | "-h" => return ParseOutcome::Help,
            other => {
                return ParseOutcome::Error(format!(
                    "dependency-automation: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    ParseOutcome::Run(Command::Evaluate { repo_root })
}

fn parse_overlay_args(mut args: impl Iterator<Item = String>) -> ParseOutcome {
    let mut buck_file = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--buck-file" => {
                let Some(value) = args.next() else {
                    return ParseOutcome::Error(
                        "dependency-automation: --buck-file requires a path".to_owned(),
                    );
                };
                if buck_file.replace(PathBuf::from(value)).is_some() {
                    return ParseOutcome::Error(
                        "dependency-automation: --buck-file may be supplied only once".to_owned(),
                    );
                }
            }
            "--help" | "-h" => return ParseOutcome::Help,
            other => {
                return ParseOutcome::Error(format!(
                    "dependency-automation: unknown overlay argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    match buck_file {
        Some(buck_file) => ParseOutcome::Run(Command::ApplyThirdPartyOverlay { buck_file }),
        None => ParseOutcome::Error(
            "dependency-automation: apply-third-party-overlay requires --buck-file <path>"
                .to_owned(),
        ),
    }
}

fn usage() -> String {
    "usage:\n  cloud-ci-dependency-automation-app-bin [--repo-root <path>]\n  cloud-ci-dependency-automation-app-bin apply-third-party-overlay --buck-file <path>"
        .to_owned()
}
