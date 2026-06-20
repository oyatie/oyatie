use std::{
    collections::BTreeMap,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
    time::timeout,
};

use crate::{
    error::{ClaudeAgentError, Result},
    messages::{Message, UserMessage},
    options::{ClaudeAgentOptions, McpServerConfig, PermissionMode},
    runtime::{
        InboundAbortHandles, Pending, SessionHandle, apply_flag_settings_request, array_field,
        background_tasks_request, cancel_async_message_request, claude_authenticate_request,
        claude_oauth_callback_request, claude_oauth_wait_for_completion_request,
        context_usage_request, enable_channel_request, enable_remote_control_request,
        file_suggestions_request, generate_session_title_request, get_binary_version_request,
        get_session_cost_request, get_settings_request, handle_inbound_control_cancel_request,
        handle_raw_message, initialize, interrupt_request, mcp_authenticate_request,
        mcp_call_request, mcp_clear_auth_request, mcp_message_request,
        mcp_oauth_callback_url_request, mcp_status_request, message_rated_request, next_request_id,
        parse_initialization, read_file_request, reconnect_mcp_server_request,
        reload_plugins_request, rename_session_request, report_mirror_errors, rewind_files_request,
        seed_read_state_request, set_color_request, set_max_thinking_tokens_request,
        set_mcp_servers_request, set_model_request, set_permission_mode_request,
        should_spawn_inbound_control_request, side_question_request, start_inbound_control_request,
        stop_task_request, submit_feedback_request, toggle_mcp_server_request,
        ultrareview_launch_request,
    },
    session_store::TranscriptMirrorBatcher,
    status::{
        AccountInfo, AgentInfo, ContextUsageResponse, McpServerStatus, McpSetServersResult,
        McpStatusResponse, ModelInfo, ReadFileEncoding, RewindFilesResult,
        SDKControlInitializeResponse, SDKControlReadFileResponse, SDKControlReloadPluginsResponse,
        SideQuestionResponse, SlashCommand,
    },
    transport::{RuntimeTransport, SubprocessTransport, user_message, user_prompt_message},
};

/// Stream returned by [`query`].
pub struct Query {
    receiver: mpsc::Receiver<Result<Message>>,
    command_tx: mpsc::Sender<QueryCommand>,
    initialization_rx: watch::Receiver<Option<Value>>,
    worker: JoinHandle<Result<()>>,
}

impl Query {
    pub async fn initialization_result(&self) -> Result<Value> {
        self.wait_initialization().await
    }

    pub async fn initialization_result_typed(&self) -> Result<SDKControlInitializeResponse> {
        parse_initialization(self.wait_initialization().await?)
    }

    pub async fn initialization_result_raw(&self) -> Result<Value> {
        self.initialization_result().await
    }

    pub async fn supported_commands(&self) -> Result<Vec<Value>> {
        Ok(array_field(&self.wait_initialization().await?, "commands"))
    }

    pub async fn supported_commands_typed(&self) -> Result<Vec<SlashCommand>> {
        Ok(self.initialization_result_typed().await?.commands)
    }

    pub async fn supported_models(&self) -> Result<Vec<Value>> {
        Ok(array_field(&self.wait_initialization().await?, "models"))
    }

    pub async fn supported_models_typed(&self) -> Result<Vec<ModelInfo>> {
        Ok(self.initialization_result_typed().await?.models)
    }

    pub async fn supported_agents(&self) -> Result<Vec<Value>> {
        Ok(array_field(&self.wait_initialization().await?, "agents"))
    }

    pub async fn supported_agents_typed(&self) -> Result<Vec<AgentInfo>> {
        Ok(self.initialization_result_typed().await?.agents)
    }

    pub async fn account_info(&self) -> Result<Option<Value>> {
        Ok(self.wait_initialization().await?.get("account").cloned())
    }

    pub async fn account_info_typed(&self) -> Result<AccountInfo> {
        Ok(self.initialization_result_typed().await?.account)
    }

    pub async fn interrupt(&self) -> Result<()> {
        self.control_unit(interrupt_request()).await
    }

    pub async fn set_permission_mode(&self, mode: PermissionMode) -> Result<()> {
        self.control_unit(set_permission_mode_request(mode)).await
    }

    pub async fn set_model(&self, model: Option<&str>) -> Result<()> {
        self.control_unit(set_model_request(model)).await
    }

    pub async fn rename_session(&self, title: &str) -> Result<()> {
        self.control_unit(rename_session_request(title)).await
    }

    pub async fn set_color(&self, color: &str) -> Result<()> {
        self.control_unit(set_color_request(color)).await
    }

    pub async fn set_max_thinking_tokens(&self, max_thinking_tokens: Option<u32>) -> Result<()> {
        self.control_unit(set_max_thinking_tokens_request(max_thinking_tokens))
            .await
    }

