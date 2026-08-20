use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "network")]
use futures::StreamExt;

#[cfg(feature = "network")]
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

#[cfg(feature = "network")]
use tokio::{sync::mpsc, task::JoinHandle};

use crate::status::{SDKControlRequest, SDKControlResponse};
use crate::{ClaudeAgentError, Result};

/// Session state reported to the claude.ai code-session bridge worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeSessionState {
    Idle,
    Running,
    RequiresAction,
}

/// Structured failure categories used by bridge/remote-control helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRemoteControlError {
    pub kind: ConnectRemoteControlErrorKind,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectRemoteControlErrorKind {
    Conflict,
    Auth,
    Network,
    Unknown,
}

/// Options for the package-exported `connectRemoteControl` alpha surface.
///
/// This crate exposes the shape and the lower-level code-session HTTP helpers;
/// the long-running SSE bridge session attachment remains an alpha/runtime
/// integration point for hosts that provide their own event transport.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRemoteControlOptions {
    pub dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registration_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_repo_url: Option<String>,
    pub base_url: String,
    #[serde(rename = "orgUUID")]
    pub org_uuid: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perpetual: Option<bool>,
    #[serde(
        default,
        rename = "initialSSESequenceNum",
        skip_serializing_if = "Option::is_none"
    )]
    pub initial_sse_sequence_num: Option<u64>,
}

/// Options for attaching a host to an existing claude.ai bridge session.
///
/// Source parity: the alpha TypeScript SDK exposes `attachBridgeSession(opts)`
/// with the same camelCase wire names. The Rust handle keeps the same durable
/// concepts while returning `Result` from write/report methods instead of using
/// a JavaScript write queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachBridgeSessionOptions {
    pub session_id: String,
    pub ingress_token: String,
    pub api_base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epoch: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_sequence_num: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heartbeat_interval_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub outbound_only: bool,
}

impl AttachBridgeSessionOptions {
    pub fn new(
        session_id: impl Into<String>,
        ingress_token: impl Into<String>,
        api_base_url: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            ingress_token: ingress_token.into(),
            api_base_url: api_base_url.into(),
            epoch: None,
            initial_sequence_num: None,
            heartbeat_interval_ms: None,
            outbound_only: false,
        }
    }

    /// Provide a known worker epoch and skip `/worker/register`.
    pub fn epoch(mut self, epoch: u64) -> Self {
        self.epoch = Some(epoch);
        self
    }

    /// Seed the SSE high-water mark for resume.
    pub fn initial_sequence_num(mut self, sequence_num: u64) -> Self {
        self.initial_sequence_num = Some(sequence_num);
        self
    }

    /// Override the heartbeat interval. The upstream default is 20 seconds.
    pub fn heartbeat_interval_ms(mut self, heartbeat_interval_ms: u64) -> Self {
        self.heartbeat_interval_ms = Some(heartbeat_interval_ms);
        self
    }

    /// Disable the inbound SSE stream while keeping outbound writes enabled.
    pub fn outbound_only(mut self, outbound_only: bool) -> Self {
        self.outbound_only = outbound_only;
        self
    }
}

/// Options for swapping the bridge transport to a fresh worker JWT.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconnectBridgeTransportOptions {
    pub ingress_token: String,
    pub api_base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epoch: Option<u64>,
}

impl ReconnectBridgeTransportOptions {
    pub fn new(ingress_token: impl Into<String>, api_base_url: impl Into<String>) -> Self {
        Self {
            ingress_token: ingress_token.into(),
            api_base_url: api_base_url.into(),
            epoch: None,
        }
    }

    pub fn epoch(mut self, epoch: u64) -> Self {
        self.epoch = Some(epoch);
        self
    }
}

/// Delivery states accepted by the bridge delivery endpoint.
///
/// The public TypeScript handle surfaces `processing` and `processed`; this
/// enum also includes `received` because the upstream transport auto-acks SSE
/// events internally with that status before marking them processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeDeliveryStatus {
    Received,
    Processing,
    Processed,
}

/// One inbound SSE event from the bridge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgeInboundEvent {
    pub event_id: Option<String>,
    pub sequence_num: Option<u64>,
    pub event_type: Option<String>,
    pub payload: Value,
    pub raw: Value,
}

/// Per-session bridge transport handle.
#[cfg(feature = "network")]
pub struct BridgeSessionHandle {
    session_id: String,
    session_url: String,
    ingress_token: String,
    client: reqwest::Client,
    worker_epoch: u64,
    sequence_num: Arc<AtomicU64>,
    connected: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
    outbound_only: bool,
    heartbeat_interval_ms: u64,
    inbound_rx: mpsc::Receiver<Result<BridgeInboundEvent>>,
    inbound_tx: mpsc::Sender<Result<BridgeInboundEvent>>,
    sse_task: Option<JoinHandle<()>>,
    heartbeat_task: Option<JoinHandle<()>>,
}

#[cfg(not(feature = "network"))]
#[derive(Debug, Clone)]
pub struct BridgeSessionHandle {
    session_id: String,
}

/// Worker credentials returned by `POST /v1/code/sessions/{id}/bridge`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RemoteCredentials {
    pub worker_jwt: String,
    pub api_base_url: String,
    pub expires_in: u64,
    pub worker_epoch: u64,
}

/// Terminal authz failure from the bridge credentials endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialsFailure {
    pub terminal: bool,
    pub reason: CredentialsFailureReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialsFailureReason {
    UntrustedDevice,
    SessionStaleRelogin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RemoteCredentialsOutcome {
    Credentials(RemoteCredentials),
    Failure(CredentialsFailure),
}

pub fn is_credentials_failure(outcome: &RemoteCredentialsOutcome) -> bool {
    matches!(outcome, RemoteCredentialsOutcome::Failure(_))
}

/// Git source/outcome context attached to a v2 code session on create.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSessionGitContext {
    pub git_repo_url: String,
    pub branch: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
}

/// Request builder for `POST /v1/code/sessions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCodeSessionRequest {
    pub base_url: String,
    pub access_token: String,
    pub title: String,
    pub timeout_ms: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_context: Option<CodeSessionGitContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl CreateCodeSessionRequest {
    pub fn new(
        base_url: impl Into<String>,
        access_token: impl Into<String>,
        title: impl Into<String>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            access_token: access_token.into(),
            title: title.into(),
            timeout_ms,
            tags: Vec::new(),
            git_context: None,
            cwd: None,
            model: None,
        }
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn git_context(mut self, git_context: CodeSessionGitContext) -> Self {
        self.git_context = Some(git_context);
        self
    }

    pub fn cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn body(&self) -> Value {
        create_code_session_body(
            &self.title,
            &self.tags,
            self.git_context.as_ref(),
            self.cwd.as_deref(),
            self.model.as_deref(),
        )
    }
}

