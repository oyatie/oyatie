//! Webhook-receiver kernel — ADR-0112 wave-A.
//!
//! This crate is the pure-domain port-in-kernel for the GitHub webhook
//! receiver substrate. It exposes three independent surfaces:
//!
//! 1. [`verify_hmac_sha256`] — verifies an `X-Hub-Signature-256` header
//!    value against the raw payload bytes using a shared webhook secret.
//!    Matches GitHub's documented `sha256=<hex>` scheme; fails closed.
//! 2. [`parse_delivery_log`] + [`find_dedup_status`] — the dedup-table
//!    parser + lookup over the `registry/vcs/webhook-delivery-log.json`
//!    contents. 7-day TTL is applied at lookup time so expired entries
//!    are reported as `Expired` and never block a fresh delivery.
//! 3. [`route_event`] — closed-mapping router lookup over a parsed
//!    event-router table (`(event, action) -> agent`).
//!
//! Discipline (ADR-0056 port-in-kernel + ADR-0083 Tier 1):
//! - No HTTP, no clock, no filesystem, no shelling out.
//! - Deterministic time-of-now is injected by the app via the
//!   `now_seconds` parameter to [`find_dedup_status`].
//! - JSON parsing is intentionally tiny + hand-rolled to avoid pulling
//!   `serde_json` into the kernel. The wire format is fixed by
//!   ADR-0112 §"Idempotency contract" and the registry README — a flat
//!   list of `{delivery_id, event, action, dedup_outcome, at}` rows.
//! - All `unwrap` / `expect` / `panic` use is gated to `cfg(test)`.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use hmac::{Hmac, Mac};
use sha2::Sha256;

// ---------------------------------------------------------------------
// HMAC verification
// ---------------------------------------------------------------------

/// Failure reasons for [`verify_hmac_sha256`].
///
/// All paths fail closed — a caller that bubbles these up to a 4xx
/// response MUST reject the delivery before any dedup-table read so
/// that crafted IDs cannot poison the dedup table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HmacVerificationError {
    /// The `X-Hub-Signature-256` header is missing or empty.
    MissingHeader,
    /// The header is present but doesn't start with the `sha256=`
    /// prefix that GitHub documents.
    UnsupportedScheme,
    /// The hex digest portion of the header is malformed (odd length,
    /// non-hex chars, or wrong byte length for SHA-256 = 32 bytes).
    MalformedDigest,
    /// The webhook secret was rejected by the HMAC primitive — this is
    /// effectively unreachable under `hmac::Hmac::<Sha256>::new_from_slice`
    /// (any byte length is accepted) but we keep the variant so the
    /// constructor's `Result` doesn't disappear into a panic.
    MalformedSecret,
    /// The HMAC was well-formed but did not match the payload.
    SignatureMismatch,
}

/// Verify a GitHub-style `X-Hub-Signature-256` header against a raw
/// payload using a shared webhook secret.
///
/// `signature_header` is the literal header value GitHub sends, e.g.
/// `"sha256=ab12…"`. Comparison uses the constant-time `hmac::Mac::verify_slice`.
pub fn verify_hmac_sha256(
    payload_bytes: &[u8],
    signature_header: &str,
    secret: &str,
) -> Result<(), HmacVerificationError> {
    if signature_header.is_empty() {
        return Err(HmacVerificationError::MissingHeader);
    }
    let Some(hex_digest) = signature_header.strip_prefix("sha256=") else {
        return Err(HmacVerificationError::UnsupportedScheme);
    };
    let expected = decode_hex(hex_digest).ok_or(HmacVerificationError::MalformedDigest)?;
    if expected.len() != 32 {
        return Err(HmacVerificationError::MalformedDigest);
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| HmacVerificationError::MalformedSecret)?;
    mac.update(payload_bytes);
    mac.verify_slice(&expected)
        .map_err(|_| HmacVerificationError::SignatureMismatch)
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Some(out)
}

fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// Dedup table
// ---------------------------------------------------------------------

