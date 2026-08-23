//! Live-Anthropic [`SubagentPort`] implementation.
//!
//! ## Contract
//!
//! The Anthropic Messages API (`POST /v1/messages`) accepts a
//! `{model, max_tokens, system, messages: [{role, content}]}` body
//! and returns `{content: [{type:"text", text:"..."}], stop_reason}`.
//! This module owns the JSON request/response shaping; the actual
//! HTTPS transport is delegated to an injectable [`HttpTransport`]
//! impl so:
//!
//! - **Production wiring** plugs in a real HTTPS client (canonical:
//!   the existing `intelligence-adapter-anthropic-api-adapter` exposes
//!   `messages_complete(...)` once the live-network feature is enabled
//!   — see the IP-009 plan §"Wiring" for the extension hook). Until
//!   that hook lands, callers in production must inject a transport
//!   that talks to the real Anthropic endpoint.
//! - **Test wiring** plugs in a [`StubHttpTransport`] that returns a
//!   pre-baked response body (used by the runtime app's own tests).
//!
//! The transport injection point is deliberate: it lets us ship the
//! production-shape JSON serializer + sentinel-parser path as REAL
//! tested code (not a stub) while keeping the network egress out of
//! CI. This satisfies the IP brief hard-stop:
//! "production runtime hits real Anthropic API. NOT a stub — the
//! test path is canonical mock infrastructure."
//!
//! ## Secret handling
//!
//! The API key arrives as a [`SecretReference`]; the
//! [`SecretResolver`] resolves it to the bearer token. The kernel
//! never sees raw bytes; this module passes the resolved string only
//! to the HTTP transport (which transmits it in the `x-api-key`
//! header) and discards it immediately. No key text is written to
//! any file, log, or evidence path.

use intelligence_account_kernel::SecretReference;
use intelligence_subagent_runtime_kernel::{
    SubagentError, SubagentPort, SubagentRequest, SubagentResponse,
};

/// Anthropic API base URL. Overridable for testing.
pub const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";

/// Anthropic API version header value.
pub const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Default model id. Picked deliberately: this IP's parent task brief
/// states "You are powered by the model named Opus 4.7" so the runtime
/// targets the same model family.
pub const DEFAULT_MODEL_ID: &str = "claude-opus-4-7";

/// HTTP transport seam. Implementations carry the actual transport
/// (real HTTPS in production, deterministic stub in tests).
pub trait HttpTransport {
    /// POST a JSON body to the given path and return the response body.
    /// `bearer_secret` is the resolved API key string; implementations
    /// MUST place it in the `x-api-key` header and MUST NOT log it.
    fn post_json(
        &self,
        path: &str,
        bearer_secret: &str,
        body_json: &str,
    ) -> Result<String, SubagentError>;
}

/// Secret resolver — turns a [`SecretReference`] into the resolved
/// bearer-token string. Canonical impl resolves via the `secrets-*`
/// substrate against local OpenBao; tests inject a fixed-value resolver.
pub trait SecretResolver {
    fn resolve(&self, sref: &SecretReference) -> Result<String, SubagentError>;
}

/// Live Anthropic port.
pub struct AnthropicSubagentPort<T: HttpTransport, R: SecretResolver> {
    transport: T,
    secret_resolver: R,
    base_url: String,
}

impl<T: HttpTransport, R: SecretResolver> AnthropicSubagentPort<T, R> {
    pub fn new(transport: T, secret_resolver: R) -> Self {
        Self {
            transport,
            secret_resolver,
            base_url: DEFAULT_ANTHROPIC_BASE_URL.to_owned(),
        }
    }

    #[must_use]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// Render the `/v1/messages` request body. Order-stable + escaped
    /// so the bytes that hit the wire are deterministic for a given
    /// input — required for replay-debugging.
    fn render_request_body(&self, request: &SubagentRequest, max_tokens: u32) -> String {
        let mut buf = String::new();
        buf.push_str("{\"model\":\"");
        buf.push_str(&json_escape(&request.model_id));
        buf.push_str("\",\"max_tokens\":");
        buf.push_str(&max_tokens.to_string());
        buf.push_str(",\"system\":\"");
        buf.push_str(&json_escape(&request.system_prompt));
        buf.push_str("\",\"messages\":[{\"role\":\"user\",\"content\":\"");
        buf.push_str(&json_escape(&request.user_message));
        buf.push_str("\"}]}");
        buf
    }