/// Create a fresh claude.ai code session.
///
/// Source parity: `@anthropic-ai/claude-agent-sdk/bridge@0.3.162` posts to
/// `/v1/code/sessions` with `{ title, bridge: {}, config }` and returns a
/// `cse_*` session id on 200/201; malformed or failed HTTP responses map to
/// `Ok(None)` like the TypeScript helper.
#[cfg(feature = "network")]
pub async fn create_code_session(request: CreateCodeSessionRequest) -> Result<Option<String>> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/v1/code/sessions",
            request.base_url.trim_end_matches('/')
        ))
        .headers(anthropic_headers(&request.access_token)?)
        .timeout(std::time::Duration::from_millis(request.timeout_ms))
        .json(&request.body())
        .send()
        .await;
    let Ok(response) = response else {
        return Ok(None);
    };
    if response.status().as_u16() >= 500 || !matches!(response.status().as_u16(), 200 | 201) {
        return Ok(None);
    }
    let value: Value = match response.json().await {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    Ok(parse_code_session_id(&value))
}

#[cfg(not(feature = "network"))]
pub async fn create_code_session(_request: CreateCodeSessionRequest) -> Result<Option<String>> {
    Err(network_feature_required())
}

/// Fetch worker credentials for an existing code session.
///
/// Source parity: `POST /v1/code/sessions/{id}/bridge`; 403 responses with
/// `error.resource` of `untrusted_device` or `session_stale_relogin` are
/// returned as terminal [`CredentialsFailure`] values, while transient failures
/// return `Ok(None)`.
#[cfg(feature = "network")]
pub async fn fetch_remote_credentials(
    session_id: &str,
    base_url: &str,
    access_token: &str,
    timeout_ms: u64,
    trusted_device_token: Option<&str>,
) -> Result<Option<RemoteCredentialsOutcome>> {
    let client = reqwest::Client::new();
    let mut headers = anthropic_headers(access_token)?;
    if let Some(token) = trusted_device_token {
        headers.insert(
            "X-Trusted-Device-Token",
            http::HeaderValue::from_str(token)
                .map_err(|error| ClaudeAgentError::InvalidOption(error.to_string()))?,
        );
    }
    let response = client
        .post(format!(
            "{}/v1/code/sessions/{session_id}/bridge",
            base_url.trim_end_matches('/')
        ))
        .headers(headers)
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .json(&serde_json::json!({}))
        .send()
        .await;
    let Ok(response) = response else {
        return Ok(None);
    };
    if response.status() != http::StatusCode::OK {
        let status = response.status();
        let value = response.json::<Value>().await.unwrap_or(Value::Null);
        if status == http::StatusCode::FORBIDDEN
            && let Some(reason) = credentials_failure_reason(&value, None)
        {
            return Ok(Some(RemoteCredentialsOutcome::Failure(
                CredentialsFailure {
                    terminal: true,
                    reason,
                },
            )));
        }
        return Ok(None);
    }
    let value = match response.json::<Value>().await {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    Ok(parse_remote_credentials(value).map(RemoteCredentialsOutcome::Credentials))
}

#[cfg(not(feature = "network"))]
pub async fn fetch_remote_credentials(
    _session_id: &str,
    _base_url: &str,
    _access_token: &str,
    _timeout_ms: u64,
    _trusted_device_token: Option<&str>,
) -> Result<Option<RemoteCredentialsOutcome>> {
    Err(network_feature_required())
}

/// Attach to an existing bridge session.
///
/// With the `network` feature enabled, this registers the worker when needed,
/// initializes `/worker`, opens the inbound SSE stream unless `outbound_only`
/// is set, and returns a per-session handle for forwarding SDK messages and
/// control frames. Without `network`, this returns an error so the default
/// crate build stays subprocess-only.
#[cfg(feature = "network")]
pub async fn attach_bridge_session(
    options: AttachBridgeSessionOptions,
) -> Result<BridgeSessionHandle> {
    let client = reqwest::Client::new();
    let session_url = bridge_session_url(&options.api_base_url, &options.session_id);
    let worker_epoch = match options.epoch {
        Some(epoch) => epoch,
        None => register_bridge_worker(&client, &session_url, &options.ingress_token).await?,
    };

    initialize_bridge_worker(&client, &session_url, &options.ingress_token, worker_epoch).await?;

    let (inbound_tx, inbound_rx) = mpsc::channel(64);
    let sequence_num = Arc::new(AtomicU64::new(options.initial_sequence_num.unwrap_or(0)));
    let connected = Arc::new(AtomicBool::new(true));
    let closed = Arc::new(AtomicBool::new(false));
    let mut handle = BridgeSessionHandle {
        session_id: options.session_id,
        session_url,
        ingress_token: options.ingress_token,
        client,
        worker_epoch,
        sequence_num,
        connected,
        closed,
        outbound_only: options.outbound_only,
        heartbeat_interval_ms: options.heartbeat_interval_ms.unwrap_or(20_000),
        inbound_rx,
        inbound_tx,
        sse_task: None,
        heartbeat_task: None,
    };
    if !handle.outbound_only {
        handle.spawn_sse_task();
    }
    handle.spawn_heartbeat_task();
    Ok(handle)
}

#[cfg(not(feature = "network"))]
pub async fn attach_bridge_session(
    options: AttachBridgeSessionOptions,
) -> Result<BridgeSessionHandle> {
    let _ = options;
    Err(network_feature_required())
}

#[cfg(feature = "network")]
impl BridgeSessionHandle {
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn api_session_url(&self) -> &str {
        &self.session_url
    }

    pub fn get_sequence_num(&self) -> u64 {
        self.sequence_num.load(Ordering::SeqCst)
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst) && !self.closed.load(Ordering::SeqCst)
    }

    /// Receive the next inbound bridge event.
    ///
    /// Returns `Ok(None)` when the inbound stream has ended and all cloned
    /// senders have been dropped.
    pub async fn next_inbound(&mut self) -> Result<Option<BridgeInboundEvent>> {
        match self.inbound_rx.recv().await {
            Some(Ok(event)) => Ok(Some(event)),
            Some(Err(error)) => Err(error),
            None => Ok(None),
        }
    }

    /// Write a single SDK message to `/worker/events`.
    pub async fn write(&self, msg: Value) -> Result<()> {
        self.ensure_open()?;
        post_bridge_worker_events(
            &self.client,
            &self.session_url,
            &self.ingress_token,
            self.worker_epoch,
            vec![bridge_client_event(&self.session_id, msg)?],
        )
        .await
    }

    /// Send the zero-cost success result sentinel that stops the remote
    /// "working" spinner at a turn boundary.
    pub async fn send_result(&self) -> Result<()> {
        self.write(bridge_success_result_message(&self.session_id))
            .await
    }

    pub async fn send_control_request(&self, request: SDKControlRequest) -> Result<()> {
        self.write(serde_json::to_value(request)?).await
    }

    pub async fn send_control_response(&self, response: SDKControlResponse) -> Result<()> {
        self.write(serde_json::to_value(response)?).await
    }

    pub async fn send_control_cancel_request(&self, request_id: &str) -> Result<()> {
        self.write(serde_json::json!({
            "type": "control_cancel_request",
            "request_id": request_id,
        }))
        .await
    }

    /// Swap the transport to a fresh ingress token/API base URL, preserving the
    /// current SSE sequence high-water mark.
    pub async fn reconnect_transport(
        &mut self,
        options: ReconnectBridgeTransportOptions,
    ) -> Result<()> {
        self.ensure_open()?;
        self.abort_transport_tasks();
        self.ingress_token = options.ingress_token;
        self.session_url = bridge_session_url(&options.api_base_url, &self.session_id);
        self.worker_epoch = match options.epoch {
            Some(epoch) => epoch,
            None => {
                register_bridge_worker(&self.client, &self.session_url, &self.ingress_token).await?
            }
        };
        initialize_bridge_worker(
            &self.client,
            &self.session_url,
            &self.ingress_token,
            self.worker_epoch,
        )
        .await?;
        self.connected.store(true, Ordering::SeqCst);
        if !self.outbound_only {
            self.spawn_sse_task();
        }
        self.spawn_heartbeat_task();
        Ok(())
    }

    pub async fn report_state(&self, state: BridgeSessionState) -> Result<()> {
        self.ensure_open()?;
        put_bridge_worker(
            &self.client,
            &self.session_url,
            &self.ingress_token,
            serde_json::json!({
                "worker_epoch": self.worker_epoch,
                "worker_status": state,
            }),
        )
        .await
    }

    pub async fn report_metadata(&self, metadata: Value) -> Result<()> {
        self.ensure_open()?;
        if !metadata.is_object() {
            return Err(ClaudeAgentError::InvalidOption(
                "bridge metadata must be a JSON object".into(),
            ));
        }
        put_bridge_worker(
            &self.client,
            &self.session_url,
            &self.ingress_token,
            serde_json::json!({
                "worker_epoch": self.worker_epoch,
                "external_metadata": metadata,
            }),
        )
        .await
    }

    pub async fn report_delivery(
        &self,
        event_id: &str,
        status: BridgeDeliveryStatus,
    ) -> Result<()> {
        self.ensure_open()?;
        post_bridge_delivery(
            &self.client,
            &self.session_url,
            &self.ingress_token,
            self.worker_epoch,
            event_id,
            status,
        )
        .await
    }

    /// Writes are immediate in this Rust implementation, so flush is a
    /// compatibility no-op that preserves the TypeScript handle shape.
    pub async fn flush(&self) -> Result<()> {
        self.ensure_open()
    }

    pub fn close(&mut self) {
        if self.closed.swap(true, Ordering::SeqCst) {
            return;
        }
        self.connected.store(false, Ordering::SeqCst);
        self.abort_transport_tasks();
    }

    fn ensure_open(&self) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(ClaudeAgentError::Connection(
                "bridge session is closed".into(),
            ));
        }
        Ok(())
    }

    fn abort_transport_tasks(&mut self) {
        if let Some(task) = self.sse_task.take() {
            task.abort();
        }
        if let Some(task) = self.heartbeat_task.take() {
            task.abort();
        }
    }

    fn spawn_sse_task(&mut self) {
        let client = self.client.clone();
        let session_url = self.session_url.clone();
        let ingress_token = self.ingress_token.clone();
        let worker_epoch = self.worker_epoch;
        let start_sequence_num = self.sequence_num.load(Ordering::SeqCst);
        let sequence_num = Arc::clone(&self.sequence_num);
        let connected = Arc::clone(&self.connected);
        let closed = Arc::clone(&self.closed);
        let inbound_tx = self.inbound_tx.clone();
        self.sse_task = Some(tokio::spawn(async move {
            let result = run_bridge_sse_stream(BridgeSseStream {
                client,
                session_url,
                ingress_token,
                worker_epoch,
                start_sequence_num,
                sequence_num,
                connected: Arc::clone(&connected),
                closed: Arc::clone(&closed),
                inbound_tx: inbound_tx.clone(),
            })
            .await;
            connected.store(false, Ordering::SeqCst);
            if let Err(error) = result
                && !closed.load(Ordering::SeqCst)
            {
                let _ = inbound_tx.send(Err(error)).await;
            }
        }));
    }

    fn spawn_heartbeat_task(&mut self) {
        let client = self.client.clone();
        let session_url = self.session_url.clone();
        let ingress_token = self.ingress_token.clone();
        let session_id = self.session_id.clone();
        let worker_epoch = self.worker_epoch;
        let interval = Duration::from_millis(self.heartbeat_interval_ms.max(1));
        let closed = Arc::clone(&self.closed);
        self.heartbeat_task = Some(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                if closed.load(Ordering::SeqCst) {
                    break;
                }
                let _ = post_bridge_heartbeat(
                    &client,
                    &session_url,
                    &ingress_token,
                    &session_id,
                    worker_epoch,
                )
                .await;
            }
        }));
    }
}