/// One row of the append-only `registry/vcs/webhook-delivery-log.json`.
///
/// Field set is fixed by ADR-0112 §"Idempotency contract":
/// - `delivery_id` — `X-GitHub-Delivery` UUID; primary dedup key.
/// - `event` / `action` — `X-GitHub-Event` header + payload `action`
///   field; preserved for audit + replay correlation.
/// - `dedup_outcome` — closed enum [`DedupOutcome`] wire-form.
/// - `at_seconds` — Unix epoch seconds when the row was appended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryLogEntry {
    /// data_class: INTERNAL_ONLY
    pub delivery_id: String, // data_class: INTERNAL_ONLY
    /// data_class: INTERNAL_ONLY
    pub event: String, // data_class: INTERNAL_ONLY
    /// data_class: INTERNAL_ONLY
    pub action: String, // data_class: INTERNAL_ONLY
    /// data_class: INTERNAL_ONLY
    pub dedup_outcome: DedupOutcome, // data_class: INTERNAL_ONLY
    /// data_class: INTERNAL_ONLY
    pub at_seconds: u64, // data_class: INTERNAL_ONLY
}

/// Closed enum of dedup outcomes per ADR-0112.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DedupOutcome {
    /// First time this `delivery_id` was seen; routing fired.
    Accepted,
    /// Same `delivery_id` seen before; receiver short-circuited.
    Deduplicated,
    /// `(event, action)` not in the router table; rejected with audit.
    RoutingFailed,
    /// Routed agent invocation returned non-zero; will be retried with
    /// a fresh idempotency key per ADR-0112 §"Crash-safe replay".
    AgentInvocationFailed,
}

impl DedupOutcome {
    /// Canonical snake_case wire-form used in the delivery-log JSON.
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Deduplicated => "deduplicated",
            Self::RoutingFailed => "routing_failed",
            Self::AgentInvocationFailed => "agent_invocation_failed",
        }
    }

    /// Parse the wire-form back into the enum.
    pub fn parse(wire: &str) -> Option<Self> {
        match wire {
            "accepted" => Some(Self::Accepted),
            "deduplicated" => Some(Self::Deduplicated),
            "routing_failed" => Some(Self::RoutingFailed),
            "agent_invocation_failed" => Some(Self::AgentInvocationFailed),
            _ => None,
        }
    }
}

/// Result of a [`find_dedup_status`] lookup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DedupLookup {
    /// `delivery_id` is novel — routing should proceed.
    FirstDelivery,
    /// `delivery_id` was seen with `outcome` at `at_seconds` and is
    /// still within the 7-day TTL — short-circuit.
    Deduplicated {
        outcome: DedupOutcome,
        at_seconds: u64,
    },
    /// `delivery_id` was seen but the row is older than 7 days — the
    /// dedup window has expired so the row should be garbage-collected
    /// and the fresh delivery routed.
    Expired { at_seconds: u64 },
    /// Multiple rows for the same `delivery_id` carry conflicting
    /// outcomes — this is an integrity anomaly; the
    /// `oya-foundry-fitness-webhook-delivery-log-monotonic` lane
    /// alerts on it (ADR-0112 wave-C).
    ConflictingOutcomes,
}

/// Apply the 7-day TTL window from ADR-0112 §"Idempotency contract".
pub const DEDUP_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

/// Find the dedup status for `delivery_id` in a parsed delivery log.
///
/// `now_seconds` is the wall-clock the app injects (Unix epoch seconds).
/// Rows older than `DEDUP_TTL_SECONDS` are reported as [`DedupLookup::Expired`].
pub fn find_dedup_status(
    log: &[DeliveryLogEntry],
    delivery_id: &str,
    now_seconds: u64,
) -> DedupLookup {
    let mut matches: Vec<&DeliveryLogEntry> = log
        .iter()
        .filter(|e| e.delivery_id == delivery_id)
        .collect();
    if matches.is_empty() {
        return DedupLookup::FirstDelivery;
    }

    // Conflicting-outcomes anomaly: two+ rows with the same delivery_id
    // disagreeing on `dedup_outcome`.
    let first_outcome = matches[0].dedup_outcome;
    if matches.iter().any(|e| e.dedup_outcome != first_outcome) {
        return DedupLookup::ConflictingOutcomes;
    }

    // Most-recent row wins for the TTL check.
    matches.sort_by_key(|e| e.at_seconds);
    let latest = matches[matches.len() - 1];
    if now_seconds.saturating_sub(latest.at_seconds) >= DEDUP_TTL_SECONDS {
        DedupLookup::Expired {
            at_seconds: latest.at_seconds,
        }
    } else {
        DedupLookup::Deduplicated {
            outcome: latest.dedup_outcome,
            at_seconds: latest.at_seconds,
        }
    }
}