    pub async fn apply_flag_settings(&self, settings: Value) -> Result<()> {
        self.control_unit(apply_flag_settings_request(settings))
            .await
    }

    pub async fn get_settings(&self) -> Result<Value> {
        self.control(get_settings_request()).await
    }

    pub async fn file_suggestions(&self, query: &str) -> Result<Value> {
        self.control(file_suggestions_request(query)).await
    }

    pub async fn get_binary_version(&self) -> Result<Value> {
        self.control(get_binary_version_request()).await
    }

    pub async fn get_session_cost(&self) -> Result<Value> {
        self.control(get_session_cost_request()).await
    }

    pub async fn rewind_files(&self, user_message_id: &str, dry_run: bool) -> Result<Value> {
        self.control(rewind_files_request(user_message_id, dry_run))
            .await
    }

    pub async fn rewind_files_typed(
        &self,
        user_message_id: &str,
        dry_run: bool,
    ) -> Result<RewindFilesResult> {
        self.control_typed(rewind_files_request(user_message_id, dry_run))
            .await
    }

    pub async fn rewind_files_raw(&self, user_message_id: &str, dry_run: bool) -> Result<Value> {
        self.rewind_files(user_message_id, dry_run).await
    }

    pub async fn get_mcp_status(&self) -> Result<Value> {
        self.control(mcp_status_request()).await
    }

    pub async fn get_mcp_status_typed(&self) -> Result<McpStatusResponse> {
        self.control_typed(mcp_status_request()).await
    }

    pub async fn mcp_server_status(&self) -> Result<Vec<McpServerStatus>> {
        Ok(self.get_mcp_status_typed().await?.mcp_servers)
    }

    pub async fn get_context_usage(&self) -> Result<Value> {
        self.control(context_usage_request()).await
    }

    pub async fn get_context_usage_typed(&self) -> Result<ContextUsageResponse> {
        self.control_typed(context_usage_request()).await
    }