#[cfg(not(feature = "network"))]
impl BridgeSessionHandle {
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn api_session_url(&self) -> &str {
        ""
    }

    pub fn get_sequence_num(&self) -> u64 {
        0
    }

    pub fn is_connected(&self) -> bool {
        false
    }

    pub async fn next_inbound(&mut self) -> Result<Option<BridgeInboundEvent>> {
        Err(network_feature_required())
    }

    pub async fn write(&self, _msg: Value) -> Result<()> {
        Err(network_feature_required())
    }

    pub async fn send_result(&self) -> Result<()> {
        Err(network_feature_required())
    }

    pub async fn send_control_request(&self, _request: SDKControlRequest) -> Result<()> {
        Err(network_feature_required())
    }

    pub async fn send_control_response(&self, _response: SDKControlResponse) -> Result<()> {
        Err(network_feature_required())
    }

    pub async fn send_control_cancel_request(&self, _request_id: &str) -> Result<()> {
        Err(network_feature_required())
    }

    pub async fn reconnect_transport(
        &mut self,
        _options: ReconnectBridgeTransportOptions,
    ) -> Result<()> {
        Err(network_feature_required())
    }

    pub async fn report_state(&self, _state: BridgeSessionState) -> Result<()> {
        Err(network_feature_required())
    }

    pub async fn report_metadata(&self, _metadata: Value) -> Result<()> {
        Err(network_feature_required())
    }

    pub async fn report_delivery(
        &self,
        _event_id: &str,
        _status: BridgeDeliveryStatus,
    ) -> Result<()> {
        Err(network_feature_required())
    }

    pub async fn flush(&self) -> Result<()> {
        Err(network_feature_required())
    }

    pub fn close(&mut self) {}
}

