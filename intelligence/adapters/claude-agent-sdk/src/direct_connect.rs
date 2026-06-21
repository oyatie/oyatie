use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Parsed direct-connect endpoint information.
///
/// This mirrors the package-exported TypeScript `parseDirectConnectUrl()` helper:
/// `cc://host/token` becomes an HTTP server URL plus bearer token, while plain
/// host/HTTP(S) inputs become a server URL without a token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectConnectUrl {
    pub server_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

/// Error used by direct-connect helpers.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct DirectConnectError {
    message: String,
    code: Option<String>,
}

impl DirectConnectError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
        }
    }

    pub fn with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: Some(code.into()),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }
}

/// Parse a Claude Code direct-connect URL.
///
/// Source parity: the published TypeScript SDK 0.3.162 exports
/// `parseDirectConnectUrl`; its bundled implementation maps `cc://` to
/// `http://{host}` with the path (minus the leading slash) as `authToken`,
/// rejects `cc+unix://`, and otherwise normalizes host/HTTP(S) inputs to
/// `{protocol}//{host}` without preserving path/query/fragment.
pub fn parse_direct_connect_url(input: &str) -> Result<DirectConnectUrl, DirectConnectError> {
    if let Some(rest) = input.strip_prefix("cc://") {
        let parsed = parse_server_url(&format!("http://{rest}"))?;
        let auth_token = parsed.path.and_then(|path| {
            let token = path.strip_prefix('/').unwrap_or(&path);
            (!token.is_empty()).then(|| token.to_owned())
        });
        return Ok(DirectConnectUrl {
            server_url: parsed.server_url,
            auth_token,
        });
    }

    if input.starts_with("cc+unix://") {
        return Err(DirectConnectError::new(
            "Unix socket connect (cc+unix://) is not supported by the SDK transport",
        ));
    }

    let normalized = if starts_with_http_scheme(input) {
        input.to_owned()
    } else {
        format!("http://{input}")
    };
    let parsed = parse_server_url(&normalized)?;
    Ok(DirectConnectUrl {
        server_url: parsed.server_url,
        auth_token: None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedServerUrl {
    server_url: String,
    path: Option<String>,
}

fn starts_with_http_scheme(input: &str) -> bool {
    input
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
        || input
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
}

fn parse_server_url(input: &str) -> Result<ParsedServerUrl, DirectConnectError> {
    let (scheme, after_scheme) = split_http_scheme(input)?;
    let (authority, path) = split_authority_and_path(after_scheme);
    if authority.is_empty() {
        return Err(invalid_url(input));
    }

    let host = normalize_host(authority, scheme, input)?;
    Ok(ParsedServerUrl {
        server_url: format!("{scheme}//{host}"),
        path,
    })
}

fn split_http_scheme(input: &str) -> Result<(&'static str, &str), DirectConnectError> {
    if input
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
    {
        return Ok(("http:", &input[7..]));
    }
    if input
        .get(..8)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
    {
        return Ok(("https:", &input[8..]));
    }
    Err(invalid_url(input))
}

fn split_authority_and_path(after_scheme: &str) -> (&str, Option<String>) {
    let split_at = after_scheme
        .char_indices()
        .find_map(|(index, character)| matches!(character, '/' | '?' | '#').then_some(index))
        .unwrap_or(after_scheme.len());
    let authority = &after_scheme[..split_at];
    let remainder = &after_scheme[split_at..];
    let path = remainder
        .strip_prefix('/')
        .map(|after_slash| {
            let path_end = after_slash
                .char_indices()
                .find_map(|(index, character)| matches!(character, '?' | '#').then_some(index))
                .unwrap_or(after_slash.len());
            format!("/{}", &after_slash[..path_end])
        })
        .filter(|path| !path.is_empty());
    (authority, path)
}

fn normalize_host(
    authority: &str,
    scheme: &str,
    original_input: &str,
) -> Result<String, DirectConnectError> {
    let host = authority.rsplit('@').next().unwrap_or(authority);
    if host.is_empty() || host.chars().any(char::is_whitespace) {
        return Err(invalid_url(original_input));
    }

    let host = host.to_ascii_lowercase();
    Ok(strip_default_port(&host, scheme).to_owned())
}

fn strip_default_port<'a>(host: &'a str, scheme: &str) -> &'a str {
    match scheme {
        "http:" => strip_port(host, "80").unwrap_or(host),
        "https:" => strip_port(host, "443").unwrap_or(host),
        _ => host,
    }
}

fn strip_port<'a>(host: &'a str, port: &str) -> Option<&'a str> {
    let suffix = format!(":{port}");
    let without_suffix = host.strip_suffix(&suffix)?;
    let colon_count = without_suffix
        .chars()
        .filter(|character| *character == ':')
        .count();
    if colon_count == 0 || without_suffix.ends_with(']') {
        Some(without_suffix)
    } else {
        None
    }
}

