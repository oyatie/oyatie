use std::{
    collections::BTreeMap,
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    sync::Arc,
};

use serde_json::{Map, Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    process::{Child, Command},
    sync::watch,
    task::JoinHandle,
};

use crate::{
    callbacks::StderrCallback,
    error::{ClaudeAgentError, Result},
    messages::UserMessage,
    options::{ClaudeAgentOptions, find_cli},
    session_store::{MaterializedSession, effective_projects_dir, materialize_resume_session},
};

/// Runtime transport used by the SDK control loop.
///
/// The official TypeScript SDK abstracts process and WebSocket transports
/// behind a JSON-line transport interface. This crate keeps the same boundary
/// internally so subprocess and network direct-connect sessions can share the
/// initialization, callback, and message parsing machinery.
pub(crate) trait RuntimeTransport {
    fn write_json_line(
        &mut self,
        value: &Value,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
    fn read_json_line(&mut self)
    -> impl std::future::Future<Output = Result<Option<Value>>> + Send;
    fn end_input(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;
    fn wait(self) -> impl std::future::Future<Output = Result<()>> + Send
    where
        Self: Sized;
    fn projects_dir(&self) -> &PathBuf;
}

/// Options passed to a caller-provided Claude Code process spawner.
#[derive(Debug, Clone)]
pub struct ProcessSpawnOptions {
    /// Command or executable path that the default transport would run.
    pub command: PathBuf,
    /// CLI arguments after the command.
    pub args: Vec<String>,
    /// Working directory for the Claude Code process, when configured.
    pub cwd: Option<PathBuf>,
    /// Environment variables projected into the Claude Code process.
    pub env: BTreeMap<String, String>,
    /// Signal that flips when the SDK abandons the process and callers should tear it down.
    pub signal: ProcessAbortSignal,
}

/// Cancellation signal passed to custom process spawners.
///
/// The signal is `false` while the SDK still owns the process. It becomes
/// `true` when the SDK drops or terminates the transport without awaiting a
/// normal process exit, giving VM/container/remote launchers a hook for
/// asynchronous teardown in addition to the synchronous kill callback required
/// by [`SpawnedClaudeProcess::new`].
#[derive(Clone)]
pub struct ProcessAbortSignal {
    receiver: watch::Receiver<bool>,
}

impl ProcessAbortSignal {
    fn new(receiver: watch::Receiver<bool>) -> Self {
        Self { receiver }
    }

    pub fn is_aborted(&self) -> bool {
        *self.receiver.borrow()
    }

    pub async fn aborted(&mut self) -> bool {
        if self.is_aborted() {
            return true;
        }
        while self.receiver.changed().await.is_ok() {
            if self.is_aborted() {
                return true;
            }
        }
        self.is_aborted()
    }
}

impl std::fmt::Debug for ProcessAbortSignal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProcessAbortSignal")
            .field("aborted", &self.is_aborted())
            .finish()
    }
}

/// Future returned by custom process spawners.
pub type ProcessSpawnerFuture = Pin<Box<dyn Future<Output = Result<SpawnedClaudeProcess>> + Send>>;

/// Future awaited when a custom process exits.
pub type ProcessWaitFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

/// Custom Claude Code process spawner.
///
/// This is the Rust equivalent of the TypeScript SDK's
/// `spawnClaudeCodeProcess` option: the SDK computes command, arguments, cwd,
/// and environment, then lets the caller run the process locally, in a
/// container, VM, or remote environment.
pub trait ClaudeProcessSpawner: Send + Sync {
    fn spawn(&self, options: ProcessSpawnOptions) -> ProcessSpawnerFuture;
}

impl<F, Fut> ClaudeProcessSpawner for F
where
    F: Fn(ProcessSpawnOptions) -> Fut + Send + Sync,
    Fut: Future<Output = Result<SpawnedClaudeProcess>> + Send + 'static,
{
    fn spawn(&self, options: ProcessSpawnOptions) -> ProcessSpawnerFuture {
        Box::pin(self(options))
    }
}

