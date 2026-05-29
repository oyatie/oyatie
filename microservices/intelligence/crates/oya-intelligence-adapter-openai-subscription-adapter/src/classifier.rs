//! HTTP-status + structured error-type classification for OpenAI API responses.
//! Uses HTTP status codes and the `error.type` JSON field — NOT substring matching.
// data_class: INTERNAL_ONLY throughout this module.

/// Classification of an OpenAI API response for pool circuit-breaker decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseClass {
    /// 200 OK — request succeeded; restore any cooling key.
    Success,
    /// 401 / 403 — key is invalid or lacks permissions; blacklist permanently.
    TerminalKeyInvalid,
    /// 429 with `error.type` = "insufficient_quota" or "quota_exceeded" — quota exhausted;
    /// blacklist permanently (no rotation will help this key).
    TerminalQuotaExhausted,
    /// 429 rate-limit (other subtypes) — transient back-pressure; enter cooldown.
    TransientRateLimit,
    /// 5xx server error — transient; enter cooldown.
    TransientServer,
    /// Other non-200 — treat as transient.
    TransientUnknown,
}

impl ResponseClass {
    /// Returns `true` for classifications that permanently disqualify a key.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::TerminalKeyInvalid | Self::TerminalQuotaExhausted)
    }

    /// Returns `true` for classifications that are transient (trigger cooldown on repeat).
    pub fn is_transient(self) -> bool {
        matches!(
            self,
            Self::TransientRateLimit | Self::TransientServer | Self::TransientUnknown
        )
    }
}

/// Classify an OpenAI API response.
///
/// `http_status`: the HTTP status code of the response.
/// `error_type`: the `error.type` field from the JSON body (if present and parseable).
pub fn classify_response(http_status: u16, error_type: Option<&str>) -> ResponseClass {
    match http_status {
        200..=299 => ResponseClass::Success,
        401 | 403 => ResponseClass::TerminalKeyInvalid,
        429 => match error_type {
            Some("insufficient_quota") | Some("quota_exceeded") => {
                ResponseClass::TerminalQuotaExhausted
            }
            _ => ResponseClass::TransientRateLimit,
        },
        500..=599 => ResponseClass::TransientServer,
        _ => ResponseClass::TransientUnknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_200() {
        assert_eq!(classify_response(200, None), ResponseClass::Success);
    }

    #[test]
    fn success_201() {
        assert_eq!(classify_response(201, None), ResponseClass::Success);
    }

    #[test]
    fn terminal_401() {
        assert_eq!(
            classify_response(401, None),
            ResponseClass::TerminalKeyInvalid
        );
        assert!(classify_response(401, None).is_terminal());
    }

    #[test]
    fn terminal_403() {
        assert_eq!(
            classify_response(403, Some("insufficient_permissions")),
            ResponseClass::TerminalKeyInvalid
        );
    }

    #[test]
    fn terminal_quota_insufficient_quota() {
        assert_eq!(
            classify_response(429, Some("insufficient_quota")),
            ResponseClass::TerminalQuotaExhausted
        );
        assert!(classify_response(429, Some("insufficient_quota")).is_terminal());
    }

    #[test]
    fn terminal_quota_quota_exceeded() {
        assert_eq!(
            classify_response(429, Some("quota_exceeded")),
            ResponseClass::TerminalQuotaExhausted
        );
    }

    #[test]
    fn transient_rate_limit_no_type() {
        assert_eq!(
            classify_response(429, None),
            ResponseClass::TransientRateLimit
        );
        assert!(classify_response(429, None).is_transient());
    }

    #[test]
    fn transient_rate_limit_rate_limit_exceeded() {
        assert_eq!(
            classify_response(429, Some("rate_limit_exceeded")),
            ResponseClass::TransientRateLimit
        );
    }

    #[test]
    fn transient_500() {
        assert_eq!(classify_response(500, None), ResponseClass::TransientServer);
        assert!(classify_response(500, None).is_transient());
    }

    #[test]
    fn transient_503() {
        assert_eq!(classify_response(503, None), ResponseClass::TransientServer);
    }

    #[test]
    fn transient_unknown_400() {
        assert_eq!(
            classify_response(400, None),
            ResponseClass::TransientUnknown
        );
    }

    #[test]
    fn transient_unknown_404() {
        assert_eq!(
            classify_response(404, None),
            ResponseClass::TransientUnknown
        );
    }

    #[test]
    fn is_terminal_false_for_success() {
        assert!(!ResponseClass::Success.is_terminal());
    }

    #[test]
    fn is_transient_false_for_terminal() {
        assert!(!ResponseClass::TerminalKeyInvalid.is_transient());
        assert!(!ResponseClass::TerminalQuotaExhausted.is_transient());
    }
}
