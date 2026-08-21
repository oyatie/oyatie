#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use shared_realtime_transport_kernel::{
    PayloadBudget, RealtimeError, RealtimeMessage, RealtimeSubscription, RealtimeTransport,
    RealtimeTransportTier, SseTransport, WebSocketTransport, enforce_client_tier,
};

#[test]
fn empty_resume_token_is_rejected() {
    let sub = RealtimeSubscription::new("sub_a".into(), RealtimeTransportTier::WebSocket).unwrap();
    assert_eq!(
        sub.with_resume(String::new()),
        Err(RealtimeError::EmptyResumeToken)
    );
}

#[test]
fn websocket_admits_bidirectional_subscription() {
    let ws = WebSocketTransport;
    let sub =
        RealtimeSubscription::new("sub_canvas_collab".into(), RealtimeTransportTier::WebSocket)
            .unwrap();
    assert!(ws.admit(&sub).is_ok());
    assert!(ws.tier().bidirectional());
}

#[test]
fn sse_payload_at_boundary_passes() {
    let sse = SseTransport;
    let budget = PayloadBudget::defaults_for(RealtimeTransportTier::Sse);
    let msg = RealtimeMessage {
        subscription_id: "sub_log_tail".into(),
        event_id: "evt_1".into(),
        payload_bytes: budget.max_payload_bytes,
    };
    assert!(sse.enforce_budget(&msg, budget).is_ok());
}

#[test]
fn grpc_streaming_blocked_for_client_facing_surfaces() {
    assert_eq!(
        enforce_client_tier(RealtimeTransportTier::GrpcStreaming),
        Err(RealtimeError::GrpcStreamingNotClientFacing)
    );
}

#[test]
fn unknown_tier_label_rejected() {
    assert!(matches!(
        RealtimeTransportTier::parse("webtransport"),
        Err(RealtimeError::UnknownTransportTier { .. })
    ));
}