fn invalid_url(input: &str) -> DirectConnectError {
    DirectConnectError::new(format!("Invalid direct connect URL: {input}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_matches_current_typescript_shape() {
        assert_eq!(
            parse_direct_connect_url("cc://localhost:4567/token").unwrap(),
            DirectConnectUrl {
                server_url: "http://localhost:4567".into(),
                auth_token: Some("token".into()),
            }
        );
        assert_eq!(
            parse_direct_connect_url("https://Example.com:443/path").unwrap(),
            DirectConnectUrl {
                server_url: "https://example.com".into(),
                auth_token: None,
            }
        );
    }

    #[test]
    fn parser_covers_edge_normalization_and_error_shapes() {
        let error = DirectConnectError::with_code("bad url", "ERR_BAD_URL");
        assert_eq!(error.message(), "bad url");
        assert_eq!(error.code(), Some("ERR_BAD_URL"));

        assert_eq!(
            parse_direct_connect_url("HTTP://User@Example.COM:80/path?ignored#also").unwrap(),
            DirectConnectUrl {
                server_url: "http://example.com".into(),
                auth_token: None,
            }
        );
        assert_eq!(
            parse_direct_connect_url("https://[::1]:443/path").unwrap(),
            DirectConnectUrl {
                server_url: "https://[::1]".into(),
                auth_token: None,
            }
        );
        assert_eq!(
            parse_direct_connect_url("cc://localhost:4567/").unwrap(),
            DirectConnectUrl {
                server_url: "http://localhost:4567".into(),
                auth_token: None,
            }
        );
        assert!(parse_direct_connect_url("http://").is_err());
        assert!(parse_direct_connect_url("http://bad host").is_err());
    }
}

use std::collections::BTreeMap;
#[cfg(feature = "network")]
use std::{path::PathBuf, time::Duration};

use futures::Stream;
use serde_json::Value;

#[cfg(feature = "network")]
use futures::{SinkExt, StreamExt};
#[cfg(feature = "network")]
use tokio::{net::TcpStream, sync::mpsc, task::JoinHandle, time};
#[cfg(feature = "network")]
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message as WebSocketMessage, client::IntoClientRequest},
};

#[cfg(feature = "network")]
use crate::query::{query_stream_with_transport, query_with_transport};
#[cfg(feature = "network")]
use crate::transport::RuntimeTransport;
use crate::{
    error::{ClaudeAgentError, Result},
    messages::UserMessage,
    options::{ClaudeAgentOptions, PermissionMode},
    query::Query,
};

/// Options for the package-exported direct-connect transport.
///
/// Source parity: `@anthropic-ai/claude-agent-sdk@0.3.162` creates a direct
/// session with `POST {serverUrl}/sessions`, optional bearer auth, and optional
/// `cwd`, `session_key`, and `permission_mode`, then connects to the returned
/// `ws_url` over WebSocket.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectConnectTransportOptions {
    pub server_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<PermissionMode>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub delete_session_on_close: bool,
}

impl DirectConnectTransportOptions {
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            ..Self::default()
        }
    }

    pub fn from_url(input: &str) -> Result<Self, DirectConnectError> {
        let parsed = parse_direct_connect_url(input)?;
        Ok(Self {
            server_url: parsed.server_url,
            auth_token: parsed.auth_token,
            ..Self::default()
        })
    }

    pub fn auth_token(mut self, auth_token: impl Into<String>) -> Self {
        self.auth_token = Some(auth_token.into());
        self
    }

    pub fn cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn session_key(mut self, session_key: impl Into<String>) -> Self {
        self.session_key = Some(session_key.into());
        self
    }

    pub fn permission_mode(mut self, permission_mode: PermissionMode) -> Self {
        self.permission_mode = Some(permission_mode);
        self
    }

    pub fn delete_session_on_close(mut self, delete: bool) -> Self {
        self.delete_session_on_close = delete;
        self
    }

    #[cfg(any(feature = "network", test))]
    fn session_create_body(&self) -> Value {
        let mut body = serde_json::Map::new();
        if let Some(cwd) = &self.cwd {
            body.insert("cwd".into(), Value::String(cwd.clone()));
        }
        if let Some(session_key) = &self.session_key {
            body.insert("session_key".into(), Value::String(session_key.clone()));
        }
        if let Some(permission_mode) = self.permission_mode {
            body.insert(
                "permission_mode".into(),
                Value::String(permission_mode.as_cli_value().into()),
            );
        }
        Value::Object(body)
    }
}