/// Cloneable wrapper for a caller-provided process spawner.
#[derive(Clone)]
pub struct SharedClaudeProcessSpawner(Arc<dyn ClaudeProcessSpawner>);

impl SharedClaudeProcessSpawner {
    pub fn new<S>(spawner: S) -> Self
    where
        S: ClaudeProcessSpawner + 'static,
    {
        Self(Arc::new(spawner))
    }

    pub(crate) fn spawn(&self, options: ProcessSpawnOptions) -> ProcessSpawnerFuture {
        self.0.spawn(options)
    }
}

impl std::fmt::Debug for SharedClaudeProcessSpawner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SharedClaudeProcessSpawner(..)")
    }
}

/// Process handles returned by [`ClaudeProcessSpawner`].
pub struct SpawnedClaudeProcess {
    stdin: Option<Box<dyn AsyncWrite + Send + Unpin>>,
    stdout: Option<Box<dyn AsyncRead + Send + Unpin>>,
    wait: Option<ProcessWaitFuture>,
    kill: Box<dyn FnMut() + Send>,
    killed: bool,
}

impl SpawnedClaudeProcess {
    pub fn new<Stdin, Stdout, Wait, Kill>(
        stdin: Stdin,
        stdout: Stdout,
        wait: Wait,
        kill: Kill,
    ) -> Self
    where
        Stdin: AsyncWrite + Send + Unpin + 'static,
        Stdout: AsyncRead + Send + Unpin + 'static,
        Wait: Future<Output = Result<()>> + Send + 'static,
        Kill: FnMut() + Send + 'static,
    {
        Self {
            stdin: Some(Box::new(stdin)),
            stdout: Some(Box::new(stdout)),
            wait: Some(Box::pin(wait)),
            kill: Box::new(kill),
            killed: false,
        }
    }

    fn take_stdin(&mut self) -> Option<Box<dyn AsyncWrite + Send + Unpin>> {
        self.stdin.take()
    }

    fn take_stdout(&mut self) -> Option<Box<dyn AsyncRead + Send + Unpin>> {
        self.stdout.take()
    }

    async fn wait(mut self) -> Result<()> {
        match self.wait.take() {
            Some(wait) => wait.await,
            None => Ok(()),
        }
    }

    fn kill_now(&mut self) {
        if !self.killed {
            (self.kill)();
            self.killed = true;
        }
    }
}

impl std::fmt::Debug for SpawnedClaudeProcess {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SpawnedClaudeProcess(..)")
    }
}

enum TransportProcess {
    Child(Child),
    Custom {
        process: SpawnedClaudeProcess,
        abort_tx: watch::Sender<bool>,
    },
}

impl TransportProcess {
    async fn wait(self) -> Result<()> {
        match self {
            Self::Child(mut child) => {
                let status = child.wait().await?;
                if status.success() {
                    Ok(())
                } else {
                    Err(ClaudeAgentError::Process {
                        exit_code: status.code(),
                        message: "Claude Code process exited with non-zero status".into(),
                    })
                }
            }
            Self::Custom { process, .. } => process.wait().await,
        }
    }

    async fn terminate(&mut self) -> Result<()> {
        match self {
            Self::Child(child) => match child.try_wait()? {
                Some(_) => Ok(()),
                None => {
                    child.kill().await?;
                    Ok(())
                }
            },
            Self::Custom { process, abort_tx } => {
                let _ = abort_tx.send(true);
                process.kill_now();
                Ok(())
            }
        }
    }

    fn start_kill(&mut self) {
        match self {
            Self::Child(child) => {
                if matches!(child.try_wait(), Ok(None)) {
                    let _ = child.start_kill();
                }
            }
            Self::Custom { process, abort_tx } => {
                let _ = abort_tx.send(true);
                process.kill_now();
            }
        }
    }
}

