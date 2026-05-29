//! Shared realtime transport kernel — ADR-0208 tier model.
//!
//! Pure model describing the canonical realtime transport tier:
//!
//! - **SSE (Server-Sent Events)** — one-way server-to-client streams
//!   (log tail, metric tail, AI streaming responses, status feed).
//! - **WebSocket** — bidirectional product surfaces (Workflow Studio
//!   canvas collab, shared cursors, in-product chat).
//! - **gRPC streaming** — service-to-service streams only; NOT
//!   client-facing. Covered by ADR-0145.
//!
//! The kernel does no I/O. It enforces (a) the transport-tier enum is
//! closed and (b) a `RealtimeMessage` carries a stable subscription
//! identifier + last-event-id resume token + payload-size budget. The
//! wire-level axum/hyper concrete impls live in adapters.
//!
//! ADR-0083 Tier 3 test exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Transport tier enum — closed. Order is canonical (SSE first because
/// it is the default for one-way; WebSocket second for bidi; gRPC
/// streaming last and only for service-to-service).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RealtimeTransportTier {
    Sse,
    WebSocket,
    GrpcStreaming,
}

impl RealtimeTransportTier {
    pub const fn wire_label(self) -> &'static str {
        match self {
            Self::Sse => "sse",
            Self::WebSocket => "websocket",
            Self::GrpcStreaming => "grpc-streaming",
        }
    }

    pub fn parse(label: &str) -> Result<Self, RealtimeError> {
        match label {
            "sse" => Ok(Self::Sse),
            "websocket" => Ok(Self::WebSocket),
            "grpc-streaming" => Ok(Self::GrpcStreaming),
            other => Err(RealtimeError::UnknownTransportTier {
                label: other.to_owned(),
            }),
        }
    }

    /// Whether the tier is allowed for client-facing surfaces.
    /// gRPC streaming is service-to-service only.
    pub const fn client_facing(self) -> bool {
        matches!(self, Self::Sse | Self::WebSocket)
    }

    /// Whether this tier supports bidirectional traffic from the
    /// kernel's perspective (SSE is one-way; the others are bi-di).
    pub const fn bidirectional(self) -> bool {
        matches!(self, Self::WebSocket | Self::GrpcStreaming)
    }

    pub const fn all() -> [Self; 3] {
        [Self::Sse, Self::WebSocket, Self::GrpcStreaming]
    }
}

/// A subscription / stream descriptor — the kernel-level handle that
/// adapters resolve to an axum SSE stream or tokio-tungstenite
/// WebSocket. Identifier is stable across reconnects so adapters can
/// resume.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealtimeSubscription {
    pub subscription_id: String,       // data_class: INTERNAL_ONLY
    pub tier: RealtimeTransportTier,   // data_class: INTERNAL_ONLY
    pub last_event_id: Option<String>, // data_class: INTERNAL_ONLY
}

impl RealtimeSubscription {
    pub fn new(
        subscription_id: String,
        tier: RealtimeTransportTier,
    ) -> Result<Self, RealtimeError> {
        validate_subscription_id(&subscription_id)?;
        Ok(Self {
            subscription_id,
            tier,
            last_event_id: None,
        })
    }

    pub fn with_resume(mut self, last_event_id: String) -> Result<Self, RealtimeError> {
        if last_event_id.is_empty() {
            return Err(RealtimeError::EmptyResumeToken);
        }
        self.last_event_id = Some(last_event_id);
        Ok(self)
    }
}

/// A single realtime message. Keep payload as bytes; serialization
/// (protobuf, JSON, CBOR) is an adapter concern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealtimeMessage {
    pub subscription_id: String, // data_class: INTERNAL_ONLY
    pub event_id: String,        // data_class: INTERNAL_ONLY
    pub payload_bytes: usize,    // data_class: INTERNAL_ONLY
}

/// Per-tier payload-size envelope. Kernel enforces caps before
/// adapters open the wire.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PayloadBudget {
    pub max_payload_bytes: usize,
}