pub fn create_code_session_body(
    title: &str,
    tags: &[String],
    git_context: Option<&CodeSessionGitContext>,
    cwd: Option<&str>,
    model: Option<&str>,
) -> Value {
    let mut config = serde_json::Map::new();
    config.insert(
        "cwd".into(),
        Value::String(cwd.map(str::to_owned).unwrap_or_else(default_cwd)),
    );
    if let Some(model) = model {
        config.insert("model".into(), Value::String(model.to_owned()));
    }
    if let Some(git_context) = git_context {
        config.insert(
            "git_context".into(),
            serde_json::to_value(git_context).unwrap_or(Value::Null),
        );
    }

    let mut body = serde_json::Map::new();
    body.insert("title".into(), Value::String(title.to_owned()));
    body.insert("bridge".into(), Value::Object(serde_json::Map::new()));
    body.insert("config".into(), Value::Object(config));
    if !tags.is_empty() {
        body.insert(
            "tags".into(),
            Value::Array(tags.iter().cloned().map(Value::String).collect()),
        );
    }
    Value::Object(body)
}

pub fn parse_code_session_id(value: &Value) -> Option<String> {
    let id = value
        .get("session")
        .and_then(|session| session.get("id"))
        .and_then(Value::as_str)?;
    id.starts_with("cse_").then(|| id.to_owned())
}

pub fn parse_remote_credentials(value: Value) -> Option<RemoteCredentials> {
    let worker_epoch = match value.get("worker_epoch")? {
        Value::Number(number) => number.as_u64()?,
        Value::String(text) => text.parse().ok()?,
        _ => return None,
    };
    Some(RemoteCredentials {
        worker_jwt: value.get("worker_jwt")?.as_str()?.to_owned(),
        api_base_url: value.get("api_base_url")?.as_str()?.to_owned(),
        expires_in: value.get("expires_in")?.as_u64()?,
        worker_epoch,
    })
}

pub fn credentials_failure_reason(
    value: &Value,
    message: Option<&str>,
) -> Option<CredentialsFailureReason> {
    let resource = value
        .get("error")
        .and_then(|error| error.get("resource"))
        .and_then(Value::as_str);
    match resource {
        Some("untrusted_device") => Some(CredentialsFailureReason::UntrustedDevice),
        Some("session_stale_relogin") => Some(CredentialsFailureReason::SessionStaleRelogin),
        _ if message.is_some_and(|message| message.contains("trusted device")) => {
            Some(CredentialsFailureReason::UntrustedDevice)
        }
        _ => None,
    }
}

#[cfg(feature = "network")]
fn bridge_session_url(api_base_url: &str, session_id: &str) -> String {
    format!(
        "{}/v1/code/sessions/{session_id}",
        api_base_url.trim_end_matches('/')
    )
}

#[cfg(feature = "network")]
async fn register_bridge_worker(
    client: &reqwest::Client,
    session_url: &str,
    ingress_token: &str,
) -> Result<u64> {
    let response = client
        .post(format!("{session_url}/worker/register"))
        .headers(bridge_json_headers(ingress_token)?)
        .timeout(Duration::from_secs(10))
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|error| {
            ClaudeAgentError::Connection(format!("register bridge worker failed: {error}"))
        })?;
    ensure_success(response.status(), "register bridge worker")?;
    let value = response.json::<Value>().await.map_err(|error| {
        ClaudeAgentError::Connection(format!(
            "register bridge worker response was invalid: {error}"
        ))
    })?;
    parse_worker_epoch(&value).ok_or_else(|| {
        ClaudeAgentError::Connection(format!(
            "register bridge worker response missing valid worker_epoch: {value}"
        ))
    })
}

#[cfg(feature = "network")]
async fn initialize_bridge_worker(
    client: &reqwest::Client,
    session_url: &str,
    ingress_token: &str,
    worker_epoch: u64,
) -> Result<()> {
    put_bridge_worker(
        client,
        session_url,
        ingress_token,
        serde_json::json!({
            "worker_status": "idle",
            "worker_epoch": worker_epoch,
            "external_metadata": {
                "pending_action": null,
                "task_summary": null,
            },
        }),
    )
    .await
}

#[cfg(feature = "network")]
async fn put_bridge_worker(
    client: &reqwest::Client,
    session_url: &str,
    ingress_token: &str,
    body: Value,
) -> Result<()> {
    let response = client
        .put(format!("{session_url}/worker"))
        .headers(bridge_json_headers(ingress_token)?)
        .timeout(Duration::from_secs(10))
        .json(&body)
        .send()
        .await
        .map_err(|error| {
            ClaudeAgentError::Connection(format!("PUT bridge worker failed: {error}"))
        })?;
    ensure_success(response.status(), "PUT bridge worker")
}

#[cfg(feature = "network")]
async fn post_bridge_worker_events(
    client: &reqwest::Client,
    session_url: &str,
    ingress_token: &str,
    worker_epoch: u64,
    events: Vec<Value>,
) -> Result<()> {
    let response = client
        .post(format!("{session_url}/worker/events"))
        .headers(bridge_json_headers(ingress_token)?)
        .timeout(Duration::from_secs(10))
        .json(&serde_json::json!({
            "worker_epoch": worker_epoch,
            "events": events,
        }))
        .send()
        .await
        .map_err(|error| {
            ClaudeAgentError::Connection(format!("POST bridge worker events failed: {error}"))
        })?;
    ensure_success(response.status(), "POST bridge worker events")
}

#[cfg(feature = "network")]
async fn post_bridge_delivery(
    client: &reqwest::Client,
    session_url: &str,
    ingress_token: &str,
    worker_epoch: u64,
    event_id: &str,
    status: BridgeDeliveryStatus,
) -> Result<()> {
    let response = client
        .post(format!("{session_url}/worker/events/delivery"))
        .headers(bridge_json_headers(ingress_token)?)
        .timeout(Duration::from_secs(10))
        .json(&serde_json::json!({
            "worker_epoch": worker_epoch,
            "updates": [{
                "event_id": event_id,
                "status": status,
            }],
        }))
        .send()
        .await
        .map_err(|error| {
            ClaudeAgentError::Connection(format!("POST bridge delivery failed: {error}"))
        })?;
    ensure_success(response.status(), "POST bridge delivery")
}

#[cfg(feature = "network")]
async fn post_bridge_heartbeat(
    client: &reqwest::Client,
    session_url: &str,
    ingress_token: &str,
    session_id: &str,
    worker_epoch: u64,
) -> Result<()> {
    let response = client
        .post(format!("{session_url}/worker/heartbeat"))
        .headers(bridge_json_headers(ingress_token)?)
        .timeout(Duration::from_secs(5))
        .json(&serde_json::json!({
            "session_id": session_id,
            "worker_epoch": worker_epoch,
        }))
        .send()
        .await
        .map_err(|error| {
            ClaudeAgentError::Connection(format!("POST bridge heartbeat failed: {error}"))
        })?;
    ensure_success(response.status(), "POST bridge heartbeat")
}

