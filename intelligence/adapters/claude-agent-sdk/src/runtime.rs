use std::{collections::BTreeMap, future::Future};

use serde_json::{Map, Value, json};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::{Duration, timeout},
};
use uuid::Uuid;

use crate::{
    callbacks::{
        CallbackRegistry, ElicitationAbortHandle, ElicitationCallbackOptions, ElicitationRequest,
        ElicitationResult, HookCallback, HookCallbackRequest, TokenRefreshCallbackOptions,
        ToolPermissionRequest, UserDialogCallbackOptions, UserDialogRequest,
    },
    error::{ClaudeAgentError, Result},
    messages::{Message, MirrorErrorMessage, UserMessage, parse_message},
    options::{ClaudeAgentOptions, McpServerConfig, PermissionMode},
    session_store::{SessionKey, TranscriptMirrorBatcher},
    status::{ReadFileEncoding, SDKControlInitializeResponse},
    transport::{
        RuntimeTransport, SubprocessTransport, control_error_response, control_request,
        user_message, user_prompt_message,
    },
};

pub(crate) struct SessionHandle {
    command_tx: mpsc::Sender<SessionCommand>,
    message_rx: mpsc::Receiver<Result<Message>>,
    initialization: Value,
    worker: Option<JoinHandle<Result<()>>>,
}

impl SessionHandle {
    pub(crate) async fn connect(
        options: ClaudeAgentOptions,
        initial_prompt: Option<String>,
    ) -> Result<Self> {
        Self::connect_with_initialize_timeout(options, initial_prompt, Duration::from_secs(60))
            .await
    }

    pub(crate) async fn connect_with_initialize_timeout(
        options: ClaudeAgentOptions,
        initial_prompt: Option<String>,
        initialize_timeout: Duration,
    ) -> Result<Self> {
        let (command_tx, command_rx) = mpsc::channel(100);
        let (message_tx, message_rx) = mpsc::channel(100);
        let (init_tx, init_rx) = oneshot::channel();
        let worker = tokio::spawn(session_worker(
            options,
            initial_prompt,
            command_rx,
            message_tx,
            init_tx,
            initialize_timeout,
        ));
        let initialization = init_rx
            .await
            .map_err(|error| ClaudeAgentError::Connection(error.to_string()))??;
        Ok(Self {
            command_tx,
            message_rx,
            initialization,
            worker: Some(worker),
        })
    }

    pub(crate) async fn query(&self, prompt: impl Into<String>) -> Result<()> {
        self.command_tx
            .send(SessionCommand::UserPrompt(prompt.into()))
            .await
            .map_err(|_| ClaudeAgentError::Connection("session worker is closed".into()))
    }

    pub(crate) async fn query_message(&self, message: UserMessage) -> Result<()> {
        self.command_tx
            .send(SessionCommand::UserMessage(Box::new(message)))
            .await
            .map_err(|_| ClaudeAgentError::Connection("session worker is closed".into()))
    }

    pub(crate) async fn control(&self, request: Value) -> Result<Value> {
        let (tx, rx) = oneshot::channel();
        let subtype = request
            .get("subtype")
            .and_then(Value::as_str)
            .unwrap_or("control")
            .to_owned();
        self.command_tx
            .send(SessionCommand::Control {
                request,
                response_tx: tx,
            })
            .await
            .map_err(|_| ClaudeAgentError::Connection("session worker is closed".into()))?;
        timeout(Duration::from_secs(60), rx)
            .await
            .map_err(|_| ClaudeAgentError::ControlTimeout(subtype))?
            .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?
    }

    pub(crate) async fn receive_next(&mut self) -> Option<Result<Message>> {
        self.message_rx.recv().await
    }

    pub(crate) fn initialization(&self) -> &Value {
        &self.initialization
    }

    pub(crate) async fn shutdown(mut self) -> Result<()> {
        let _ = self.command_tx.send(SessionCommand::Shutdown).await;
        let Some(mut worker) = self.worker.take() else {
            return Ok(());
        };
        match timeout(Duration::from_secs(60), &mut worker).await {
            Ok(joined) => {
                joined.map_err(|error| ClaudeAgentError::Connection(error.to_string()))?
            }
            Err(_) => {
                worker.abort();
                Err(ClaudeAgentError::ControlTimeout("shutdown".into()))
            }
        }
    }
}

impl Drop for SessionHandle {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take()
            && !worker.is_finished()
        {
            let _ = self.command_tx.try_send(SessionCommand::Shutdown);
            worker.abort();
        }
    }
}

