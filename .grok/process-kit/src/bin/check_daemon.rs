//! Orchestrator check-daemon stub (Rust-first).
//!
//! Full `buck2 build //...[check]` fan-out remains a follow-on once this crate
//! is buck-wired. This binary encodes admission + env-escape refuse now.

use oya_process_kit::{detect_env_escapes, require_orchestrator};
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
    eprintln!(
        "check-daemon: OK (stub) — buck2 [check] fan-out not yet wired; see .grok/process-kit/README.md"
    );
    ExitCode::SUCCESS
}
