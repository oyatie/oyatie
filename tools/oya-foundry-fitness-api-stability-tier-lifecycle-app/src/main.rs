//! API-stability-tier lifecycle fitness dev-CLI (ADR-0109 instance).
use oya_foundry_fitness_lifecycle_kernel::cli;
use std::process::ExitCode;

const LANE: &str = "api-stability-tier-lifecycle";
const DEFAULT_CONFIG: &str = "specs/lifecycle-configs/api-stability-tier-lifecycle.json";

fn main() -> ExitCode {
    cli::run_default(LANE, DEFAULT_CONFIG)
}
