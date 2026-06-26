//! Canonical provider-error taxonomy (pure kernel).
//!
//! litellm §9 gap #25 (exceptions_error_mapping) + findings §10 P1
//! status-code→cooldown matrix: every provider's HTTP/body error is folded into
//! one canonical [`ErrorClass`] so the cooldown/retry/fallback machinery acts on
//! a typed predicate instead of guessing per call site.
//!
//! ZERO I/O — classification is a pure function of `(status, body, headers)`.
//! The *action* taken on a class (cooldown vs blacklist vs failover) is the
//! pool's job ([`crate::SubscriptionPool::record_outcome`]); this module only
//! names the class and whether retrying the same request can help.
//!
//! Sources (see `cloud/cloud-intelligence/design/reference-dissection-findings.md`):
//! - §4(B) — Codex `usage_limit_reached` (quota exhausted, non-retryable now)
//!   vs transient `rate_limit_error` (retryable with backoff).
//! - §5 — CLIProxyAPI status→reason ladder (401/403 auth, 429 quota, 5xx transient).
//! - §8 — Codex refresh is non-retryable on `refresh_token_reused`.
//! - RFC 6749 §5.2 OAuth token error codes (`invalid_grant`,
//!   `temporarily_unavailable`, `server_error`).

use crate::SeatOutcome;

/// Canonical, provider-agnostic classification of an upstream error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorClass {
    /// 2xx — not an error.
    Ok,
    /// HTTP 429. `transient` distinguishes a short-lived `rate_limit_error`
    /// (retryable with backoff) from `usage_limit_reached` quota exhaustion
    /// (not retryable on the same seat until its window resets). Findings §4(B).
    RateLimited429 { transient: bool },
    /// 401/403, OAuth `invalid_grant`, or Codex `refresh_token_reused` — the
    /// credential is rejected. Not retryable as-is.
    AuthFailure,
    /// 5xx upstream — transient, retryable.
    ServerError5xx,
    /// Request exceeds the model context window (client error) — retrying the
    /// same request cannot help; the seat is healthy.
    ContextWindowExceeded,
    /// Transient OAuth refresh-leg failure (`temporarily_unavailable` /
    /// `server_error` from the token endpoint) — retryable.
    RefreshFailed,
    /// Anything unclassified — fail closed: not retryable, no seat penalty.
    Other,
}

/// Policy-as-data: HTTP status → canonical class, before body refinement.
/// Statuses absent here are decided by [`map_error`]'s range/body logic.
/// Source: findings §5 CLIProxyAPI status→reason ladder.
const STATUS_CLASS_TABLE: &[(u16, ErrorClass)] = &[
    (401, ErrorClass::AuthFailure),
    (403, ErrorClass::AuthFailure),
];

/// Classify an upstream response into a canonical [`ErrorClass`].
///
/// Pure: depends only on the HTTP status and the (already-read) response body.
/// `headers` is part of the port contract and reserved for future `Retry-After`
/// extraction; v1 classification is status+body only.
// ponytail: headers reserved — no error-class signal lives in headers per
// findings (the utilization/representative-claim headers are routing/SC8, not
// error mapping). Kept in the signature so the seam is cutover-stable.
pub fn map_error(status: u16, body: &str, _headers: &[(String, String)]) -> ErrorClass {
    if (200..300).contains(&status) {
        return ErrorClass::Ok;
    }

    let body = body.to_ascii_lowercase();

    // OAuth/credential body signals win over status — a token endpoint returns
    // these under a 400, and a permanently-rejected credential must not be read
    // as a transient server error.
    if body.contains("invalid_grant") || body.contains("refresh_token_reused") {
        return ErrorClass::AuthFailure;
    }
    if body.contains("temporarily_unavailable") || body.contains("server_error") {
        return ErrorClass::RefreshFailed;
    }

    if let Some((_, class)) = STATUS_CLASS_TABLE.iter().find(|(code, _)| *code == status) {
        return *class;
    }

    if status == 429 {
        // Findings §4(B): usage_limit_reached = quota exhausted (non-retryable
        // now); everything else 429 is treated as a transient rate_limit_error.
        let transient = !body.contains("usage_limit_reached");
        return ErrorClass::RateLimited429 { transient };
    }

    if (400..500).contains(&status) && is_context_window_exceeded(&body) {
        return ErrorClass::ContextWindowExceeded;
    }

    if (500..600).contains(&status) {
        return ErrorClass::ServerError5xx;
    }

    ErrorClass::Other
}

/// True when the body indicates the prompt exceeded the model context window.
/// Covers OpenAI (`context_length_exceeded`) and Anthropic
/// (`prompt is too long` / `maximum context length`) phrasings. Body is already
/// lowercased by [`map_error`].
fn is_context_window_exceeded(lower_body: &str) -> bool {
    lower_body.contains("context_length_exceeded")
        || lower_body.contains("context window")
        || lower_body.contains("maximum context length")
        || lower_body.contains("prompt is too long")
}

/// Whether retrying the *same request* can succeed. Quota exhaustion, auth
/// rejection, context-window, and unknown errors are not retryable; transient
/// rate limits, 5xx, and transient refresh failures are.
pub fn is_retryable(class: ErrorClass) -> bool {
    match class {
        ErrorClass::ServerError5xx | ErrorClass::RefreshFailed => true,
        ErrorClass::RateLimited429 { transient } => transient,
        ErrorClass::Ok
        | ErrorClass::AuthFailure
        | ErrorClass::ContextWindowExceeded
        | ErrorClass::Other => false,
    }
}

