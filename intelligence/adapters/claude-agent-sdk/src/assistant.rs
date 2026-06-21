use std::{future::Future, pin::Pin, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{RwLock, mpsc, oneshot};

use crate::{
    AttachBridgeSessionOptions, ClaudeAgentError, ClaudeAgentOptions, Message, Result, UserMessage,
};

#[cfg(feature = "network")]
use crate::{
    BridgeInboundEvent, BridgeSessionHandle, BridgeSessionState, UserContent, query_stream,
};

/// Worker-persisted state for assistant/bridge sessions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_session_id: Option<String>,
    #[serde(
        default,
        rename = "lastSSESequenceNum",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_sse_sequence_num: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge_session_id: Option<String>,
}

pub type WorkerStateFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

/// Async persistence hook for assistant worker state.
pub trait WorkerStateAdapter: Send + Sync {
    fn load(&self) -> WorkerStateFuture<Option<WorkerState>>;
    fn save(&self, state: WorkerState) -> WorkerStateFuture<()>;
}

pub type BuildAssistantQueryOptionsFuture =
    Pin<Box<dyn Future<Output = Result<ClaudeAgentOptions>> + Send>>;
pub type BuildAssistantQueryOptionsCallback =
    Arc<dyn Fn(ClaudeAgentOptions) -> BuildAssistantQueryOptionsFuture + Send + Sync>;
pub type TransformAssistantOutboundCallback = Arc<dyn Fn(Message) -> Option<Value> + Send + Sync>;
pub type AssistantLogCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Options for the Rust assistant worker wrapper.
///
/// This is the host-attached variant of the alpha TypeScript assistant worker:
/// callers provide an already-mintable bridge session (`AttachBridgeSessionOptions`)
/// plus query options for spawning Claude Code. Credential polling/environment
/// registration remains host-specific; the worker loop itself is implemented
/// here.
#[derive(Clone)]
pub struct AssistantWorkerOptions {
    pub bridge: AttachBridgeSessionOptions,
    pub query_options: ClaudeAgentOptions,
    pub sandboxed: bool,
    pub initial_prompt: Option<String>,
    pub user_idle_ms: u64,
    pub state_adapter: Option<Arc<dyn WorkerStateAdapter>>,
    pub build_query_options: Option<BuildAssistantQueryOptionsCallback>,
    pub transform_outbound: Option<TransformAssistantOutboundCallback>,
    pub log: Option<AssistantLogCallback>,
}

impl AssistantWorkerOptions {
    pub fn new(bridge: AttachBridgeSessionOptions, query_options: ClaudeAgentOptions) -> Self {
        Self {
            bridge,
            query_options,
            sandboxed: false,
            initial_prompt: None,
            user_idle_ms: 300_000,
            state_adapter: None,
            build_query_options: None,
            transform_outbound: None,
            log: None,
        }
    }

    pub fn sandboxed(mut self, sandboxed: bool) -> Self {
        self.sandboxed = sandboxed;
        self
    }

    pub fn initial_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.initial_prompt = Some(prompt.into());
        self
    }

    pub fn user_idle_ms(mut self, user_idle_ms: u64) -> Self {
        self.user_idle_ms = user_idle_ms;
        self
    }

    pub fn state_adapter(mut self, adapter: Arc<dyn WorkerStateAdapter>) -> Self {
        self.state_adapter = Some(adapter);
        self
    }

    pub fn build_query_options<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn(ClaudeAgentOptions) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ClaudeAgentOptions>> + Send + 'static,
    {
        self.build_query_options = Some(Arc::new(move |base| Box::pin(callback(base))));
        self
    }

    pub fn transform_outbound<F>(mut self, callback: F) -> Self
    where
        F: Fn(Message) -> Option<Value> + Send + Sync + 'static,
    {
        self.transform_outbound = Some(Arc::new(callback));
        self
    }

    pub fn log<F>(mut self, callback: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.log = Some(Arc::new(callback));
        self
    }
}

/// Structured failure returned by [`run_assistant_worker`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantWorkerError {
    pub kind: AssistantWorkerErrorKind,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssistantWorkerErrorKind {
    Conflict,
    Auth,
    Network,
    Unknown,
}

