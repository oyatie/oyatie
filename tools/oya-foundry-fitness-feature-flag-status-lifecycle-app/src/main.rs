//! Feature-flag-status lifecycle fitness dev-CLI (ADR-0109 instance).
use oya_foundry_fitness_lifecycle_kernel::cli;
use std::process::ExitCode;

const LANE: &str = "feature-flag-status-lifecycle";
const DEFAULT_CONFIG: &str = "specs/cross-cutting/lifecycle-configs/feature-flag-status-lifecycle.json";

fn main() -> ExitCode {
    cli::run_default(LANE, DEFAULT_CONFIG)
}