    pub async fn read_file(
        &self,
        path: &str,
        max_bytes: Option<u64>,
        encoding: Option<ReadFileEncoding>,
    ) -> Result<Option<SDKControlReadFileResponse>> {
        match self
            .control(read_file_request(path, max_bytes, encoding))
            .await
        {
            Ok(response) => serde_json::from_value(response)
                .map(Some)
                .map_err(Into::into),
            Err(ClaudeAgentError::Control(_))
            | Err(ClaudeAgentError::Connection(_))
            | Err(ClaudeAgentError::Process { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub async fn reload_plugins(&self) -> Result<Value> {
        self.control(reload_plugins_request()).await
    }

    pub async fn reload_plugins_typed(&self) -> Result<SDKControlReloadPluginsResponse> {
        self.control_typed(reload_plugins_request()).await
    }

    pub async fn seed_read_state(&self, path: &str, mtime: u64) -> Result<()> {
        self.control_unit(seed_read_state_request(path, mtime))
            .await
    }

    pub async fn reconnect_mcp_server(&self, server_name: &str) -> Result<()> {
        self.control_unit(reconnect_mcp_server_request(server_name))
            .await
    }

    pub async fn toggle_mcp_server(&self, server_name: &str, enabled: bool) -> Result<()> {
        self.control_unit(toggle_mcp_server_request(server_name, enabled))
            .await
    }

    pub async fn enable_channel(&self, server_name: &str) -> Result<()> {
        self.control_unit(enable_channel_request(server_name)).await
    }

    pub async fn mcp_authenticate(&self, server_name: &str, redirect_uri: &str) -> Result<Value> {
        self.control(mcp_authenticate_request(server_name, redirect_uri))
            .await
    }

    pub async fn mcp_clear_auth(&self, server_name: &str) -> Result<Value> {
        self.control(mcp_clear_auth_request(server_name)).await
    }

    pub async fn mcp_call(&self, tool: &str, arguments: Option<Value>) -> Result<Value> {
        self.control(mcp_call_request(tool, arguments)).await
    }

    pub async fn mcp_message(&self, server_name: &str, message: Value) -> Result<Value> {
        self.control(mcp_message_request(server_name, message))
            .await
    }

    pub async fn mcp_submit_oauth_callback_url(
        &self,
        server_name: &str,
        callback_url: &str,
    ) -> Result<Value> {
        self.control(mcp_oauth_callback_url_request(server_name, callback_url))
            .await
    }

    pub async fn claude_authenticate(&self, login_with_claude_ai: bool) -> Result<Value> {
        self.control(claude_authenticate_request(login_with_claude_ai))
            .await
    }

    pub async fn claude_oauth_callback(
        &self,
        authorization_code: &str,
        state: &str,
    ) -> Result<Value> {
        self.control(claude_oauth_callback_request(authorization_code, state))
            .await
    }

    pub async fn claude_oauth_wait_for_completion(&self) -> Result<Value> {
        self.control(claude_oauth_wait_for_completion_request())
            .await
    }

    pub async fn set_mcp_servers(
        &self,
        servers: &BTreeMap<String, McpServerConfig>,
    ) -> Result<Value> {
        self.control(set_mcp_servers_request(servers)?).await
    }

    pub async fn set_mcp_servers_typed(
        &self,
        servers: &BTreeMap<String, McpServerConfig>,
    ) -> Result<McpSetServersResult> {
        serde_json::from_value(self.set_mcp_servers(servers).await?).map_err(Into::into)
    }

    pub async fn set_mcp_servers_raw(
        &self,
        servers: &BTreeMap<String, McpServerConfig>,
    ) -> Result<Value> {
        self.set_mcp_servers(servers).await
    }

    pub async fn stop_task(&self, task_id: &str) -> Result<()> {
        self.control_unit(stop_task_request(task_id)).await
    }

    pub async fn background_tasks(&self, tool_use_id: Option<&str>) -> Result<bool> {
        let response = self.control(background_tasks_request(tool_use_id)).await?;
        Ok(response
            .get("backgrounded")
            .and_then(Value::as_bool)
            .unwrap_or(true))
    }

    pub async fn cancel_async_message(&self, message_uuid: &str) -> Result<bool> {
        let response = self
            .control(cancel_async_message_request(message_uuid))
            .await?;
        Ok(response
            .get("cancelled")
            .and_then(Value::as_bool)
            .unwrap_or(false))
    }

    pub async fn enable_remote_control(&self, enabled: bool, name: Option<&str>) -> Result<Value> {
        self.control(enable_remote_control_request(enabled, name))
            .await
    }

    pub async fn submit_feedback(&self, description: &str, surface: Option<&str>) -> Result<Value> {
        self.control(submit_feedback_request(description, surface))
            .await
    }

    pub async fn generate_session_title(
        &self,
        description: &str,
        persist: Option<bool>,
    ) -> Result<String> {
        let response = self
            .control(generate_session_title_request(description, persist))
            .await?;
        response
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| ClaudeAgentError::Control("missing generated session title".into()))
    }

    pub async fn ask_side_question(&self, question: &str) -> Result<Option<SideQuestionResponse>> {
        let response = self.control(side_question_request(question)).await?;
        if response.get("response").is_some_and(Value::is_null) {
            return Ok(None);
        }
        serde_json::from_value(response)
            .map(Some)
            .map_err(Into::into)
    }

    pub async fn launch_ultrareview(&self, args: &[String], confirm: bool) -> Result<Value> {
        self.control(ultrareview_launch_request(args, confirm))
            .await
    }

    pub async fn message_rated(
        &self,
        message_uuid: &str,
        sentiment: &str,
        surface: &str,
        cleared: bool,
    ) -> Result<()> {
        self.control_unit(message_rated_request(
            message_uuid,
            sentiment,
            surface,
            cleared,
        ))
        .await
    }

    pub async fn stream_input<S>(&self, messages: S) -> Result<()>
    where
        S: Stream<Item = UserMessage>,
    {
        futures::pin_mut!(messages);
        while let Some(message) = messages.next().await {
            self.send_user_message(message).await?;
        }
        Ok(())
    }

    pub fn close(&mut self) {
        if !self.worker.is_finished() {
            self.worker.abort();
        }
    }

    async fn send_user_message(&self, message: UserMessage) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();
        self.command_tx
            .send(QueryCommand::UserMessage {
                message: Box::new(message),
                response_tx,
            })
            .await
            .map_err(|_| ClaudeAgentError::Connection("query worker is closed".into()))?;
        timeout(Duration::from_secs(60), response_rx)
            .await
            .map_err(|_| ClaudeAgentError::ControlTimeout("stream_input".into()))?
            .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?
    }

    async fn wait_initialization(&self) -> Result<Value> {
        let mut initialization_rx = self.initialization_rx.clone();
        if let Some(initialization) = initialization_rx.borrow().clone() {
            return Ok(initialization);
        }
        timeout(Duration::from_secs(60), async {
            loop {
                initialization_rx.changed().await.map_err(|_| {
                    ClaudeAgentError::Connection("query initialization is unavailable".into())
                })?;
                if let Some(initialization) = initialization_rx.borrow().clone() {
                    return Ok(initialization);
                }
            }
        })
        .await
        .map_err(|_| ClaudeAgentError::ControlTimeout("initialize".into()))?
    }

    async fn control_unit(&self, request: Value) -> Result<()> {
        self.control(request).await.map(|_| ())
    }

    async fn control_typed<T>(&self, request: Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(self.control(request).await?).map_err(Into::into)
    }

    async fn control(&self, request: Value) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        let subtype = request
            .get("subtype")
            .and_then(Value::as_str)
            .unwrap_or("control")
            .to_owned();
        self.command_tx
            .send(QueryCommand::Control {
                request,
                response_tx,
            })
            .await
            .map_err(|_| ClaudeAgentError::Connection("query worker is closed".into()))?;
        timeout(Duration::from_secs(60), response_rx)
            .await
            .map_err(|_| ClaudeAgentError::ControlTimeout(subtype))?
            .map_err(|error| ClaudeAgentError::Connection(error.to_string()))?
    }
}

enum QueryCommand {
    UserMessage {
        message: Box<UserMessage>,
        response_tx: oneshot::Sender<Result<()>>,
    },
    Control {
        request: Value,
        response_tx: oneshot::Sender<Result<Value>>,
    },
}

impl Stream for Query {
    type Item = Result<Message>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        if !self.worker.is_finished() {
            self.worker.abort();
        }
    }
}