enum SessionCommand {
    UserPrompt(String),
    UserMessage(Box<UserMessage>),
    Control {
        request: Value,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    Shutdown,
}

pub(crate) type Pending = BTreeMap<String, oneshot::Sender<Result<Value>>>;
pub(crate) type HookCallbacks = BTreeMap<String, HookCallback>;
pub(crate) type InboundAbortHandles = BTreeMap<String, ElicitationAbortHandle>;
const SDK_MCP_TOOL_TIMEOUT: Duration = Duration::from_secs(60);
const SUPPORTED_MCP_PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2024-11-05"];

pub(crate) struct InboundControlResult {
    pub(crate) request_id: String,
    pub(crate) message: Value,
}

async fn session_worker(
    options: ClaudeAgentOptions,
    initial_prompt: Option<String>,
    command_rx: mpsc::Receiver<SessionCommand>,
    message_tx: mpsc::Sender<Result<Message>>,
    init_tx: oneshot::Sender<Result<Value>>,
    initialize_timeout: Duration,
) -> Result<()> {
    let mut init_tx = Some(init_tx);
    let error_message_tx = message_tx.clone();
    let result = async {
        let mut transport = SubprocessTransport::spawn(&options).await?;
        let (init_response, hook_callbacks) =
            initialize_with_timeout(&mut transport, &options, &message_tx, initialize_timeout)
                .await?;
        let mirror_batcher = options.session_store.as_ref().map(|store| {
            TranscriptMirrorBatcher::new(
                store.clone(),
                transport.projects_dir().to_path_buf(),
                options.session_store_flush,
            )
        });
        if let Some(tx) = init_tx.take() {
            let _ = tx.send(Ok(init_response));
        }
        if let Some(prompt) = initial_prompt {
            transport
                .write_json_line(&user_prompt_message(&prompt))
                .await?;
        }
        run_loop(
            transport,
            command_rx,
            message_tx,
            options.callbacks.clone(),
            hook_callbacks,
            mirror_batcher,
        )
        .await
    }
    .await;

    if let Err(error) = result {
        let stream_error = ClaudeAgentError::Connection(error.to_string());
        if let Some(tx) = init_tx.take() {
            let _ = tx.send(Err(ClaudeAgentError::Connection(error.to_string())));
        } else {
            let _ = error_message_tx.send(Err(stream_error)).await;
        }
        return Err(error);
    }
    Ok(())
}

pub(crate) async fn initialize<T>(
    transport: &mut T,
    options: &ClaudeAgentOptions,
    message_tx: &mpsc::Sender<Result<Message>>,
) -> Result<(Value, HookCallbacks)>
where
    T: RuntimeTransport + Send,
{
    initialize_with_timeout(transport, options, message_tx, Duration::from_secs(60)).await
}

pub(crate) async fn initialize_with_timeout<T>(
    transport: &mut T,
    options: &ClaudeAgentOptions,
    message_tx: &mpsc::Sender<Result<Message>>,
    initialize_timeout: Duration,
) -> Result<(Value, HookCallbacks)>
where
    T: RuntimeTransport + Send,
{
    let initialize = async {
        let request_id = next_request_id();
        let mut hook_callbacks = HookCallbacks::new();
        let initialize_payload = initialize_payload_with_callbacks(options, &mut hook_callbacks);
        transport
            .write_json_line(&control_request(&request_id, initialize_payload))
            .await?;
        loop {
            let Some(raw) = transport.read_json_line().await? else {
                return Err(ClaudeAgentError::Connection(
                    "Claude Code exited before initialize response".into(),
                ));
            };
            if raw.is_null() {
                continue;
            }
            if is_control_response_for(&raw, &request_id) {
                return Ok((parse_control_response(raw)?, hook_callbacks));
            }
            let mut pending = Pending::new();
            handle_raw_message(
                transport,
                raw,
                &mut pending,
                message_tx,
                &options.callbacks,
                &hook_callbacks,
                None,
            )
            .await?;
        }
    };

    timeout(initialize_timeout, initialize)
        .await
        .map_err(|_| ClaudeAgentError::ControlTimeout("initialize".into()))?
}

async fn run_loop<T>(
    mut transport: T,
    mut command_rx: mpsc::Receiver<SessionCommand>,
    message_tx: mpsc::Sender<Result<Message>>,
    callbacks: CallbackRegistry,
    hook_callbacks: HookCallbacks,
    mut mirror_batcher: Option<TranscriptMirrorBatcher>,
) -> Result<()>
where
    T: RuntimeTransport + Send,
{
    let mut pending = Pending::new();
    let mut inbound_abort_handles = InboundAbortHandles::new();
    let (inbound_response_tx, mut inbound_response_rx) = mpsc::channel(100);
    loop {
        tokio::select! {
            raw = transport.read_json_line() => {
                match raw? {
                    Some(raw) if raw.is_null() => continue,
                    Some(raw) => {
                        match raw.get("type").and_then(Value::as_str) {
                            Some("control_request") => {
                                if should_spawn_inbound_control_request(&raw, &callbacks) {
                                    start_inbound_control_request(
                                        raw,
                                        &callbacks,
                                        &inbound_response_tx,
                                        &mut inbound_abort_handles,
                                    )
                                    .await?;
                                } else {
                                    handle_raw_message(
                                        &mut transport,
                                        raw,
                                        &mut pending,
                                        &message_tx,
                                        &callbacks,
                                        &hook_callbacks,
                                        mirror_batcher.as_mut(),
                                    )
                                    .await?
                                }
                            }
                            Some("control_cancel_request") => {
                                handle_inbound_control_cancel_request(
                                    &raw,
                                    &mut inbound_abort_handles,
                                );
                            }
                            _ => {
                                handle_raw_message(
                                    &mut transport,
                                    raw,
                                    &mut pending,
                                    &message_tx,
                                    &callbacks,
                                    &hook_callbacks,
                                    mirror_batcher.as_mut(),
                                )
                                .await?
                            }
                        }
                    }
                    None => break,
                }
            }
            inbound = inbound_response_rx.recv() => {
                match inbound {
                    Some(inbound) => {
                        inbound_abort_handles.remove(&inbound.request_id);
                        transport.write_json_line(&inbound.message).await?;
                    }
                    None => break,
                }
            }
            command = command_rx.recv() => {
                match command {
                    Some(SessionCommand::UserPrompt(prompt)) => {
                        transport.write_json_line(&user_prompt_message(&prompt)).await?;
                    }
                    Some(SessionCommand::UserMessage(message)) => {
                        transport.write_json_line(&user_message(&message)?).await?;
                    }
                    Some(SessionCommand::Control { request, response_tx }) => {
                        let request_id = next_request_id();
                        transport.write_json_line(&control_request(&request_id, request)).await?;
                        pending.insert(request_id, response_tx);
                    }
                    Some(SessionCommand::Shutdown) | None => {
                        transport.end_input().await?;
                        break;
                    }
                }
            }
        }
    }

    for (_, handle) in inbound_abort_handles {
        handle.abort();
    }

    if let Some(batcher) = mirror_batcher.as_mut() {
        report_mirror_errors(batcher.flush().await, &message_tx).await;
    }

    for (_, tx) in pending {
        let _ = tx.send(Err(ClaudeAgentError::Connection(
            "session closed before control response".into(),
        )));
    }

    transport.wait().await
}

pub(crate) async fn start_inbound_control_request(
    raw: Value,
    callbacks: &CallbackRegistry,
    response_tx: &mpsc::Sender<InboundControlResult>,
    abort_handles: &mut InboundAbortHandles,
) -> Result<()> {
    let request_id = raw
        .get("request_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_owned();
    let Some(request) = raw.get("request").cloned() else {
        send_inbound_control_response(
            response_tx,
            request_id,
            Err(ClaudeAgentError::Connection(
                "missing control request body".into(),
            )),
        )
        .await;
        return Ok(());
    };

    match request.get("subtype").and_then(Value::as_str) {
        Some("elicitation") if callbacks.on_elicitation.is_some() => {
            let callback = callbacks.on_elicitation.clone().unwrap();
            let elicitation_request: ElicitationRequest = serde_json::from_value(request)?;
            let (options, mut abort_guard, abort_handle) =
                ElicitationCallbackOptions::new_with_abort_handle();
            abort_handles.insert(request_id.clone(), abort_handle);
            spawn_inbound_control_response(response_tx.clone(), request_id, async move {
                let result = callback(elicitation_request, options).await;
                abort_guard.complete();
                serde_json::to_value(result?).map_err(ClaudeAgentError::from)
            });
        }
        Some("oauth_token_refresh") if callbacks.get_oauth_token.is_some() => {
            let callback = callbacks.get_oauth_token.clone().unwrap();
            let (options, mut abort_guard, abort_handle) =
                TokenRefreshCallbackOptions::new_with_abort_handle();
            abort_handles.insert(request_id.clone(), abort_handle);
            spawn_inbound_control_response(response_tx.clone(), request_id, async move {
                let token = callback(options).await;
                abort_guard.complete();
                Ok(json!({ "accessToken": token? }))
            });
        }
        Some("host_auth_token_refresh") if callbacks.get_host_auth_token.is_some() => {
            let callback = callbacks.get_host_auth_token.clone().unwrap();
            let (options, mut abort_guard, abort_handle) =
                TokenRefreshCallbackOptions::new_with_abort_handle();
            abort_handles.insert(request_id.clone(), abort_handle);
            spawn_inbound_control_response(response_tx.clone(), request_id, async move {
                let token = callback(options).await;
                abort_guard.complete();
                Ok(json!({ "authToken": token? }))
            });
        }
        Some("request_user_dialog") if callbacks.on_user_dialog.is_some() => {
            let callback = callbacks.on_user_dialog.clone().unwrap();
            let dialog_request: UserDialogRequest = serde_json::from_value(request)?;
            let (options, mut abort_guard, abort_handle) =
                UserDialogCallbackOptions::new_with_abort_handle();
            abort_handles.insert(request_id.clone(), abort_handle);
            spawn_inbound_control_response(response_tx.clone(), request_id, async move {
                let response = callback(dialog_request, options).await;
                abort_guard.complete();
                response
            });
        }
        _ => {}
    }

    Ok(())
}

pub(crate) fn should_spawn_inbound_control_request(
    raw: &Value,
    callbacks: &CallbackRegistry,
) -> bool {
    match raw
        .get("request")
        .and_then(|request| request.get("subtype"))
        .and_then(Value::as_str)
    {
        Some("elicitation") => callbacks.on_elicitation.is_some(),
        Some("oauth_token_refresh") => callbacks.get_oauth_token.is_some(),
        Some("host_auth_token_refresh") => callbacks.get_host_auth_token.is_some(),
        Some("request_user_dialog") => callbacks.on_user_dialog.is_some(),
        _ => false,
    }
}

pub(crate) fn handle_inbound_control_cancel_request(
    raw: &Value,
    abort_handles: &mut InboundAbortHandles,
) {
    if let Some(request_id) = raw.get("request_id").and_then(Value::as_str)
        && let Some(handle) = abort_handles.remove(request_id)
    {
        handle.abort();
    }
}

fn spawn_inbound_control_response<Fut>(
    response_tx: mpsc::Sender<InboundControlResult>,
    request_id: String,
    future: Fut,
) where
    Fut: Future<Output = Result<Value>> + Send + 'static,
{
    tokio::spawn(async move {
        let result = future.await;
        send_inbound_control_response(&response_tx, request_id, result).await;
    });
}

async fn send_inbound_control_response(
    response_tx: &mpsc::Sender<InboundControlResult>,
    request_id: String,
    result: Result<Value>,
) {
    let message = match result {
        Ok(response) => inbound_control_success_response(&request_id, response),
        Err(error) => control_error_response(&request_id, error.to_string()),
    };
    let _ = response_tx
        .send(InboundControlResult {
            request_id,
            message,
        })
        .await;
}

fn inbound_control_success_response(request_id: &str, response: Value) -> Value {
    json!({
        "type": "control_response",
        "response": {
            "subtype": "success",
            "request_id": request_id,
            "response": response,
        }
    })
}

pub(crate) async fn handle_raw_message<T>(
    transport: &mut T,
    raw: Value,
    pending: &mut Pending,
    message_tx: &mpsc::Sender<Result<Message>>,
    callbacks: &CallbackRegistry,
    hook_callbacks: &HookCallbacks,
    mut mirror_batcher: Option<&mut TranscriptMirrorBatcher>,
) -> Result<()>
where
    T: RuntimeTransport + Send,
{
    match raw.get("type").and_then(Value::as_str) {
        Some("transcript_mirror") => {
            if let Some(batcher) = mirror_batcher.as_mut() {
                report_mirror_errors(batcher.enqueue_frame(&raw).await, message_tx).await;
            }
        }
        Some("control_response") => {
            if let Some(request_id) = raw
                .get("response")
                .and_then(|r| r.get("request_id"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                && let Some(tx) = pending.remove(&request_id)
            {
                let _ = tx.send(parse_control_response(raw));
            }
        }
        Some("control_request") => {
            handle_inbound_control_request(transport, raw, callbacks, hook_callbacks).await?;
        }
        _ => {
            if raw.get("type").and_then(Value::as_str) == Some("result")
                && let Some(batcher) = mirror_batcher.as_mut()
            {
                report_mirror_errors(batcher.flush().await, message_tx).await;
            }
            if let Some(message) = parse_message(raw)? {
                let _ = message_tx.send(Ok(message)).await;
            }
        }
    }
    Ok(())
}

pub(crate) async fn report_mirror_errors(
    errors: Vec<(SessionKey, String)>,
    message_tx: &mpsc::Sender<Result<Message>>,
) {
    for (key, error) in errors {
        let data = json!({
            "type": "system",
            "subtype": "mirror_error",
            "key": key,
            "error": error,
        });
        let _ = message_tx
            .send(Ok(Message::MirrorError(MirrorErrorMessage {
                subtype: "mirror_error".into(),
                key: data.get("key").cloned(),
                error,
                data,
            })))
            .await;
    }
}

fn initialize_payload_with_callbacks(
    options: &ClaudeAgentOptions,
    hook_callbacks: &mut HookCallbacks,
) -> Value {
    let mut payload = options.initialize_payload();
    if options.callbacks.hooks.is_empty() {
        return payload;
    }

    let mut hooks = serde_json::Map::new();
    let mut next_id = 0usize;
    for (event, matchers) in &options.callbacks.hooks {
        let mut matcher_values = Vec::new();
        for matcher in matchers {
            let mut callback_ids = Vec::new();
            for callback in &matcher.hooks {
                let callback_id = format!("hook_{next_id}");
                next_id += 1;
                hook_callbacks.insert(callback_id.clone(), callback.clone());
                callback_ids.push(Value::String(callback_id));
            }
            let mut matcher_object = serde_json::Map::new();
            matcher_object.insert(
                "matcher".into(),
                matcher
                    .matcher
                    .as_ref()
                    .map(|value| Value::String(value.clone()))
                    .unwrap_or(Value::Null),
            );
            matcher_object.insert("hookCallbackIds".into(), Value::Array(callback_ids));
            if let Some(timeout) = matcher.timeout {
                matcher_object.insert("timeout".into(), serde_json::json!(timeout));
            }
            matcher_values.push(Value::Object(matcher_object));
        }
        hooks.insert(event.clone(), Value::Array(matcher_values));
    }

    if let Some(object) = payload.as_object_mut() {
        object.insert("hooks".into(), Value::Object(hooks));
    }
    payload
}

async fn handle_inbound_control_request<T>(
    transport: &mut T,
    raw: Value,
    callbacks: &CallbackRegistry,
    hook_callbacks: &HookCallbacks,
) -> Result<()>
where
    T: RuntimeTransport + Send,
{
    let request_id = raw
        .get("request_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_owned();
    let Some(request) = raw.get("request") else {
        transport
            .write_json_line(&control_error_response(
                &request_id,
                "missing control request body",
            ))
            .await?;
        return Ok(());
    };

    let response = match request.get("subtype").and_then(Value::as_str) {
        Some("can_use_tool") => {
            let Some(callback) = callbacks.can_use_tool.as_ref() else {
                transport
                    .write_json_line(&control_error_response(
                        &request_id,
                        "no can_use_tool callback registered",
                    ))
                    .await?;
                return Ok(());
            };
            let permission_request: ToolPermissionRequest =
                serde_json::from_value(request.clone())?;
            serde_json::to_value(callback(permission_request).await?)?
        }
        Some("elicitation") => {
            let elicitation_request: ElicitationRequest = serde_json::from_value(request.clone())?;
            let result = if let Some(callback) = callbacks.on_elicitation.as_ref() {
                let (options, mut abort_guard) = ElicitationCallbackOptions::new();
                let result = callback(elicitation_request, options).await;
                abort_guard.complete();
                result?
            } else {
                ElicitationResult::decline()
            };
            serde_json::to_value(result)?
        }
        Some("oauth_token_refresh") => {
            let Some(callback) = callbacks.get_oauth_token.as_ref() else {
                transport
                    .write_json_line(&control_error_response(
                        &request_id,
                        "get_oauth_token callback is not provided",
                    ))
                    .await?;
                return Ok(());
            };
            let (options, mut abort_guard) = TokenRefreshCallbackOptions::new();
            let token = callback(options).await?;
            abort_guard.complete();
            json!({ "accessToken": token })
        }
        Some("host_auth_token_refresh") => {
            let Some(callback) = callbacks.get_host_auth_token.as_ref() else {
                transport
                    .write_json_line(&control_error_response(
                        &request_id,
                        "get_host_auth_token callback is not provided",
                    ))
                    .await?;
                return Ok(());
            };
            let (options, mut abort_guard) = TokenRefreshCallbackOptions::new();
            let token = callback(options).await?;
            abort_guard.complete();
            json!({ "authToken": token })
        }
        Some("request_user_dialog") => {
            let Some(callback) = callbacks.on_user_dialog.as_ref() else {
                transport
                    .write_json_line(&control_error_response(
                        &request_id,
                        "on_user_dialog callback is not provided",
                    ))
                    .await?;
                return Ok(());
            };
            let dialog_request: UserDialogRequest = serde_json::from_value(request.clone())?;
            let (options, mut abort_guard) = UserDialogCallbackOptions::new();
            let response = callback(dialog_request, options).await?;
            abort_guard.complete();
            response
        }
        Some("hook_callback") => {
            let callback_request: HookCallbackRequest = serde_json::from_value(request.clone())?;
            let Some(callback) = hook_callbacks.get(&callback_request.callback_id) else {
                transport
                    .write_json_line(&control_error_response(
                        &request_id,
                        format!(
                            "no hook callback found for {}",
                            callback_request.callback_id
                        ),
                    ))
                    .await?;
                return Ok(());
            };
            callback(callback_request).await?
        }
        Some("mcp_message") => {
            let Some(server_name) = request
                .get("server_name")
                .or_else(|| request.get("serverName"))
                .and_then(Value::as_str)
            else {
                transport
                    .write_json_line(&control_error_response(
                        &request_id,
                        "mcp_message missing server_name",
                    ))
                    .await?;
                return Ok(());
            };
            let Some(message) = request.get("message").cloned() else {
                transport
                    .write_json_line(&control_error_response(
                        &request_id,
                        "mcp_message missing message",
                    ))
                    .await?;
                return Ok(());
            };
            let mcp_response = handle_sdk_mcp_message(callbacks, server_name, message).await?;
            json!({"mcp_response": mcp_response})
        }
        Some(other) => {
            transport
                .write_json_line(&control_error_response(
                    &request_id,
                    format!("unsupported control request subtype: {other}"),
                ))
                .await?;
            return Ok(());
        }
        None => {
            transport
                .write_json_line(&control_error_response(
                    &request_id,
                    "missing control request subtype",
                ))
                .await?;
            return Ok(());
        }
    };

    transport
        .write_json_line(&json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": request_id,
                "response": response,
            }
        }))
        .await?;
    Ok(())
}

async fn handle_sdk_mcp_message(
    callbacks: &CallbackRegistry,
    server_name: &str,
    message: Value,
) -> Result<Value> {
    let Some(server) = callbacks.sdk_mcp_servers.get(server_name) else {
        return Ok(json!({
            "jsonrpc": "2.0",
            "id": message.get("id").cloned().unwrap_or(Value::Null),
            "error": {"code": -32601, "message": format!("Server '{server_name}' not found")},
        }));
    };

    let id = message.get("id").cloned().unwrap_or(Value::Null);
    match message.get("method").and_then(Value::as_str) {
        Some("initialize") => Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": negotiated_mcp_protocol_version(&message),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": server.name, "version": server.version},
            }
        })),
        Some("notifications/initialized") => Ok(json!({"jsonrpc": "2.0", "result": {}})),
        Some("tools/list") => {
            let tools = match server
                .tools
                .iter()
                .map(|tool| {
                    let mut value = json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool.input_schema,
                    });
                    if let Some(annotations) = &tool.annotations
                        && let Some(object) = value.as_object_mut()
                    {
                        object.insert("annotations".into(), serde_json::to_value(annotations)?);
                    }
                    if !tool.meta.is_empty()
                        && let Some(object) = value.as_object_mut()
                    {
                        object.insert("_meta".into(), Value::Object(tool.meta.clone()));
                    }
                    Ok(value)
                })
                .collect::<Result<Vec<_>>>()
            {
                Ok(tools) => tools,
                Err(error) => return Ok(json_rpc_error(id, -32603, error.to_string())),
            };
            Ok(json!({"jsonrpc": "2.0", "id": id, "result": {"tools": tools}}))
        }
        Some("tools/call") => {
            let params = message.get("params").cloned().unwrap_or(Value::Null);
            let Some(name) = params.get("name").and_then(Value::as_str) else {
                return Ok(json_rpc_error(id, -32602, "tools/call missing params.name"));
            };
            let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
            let Some(tool) = server.tools.iter().find(|tool| tool.name == name) else {
                return Ok(json_rpc_error(
                    id,
                    -32602,
                    format!("Tool '{name}' not found"),
                ));
            };
            if let Err(message) = validate_tool_arguments(&arguments, &tool.input_schema) {
                return Ok(json_rpc_error(id, -32602, message));
            }
            match timeout(
                SDK_MCP_TOOL_TIMEOUT,
                (tool.handler)(arguments, crate::ToolCallExtra { raw: params }),
            )
            .await
            {
                Err(_) => Ok(json_rpc_error(id, -32603, "tools/call handler timed out")),
                Ok(Ok(result)) => Ok(json!({"jsonrpc": "2.0", "id": id, "result": result})),
                Ok(Err(ClaudeAgentError::ToolArguments(message))) => {
                    Ok(json_rpc_error(id, -32602, message))
                }
                Ok(Err(error)) => Ok(json_rpc_error(id, -32603, error.to_string())),
            }
        }
        Some(method) => Ok(json_rpc_error(
            id,
            -32601,
            format!("Method '{method}' not found"),
        )),
        None => Ok(json_rpc_error(id, -32600, "Missing JSON-RPC method")),
    }
}