/// Failure reasons for [`parse_delivery_log`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryLogParseError {
    /// The top-level shape isn't `{"deliveries": [...]}`.
    MalformedEnvelope,
    /// One of the array entries is missing a required field or carries
    /// a non-string value.
    MalformedEntry { index: usize, reason: String },
    /// An entry's `dedup_outcome` value isn't one of the four wire-form
    /// strings.
    UnknownDedupOutcome { index: usize, value: String },
}

/// Parse the textual contents of `registry/vcs/webhook-delivery-log.json`
/// into a strongly-typed list of rows.
///
/// The wire format is intentionally narrow:
///
/// ```text
/// {
///   "deliveries": [
///     {
///       "delivery_id": "uuid",
///       "event": "pull_request",
///       "action": "opened",
///       "dedup_outcome": "accepted",
///       "at_seconds": 1715000000
///     },
///     ...
///   ]
/// }
/// ```
///
/// The kernel ships its own minimal recursive-descent parser to avoid
/// dragging `serde_json` into the port-in-kernel layer; the app crate
/// is the one allowed to depend on `serde_json` for writes.
pub fn parse_delivery_log(text: &str) -> Result<Vec<DeliveryLogEntry>, DeliveryLogParseError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let value = json::parse(trimmed).map_err(|_| DeliveryLogParseError::MalformedEnvelope)?;
    let json::Value::Object(obj) = value else {
        return Err(DeliveryLogParseError::MalformedEnvelope);
    };
    let Some(rows_value) = obj
        .into_iter()
        .find(|(k, _)| k == "deliveries")
        .map(|(_, v)| v)
    else {
        return Err(DeliveryLogParseError::MalformedEnvelope);
    };
    let json::Value::Array(rows) = rows_value else {
        return Err(DeliveryLogParseError::MalformedEnvelope);
    };
    let mut out = Vec::with_capacity(rows.len());
    for (index, row) in rows.into_iter().enumerate() {
        let json::Value::Object(fields) = row else {
            return Err(DeliveryLogParseError::MalformedEntry {
                index,
                reason: "row is not an object".to_string(),
            });
        };
        let mut delivery_id: Option<String> = None;
        let mut event: Option<String> = None;
        let mut action: Option<String> = None;
        let mut outcome_wire: Option<String> = None;
        let mut at_seconds: Option<u64> = None;
        for (k, v) in fields {
            match k.as_str() {
                "delivery_id" => delivery_id = string_of(v),
                "event" => event = string_of(v),
                "action" => action = string_of(v),
                "dedup_outcome" => outcome_wire = string_of(v),
                "at_seconds" => at_seconds = u64_of(v),
                _ => {} // forward-compat: tolerate unknown keys
            }
        }
        let delivery_id = delivery_id.ok_or_else(|| DeliveryLogParseError::MalformedEntry {
            index,
            reason: "missing delivery_id".to_string(),
        })?;
        let event = event.ok_or_else(|| DeliveryLogParseError::MalformedEntry {
            index,
            reason: "missing event".to_string(),
        })?;
        let action = action.ok_or_else(|| DeliveryLogParseError::MalformedEntry {
            index,
            reason: "missing action".to_string(),
        })?;
        let outcome_wire = outcome_wire.ok_or_else(|| DeliveryLogParseError::MalformedEntry {
            index,
            reason: "missing dedup_outcome".to_string(),
        })?;
        let at_seconds = at_seconds.ok_or_else(|| DeliveryLogParseError::MalformedEntry {
            index,
            reason: "missing at_seconds".to_string(),
        })?;
        let dedup_outcome = DedupOutcome::parse(&outcome_wire).ok_or_else(|| {
            DeliveryLogParseError::UnknownDedupOutcome {
                index,
                value: outcome_wire.clone(),
            }
        })?;
        out.push(DeliveryLogEntry {
            delivery_id,
            event,
            action,
            dedup_outcome,
            at_seconds,
        });
    }
    Ok(out)
}

