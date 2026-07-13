//! M02-P02-IP-004 — OAuth subscription-token capture kernel (pure value types).
//!
//! Pure-Rust kernel that captures the OAuth subscription-token flow as
//! deterministic value objects (PKCE challenge/verifier, flow kind, capture
//! request/response). I/O (loopback server, browser open, upstream HTTP
//! exchange) lives in a runtime adapter that consumes this kernel. The kernel
//! exposes:
//!
//!   - `SubscriptionOAuthFlow` — PKCE-pinned flow record.
//!   - `capture_subscription_token` — pure function that constructs the flow
//!     prerequisites and the authorization URL.
//!   - `OAuthLoopbackServer` — value object describing the loopback the
//!     runtime adapter will spin up (port + redirect URI + scopes).
//!   - `FlowKind` — enum with `AnthropicSubscriptionOAuth | OpenAiOAuth |
//!     ApiKeyImport` variants. Linus good-taste: API-key import is a
//!     degenerate flow variant; there is one entry point, no branching.
//!
//! Per ADR-0043: tokens are NEVER persisted in raw form — only the resulting
//! `SecretReference` (sref://…) is. The kernel does not handle raw tokens
//! itself; it only provides the PKCE handshake primitives.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use intelligence_account_kernel::{ProviderFamily, SecretReference};
use std::fmt;

/// data_class: INTERNAL_ONLY — PKCE verifier (high-entropy random secret).
/// Held in this type while the flow is in flight; runtime adapter zeroizes
/// on drop via `secrecy` (adapter-layer concern).
#[derive(Clone, Eq, PartialEq)]
pub struct PkceVerifier(String);

impl PkceVerifier {
    /// Construct from a pre-generated verifier string. Verifier must be
    /// 43..=128 chars of `[A-Za-z0-9\-._~]` per RFC 7636.
    pub fn new(raw: String) -> Result<Self, OAuthError> {
        if raw.len() < 43 || raw.len() > 128 {
            return Err(OAuthError::InvalidVerifierLength);
        }
        if !raw.chars().all(is_unreserved) {
            return Err(OAuthError::InvalidVerifierCharset);
        }
        Ok(Self(raw))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PkceVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PkceVerifier([REDACTED])")
    }
}

fn is_unreserved(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~')
}

/// data_class: INTERNAL_ONLY — base64url-encoded SHA-256 of the verifier.
/// Safe to send over the wire.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PkceChallenge(String);

impl PkceChallenge {
    /// Derive challenge from verifier using SHA-256 — pure, allocation-free
    /// implementation suitable for kernel layer. base64url-no-pad encoding.
    pub fn derive_s256(verifier: &PkceVerifier) -> Self {
        let digest = sha256(verifier.as_str().as_bytes());
        let encoded = base64url_no_pad(&digest);
        Self(encoded)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// data_class: INTERNAL_ONLY — flow modality.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlowKind {
    AnthropicSubscriptionOAuth,
    OpenAiOAuth,
    ApiKeyImport,
}

impl FlowKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::AnthropicSubscriptionOAuth => "anthropic_subscription_oauth",
            Self::OpenAiOAuth => "openai_oauth",
            Self::ApiKeyImport => "api_key_import",
        }
    }
}

/// Loopback redirect descriptor. Mirrors ccproxy-api `oauth_claude` default
/// (port 35593, `/callback`). The runtime adapter binds the listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthLoopbackServer {
    // data_class: INTERNAL_ONLY
    pub port: u16, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub callback_path: String, // data_class: INTERNAL_ONLY
}

impl OAuthLoopbackServer {
    pub const DEFAULT_PORT: u16 = 35593;
    pub const DEFAULT_PATH: &'static str = "/callback";

    pub fn default_claude() -> Self {
        Self {
            port: Self::DEFAULT_PORT,
            callback_path: Self::DEFAULT_PATH.to_owned(),
        }
    }

    pub fn redirect_uri(&self) -> String {
        format!("http://localhost:{}{}", self.port, self.callback_path)
    }
}