pub type AssistantWorkerResult = std::result::Result<AssistantWorkerHandle, AssistantWorkerError>;

/// Running assistant worker handle.
pub struct AssistantWorkerHandle {
    session_url: String,
    bridge_session_id: String,
    claude_session_id: Arc<RwLock<Option<String>>>,
    command_tx: mpsc::Sender<AssistantWorkerCommand>,
    worker: Option<tokio::task::JoinHandle<Result<()>>>,
}

impl AssistantWorkerHandle {
    pub fn session_url(&self) -> &str {
        &self.session_url
    }

    pub fn bridge_session_id(&self) -> &str {
        &self.bridge_session_id
    }

    pub async fn claude_session_id(&self) -> Option<String> {
        self.claude_session_id.read().await.clone()
    }

    pub async fn push_prompt(&self, content: impl Into<String>) -> Result<()> {
        self.push_user_message(UserMessage::text(content)).await
    }

    pub async fn push_user_message(&self, message: UserMessage) -> Result<()> {
        self.command_tx
            .send(AssistantWorkerCommand::Prompt(Box::new(message)))
            .await
            .map_err(|_| ClaudeAgentError::Connection("assistant worker is closed".into()))
    }

    pub async fn interrupt(&self) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();
        self.command_tx
            .send(AssistantWorkerCommand::Interrupt(response_tx))
            .await
            .map_err(|_| ClaudeAgentError::Connection("assistant worker is closed".into()))?;
        response_rx
            .await
            .map_err(|_| ClaudeAgentError::Connection("assistant worker is closed".into()))?
    }

    pub async fn teardown(&mut self) -> Result<()> {
        if let Some(worker) = self.worker.take() {
            let (response_tx, response_rx) = oneshot::channel();
            let _ = self
                .command_tx
                .send(AssistantWorkerCommand::Shutdown(response_tx))
                .await;
            let _ = response_rx.await;
            worker
                .await
                .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?
        } else {
            Ok(())
        }
    }

    pub async fn done(&mut self) -> Result<()> {
        if let Some(worker) = self.worker.take() {
            worker
                .await
                .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?
        } else {
            Ok(())
        }
    }
}

impl Drop for AssistantWorkerHandle {
    fn drop(&mut self) {
        if let Some(worker) = &self.worker
            && !worker.is_finished()
        {
            worker.abort();
        }
    }
}

#[allow(dead_code)]
enum AssistantWorkerCommand {
    Prompt(Box<UserMessage>),
    Interrupt(oneshot::Sender<Result<()>>),
    Shutdown(oneshot::Sender<()>),
}

#[cfg(feature = "network")]
pub async fn run_assistant_worker(options: AssistantWorkerOptions) -> AssistantWorkerResult {
    run_assistant_worker_inner(options)
        .await
        .map_err(AssistantWorkerError::from)
}

#[cfg(not(feature = "network"))]
pub async fn run_assistant_worker(_options: AssistantWorkerOptions) -> AssistantWorkerResult {
    Err(AssistantWorkerError {
        kind: AssistantWorkerErrorKind::Unknown,
        detail: "assistant worker requires enabling the `network` crate feature".into(),
    })
}

#[cfg(feature = "network")]
async fn run_assistant_worker_inner(
    mut options: AssistantWorkerOptions,
) -> Result<AssistantWorkerHandle> {
    let state = match &options.state_adapter {
        Some(adapter) => adapter.load().await?,
        None => None,
    };
    if options.bridge.initial_sequence_num.is_none() {
        options.bridge.initial_sequence_num =
            state.as_ref().and_then(|state| state.last_sse_sequence_num);
    }
    let bridge = crate::attach_bridge_session(options.bridge.clone()).await?;
    let session_url = bridge.api_session_url().to_owned();
    let bridge_session_id = bridge.session_id().to_owned();
    let claude_session_id = Arc::new(RwLock::new(state.and_then(|state| state.claude_session_id)));
    let (command_tx, command_rx) = mpsc::channel(100);
    if let Some(prompt) = options.initial_prompt.take() {
        command_tx
            .send(AssistantWorkerCommand::Prompt(Box::new(UserMessage::text(
                prompt,
            ))))
            .await
            .map_err(|_| ClaudeAgentError::Connection("assistant worker failed to start".into()))?;
    }
    let worker_claude_session_id = Arc::clone(&claude_session_id);
    let worker = tokio::spawn(async move {
        assistant_worker_loop(options, bridge, command_rx, worker_claude_session_id).await
    });
    Ok(AssistantWorkerHandle {
        session_url,
        bridge_session_id,
        claude_session_id,
        command_tx,
        worker: Some(worker),
    })
}

