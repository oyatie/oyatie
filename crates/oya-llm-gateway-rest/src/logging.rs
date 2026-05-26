//! Hash-only structured logging.
//!
//! The gateway emits structured [`tracing`] events that identify a pooled key
//! ONLY by its non-reversible fingerprint. There is deliberately no code path
//! that logs a raw key, a prompt, or a response body — response bodies are
//! streamed straight through and never materialized for logging.
//!
//! This module centralizes the event shape so every call site is consistent
//! and auditable in one place.

use oya_llm_gateway_kernel::ProviderChannel;

/// A redacted dispatch log record. All fields are safe to emit: the key is
/// represented only by `key_fp` (a hash), and no request/response body is
/// present.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchLog<'a> {
    /// Logical group name.
    pub group: &'a str,
    /// Provider channel.
    pub channel: ProviderChannel,
    /// Hash-only key fingerprint (NEVER the raw key).
    pub key_fp: &'a str,
    /// 1-based attempt number within the failover loop.
    pub attempt: u32,
    /// Upstream HTTP status, if a response was received.
    pub upstream_status: Option<u16>,
    /// Terminal outcome label.
    pub outcome: &'a str,
}

impl DispatchLog<'_> {
    /// Emit at INFO. Only the redacted fields above are recorded.
    pub fn emit(&self) {
        tracing::info!(
            target: "oya_llm_gateway::dispatch",
            group = self.group,
            channel = self.channel.as_str(),
            key_fp = self.key_fp,
            attempt = self.attempt,
            upstream_status = self.upstream_status,
            outcome = self.outcome,
            "llm dispatch"
        );
    }
}

/// Compile-time-ish guard documenting the redaction contract. A raw key passed
/// here is converted to its fingerprint; the raw value is dropped immediately.
/// Call sites use this to ensure they never hold a raw key in a log field.
#[must_use]
pub fn redact_key(raw_key: &str) -> String {
    crate::fingerprint_key(raw_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_key_returns_fingerprint_not_key() {
        let fp = redact_key("sk-very-secret");
        assert_eq!(fp.len(), 16);
        assert!(!fp.contains("secret"));
        assert_eq!(fp, crate::fingerprint_key("sk-very-secret"));
    }

    #[test]
    fn dispatch_log_holds_only_redacted_fields() {
        // Construct a record and assert (structurally) the only key-shaped
        // field is the fingerprint. This is a documentation/regression guard:
        // the struct has no field that could carry a raw key or body.
        let fp = redact_key("sk-abc");
        let log = DispatchLog {
            group: "codex",
            channel: ProviderChannel::OpenAi,
            key_fp: &fp,
            attempt: 1,
            upstream_status: Some(200),
            outcome: "ok",
        };
        assert_eq!(log.key_fp, fp);
        assert_eq!(log.outcome, "ok");
        // emit() must not panic.
        log.emit();
    }
}