/// Per-flow PKCE state + endpoint metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionOAuthFlow {
    pub flow_kind: FlowKind,            // data_class: INTERNAL_ONLY
    pub provider: ProviderFamily,       // data_class: INTERNAL_ONLY
    pub authorization_endpoint: String, // data_class: INTERNAL_ONLY
    pub token_endpoint: String,         // data_class: INTERNAL_ONLY
    pub scopes: Vec<String>,            // data_class: INTERNAL_ONLY
    pub challenge: PkceChallenge,       // data_class: INTERNAL_ONLY
    pub loopback: OAuthLoopbackServer,  // data_class: INTERNAL_ONLY
    pub state_nonce: String,            // data_class: INTERNAL_ONLY
}

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionTokenCaptureRequest {
    // data_class: INTERNAL_ONLY
    pub flow_kind: FlowKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub provider: ProviderFamily, // data_class: INTERNAL_ONLY
    pub verifier: PkceVerifier,   // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub state_nonce: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub loopback: OAuthLoopbackServer, // data_class: INTERNAL_ONLY
}

/// data_class: INTERNAL_ONLY — payload returned after a successful capture.
/// Carries only the `SecretReference`; raw token bytes never travel through
/// this struct.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionTokenCaptureResponse {
    // data_class: INTERNAL_ONLY
    pub sref: SecretReference, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub flow_kind: FlowKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub provider: ProviderFamily, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub captured_unix_secs: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OAuthError {
    InvalidVerifierLength,
    InvalidVerifierCharset,
    EmptyAuthorizationEndpoint,
    EmptyTokenEndpoint,
    EmptyScopes,
    EmptyStateNonce,
    ProviderRejected(String),
    LoopbackPortReserved,
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVerifierLength => {
                write!(f, "PKCE verifier must be 43..=128 chars (RFC 7636)")
            }
            Self::InvalidVerifierCharset => {
                write!(
                    f,
                    "PKCE verifier must use unreserved chars [A-Za-z0-9\\-._~]"
                )
            }
            Self::EmptyAuthorizationEndpoint => write!(f, "authorization endpoint is empty"),
            Self::EmptyTokenEndpoint => write!(f, "token endpoint is empty"),
            Self::EmptyScopes => write!(f, "scope list is empty"),
            Self::EmptyStateNonce => write!(f, "state nonce is empty"),
            Self::ProviderRejected(s) => write!(f, "provider rejected token exchange: {s}"),
            Self::LoopbackPortReserved => write!(f, "loopback port is reserved"),
        }
    }
}

/// Canonical scope set for Claude.ai subscription OAuth — matches ccproxy-api
/// `oauth_claude` default (`org:create_api_key`, `user:profile`,
/// `user:inference`).
pub fn anthropic_subscription_scopes() -> Vec<String> {
    vec![
        "org:create_api_key".into(),
        "user:profile".into(),
        "user:inference".into(),
    ]
}

pub const ANTHROPIC_AUTHORIZATION_ENDPOINT: &str = "https://claude.ai/oauth/authorize";
pub const ANTHROPIC_TOKEN_ENDPOINT: &str = "https://console.anthropic.com/v1/oauth/token";

