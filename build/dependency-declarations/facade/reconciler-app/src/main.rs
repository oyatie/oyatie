use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!(
        "dependency declaration process adapter is not qualified; use the typed reconciler API"
    );
    ExitCode::FAILURE
}