/// Low-level subprocess transport for the Claude Code CLI.
pub struct SubprocessTransport {
    process: Option<TransportProcess>,
    stdin: Option<Box<dyn AsyncWrite + Send + Unpin>>,
    stdout: BufReader<Box<dyn AsyncRead + Send + Unpin>>,
    stderr_worker: Option<JoinHandle<()>>,
    stdout_line_buffer: Vec<u8>,
    max_buffer_size: Option<usize>,
    projects_dir: PathBuf,
    _materialized_session: Option<MaterializedSession>,
}

impl SubprocessTransport {
    pub async fn spawn(options: &ClaudeAgentOptions) -> Result<Self> {
        if options.spawn_claude_code_process.is_none()
            && let Some(cwd) = &options.cwd
            && !cwd.exists()
        {
            return Err(ClaudeAgentError::WorkingDirectoryNotFound { path: cwd.clone() });
        }

        let materialized_session = materialize_resume_session(options).await?;
        let mut effective_options = options.clone();
        if let Some(materialized) = &materialized_session {
            effective_options.env.insert(
                "CLAUDE_CONFIG_DIR".into(),
                materialized.config_dir().to_string_lossy().into_owned(),
            );
            effective_options.resume = Some(materialized.resume_session_id.clone());
            effective_options.continue_conversation = false;
        }

        let mut env = process_env(&effective_options);
        if materialized_session.is_some() {
            apply_windows_securestorage_config_dir_for_materialized_session(&mut env, &options.env);
        }
        if let Some(cwd) = &effective_options.cwd {
            env.insert("PWD".into(), cwd.to_string_lossy().into_owned());
        }
        let projects_dir = effective_projects_dir(&effective_options.env);
        let (process, stdin, stdout, stderr_worker) = match &effective_options
            .spawn_claude_code_process
        {
            Some(spawner) => {
                let (abort_tx, abort_rx) = watch::channel(false);
                let cli_path = effective_options
                    .cli_path
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("claude"));
                let (command, args) = command_and_args_for_cli_path(&effective_options, cli_path)?;
                let mut process = spawner
                    .spawn(ProcessSpawnOptions {
                        command,
                        args,
                        cwd: effective_options.cwd.clone(),
                        env,
                        signal: ProcessAbortSignal::new(abort_rx),
                    })
                    .await?;
                let stdin = process
                    .take_stdin()
                    .ok_or_else(|| ClaudeAgentError::Connection("failed to open stdin".into()))?;
                let stdout = process
                    .take_stdout()
                    .ok_or_else(|| ClaudeAgentError::Connection("failed to open stdout".into()))?;
                (
                    TransportProcess::Custom { process, abort_tx },
                    stdin,
                    stdout,
                    None,
                )
            }
            None => {
                let cli = find_cli(effective_options.cli_path.as_ref())?;
                let (command_path, args) = command_and_args_for_cli_path(&effective_options, cli)?;
                let mut command = Command::new(&command_path);
                command.args(&args);
                command.stdin(Stdio::piped()).stdout(Stdio::piped());
                // Do not inherit stderr by default: CLI diagnostics can include user prompts,
                // paths, or provider metadata that callers did not opt in to expose.
                if effective_options.callbacks.stderr.is_some() {
                    command.stderr(Stdio::piped());
                } else {
                    command.stderr(Stdio::null());
                }
                if let Some(cwd) = &effective_options.cwd {
                    command.current_dir(cwd);
                }
                for (key, value) in env {
                    command.env(key, value);
                }

                let mut child = command.spawn().map_err(|error| match error.kind() {
                    std::io::ErrorKind::NotFound => ClaudeAgentError::CliNotFoundAt {
                        path: command_path.clone(),
                    },
                    _ => ClaudeAgentError::Connection(error.to_string()),
                })?;
                let stdin: Box<dyn AsyncWrite + Send + Unpin> =
                    Box::new(child.stdin.take().ok_or_else(|| {
                        ClaudeAgentError::Connection("failed to open stdin".into())
                    })?);
                let stdout: Box<dyn AsyncRead + Send + Unpin> =
                    Box::new(child.stdout.take().ok_or_else(|| {
                        ClaudeAgentError::Connection("failed to open stdout".into())
                    })?);
                let stderr_worker = match effective_options.callbacks.stderr.clone() {
                    Some(callback) => {
                        let stderr = child.stderr.take().ok_or_else(|| {
                            ClaudeAgentError::Connection("failed to open stderr".into())
                        })?;
                        Some(tokio::spawn(forward_stderr(stderr, callback)))
                    }
                    None => None,
                };
                (TransportProcess::Child(child), stdin, stdout, stderr_worker)
            }
        };

