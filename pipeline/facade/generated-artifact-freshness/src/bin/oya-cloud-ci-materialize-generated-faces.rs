#![forbid(unsafe_code)]

use std::process::ExitCode;

use ci_generated_artifact_freshness::{
    materialize_generated_faces_from_args, parse_materialize_generated_faces_args,
};

fn main() -> ExitCode {
    let args = match parse_materialize_generated_faces_args(std::env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };

    match materialize_generated_faces_from_args(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
