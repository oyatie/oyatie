//! Payments subscription-lifecycle BC domain — `Subscription`,
//! `BillingCycle`, `DunningStep`, `Trial`, `UsageRecord`.
//!
//! Wave 15-IMPL-truth-up scaffold; full state machine + COPPA/KOSA refusal
//! + dunning ladder in IP-011.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SubscriptionState {
    Trialing,
    Active,
    PastDue,
    Unpaid,
    Cancelled,
    Paused,
}

#[allow(dead_code)]
pub struct Subscription {
    state: SubscriptionState,
}
