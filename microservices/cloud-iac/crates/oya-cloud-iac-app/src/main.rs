use std::process::ExitCode;

fn main() -> ExitCode {
    match oya_cloud_iac_app::run_cloud_iac_app_from_env() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
