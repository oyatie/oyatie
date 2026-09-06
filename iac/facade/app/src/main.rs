use std::process::ExitCode;

fn main() -> ExitCode {
    match iac_app::run_iac_app_with_signals_from_env() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