fn negotiated_mcp_protocol_version(message: &Value) -> &'static str {
    let requested = message
        .get("params")
        .and_then(|params| params.get("protocolVersion"))
        .and_then(Value::as_str);
    requested
        .and_then(|version| {
            SUPPORTED_MCP_PROTOCOL_VERSIONS
                .iter()
                .copied()
                .find(|supported| *supported == version)
        })
        .unwrap_or(SUPPORTED_MCP_PROTOCOL_VERSIONS[0])
}

fn json_rpc_error(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message.into()},
    })
}

const MAX_SCHEMA_VALIDATION_DEPTH: usize = 64;

fn validate_tool_arguments(arguments: &Value, schema: &Value) -> std::result::Result<(), String> {
    validate_value_against_schema(arguments, schema, "arguments", schema, 0)
}

fn validate_value_against_schema(
    value: &Value,
    schema: &Value,
    path: &str,
    root_schema: &Value,
    depth: usize,
) -> std::result::Result<(), String> {
    if depth > MAX_SCHEMA_VALIDATION_DEPTH {
        return Err(format!("{path} exceeded maximum schema validation depth"));
    }

    match schema {
        Value::Bool(true) => Ok(()),
        Value::Bool(false) => Err(format!("{path} is disallowed by input_schema")),
        Value::Object(schema) => {
            validate_value_against_schema_object(value, schema, path, root_schema, depth)
        }
        _ => Ok(()),
    }
}