impl PayloadBudget {
    /// Hyperscaler default: SSE 1 MiB / WS 4 MiB / gRPC 16 MiB. Tuned
    /// per-µservice in manifest; kernel ships these defaults so adapters
    /// fail closed rather than allocate unbounded.
    pub const fn defaults_for(tier: RealtimeTransportTier) -> Self {
        match tier {
            RealtimeTransportTier::Sse => Self {
                max_payload_bytes: 1 << 20,
            },
            RealtimeTransportTier::WebSocket => Self {
                max_payload_bytes: 4 << 20,
            },
            RealtimeTransportTier::GrpcStreaming => Self {
                max_payload_bytes: 16 << 20,
            },
        }
    }
}

/// Realtime transport trait — adapters implement.
pub trait RealtimeTransport {
    fn tier(&self) -> RealtimeTransportTier;

    /// Accept a new subscription. Returns `Ok` if accepted; `Err` when
    /// the subscription identifier is malformed or the resume token
    /// no longer maps to a server-side cursor.
    fn admit(&self, subscription: &RealtimeSubscription) -> Result<(), RealtimeError>;

    /// Enforce per-message payload budget. Adapters call this before
    /// emitting on the wire.
    fn enforce_budget(
        &self,
        message: &RealtimeMessage,
        budget: PayloadBudget,
    ) -> Result<(), RealtimeError> {
        if message.payload_bytes > budget.max_payload_bytes {
            return Err(RealtimeError::PayloadBudgetExceeded {
                actual_bytes: message.payload_bytes,
                budget_bytes: budget.max_payload_bytes,
            });
        }
        Ok(())
    }
}

/// In-kernel SSE transport — pure model, no I/O. Adapters wrap real
/// axum SSE streams; this default exists so kernel-level tests can
/// exercise the trait without dragging in adapter dependencies.
#[derive(Clone, Debug, Default)]
pub struct SseTransport;