/// Start a one-shot Claude Agent SDK query.
///
/// This first Rust slice follows the upstream SDK subprocess protocol: spawn `claude`
/// in `--output-format stream-json --input-format stream-json`, send an initialize
/// control request, write a user message, then stream typed SDK messages from stdout.
pub fn query(prompt: impl Into<String>, options: ClaudeAgentOptions) -> Result<Query> {
    let prompt = prompt.into();
    let (sender, receiver) = mpsc::channel(100);
    let (command_tx, command_rx) = mpsc::channel(100);
    let (initialization_tx, initialization_rx) = watch::channel(None);
    let worker = tokio::spawn(async move {
        let result = run_query(
            prompt,
            options,
            sender.clone(),
            command_rx,
            initialization_tx,
        )
        .await;
        if let Err(error) = &result {
            let _ = sender.send(Err(error_for_channel(error))).await;
        }
        result
    });
    Ok(Query {
        receiver,
        command_tx,
        initialization_rx,
        worker,
    })
}

/// Start a one-shot Claude Agent SDK query from streaming user messages.
pub fn query_stream<S>(prompt: S, options: ClaudeAgentOptions) -> Result<Query>
where
    S: Stream<Item = UserMessage> + Send + 'static,
{
    let prompt = Box::pin(prompt);
    let (sender, receiver) = mpsc::channel(100);
    let (command_tx, command_rx) = mpsc::channel(100);
    let (initialization_tx, initialization_rx) = watch::channel(None);
    let worker = tokio::spawn(async move {
        let result = run_query_stream(
            prompt,
            options,
            sender.clone(),
            command_rx,
            initialization_tx,
        )
        .await;
        if let Err(error) = &result {
            let _ = sender.send(Err(error_for_channel(error))).await;
        }
        result
    });
    Ok(Query {
        receiver,
        command_tx,
        initialization_rx,
        worker,
    })
}

#[cfg(feature = "network")]
pub(crate) fn query_with_transport<T>(
    prompt: impl Into<String>,
    options: ClaudeAgentOptions,
    transport: T,
) -> Query
where
    T: RuntimeTransport + Send + 'static,
{
    let prompt = prompt.into();
    let (sender, receiver) = mpsc::channel(100);
    let (command_tx, command_rx) = mpsc::channel(100);
    let (initialization_tx, initialization_rx) = watch::channel(None);
    let worker = tokio::spawn(async move {
        let result = run_query_with_transport(
            prompt,
            options,
            sender.clone(),
            command_rx,
            initialization_tx,
            transport,
        )
        .await;
        if let Err(error) = &result {
            let _ = sender.send(Err(error_for_channel(error))).await;
        }
        result
    });
    Query {
        receiver,
        command_tx,
        initialization_rx,
        worker,
    }
}

#[cfg(feature = "network")]
pub(crate) fn query_stream_with_transport<S, T>(
    prompt: S,
    options: ClaudeAgentOptions,
    transport: T,
) -> Query
where
    S: Stream<Item = UserMessage> + Send + 'static,
    T: RuntimeTransport + Send + 'static,
{
    let prompt = Box::pin(prompt);
    let (sender, receiver) = mpsc::channel(100);
    let (command_tx, command_rx) = mpsc::channel(100);
    let (initialization_tx, initialization_rx) = watch::channel(None);
    let worker = tokio::spawn(async move {
        let result = run_query_stream_with_transport(
            prompt,
            options,
            sender.clone(),
            command_rx,
            initialization_tx,
            transport,
        )
        .await;
        if let Err(error) = &result {
            let _ = sender.send(Err(error_for_channel(error))).await;
        }
        result
    });
    Query {
        receiver,
        command_tx,
        initialization_rx,
        worker,
    }
}

