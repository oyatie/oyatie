#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use oya_cloud_ci_freshness_app::{check_repo, render_findings};

fn main() -> ExitCode {
    let repo_root = match parse_repo_root(std::env::args().skip(1).collect()) {
        Ok(repo_root) => repo_root,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    match check_repo(&repo_root) {
        Ok(report) => {
            println!("{}", render_findings(&report.findings));
            if report.is_green() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("freshness gate failed to run: {error}");
            ExitCode::FAILURE
        }
    }
}

fn parse_repo_root(args: Vec<String>) -> Result<PathBuf, String> {
    let mut repo_root = PathBuf::from(".");
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return Err("freshness gate: --repo-root requires a path".to_owned());
                };
                repo_root = PathBuf::from(value);
            }
            "--help" | "-h" => {
                return Err("usage: oya-cloud-ci-freshness-app [--repo-root <path>]".to_owned());
            }
            other => {
                return Err(format!(
                    "freshness gate: unknown argument {other:?}; usage: oya-cloud-ci-freshness-app [--repo-root <path>]"
                ));
            }
        }
    }
    Ok(repo_root)
}