#[cfg(feature = "network")]
struct ActiveQuery {
    query: crate::Query,
    input_tx: mpsc::Sender<UserMessage>,
}

#[cfg(feature = "network")]
enum WorkerEvent {
    Command(Option<AssistantWorkerCommand>),
    Bridge(Result<Option<BridgeInboundEvent>>),
    Query(Box<Option<Result<Message>>>),
}

#[cfg(feature = "network")]
async fn assistant_worker_loop(
    options: AssistantWorkerOptions,
    mut bridge: BridgeSessionHandle,
    mut command_rx: mpsc::Receiver<AssistantWorkerCommand>,
    claude_session_id: Arc<RwLock<Option<String>>>,
) -> Result<()> {
    let mut active: Option<ActiveQuery> = None;
    loop {
        let event = if let Some(active_query) = active.as_mut() {
            tokio::select! {
                command = command_rx.recv() => WorkerEvent::Command(command),
                inbound = bridge.next_inbound() => WorkerEvent::Bridge(inbound),
                message = futures::StreamExt::next(&mut active_query.query) => WorkerEvent::Query(Box::new(message)),
            }
        } else {
            tokio::select! {
                command = command_rx.recv() => WorkerEvent::Command(command),
                inbound = bridge.next_inbound() => WorkerEvent::Bridge(inbound),
            }
        };

        match event {
            WorkerEvent::Command(Some(AssistantWorkerCommand::Prompt(message))) => {
                send_or_start_query(
                    &options,
                    &mut bridge,
                    &mut active,
                    *message,
                    Arc::clone(&claude_session_id),
                )
                .await?;
            }
            WorkerEvent::Command(Some(AssistantWorkerCommand::Interrupt(response_tx))) => {
                let result = match active.as_ref() {
                    Some(active_query) => active_query.query.interrupt().await,
                    None => Ok(()),
                };
                let _ = response_tx.send(result);
            }
            WorkerEvent::Command(Some(AssistantWorkerCommand::Shutdown(response_tx))) => {
                if let Some(mut active_query) = active.take() {
                    active_query.query.close();
                }
                bridge.close();
                let _ = response_tx.send(());
                break;
            }
            WorkerEvent::Command(None) => break,
            WorkerEvent::Bridge(Ok(Some(event))) => {
                if let Some(message) = inbound_user_message(event.payload)? {
                    send_or_start_query(
                        &options,
                        &mut bridge,
                        &mut active,
                        message,
                        Arc::clone(&claude_session_id),
                    )
                    .await?;
                }
            }
            WorkerEvent::Bridge(Ok(None)) => {}
            WorkerEvent::Bridge(Err(error)) => return Err(error),
            WorkerEvent::Query(message) => match *message {
                Some(Ok(message)) => {
                    let is_result = matches!(message, Message::Result(_));
                    update_claude_session_id(&claude_session_id, &message).await;
                    persist_worker_state(&options, &bridge, &claude_session_id).await;
                    let outbound = match &options.transform_outbound {
                        Some(transform) => transform(message),
                        None => Some(serde_json::to_value(&message)?),
                    };
                    if let Some(outbound) = outbound {
                        bridge.write(outbound).await?;
                    }
                    if is_result {
                        bridge.report_state(BridgeSessionState::Idle).await?;
                        active = None;
                    }
                }
                Some(Err(error)) => return Err(error),
                None => {
                    bridge.report_state(BridgeSessionState::Idle).await?;
                    active = None;
                }
            },
        }
    }
    persist_worker_state(&options, &bridge, &claude_session_id).await;
    Ok(())
}

