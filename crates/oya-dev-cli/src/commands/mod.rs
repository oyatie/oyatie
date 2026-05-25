pub(crate) mod catalog;
pub(crate) mod check;
pub(crate) mod cleanup;
pub(crate) mod codex_thread_sweep;
pub(crate) mod demo;
pub(crate) mod doc;
pub(crate) mod gate;
pub(crate) mod git;
pub(crate) mod lint;
pub(crate) mod onprem;
pub(crate) mod ops;
pub(crate) mod submit;
pub(crate) mod supply_chain;
pub(crate) mod vcs;
pub(crate) mod verify;
pub(crate) mod verify_affected;
// O2 (ADR-0360): nextest JUnit results-ingest for the gate-only overlay.
// O6 (ADR-0360): speculative merge-queue algorithm (ADR-0111 projected state).
// Both are tested decision cores; CLI/runtime wiring layers on top, so the
// public surface is intentionally unused for now.
#[allow(dead_code)]
pub(crate) mod merge_queue;
#[allow(dead_code)]
pub(crate) mod verify_results;
