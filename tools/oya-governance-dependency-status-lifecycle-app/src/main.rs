//! Dependency-status lifecycle fitness dev-CLI (ADR-0109 instance).
use oya_governance_lifecycle_kernel::cli;
use std::process::ExitCode;

const LANE: &str = "dependency-status-lifecycle";
const DEFAULT_CONFIG: &str = "specs/lifecycle-configs/dependency-status-lifecycle.json";

fn main() -> ExitCode {
    cli::run_default(LANE, DEFAULT_CONFIG)
}