/// Pre-warmed one-shot query handle returned by [`startup`].
///
/// The Claude Code subprocess has already been spawned and completed the SDK
/// initialize handshake. Calling [`WarmQuery::query`] writes the first prompt to
/// that ready process and returns a [`Query`] stream. A warm handle can only be
/// used once; call [`WarmQuery::close`] to discard it without sending a prompt.
pub struct WarmQuery {
    session: Option<SessionHandle>,
}

/// Spawn Claude Code and complete the SDK initialize handshake before a prompt is available.
///
/// This mirrors the TypeScript SDK's `startup()` helper with Rust async
/// conventions: await this function during application boot, then call
/// [`WarmQuery::query`] once the prompt is ready.
pub async fn startup(options: ClaudeAgentOptions) -> Result<WarmQuery> {
    startup_with_timeout(options, Duration::from_secs(60)).await
}

/// Variant of [`startup`] with a caller-supplied initialize timeout.
pub async fn startup_with_timeout(
    options: ClaudeAgentOptions,
    initialize_timeout: Duration,
) -> Result<WarmQuery> {
    let session =
        SessionHandle::connect_with_initialize_timeout(options, None, initialize_timeout).await?;
    Ok(WarmQuery {
        session: Some(session),
    })
}

impl WarmQuery {
    /// Send the one allowed prompt to the pre-warmed subprocess.
    pub fn query(&mut self, prompt: impl Into<String>) -> Result<Query> {
        let Some(session) = self.session.take() else {
            return Err(ClaudeAgentError::InvalidOption(
                "WarmQuery.query() can only be called once".into(),
            ));
        };
        Ok(query_warm_session(prompt.into(), session))
    }

    /// Send the one allowed streaming prompt to the pre-warmed subprocess.
    pub fn query_stream<S>(&mut self, prompt: S) -> Result<Query>
    where
        S: Stream<Item = UserMessage> + Send + 'static,
    {
        let Some(session) = self.session.take() else {
            return Err(ClaudeAgentError::InvalidOption(
                "WarmQuery.query() can only be called once".into(),
            ));
        };
        Ok(query_warm_session_stream(Box::pin(prompt), session))
    }

    /// Close the pre-warmed subprocess without sending a prompt.
    pub async fn close(&mut self) -> Result<()> {
        if let Some(session) = self.session.take() {
            session.shutdown().await?;
        }
        Ok(())
    }
}

fn query_warm_session(prompt: String, mut session: SessionHandle) -> Query {
    let (sender, receiver) = mpsc::channel(100);
    let (command_tx, command_rx) = mpsc::channel(100);
    let (_, initialization_rx) = watch::channel(Some(session.initialization().clone()));
    let worker = tokio::spawn(async move {
        let result = async {
            session.query(prompt).await?;
            receive_warm_response(&mut session, &sender, command_rx).await?;
            session.shutdown().await
        }
        .await;
        if let Err(error) = &result {
            let _ = sender.send(Err(error_for_channel(error))).await;
        }
        result
    });
    Query {
        receiver,
        command_tx,
        initialization_rx,
        worker,
    }
}

fn query_warm_session_stream<S>(mut prompt: Pin<Box<S>>, mut session: SessionHandle) -> Query
where
    S: Stream<Item = UserMessage> + Send + 'static,
{
    let (sender, receiver) = mpsc::channel(100);
    let (command_tx, command_rx) = mpsc::channel(100);
    let (_, initialization_rx) = watch::channel(Some(session.initialization().clone()));
    let worker = tokio::spawn(async move {
        let result = async {
            while let Some(message) = prompt.as_mut().next().await {
                session.query_message(message).await?;
            }
            receive_warm_response(&mut session, &sender, command_rx).await?;
            session.shutdown().await
        }
        .await;
        if let Err(error) = &result {
            let _ = sender.send(Err(error_for_channel(error))).await;
        }
        result
    });
    Query {
        receiver,
        command_tx,
        initialization_rx,
        worker,
    }
}

async fn receive_warm_response(
    session: &mut SessionHandle,
    sender: &mpsc::Sender<Result<Message>>,
    mut command_rx: mpsc::Receiver<QueryCommand>,
) -> Result<()> {
    loop {
        tokio::select! {
            message = session.receive_next() => {
                let Some(message) = message else {
                    break;
                };
                let message = message?;
                let is_result = matches!(message, Message::Result(_));
                if sender.send(Ok(message)).await.is_err() {
                    break;
                }
                if is_result {
                    break;
                }
            }
            command = command_rx.recv() => {
                match command {
                    Some(QueryCommand::UserMessage { message, response_tx }) => {
                        let result = session.query_message(*message).await;
                        let _ = response_tx.send(result);
                    }
                    Some(QueryCommand::Control { request, response_tx }) => {
                        let result = session.control(request).await;
                        let _ = response_tx.send(result);
                    }
                    None => {}
                }
            }
        }
    }
    Ok(())
}

