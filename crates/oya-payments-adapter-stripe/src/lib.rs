//! Stripe PSP adapter — `PspAdapter` implementation for Stripe Connect.
//!
//! Wave 15-IMPL-truth-up scaffold; full HTTP/3 + HMAC-SHA256 webhook
//! verification + OpenBao BYOK secret retrieval in IP-004.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// StripeAdapter implements PspAdapter for the Stripe platform-
/// facilitator pattern. Concrete HTTP wiring lands in IP-004.
#[allow(dead_code)]
pub struct StripeAdapter {
    _placeholder: (),
}