/// Pure entry point. Builds the `SubscriptionOAuthFlow` from the capture
/// request. No I/O — the runtime adapter is responsible for opening the
/// browser, listening on the loopback, exchanging the code, wrapping the
/// resulting token in `secrecy::SecretString` and writing it through
/// `SecretStorePort` to obtain the `SecretReference`.
///
/// Linus good-taste: `ApiKeyImport` is a single variant on the same entry
/// point; the surface is one function, no parallel `import_api_key` helper.
pub fn capture_subscription_token(
    req: &SubscriptionTokenCaptureRequest,
) -> Result<SubscriptionOAuthFlow, OAuthError> {
    if req.state_nonce.is_empty() {
        return Err(OAuthError::EmptyStateNonce);
    }
    let loopback = &req.loopback;
    // Per OAuth spec & ccproxy-api: loopback ports below 1024 are privileged
    // and reserved.
    if loopback.port < 1024 {
        return Err(OAuthError::LoopbackPortReserved);
    }
    let challenge = PkceChallenge::derive_s256(&req.verifier);
    let (auth_ep, token_ep, scopes) = match req.flow_kind {
        FlowKind::AnthropicSubscriptionOAuth => (
            ANTHROPIC_AUTHORIZATION_ENDPOINT.to_owned(),
            ANTHROPIC_TOKEN_ENDPOINT.to_owned(),
            anthropic_subscription_scopes(),
        ),
        FlowKind::OpenAiOAuth => (
            "https://auth.openai.com/oauth/authorize".to_owned(),
            "https://auth.openai.com/oauth/token".to_owned(),
            vec!["openid".into(), "profile".into(), "offline_access".into()],
        ),
        FlowKind::ApiKeyImport => (
            // ApiKeyImport: no upstream endpoint — operator supplies the key
            // directly. Empty endpoints are valid for this variant; the
            // adapter skips the browser handshake.
            String::new(),
            String::new(),
            vec!["import".into()],
        ),
    };
    if req.flow_kind != FlowKind::ApiKeyImport {
        if auth_ep.is_empty() {
            return Err(OAuthError::EmptyAuthorizationEndpoint);
        }
        if token_ep.is_empty() {
            return Err(OAuthError::EmptyTokenEndpoint);
        }
        if scopes.is_empty() {
            return Err(OAuthError::EmptyScopes);
        }
    }
    Ok(SubscriptionOAuthFlow {
        flow_kind: req.flow_kind,
        provider: req.provider,
        authorization_endpoint: auth_ep,
        token_endpoint: token_ep,
        scopes,
        challenge,
        loopback: loopback.clone(),
        state_nonce: req.state_nonce.clone(),
    })
}

/// Build the authorization URL the browser will navigate to. Returns the URL
/// with PKCE challenge, state nonce, scopes, and the loopback redirect URI.
pub fn build_authorization_url(flow: &SubscriptionOAuthFlow) -> Result<String, OAuthError> {
    if flow.flow_kind == FlowKind::ApiKeyImport {
        // ApiKeyImport has no browser step — the URL surface degenerates to
        // the empty string per Linus row.
        return Ok(String::new());
    }
    if flow.authorization_endpoint.is_empty() {
        return Err(OAuthError::EmptyAuthorizationEndpoint);
    }
    let scope_list = flow.scopes.join(" ");
    let url = format!(
        "{}?response_type=code&code_challenge={}&code_challenge_method=S256&state={}&redirect_uri={}&scope={}",
        flow.authorization_endpoint,
        url_encode(flow.challenge.as_str()),
        url_encode(&flow.state_nonce),
        url_encode(&flow.loopback.redirect_uri()),
        url_encode(&scope_list),
    );
    Ok(url)
}

/// Wrap a successful capture result. The raw token never enters this kernel —
/// the caller supplies a pre-issued `SecretReference` that was minted by the
/// secrets store adapter from a `secrecy::SecretString` in transit.
pub fn record_capture(
    sref: SecretReference,
    flow_kind: FlowKind,
    provider: ProviderFamily,
    captured_unix_secs: u64,
) -> SubscriptionTokenCaptureResponse {
    SubscriptionTokenCaptureResponse {
        sref,
        flow_kind,
        provider,
        captured_unix_secs,
    }
}

