//! Fail-closed CLI stub for W0-B Slice 1. Real subcommands land in Slice 6.
use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!("port-engine-app: W0-B Slice 1 skeleton — driver not ready");
    ExitCode::from(2)
}