#[cfg(feature = "network")]
async fn send_or_start_query(
    options: &AssistantWorkerOptions,
    bridge: &mut BridgeSessionHandle,
    active: &mut Option<ActiveQuery>,
    message: UserMessage,
    claude_session_id: Arc<RwLock<Option<String>>>,
) -> Result<()> {
    if let Some(active_query) = active.as_ref() {
        active_query
            .input_tx
            .send(message)
            .await
            .map_err(|_| ClaudeAgentError::Connection("active assistant query is closed".into()))?;
        return Ok(());
    }

    bridge.report_state(BridgeSessionState::Running).await?;
    let (input_tx, input_rx) = mpsc::channel(100);
    let input_stream = futures::stream::unfold(input_rx, |mut rx| async move {
        rx.recv().await.map(|message| (message, rx))
    });
    let mut query_options = options.query_options.clone();
    if query_options.resume.is_none() {
        query_options.resume = claude_session_id.read().await.clone();
    }
    if options.sandboxed {
        query_options
            .env
            .insert("CLAUDE_CODE_SANDBOXED".into(), "1".into());
    }
    if let Some(build) = &options.build_query_options {
        query_options = build(query_options).await?;
    }
    let query = query_stream(input_stream, query_options)?;
    input_tx
        .send(message)
        .await
        .map_err(|_| ClaudeAgentError::Connection("assistant query input closed".into()))?;
    *active = Some(ActiveQuery { query, input_tx });
    Ok(())
}

#[cfg(feature = "network")]
fn inbound_user_message(payload: Value) -> Result<Option<UserMessage>> {
    if payload.get("type").and_then(Value::as_str) != Some("user") {
        return Ok(None);
    }
    let content = payload
        .get("message")
        .and_then(|message| message.get("content"))
        .cloned()
        .ok_or_else(|| ClaudeAgentError::MessageParse {
            message: "bridge user payload missing message.content".into(),
            data: payload.clone(),
        })?;
    let content = serde_json::from_value::<UserContent>(content)?;
    let mut message = UserMessage::new_for_assistant_worker(content);
    message.uuid = payload
        .get("uuid")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    Ok(Some(message))
}

#[cfg(feature = "network")]
async fn update_claude_session_id(
    claude_session_id: &Arc<RwLock<Option<String>>>,
    message: &Message,
) {
    let session_id = match message {
        Message::Assistant(message) => message.session_id.clone(),
        Message::Status(message) => Some(message.session_id.clone()),
        Message::Result(message) => Some(message.session_id.clone()),
        Message::StreamEvent(message) => Some(message.session_id.clone()),
        _ => None,
    };
    if let Some(session_id) = session_id {
        *claude_session_id.write().await = Some(session_id);
    }
}

#[cfg(feature = "network")]
async fn persist_worker_state(
    options: &AssistantWorkerOptions,
    bridge: &BridgeSessionHandle,
    claude_session_id: &Arc<RwLock<Option<String>>>,
) {
    let Some(adapter) = &options.state_adapter else {
        return;
    };
    let state = WorkerState {
        claude_session_id: claude_session_id.read().await.clone(),
        last_sse_sequence_num: Some(bridge.get_sequence_num()),
        bridge_session_id: Some(bridge.session_id().to_owned()),
    };
    if let Err(error) = adapter.save(state).await {
        if let Some(log) = &options.log {
            log(format!("stateAdapter.save failed: {error}"));
        }
    }
}

#[cfg(feature = "network")]
impl AssistantWorkerError {
    fn from(error: ClaudeAgentError) -> Self {
        let detail = error.to_string();
        let kind = match &error {
            ClaudeAgentError::Connection(message)
                if message.contains("401")
                    || message.contains("403")
                    || message.to_ascii_lowercase().contains("auth") =>
            {
                AssistantWorkerErrorKind::Auth
            }
            ClaudeAgentError::Connection(message) if message.contains("409") => {
                AssistantWorkerErrorKind::Conflict
            }
            ClaudeAgentError::Connection(_) | ClaudeAgentError::Process { .. } => {
                AssistantWorkerErrorKind::Network
            }
            _ => AssistantWorkerErrorKind::Unknown,
        };
        Self { kind, detail }
    }
}

#[cfg(feature = "network")]
trait AssistantWorkerUserMessageExt {
    fn new_for_assistant_worker(content: UserContent) -> Self;
}

