//! Binary wrapper for the ADR-0535 dependency automation gate.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use ci_dependency_automation::{Verdict, evaluate_repo, render_findings};

struct Args {
    repo_root: PathBuf,
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

    match evaluate_repo(&args.repo_root) {
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
    ParseOutcome::Run(Args { repo_root })
}

fn usage() -> String {
    "usage: oya-cloud-ci-dependency-automation-app-bin [--repo-root <path>]".to_owned()
}