fn validate_value_against_schema_object(
    value: &Value,
    schema: &serde_json::Map<String, Value>,
    path: &str,
    root_schema: &Value,
    depth: usize,
) -> std::result::Result<(), String> {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let referenced = resolve_schema_ref(root_schema, reference)?;
        validate_value_against_schema(value, referenced, path, root_schema, depth + 1)?;
    }
    if let Some(schema_type) = schema.get("type") {
        validate_type(value, schema_type, path)?;
    }
    if let Some(constant) = schema.get("const")
        && value != constant
    {
        return Err(format!("{path} must equal the schema const value"));
    }
    if let Some(enum_values) = schema.get("enum").and_then(Value::as_array)
        && !enum_values.iter().any(|candidate| candidate == value)
    {
        return Err(format!("{path} must be one of the schema enum values"));
    }
    validate_numeric_bounds(value, schema, path)?;
    if let Some(any_of) = schema.get("anyOf").and_then(Value::as_array) {
        validate_any_of(value, any_of, path, root_schema, depth)?;
    }
    validate_array_keywords(value, schema, path, root_schema, depth)?;
    validate_object_keywords(value, schema, path, root_schema, depth)?;
    Ok(())
}

fn resolve_schema_ref<'a>(
    root_schema: &'a Value,
    reference: &str,
) -> std::result::Result<&'a Value, String> {
    if reference == "#" {
        return Ok(root_schema);
    }
    let Some(pointer) = reference.strip_prefix('#') else {
        return Err(format!(
            "schema $ref '{reference}' is not a local JSON pointer"
        ));
    };
    root_schema
        .pointer(pointer)
        .ok_or_else(|| format!("schema $ref '{reference}' could not be resolved"))
}

