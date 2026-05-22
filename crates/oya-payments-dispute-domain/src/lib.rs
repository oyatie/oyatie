//! Payments dispute-BC domain — `Dispute` aggregate, `Evidence`,
//! `Representment`.
//!
//! Wave 15-IMPL-truth-up scaffold; full chargeback lifecycle + evidence-
//! window invariant + Intelligence-assisted representment port in IP-009.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Dispute aggregate state machine.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DisputeState {
    Received,
    EvidenceDue,
    EvidenceSubmitted,
    UnderReview,
    Won,
    Lost,
    Accepted,
}

#[allow(dead_code)]
pub struct Dispute {
    state: DisputeState,
}
