#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    match oya_doc_staleness_inventory_app::run_from_env() {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("oya-doc-staleness-inventory-app: {error}");
            ExitCode::FAILURE
        }
    }
}