impl RealtimeTransport for SseTransport {
    fn tier(&self) -> RealtimeTransportTier {
        RealtimeTransportTier::Sse
    }
    fn admit(&self, subscription: &RealtimeSubscription) -> Result<(), RealtimeError> {
        if subscription.tier != RealtimeTransportTier::Sse {
            return Err(RealtimeError::TierMismatch {
                expected: RealtimeTransportTier::Sse,
                actual: subscription.tier,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct WebSocketTransport;

impl RealtimeTransport for WebSocketTransport {
    fn tier(&self) -> RealtimeTransportTier {
        RealtimeTransportTier::WebSocket
    }
    fn admit(&self, subscription: &RealtimeSubscription) -> Result<(), RealtimeError> {
        if subscription.tier != RealtimeTransportTier::WebSocket {
            return Err(RealtimeError::TierMismatch {
                expected: RealtimeTransportTier::WebSocket,
                actual: subscription.tier,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RealtimeError {
    UnknownTransportTier {
        label: String,
    },
    EmptySubscriptionId,
    MalformedSubscriptionId {
        subscription_id: String,
    },
    EmptyResumeToken,
    TierMismatch {
        expected: RealtimeTransportTier,
        actual: RealtimeTransportTier,
    },
    PayloadBudgetExceeded {
        actual_bytes: usize,
        budget_bytes: usize,
    },
    GrpcStreamingNotClientFacing,
}

impl RealtimeError {
    pub fn message(&self) -> String {
        match self {
            Self::UnknownTransportTier { label } => {
                format!("unknown realtime transport tier label: {label}")
            }
            Self::EmptySubscriptionId => "subscription id is empty".to_owned(),
            Self::MalformedSubscriptionId { subscription_id } => {
                format!("subscription id malformed: {subscription_id}")
            }
            Self::EmptyResumeToken => "resume token is empty".to_owned(),
            Self::TierMismatch { expected, actual } => format!(
                "transport tier mismatch: expected {} got {}",
                expected.wire_label(),
                actual.wire_label()
            ),
            Self::PayloadBudgetExceeded {
                actual_bytes,
                budget_bytes,
            } => format!("payload budget exceeded: actual={actual_bytes}B budget={budget_bytes}B"),
            Self::GrpcStreamingNotClientFacing => {
                "gRPC streaming is not allowed for client-facing surfaces".to_owned()
            }
        }
    }
}

/// Refuse gRPC streaming for client-facing surfaces.
pub fn enforce_client_tier(tier: RealtimeTransportTier) -> Result<(), RealtimeError> {
    if !tier.client_facing() {
        return Err(RealtimeError::GrpcStreamingNotClientFacing);
    }
    Ok(())
}

fn validate_subscription_id(id: &str) -> Result<(), RealtimeError> {
    if id.is_empty() {
        return Err(RealtimeError::EmptySubscriptionId);
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(RealtimeError::MalformedSubscriptionId {
            subscription_id: id.to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_labels_are_distinct_and_round_trip() {
        for tier in RealtimeTransportTier::all() {
            assert_eq!(RealtimeTransportTier::parse(tier.wire_label()), Ok(tier));
        }
        assert!(matches!(
            RealtimeTransportTier::parse("long-polling"),
            Err(RealtimeError::UnknownTransportTier { .. })
        ));
    }

    #[test]
    fn grpc_streaming_is_not_client_facing() {
        assert!(RealtimeTransportTier::Sse.client_facing());
        assert!(RealtimeTransportTier::WebSocket.client_facing());
        assert!(!RealtimeTransportTier::GrpcStreaming.client_facing());

        assert_eq!(
            enforce_client_tier(RealtimeTransportTier::GrpcStreaming),
            Err(RealtimeError::GrpcStreamingNotClientFacing)
        );
        assert!(enforce_client_tier(RealtimeTransportTier::Sse).is_ok());
        assert!(enforce_client_tier(RealtimeTransportTier::WebSocket).is_ok());
    }

    #[test]
    fn bidirectional_flag_marks_websocket_and_grpc() {
        assert!(!RealtimeTransportTier::Sse.bidirectional());
        assert!(RealtimeTransportTier::WebSocket.bidirectional());
        assert!(RealtimeTransportTier::GrpcStreaming.bidirectional());
    }

    #[test]
    fn subscription_validates_id_and_supports_resume() {
        let sub =
            RealtimeSubscription::new("sub_canvas_42".into(), RealtimeTransportTier::WebSocket)
                .unwrap();
        assert_eq!(sub.last_event_id, None);
        let resumed = sub.with_resume("evt_99".into()).unwrap();
        assert_eq!(resumed.last_event_id.as_deref(), Some("evt_99"));

        assert!(matches!(
            RealtimeSubscription::new(String::new(), RealtimeTransportTier::Sse),
            Err(RealtimeError::EmptySubscriptionId)
        ));
        assert!(matches!(
            RealtimeSubscription::new("has space".into(), RealtimeTransportTier::Sse),
            Err(RealtimeError::MalformedSubscriptionId { .. })
        ));
    }

    #[test]
    fn payload_budget_defaults_match_tier_caps() {
        assert_eq!(
            PayloadBudget::defaults_for(RealtimeTransportTier::Sse).max_payload_bytes,
            1 << 20
        );
        assert_eq!(
            PayloadBudget::defaults_for(RealtimeTransportTier::WebSocket).max_payload_bytes,
            4 << 20
        );
        assert_eq!(
            PayloadBudget::defaults_for(RealtimeTransportTier::GrpcStreaming).max_payload_bytes,
            16 << 20
        );
    }

    #[test]
    fn enforce_budget_rejects_oversized_payload() {
        let t = SseTransport;
        let msg = RealtimeMessage {
            subscription_id: "sub_x".into(),
            event_id: "evt_1".into(),
            payload_bytes: 2 << 20, // 2 MiB > SSE 1 MiB default
        };
        let budget = PayloadBudget::defaults_for(RealtimeTransportTier::Sse);
        assert!(matches!(
            t.enforce_budget(&msg, budget),
            Err(RealtimeError::PayloadBudgetExceeded { .. })
        ));
    }

    #[test]
    fn admit_rejects_tier_mismatch() {
        let sub = RealtimeSubscription::new("sub_x".into(), RealtimeTransportTier::Sse).unwrap();
        let ws = WebSocketTransport;
        assert!(matches!(
            ws.admit(&sub),
            Err(RealtimeError::TierMismatch { .. })
        ));
        let sse = SseTransport;
        assert!(sse.admit(&sub).is_ok());
    }
}
