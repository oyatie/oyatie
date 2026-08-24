//! Pure OAuth subscription flow construction.

use super::{
    FlowKind, OAuthError, OAuthLoopbackServer, PkceChallenge, ProviderFamily, SecretReference,
    SubscriptionOAuthFlow, SubscriptionTokenCaptureRequest, SubscriptionTokenCaptureResponse,
    url_encode,
};

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