/// Session-create response returned by a direct-connect server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DirectConnectSessionResponse {
    pub session_id: String,
    pub ws_url: String,
    pub work_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
}

/// Browser/WebSocket SDK auth credential shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthCredential {
    #[serde(rename = "type")]
    pub credential_type: OAuthCredentialType,
    pub token: String,
}

impl OAuthCredential {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            credential_type: OAuthCredentialType::Oauth,
            token: token.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthCredentialType {
    Oauth,
}

/// Browser/WebSocket SDK auth message shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthMessage {
    #[serde(rename = "type")]
    pub message_type: AuthMessageType,
    pub credential: OAuthCredential,
}

impl AuthMessage {
    pub fn oauth(token: impl Into<String>) -> Self {
        Self {
            message_type: AuthMessageType::Auth,
            credential: OAuthCredential::new(token),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMessageType {
    Auth,
}

/// Options for connecting directly to a WebSocket Agent SDK endpoint.
///
/// This mirrors the package `/browser` export's `WebSocketOptions`, with
/// headers usable by non-browser Rust hosts and optional auth/keep-alive frames.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketOptions {
    pub url: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_message: Option<AuthMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keep_alive_interval_ms: Option<u64>,
}

impl WebSocketOptions {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Self::default()
        }
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn auth_message(mut self, auth_message: AuthMessage) -> Self {
        self.auth_message = Some(auth_message);
        self
    }

    pub fn keep_alive_interval_ms(mut self, interval: u64) -> Self {
        self.keep_alive_interval_ms = Some(interval);
        self
    }
}

#[cfg(feature = "network")]
type AgentWebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Direct-connect WebSocket transport.
///
/// Enable with the `network` crate feature. Without that feature the type still
/// exists, but connection/query constructors return a clear feature-gate error.
#[cfg(feature = "network")]
pub struct DirectConnectTransport {
    inner: JsonLineWebSocketTransport,
    session_id: String,
    work_dir: String,
    server_url: String,
    auth_token: Option<String>,
    delete_session_on_close: bool,
}

#[cfg(not(feature = "network"))]
#[derive(Debug, Clone)]
pub struct DirectConnectTransport;

#[cfg(feature = "network")]
impl DirectConnectTransport {
    pub async fn connect(options: DirectConnectTransportOptions) -> Result<Self> {
        let session = create_direct_connect_session(&options).await?;
        let mut headers = BTreeMap::new();
        if let Some(auth_token) = &options.auth_token {
            headers.insert("authorization".into(), format!("Bearer {auth_token}"));
        }
        let inner = JsonLineWebSocketTransport::connect(
            &session.ws_url,
            &headers,
            None,
            None,
            PathBuf::from(&session.work_dir),
        )
        .await?;
        Ok(Self {
            inner,
            session_id: session.session_id,
            work_dir: session.work_dir,
            server_url: options.server_url,
            auth_token: options.auth_token,
            delete_session_on_close: options.delete_session_on_close,
        })
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn work_dir(&self) -> &str {
        &self.work_dir
    }
}

#[cfg(not(feature = "network"))]
impl DirectConnectTransport {
    pub async fn connect(_options: DirectConnectTransportOptions) -> Result<Self> {
        Err(network_feature_required())
    }
}

#[cfg(feature = "network")]
impl RuntimeTransport for DirectConnectTransport {
    async fn write_json_line(&mut self, value: &Value) -> Result<()> {
        self.inner.write_json_line(value).await
    }

    async fn read_json_line(&mut self) -> Result<Option<Value>> {
        self.inner.read_json_line().await
    }

    async fn end_input(&mut self) -> Result<()> {
        self.inner.end_input().await
    }

