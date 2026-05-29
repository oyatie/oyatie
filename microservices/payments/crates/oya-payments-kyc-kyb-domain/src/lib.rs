//! Payments KYC/KYB BC domain — `SubMerchant` aggregate, `KycKybDocument`,
//! `VerificationResult`, `RestrictedReason`.
//!
//! Wave 15-IMPL-truth-up scaffold; full onboarding lifecycle + AML auto-
//! restrict + KR-PASS enforcement in IP-013.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SubMerchantState {
    Pending,
    UnderReview,
    Verified,
    Restricted,
    Suspended,
    Deactivated,
}

#[allow(dead_code)]
pub struct SubMerchant {
    state: SubMerchantState,
}