async fn handle_query_command<T>(
    transport: &mut T,
    pending: &mut Pending,
    input_closed: bool,
    command: QueryCommand,
) -> Result<()>
where
    T: RuntimeTransport + Send,
{
    match command {
        QueryCommand::UserMessage {
            message,
            response_tx,
        } => {
            if input_closed {
                let _ = response_tx.send(Err(query_input_closed_error()));
                return Ok(());
            }
            let wire_message = match user_message(&message) {
                Ok(message) => message,
                Err(error) => {
                    let _ = response_tx.send(Err(error_for_channel(&error)));
                    return Err(error);
                }
            };
            match transport.write_json_line(&wire_message).await {
                Ok(()) => {
                    let _ = response_tx.send(Ok(()));
                    Ok(())
                }
                Err(error) => {
                    let _ = response_tx.send(Err(error_for_channel(&error)));
                    Err(error)
                }
            }
        }
        QueryCommand::Control {
            request,
            response_tx,
        } => {
            if input_closed {
                let _ = response_tx.send(Err(query_input_closed_error()));
                return Ok(());
            }
            let request_id = next_request_id();
            match transport
                .write_json_line(&crate::transport::control_request(&request_id, request))
                .await
            {
                Ok(()) => {
                    pending.insert(request_id, response_tx);
                    Ok(())
                }
                Err(error) => {
                    let _ = response_tx.send(Err(error_for_channel(&error)));
                    Err(error)
                }
            }
        }
    }
}

async fn close_query_input<T>(transport: &mut T, input_closed: &mut bool) -> Result<()>
where
    T: RuntimeTransport + Send,
{
    if !*input_closed {
        transport.end_input().await?;
        *input_closed = true;
    }
    Ok(())
}

fn fail_pending_controls(pending: Pending) {
    for (_, tx) in pending {
        let _ = tx.send(Err(ClaudeAgentError::Connection(
            "query closed before control response".into(),
        )));
    }
}

fn query_input_closed_error() -> ClaudeAgentError {
    ClaudeAgentError::InvalidOption(
        "Query control requests require streaming input while stdin is still open".into(),
    )
}

async fn run_query(
    prompt: String,
    options: ClaudeAgentOptions,
    sender: mpsc::Sender<Result<Message>>,
    command_rx: mpsc::Receiver<QueryCommand>,
    initialization_tx: watch::Sender<Option<Value>>,
) -> Result<()> {
    let transport = SubprocessTransport::spawn(&options).await?;
    run_query_with_transport(
        prompt,
        options,
        sender,
        command_rx,
        initialization_tx,
        transport,
    )
    .await
}

async fn run_query_with_transport<T>(
    prompt: String,
    options: ClaudeAgentOptions,
    sender: mpsc::Sender<Result<Message>>,
    mut command_rx: mpsc::Receiver<QueryCommand>,
    initialization_tx: watch::Sender<Option<Value>>,
    mut transport: T,
) -> Result<()>
where
    T: RuntimeTransport + Send,
{
    let (initialization, hook_callbacks) = initialize(&mut transport, &options, &sender).await?;
    publish_query_initialization(&initialization_tx, &initialization);

    transport
        .write_json_line(&user_prompt_message(&prompt))
        .await?;
    let mut keep_input_open_for_callbacks = options.callbacks.can_use_tool.is_some()
        || options.callbacks.on_elicitation.is_some()
        || options.callbacks.get_oauth_token.is_some()
        || options.callbacks.get_host_auth_token.is_some()
        || options.callbacks.on_user_dialog.is_some()
        || !options.callbacks.sdk_mcp_servers.is_empty()
        || !hook_callbacks.is_empty();
    let mut input_closed = false;
    if !keep_input_open_for_callbacks {
        close_query_input(&mut transport, &mut input_closed).await?;
    }

    let mut mirror_batcher = options.session_store.as_ref().map(|store| {
        TranscriptMirrorBatcher::new(
            store.clone(),
            transport.projects_dir().to_path_buf(),
            options.session_store_flush,
        )
    });
    let mut pending: Pending = BTreeMap::new();
    let mut inbound_abort_handles = InboundAbortHandles::new();
    let (inbound_response_tx, mut inbound_response_rx) = mpsc::channel(100);
    let mut stdout_done = false;
    loop {
        tokio::select! {
            raw = transport.read_json_line(), if !stdout_done => {
                match raw? {
                    Some(raw) if raw.is_null() => continue,
                    Some(raw) => {
                        let is_result = raw.get("type").and_then(serde_json::Value::as_str) == Some("result");
                        match raw.get("type").and_then(serde_json::Value::as_str) {
                            Some("control_request")
                                if should_spawn_inbound_control_request(&raw, &options.callbacks) =>
                            {
                                start_inbound_control_request(
                                    raw,
                                    &options.callbacks,
                                    &inbound_response_tx,
                                    &mut inbound_abort_handles,
                                )
                                .await?;
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
                                    &sender,
                                    &options.callbacks,
                                    &hook_callbacks,
                                    mirror_batcher.as_mut(),
                                )
                                .await?;
                            }
                        }
                        if is_result && keep_input_open_for_callbacks {
                            close_query_input(&mut transport, &mut input_closed).await?;
                            keep_input_open_for_callbacks = false;
                        }
                    }
                    None => stdout_done = true,
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
                if let Some(command) = command {
                    handle_query_command(
                        &mut transport,
                        &mut pending,
                        input_closed,
                        command,
                    )
                    .await?;
                }
            }
            else => break,
        }
        if stdout_done {
            break;
        }
    }

    for (_, handle) in inbound_abort_handles {
        handle.abort();
    }

    if let Some(batcher) = mirror_batcher.as_mut() {
        report_mirror_errors(batcher.flush().await, &sender).await;
    }

    fail_pending_controls(pending);

    transport.wait().await
}