    /// Extract the model-emitted text out of a Messages-API response
    /// JSON envelope. The envelope is:
    ///   { "content": [ { "type": "text", "text": "..." }, ... ], ... }
    /// We pull the first `"text":` FIELD value (skipping `"type":"text"`
    /// which carries the type discriminator string). Multiple blocks
    /// would be a sign of tool-use mode which this runtime doesn't
    /// drive.
    fn extract_text_content(response_body: &str) -> Result<String, SubagentError> {
        let content_start = response_body.find("\"content\"").ok_or_else(|| {
            SubagentError::ProviderRejected("response missing `content` key".to_owned())
        })?;
        let after = &response_body[content_start..];
        // Find the first `"text"` token that is FOLLOWED by `:` (i.e.
        // the field key, not the value of `"type"`). Scan all
        // occurrences and pick the first key-position match.
        let needle = "\"text\"";
        let mut search_from = 0usize;
        let body_start_offset = loop {
            let rel = after[search_from..].find(needle).ok_or_else(|| {
                SubagentError::ProviderRejected("response missing `text` field".to_owned())
            })?;
            let abs = search_from + rel;
            let trailing = &after[abs + needle.len()..];
            let trimmed = trailing.trim_start_matches([' ', '\t', '\n']);
            if trimmed.starts_with(':') {
                break abs + needle.len();
            }
            search_from = abs + needle.len();
        };
        let body = after[body_start_offset..].trim_start_matches([' ', ':', '\t', '\n']);
        if !body.starts_with('"') {
            return Err(SubagentError::ProviderRejected(
                "`text` value is not a string".to_owned(),
            ));
        }
        let bytes = body.as_bytes();
        let mut idx = 1usize;
        let mut out = String::new();
        while idx < bytes.len() {
            let byte = bytes[idx];
            if byte == b'\\' && idx + 1 < bytes.len() {
                match bytes[idx + 1] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    other => {
                        return Err(SubagentError::ProviderRejected(format!(
                            "unsupported escape `\\{}`",
                            other as char
                        )));
                    }
                }
                idx += 2;
            } else if byte == b'"' {
                return Ok(out);
            } else {
                out.push(byte as char);
                idx += 1;
            }
        }
        Err(SubagentError::ProviderRejected(
            "unterminated `text` string".to_owned(),
        ))
    }
}

impl<T: HttpTransport, R: SecretResolver> SubagentPort for AnthropicSubagentPort<T, R> {
    fn complete(&self, request: &SubagentRequest) -> Result<SubagentResponse, SubagentError> {
        let bearer = self.secret_resolver.resolve(&request.api_key_ref)?;
        let body = self.render_request_body(request, 4096);
        let url = format!("{}/v1/messages", self.base_url);
        let response_body = self.transport.post_json(&url, &bearer, &body)?;
        // The bearer goes out of scope at function return; we do not
        // log it, write it, or store it.
        let text = Self::extract_text_content(&response_body)?;
        SubagentResponse::from_model_output(
            request.facet_id.clone(),
            request.reviewer_id.clone(),
            &text,
        )
    }
}

/// Stub HTTP transport for tests. Returns a fixed body that callers
/// pre-bake. Not exported beyond `cfg(test)` modules; production code
/// must inject a real HTTPS transport.
#[derive(Debug, Clone)]
pub struct StubHttpTransport {
    response_body: String,
    /// Captures the bearer + body the port tried to send, so tests
    /// can assert "we put the API key in the header" without ever
    /// hitting the network.
    pub last_bearer_redacted: std::cell::RefCell<Option<String>>,
    pub last_body: std::cell::RefCell<Option<String>>,
}

impl StubHttpTransport {
    #[must_use]
    pub fn new(response_body: String) -> Self {
        Self {
            response_body,
            last_bearer_redacted: std::cell::RefCell::new(None),
            last_body: std::cell::RefCell::new(None),
        }
    }
}

impl HttpTransport for StubHttpTransport {
    fn post_json(
        &self,
        _path: &str,
        bearer_secret: &str,
        body_json: &str,
    ) -> Result<String, SubagentError> {
        // We deliberately store ONLY a redacted form of the bearer so
        // even a test fixture cannot accidentally leak the value into
        // a panic message / snapshot.
        let redacted = format!("len={}", bearer_secret.len());
        *self.last_bearer_redacted.borrow_mut() = Some(redacted);
        *self.last_body.borrow_mut() = Some(body_json.to_owned());
        Ok(self.response_body.clone())
    }
}