    async fn wait(self) -> Result<()> {
        let result = self.inner.wait().await;
        if self.delete_session_on_close {
            let _ = delete_direct_connect_session(
                &self.server_url,
                &self.session_id,
                self.auth_token.as_deref(),
            )
            .await;
        }
        result
    }

    fn projects_dir(&self) -> &PathBuf {
        self.inner.projects_dir()
    }
}

#[cfg(feature = "network")]
pub struct WebSocketTransport {
    inner: JsonLineWebSocketTransport,
}

#[cfg(not(feature = "network"))]
#[derive(Debug, Clone)]
pub struct WebSocketTransport;

#[cfg(feature = "network")]
impl WebSocketTransport {
    pub async fn connect(options: WebSocketOptions) -> Result<Self> {
        let keep_alive = options
            .keep_alive_interval_ms
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_secs(50));
        let inner = JsonLineWebSocketTransport::connect(
            &options.url,
            &options.headers,
            options.auth_message,
            Some(keep_alive),
            PathBuf::from("."),
        )
        .await?;
        Ok(Self { inner })
    }
}

#[cfg(not(feature = "network"))]
impl WebSocketTransport {
    pub async fn connect(_options: WebSocketOptions) -> Result<Self> {
        Err(network_feature_required())
    }
}

#[cfg(feature = "network")]
impl RuntimeTransport for WebSocketTransport {
    async fn write_json_line(&mut self, value: &Value) -> Result<()> {
        self.inner.write_json_line(value).await
    }

    async fn read_json_line(&mut self) -> Result<Option<Value>> {
        self.inner.read_json_line().await
    }

    async fn end_input(&mut self) -> Result<()> {
        self.inner.end_input().await
    }

    async fn wait(self) -> Result<()> {
        self.inner.wait().await
    }

    fn projects_dir(&self) -> &PathBuf {
        self.inner.projects_dir()
    }
}

/// Run an SDK query over a direct-connect server instead of a local CLI process.
#[cfg(feature = "network")]
pub async fn query_direct_connect(
    prompt: impl Into<String>,
    direct_connect: DirectConnectTransportOptions,
    options: ClaudeAgentOptions,
) -> Result<Query> {
    let transport = DirectConnectTransport::connect(direct_connect).await?;
    Ok(query_with_transport(prompt, options, transport))
}

#[cfg(not(feature = "network"))]
pub async fn query_direct_connect(
    _prompt: impl Into<String>,
    _direct_connect: DirectConnectTransportOptions,
    _options: ClaudeAgentOptions,
) -> Result<Query> {
    Err(network_feature_required())
}

/// Run a streaming SDK query over a direct-connect server.
#[cfg(feature = "network")]
pub async fn query_direct_connect_stream<S>(
    prompt: S,
    direct_connect: DirectConnectTransportOptions,
    options: ClaudeAgentOptions,
) -> Result<Query>
where
    S: Stream<Item = UserMessage> + Send + 'static,
{
    let transport = DirectConnectTransport::connect(direct_connect).await?;
    Ok(query_stream_with_transport(prompt, options, transport))
}

#[cfg(not(feature = "network"))]
pub async fn query_direct_connect_stream<S>(
    _prompt: S,
    _direct_connect: DirectConnectTransportOptions,
    _options: ClaudeAgentOptions,
) -> Result<Query>
where
    S: Stream<Item = UserMessage> + Send + 'static,
{
    Err(network_feature_required())
}

/// Run an SDK query over an already-created WebSocket endpoint.
#[cfg(feature = "network")]
pub async fn query_websocket(
    prompt: impl Into<String>,
    websocket: WebSocketOptions,
    options: ClaudeAgentOptions,
) -> Result<Query> {
    let transport = WebSocketTransport::connect(websocket).await?;
    Ok(query_with_transport(prompt, options, transport))
}

#[cfg(not(feature = "network"))]
pub async fn query_websocket(
    _prompt: impl Into<String>,
    _websocket: WebSocketOptions,
    _options: ClaudeAgentOptions,
) -> Result<Query> {
    Err(network_feature_required())
}

/// Run a streaming SDK query over an already-created WebSocket endpoint.
#[cfg(feature = "network")]
pub async fn query_websocket_stream<S>(
    prompt: S,
    websocket: WebSocketOptions,
    options: ClaudeAgentOptions,
) -> Result<Query>
where
    S: Stream<Item = UserMessage> + Send + 'static,
{
    let transport = WebSocketTransport::connect(websocket).await?;
    Ok(query_stream_with_transport(prompt, options, transport))
}

#[cfg(not(feature = "network"))]
pub async fn query_websocket_stream<S>(
    _prompt: S,
    _websocket: WebSocketOptions,
    _options: ClaudeAgentOptions,
) -> Result<Query>
where
    S: Stream<Item = UserMessage> + Send + 'static,
{
    Err(network_feature_required())
}

#[cfg(feature = "network")]
async fn create_direct_connect_session(
    options: &DirectConnectTransportOptions,
) -> Result<DirectConnectSessionResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}/sessions", options.server_url.trim_end_matches('/'));
    let mut request = client
        .post(url)
        .header("content-type", "application/json")
        .json(&options.session_create_body());
    if let Some(auth_token) = &options.auth_token {
        request = request.bearer_auth(auth_token);
    }
    let response = request.send().await.map_err(|error| {
        ClaudeAgentError::Connection(format!(
            "Failed to connect to server at {}: {error}",
            options.server_url
        ))
    })?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(ClaudeAgentError::Connection(format!(
            "Failed to create session: {status}{suffix}",
            suffix = if body.is_empty() {
                String::new()
            } else {
                format!(" — {body}")
            }
        )));
    }
    let value: Value = response
        .json()
        .await
        .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?;
    parse_direct_connect_session_response(value).map_err(ClaudeAgentError::Connection)
}

