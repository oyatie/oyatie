//! Fix-3: OAuthSubscription Debug must not expose refresh_token_handle value.
#![cfg_attr(test, allow(clippy::unwrap_used))]

use oya_llm_gateway_oauth_pool_kernel::{
    OAuthSubscription, Provider, SeatId, SubscriptionId, SubscriptionState, TenantId,
};

#[test]
fn debug_does_not_contain_handle_string() {
    let secret_handle = "openbao://t-1/seat-a/refresh-VERY-SECRET";
    let sub = OAuthSubscription {
        tenant_id: TenantId::new("t-1").unwrap(),
        seat_id: SeatId::new("seat-a").unwrap(),
        subscription_id: SubscriptionId::new("sub-1").unwrap(),
        provider: Provider::Anthropic,
        state: SubscriptionState::Active,
        refresh_token_handle: secret_handle.to_string(),
        failure_count: 0,
    };
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
    let sub = OAuthSubscription {
        tenant_id: TenantId::new("tenant-xyz").unwrap(),
        seat_id: SeatId::new("seat-42").unwrap(),
        subscription_id: SubscriptionId::new("sub-99").unwrap(),
        provider: Provider::Anthropic,
        state: SubscriptionState::Active,
        refresh_token_handle: "handle-should-be-hidden".to_string(),
        failure_count: 3,
    };
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
