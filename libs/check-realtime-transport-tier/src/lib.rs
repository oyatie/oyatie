//! ADR-0208 realtime transport tier discipline gate.
//!
//! Advisory lane that scans per-µservice realtime stream declarations
//! and flags:
//!
//! 1. Bidirectional traffic on SSE (must promote to WebSocket).
//! 2. WebSocket used for one-way streams that SSE handles (cost waste +
//!    complexity).
//! 3. gRPC streaming used for client-facing surfaces (forbidden per
//!    ADR-0208).
//! 4. Unknown transport-tier labels.
//!
//! Pure model; no I/O.
//!
//! ADR-0083 Tier 3 test exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use shared_realtime_transport_kernel::{
    RealtimeError, RealtimeTransportTier, enforce_client_tier,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceKind {
    ClientFacing,
    ServiceToService,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrafficDirection {
    OneWay,
    Bidirectional,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealtimeStreamDeclaration {
    pub microservice: String,
    pub stream_id: String,
    pub tier_label: String,
    pub direction: TrafficDirection,
    pub surface: SurfaceKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierGap {
    pub microservice: String,
    pub stream_id: String,
    pub reason: TierGapReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TierGapReason {
    UnknownTierLabel { label: String },
    BidirectionalOnSse,
    GrpcStreamingOnClientSurface,
    WebSocketOnOneWaySurface,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierReport {
    pub streams_checked: usize,
    pub gaps: Vec<TierGap>,
}

pub fn check(decls: &[RealtimeStreamDeclaration]) -> TierReport {
    let mut gaps = Vec::new();
    for d in decls {
        let tier = match RealtimeTransportTier::parse(&d.tier_label) {
            Ok(t) => t,
            Err(RealtimeError::UnknownTransportTier { label }) => {
                gaps.push(TierGap {
                    microservice: d.microservice.clone(),
                    stream_id: d.stream_id.clone(),
                    reason: TierGapReason::UnknownTierLabel { label },
                });
                continue;
            }
            Err(_) => continue,
        };

        if let (SurfaceKind::ClientFacing, RealtimeTransportTier::GrpcStreaming) = (d.surface, tier)
        {
            // enforce_client_tier covers this; explicit gap.
            let _ = enforce_client_tier(tier);
            gaps.push(TierGap {
                microservice: d.microservice.clone(),
                stream_id: d.stream_id.clone(),
                reason: TierGapReason::GrpcStreamingOnClientSurface,
            });
            continue;
        }

        match (d.direction, tier) {
            (TrafficDirection::Bidirectional, RealtimeTransportTier::Sse) => {
                gaps.push(TierGap {
                    microservice: d.microservice.clone(),
                    stream_id: d.stream_id.clone(),
                    reason: TierGapReason::BidirectionalOnSse,
                });
            }
            (TrafficDirection::OneWay, RealtimeTransportTier::WebSocket) => {
                gaps.push(TierGap {
                    microservice: d.microservice.clone(),
                    stream_id: d.stream_id.clone(),
                    reason: TierGapReason::WebSocketOnOneWaySurface,
                });
            }
            _ => {}
        }
    }
    TierReport {
        streams_checked: decls.len(),
        gaps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn sse_one_way_client_passes() {
        let r = check(&[d(
            "observability",
            "log_tail",
            "sse",
            TrafficDirection::OneWay,
            SurfaceKind::ClientFacing,
        )]);
        assert!(r.gaps.is_empty());
    }

    #[test]
    fn websocket_bidi_client_passes() {
        let r = check(&[d(
            "workflow-studio",
            "canvas_collab",
            "websocket",
            TrafficDirection::Bidirectional,
            SurfaceKind::ClientFacing,
        )]);
        assert!(r.gaps.is_empty());
    }

    #[test]
    fn grpc_streaming_client_facing_flagged() {
        let r = check(&[d(
            "workflow-studio",
            "internal_step_events",
            "grpc-streaming",
            TrafficDirection::OneWay,
            SurfaceKind::ClientFacing,
        )]);
        assert_eq!(r.gaps.len(), 1);
        assert_eq!(
            r.gaps[0].reason,
            TierGapReason::GrpcStreamingOnClientSurface
        );
    }

    #[test]
    fn bidi_on_sse_flagged() {
        let r = check(&[d(
            "messenger",
            "chat_room",
            "sse",
            TrafficDirection::Bidirectional,
            SurfaceKind::ClientFacing,
        )]);
        assert_eq!(r.gaps.len(), 1);
        assert_eq!(r.gaps[0].reason, TierGapReason::BidirectionalOnSse);
    }

    #[test]
    fn websocket_one_way_flagged() {
        let r = check(&[d(
            "observability",
            "log_tail",
            "websocket",
            TrafficDirection::OneWay,
            SurfaceKind::ClientFacing,
        )]);
        assert_eq!(r.gaps.len(), 1);
        assert_eq!(r.gaps[0].reason, TierGapReason::WebSocketOnOneWaySurface);
    }

    #[test]
    fn unknown_label_flagged() {
        let r = check(&[d(
            "ms-x",
            "weird",
            "webtransport",
            TrafficDirection::Bidirectional,
            SurfaceKind::ClientFacing,
        )]);
        assert_eq!(r.gaps.len(), 1);
        assert!(matches!(
            r.gaps[0].reason,
            TierGapReason::UnknownTierLabel { .. }
        ));
    }

    #[test]
    fn grpc_streaming_service_to_service_passes() {
        let r = check(&[d(
            "workflow-studio",
            "step_event_stream",
            "grpc-streaming",
            TrafficDirection::Bidirectional,
            SurfaceKind::ServiceToService,
        )]);
        assert!(r.gaps.is_empty());
    }
}
