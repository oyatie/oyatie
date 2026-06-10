use std::{collections::BTreeMap, pin::Pin};

use futures::{Stream, StreamExt, stream};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    error::{ClaudeAgentError, Result},
    messages::{Message, UserMessage},
    options::{ClaudeAgentOptions, McpServerConfig, PermissionMode},
    runtime::{
        SessionHandle, apply_flag_settings_request, array_field, background_tasks_request,
        cancel_async_message_request, claude_authenticate_request, claude_oauth_callback_request,
        claude_oauth_wait_for_completion_request, context_usage_request, enable_channel_request,
        enable_remote_control_request, file_suggestions_request, generate_session_title_request,
        get_binary_version_request, get_session_cost_request, get_settings_request,
        interrupt_request, mcp_authenticate_request, mcp_call_request, mcp_clear_auth_request,
        mcp_message_request, mcp_oauth_callback_url_request, mcp_status_request,
        message_rated_request, parse_initialization, read_file_request,
        reconnect_mcp_server_request, reload_plugins_request, rename_session_request,
        rewind_files_request, seed_read_state_request, set_color_request,
        set_max_thinking_tokens_request, set_mcp_servers_request, set_model_request,
        set_permission_mode_request, side_question_request, stop_task_request,
        submit_feedback_request, toggle_mcp_server_request, ultrareview_launch_request,
    },
    status::{
        AccountInfo, AgentInfo, ContextUsageResponse, McpServerStatus, McpSetServersResult,
        McpStatusResponse, ModelInfo, ReadFileEncoding, RewindFilesResult,
        SDKControlInitializeResponse, SDKControlReadFileResponse, SDKControlReloadPluginsResponse,
        SideQuestionResponse, SlashCommand,
    },
};

/// Stateful client for continuous Claude Agent SDK conversations.
pub struct ClaudeSDKClient {
    options: ClaudeAgentOptions,
    session: Option<SessionHandle>,
}

impl ClaudeSDKClient {
    pub fn new(options: ClaudeAgentOptions) -> Self {
        Self {
            options,
            session: None,
        }
    }

    pub fn options(&self) -> &ClaudeAgentOptions {
        &self.options
    }

    pub async fn connect(&mut self, prompt: Option<String>) -> Result<()> {
        if self.session.is_some() {
            return Ok(());
        }
        self.session = Some(SessionHandle::connect(self.options.clone(), prompt).await?);
        Ok(())
    }

    pub async fn query(&mut self, prompt: impl Into<String>) -> Result<()> {
        if self.session.is_none() {
            self.connect(None).await?;
        }
        self.session()?.query(prompt.into()).await
    }

    pub async fn query_with_session_id(
        &mut self,
        prompt: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Result<()> {
        if self.session.is_none() {
            self.connect(None).await?;
        }
        self.session()?
            .query_message(UserMessage::text(prompt).session_id(session_id))
            .await
    }

    pub async fn query_stream<S>(&mut self, messages: S) -> Result<()>
    where
        S: Stream<Item = UserMessage>,
    {
        if self.session.is_none() {
            self.connect(None).await?;
        }
        futures::pin_mut!(messages);
        while let Some(message) = messages.next().await {
            self.session()?.query_message(message).await?;
        }
        Ok(())
    }

    pub async fn connect_stream<S>(&mut self, messages: S) -> Result<()>
    where
        S: Stream<Item = UserMessage>,
    {
        self.connect(None).await?;
        self.query_stream(messages).await
    }

    pub async fn receive_next(&mut self) -> Option<Result<Message>> {
        match &mut self.session {
            Some(session) => session.receive_next().await,
            None => None,
        }
    }

    pub fn receive_messages(&mut self) -> Pin<Box<dyn Stream<Item = Result<Message>> + '_>> {
        Box::pin(stream::unfold(self, |client| async {
            client.receive_next().await.map(|message| (message, client))
        }))
    }