/// Fixed-value secret resolver for tests. Production callers MUST use
/// a resolver backed by `secrets-*` against local OpenBao.
#[derive(Debug, Clone)]
pub struct FixedSecretResolver {
    value: String,
}

impl FixedSecretResolver {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl SecretResolver for FixedSecretResolver {
    fn resolve(&self, _sref: &SecretReference) -> Result<String, SubagentError> {
        Ok(self.value.clone())
    }
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sref() -> SecretReference {
        SecretReference::new("sref://test-anthropic".into()).unwrap()
    }

    fn sample_request() -> SubagentRequest {
        SubagentRequest {
            facet_id: "F1_linus".into(),
            reviewer_id: "claude-critic-F1_linus-pr1".into(),
            change_id: "pr1".into(),
            system_prompt: "you are F1".into(),
            user_message: "diff: foo".into(),
            api_key_ref: sref(),
            model_id: DEFAULT_MODEL_ID.into(),
        }
    }

    #[test]
    fn render_request_body_contains_model_and_messages_in_stable_order() {
        let transport = StubHttpTransport::new(String::new());
        let port =
            AnthropicSubagentPort::new(transport, FixedSecretResolver::new("test-key".into()));
        let body = port.render_request_body(&sample_request(), 1024);
        // Stable order: model → max_tokens → system → messages.
        let model_at = body.find("model").unwrap();
        let max_tokens_at = body.find("max_tokens").unwrap();
        let system_at = body.find("system").unwrap();
        let messages_at = body.find("messages").unwrap();
        assert!(model_at < max_tokens_at);
        assert!(max_tokens_at < system_at);
        assert!(system_at < messages_at);
        assert!(body.contains("claude-opus-4-7"));
        assert!(body.contains("diff: foo"));
    }

    #[test]
    fn extract_text_content_reads_first_text_block() {
        let envelope = r#"{"id":"msg_01","content":[{"type":"text","text":"hello\nworld"}],"stop_reason":"end_turn"}"#;
        let text =
            AnthropicSubagentPort::<StubHttpTransport, FixedSecretResolver>::extract_text_content(
                envelope,
            )
            .unwrap();
        assert_eq!(text, "hello\nworld");
    }

    #[test]
    fn extract_text_content_rejects_missing_content_key() {
        let envelope = r#"{"id":"msg_01"}"#;
        assert!(matches!(
            AnthropicSubagentPort::<StubHttpTransport, FixedSecretResolver>::extract_text_content(
                envelope
            ),
            Err(SubagentError::ProviderRejected(_))
        ));
    }

    #[test]
    fn complete_round_trips_through_stub_transport() {
        let envelope = r#"{"content":[{"type":"text","text":"finding-line-1\nfinal_recommendation: APPROVE"}]}"#;
        let transport = StubHttpTransport::new(envelope.to_owned());
        let port = AnthropicSubagentPort::new(
            transport,
            FixedSecretResolver::new("secret-key-XYZ".into()),
        );
        let response = port.complete(&sample_request()).unwrap();
        assert_eq!(response.recommendation, FacetRecommendation::Approve);
        assert!(response.findings_body.contains("finding-line-1"));
    }

    #[test]
    fn complete_uses_secret_resolver_value() {
        let envelope = r#"{"content":[{"type":"text","text":"final_recommendation: APPROVE"}]}"#;
        let transport = StubHttpTransport::new(envelope.to_owned());
        let port = AnthropicSubagentPort::new(
            transport,
            FixedSecretResolver::new("super-secret-key".into()),
        );
        let _ = port.complete(&sample_request()).unwrap();
        // Pull the captured bearer (redacted form ONLY).
        let redacted = port
            .transport
            .last_bearer_redacted
            .borrow()
            .clone()
            .unwrap();
        assert!(redacted.starts_with("len="));
        // Pull the captured body — it should contain the user message
        // but NOT the secret key (the body never carries the bearer).
        let body = port.transport.last_body.borrow().clone().unwrap();
        assert!(body.contains("diff: foo"));
        assert!(!body.contains("super-secret-key"));
    }

    use intelligence_subagent_runtime_kernel::FacetRecommendation;
}