fn string_of(v: json::Value) -> Option<String> {
    match v {
        json::Value::String(s) => Some(s),
        _ => None,
    }
}

fn u64_of(v: json::Value) -> Option<u64> {
    match v {
        json::Value::Number(n) => {
            if n < 0.0 || !n.is_finite() {
                None
            } else {
                Some(n as u64)
            }
        }
        json::Value::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// Event router
// ---------------------------------------------------------------------

/// One row of the `(event, action [, conclusion]) -> agent` router
/// table per ADR-0112 §"Event-router table".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRouterRow {
    /// data_class: INTERNAL_ONLY
    pub event: String, // data_class: INTERNAL_ONLY
    /// `""` (empty string) when ADR-0112 records `—` (no action match
    /// needed) — currently the case for `push`-to-branch entries.
    /// data_class: INTERNAL_ONLY
    pub action: String, // data_class: INTERNAL_ONLY
    /// Optional `conclusion` discriminator per ADR-0112 §"Event-router
    /// table" (rows 4-5: `workflow_run.completed` is split by
    /// success/failure). `None` means the row matches regardless of
    /// payload conclusion; `Some("success")` requires the payload to
    /// carry that conclusion.
    /// data_class: INTERNAL_ONLY
    pub conclusion: Option<String>, // data_class: INTERNAL_ONLY
    /// data_class: INTERNAL_ONLY
    pub agent: String, // data_class: INTERNAL_ONLY
    /// data_class: INTERNAL_ONLY
    pub purpose: String, // data_class: INTERNAL_ONLY
}

/// Look up an event-router row by `(event, action, conclusion)`.
///
/// Matching precedence — most-specific row wins:
/// 1. exact `(event, action, conclusion)` match (row declares a
///    `conclusion` AND the payload carries one matching it),
/// 2. exact `(event, action)` with no `conclusion` declared on the row
///    (action-level row that ignores conclusion),
/// 3. row-side `action` wildcard (empty `action` string).
///
/// `conclusion` is `""` when the payload event doesn't carry a
/// conclusion (most events) — that simply means rule 1 can't fire.
pub fn route_event<'a>(
    event: &str,
    action: &str,
    conclusion: &str,
    table: &'a [EventRouterRow],
) -> Option<&'a EventRouterRow> {
    // 1. exact (event, action, conclusion) — only when the row
    //    declares a non-empty conclusion AND the payload's conclusion
    //    matches.
    if !conclusion.is_empty()
        && let Some(row) = table.iter().find(|r| {
            r.event == event && r.action == action && r.conclusion.as_deref() == Some(conclusion)
        })
    {
        return Some(row);
    }
    // 2. exact (event, action) with no conclusion declared on the row.
    if let Some(row) = table
        .iter()
        .find(|r| r.event == event && r.action == action && r.conclusion.is_none())
    {
        return Some(row);
    }
    // 3. row-side action wildcard.
    table
        .iter()
        .find(|r| r.event == event && r.action.is_empty() && r.conclusion.is_none())
}

// ---------------------------------------------------------------------
// Tiny JSON parser (private)
// ---------------------------------------------------------------------
//
// We keep the parser tiny on purpose: the kernel deliberately avoids
// `serde_json` so the port-in-kernel layer stays dep-light. The app
// crate uses `serde_json` for writes.

mod json {
    use std::iter::Peekable;
    use std::str::Chars;