// --- Minimal SHA-256 (pure-Rust, FIPS-180-4) ---------------------------------
// Used to derive PKCE S256 challenges. ~50 lines; avoids pulling in a crate
// dependency for the kernel layer (workspace directive: "support everything
// ourselves with 0 to minimal dependency").

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256(msg: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    // Pre-processing
    let bit_len = (msg.len() as u64) * 8;
    let mut padded = msg.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, w_word) in w.iter_mut().enumerate().take(16) {
            *w_word = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn base64url_no_pad(bytes: &[u8]) -> String {
    const ALPHA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8) | (bytes[i + 2] as u32);
        out.push(ALPHA[((n >> 18) & 0x3f) as usize] as char);
        out.push(ALPHA[((n >> 12) & 0x3f) as usize] as char);
        out.push(ALPHA[((n >> 6) & 0x3f) as usize] as char);
        out.push(ALPHA[(n & 0x3f) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let n = (bytes[i] as u32) << 16;
        out.push(ALPHA[((n >> 18) & 0x3f) as usize] as char);
        out.push(ALPHA[((n >> 12) & 0x3f) as usize] as char);
    } else if rem == 2 {
        let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8);
        out.push(ALPHA[((n >> 18) & 0x3f) as usize] as char);
        out.push(ALPHA[((n >> 12) & 0x3f) as usize] as char);
        out.push(ALPHA[((n >> 6) & 0x3f) as usize] as char);
    }
    out
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        let c = *b as char;
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~') {
            out.push(c);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good_verifier() -> PkceVerifier {
        // 43-char fixed test verifier (RFC 7636 minimum).
        PkceVerifier::new("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk".to_owned()).unwrap()
    }

    #[test]
    fn pkce_verifier_length_enforced() {
        let too_short = "a".repeat(10);
        assert_eq!(
            PkceVerifier::new(too_short),
            Err(OAuthError::InvalidVerifierLength)
        );
        let too_long = "a".repeat(200);
        assert_eq!(
            PkceVerifier::new(too_long),
            Err(OAuthError::InvalidVerifierLength)
        );
    }

    #[test]
    fn pkce_verifier_charset_enforced() {
        let bad = "!".repeat(50);
        assert_eq!(
            PkceVerifier::new(bad),
            Err(OAuthError::InvalidVerifierCharset)
        );
    }

    #[test]
    fn pkce_verifier_debug_is_redacted() {
        let v = good_verifier();
        let dbg = format!("{v:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("dBjftJeZ"));
    }

    #[test]
    fn pkce_challenge_matches_rfc7636_appendix_b() {
        // RFC 7636 Appendix B test vector:
        //   verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
        //   challenge (S256) = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        let v = good_verifier();
        let c = PkceChallenge::derive_s256(&v);
        assert_eq!(c.as_str(), "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn loopback_default_matches_ccproxy_port() {
        let l = OAuthLoopbackServer::default_claude();
        assert_eq!(l.port, 35593);
        assert_eq!(l.callback_path, "/callback");
        assert_eq!(l.redirect_uri(), "http://localhost:35593/callback");
    }

    #[test]
    fn anthropic_scopes_match_ccproxy() {
        let s = anthropic_subscription_scopes();
        assert!(s.contains(&"org:create_api_key".to_owned()));
        assert!(s.contains(&"user:profile".to_owned()));
        assert!(s.contains(&"user:inference".to_owned()));
    }

    #[test]
    fn capture_request_builds_anthropic_flow() {
        let req = SubscriptionTokenCaptureRequest {
            flow_kind: FlowKind::AnthropicSubscriptionOAuth,
            provider: ProviderFamily::Claude,
            verifier: good_verifier(),
            state_nonce: "abc123".into(),
            loopback: OAuthLoopbackServer::default_claude(),
        };
        let flow = capture_subscription_token(&req).unwrap();
        assert_eq!(
            flow.authorization_endpoint,
            ANTHROPIC_AUTHORIZATION_ENDPOINT
        );
        assert_eq!(flow.token_endpoint, ANTHROPIC_TOKEN_ENDPOINT);
        assert_eq!(flow.scopes, anthropic_subscription_scopes());
    }

    #[test]
    fn capture_request_rejects_empty_state_nonce() {
        let req = SubscriptionTokenCaptureRequest {
            flow_kind: FlowKind::AnthropicSubscriptionOAuth,
            provider: ProviderFamily::Claude,
            verifier: good_verifier(),
            state_nonce: String::new(),
            loopback: OAuthLoopbackServer::default_claude(),
        };
        assert_eq!(
            capture_subscription_token(&req),
            Err(OAuthError::EmptyStateNonce)
        );
    }

    #[test]
    fn capture_request_rejects_privileged_port() {
        let req = SubscriptionTokenCaptureRequest {
            flow_kind: FlowKind::AnthropicSubscriptionOAuth,
            provider: ProviderFamily::Claude,
            verifier: good_verifier(),
            state_nonce: "abc".into(),
            loopback: OAuthLoopbackServer {
                port: 80,
                callback_path: "/callback".into(),
            },
        };
        assert_eq!(
            capture_subscription_token(&req),
            Err(OAuthError::LoopbackPortReserved)
        );
    }

    #[test]
    fn api_key_import_flow_degenerates_to_empty_url() {
        let req = SubscriptionTokenCaptureRequest {
            flow_kind: FlowKind::ApiKeyImport,
            provider: ProviderFamily::Claude,
            verifier: good_verifier(),
            state_nonce: "abc".into(),
            loopback: OAuthLoopbackServer::default_claude(),
        };
        let flow = capture_subscription_token(&req).unwrap();
        assert_eq!(flow.flow_kind, FlowKind::ApiKeyImport);
        assert!(flow.authorization_endpoint.is_empty());
        // URL builder degenerates to empty string for ApiKeyImport.
        assert_eq!(build_authorization_url(&flow).unwrap(), "");
    }

    #[test]
    fn build_url_contains_pkce_challenge_state_redirect_scope() {
        let req = SubscriptionTokenCaptureRequest {
            flow_kind: FlowKind::AnthropicSubscriptionOAuth,
            provider: ProviderFamily::Claude,
            verifier: good_verifier(),
            state_nonce: "nonce42".into(),
            loopback: OAuthLoopbackServer::default_claude(),
        };
        let flow = capture_subscription_token(&req).unwrap();
        let url = build_authorization_url(&flow).unwrap();
        assert!(url.starts_with(ANTHROPIC_AUTHORIZATION_ENDPOINT));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state=nonce42"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("scope="));
    }

    #[test]
    fn record_capture_carries_sref_only() {
        let sref = SecretReference::new("sref://abc-123-token-id".to_owned()).unwrap();
        let resp = record_capture(
            sref.clone(),
            FlowKind::AnthropicSubscriptionOAuth,
            ProviderFamily::Claude,
            1_700_000_000,
        );
        assert_eq!(resp.sref, sref);
        assert_eq!(resp.flow_kind, FlowKind::AnthropicSubscriptionOAuth);
        // Debug of sref is redacted (kernel rule); the response struct itself
        // does not leak the raw token because there is no raw-token field.
        let dbg = format!("{:?}", resp.sref);
        assert!(dbg.contains("[REDACTED]"));
    }

    #[test]
    fn openai_flow_uses_openai_endpoints() {
        let req = SubscriptionTokenCaptureRequest {
            flow_kind: FlowKind::OpenAiOAuth,
            provider: ProviderFamily::OpenAiOrCodex,
            verifier: good_verifier(),
            state_nonce: "abc".into(),
            loopback: OAuthLoopbackServer::default_claude(),
        };
        let flow = capture_subscription_token(&req).unwrap();
        assert!(flow.authorization_endpoint.contains("openai.com"));
        assert!(flow.token_endpoint.contains("openai.com"));
    }

    #[test]
    fn flow_kind_names_distinct() {
        let s: std::collections::HashSet<&str> = [
            FlowKind::AnthropicSubscriptionOAuth,
            FlowKind::OpenAiOAuth,
            FlowKind::ApiKeyImport,
        ]
        .iter()
        .map(|f| f.name())
        .collect();
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn oauth_error_display_distinct() {
        let m: Vec<String> = vec![
            format!("{}", OAuthError::InvalidVerifierLength),
            format!("{}", OAuthError::InvalidVerifierCharset),
            format!("{}", OAuthError::EmptyAuthorizationEndpoint),
            format!("{}", OAuthError::EmptyTokenEndpoint),
            format!("{}", OAuthError::EmptyScopes),
            format!("{}", OAuthError::EmptyStateNonce),
            format!("{}", OAuthError::ProviderRejected("x".into())),
            format!("{}", OAuthError::LoopbackPortReserved),
        ];
        let uniq: std::collections::HashSet<_> = m.iter().collect();
        assert_eq!(uniq.len(), m.len());
    }

    #[test]
    fn sha256_known_vector_abc() {
        // SHA-256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        let h = sha256(b"abc");
        let expected: [u8; 32] = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ];
        assert_eq!(h, expected);
    }

    #[test]
    fn base64url_known_vector() {
        // base64url("foo") = "Zm9v"
        let s = base64url_no_pad(b"foo");
        assert_eq!(s, "Zm9v");
    }

    #[test]
    fn url_encode_escapes_non_unreserved() {
        let s = url_encode("a b/c");
        assert_eq!(s, "a%20b%2Fc");
    }
}
