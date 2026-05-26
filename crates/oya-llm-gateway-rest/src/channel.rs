//! Per-provider channel adapters: auth-header injection + upstream URL shaping.
//!
//! Each [`ProviderChannel`] speaks a slightly different auth dialect. The
//! adapter's whole job is to translate "I picked pooled key K for group G"
//! into the exact request headers + URL the upstream expects:
//!
//! | Channel   | Auth header(s)                                  |
//! |-----------|-------------------------------------------------|
//! | OpenAI    | `Authorization: Bearer <key>`                   |
//! | Anthropic | `x-api-key: <key>` + `anthropic-version: <ver>` |
//! | Gemini    | `X-Goog-Api-Key: <key>`                         |
//!
//! The adapter never logs the key; it only writes it into outbound request
//! headers. Header construction is a pure function ([`ChannelAdapter::auth_headers`])
//! so it is exhaustively unit-tested without any network.

use oya_llm_gateway_kernel::ProviderChannel;

/// A resolved channel adapter for one group: the dialect + the upstream base
/// URL + (for Anthropic) the API version to pin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelAdapter {
    channel: ProviderChannel,
    upstream_base_url: String,
    anthropic_version: String,
}

/// Default `anthropic-version` if a group does not pin one. Anthropic requires
/// this header on every request; this is a stable, widely-supported value.
const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";

/// A single outbound header (name, value). Names are the lowercase wire form.
pub type HeaderPair = (&'static str, String);

impl ChannelAdapter {
    /// Build an adapter for `channel` forwarding to `upstream_base_url`.
    /// `anthropic_version` is only consulted for the Anthropic channel; pass
    /// `None` to use [`DEFAULT_ANTHROPIC_VERSION`].
    #[must_use]
    pub fn new(
        channel: ProviderChannel,
        upstream_base_url: impl Into<String>,
        anthropic_version: Option<String>,
    ) -> Self {
        ChannelAdapter {
            channel,
            upstream_base_url: trim_trailing_slash(upstream_base_url.into()),
            anthropic_version: anthropic_version
                .unwrap_or_else(|| DEFAULT_ANTHROPIC_VERSION.to_string()),
        }
    }

    /// The provider channel served.
    #[must_use]
    pub fn channel(&self) -> ProviderChannel {
        self.channel
    }

    /// The upstream base URL (no trailing slash).
    #[must_use]
    pub fn upstream_base_url(&self) -> &str {
        &self.upstream_base_url
    }

    /// Compose the full upstream URL for a forwarded request whose remaining
    /// path (after the group prefix) is `tail` (which may include a leading
    /// `/` and a `?query`). The base URL never has a trailing slash, so we
    /// join with exactly one `/`.
    #[must_use]
    pub fn upstream_url(&self, tail: &str) -> String {
        let tail = tail.trim_start_matches('/');
        if tail.is_empty() {
            self.upstream_base_url.clone()
        } else {
            format!("{}/{}", self.upstream_base_url, tail)
        }
    }

    /// The auth headers to inject for this channel given the chosen raw key.
    ///
    /// SECURITY: the returned [`String`] values contain the live key and must
    /// only ever be written to the outbound upstream request. They are never
    /// logged. Callers move them straight into the request builder.
    #[must_use]
    pub fn auth_headers(&self, raw_key: &str) -> Vec<HeaderPair> {
        match self.channel {
            ProviderChannel::OpenAi => {
                vec![("authorization", format!("Bearer {raw_key}"))]
            }
            ProviderChannel::Anthropic => vec![
                ("x-api-key", raw_key.to_string()),
                ("anthropic-version", self.anthropic_version.clone()),
            ],
            ProviderChannel::Gemini => {
                vec![("x-goog-api-key", raw_key.to_string())]
            }
        }
    }

    /// Header names this channel will OVERWRITE on the forwarded request.
    /// Used to strip any client-supplied auth before injecting pooled auth, so
    /// a caller can never smuggle their own upstream credentials through.
    #[must_use]
    pub fn managed_header_names(&self) -> &'static [&'static str] {
        match self.channel {
            ProviderChannel::OpenAi => &["authorization"],
            ProviderChannel::Anthropic => &["x-api-key", "anthropic-version", "authorization"],
            ProviderChannel::Gemini => &["x-goog-api-key", "authorization"],
        }
    }
}

fn trim_trailing_slash(mut url: String) -> String {
    while url.ends_with('/') {
        url.pop();
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_injects_bearer_authorization() {
        let a = ChannelAdapter::new(ProviderChannel::OpenAi, "https://api.openai.com/", None);
        let headers = a.auth_headers("sk-xyz");
        assert_eq!(headers, vec![("authorization", "Bearer sk-xyz".to_string())]);
    }

    #[test]
    fn anthropic_injects_xapikey_and_version() {
        let a = ChannelAdapter::new(
            ProviderChannel::Anthropic,
            "https://api.anthropic.com",
            Some("2099-01-01".to_string()),
        );
        let headers = a.auth_headers("ak-123");
        assert_eq!(
            headers,
            vec![
                ("x-api-key", "ak-123".to_string()),
                ("anthropic-version", "2099-01-01".to_string()),
            ]
        );
    }

    #[test]
    fn anthropic_falls_back_to_default_version() {
        let a = ChannelAdapter::new(ProviderChannel::Anthropic, "https://api.anthropic.com", None);
        let headers = a.auth_headers("ak-123");
        assert_eq!(headers[1], ("anthropic-version", DEFAULT_ANTHROPIC_VERSION.to_string()));
    }

    #[test]
    fn gemini_injects_goog_api_key() {
        let a = ChannelAdapter::new(
            ProviderChannel::Gemini,
            "https://generativelanguage.googleapis.com",
            None,
        );
        let headers = a.auth_headers("g-key");
        assert_eq!(headers, vec![("x-goog-api-key", "g-key".to_string())]);
    }

    #[test]
    fn upstream_url_joins_with_single_slash() {
        let a = ChannelAdapter::new(ProviderChannel::OpenAi, "https://api.openai.com/", None);
        assert_eq!(a.upstream_url("/v1/chat/completions"), "https://api.openai.com/v1/chat/completions");
        assert_eq!(a.upstream_url("v1/models"), "https://api.openai.com/v1/models");
        assert_eq!(a.upstream_url(""), "https://api.openai.com");
        assert_eq!(a.upstream_url("/"), "https://api.openai.com");
    }

    #[test]
    fn upstream_url_preserves_query_string() {
        let a = ChannelAdapter::new(
            ProviderChannel::Gemini,
            "https://generativelanguage.googleapis.com",
            None,
        );
        assert_eq!(
            a.upstream_url("/v1beta/models/gemini-pro:generateContent?alt=sse"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?alt=sse"
        );
    }

    #[test]
    fn managed_headers_cover_each_dialect() {
        assert!(ChannelAdapter::new(ProviderChannel::OpenAi, "https://x", None)
            .managed_header_names()
            .contains(&"authorization"));
        let anth = ChannelAdapter::new(ProviderChannel::Anthropic, "https://x", None);
        assert!(anth.managed_header_names().contains(&"x-api-key"));
        assert!(anth.managed_header_names().contains(&"anthropic-version"));
        assert!(ChannelAdapter::new(ProviderChannel::Gemini, "https://x", None)
            .managed_header_names()
            .contains(&"x-goog-api-key"));
    }
}