    #[derive(Clone, Debug, PartialEq)]
    pub(super) enum Value {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<Value>),
        Object(Vec<(String, Value)>),
    }

    #[derive(Debug)]
    pub(super) struct ParseError;

    pub(super) fn parse(text: &str) -> Result<Value, ParseError> {
        let mut chars = text.chars().peekable();
        let value = parse_value(&mut chars)?;
        skip_ws(&mut chars);
        if chars.peek().is_some() {
            return Err(ParseError);
        }
        Ok(value)
    }

    fn parse_value(chars: &mut Peekable<Chars<'_>>) -> Result<Value, ParseError> {
        skip_ws(chars);
        let Some(&c) = chars.peek() else {
            return Err(ParseError);
        };
        match c {
            '{' => parse_object(chars),
            '[' => parse_array(chars),
            '"' => parse_string(chars).map(Value::String),
            't' | 'f' => parse_bool(chars),
            'n' => parse_null(chars),
            '-' | '0'..='9' => parse_number(chars),
            _ => Err(ParseError),
        }
    }

    fn parse_object(chars: &mut Peekable<Chars<'_>>) -> Result<Value, ParseError> {
        let _ = chars.next(); // consume '{'
        let mut entries: Vec<(String, Value)> = Vec::new();
        skip_ws(chars);
        if chars.peek() == Some(&'}') {
            chars.next();
            return Ok(Value::Object(entries));
        }
        loop {
            skip_ws(chars);
            let key = parse_string(chars)?;
            skip_ws(chars);
            if chars.next() != Some(':') {
                return Err(ParseError);
            }
            let value = parse_value(chars)?;
            entries.push((key, value));
            skip_ws(chars);
            match chars.next() {
                Some(',') => continue,
                Some('}') => return Ok(Value::Object(entries)),
                _ => return Err(ParseError),
            }
        }
    }

    fn parse_array(chars: &mut Peekable<Chars<'_>>) -> Result<Value, ParseError> {
        let _ = chars.next(); // consume '['
        let mut entries: Vec<Value> = Vec::new();
        skip_ws(chars);
        if chars.peek() == Some(&']') {
            chars.next();
            return Ok(Value::Array(entries));
        }
        loop {
            let value = parse_value(chars)?;
            entries.push(value);
            skip_ws(chars);
            match chars.next() {
                Some(',') => continue,
                Some(']') => return Ok(Value::Array(entries)),
                _ => return Err(ParseError),
            }
        }
    }

    fn parse_string(chars: &mut Peekable<Chars<'_>>) -> Result<String, ParseError> {
        if chars.next() != Some('"') {
            return Err(ParseError);
        }
        let mut out = String::new();
        while let Some(c) = chars.next() {
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let escaped = chars.next().ok_or(ParseError)?;
                    match escaped {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        'r' => out.push('\r'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000C}'),
                        'u' => {
                            let mut hex = String::with_capacity(4);
                            for _ in 0..4 {
                                hex.push(chars.next().ok_or(ParseError)?);
                            }
                            let code = u32::from_str_radix(&hex, 16).map_err(|_| ParseError)?;
                            if let Some(ch) = char::from_u32(code) {
                                out.push(ch);
                            } else {
                                return Err(ParseError);
                            }
                        }
                        _ => return Err(ParseError),
                    }
                }
                other => out.push(other),
            }
        }
        Err(ParseError)
    }

    fn parse_bool(chars: &mut Peekable<Chars<'_>>) -> Result<Value, ParseError> {
        let mut buf = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_alphabetic() {
                buf.push(c);
                chars.next();
            } else {
                break;
            }
        }
        match buf.as_str() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(ParseError),
        }
    }

    fn parse_null(chars: &mut Peekable<Chars<'_>>) -> Result<Value, ParseError> {
        let mut buf = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_alphabetic() {
                buf.push(c);
                chars.next();
            } else {
                break;
            }
        }
        if buf == "null" {
            Ok(Value::Null)
        } else {
            Err(ParseError)
        }
    }

    fn parse_number(chars: &mut Peekable<Chars<'_>>) -> Result<Value, ParseError> {
        let mut buf = String::new();
        while let Some(&c) = chars.peek() {
            if matches!(c, '-' | '+' | '.' | 'e' | 'E' | '0'..='9') {
                buf.push(c);
                chars.next();
            } else {
                break;
            }
        }
        buf.parse::<f64>()
            .map(Value::Number)
            .map_err(|_| ParseError)
    }

    fn skip_ws(chars: &mut Peekable<Chars<'_>>) {
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Reference HMAC fixture computed offline:
    //   secret = "It's a Secret to Everybody"
    //   payload = b"Hello, World!"
    //   sha256 hmac = 757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17
    const FIXTURE_SECRET: &str = "It's a Secret to Everybody";
    const FIXTURE_PAYLOAD: &[u8] = b"Hello, World!";
    const FIXTURE_DIGEST_HEX: &str =
        "757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";

    fn fixture_header() -> String {
        format!("sha256={FIXTURE_DIGEST_HEX}")
    }

    // ---- HMAC verification ----

    #[test]
    fn valid_hmac_is_accepted() {
        let header = fixture_header();
        let result = verify_hmac_sha256(FIXTURE_PAYLOAD, &header, FIXTURE_SECRET);
        assert!(result.is_ok(), "valid HMAC must verify; got {result:?}");
    }

    #[test]
    fn invalid_hmac_is_rejected() {
        let result = verify_hmac_sha256(b"Hello, World?", &fixture_header(), FIXTURE_SECRET);
        assert_eq!(result, Err(HmacVerificationError::SignatureMismatch));
    }

    #[test]
    fn missing_header_is_rejected() {
        let result = verify_hmac_sha256(FIXTURE_PAYLOAD, "", FIXTURE_SECRET);
        assert_eq!(result, Err(HmacVerificationError::MissingHeader));
    }

    #[test]
    fn malformed_digest_is_rejected() {
        let result = verify_hmac_sha256(FIXTURE_PAYLOAD, "sha256=NOTHEX!!", FIXTURE_SECRET);
        assert_eq!(result, Err(HmacVerificationError::MalformedDigest));
        let result = verify_hmac_sha256(FIXTURE_PAYLOAD, "sha256=deadbeefdeadbeef", FIXTURE_SECRET);
        assert_eq!(result, Err(HmacVerificationError::MalformedDigest));
    }

    #[test]
    fn unsupported_scheme_is_rejected() {
        let result = verify_hmac_sha256(
            FIXTURE_PAYLOAD,
            "sha1=0a0b0c0d0e0f1011121314151617181920212223",
            FIXTURE_SECRET,
        );
        assert_eq!(result, Err(HmacVerificationError::UnsupportedScheme));
    }

    // ---- Dedup table ----

    fn row(delivery_id: &str, outcome: DedupOutcome, at_seconds: u64) -> DeliveryLogEntry {
        DeliveryLogEntry {
            delivery_id: delivery_id.to_string(),
            event: "pull_request".to_string(),
            action: "opened".to_string(),
            dedup_outcome: outcome,
            at_seconds,
        }
    }

    #[test]
    fn first_delivery_is_accepted() {
        let log = vec![row("delivery-001", DedupOutcome::Accepted, 1_000_000)];
        let outcome = find_dedup_status(&log, "delivery-NEW", 1_000_500);
        assert_eq!(outcome, DedupLookup::FirstDelivery);
    }

    #[test]
    fn redelivery_is_deduped() {
        let log = vec![row("delivery-XYZ", DedupOutcome::Accepted, 1_000_000)];
        let outcome = find_dedup_status(&log, "delivery-XYZ", 1_000_500);
        assert_eq!(
            outcome,
            DedupLookup::Deduplicated {
                outcome: DedupOutcome::Accepted,
                at_seconds: 1_000_000
            }
        );
    }

    #[test]
    fn expired_entry_is_reported_as_expired() {
        let log = vec![row("delivery-OLD", DedupOutcome::Accepted, 1_000_000)];
        let outcome = find_dedup_status(&log, "delivery-OLD", 1_000_000 + DEDUP_TTL_SECONDS + 1);
        assert_eq!(
            outcome,
            DedupLookup::Expired {
                at_seconds: 1_000_000
            }
        );
    }

    #[test]
    fn conflicting_outcomes_are_detected() {
        let log = vec![
            row("delivery-CONFLICT", DedupOutcome::Accepted, 1_000_000),
            row(
                "delivery-CONFLICT",
                DedupOutcome::AgentInvocationFailed,
                1_000_010,
            ),
        ];
        let outcome = find_dedup_status(&log, "delivery-CONFLICT", 1_000_500);
        assert_eq!(outcome, DedupLookup::ConflictingOutcomes);
    }

    #[test]
    fn delivery_log_parser_roundtrip() {
        let text = r#"{
            "deliveries": [
                {
                    "delivery_id": "abc-123",
                    "event": "pull_request",
                    "action": "opened",
                    "dedup_outcome": "accepted",
                    "at_seconds": 1715000000
                },
                {
                    "delivery_id": "def-456",
                    "event": "workflow_run",
                    "action": "completed",
                    "dedup_outcome": "agent_invocation_failed",
                    "at_seconds": 1715000100
                }
            ]
        }"#;
        let parsed = parse_delivery_log(text).expect("parses");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].delivery_id, "abc-123");
        assert_eq!(parsed[0].dedup_outcome, DedupOutcome::Accepted);
        assert_eq!(parsed[1].delivery_id, "def-456");
        assert_eq!(parsed[1].dedup_outcome, DedupOutcome::AgentInvocationFailed);
        assert_eq!(parsed[1].at_seconds, 1715000100);
    }

    #[test]
    fn delivery_log_parser_handles_empty() {
        let parsed = parse_delivery_log(r#"{"deliveries": []}"#).expect("parses");
        assert!(parsed.is_empty());
    }

    // ---- Event router ----

    fn router_table() -> Vec<EventRouterRow> {
        vec![
            EventRouterRow {
                event: "pull_request".to_string(),
                action: "opened".to_string(),
                conclusion: None,
                agent: "oya-foundry-vcs-orchestrator-app".to_string(),
                purpose: "Begin changeset state transition to pr_open".to_string(),
            },
            EventRouterRow {
                event: "pull_request".to_string(),
                action: "synchronize".to_string(),
                conclusion: None,
                agent: "merge-queue + IP-005".to_string(),
                purpose: "Fix-at-any-stage re-validate".to_string(),
            },
            EventRouterRow {
                event: "workflow_run".to_string(),
                action: "completed".to_string(),
                conclusion: Some("success".to_string()),
                agent: "IP-004 dispatcher".to_string(),
                purpose: "Run multispectrum review".to_string(),
            },
            EventRouterRow {
                event: "workflow_run".to_string(),
                action: "completed".to_string(),
                conclusion: Some("failure".to_string()),
                agent: "IP-005 dispatcher".to_string(),
                purpose: "Run fix-loop with retry budget".to_string(),
            },
            EventRouterRow {
                event: "push".to_string(),
                action: "".to_string(),
                conclusion: None,
                agent: "promotion workflow".to_string(),
                purpose: "Trigger promotion".to_string(),
            },
        ]
    }

    #[test]
    fn event_router_lookup_hits_exact_match() {
        let table = router_table();
        let hit = route_event("pull_request", "opened", "", &table).expect("hit");
        assert_eq!(hit.agent, "oya-foundry-vcs-orchestrator-app");
    }

    #[test]
    fn event_router_lookup_hits_wildcard_action() {
        let table = router_table();
        let hit = route_event("push", "anything-goes", "", &table).expect("hit");
        assert_eq!(hit.agent, "promotion workflow");
    }

    #[test]
    fn event_router_lookup_misses_for_unknown_event() {
        let table = router_table();
        assert!(route_event("issue_comment", "created", "", &table).is_none());
    }

    #[test]
    fn event_router_splits_workflow_run_by_conclusion_success() {
        let table = router_table();
        let hit = route_event("workflow_run", "completed", "success", &table).expect("hit");
        assert_eq!(hit.agent, "IP-004 dispatcher");
    }

    #[test]
    fn event_router_splits_workflow_run_by_conclusion_failure() {
        let table = router_table();
        let hit = route_event("workflow_run", "completed", "failure", &table).expect("hit");
        assert_eq!(hit.agent, "IP-005 dispatcher");
    }

    #[test]
    fn event_router_workflow_run_cancelled_is_not_routed() {
        // ADR-0112 explicitly says cancelled / timed_out / skipped fall
        // through to RoutingFailed so the completeness lane alerts.
        let table = router_table();
        assert!(route_event("workflow_run", "completed", "cancelled", &table).is_none());
    }
}
