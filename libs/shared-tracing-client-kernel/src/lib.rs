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
//! `shared-tracing-client-kernel` follows BNF v4.1:
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

impl Traceparent {
    /// Validate this `Traceparent` against the W3C canonical form and
    /// return a parsed value type on success.
    ///
    /// # Errors
    /// - `MissingTraceparent` when the inner string is empty.
    /// - `MalformedTraceparent` when the header fails the canonical
    ///   W3C parse.
    pub fn validate(&self) -> Result<ParsedTraceparent, TracingClientError> {
        parse_traceparent(&self.0)
    }
}

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
                "shared-tracing-client-kernel: {method} is skeleton-only \
                 (tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-tracing-client-impl)"
            ),
            TracingClientError::MissingTraceparent => {
                write!(
                    f,
                    "shared-tracing-client-kernel: traceparent header missing"
                )
            }
            TracingClientError::MalformedTraceparent(value) => {
                write!(
                    f,
                    "shared-tracing-client-kernel: traceparent malformed: {value:?}"
                )
            }
        }
    }
}

impl std::error::Error for TracingClientError {}

/// A successfully parsed W3C `traceparent` value.
///
/// The canonical wire form is:
/// `00-<32-hex trace-id>-<16-hex parent-id>-<2-hex flags>`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedTraceparent {
    /// 32 lowercase hex characters; never all-zeros.
    pub trace_id: String,
    /// 16 lowercase hex characters; never all-zeros.
    pub parent_id: String,
    /// `true` when the W3C "sampled" flag bit (`flags & 0x01`) is set.
    pub sampled: bool,
}

impl ParsedTraceparent {
    /// Render back to the canonical `traceparent` header value.
    ///
    /// The round-trip property holds: for any value that parses
    /// successfully, `parse_traceparent(value)?.to_header_value() == value`.
    pub fn to_header_value(&self) -> String {
        let flags = if self.sampled { "01" } else { "00" };
        format!("00-{}-{}-{}", self.trace_id, self.parent_id, flags)
    }
}

/// Parse and validate a W3C Trace Context `traceparent` header value.
///
/// # Canonical form
///
/// ```text
/// 00-<32-hex trace-id>-<16-hex parent-id>-<2-hex flags>
/// ```
///
/// # Errors
/// - [`TracingClientError::MissingTraceparent`] — `value` is empty.
/// - [`TracingClientError::MalformedTraceparent`] — any W3C rule
///   violation (wrong version, wrong field count, non-hex chars,
///   wrong-length fields, all-zero trace-id, all-zero parent-id).
pub fn parse_traceparent(value: &str) -> Result<ParsedTraceparent, TracingClientError> {
    if value.is_empty() {
        return Err(TracingClientError::MissingTraceparent);
    }

    let malformed = || TracingClientError::MalformedTraceparent(value.to_string());

    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 4 {
        return Err(malformed());
    }

    let version = parts[0];
    let trace_id = parts[1];
    let parent_id = parts[2];
    let flags_str = parts[3];

    // Version MUST be "00".
    if version != "00" {
        return Err(malformed());
    }

    // trace-id: exactly 32 lowercase hex chars.
    if trace_id.len() != 32 || !is_lowercase_hex(trace_id) {
        return Err(malformed());
    }
    // trace-id MUST NOT be all zeros.
    if trace_id.chars().all(|c| c == '0') {
        return Err(malformed());
    }

    // parent-id: exactly 16 lowercase hex chars.
    if parent_id.len() != 16 || !is_lowercase_hex(parent_id) {
        return Err(malformed());
    }
    // parent-id MUST NOT be all zeros.
    if parent_id.chars().all(|c| c == '0') {
        return Err(malformed());
    }

    // flags: exactly 2 lowercase hex chars.
    if flags_str.len() != 2 || !is_lowercase_hex(flags_str) {
        return Err(malformed());
    }

    // Parse the flags byte to extract the sampled bit.
    let flags_byte = u8::from_str_radix(flags_str, 16).map_err(|_| malformed())?;
    let sampled = (flags_byte & 0x01) == 1;

    Ok(ParsedTraceparent {
        trace_id: trace_id.to_string(),
        parent_id: parent_id.to_string(),
        sampled,
    })
}