#[cfg(feature = "network")]
async fn delete_direct_connect_session(
    server_url: &str,
    session_id: &str,
    auth_token: Option<&str>,
) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/sessions/{session_id}", server_url.trim_end_matches('/'));
    let mut request = client.delete(url);
    if let Some(auth_token) = auth_token {
        request = request.bearer_auth(auth_token);
    }
    let _ = request
        .send()
        .await
        .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?;
    Ok(())
}

pub fn parse_direct_connect_session_response(
    value: Value,
) -> std::result::Result<DirectConnectSessionResponse, String> {
    let response: DirectConnectSessionResponse = serde_json::from_value(value)
        .map_err(|error| format!("Invalid session response: {error}"))?;
    if response.session_id.is_empty() || response.ws_url.is_empty() || response.work_dir.is_empty()
    {
        return Err(
            "Invalid session response: session_id, ws_url, and work_dir are required".into(),
        );
    }
    Ok(response)
}

#[cfg(feature = "network")]
struct JsonLineWebSocketTransport {
    outbound_tx: Option<mpsc::Sender<String>>,
    inbound_rx: mpsc::Receiver<Result<Value>>,
    worker: Option<JoinHandle<Result<()>>>,
    projects_dir: PathBuf,
}

#[cfg(feature = "network")]
impl JsonLineWebSocketTransport {
    async fn connect(
        url: &str,
        headers: &BTreeMap<String, String>,
        auth_message: Option<AuthMessage>,
        keep_alive_interval: Option<Duration>,
        projects_dir: PathBuf,
    ) -> Result<Self> {
        let mut request = url
            .into_client_request()
            .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?;
        for (key, value) in headers {
            let header_name = http::HeaderName::from_bytes(key.as_bytes())
                .map_err(|error| ClaudeAgentError::InvalidOption(error.to_string()))?;
            let header_value = http::HeaderValue::from_str(value)
                .map_err(|error| ClaudeAgentError::InvalidOption(error.to_string()))?;
            request.headers_mut().insert(header_name, header_value);
        }
        let (ws, _) = connect_async(request)
            .await
            .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?;
        Ok(Self::from_socket(
            ws,
            auth_message,
            keep_alive_interval,
            projects_dir,
        ))
    }

    fn from_socket(
        ws: AgentWebSocket,
        auth_message: Option<AuthMessage>,
        keep_alive_interval: Option<Duration>,
        projects_dir: PathBuf,
    ) -> Self {
        let (outbound_tx, outbound_rx) = mpsc::channel(100);
        let (inbound_tx, inbound_rx) = mpsc::channel(100);
        let worker = tokio::spawn(websocket_worker(
            ws,
            outbound_rx,
            inbound_tx,
            auth_message,
            keep_alive_interval,
        ));
        Self {
            outbound_tx: Some(outbound_tx),
            inbound_rx,
            worker: Some(worker),
            projects_dir,
        }
    }