        Ok(Self {
            process: Some(process),
            stdin: Some(stdin),
            stdout: BufReader::new(stdout),
            stderr_worker,
            stdout_line_buffer: Vec::new(),
            max_buffer_size: effective_options.max_buffer_size,
            projects_dir,
            _materialized_session: materialized_session,
        })
    }

    pub async fn write_json_line(&mut self, value: &Value) -> Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| ClaudeAgentError::Connection("stdin already closed".into()))?;
        stdin
            .write_all(serde_json::to_string(value)?.as_bytes())
            .await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }

    pub async fn read_json_line(&mut self) -> Result<Option<Value>> {
        loop {
            let available = self.stdout.fill_buf().await?;
            if available.is_empty() {
                if self.stdout_line_buffer.is_empty() {
                    return Ok(None);
                }
                break;
            }
            let consumed = available
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(available.len(), |position| position + 1);
            let next_len = self.stdout_line_buffer.len() + consumed;
            if self
                .max_buffer_size
                .is_some_and(|max_buffer_size| next_len > max_buffer_size)
            {
                return Err(ClaudeAgentError::Connection(format!(
                    "Claude Code output line exceeded max_buffer_size ({})",
                    self.max_buffer_size.unwrap_or_default()
                )));
            }
            self.stdout_line_buffer
                .extend_from_slice(&available[..consumed]);
            self.stdout.consume(consumed);
            if self.stdout_line_buffer.ends_with(b"\n") {
                break;
            }
        }

        let line_bytes = std::mem::take(&mut self.stdout_line_buffer);
        let line = String::from_utf8_lossy(&line_bytes).into_owned();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(Some(Value::Null));
        }
        serde_json::from_str(trimmed)
            .map(Some)
            .map_err(|source| ClaudeAgentError::JsonDecode { source, line })
    }

    pub async fn end_input(&mut self) -> Result<()> {
        if let Some(mut stdin) = self.stdin.take() {
            stdin.shutdown().await?;
        }
        Ok(())
    }

    pub async fn wait(mut self) -> Result<()> {
        let result = match self.process.take() {
            Some(process) => process.wait().await,
            None => Ok(()),
        };
        if let Some(worker) = self.stderr_worker.take() {
            let _ = worker.await;
        }
        result
    }

    pub(crate) fn projects_dir(&self) -> &PathBuf {
        &self.projects_dir
    }

    #[allow(dead_code)]
    pub async fn terminate(&mut self) -> Result<()> {
        if let Some(mut stdin) = self.stdin.take() {
            let _ = stdin.shutdown().await;
        }
        match self.process.as_mut() {
            Some(process) => process.terminate().await,
            None => Ok(()),
        }
    }
}

impl RuntimeTransport for SubprocessTransport {
    async fn write_json_line(&mut self, value: &Value) -> Result<()> {
        Self::write_json_line(self, value).await
    }

    async fn read_json_line(&mut self) -> Result<Option<Value>> {
        Self::read_json_line(self).await
    }

    async fn end_input(&mut self) -> Result<()> {
        Self::end_input(self).await
    }

    async fn wait(self) -> Result<()> {
        Self::wait(self).await
    }

    fn projects_dir(&self) -> &PathBuf {
        Self::projects_dir(self)
    }
}

