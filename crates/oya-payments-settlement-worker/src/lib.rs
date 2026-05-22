//! Payments settlement-BC worker — daily reconciliation CronJob.
//!
//! Wave 15-IMPL-truth-up scaffold; full PSP settlement-report ingest +
//! per-PSP parallelism in IP-017.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[allow(dead_code)]
pub struct ReconciliationWorker {
    _placeholder: (),
}