fn validate_any_of(
    value: &Value,
    schemas: &[Value],
    path: &str,
    root_schema: &Value,
    depth: usize,
) -> std::result::Result<(), String> {
    if schemas.iter().any(|schema| {
        validate_value_against_schema(value, schema, path, root_schema, depth + 1).is_ok()
    }) {
        Ok(())
    } else {
        Err(format!("{path} does not match any anyOf schema"))
    }
}

fn validate_array_keywords(
    value: &Value,
    schema: &serde_json::Map<String, Value>,
    path: &str,
    root_schema: &Value,
    depth: usize,
) -> std::result::Result<(), String> {
    if !schema_contains_array_keywords(schema) {
        return Ok(());
    }
    let Some(array) = value.as_array() else {
        if schema
            .get("type")
            .is_some_and(|schema_type| schema_type_matches_value(value, schema_type, path))
        {
            return Ok(());
        }
        return Err(format!("{path} must be an array"));
    };
    let Some(items) = schema.get("items") else {
        return Ok(());
    };
    for (index, item) in array.iter().enumerate() {
        validate_value_against_schema(
            item,
            items,
            &format!("{path}[{index}]"),
            root_schema,
            depth + 1,
        )?;
    }
    Ok(())
}

fn validate_object_keywords(
    value: &Value,
    schema: &serde_json::Map<String, Value>,
    path: &str,
    root_schema: &Value,
    depth: usize,
) -> std::result::Result<(), String> {
    if !schema_contains_object_keywords(schema) {
        return Ok(());
    }
    let Some(object) = value.as_object() else {
        if schema
            .get("type")
            .is_some_and(|schema_type| schema_type_matches_value(value, schema_type, path))
        {
            return Ok(());
        }
        return Err(format!("{path} must be an object"));
    };
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for field in required.iter().filter_map(Value::as_str) {
            if !object.contains_key(field) {
                return Err(format!("{path}.{field} is required"));
            }
        }
    }
    let properties = schema.get("properties").and_then(Value::as_object);
    if let Some(properties) = properties {
        for (name, property_schema) in properties {
            let Some(property_value) = object.get(name) else {
                continue;
            };
            validate_value_against_schema(
                property_value,
                property_schema,
                &format!("{path}.{name}"),
                root_schema,
                depth + 1,
            )?;
        }
    }
    if let Some(additional_properties) = schema.get("additionalProperties") {
        for (name, property_value) in object {
            if properties.is_some_and(|properties| properties.contains_key(name)) {
                continue;
            }
            match additional_properties {
                Value::Bool(false) => {
                    return Err(format!("{path}.{name} is not allowed by input_schema"));
                }
                Value::Bool(true) => {}
                schema => validate_value_against_schema(
                    property_value,
                    schema,
                    &format!("{path}.{name}"),
                    root_schema,
                    depth + 1,
                )?,
            }
        }
    }
    Ok(())
}