impl Drop for SubprocessTransport {
    fn drop(&mut self) {
        if let Some(process) = self.process.as_mut() {
            process.start_kill();
        }
        if let Some(worker) = self.stderr_worker.take() {
            worker.abort();
        }
    }
}

async fn forward_stderr<R>(mut stderr: R, callback: StderrCallback)
where
    R: AsyncRead + Send + Unpin + 'static,
{
    let mut buffer = vec![0; 4096];
    loop {
        match stderr.read(&mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(bytes_read) => {
                let chunk = String::from_utf8_lossy(&buffer[..bytes_read]).into_owned();
                let _ = catch_unwind(AssertUnwindSafe(|| callback(chunk)));
            }
        }
    }
}

fn process_env(options: &ClaudeAgentOptions) -> BTreeMap<String, String> {
    let mut env = std::env::vars()
        .filter(|(key, _)| key != "CLAUDECODE")
        .collect::<BTreeMap<_, _>>();
    env.insert("CLAUDE_CODE_ENTRYPOINT".into(), "sdk-rs".into());
    env.insert("CLAUDE_AGENT_SDK_VERSION".into(), crate::SDK_VERSION.into());
    if options.enable_file_checkpointing {
        env.insert(
            "CLAUDE_CODE_ENABLE_SDK_FILE_CHECKPOINTING".into(),
            "true".into(),
        );
    }
    if let Some(preview_format) = options
        .tool_config
        .as_ref()
        .and_then(|config| config.ask_user_question.as_ref())
        .and_then(|config| config.preview_format)
    {
        env.insert(
            "CLAUDE_CODE_QUESTION_PREVIEW_FORMAT".into(),
            preview_format.as_env_value().into(),
        );
    }
    env.extend(options.env.clone());
    env.remove("NODE_OPTIONS");
    if env_bool(env.get("DEBUG_CLAUDE_AGENT_SDK")) {
        env.insert("DEBUG".into(), "1".into());
    } else {
        env.remove("DEBUG");
    }
    if options.callbacks.get_oauth_token.is_some() {
        env.insert("CLAUDE_CODE_SDK_HAS_OAUTH_REFRESH".into(), "1".into());
    }
    if options.callbacks.get_host_auth_token.is_some() {
        env.insert("CLAUDE_CODE_SDK_HAS_HOST_AUTH_REFRESH".into(), "1".into());
    }
    env
}