    async fn write_json_line(&mut self, value: &Value) -> Result<()> {
        let tx = self
            .outbound_tx
            .as_ref()
            .ok_or_else(|| ClaudeAgentError::Connection("WebSocket input already closed".into()))?;
        let line = format!("{}\n", serde_json::to_string(value)?);
        tx.send(line)
            .await
            .map_err(|_| ClaudeAgentError::Connection("WebSocket writer is closed".into()))
    }

    async fn read_json_line(&mut self) -> Result<Option<Value>> {
        match self.inbound_rx.recv().await {
            Some(Ok(value)) => Ok(Some(value)),
            Some(Err(error)) => Err(error),
            None => Ok(None),
        }
    }

    async fn end_input(&mut self) -> Result<()> {
        // WebSocket transports do not have a stdin EOF equivalent; upstream
        // TypeScript browser/direct transports intentionally implement no-op
        // endInput().
        Ok(())
    }

    async fn wait(mut self) -> Result<()> {
        self.outbound_tx.take();
        if let Some(worker) = self.worker.take() {
            worker
                .await
                .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?
        } else {
            Ok(())
        }
    }

    fn projects_dir(&self) -> &PathBuf {
        &self.projects_dir
    }
}

#[cfg(feature = "network")]
async fn websocket_worker(
    ws: AgentWebSocket,
    mut outbound_rx: mpsc::Receiver<String>,
    inbound_tx: mpsc::Sender<Result<Value>>,
    auth_message: Option<AuthMessage>,
    keep_alive_interval: Option<Duration>,
) -> Result<()> {
    let (mut write, mut read) = ws.split();
    if let Some(auth_message) = auth_message {
        write
            .send(WebSocketMessage::Text(
                format!("{}\n", serde_json::to_string(&auth_message)?).into(),
            ))
            .await
            .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?;
    }
    let mut keep_alive = keep_alive_interval.map(time::interval);
    let mut partial_line = String::new();
    loop {
        tokio::select! {
            outbound = outbound_rx.recv() => {
                match outbound {
                    Some(line) => write
                        .send(WebSocketMessage::Text(line.into()))
                        .await
                        .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?,
                    None => {
                        let _ = write.close().await;
                        return Ok(());
                    }
                }
            }
            _ = tick_if_enabled(&mut keep_alive), if keep_alive.is_some() => {
                write
                    .send(WebSocketMessage::Text("{\"type\":\"keep_alive\"}\n".into()))
                    .await
                    .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?;
            }
            incoming = read.next() => {
                let Some(incoming) = incoming else { return Ok(()); };
                match incoming.map_err(|error| ClaudeAgentError::Connection(error.to_string()))? {
                    WebSocketMessage::Text(text) => {
                        enqueue_json_lines(&mut partial_line, &text, &inbound_tx).await?;
                    }
                    WebSocketMessage::Binary(bytes) => {
                        if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                            enqueue_json_lines(&mut partial_line, &text, &inbound_tx).await?;
                        }
                    }
                    WebSocketMessage::Close(frame) => {
                        if let Some(frame) = frame
                            && frame.code != tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal
                            && frame.code != tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Away
                        {
                            let _ = inbound_tx
                                .send(Err(ClaudeAgentError::Connection(format!(
                                    "WebSocket closed abnormally: {} {}",
                                    u16::from(frame.code),
                                    frame.reason
                                ))))
                                .await;
                        }
                        return Ok(());
                    }
                    WebSocketMessage::Ping(payload) => {
                        write
                            .send(WebSocketMessage::Pong(payload))
                            .await
                            .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?;
                    }
                    WebSocketMessage::Pong(_) | WebSocketMessage::Frame(_) => {}
                }
            }
        }
    }
}

#[cfg(feature = "network")]
async fn tick_if_enabled(keep_alive: &mut Option<time::Interval>) {
    if let Some(interval) = keep_alive.as_mut() {
        interval.tick().await;
    } else {
        std::future::pending::<()>().await;
    }
}

