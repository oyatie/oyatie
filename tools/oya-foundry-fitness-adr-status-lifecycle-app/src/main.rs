//! ADR-status lifecycle fitness dev-CLI (ADR-0109 instance).
//!
//! Wraps the framework kernel's `cli::run_default` with this lane's
//! identity. All discovery + config loading lives in the kernel crate.

use oya_foundry_fitness_lifecycle_kernel::cli;
use std::process::ExitCode;

const LANE: &str = "adr-status-lifecycle";
const DEFAULT_CONFIG: &str = "specs/cross-cutting/lifecycle-configs/adr-status-lifecycle.json";

fn main() -> ExitCode {
    cli::run_default(LANE, DEFAULT_CONFIG)
}
