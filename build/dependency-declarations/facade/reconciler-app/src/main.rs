//! Non-serving Wave S process entrypoint.

use std::process::ExitCode;

fn main() -> ExitCode {
    dependency_declarations_reconciler_app::structural_not_ready()
}
