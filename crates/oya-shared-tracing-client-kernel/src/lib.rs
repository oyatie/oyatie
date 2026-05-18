//! Tracing client kernel — per-µservice trait surface for ADR-0145
//! Invariant 2 (OpenTelemetry trace context propagation).
//!
//! # ADR-0145 Invariant 2
//!
//! Every inter-µservice call MUST propagate the OpenTelemetry trace
//! context for distributed observability. Cross-µservice flow
//! traceability lives in Tempo, not in a central mediator.
//!
//! # Skeleton scope
//!
//! Trait surface only. Production impl tracked under
//! `registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-tracing-client-impl`.
//!
//! # Naming justification
//!
//! `oya-shared-tracing-client-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:tracing-client>-<layer:kernel>`. The
//! `shared` axis is the canonical Oyatie identifier for cross-µservice
//! substrate.
//!
//! # References
//!
//! - ADR-0145 — inter-microservice communication reform.
//! - ADR-0056 — port-in-kernel.
//! - OpenTelemetry spec — W3C Trace Context propagation.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

/// W3C `traceparent` header value. The canonical form is documented in
/// the W3C Trace Context spec.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Traceparent(pub String);

/// Optional W3C `tracestate` header value.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tracestate(pub String);

/// Outbound context the caller asks the kernel to inject into a
/// downstream request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboundContext {
    pub traceparent: Traceparent,
    pub tracestate: Tracestate,
    pub baggage: BTreeMap<String, String>,
}

/// Inbound context the kernel extracted from a received request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InboundContext {
    pub traceparent: Option<Traceparent>,
    pub tracestate: Option<Tracestate>,
    pub baggage: BTreeMap<String, String>,
}

/// Failure surface for the kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TracingClientError {
    SkeletonNotYetImplemented(&'static str),
    MissingTraceparent,
    MalformedTraceparent(String),
}

impl fmt::Display for TracingClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TracingClientError::SkeletonNotYetImplemented(method) => write!(
                f,
                "oya-shared-tracing-client-kernel: {method} is skeleton-only \
                 (tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-tracing-client-impl)"
            ),
            TracingClientError::MissingTraceparent => {
                write!(
                    f,
                    "oya-shared-tracing-client-kernel: traceparent header missing"
                )
            }
            TracingClientError::MalformedTraceparent(value) => {
                write!(
                    f,
                    "oya-shared-tracing-client-kernel: traceparent malformed: {value:?}"
                )
            }
        }
    }
}

impl std::error::Error for TracingClientError {}

/// The trait every µservice integrates to inject/extract W3C trace
/// context for inter-µservice calls.
pub trait TracingClient: Send + Sync {
    /// Inject the current trace context into headers that the caller
    /// will send on the outbound request.
    ///
    /// # Errors
    /// - `SkeletonNotYetImplemented` for the skeleton impl.
    fn inject(&self, headers: &mut BTreeMap<String, String>) -> Result<(), TracingClientError>;

    /// Extract trace context from headers on an inbound request.
    ///
    /// # Errors
    /// - `MissingTraceparent` when the header is absent.
    /// - `MalformedTraceparent` when the header fails the canonical
    ///   W3C parse.
    fn extract(
        &self,
        headers: &BTreeMap<String, String>,
    ) -> Result<InboundContext, TracingClientError>;
}

/// No-op client that round-trips a single fixed traceparent. Useful
/// for compile-time wiring + smoke-tests in dependents.
#[derive(Clone, Debug, Default)]
pub struct NoopTracingClient;

impl TracingClient for NoopTracingClient {
    fn inject(&self, headers: &mut BTreeMap<String, String>) -> Result<(), TracingClientError> {
        // Inject the canonical "all-zero" traceparent so dependents'
        // outbound code paths are exercised; the production impl will
        // resolve the current span's traceparent.
        headers.insert(
            "traceparent".to_string(),
            "00-00000000000000000000000000000000-0000000000000000-00".to_string(),
        );
        Ok(())
    }

    fn extract(
        &self,
        headers: &BTreeMap<String, String>,
    ) -> Result<InboundContext, TracingClientError> {
        let traceparent = headers.get("traceparent").map(|v| Traceparent(v.clone()));
        let tracestate = headers.get("tracestate").map(|v| Tracestate(v.clone()));
        Ok(InboundContext {
            traceparent,
            tracestate,
            baggage: BTreeMap::new(),
        })
    }
}

/// Skeleton "production-equivalent" placeholder that returns
/// `SkeletonNotYetImplemented` for callers that want compile-time
/// proof of unfinished impl.
#[derive(Clone, Debug, Default)]
pub struct SkeletonTracingClient;

impl TracingClient for SkeletonTracingClient {
    fn inject(&self, _headers: &mut BTreeMap<String, String>) -> Result<(), TracingClientError> {
        Err(TracingClientError::SkeletonNotYetImplemented("inject"))
    }

    fn extract(
        &self,
        _headers: &BTreeMap<String, String>,
    ) -> Result<InboundContext, TracingClientError> {
        Err(TracingClientError::SkeletonNotYetImplemented("extract"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_client_injects_and_extracts_round_trip() {
        let client = NoopTracingClient;
        let mut headers = BTreeMap::new();
        client.inject(&mut headers).expect("inject");
        assert!(headers.contains_key("traceparent"));
        let inbound = client.extract(&headers).expect("extract");
        assert!(inbound.traceparent.is_some());
    }

    #[test]
    fn skeleton_client_returns_not_yet_implemented() {
        let client = SkeletonTracingClient;
        let mut headers = BTreeMap::new();
        assert_eq!(
            client.inject(&mut headers),
            Err(TracingClientError::SkeletonNotYetImplemented("inject"))
        );
        assert_eq!(
            client.extract(&headers),
            Err(TracingClientError::SkeletonNotYetImplemented("extract"))
        );
    }

    #[test]
    fn error_display_carries_follow_up_pointer() {
        let err = TracingClientError::SkeletonNotYetImplemented("inject");
        let msg = format!("{err}");
        assert!(msg.contains("adr-0145-tracing-client-impl"));
    }
}
