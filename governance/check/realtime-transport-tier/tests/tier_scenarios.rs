#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use check_realtime_transport_tier::{
    RealtimeStreamDeclaration, SurfaceKind, TierGapReason, TrafficDirection, check,
};

fn d(
    ms: &str,
    sid: &str,
    tier: &str,
    dir: TrafficDirection,
    surf: SurfaceKind,
) -> RealtimeStreamDeclaration {
    RealtimeStreamDeclaration {
        microservice: ms.into(),
        stream_id: sid.into(),
        tier_label: tier.into(),
        direction: dir,
        surface: surf,
    }
}

#[test]
fn mixed_streams_emit_only_violating_gaps() {
    let r = check(&[
        d(
            "ms-a",
            "ok1",
            "sse",
            TrafficDirection::OneWay,
            SurfaceKind::ClientFacing,
        ),
        d(
            "ms-a",
            "bad1",
            "sse",
            TrafficDirection::Bidirectional,
            SurfaceKind::ClientFacing,
        ),
        d(
            "ms-b",
            "ok2",
            "websocket",
            TrafficDirection::Bidirectional,
            SurfaceKind::ClientFacing,
        ),
        d(
            "ms-b",
            "bad2",
            "grpc-streaming",
            TrafficDirection::OneWay,
            SurfaceKind::ClientFacing,
        ),
    ]);
    assert_eq!(r.streams_checked, 4);
    assert_eq!(r.gaps.len(), 2);
}

#[test]
fn empty_input_yields_empty_report() {
    let r = check(&[]);
    assert_eq!(r.streams_checked, 0);
    assert!(r.gaps.is_empty());
}

#[test]
fn long_polling_label_is_unknown() {
    let r = check(&[d(
        "ms-a",
        "feed",
        "long-polling",
        TrafficDirection::OneWay,
        SurfaceKind::ClientFacing,
    )]);
    assert_eq!(r.gaps.len(), 1);
    assert!(matches!(
        r.gaps[0].reason,
        TierGapReason::UnknownTierLabel { .. }
    ));
}

#[test]
fn sse_bidirectional_service_to_service_still_flagged() {
    let r = check(&[d(
        "ms-internal",
        "evt",
        "sse",
        TrafficDirection::Bidirectional,
        SurfaceKind::ServiceToService,
    )]);
    assert_eq!(r.gaps.len(), 1);
    assert_eq!(r.gaps[0].reason, TierGapReason::BidirectionalOnSse);
}

#[test]
fn websocket_service_to_service_one_way_flagged() {
    let r = check(&[d(
        "ms-internal",
        "evt",
        "websocket",
        TrafficDirection::OneWay,
        SurfaceKind::ServiceToService,
    )]);
    assert_eq!(r.gaps.len(), 1);
    assert_eq!(r.gaps[0].reason, TierGapReason::WebSocketOnOneWaySurface);
}
