//! Agent-dispatch gateway — REST / adapter layer (ADR-0105 Layer 5).
//!
//! Clean-room reverse-proxy that multiplexes an AI-agent fleet over pooled
//! Codex / Claude / OpenAI (and Gemini) API keys. It is an original
//! reimplementation of the *concept* of an LLM key-pool reverse proxy; no
//! third-party source was read or copied.
//!
//! # Layering
//! The pure key-pool state machine lives in [`oya_llm_gateway_kernel`]
//! (Layer 1). This crate is the only I/O-bearing consumer: it owns the axum
//! app, the per-provider channel adapters, the OpenBao-sourced key store, the
//! failover/retry loop, the two constant-time auth realms, the metrics surface
//! (tracing/OTel-aligned, no `prometheus` dep), and hash-only structured
//! logging.
//!
//! # Security invariants (load-bearing)
//! - **No plaintext key from file/env.** Pooled provider keys are read from
//!   OpenBao KV v2 at startup and on periodic refresh. The only secret the
//!   process reads from the environment is the OpenBao token (`BAO_TOKEN`).
//!   See [`keystore`].
//! - **Never log the key, prompt, or response body.** Logs and metrics carry
//!   only a SHA-256-derived [`KeyFingerprint`](oya_llm_gateway_kernel::KeyFingerprint)
//!   hash. Response bodies are streamed straight through and never buffered or
//!   parsed. See [`proxy`] and [`logging`].
//! - **Constant-time auth compares.** Both the admin/control realm and the
//!   ingress proxy-key realm compare presented credentials with
//!   `ring::constant_time`-backed constant-time equality. See [`auth`].
//! - **Encryption-at-rest is delegated to OpenBao.** The gateway holds keys in
//!   memory only (fetched per-refresh from OpenBao); it embeds no KDF/AEAD.
//!   There is no local encrypt-at-rest path. See [`keystore`].

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod auth;
pub mod channel;
pub mod config;
pub mod keystore;
pub mod logging;
pub mod metrics;
pub mod openai;
pub mod proxy;
pub mod state;

pub use auth::{AuthRealm, AuthVerifier};
pub use channel::ChannelAdapter;
pub use config::{GatewayConfig, GroupConfig, RetryPolicyConfig};
pub use keystore::{InMemoryKeyStore, KeyMaterial, KeyStore, KeyStoreError, OpenBaoKeyStore};
pub use metrics::GatewayMetrics;
pub use openai::{
    ChatRequestPeek, OpenAiAppState, OpenAiError, OpenAiErrorBody, Unimplemented, UpstreamBody,
    UpstreamError, UpstreamResponse, UpstreamTransport, build_openai_router,
    extract_retry_after_seconds, retry_after_from_headers,
};
pub use proxy::{ProxyError, ProxyOutcome};
pub use state::{GatewayState, GroupRuntime};

/// Stable hex fingerprint of a raw API key for hash-only logging/metrics.
///
/// SHA-256 (via `ring::digest`) over the UTF-8 key bytes, truncated to the
/// first 16 hex chars (64 bits) — enough to disambiguate keys in a pool without
/// being reversible to the key. This is the ONLY representation of a key that
/// ever leaves this module toward a log line or metric label.
#[must_use]
pub fn fingerprint_key(raw_key: &str) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, raw_key.as_bytes());
    let mut out = String::with_capacity(16);
    for byte in digest.as_ref().iter().take(8) {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_stable_and_hex_16() {
        let a = fingerprint_key("sk-abc123");
        let b = fingerprint_key("sk-abc123");
        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn fingerprint_differs_per_key_and_hides_the_key() {
        let a = fingerprint_key("sk-secret-aaaa");
        let b = fingerprint_key("sk-secret-bbbb");
        assert_ne!(a, b);
        // The fingerprint must not contain the raw key substring.
        assert!(!a.contains("secret"));
    }
}
