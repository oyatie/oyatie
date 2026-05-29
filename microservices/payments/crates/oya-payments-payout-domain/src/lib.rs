//! Payments payout-BC domain — `Payout` aggregate, `BankAccount`,
//! `CoolingPeriod`, `PayoutSchedule`.
//!
//! Wave 15-IMPL-truth-up scaffold; full cooling-period + bank-account
//! verification + SOX dual-signoff invariants in IP-007.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Payout aggregate state machine.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PayoutState {
    Pending,
    Scheduled,
    Initiated,
    Completed,
    Failed,
    Reversed,
}

/// Cooling-period value object; tiered defaults land in IP-007.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CoolingPeriod {
    pub days: u16,
}

/// Payout aggregate placeholder; invariants in IP-007.
#[allow(dead_code)]
pub struct Payout {
    state: PayoutState,
}
