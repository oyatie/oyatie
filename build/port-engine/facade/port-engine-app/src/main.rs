//! Fail-closed CLI stub for W0-B Slice 3. Real subcommands land in Slice 6.
use std::process::ExitCode;

fn main() -> ExitCode {
    if port_engine_app::w0_ready() {
        eprintln!("port-engine-app: Slice 3 driver wired — CLI lands Slice 6");
        ExitCode::from(2)
    } else {
        eprintln!("port-engine-app: driver not ready");
        ExitCode::from(2)
    }
}
