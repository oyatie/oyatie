#![forbid(unsafe_code)]

use std::process::ExitCode;

use oya_cloud_ci_freshness_app::{
    materialize_generated_faces_with_buck2, parse_materialize_generated_faces_args,
};

fn main() -> ExitCode {
    let args = match parse_materialize_generated_faces_args(std::env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };

    match materialize_generated_faces_with_buck2(&args.repo_root) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