async fn run_query_stream<S>(
    prompt: Pin<Box<S>>,
    options: ClaudeAgentOptions,
    sender: mpsc::Sender<Result<Message>>,
    command_rx: mpsc::Receiver<QueryCommand>,
    initialization_tx: watch::Sender<Option<Value>>,
) -> Result<()>
where
    S: Stream<Item = UserMessage> + Send + 'static,
{
    let transport = SubprocessTransport::spawn(&options).await?;
    run_query_stream_with_transport(
        prompt,
        options,
        sender,
        command_rx,
        initialization_tx,
        transport,
    )
    .await
}

async fn run_query_stream_with_transport<S, T>(
    mut prompt: Pin<Box<S>>,
    options: ClaudeAgentOptions,
    sender: mpsc::Sender<Result<Message>>,
    command_rx: mpsc::Receiver<QueryCommand>,
    initialization_tx: watch::Sender<Option<Value>>,
    mut transport: T,
) -> Result<()>
where
    S: Stream<Item = UserMessage> + Send + 'static,
    T: RuntimeTransport + Send,
{
    let (initialization, hook_callbacks) = initialize(&mut transport, &options, &sender).await?;
    publish_query_initialization(&initialization_tx, &initialization);
    let keep_input_open_for_callbacks = options.callbacks.can_use_tool.is_some()
        || options.callbacks.on_elicitation.is_some()
        || options.callbacks.get_oauth_token.is_some()
        || options.callbacks.get_host_auth_token.is_some()
        || options.callbacks.on_user_dialog.is_some()
        || !options.callbacks.sdk_mcp_servers.is_empty()
        || !hook_callbacks.is_empty();

    let mut mirror_batcher = options.session_store.as_ref().map(|store| {
        TranscriptMirrorBatcher::new(
            store.clone(),
            transport.projects_dir().to_path_buf(),
            options.session_store_flush,
        )
    });
    let context = StreamingQueryLoopContext {
        sender: &sender,
        options: &options,
        hook_callbacks: &hook_callbacks,
        mirror_batcher: mirror_batcher.as_mut(),
        command_rx,
    };
    run_streaming_query_loop(
        &mut transport,
        &mut prompt,
        keep_input_open_for_callbacks,
        context,
    )
    .await?;

    if let Some(batcher) = mirror_batcher.as_mut() {
        report_mirror_errors(batcher.flush().await, &sender).await;
    }

    transport.wait().await
}

struct StreamingQueryLoopContext<'a> {
    sender: &'a mpsc::Sender<Result<Message>>,
    options: &'a ClaudeAgentOptions,
    hook_callbacks: &'a BTreeMap<String, crate::callbacks::HookCallback>,
    mirror_batcher: Option<&'a mut TranscriptMirrorBatcher>,
    command_rx: mpsc::Receiver<QueryCommand>,
}

