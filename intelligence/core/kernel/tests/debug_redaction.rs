//! Fix-3: OAuthSubscription Debug must not expose refresh_token_handle value.
#![cfg_attr(test, allow(clippy::unwrap_used))]

use intelligence_kernel::{
    OAuthSubscription, Provider, SeatId, SubscriptionId, SubscriptionState, TenantId,
};

#[test]
fn debug_does_not_contain_handle_string() {
    let secret_handle = "secret-ref://t-1/seat-a/refresh-VERY-SECRET";
    let sub = OAuthSubscription::new(
        TenantId::new("t-1").unwrap(),
        SeatId::new("seat-a").unwrap(),
        SubscriptionId::new("sub-1").unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        secret_handle.to_string(),
        0,
    );
    let debug_output = format!("{sub:?}");
    assert!(
        !debug_output.contains(secret_handle),
        "Debug output must not contain the refresh token handle value; got: {debug_output}"
    );
    assert!(
        debug_output.contains("<REDACTED>"),
        "Debug output must contain '<REDACTED>'; got: {debug_output}"
    );
}

#[test]
fn debug_contains_non_secret_fields() {
    let sub = OAuthSubscription::new(
        TenantId::new("tenant-xyz").unwrap(),
        SeatId::new("seat-42").unwrap(),
        SubscriptionId::new("sub-99").unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        "handle-should-be-hidden".to_string(),
        3,
    );
    let debug_output = format!("{sub:?}");
    // Non-secret fields must still appear.
    assert!(
        debug_output.contains("seat-42"),
        "seat_id should appear in debug"
    );
    assert!(
        debug_output.contains("sub-99"),
        "subscription_id should appear in debug"
    );
    assert!(
        debug_output.contains("3"),
        "failure_count should appear in debug"
    );
    // Secret must not appear.
    assert!(!debug_output.contains("handle-should-be-hidden"));
}