    /// Receive messages through the next result message, inclusive.
    pub async fn receive_response(&mut self) -> Result<Vec<Message>> {
        let mut messages = Vec::new();
        while let Some(message) = self.receive_next().await {
            let message = message?;
            let is_result = matches!(message, Message::Result(_));
            messages.push(message);
            if is_result {
                break;
            }
        }
        Ok(messages)
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

    pub async fn rewind_files(&self, user_message_id: &str, dry_run: bool) -> Result<Value> {
        self.session()?
            .control(rewind_files_request(user_message_id, dry_run))
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

    /// Return raw MCP server status JSON.
    ///
    /// Existing callers can keep using this unvalidated JSON accessor. Use
    /// [`Self::get_mcp_status_typed`] when you want this crate to validate the
    /// documented protocol envelope and parse known fields.
    pub async fn get_mcp_status(&self) -> Result<Value> {
        self.session()?.control(mcp_status_request()).await
    }

    /// Return typed MCP server status protocol data.
    ///
    /// Use the raw [`Self::get_mcp_status`] / [`Self::get_mcp_status_raw`]
    /// accessors when inspecting newer or malformed Claude Code responses that
    /// this crate cannot yet deserialize.
    pub async fn get_mcp_status_typed(&self) -> Result<McpStatusResponse> {
        self.control_typed(mcp_status_request()).await
    }

    /// Alias for [`Self::get_mcp_status`].
    pub async fn get_mcp_status_raw(&self) -> Result<Value> {
        self.get_mcp_status().await
    }

    pub async fn mcp_server_status(&self) -> Result<Vec<McpServerStatus>> {
        Ok(self.get_mcp_status_typed().await?.mcp_servers)
    }

    /// Return raw context usage JSON.
    ///
    /// Existing callers can keep using this unvalidated JSON accessor. Use
    /// [`Self::get_context_usage_typed`] when you want this crate to validate
    /// the documented protocol envelope and parse known fields.
    pub async fn get_context_usage(&self) -> Result<Value> {
        self.session()?.control(context_usage_request()).await
    }

    /// Return typed context usage protocol data.
    ///
    /// Use the raw [`Self::get_context_usage`] /
    /// [`Self::get_context_usage_raw`] accessors when inspecting newer or
    /// malformed Claude Code responses that this crate cannot yet deserialize.
    pub async fn get_context_usage_typed(&self) -> Result<ContextUsageResponse> {
        self.control_typed(context_usage_request()).await
    }

    /// Alias for [`Self::get_context_usage`].
    pub async fn get_context_usage_raw(&self) -> Result<Value> {
        self.get_context_usage().await
    }

    /// Return the current effective settings JSON for the running session.
    pub async fn get_settings(&self) -> Result<Value> {
        self.session()?.control(get_settings_request()).await
    }

    pub async fn file_suggestions(&self, query: &str) -> Result<Value> {
        self.session()?
            .control(file_suggestions_request(query))
            .await
    }

    pub async fn get_binary_version(&self) -> Result<Value> {
        self.session()?.control(get_binary_version_request()).await
    }

    pub async fn get_session_cost(&self) -> Result<Value> {
        self.session()?.control(get_session_cost_request()).await
    }

    /// Read a file through the running session's filesystem and read-permission rules.
    ///
    /// The upstream TypeScript SDK returns `null` for permission denial, missing
    /// files, or transport errors. This Rust method preserves that behavior as
    /// `Ok(None)` for control-layer failures while still returning JSON decode
    /// errors when a claimed success response does not match the documented
    /// response shape.
    pub async fn read_file(
        &self,
        path: &str,
        max_bytes: Option<u64>,
        encoding: Option<ReadFileEncoding>,
    ) -> Result<Option<SDKControlReadFileResponse>> {
        match self
            .session()?
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

    /// Reload plugins and return raw refreshed session components.
    pub async fn reload_plugins(&self) -> Result<Value> {
        self.session()?.control(reload_plugins_request()).await
    }

    /// Reload plugins and parse the documented refreshed session components.
    pub async fn reload_plugins_typed(&self) -> Result<SDKControlReloadPluginsResponse> {
        self.control_typed(reload_plugins_request()).await
    }

    /// Return raw initialization JSON captured during `connect`.
    ///
    /// Existing callers can keep using this unvalidated JSON accessor. Use
    /// [`Self::initialization_result_typed`] when you want this crate to
    /// validate the documented protocol envelope and parse known fields.
    pub fn initialization_result(&self) -> Result<Value> {
        Ok(self.session()?.initialization().clone())
    }

    /// Return typed initialization protocol data captured during `connect`.
    ///
    /// Use the raw [`Self::initialization_result`] /
    /// [`Self::initialization_result_raw`] accessors when inspecting newer or
    /// malformed Claude Code responses that this crate cannot yet deserialize.
    pub fn initialization_result_typed(&self) -> Result<SDKControlInitializeResponse> {
        parse_initialization(self.session()?.initialization().clone())
    }

    /// Alias for [`Self::initialization_result`].
    pub fn initialization_result_raw(&self) -> Result<Value> {
        self.initialization_result()
    }

    pub fn supported_commands(&self) -> Result<Vec<Value>> {
        Ok(array_field(self.session()?.initialization(), "commands"))
    }

    pub fn supported_commands_typed(&self) -> Result<Vec<SlashCommand>> {
        Ok(self.initialization_result_typed()?.commands)
    }

    pub fn supported_models(&self) -> Result<Vec<Value>> {
        Ok(array_field(self.session()?.initialization(), "models"))
    }

    pub fn supported_models_typed(&self) -> Result<Vec<ModelInfo>> {
        Ok(self.initialization_result_typed()?.models)
    }

    pub fn supported_agents(&self) -> Result<Vec<Value>> {
        Ok(array_field(self.session()?.initialization(), "agents"))
    }

    pub fn supported_agents_typed(&self) -> Result<Vec<AgentInfo>> {
        Ok(self.initialization_result_typed()?.agents)
    }

    pub fn account_info(&self) -> Result<Option<Value>> {
        Ok(self.session()?.initialization().get("account").cloned())
    }

    pub fn account_info_typed(&self) -> Result<AccountInfo> {
        Ok(self.initialization_result_typed()?.account)
    }

    /// Alias for [`Self::initialization_result`].
    pub fn get_server_info(&self) -> Result<Value> {
        self.initialization_result()
    }

    /// Alias for [`Self::initialization_result_typed`].
    pub fn get_server_info_typed(&self) -> Result<SDKControlInitializeResponse> {
        self.initialization_result_typed()
    }

    /// Alias for [`Self::initialization_result_raw`].
    pub fn get_server_info_raw(&self) -> Result<Value> {
        self.initialization_result_raw()
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
        self.session()?
            .control(mcp_authenticate_request(server_name, redirect_uri))
            .await
    }

    pub async fn mcp_clear_auth(&self, server_name: &str) -> Result<Value> {
        self.session()?
            .control(mcp_clear_auth_request(server_name))
            .await
    }

    pub async fn mcp_call(&self, tool: &str, arguments: Option<Value>) -> Result<Value> {
        self.session()?
            .control(mcp_call_request(tool, arguments))
            .await
    }

    pub async fn mcp_message(&self, server_name: &str, message: Value) -> Result<Value> {
        self.session()?
            .control(mcp_message_request(server_name, message))
            .await
    }

    pub async fn mcp_submit_oauth_callback_url(
        &self,
        server_name: &str,
        callback_url: &str,
    ) -> Result<Value> {
        self.session()?
            .control(mcp_oauth_callback_url_request(server_name, callback_url))
            .await
    }

    pub async fn claude_authenticate(&self, login_with_claude_ai: bool) -> Result<Value> {
        self.session()?
            .control(claude_authenticate_request(login_with_claude_ai))
            .await
    }

    pub async fn claude_oauth_callback(
        &self,
        authorization_code: &str,
        state: &str,
    ) -> Result<Value> {
        self.session()?
            .control(claude_oauth_callback_request(authorization_code, state))
            .await
    }

    pub async fn claude_oauth_wait_for_completion(&self) -> Result<Value> {
        self.session()?
            .control(claude_oauth_wait_for_completion_request())
            .await
    }

    pub async fn set_mcp_servers(
        &self,
        servers: &BTreeMap<String, McpServerConfig>,
    ) -> Result<Value> {
        self.session()?
            .control(set_mcp_servers_request(servers)?)
            .await
    }

    /// Dynamically replace SDK-managed MCP servers and parse the documented
    /// added/removed/error result.
    pub async fn set_mcp_servers_typed(
        &self,
        servers: &BTreeMap<String, McpServerConfig>,
    ) -> Result<McpSetServersResult> {
        serde_json::from_value(self.set_mcp_servers(servers).await?).map_err(Into::into)
    }

    /// Alias for [`Self::set_mcp_servers`].
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
        let response = self
            .session()?
            .control(background_tasks_request(tool_use_id))
            .await?;
        Ok(response
            .get("backgrounded")
            .and_then(Value::as_bool)
            .unwrap_or(true))
    }

    pub async fn seed_read_state(&self, path: &str, mtime: u64) -> Result<()> {
        self.control_unit(seed_read_state_request(path, mtime))
            .await
    }

    pub async fn cancel_async_message(&self, message_uuid: &str) -> Result<bool> {
        let response = self
            .session()?
            .control(cancel_async_message_request(message_uuid))
            .await?;
        Ok(response
            .get("cancelled")
            .and_then(Value::as_bool)
            .unwrap_or(false))
    }

    pub async fn enable_remote_control(&self, enabled: bool, name: Option<&str>) -> Result<Value> {
        self.session()?
            .control(enable_remote_control_request(enabled, name))
            .await
    }

    pub async fn submit_feedback(&self, description: &str, surface: Option<&str>) -> Result<Value> {
        self.session()?
            .control(submit_feedback_request(description, surface))
            .await
    }

    pub async fn generate_session_title(
        &self,
        description: &str,
        persist: Option<bool>,
    ) -> Result<String> {
        let response = self
            .session()?
            .control(generate_session_title_request(description, persist))
            .await?;
        response
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| ClaudeAgentError::Control("missing generated session title".into()))
    }

    pub async fn ask_side_question(&self, question: &str) -> Result<Option<SideQuestionResponse>> {
        let response = self
            .session()?
            .control(side_question_request(question))
            .await?;
        if response.get("response").is_some_and(Value::is_null) {
            return Ok(None);
        }
        serde_json::from_value(response)
            .map(Some)
            .map_err(Into::into)
    }

    pub async fn launch_ultrareview(&self, args: &[String], confirm: bool) -> Result<Value> {
        self.session()?
            .control(ultrareview_launch_request(args, confirm))
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

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(session) = self.session.take() {
            session.shutdown().await?;
        }
        Ok(())
    }

    fn session(&self) -> Result<&SessionHandle> {
        self.session.as_ref().ok_or_else(|| {
            ClaudeAgentError::Connection(
                "client is not connected; call connect() or query() first".into(),
            )
        })
    }

    async fn control_unit(&self, request: Value) -> Result<()> {
        self.session()?.control(request).await.map(|_| ())
    }

    async fn control_typed<T>(&self, request: Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(self.session()?.control(request).await?).map_err(Into::into)
    }
}

impl Default for ClaudeSDKClient {
    fn default() -> Self {
        Self::new(ClaudeAgentOptions::default())
    }
}