fn env_bool(value: Option<&String>) -> bool {
    value
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().trim(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn apply_windows_securestorage_config_dir_for_materialized_session(
    env: &mut BTreeMap<String, String>,
    original_options_env: &BTreeMap<String, String>,
) {
    #[cfg(windows)]
    {
        let ambient_securestorage_config_dir =
            std::env::var("CLAUDE_SECURESTORAGE_CONFIG_DIR").ok();
        let ambient_config_dir = std::env::var("CLAUDE_CONFIG_DIR").ok();
        env.insert(
            "CLAUDE_SECURESTORAGE_CONFIG_DIR".into(),
            securestorage_config_dir_for_materialized_session(
                original_options_env,
                ambient_securestorage_config_dir.as_deref(),
                ambient_config_dir.as_deref(),
            ),
        );
    }
    #[cfg(not(windows))]
    {
        let _ = (env, original_options_env);
    }
}

#[cfg(any(test, windows))]
fn securestorage_config_dir_for_materialized_session(
    original_options_env: &BTreeMap<String, String>,
    ambient_securestorage_config_dir: Option<&str>,
    ambient_config_dir: Option<&str>,
) -> String {
    original_options_env
        .get("CLAUDE_SECURESTORAGE_CONFIG_DIR")
        .map(String::as_str)
        .or(ambient_securestorage_config_dir)
        .or_else(|| {
            original_options_env
                .get("CLAUDE_CONFIG_DIR")
                .map(String::as_str)
        })
        .or(ambient_config_dir)
        .unwrap_or_default()
        .to_owned()
}

pub(crate) fn control_request(request_id: &str, request: Value) -> Value {
    json!({
        "type": "control_request",
        "request_id": request_id,
        "request": request,
    })
}

pub(crate) fn control_error_response(request_id: &str, error: impl Into<String>) -> Value {
    json!({
        "type": "control_response",
        "response": {
            "subtype": "error",
            "request_id": request_id,
            "error": error.into(),
        }
    })
}

pub(crate) fn user_prompt_message(prompt: &str) -> Value {
    json!({
        "type": "user",
        "session_id": "",
        "message": {"role": "user", "content": prompt},
        "parent_tool_use_id": null,
    })
}

pub(crate) fn user_message(message: &UserMessage) -> Result<Value> {
    let mut object = Map::new();
    object.insert("type".into(), Value::String("user".into()));
    object.insert(
        "message".into(),
        json!({
            "role": "user",
            "content": serde_json::to_value(&message.content)?,
        }),
    );
    object.insert(
        "parent_tool_use_id".into(),
        message
            .parent_tool_use_id
            .as_ref()
            .map(|value| Value::String(value.clone()))
            .unwrap_or(Value::Null),
    );
    insert_optional_string(&mut object, "uuid", &message.uuid);
    insert_optional_string(&mut object, "session_id", &message.session_id);
    if let Some(tool_use_result) = &message.tool_use_result {
        object.insert("tool_use_result".into(), tool_use_result.clone());
    }
    insert_optional_value(&mut object, "priority", &message.priority)?;
    insert_optional_bool(&mut object, "isSynthetic", message.is_synthetic);
    insert_optional_bool(&mut object, "shouldQuery", message.should_query);
    insert_optional_string(&mut object, "timestamp", &message.timestamp);
    insert_optional_value(&mut object, "origin", &message.origin)?;
    insert_optional_string(&mut object, "subagent_type", &message.subagent_type);
    insert_optional_string(&mut object, "task_description", &message.task_description);
    insert_optional_bool(&mut object, "isReplay", message.is_replay);
    insert_optional_value(&mut object, "file_attachments", &message.file_attachments)?;
    Ok(Value::Object(object))
}

fn insert_optional_string(object: &mut Map<String, Value>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        object.insert(key.into(), Value::String(value.clone()));
    }
}

fn insert_optional_bool(object: &mut Map<String, Value>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        object.insert(key.into(), Value::Bool(value));
    }
}

fn insert_optional_value<T: serde::Serialize>(
    object: &mut Map<String, Value>,
    key: &str,
    value: &Option<T>,
) -> Result<()> {
    if let Some(value) = value {
        object.insert(key.into(), serde_json::to_value(value)?);
    }
    Ok(())
}

#[allow(dead_code)]
pub(crate) fn command_for_debug(options: &ClaudeAgentOptions) -> Result<(PathBuf, Vec<String>)> {
    let cli = find_cli(options.cli_path.as_ref())?;
    command_and_args_for_cli_path(options, cli)
}

fn command_and_args_for_cli_path(
    options: &ClaudeAgentOptions,
    cli_path: PathBuf,
) -> Result<(PathBuf, Vec<String>)> {
    let mut cli_args = options.to_cli_args()?;
    let mut args = options.executable_args.clone();
    if is_javascript_cli_path(&cli_path) {
        let command = options
            .executable
            .clone()
            .unwrap_or_else(|| PathBuf::from("node"));
        args.push(cli_path.to_string_lossy().into_owned());
        args.append(&mut cli_args);
        Ok((command, args))
    } else {
        args.append(&mut cli_args);
        Ok((cli_path, args))
    }
}