/// Returns `true` when `s` consists entirely of lowercase hexadecimal
/// digits (`0-9`, `a-f`).
fn is_lowercase_hex(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

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
///
/// # Note
///
/// The all-zero traceparent injected here
/// (`00-00000000000000000000000000000000-0000000000000000-00`) is
/// intentionally **invalid** per the W3C spec (all-zero trace-id and
/// all-zero parent-id are rejected by `parse_traceparent`). This is
/// the correct no-op behavior: it exercises outbound wiring at
/// compile-time without producing a valid span context. Production
/// impls will inject a real OTel span context.
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

    // ── existing smoke tests ─────────────────────────────────────────

    #[test]
    fn noop_client_injects_and_extracts_round_trip() {
        let client = NoopTracingClient;
        let mut headers = BTreeMap::new();
        client.inject(&mut headers).expect("inject");
        assert!(headers.contains_key("traceparent"));
        let inbound = client.extract(&headers).expect("extract");
        assert!(inbound.traceparent.is_some());
    }

    /// The all-zero value that `NoopTracingClient` injects is
    /// intentionally invalid per the W3C spec. Verify it compiles and
    /// that `parse_traceparent` rejects it as expected.
    #[test]
    fn noop_client_all_zero_value_is_intentionally_invalid_per_spec() {
        let noop_value = "00-00000000000000000000000000000000-0000000000000000-00";
        let result = parse_traceparent(noop_value);
        assert_eq!(
            result,
            Err(TracingClientError::MalformedTraceparent(
                noop_value.to_string()
            )),
            "NoopTracingClient all-zero traceparent must be rejected by parse_traceparent \
             (all-zero trace-id is invalid per W3C spec)"
        );
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

    // ── parse_traceparent acceptance tests ───────────────────────────

    // (a) valid sampled traceparent
    #[test]
    fn parse_valid_sampled_traceparent() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let parsed = parse_traceparent(value).expect("must parse");
        assert_eq!(parsed.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(parsed.parent_id, "00f067aa0ba902b7");
        assert!(parsed.sampled, "flags=01 -> sampled=true");
    }

    // (a) valid non-sampled traceparent
    #[test]
    fn parse_valid_non_sampled_traceparent() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00";
        let parsed = parse_traceparent(value).expect("must parse");
        assert_eq!(parsed.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(parsed.parent_id, "00f067aa0ba902b7");
        assert!(!parsed.sampled, "flags=00 -> sampled=false");
    }

    // (b) wrong version
    #[test]
    fn parse_wrong_version_yields_malformed() {
        let value = "01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        assert_eq!(
            parse_traceparent(value),
            Err(TracingClientError::MalformedTraceparent(value.to_string()))
        );
    }

    // (b) wrong field count — too few
    #[test]
    fn parse_three_fields_yields_malformed() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7";
        assert_eq!(
            parse_traceparent(value),
            Err(TracingClientError::MalformedTraceparent(value.to_string()))
        );
    }

    // (b) wrong field count — too many
    #[test]
    fn parse_five_fields_yields_malformed() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-extra";
        assert_eq!(
            parse_traceparent(value),
            Err(TracingClientError::MalformedTraceparent(value.to_string()))
        );
    }

    // (b) non-hex chars in trace-id
    #[test]
    fn parse_non_hex_trace_id_yields_malformed() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4ZZZ-00f067aa0ba902b7-01";
        assert_eq!(
            parse_traceparent(value),
            Err(TracingClientError::MalformedTraceparent(value.to_string()))
        );
    }

    // (b) non-hex chars in parent-id (uppercase hex is also rejected)
    #[test]
    fn parse_uppercase_hex_parent_id_yields_malformed() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00F067AA0BA902B7-01";
        assert_eq!(
            parse_traceparent(value),
            Err(TracingClientError::MalformedTraceparent(value.to_string()))
        );
    }

    // (b) wrong-length trace-id (31 chars instead of 32)
    #[test]
    fn parse_short_trace_id_yields_malformed() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e47-00f067aa0ba902b7-01";
        assert_eq!(
            parse_traceparent(value),
            Err(TracingClientError::MalformedTraceparent(value.to_string()))
        );
    }

    // (b) wrong-length parent-id (15 chars instead of 16)
    #[test]
    fn parse_short_parent_id_yields_malformed() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b-01";
        assert_eq!(
            parse_traceparent(value),
            Err(TracingClientError::MalformedTraceparent(value.to_string()))
        );
    }

    // (b) wrong-length flags (1 char instead of 2)
    #[test]
    fn parse_short_flags_yields_malformed() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1";
        assert_eq!(
            parse_traceparent(value),
            Err(TracingClientError::MalformedTraceparent(value.to_string()))
        );
    }

    // (b) all-zero trace-id
    #[test]
    fn parse_all_zero_trace_id_yields_malformed() {
        let value = "00-00000000000000000000000000000000-00f067aa0ba902b7-01";
        assert_eq!(
            parse_traceparent(value),
            Err(TracingClientError::MalformedTraceparent(value.to_string()))
        );
    }

    // (b) all-zero parent-id
    #[test]
    fn parse_all_zero_parent_id_yields_malformed() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01";
        assert_eq!(
            parse_traceparent(value),
            Err(TracingClientError::MalformedTraceparent(value.to_string()))
        );
    }

    // (c) empty input
    #[test]
    fn parse_empty_input_yields_missing_traceparent() {
        assert_eq!(
            parse_traceparent(""),
            Err(TracingClientError::MissingTraceparent)
        );
    }

    // (d) round-trip: sampled
    #[test]
    fn round_trip_sampled_traceparent() {
        let original = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let parsed = parse_traceparent(original).expect("must parse");
        assert_eq!(parsed.to_header_value(), original);
    }

    // (d) round-trip: non-sampled
    #[test]
    fn round_trip_non_sampled_traceparent() {
        let original = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00";
        let parsed = parse_traceparent(original).expect("must parse");
        assert_eq!(parsed.to_header_value(), original);
    }

    // Traceparent::validate delegates to parse_traceparent
    #[test]
    fn traceparent_validate_delegates_to_parse() {
        let tp = Traceparent("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_string());
        let parsed = tp.validate().expect("must validate");
        assert_eq!(parsed.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert!(parsed.sampled);
    }

    #[test]
    fn traceparent_validate_empty_yields_missing() {
        let tp = Traceparent(String::new());
        assert_eq!(tp.validate(), Err(TracingClientError::MissingTraceparent));
    }

    // Flags byte 0x03 sets sampled bit
    #[test]
    fn parse_flags_0x03_sampled_true() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-03";
        let parsed = parse_traceparent(value).expect("must parse");
        assert!(parsed.sampled, "flags=03 has sampled bit set");
    }

    // Flags byte 0x02 does NOT set sampled bit
    #[test]
    fn parse_flags_0x02_sampled_false() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-02";
        let parsed = parse_traceparent(value).expect("must parse");
        assert!(!parsed.sampled, "flags=02 has no sampled bit");
    }
}
