//! R-DOC command-line gate for the Kubernetes port program.
#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use ci_k8s_program_docs::load_repository;

fn main() -> ExitCode {
    match parse_repo_root(env::args().skip(1).collect()) {
        Ok(repo_root) => match load_repository(&repo_root) {
            Ok(corpus) => {
                let evaluation = ci_k8s_program_docs::evaluate(&corpus);
                let status = if evaluation.is_green() {
                    "GREEN"
                } else {
                    "RED"
                };
                println!(
                    "ci-k8s-program-docs status={status} scanned_population={} finding_count={}",
                    evaluation.counters.scanned_population, evaluation.counters.finding_count
                );
                for finding in &evaluation.findings {
                    println!(
                        "{} {}: {}",
                        finding.code.as_str(),
                        finding.path,
                        finding.message
                    );
                }
                if evaluation.is_green() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                }
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(2)
            }
        },
        Err(message) => {
            eprintln!("R-DOC-ARGUMENT-ERROR: {message}");
            ExitCode::from(2)
        }
    }
}

fn parse_repo_root(arguments: Vec<String>) -> Result<PathBuf, String> {
    match arguments.as_slice() {
        [] => Ok(PathBuf::from(".")),
        [flag, path] if flag == "--repo-root" && !path.is_empty() => Ok(PathBuf::from(path)),
        _ => Err("usage: ci-k8s-program-docs [--repo-root <path>]".to_owned()),
    }
}
