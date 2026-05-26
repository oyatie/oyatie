//! Capability-status lifecycle fitness dev-CLI (ADR-0109 instance).
use oya_governance_lifecycle_kernel::cli;
use std::process::ExitCode;

const LANE: &str = "capability-status-lifecycle";
const DEFAULT_CONFIG: &str = "specs/lifecycle-configs/capability-status-lifecycle.json";

fn main() -> ExitCode {
    cli::run_default(LANE, DEFAULT_CONFIG)
}
