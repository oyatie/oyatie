#![forbid(unsafe_code)]

use std::process::ExitCode;

use ci_generated_artifact_freshness::{parse_face_settle_args, run_face_settle_with_buck2};

fn main() -> ExitCode {
    let args = match parse_face_settle_args(std::env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };

    match run_face_settle_with_buck2(&args.repo_root, args.mode) {
        Ok(report) => {
            println!("{}", report.message);
            if report.is_success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