#[cfg(feature = "network")]
struct BridgeSseStream {
    client: reqwest::Client,
    session_url: String,
    ingress_token: String,
    worker_epoch: u64,
    start_sequence_num: u64,
    sequence_num: Arc<AtomicU64>,
    connected: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
    inbound_tx: mpsc::Sender<Result<BridgeInboundEvent>>,
}

#[cfg(feature = "network")]
async fn run_bridge_sse_stream(config: BridgeSseStream) -> Result<()> {
    let BridgeSseStream {
        client,
        session_url,
        ingress_token,
        worker_epoch,
        start_sequence_num,
        sequence_num,
        connected,
        closed,
        inbound_tx,
    } = config;
    let mut url = format!("{session_url}/worker/events/stream");
    if start_sequence_num > 0 {
        url.push_str(&format!("?from_sequence_num={start_sequence_num}"));
    }
    let response = client
        .get(url)
        .headers(bridge_sse_headers(&ingress_token, start_sequence_num)?)
        .send()
        .await
        .map_err(|error| {
            ClaudeAgentError::Connection(format!("bridge SSE stream failed to connect: {error}"))
        })?;
    ensure_success(response.status(), "bridge SSE stream")?;
    connected.store(true, Ordering::SeqCst);

    let mut buffer = String::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if closed.load(Ordering::SeqCst) {
            break;
        }
        let chunk = chunk.map_err(|error| {
            ClaudeAgentError::Connection(format!("bridge SSE stream read failed: {error}"))
        })?;
        let chunk = String::from_utf8_lossy(&chunk);
        for frame in parse_sse_frames(&mut buffer, &chunk) {
            if frame.event.as_deref() != Some("client_event") {
                continue;
            }
            let value = serde_json::from_str::<Value>(&frame.data)?;
            if let Some(mut event) = parse_bridge_inbound_event(value) {
                if event.event_id.is_none() {
                    event.event_id.clone_from(&frame.id);
                }
                if let Some(sequence) = event.sequence_num {
                    update_sequence_num(&sequence_num, sequence);
                }
                if let Some(event_id) = event.event_id.clone() {
                    let ack_client = client.clone();
                    let ack_session_url = session_url.clone();
                    let ack_ingress_token = ingress_token.clone();
                    tokio::spawn(async move {
                        let _ = post_bridge_delivery(
                            &ack_client,
                            &ack_session_url,
                            &ack_ingress_token,
                            worker_epoch,
                            &event_id,
                            BridgeDeliveryStatus::Received,
                        )
                        .await;
                        let _ = post_bridge_delivery(
                            &ack_client,
                            &ack_session_url,
                            &ack_ingress_token,
                            worker_epoch,
                            &event_id,
                            BridgeDeliveryStatus::Processed,
                        )
                        .await;
                    });
                }
                if inbound_tx.send(Ok(event)).await.is_err() {
                    break;
                }
            }
        }
    }
    Ok(())
}

#[cfg(feature = "network")]
fn bridge_client_event(session_id: &str, msg: Value) -> Result<Value> {
    Ok(serde_json::json!({
        "payload": bridge_message_with_session_id(session_id, msg)?,
    }))
}

#[cfg(feature = "network")]
fn bridge_message_with_session_id(session_id: &str, mut msg: Value) -> Result<Value> {
    let Some(object) = msg.as_object_mut() else {
        return Err(ClaudeAgentError::InvalidOption(
            "bridge messages must be JSON objects".into(),
        ));
    };
    object
        .entry("session_id")
        .or_insert_with(|| Value::String(session_id.to_owned()));
    object
        .entry("uuid")
        .or_insert_with(|| Value::String(uuid::Uuid::new_v4().to_string()));
    Ok(msg)
}

#[cfg(feature = "network")]
fn bridge_success_result_message(session_id: &str) -> Value {
    serde_json::json!({
        "type": "result",
        "subtype": "success",
        "duration_ms": 0,
        "duration_api_ms": 0,
        "is_error": false,
        "num_turns": 0,
        "result": "",
        "stop_reason": null,
        "total_cost_usd": 0.0,
        "usage": {
            "input_tokens": 0,
            "cache_creation_input_tokens": 0,
            "cache_read_input_tokens": 0,
            "output_tokens": 0,
            "server_tool_use": {
                "web_search_requests": 0,
                "web_fetch_requests": 0,
            },
            "service_tier": "standard",
            "cache_creation": {
                "ephemeral_1h_input_tokens": 0,
                "ephemeral_5m_input_tokens": 0,
            },
            "inference_geo": "",
            "iterations": [],
            "speed": "standard",
        },
        "modelUsage": {},
        "permission_denials": [],
        "session_id": session_id,
        "uuid": uuid::Uuid::new_v4().to_string(),
    })
}