fn validate_numeric_bounds(
    value: &Value,
    schema: &serde_json::Map<String, Value>,
    path: &str,
) -> std::result::Result<(), String> {
    let Some(number) = value.as_f64() else {
        return Ok(());
    };
    if let Some(minimum) = schema.get("minimum").and_then(Value::as_f64)
        && number < minimum
    {
        return Err(format!("{path} must be >= {minimum}"));
    }
    if let Some(maximum) = schema.get("maximum").and_then(Value::as_f64)
        && number > maximum
    {
        return Err(format!("{path} must be <= {maximum}"));
    }
    Ok(())
}

fn schema_contains_array_keywords(schema: &serde_json::Map<String, Value>) -> bool {
    schema.get("type").and_then(Value::as_str) == Some("array") || schema.contains_key("items")
}

fn schema_contains_object_keywords(schema: &serde_json::Map<String, Value>) -> bool {
    schema.get("type").and_then(Value::as_str) == Some("object")
        || schema.contains_key("properties")
        || schema.contains_key("required")
        || schema.contains_key("additionalProperties")
}

fn schema_type_matches_value(value: &Value, schema_type: &Value, path: &str) -> bool {
    validate_type(value, schema_type, path).is_ok()
}

fn validate_type(
    value: &Value,
    schema_type: &Value,
    path: &str,
) -> std::result::Result<(), String> {
    match schema_type {
        Value::String(expected) => validate_single_type(value, expected, path),
        Value::Array(types) => {
            if types
                .iter()
                .filter_map(Value::as_str)
                .any(|expected| validate_single_type(value, expected, path).is_ok())
            {
                Ok(())
            } else {
                Err(format!("{path} does not match any allowed schema type"))
            }
        }
        _ => Ok(()),
    }
}

fn validate_single_type(
    value: &Value,
    expected: &str,
    path: &str,
) -> std::result::Result<(), String> {
    let valid = match expected {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(format!("{path} must be {expected}"))
    }
}

pub(crate) fn interrupt_request() -> Value {
    json!({"subtype": "interrupt"})
}

pub(crate) fn set_permission_mode_request(mode: PermissionMode) -> Value {
    json!({"subtype": "set_permission_mode", "mode": mode.as_cli_value()})
}