#[cfg(feature = "network")]
async fn enqueue_json_lines(
    partial_line: &mut String,
    chunk: &str,
    inbound_tx: &mpsc::Sender<Result<Value>>,
) -> Result<()> {
    partial_line.push_str(chunk);
    while let Some(index) = partial_line.find('\n') {
        let line = partial_line[..index].trim().to_owned();
        partial_line.replace_range(..=index, "");
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(&line) {
            Ok(value) => {
                if inbound_tx.send(Ok(value)).await.is_err() {
                    return Ok(());
                }
            }
            Err(source) => {
                let error = ClaudeAgentError::JsonDecode { source, line };
                if inbound_tx.send(Err(error)).await.is_err() {
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

#[cfg(not(feature = "network"))]
fn network_feature_required() -> ClaudeAgentError {
    ClaudeAgentError::InvalidOption(
        "network transports require enabling the `network` crate feature".into(),
    )
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod direct_connect_networkless_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_current_direct_connect_session_create_body() {
        let options = DirectConnectTransportOptions::new("http://localhost:4567")
            .cwd("/workspace")
            .session_key("key-1")
            .permission_mode(PermissionMode::Plan)
            .delete_session_on_close(true);
        assert_eq!(options.server_url, "http://localhost:4567");
        assert!(options.delete_session_on_close);
        assert_eq!(
            options.session_create_body(),
            json!({
                "cwd": "/workspace",
                "session_key": "key-1",
                "permission_mode": "plan"
            })
        );

        let from_url = DirectConnectTransportOptions::from_url("cc://localhost:4567/token")
            .unwrap()
            .auth_token("override");
        assert_eq!(from_url.server_url, "http://localhost:4567");
        assert_eq!(from_url.auth_token.as_deref(), Some("override"));
        assert_eq!(from_url.session_create_body(), json!({}));
    }

    #[test]
    fn parses_current_direct_connect_session_response() {
        let response = parse_direct_connect_session_response(json!({
            "session_id": "session-1",
            "ws_url": "ws://127.0.0.1:1234/session-1",
            "work_dir": "/workspace",
            "session_key": "key-1"
        }))
        .unwrap();
        assert_eq!(response.session_id, "session-1");
        assert_eq!(response.ws_url, "ws://127.0.0.1:1234/session-1");
        assert_eq!(response.work_dir, "/workspace");
        assert_eq!(response.session_key.as_deref(), Some("key-1"));

        assert!(parse_direct_connect_session_response(json!({})).is_err());
        assert!(
            parse_direct_connect_session_response(json!({
                "session_id": "",
                "ws_url": "ws://127.0.0.1:1234/session-1",
                "work_dir": "/workspace"
            }))
            .is_err()
        );
    }

    #[test]
    fn serializes_browser_websocket_auth_message_shape() {
        let message = AuthMessage::oauth("token-1");
        assert_eq!(
            serde_json::to_value(message).unwrap(),
            json!({"type": "auth", "credential": {"type": "oauth", "token": "token-1"}})
        );

        let options = WebSocketOptions::new("ws://127.0.0.1:1234/sdk")
            .header("x-sdk-test", "yes")
            .auth_message(AuthMessage::oauth("token-2"))
            .keep_alive_interval_ms(1_000);
        let value = serde_json::to_value(options).unwrap();
        assert_eq!(value["url"], "ws://127.0.0.1:1234/sdk");
        assert_eq!(value["headers"]["x-sdk-test"], "yes");
        assert_eq!(value["keepAliveIntervalMs"], 1_000);
        assert_eq!(value["authMessage"]["credential"]["token"], "token-2");
    }

    #[cfg(feature = "network")]
    #[tokio::test]
    async fn websocket_line_framer_handles_partial_empty_invalid_and_closed_receivers() {
        let (tx, mut rx) = mpsc::channel(8);
        let mut partial_line = String::new();

        enqueue_json_lines(&mut partial_line, "{\"a\":", &tx)
            .await
            .unwrap();
        assert_eq!(partial_line, "{\"a\":");
        assert!(rx.try_recv().is_err());

        enqueue_json_lines(&mut partial_line, "1}\n\nnot-json\n{\"b\":2}\n", &tx)
            .await
            .unwrap();
        assert_eq!(rx.recv().await.unwrap().unwrap(), json!({"a": 1}));
        assert!(rx.recv().await.unwrap().is_err());
        assert_eq!(rx.recv().await.unwrap().unwrap(), json!({"b": 2}));
        assert!(partial_line.is_empty());

        drop(rx);
        enqueue_json_lines(&mut partial_line, "{\"ignored\":true}\n", &tx)
            .await
            .unwrap();
    }
}