#[cfg(feature = "network")]
fn parse_worker_epoch(value: &Value) -> Option<u64> {
    match value.get("worker_epoch")? {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

#[cfg(feature = "network")]
fn parse_bridge_inbound_event(value: Value) -> Option<BridgeInboundEvent> {
    let payload = value.get("payload")?.clone();
    let event_id = value
        .get("event_id")
        .or_else(|| value.get("eventId"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let sequence_num = value
        .get("sequence_num")
        .or_else(|| value.get("sequenceNum"))
        .and_then(json_u64);
    let event_type = value
        .get("event_type")
        .or_else(|| value.get("eventType"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    Some(BridgeInboundEvent {
        event_id,
        sequence_num,
        event_type,
        payload,
        raw: value,
    })
}

#[cfg(feature = "network")]
fn json_u64(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

#[cfg(feature = "network")]
fn update_sequence_num(sequence_num: &AtomicU64, sequence: u64) {
    let _ = sequence_num.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
        (sequence > current).then_some(sequence)
    });
}

#[cfg(feature = "network")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct SseFrame {
    event: Option<String>,
    id: Option<String>,
    data: String,
}

#[cfg(feature = "network")]
fn parse_sse_frames(buffer: &mut String, chunk: &str) -> Vec<SseFrame> {
    buffer.push_str(chunk);
    *buffer = buffer.replace("\r\n", "\n").replace('\r', "\n");
    let mut frames = Vec::new();
    while let Some(frame_end) = buffer.find("\n\n") {
        let raw_frame = buffer[..frame_end].to_owned();
        buffer.drain(..frame_end + 2);
        if let Some(frame) = parse_sse_frame(&raw_frame) {
            frames.push(frame);
        }
    }
    frames
}

#[cfg(feature = "network")]
fn parse_sse_frame(raw_frame: &str) -> Option<SseFrame> {
    let mut event = None;
    let mut id = None;
    let mut data = Vec::new();
    for line in raw_frame.lines() {
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        let (field, value) = line.split_once(':').unwrap_or((line, ""));
        let value = value.strip_prefix(' ').unwrap_or(value);
        match field {
            "event" => event = Some(value.to_owned()),
            "id" => id = Some(value.to_owned()),
            "data" => data.push(value.to_owned()),
            _ => {}
        }
    }
    (!data.is_empty()).then(|| SseFrame {
        event,
        id,
        data: data.join("\n"),
    })
}

#[cfg(feature = "network")]
fn bridge_json_headers(access_token: &str) -> Result<http::HeaderMap> {
    let mut headers = anthropic_headers(access_token)?;
    headers.insert(
        "anthropic-client-platform",
        http::HeaderValue::from_static("claude_code_sdk"),
    );
    Ok(headers)
}

#[cfg(feature = "network")]
fn bridge_sse_headers(access_token: &str, sequence_num: u64) -> Result<http::HeaderMap> {
    let mut headers = bridge_json_headers(access_token)?;
    headers.insert(
        http::header::ACCEPT,
        http::HeaderValue::from_static("text/event-stream"),
    );
    if sequence_num > 0 {
        headers.insert(
            "Last-Event-ID",
            http::HeaderValue::from_str(&sequence_num.to_string())
                .map_err(|error| ClaudeAgentError::InvalidOption(error.to_string()))?,
        );
    }
    Ok(headers)
}

#[cfg(feature = "network")]
fn ensure_success(status: http::StatusCode, context: &str) -> Result<()> {
    if status.is_success() {
        Ok(())
    } else {
        Err(ClaudeAgentError::Connection(format!(
            "{context} returned HTTP {status}"
        )))
    }
}

#[cfg(feature = "network")]
fn anthropic_headers(access_token: &str) -> Result<http::HeaderMap> {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::AUTHORIZATION,
        http::HeaderValue::from_str(&format!("Bearer {access_token}"))
            .map_err(|error| ClaudeAgentError::InvalidOption(error.to_string()))?,
    );
    headers.insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "anthropic-version",
        http::HeaderValue::from_static("2023-06-01"),
    );
    headers.insert(
        http::header::USER_AGENT,
        http::HeaderValue::from_str(&format!("claude-agent-sdk-rust/{}", crate::SDK_VERSION))
            .map_err(|error| ClaudeAgentError::InvalidOption(error.to_string()))?,
    );
    Ok(headers)
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(not(feature = "network"))]
fn network_feature_required() -> ClaudeAgentError {
    ClaudeAgentError::InvalidOption(
        "bridge network helpers require enabling the `network` crate feature".into(),
    )
}

fn default_cwd() -> String {
    std::env::current_dir()
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_code_session_body_shape() {
        let tags = vec!["sdk".to_owned()];
        let request = CreateCodeSessionRequest::new(
            "https://console.anthropic.test/",
            "access-token",
            "My session",
            5000,
        )
        .tag("sdk")
        .cwd("/workspace")
        .model("sonnet")
        .git_context(CodeSessionGitContext {
            git_repo_url: "https://github.com/a/b".into(),
            branch: "main".into(),
            default_branch: Some("main".into()),
        });
        assert_eq!(request.tags, tags);

        let body = create_code_session_body(
            "My session",
            &tags,
            Some(&CodeSessionGitContext {
                git_repo_url: "https://github.com/a/b".into(),
                branch: "main".into(),
                default_branch: Some("main".into()),
            }),
            Some("/workspace"),
            Some("sonnet"),
        );
        assert_eq!(body["title"], "My session");
        assert_eq!(body["bridge"], json!({}));
        assert_eq!(body["tags"], json!(["sdk"]));
        assert_eq!(body["config"]["cwd"], "/workspace");
        assert_eq!(body["config"]["model"], "sonnet");
        assert_eq!(
            body["config"]["git_context"]["gitRepoUrl"],
            "https://github.com/a/b"
        );
        assert_eq!(request.body(), body);
    }

    #[test]
    fn parses_bridge_response_shapes() {
        assert_eq!(
            parse_code_session_id(&json!({"session": {"id": "cse_123"}})).as_deref(),
            Some("cse_123")
        );
        assert!(parse_code_session_id(&json!({"session": {"id": "bad"}})).is_none());

        let credentials = parse_remote_credentials(json!({
            "worker_jwt": "jwt",
            "api_base_url": "https://api.example",
            "expires_in": 3600,
            "worker_epoch": "7"
        }))
        .unwrap();
        assert_eq!(credentials.worker_epoch, 7);
        assert_eq!(
            parse_remote_credentials(json!({
                "worker_jwt": "jwt",
                "api_base_url": "https://api.example",
                "expires_in": 3600,
                "worker_epoch": 8
            }))
            .unwrap()
            .worker_epoch,
            8
        );
        assert!(parse_remote_credentials(json!({"worker_epoch": {}})).is_none());

        assert_eq!(
            credentials_failure_reason(&json!({"error": {"resource": "untrusted_device"}}), None),
            Some(CredentialsFailureReason::UntrustedDevice)
        );
        assert_eq!(
            credentials_failure_reason(
                &json!({"error": {"resource": "session_stale_relogin"}}),
                None
            ),
            Some(CredentialsFailureReason::SessionStaleRelogin)
        );
        assert_eq!(
            credentials_failure_reason(&json!({}), Some("please use a trusted device")),
            Some(CredentialsFailureReason::UntrustedDevice)
        );
        assert_eq!(credentials_failure_reason(&json!({}), None), None);
    }

    #[test]
    fn bridge_contract_types_serialize_current_wire_names() {
        assert_eq!(
            serde_json::to_value(BridgeSessionState::RequiresAction).unwrap(),
            "requires_action"
        );
        assert_eq!(
            serde_json::to_value(ConnectRemoteControlError {
                kind: ConnectRemoteControlErrorKind::Auth,
                detail: "expired".into(),
            })
            .unwrap(),
            json!({"kind": "auth", "detail": "expired"})
        );

        let options = ConnectRemoteControlOptions {
            dir: "/repo".into(),
            registration_dir: None,
            name: Some("worker".into()),
            worker_type: Some("assistant".into()),
            branch: Some("main".into()),
            git_repo_url: Some("https://github.com/a/b".into()),
            base_url: "https://claude.ai".into(),
            org_uuid: "org-1".into(),
            model: "sonnet".into(),
            perpetual: Some(true),
            initial_sse_sequence_num: Some(42),
        };
        let serialized = serde_json::to_value(options).unwrap();
        assert_eq!(serialized["orgUUID"], "org-1");
        assert_eq!(serialized["initialSSESequenceNum"], 42);

        let failure = RemoteCredentialsOutcome::Failure(CredentialsFailure {
            terminal: true,
            reason: CredentialsFailureReason::UntrustedDevice,
        });
        assert!(is_credentials_failure(&failure));
        assert!(!is_credentials_failure(
            &RemoteCredentialsOutcome::Credentials(RemoteCredentials {
                worker_jwt: "jwt".into(),
                api_base_url: "https://api.example".into(),
                expires_in: 1,
                worker_epoch: 2,
            })
        ));
    }

    #[cfg(feature = "network")]
    #[tokio::test]
    async fn create_code_session_posts_current_http_contract() {
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_http_request(&mut stream).await;
            assert!(
                request.starts_with("POST /v1/code/sessions HTTP/1.1"),
                "{request}"
            );
            assert!(
                request.contains("Authorization: Bearer access-token")
                    || request.contains("authorization: Bearer access-token"),
                "{request}"
            );
            assert!(
                request.contains("anthropic-version: 2023-06-01")
                    || request.contains("Anthropic-Version: 2023-06-01"),
                "{request}"
            );
            assert!(
                request.contains("\"title\":\"Network session\""),
                "{request}"
            );
            assert!(request.contains("\"bridge\":{}"), "{request}");
            assert!(request.contains("\"cwd\":\"/workspace\""), "{request}");
            assert!(request.contains("\"model\":\"sonnet\""), "{request}");
            let body = json!({"session": {"id": "cse_network_123"}}).to_string();
            write_http_json(&mut stream, "201 Created", &body).await;
        });

        let session_id = create_code_session(
            CreateCodeSessionRequest::new(base_url, "access-token", "Network session", 5000)
                .cwd("/workspace")
                .model("sonnet"),
        )
        .await
        .unwrap();
        assert_eq!(session_id.as_deref(), Some("cse_network_123"));
        server.await.unwrap();
    }

    #[cfg(feature = "network")]
    #[tokio::test]
    async fn fetch_remote_credentials_handles_success_and_terminal_failure() {
        use tokio::net::TcpListener;

        let success_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let success_base_url = format!("http://{}", success_listener.local_addr().unwrap());
        let success_server = tokio::spawn(async move {
            let (mut stream, _) = success_listener.accept().await.unwrap();
            let request = read_http_request(&mut stream).await;
            assert!(
                request.starts_with("POST /v1/code/sessions/cse_1/bridge HTTP/1.1"),
                "{request}"
            );
            assert!(
                request.contains("X-Trusted-Device-Token: trusted-1")
                    || request.contains("x-trusted-device-token: trusted-1"),
                "{request}"
            );
            let body = json!({
                "worker_jwt": "jwt",
                "api_base_url": "https://api.example",
                "expires_in": 3600,
                "worker_epoch": 11
            })
            .to_string();
            write_http_json(&mut stream, "200 OK", &body).await;
        });

        let credentials = fetch_remote_credentials(
            "cse_1",
            &success_base_url,
            "access-token",
            5000,
            Some("trusted-1"),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(matches!(
            credentials,
            RemoteCredentialsOutcome::Credentials(RemoteCredentials {
                worker_epoch: 11,
                ..
            })
        ));
        success_server.await.unwrap();

        let failure_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let failure_base_url = format!("http://{}", failure_listener.local_addr().unwrap());
        let failure_server = tokio::spawn(async move {
            let (mut stream, _) = failure_listener.accept().await.unwrap();
            let _request = read_http_request(&mut stream).await;
            let body = json!({"error": {"resource": "session_stale_relogin"}}).to_string();
            write_http_json(&mut stream, "403 Forbidden", &body).await;
        });

        let failure =
            fetch_remote_credentials("cse_2", &failure_base_url, "access-token", 5000, None)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(
            failure,
            RemoteCredentialsOutcome::Failure(CredentialsFailure {
                terminal: true,
                reason: CredentialsFailureReason::SessionStaleRelogin,
            })
        );
        failure_server.await.unwrap();
    }

    #[test]
    fn bridge_attach_contract_types_serialize_current_wire_names() {
        let options =
            AttachBridgeSessionOptions::new("cse_attach_1", "worker-jwt", "https://api.example/")
                .epoch(12)
                .initial_sequence_num(34)
                .heartbeat_interval_ms(60_000)
                .outbound_only(true);
        let serialized = serde_json::to_value(options).unwrap();
        assert_eq!(serialized["sessionId"], "cse_attach_1");
        assert_eq!(serialized["ingressToken"], "worker-jwt");
        assert_eq!(serialized["apiBaseUrl"], "https://api.example/");
        assert_eq!(serialized["epoch"], 12);
        assert_eq!(serialized["initialSequenceNum"], 34);
        assert_eq!(serialized["heartbeatIntervalMs"], 60_000);
        assert_eq!(serialized["outboundOnly"], true);

        assert_eq!(
            serde_json::to_value(BridgeDeliveryStatus::Processing).unwrap(),
            "processing"
        );
        assert_eq!(
            serde_json::to_value(BridgeDeliveryStatus::Processed).unwrap(),
            "processed"
        );
    }

    #[cfg(feature = "network")]
    #[tokio::test]
    async fn attach_bridge_session_registers_initializes_and_posts_events() {
        use crate::status::{SDKControlRequest, SDKControlResponse};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            for step in 0..10 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_http_request(&mut stream).await;
                match step {
                    0 => {
                        assert!(
                            request.starts_with(
                                "POST /v1/code/sessions/cse_attach_1/worker/register HTTP/1.1"
                            ),
                            "{request}"
                        );
                        assert!(
                            request.contains("Authorization: Bearer worker-jwt")
                                || request.contains("authorization: Bearer worker-jwt"),
                            "{request}"
                        );
                        write_http_json(&mut stream, "200 OK", r#"{"worker_epoch":"12"}"#).await;
                    }
                    1 => {
                        assert!(
                            request
                                .starts_with("PUT /v1/code/sessions/cse_attach_1/worker HTTP/1.1"),
                            "{request}"
                        );
                        let body = request_json_body(&request);
                        assert_eq!(body["worker_epoch"], 12);
                        assert_eq!(body["worker_status"], "idle");
                        assert_eq!(body["external_metadata"]["pending_action"], Value::Null);
                        assert_eq!(body["external_metadata"]["task_summary"], Value::Null);
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    2 => {
                        assert!(
                            request.starts_with(
                                "POST /v1/code/sessions/cse_attach_1/worker/events HTTP/1.1"
                            ),
                            "{request}"
                        );
                        let payload = request_json_body(&request)["events"][0]["payload"].clone();
                        assert_eq!(payload["type"], "user");
                        assert_eq!(payload["session_id"], "cse_attach_1");
                        assert!(payload["uuid"].as_str().is_some());
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    3 => {
                        let payload = request_json_body(&request)["events"][0]["payload"].clone();
                        assert_eq!(payload["type"], "result");
                        assert_eq!(payload["subtype"], "success");
                        assert_eq!(payload["session_id"], "cse_attach_1");
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    4 => {
                        let payload = request_json_body(&request)["events"][0]["payload"].clone();
                        assert_eq!(payload["type"], "control_request");
                        assert_eq!(payload["request_id"], "ctrl-1");
                        assert_eq!(payload["session_id"], "cse_attach_1");
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    5 => {
                        let payload = request_json_body(&request)["events"][0]["payload"].clone();
                        assert_eq!(payload["type"], "control_response");
                        assert_eq!(payload["response"]["subtype"], "success");
                        assert_eq!(payload["response"]["request_id"], "ctrl-1");
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    6 => {
                        let payload = request_json_body(&request)["events"][0]["payload"].clone();
                        assert_eq!(payload["type"], "control_cancel_request");
                        assert_eq!(payload["request_id"], "ctrl-2");
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    7 => {
                        assert!(
                            request
                                .starts_with("PUT /v1/code/sessions/cse_attach_1/worker HTTP/1.1"),
                            "{request}"
                        );
                        let body = request_json_body(&request);
                        assert_eq!(body["worker_epoch"], 12);
                        assert_eq!(body["worker_status"], "running");
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    8 => {
                        let body = request_json_body(&request);
                        assert_eq!(body["external_metadata"]["branch"], "main");
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    9 => {
                        assert!(
                            request.starts_with(
                                "POST /v1/code/sessions/cse_attach_1/worker/events/delivery HTTP/1.1"
                            ),
                            "{request}"
                        );
                        let body = request_json_body(&request);
                        assert_eq!(body["updates"][0]["event_id"], "event-1");
                        assert_eq!(body["updates"][0]["status"], "processed");
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    _ => unreachable!(),
                }
            }
        });

        let mut handle = attach_bridge_session(
            AttachBridgeSessionOptions::new("cse_attach_1", "worker-jwt", base_url)
                .outbound_only(true)
                .heartbeat_interval_ms(60_000),
        )
        .await
        .unwrap();
        assert!(handle.is_connected());
        assert_eq!(handle.session_id(), "cse_attach_1");
        assert_eq!(handle.get_sequence_num(), 0);

        handle
            .write(json!({"type": "user", "message": {"role": "user", "content": "hi"}}))
            .await
            .unwrap();
        handle.send_result().await.unwrap();
        handle
            .send_control_request(SDKControlRequest::new(
                "ctrl-1",
                json!({"subtype": "can_use_tool"}),
            ))
            .await
            .unwrap();
        handle
            .send_control_response(SDKControlResponse::success(
                "ctrl-1",
                json!({"behavior": "allow"}),
            ))
            .await
            .unwrap();
        handle.send_control_cancel_request("ctrl-2").await.unwrap();
        handle
            .report_state(BridgeSessionState::Running)
            .await
            .unwrap();
        handle
            .report_metadata(json!({"branch": "main"}))
            .await
            .unwrap();
        handle
            .report_delivery("event-1", BridgeDeliveryStatus::Processed)
            .await
            .unwrap();
        handle.flush().await.unwrap();
        handle.close();
        server.await.unwrap();
    }

    #[cfg(feature = "network")]
    #[tokio::test]
    async fn attach_bridge_session_streams_sse_inbound_events_and_delivery_acks() {
        use std::time::Duration;
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            for step in 0..4 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_http_request(&mut stream).await;
                match step {
                    0 => {
                        assert!(
                            request.starts_with("PUT /v1/code/sessions/cse_sse/worker HTTP/1.1"),
                            "{request}"
                        );
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    1 => {
                        assert!(
                            request.starts_with(
                                "GET /v1/code/sessions/cse_sse/worker/events/stream?from_sequence_num=5 HTTP/1.1"
                            ),
                            "{request}"
                        );
                        assert!(
                            request.contains("Accept: text/event-stream")
                                || request.contains("accept: text/event-stream"),
                            "{request}"
                        );
                        let payload = json!({
                            "event_id": "evt-1",
                            "sequence_num": 6,
                            "event_type": "user",
                            "payload": {
                                "type": "user",
                                "message": {"role": "user", "content": "remote"},
                                "session_id": "cse_sse"
                            }
                        });
                        write_http_sse(
                            &mut stream,
                            &format!("id: 6\nevent: client_event\ndata: {}\n\n", payload),
                        )
                        .await;
                    }
                    2 | 3 => {
                        assert!(
                            request.starts_with(
                                "POST /v1/code/sessions/cse_sse/worker/events/delivery HTTP/1.1"
                            ),
                            "{request}"
                        );
                        let body = request_json_body(&request);
                        assert_eq!(body["updates"][0]["event_id"], "evt-1");
                        let status = body["updates"][0]["status"].as_str().unwrap();
                        assert!(matches!(status, "received" | "processed"));
                        write_http_json(&mut stream, "200 OK", "{}").await;
                    }
                    _ => unreachable!(),
                }
            }
        });

        let mut handle = attach_bridge_session(
            AttachBridgeSessionOptions::new("cse_sse", "worker-jwt", base_url)
                .epoch(7)
                .initial_sequence_num(5)
                .heartbeat_interval_ms(60_000),
        )
        .await
        .unwrap();
        let inbound = tokio::time::timeout(Duration::from_secs(3), handle.next_inbound())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(inbound.event_id.as_deref(), Some("evt-1"));
        assert_eq!(inbound.sequence_num, Some(6));
        assert_eq!(inbound.event_type.as_deref(), Some("user"));
        assert_eq!(inbound.payload["type"], "user");
        assert_eq!(handle.get_sequence_num(), 6);
        handle.close();
        server.await.unwrap();
    }

    #[cfg(feature = "network")]
    async fn read_http_request(stream: &mut tokio::net::TcpStream) -> String {
        use tokio::io::AsyncReadExt;

        let mut buffer = Vec::new();
        let mut temp = [0; 1024];
        loop {
            let read = stream.read(&mut temp).await.unwrap();
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..read]);
            if let Some(header_end) = find_subslice(&buffer, b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&buffer[..header_end + 4]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        key.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())?
                    })
                    .unwrap_or(0);
                while buffer.len().saturating_sub(header_end + 4) < content_length {
                    let read = stream.read(&mut temp).await.unwrap();
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&temp[..read]);
                }
                break;
            }
        }
        String::from_utf8_lossy(&buffer).into_owned()
    }

    #[cfg(feature = "network")]
    async fn write_http_json(stream: &mut tokio::net::TcpStream, status: &str, body: &str) {
        use tokio::io::AsyncWriteExt;

        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    }

    #[cfg(feature = "network")]
    async fn write_http_sse(stream: &mut tokio::net::TcpStream, body: &str) {
        use tokio::io::AsyncWriteExt;

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    }

    #[cfg(feature = "network")]
    fn request_json_body(request: &str) -> Value {
        let body = request
            .split_once("\r\n\r\n")
            .map(|(_, body)| body)
            .unwrap_or("");
        serde_json::from_str(body).unwrap_or_else(|error| {
            panic!("failed to parse request body as JSON: {error}; request={request}")
        })
    }

    #[cfg(feature = "network")]
    fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}
