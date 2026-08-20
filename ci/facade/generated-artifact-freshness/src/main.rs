#![forbid(unsafe_code)]

use std::process::ExitCode;

use ci_generated_artifact_freshness::{
    check_repo_from_args, parse_freshness_check_args, render_findings,
};

fn main() -> ExitCode {
    let args = match parse_freshness_check_args(std::env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    match check_repo_from_args(&args) {
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
