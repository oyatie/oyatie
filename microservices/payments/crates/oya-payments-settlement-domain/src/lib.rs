//! Payments settlement-BC domain — `SettlementBatch`, `Reconciliation`,
//! `Discrepancy`.
//!
//! Wave 15-IMPL-truth-up scaffold; full reconciliation + TrueTime-anchored
//! period boundaries + SOX dual-signoff invariant in IP-016.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SettlementBatchState {
    Pending,
    Settled,
    Reconciled,
    Discrepant,
}

#[allow(dead_code)]
pub struct SettlementBatch {
    state: SettlementBatchState,
}
