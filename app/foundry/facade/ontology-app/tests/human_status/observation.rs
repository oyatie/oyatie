use crate::facade_support::Fixture;
use crate::human_support::*;
use axum::http::StatusCode;
use foundry_ontology_app::router_with_user_sessions;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[tokio::test]
async fn unreadable_source_after_real_cedar_is_unavailable_without_live_zeros() {
    let fixture = Fixture::new("human-unreadable-observation");
    let state = crate::failing_log::state_with_a_failing_log(
        &fixture.config(),
        "injected unreadable source",
    );
    let (router, reads, pdp) = instrument(state, FakeSessions::new(Ok(view())));
    let refusal = body(
        request(&router, PATH, &[&cookie()]).await,
        StatusCode::SERVICE_UNAVAILABLE,
    )
    .await;
    assert_eq!(refusal["gate"], "observation");
    assert!(refusal.get("projection_lag").is_none());
    assert!(refusal.get("poisoned_entries").is_none());
    assert_eq!(pdp.calls.load(Ordering::SeqCst), 1);
    assert!(
        reads.load(Ordering::SeqCst) > 0,
        "the permitted source read fails; authorization has already completed"
    );
}

#[tokio::test]
async fn contended_source_is_unavailable_then_recovers_after_the_lock_is_released() {
    let fixture = Fixture::new("human-contended-observation");
    let state = Arc::new(fixture.state());
    let router = router_with_user_sessions(state.clone(), FakeSessions::new(Ok(view())));
    let held = state.tenants.get("ten_acme").unwrap().lock().await;
    let refusal = body(
        request(&router, PATH, &[&cookie()]).await,
        StatusCode::SERVICE_UNAVAILABLE,
    )
    .await;
    assert_eq!(refusal["gate"], "observation");
    assert!(refusal.get("projection_lag").is_none());
    drop(held);
    let current = body(request(&router, PATH, &[&cookie()]).await, StatusCode::OK).await;
    assert_eq!(current["observed_tenants"], 1);
    assert_eq!(current["projection_lag"], 0);
}