#[cfg(feature = "network")]
impl AssistantWorkerUserMessageExt for UserMessage {
    fn new_for_assistant_worker(content: UserContent) -> Self {
        match content {
            UserContent::Text(text) => UserMessage::text(text),
            UserContent::Blocks(blocks) => UserMessage::blocks(blocks),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn assistant_worker_contract_types_serialize_current_wire_names() {
        let state = WorkerState {
            claude_session_id: Some("claude-1".into()),
            last_sse_sequence_num: Some(42),
            bridge_session_id: Some("cse_1".into()),
        };
        assert_eq!(
            serde_json::to_value(state).unwrap(),
            json!({
                "claudeSessionId": "claude-1",
                "lastSSESequenceNum": 42,
                "bridgeSessionId": "cse_1",
            })
        );
        assert_eq!(
            serde_json::to_value(AssistantWorkerError {
                kind: AssistantWorkerErrorKind::Auth,
                detail: "expired".into(),
            })
            .unwrap(),
            json!({"kind": "auth", "detail": "expired"})
        );
    }

    #[test]
    fn assistant_worker_options_builders_set_optional_hooks() {
        struct TestStateAdapter;

        impl WorkerStateAdapter for TestStateAdapter {
            fn load(&self) -> WorkerStateFuture<Option<WorkerState>> {
                Box::pin(async { Ok(None) })
            }

            fn save(&self, _state: WorkerState) -> WorkerStateFuture<()> {
                Box::pin(async { Ok(()) })
            }
        }

        let options = AssistantWorkerOptions::new(
            AttachBridgeSessionOptions::new("cse_builder", "jwt", "http://localhost"),
            ClaudeAgentOptions::default(),
        )
        .sandboxed(true)
        .initial_prompt("hello")
        .user_idle_ms(123)
        .state_adapter(Arc::new(TestStateAdapter))
        .build_query_options(|mut base| async move {
            base.model = Some("claude-test".into());
            Ok(base)
        })
        .transform_outbound(|message| serde_json::to_value(message).ok())
        .log(|_message| {});

        assert!(options.sandboxed);
        assert_eq!(options.initial_prompt.as_deref(), Some("hello"));
        assert_eq!(options.user_idle_ms, 123);
        assert!(options.state_adapter.is_some());
        assert!(options.build_query_options.is_some());
        assert!(options.transform_outbound.is_some());
        assert!(options.log.is_some());
    }

    #[cfg(feature = "network")]
    #[test]
    fn assistant_worker_parses_inbound_user_payloads_and_classifies_errors() {
        let message = inbound_user_message(json!({
            "type": "user",
            "uuid": "user-uuid",
            "message": {"role": "user", "content": "from bridge"}
        }))
        .unwrap()
        .unwrap();
        assert_eq!(message.uuid.as_deref(), Some("user-uuid"));
        assert!(matches!(message.content, UserContent::Text(text) if text == "from bridge"));
        assert!(
            inbound_user_message(json!({"type": "assistant", "content": []}))
                .unwrap()
                .is_none()
        );

        let auth = AssistantWorkerError::from(ClaudeAgentError::Connection(
            "bridge returned HTTP 401".into(),
        ));
        assert_eq!(auth.kind, AssistantWorkerErrorKind::Auth);
        let conflict = AssistantWorkerError::from(ClaudeAgentError::Connection(
            "bridge returned HTTP 409".into(),
        ));
        assert_eq!(conflict.kind, AssistantWorkerErrorKind::Conflict);
        let network = AssistantWorkerError::from(ClaudeAgentError::Process {
            exit_code: Some(1),
            message: "child exited".into(),
        });
        assert_eq!(network.kind, AssistantWorkerErrorKind::Network);
    }

    // `run_assistant_worker_forwards_initial_prompt_output_to_bridge` moved to
    // `tests/assistant_worker_fake_cli.rs`: it now drives an in-process Rust fake
    // CLI through the SDK `spawn_claude_code_process` hook (shared
    // `support_fake_cli.rs` harness) instead of spawning an on-disk external
    // script, matching the hermetic pattern used by the sibling fake-CLI tests.
}