impl ErrorClass {
    /// Bridge to the existing pool state machine. `None` means "no seat-health
    /// change" — a client error (context-window/unknown) leaves the seat
    /// healthy. `AuthFailure` maps to the transient-cooldown outcome so a single
    /// 401 does not permanently blacklist a seat; the existing
    /// `failure_count`/`BLACKLIST_THRESHOLD` escalation still blacklists a
    /// genuinely dead credential after repeated failures.
    pub fn to_seat_outcome(self) -> Option<SeatOutcome> {
        match self {
            ErrorClass::Ok => Some(SeatOutcome::Ok),
            ErrorClass::RateLimited429 { .. } => Some(SeatOutcome::RateLimited429),
            ErrorClass::ServerError5xx => Some(SeatOutcome::ServerError5xx),
            ErrorClass::AuthFailure | ErrorClass::RefreshFailed => Some(SeatOutcome::RefreshFailed),
            ErrorClass::ContextWindowExceeded | ErrorClass::Other => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NO_HEADERS: &[(String, String)] = &[];

    fn classify(status: u16, body: &str) -> ErrorClass {
        map_error(status, body, NO_HEADERS)
    }

    #[test]
    fn status_and_body_map_to_canonical_classes() {
        // (status, body, expected class, expected retryable)
        let cases: &[(u16, &str, ErrorClass, bool)] = &[
            (200, "", ErrorClass::Ok, false),
            (204, "", ErrorClass::Ok, false),
            // Codex distinction (findings §4B): transient vs quota-exhausted 429.
            (
                429,
                r#"{"error":{"type":"rate_limit_error","message":"slow down"}}"#,
                ErrorClass::RateLimited429 { transient: true },
                true,
            ),
            (
                429,
                r#"{"error":{"type":"usage_limit_reached","resets_in_seconds":3600}}"#,
                ErrorClass::RateLimited429 { transient: false },
                false,
            ),
            (429, "", ErrorClass::RateLimited429 { transient: true }, true),
            (401, "", ErrorClass::AuthFailure, false),
            (403, r#"{"error":"forbidden"}"#, ErrorClass::AuthFailure, false),
            // OAuth refresh: invalid_grant is returned under a 400 but is a
            // permanent credential rejection, not a generic bad request.
            (
                400,
                r#"{"error":"invalid_grant","error_description":"refresh token expired"}"#,
                ErrorClass::AuthFailure,
                false,
            ),
            // Codex §8: refresh_token_reused is non-retryable.
            (
                400,
                r#"{"error":"refresh_token_reused"}"#,
                ErrorClass::AuthFailure,
                false,
            ),
            // OAuth transient token-endpoint failures (RFC 6749 §5.2).
            (
                400,
                r#"{"error":"temporarily_unavailable"}"#,
                ErrorClass::RefreshFailed,
                true,
            ),
            (
                503,
                r#"{"error":"server_error"}"#,
                ErrorClass::RefreshFailed,
                true,
            ),
            (500, "", ErrorClass::ServerError5xx, true),
            (502, "bad gateway", ErrorClass::ServerError5xx, true),
            // Context window (client error) — OpenAI + Anthropic phrasings.
            (
                400,
                r#"{"error":{"code":"context_length_exceeded"}}"#,
                ErrorClass::ContextWindowExceeded,
                false,
            ),
            (
                400,
                r#"{"error":{"type":"invalid_request_error","message":"prompt is too long: 250000 tokens > 200000 maximum"}}"#,
                ErrorClass::ContextWindowExceeded,
                false,
            ),
            // Unclassified.
            (400, r#"{"error":"bad request"}"#, ErrorClass::Other, false),
            (418, "i am a teapot", ErrorClass::Other, false),
        ];

        for (status, body, expected_class, expected_retryable) in cases {
            let class = classify(*status, body);
            assert_eq!(
                class, *expected_class,
                "status={status} body={body:?} classified as {class:?}, expected {expected_class:?}"
            );
            assert_eq!(
                is_retryable(class),
                *expected_retryable,
                "retryability mismatch for status={status} body={body:?} class={class:?}"
            );
        }
    }

    #[test]
    fn quota_exhausted_429_is_not_retryable_but_transient_429_is() {
        // The single most load-bearing distinction (findings §4B): both are 429,
        // only the body separates a dead-until-reset seat from a back-off-and-go.
        assert!(!is_retryable(classify(
            429,
            r#"{"error":{"type":"usage_limit_reached"}}"#
        )));
        assert!(is_retryable(classify(
            429,
            r#"{"error":{"type":"rate_limit_error"}}"#
        )));
    }

    #[test]
    fn seat_outcome_bridge_preserves_existing_machine_semantics() {
        assert_eq!(ErrorClass::Ok.to_seat_outcome(), Some(SeatOutcome::Ok));
        assert_eq!(
            ErrorClass::RateLimited429 { transient: true }.to_seat_outcome(),
            Some(SeatOutcome::RateLimited429)
        );
        assert_eq!(
            ErrorClass::RateLimited429 { transient: false }.to_seat_outcome(),
            Some(SeatOutcome::RateLimited429)
        );
        assert_eq!(
            ErrorClass::ServerError5xx.to_seat_outcome(),
            Some(SeatOutcome::ServerError5xx)
        );
        assert_eq!(
            ErrorClass::AuthFailure.to_seat_outcome(),
            Some(SeatOutcome::RefreshFailed)
        );
        assert_eq!(
            ErrorClass::RefreshFailed.to_seat_outcome(),
            Some(SeatOutcome::RefreshFailed)
        );
        // Client errors leave seat health untouched.
        assert_eq!(ErrorClass::ContextWindowExceeded.to_seat_outcome(), None);
        assert_eq!(ErrorClass::Other.to_seat_outcome(), None);
    }
}