pub(crate) fn set_model_request(model: Option<&str>) -> Value {
    json!({"subtype": "set_model", "model": model})
}

pub(crate) fn rename_session_request(title: &str) -> Value {
    json!({"subtype": "rename_session", "title": title})
}

pub(crate) fn set_color_request(color: &str) -> Value {
    json!({"subtype": "set_color", "color": color})
}

pub(crate) fn set_max_thinking_tokens_request(max_thinking_tokens: Option<u32>) -> Value {
    json!({"subtype": "set_max_thinking_tokens", "max_thinking_tokens": max_thinking_tokens})
}

pub(crate) fn apply_flag_settings_request(settings: Value) -> Value {
    json!({"subtype": "apply_flag_settings", "settings": settings})
}

pub(crate) fn rewind_files_request(user_message_id: &str, dry_run: bool) -> Value {
    json!({"subtype": "rewind_files", "user_message_id": user_message_id, "dry_run": dry_run})
}

pub(crate) fn read_file_request(
    path: &str,
    max_bytes: Option<u64>,
    encoding: Option<ReadFileEncoding>,
) -> Value {
    json!({
        "subtype": "read_file",
        "path": path,
        "max_bytes": max_bytes,
        "encoding": encoding.map(ReadFileEncoding::as_protocol_value),
    })
}

pub(crate) fn seed_read_state_request(path: &str, mtime: u64) -> Value {
    json!({"subtype": "seed_read_state", "path": path, "mtime": mtime})
}

pub(crate) fn mcp_status_request() -> Value {
    json!({"subtype": "mcp_status"})
}

pub(crate) fn get_settings_request() -> Value {
    json!({"subtype": "get_settings"})
}

pub(crate) fn file_suggestions_request(query: &str) -> Value {
    json!({"subtype": "file_suggestions", "query": query})
}

pub(crate) fn get_binary_version_request() -> Value {
    json!({"subtype": "get_binary_version"})
}

pub(crate) fn get_session_cost_request() -> Value {
    json!({"subtype": "get_session_cost"})
}

pub(crate) fn reload_plugins_request() -> Value {
    json!({"subtype": "reload_plugins"})
}

pub(crate) fn cancel_async_message_request(message_uuid: &str) -> Value {
    json!({"subtype": "cancel_async_message", "message_uuid": message_uuid})
}

pub(crate) fn context_usage_request() -> Value {
    json!({"subtype": "get_context_usage"})
}

pub(crate) fn reconnect_mcp_server_request(server_name: &str) -> Value {
    json!({"subtype": "mcp_reconnect", "serverName": server_name})
}

pub(crate) fn toggle_mcp_server_request(server_name: &str, enabled: bool) -> Value {
    json!({"subtype": "mcp_toggle", "serverName": server_name, "enabled": enabled})
}

pub(crate) fn enable_channel_request(server_name: &str) -> Value {
    json!({"subtype": "channel_enable", "serverName": server_name})
}

pub(crate) fn mcp_authenticate_request(server_name: &str, redirect_uri: &str) -> Value {
    json!({"subtype": "mcp_authenticate", "serverName": server_name, "redirectUri": redirect_uri})
}

pub(crate) fn mcp_clear_auth_request(server_name: &str) -> Value {
    json!({"subtype": "mcp_clear_auth", "serverName": server_name})
}

pub(crate) fn mcp_call_request(tool: &str, arguments: Option<Value>) -> Value {
    let mut request = Map::new();
    request.insert("subtype".into(), Value::String("mcp_call".into()));
    request.insert("tool".into(), Value::String(tool.to_owned()));
    if let Some(arguments) = arguments {
        request.insert("arguments".into(), arguments);
    }
    Value::Object(request)
}

pub(crate) fn mcp_message_request(server_name: &str, message: Value) -> Value {
    json!({"subtype": "mcp_message", "server_name": server_name, "message": message})
}

pub(crate) fn mcp_oauth_callback_url_request(server_name: &str, callback_url: &str) -> Value {
    json!({"subtype": "mcp_oauth_callback_url", "serverName": server_name, "callbackUrl": callback_url})
}

pub(crate) fn claude_authenticate_request(login_with_claude_ai: bool) -> Value {
    json!({"subtype": "claude_authenticate", "loginWithClaudeAi": login_with_claude_ai})
}

pub(crate) fn claude_oauth_callback_request(authorization_code: &str, state: &str) -> Value {
    json!({"subtype": "claude_oauth_callback", "authorizationCode": authorization_code, "state": state})
}

pub(crate) fn claude_oauth_wait_for_completion_request() -> Value {
    json!({"subtype": "claude_oauth_wait_for_completion"})
}

pub(crate) fn stop_task_request(task_id: &str) -> Value {
    json!({"subtype": "stop_task", "task_id": task_id})
}

pub(crate) fn background_tasks_request(tool_use_id: Option<&str>) -> Value {
    json!({"subtype": "background_tasks", "tool_use_id": tool_use_id})
}

pub(crate) fn enable_remote_control_request(enabled: bool, name: Option<&str>) -> Value {
    let mut request = Map::new();
    request.insert("subtype".into(), Value::String("remote_control".into()));
    request.insert("enabled".into(), Value::Bool(enabled));
    if let Some(name) = name {
        request.insert("name".into(), Value::String(name.to_owned()));
    }
    Value::Object(request)
}

pub(crate) fn submit_feedback_request(description: &str, surface: Option<&str>) -> Value {
    let mut request = Map::new();
    request.insert("subtype".into(), Value::String("submit_feedback".into()));
    request.insert("description".into(), Value::String(description.to_owned()));
    if let Some(surface) = surface {
        request.insert("surface".into(), Value::String(surface.to_owned()));
    }
    Value::Object(request)
}

pub(crate) fn generate_session_title_request(description: &str, persist: Option<bool>) -> Value {
    let mut request = Map::new();
    request.insert(
        "subtype".into(),
        Value::String("generate_session_title".into()),
    );
    request.insert("description".into(), Value::String(description.to_owned()));
    if let Some(persist) = persist {
        request.insert("persist".into(), Value::Bool(persist));
    }
    Value::Object(request)
}

