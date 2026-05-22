//! Payments refund-BC domain — `Refund` aggregate, `RefundReason`,
//! `RefundEvidence`, `RefundRepository` port.
//!
//! Wave 15-IMPL-truth-up scaffold; refund-window invariant + partial-refund
//! accumulation invariant implementation in IP-005.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Refund aggregate state machine. Full transitions land in IP-005.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum RefundState {
    Requested,
    Processing,
    Succeeded,
    Failed,
    Voided,
}

/// Reason taxonomy aligned with Stripe + Adyen.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RefundReason {
    Duplicate,
    Fraudulent,
    RequestedByCustomer,
    Other(String),
}

/// Refund aggregate placeholder; full struct + invariants in IP-005.
#[allow(dead_code)]
pub struct Refund {
    state: RefundState,
}