async fn run_streaming_query_loop<S, T>(
    transport: &mut T,
    prompt: &mut Pin<Box<S>>,
    mut keep_input_open_for_callbacks: bool,
    mut context: StreamingQueryLoopContext<'_>,
) -> Result<()>
where
    S: Stream<Item = UserMessage> + Send + 'static,
    T: RuntimeTransport + Send,
{
    let mut pending: Pending = BTreeMap::new();
    let mut inbound_abort_handles = InboundAbortHandles::new();
    let (inbound_response_tx, mut inbound_response_rx) = mpsc::channel(100);
    let mut input_done = false;
    let mut input_closed = false;
    let mut stdout_done = false;
    loop {
        tokio::select! {
            next = stream_next_if(prompt.as_mut(), !input_done) => {
                match next {
                    Some(message) => transport.write_json_line(&user_message(&message)?).await?,
                    None => {
                        input_done = true;
                        if !keep_input_open_for_callbacks {
                            close_query_input(transport, &mut input_closed).await?;
                        }
                    }
                }
            }
            raw = transport.read_json_line(), if !stdout_done => {
                match raw? {
                    Some(raw) if raw.is_null() => continue,
                    Some(raw) => {
                        let is_result = raw.get("type").and_then(serde_json::Value::as_str) == Some("result");
                        match raw.get("type").and_then(serde_json::Value::as_str) {
                            Some("control_request")
                                if should_spawn_inbound_control_request(
                                    &raw,
                                    &context.options.callbacks,
                                ) =>
                            {
                                start_inbound_control_request(
                                    raw,
                                    &context.options.callbacks,
                                    &inbound_response_tx,
                                    &mut inbound_abort_handles,
                                )
                                .await?;
                            }
                            Some("control_cancel_request") => {
                                handle_inbound_control_cancel_request(
                                    &raw,
                                    &mut inbound_abort_handles,
                                );
                            }
                            _ => {
                                handle_raw_message(
                                    transport,
                                    raw,
                                    &mut pending,
                                    context.sender,
                                    &context.options.callbacks,
                                    context.hook_callbacks,
                                    context.mirror_batcher.as_deref_mut(),
                                )
                                .await?;
                            }
                        }
                        if is_result && keep_input_open_for_callbacks {
                            close_query_input(transport, &mut input_closed).await?;
                            keep_input_open_for_callbacks = false;
                        }
                    }
                    None => stdout_done = true,
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
            command = context.command_rx.recv() => {
                if let Some(command) = command {
                    handle_query_command(
                        transport,
                        &mut pending,
                        input_closed,
                        command,
                    )
                    .await?;
                }
            }
            else => break,
        }
        if stdout_done {
            break;
        }
    }
    for (_, handle) in inbound_abort_handles {
        handle.abort();
    }
    fail_pending_controls(pending);
    Ok(())
}

fn publish_query_initialization(
    initialization_tx: &watch::Sender<Option<Value>>,
    initialization: &Value,
) {
    let _ = initialization_tx.send(Some(initialization.clone()));
}

async fn stream_next_if<S>(mut stream: Pin<&mut S>, enabled: bool) -> Option<UserMessage>
where
    S: Stream<Item = UserMessage>,
{
    if enabled {
        stream.as_mut().next().await
    } else {
        std::future::pending().await
    }
}

fn error_for_channel(error: &ClaudeAgentError) -> ClaudeAgentError {
    match error {
        ClaudeAgentError::CliNotFound => ClaudeAgentError::CliNotFound,
        ClaudeAgentError::CliNotFoundAt { path } => {
            ClaudeAgentError::CliNotFoundAt { path: path.clone() }
        }
        ClaudeAgentError::WorkingDirectoryNotFound { path } => {
            ClaudeAgentError::WorkingDirectoryNotFound { path: path.clone() }
        }
        ClaudeAgentError::Connection(message) => ClaudeAgentError::Connection(message.clone()),
        ClaudeAgentError::Process { exit_code, message } => ClaudeAgentError::Process {
            exit_code: *exit_code,
            message: message.clone(),
        },
        ClaudeAgentError::MessageParse { message, data } => ClaudeAgentError::MessageParse {
            message: message.clone(),
            data: data.clone(),
        },
        ClaudeAgentError::ControlTimeout(message) => {
            ClaudeAgentError::ControlTimeout(message.clone())
        }
        ClaudeAgentError::Control(message) => ClaudeAgentError::Control(message.clone()),
        ClaudeAgentError::InvalidOption(message) => {
            ClaudeAgentError::InvalidOption(message.clone())
        }
        ClaudeAgentError::ToolArguments(message) => {
            ClaudeAgentError::ToolArguments(message.clone())
        }
        ClaudeAgentError::SessionNotFound { session_id } => ClaudeAgentError::SessionNotFound {
            session_id: session_id.clone(),
        },
        ClaudeAgentError::JsonDecode { .. }
        | ClaudeAgentError::Io(_)
        | ClaudeAgentError::Json(_) => ClaudeAgentError::Connection(error.to_string()),
    }
}