fn is_javascript_cli_path(path: &Path) -> bool {
    let path = path.to_string_lossy();
    [".js", ".mjs", ".tsx", ".ts", ".jsx"]
        .iter()
        .any(|extension| path.ends_with(extension))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::OsString, sync::Mutex};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            // SAFETY: Tests serialize environment mutations with ENV_LOCK and
            // restore values in Drop before releasing the lock.
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var_os(key);
            // SAFETY: Protected by ENV_LOCK for the full guard lifetime.
            unsafe {
                std::env::remove_var(key);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // SAFETY: Protected by ENV_LOCK for the full guard lifetime.
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(self.key, value),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    #[test]
    fn command_args_wrap_javascript_cli_paths_but_not_native_paths() {
        let script_options = ClaudeAgentOptions::builder()
            .executable("deno")
            .executable_arg("--allow-read")
            .build();
        let (script_command, script_args) =
            command_and_args_for_cli_path(&script_options, PathBuf::from("claude.ts")).unwrap();
        assert_eq!(script_command, PathBuf::from("deno"));
        assert_eq!(
            script_args.first().map(String::as_str),
            Some("--allow-read")
        );
        assert_eq!(script_args.get(1).map(String::as_str), Some("claude.ts"));
        assert_eq!(
            script_args.get(2).map(String::as_str),
            Some("--output-format")
        );

        let native_options = ClaudeAgentOptions::builder()
            .executable("node")
            .executable_arg("--ignored-for-native-runtime-but-preserved-as-leading-arg")
            .build();
        let (native_command, native_args) =
            command_and_args_for_cli_path(&native_options, PathBuf::from("claude")).unwrap();
        assert_eq!(native_command, PathBuf::from("claude"));
        assert_eq!(
            native_args.first().map(String::as_str),
            Some("--ignored-for-native-runtime-but-preserved-as-leading-arg")
        );
        assert_eq!(
            native_args.get(1).map(String::as_str),
            Some("--output-format")
        );
    }

    #[test]
    fn process_env_scrubs_node_options_and_debug_like_typescript() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _node_options = EnvGuard::set("NODE_OPTIONS", "--inspect");
        let _debug = EnvGuard::set("DEBUG", "ambient-debug");
        let _sdk_debug = EnvGuard::remove("DEBUG_CLAUDE_AGENT_SDK");

        let env = process_env(&ClaudeAgentOptions::default());
        assert!(!env.contains_key("NODE_OPTIONS"));
        assert!(!env.contains_key("DEBUG"));

        let sdk_debug_options = ClaudeAgentOptions::builder()
            .env("DEBUG_CLAUDE_AGENT_SDK", "true")
            .env("DEBUG", "caller-debug")
            .build();
        let env = process_env(&sdk_debug_options);
        assert_eq!(env.get("DEBUG").map(String::as_str), Some("1"));
        assert_eq!(
            env.get("DEBUG_CLAUDE_AGENT_SDK").map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn securestorage_config_dir_follows_typescript_materialization_precedence() {
        let mut env = BTreeMap::new();
        env.insert(
            "CLAUDE_SECURESTORAGE_CONFIG_DIR".into(),
            "option-secure".into(),
        );
        env.insert("CLAUDE_CONFIG_DIR".into(), "option-config".into());
        assert_eq!(
            securestorage_config_dir_for_materialized_session(
                &env,
                Some("ambient-secure"),
                Some("ambient-config")
            ),
            "option-secure"
        );

        env.remove("CLAUDE_SECURESTORAGE_CONFIG_DIR");
        assert_eq!(
            securestorage_config_dir_for_materialized_session(
                &env,
                Some("ambient-secure"),
                Some("ambient-config")
            ),
            "ambient-secure"
        );

        assert_eq!(
            securestorage_config_dir_for_materialized_session(&env, None, Some("ambient-config")),
            "option-config"
        );

        env.remove("CLAUDE_CONFIG_DIR");
        assert_eq!(
            securestorage_config_dir_for_materialized_session(&env, None, Some("ambient-config")),
            "ambient-config"
        );

        assert_eq!(
            securestorage_config_dir_for_materialized_session(&env, None, None),
            ""
        );
    }
}