pub(crate) fn side_question_request(question: &str) -> Value {
    json!({"subtype": "side_question", "question": question})
}

pub(crate) fn ultrareview_launch_request(args: &[String], confirm: bool) -> Value {
    json!({"subtype": "ultrareview_launch", "args": args, "confirm": confirm})
}

pub(crate) fn message_rated_request(
    message_uuid: &str,
    sentiment: &str,
    surface: &str,
    cleared: bool,
) -> Value {
    json!({
        "subtype": "message_rated",
        "messageUuid": message_uuid,
        "sentiment": sentiment,
        "surface": surface,
        "cleared": cleared,
    })
}

pub(crate) fn set_mcp_servers_request(
    servers: &BTreeMap<String, McpServerConfig>,
) -> Result<Value> {
    Ok(json!({"subtype": "mcp_set_servers", "servers": servers}))
}

pub(crate) fn next_request_id() -> String {
    format!("req_{}", Uuid::new_v4().simple())
}

pub(crate) fn parse_initialization(value: Value) -> Result<SDKControlInitializeResponse> {
    serde_json::from_value(value).map_err(Into::into)
}

pub(crate) fn array_field(value: &Value, field: &str) -> Vec<Value> {
    value
        .get(field)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn is_control_response_for(raw: &Value, request_id: &str) -> bool {
    raw.get("type").and_then(Value::as_str) == Some("control_response")
        && raw
            .get("response")
            .and_then(|r| r.get("request_id"))
            .and_then(Value::as_str)
            == Some(request_id)
}

fn parse_control_response(raw: Value) -> Result<Value> {
    let response = raw
        .get("response")
        .ok_or_else(|| ClaudeAgentError::Control("missing control response body".into()))?;
    match response.get("subtype").and_then(Value::as_str) {
        Some("success") => Ok(response.get("response").cloned().unwrap_or(Value::Null)),
        Some("error") => Err(ClaudeAgentError::Control(
            response
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("unknown control error")
                .to_owned(),
        )),
        _ => Err(ClaudeAgentError::Control(
            "unknown control response subtype".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_argument_validator_allows_nullable_object_schema() {
        let schema = json!({
            "type": ["object", "null"],
            "properties": {"name": {"type": "string"}},
            "required": ["name"]
        });
        assert!(validate_tool_arguments(&Value::Null, &schema).is_ok());
        assert!(validate_tool_arguments(&json!({"name": "ok"}), &schema).is_ok());
        assert!(validate_tool_arguments(&json!({}), &schema).is_err());
    }

    #[test]
    fn tool_argument_validator_rejects_extra_properties_without_properties_map() {
        let schema = json!({"type": "object", "additionalProperties": false});
        assert!(validate_tool_arguments(&json!({}), &schema).is_ok());
        assert!(validate_tool_arguments(&json!({"extra": true}), &schema).is_err());
    }

    #[test]
    fn tool_argument_validator_covers_json_schema_helper_subset() {
        let schema = json!({
            "type": "object",
            "properties": {
                "tags": {"type": "array", "items": {"type": "string"}},
                "choice": {"anyOf": [{"type": "integer"}, {"type": "string"}]},
                "kind": {"const": "search"},
                "limit": {"type": "number", "minimum": 1, "maximum": 10},
                "filter": {"$ref": "#/$defs/filter"}
            },
            "required": ["tags", "choice", "kind", "limit", "filter"],
            "additionalProperties": {"type": "boolean"},
            "$defs": {
                "filter": {
                    "type": "object",
                    "properties": {"field": {"type": "string"}},
                    "required": ["field"],
                    "additionalProperties": false
                }
            }
        });

        assert!(
            validate_tool_arguments(
                &json!({
                    "tags": ["docs"],
                    "choice": "fast",
                    "kind": "search",
                    "limit": 2.5,
                    "filter": {"field": "title"},
                    "debug": true
                }),
                &schema
            )
            .is_ok()
        );

        assert_eq!(
            validate_tool_arguments(
                &json!({
                    "tags": [1],
                    "choice": "fast",
                    "kind": "search",
                    "limit": 2,
                    "filter": {"field": "title"}
                }),
                &schema
            )
            .unwrap_err(),
            "arguments.tags[0] must be string"
        );
        assert_eq!(
            validate_tool_arguments(
                &json!({
                    "tags": ["docs"],
                    "choice": false,
                    "kind": "search",
                    "limit": 2,
                    "filter": {"field": "title"}
                }),
                &schema
            )
            .unwrap_err(),
            "arguments.choice does not match any anyOf schema"
        );
        assert_eq!(
            validate_tool_arguments(
                &json!({
                    "tags": ["docs"],
                    "choice": "fast",
                    "kind": "other",
                    "limit": 2,
                    "filter": {"field": "title"}
                }),
                &schema
            )
            .unwrap_err(),
            "arguments.kind must equal the schema const value"
        );
        assert_eq!(
            validate_tool_arguments(
                &json!({
                    "tags": ["docs"],
                    "choice": "fast",
                    "kind": "search",
                    "limit": 11,
                    "filter": {"field": "title"}
                }),
                &schema
            )
            .unwrap_err(),
            "arguments.limit must be <= 10"
        );
        assert_eq!(
            validate_tool_arguments(
                &json!({
                    "tags": ["docs"],
                    "choice": "fast",
                    "kind": "search",
                    "limit": 2,
                    "filter": {}
                }),
                &schema
            )
            .unwrap_err(),
            "arguments.filter.field is required"
        );
        assert_eq!(
            validate_tool_arguments(
                &json!({
                    "tags": ["docs"],
                    "choice": "fast",
                    "kind": "search",
                    "limit": 2,
                    "filter": {"field": "title"},
                    "debug": "yes"
                }),
                &schema
            )
            .unwrap_err(),
            "arguments.debug must be boolean"
        );
    }
}
