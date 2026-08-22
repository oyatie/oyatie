//! Orchestrator check-daemon stub (Rust-first).
//!
//! Full `buck2 build //...[check]` fan-out remains a follow-on once this crate
//! is workspace-absorbed via integ/build. This binary encodes admission + refuses now.

use process_kit::git_shim::refuse_no_verify;
use process_kit::{detect_env_escapes, require_orchestrator};
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(e) = require_orchestrator() {
        eprintln!("{e}");
        return ExitCode::from(2);
    }
    let escapes = detect_env_escapes(true);
    if !escapes.is_empty() {
        eprintln!("check-daemon: REFUSE — env escapes {escapes:?}");
        return ExitCode::from(3);
    }
    if let Ok(extra) = env::var("SWARM_CHECK_DAEMON_GIT_ARGV") {
        let args: Vec<&str> = extra.split_whitespace().collect();
        if let Err(e) = refuse_no_verify(&args) {
            eprintln!("{e}");
            return ExitCode::from(4);
        }
    }
    eprintln!(
        "check-daemon: OK (stub) — buck target //ci/process-kit:process-kit-check-daemon; \
         full //[check] fan-out pending integ/build membership"
    );
    ExitCode::SUCCESS
}
