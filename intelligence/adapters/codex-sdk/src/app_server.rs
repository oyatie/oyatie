use std::collections::{HashMap, VecDeque};
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[cfg(feature = "runtime")]
use std::ffi::OsString;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::error::{CodexError, Result};

type PendingResponses = Arc<Mutex<HashMap<String, SyncSender<Result<Value>>>>>;
type NotificationResult = Result<Notification>;
type NotificationReceiver = Arc<Mutex<Receiver<NotificationResult>>>;
type SharedStdin = Arc<Mutex<ChildStdin>>;
type SharedStderrTail = Arc<Mutex<Vec<u8>>>;
type ServerRequestHandler =
    Arc<dyn Fn(&str, Option<Value>) -> Result<Value> + Send + Sync + 'static>;

const STDERR_TAIL_BYTES: usize = 64 * 1024;

/// Upstream `openai/codex` commit used for the current app-server contract manifest.
pub const CURRENT_UPSTREAM_MAIN_SHA: &str = "ad2012d645b7146d31bb03f98e2bd9371635d11a";

/// Current generated app-server request methods from the Python SDK contract.
///
/// This is intentionally a method-name manifest rather than a full generated Rust
/// model set. `AppServerClient::request_object` keeps every method reachable with
/// raw JSON params while typed Rust models remain partial and forward-compatible.
pub const CURRENT_APP_SERVER_REQUEST_METHODS: &[&str] = &[
    "initialize",
    "thread/resume",
    "thread/archive",
    "thread/unsubscribe",
    "thread/name/set",
    "thread/goal/get",
    "thread/goal/clear",
    "thread/metadata/update",
    "thread/unarchive",
    "thread/compact/start",
    "thread/shellCommand",
    "thread/approveGuardianDeniedAction",
    "thread/rollback",
    "thread/loaded/list",
    "thread/read",
    "thread/inject_items",
    "skills/list",
    "skills/extraRoots/set",
    "hooks/list",
    "marketplace/add",
    "marketplace/remove",
    "marketplace/upgrade",
    "plugin/list",
    "plugin/installed",
    "plugin/read",
    "plugin/skill/read",
    "plugin/share/list",
    "plugin/share/checkout",
    "plugin/share/delete",
    "app/list",
    "fs/readFile",
    "fs/writeFile",
    "fs/createDirectory",
    "fs/getMetadata",
    "fs/readDirectory",
    "fs/remove",
    "fs/copy",
    "fs/watch",
    "fs/unwatch",
    "skills/config/write",
    "plugin/install",
    "plugin/uninstall",
    "turn/interrupt",
    "model/list",
    "modelProvider/capabilities/read",
    "experimentalFeature/list",
    "permissionProfile/list",
    "experimentalFeature/enablement/set",
    "mcpServer/oauth/login",
    "config/mcpServer/reload",
    "mcpServer/resource/read",
    "mcpServer/tool/call",
    "windowsSandbox/setupStart",
    "windowsSandbox/readiness",
    "account/login/start",
    "account/login/cancel",
    "account/logout",
    "account/rateLimits/read",
    "account/sendAddCreditsNudgeEmail",
    "feedback/upload",
    "command/exec/write",
    "command/exec/terminate",
    "config/read",
    "externalAgentConfig/detect",
    "configRequirements/read",
    "account/read",
    "fuzzyFileSearch",
    "thread/start",
    "thread/fork",
    "thread/goal/set",
    "thread/list",
    "plugin/share/updateTargets",
    "turn/start",
    "turn/steer",
    "review/start",
    "mcpServerStatus/list",
    "command/exec",
    "command/exec/resize",
    "config/value/write",
    "plugin/share/save",
    "config/batchWrite",
    "externalAgentConfig/import",
];

#[derive(Clone)]
struct ScopedNotificationQueue {
    tx: Sender<NotificationResult>,
    rx: NotificationReceiver,
}

struct NotificationRouter {
    state: Mutex<NotificationRouterState>,
    global_tx: Sender<NotificationResult>,
    global_rx: NotificationReceiver,
}

#[derive(Default)]
struct NotificationRouterState {
    login_queues: HashMap<String, ScopedNotificationQueue>,
    pending_login_notifications: HashMap<String, VecDeque<Notification>>,
    turn_queues: HashMap<String, ScopedNotificationQueue>,
    pending_turn_notifications: HashMap<String, VecDeque<Notification>>,
}

/// JSON object used by the app-server JSON-RPC contract.
pub type JsonObject = Map<String, Value>;

/// Server metadata returned by `initialize`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub name: Option<String>,
    pub version: Option<String>,
}

/// Result returned by the app-server `initialize` request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    pub server_info: Option<ServerInfo>,
    pub user_agent: Option<String>,
    pub platform_family: Option<String>,
    pub platform_os: Option<String>,
}

/// JSON-RPC notification emitted by `codex app-server`.
#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

/// Turn input item for the app-server contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppInput {
    Text { text: String },
    Image { url: String },
    LocalImage { path: String },
    Skill { name: String, path: String },
    Mention { name: String, path: String },
}

impl AppInput {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub fn image(url: impl Into<String>) -> Self {
        Self::Image { url: url.into() }
    }

    pub fn local_image(path: impl Into<String>) -> Self {
        Self::LocalImage { path: path.into() }
    }

    pub fn skill(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self::Skill {
            name: name.into(),
            path: path.into(),
        }
    }

    pub fn mention(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self::Mention {
            name: name.into(),
            path: path.into(),
        }
    }

    fn into_wire(self) -> Value {
        match self {
            Self::Text { text } => json!({ "type": "text", "text": text }),
            Self::Image { url } => json!({ "type": "image", "url": url }),
            Self::LocalImage { path } => json!({ "type": "localImage", "path": path }),
            Self::Skill { name, path } => {
                json!({ "type": "skill", "name": name, "path": path })
            }
            Self::Mention { name, path } => {
                json!({ "type": "mention", "name": name, "path": path })
            }
        }
    }
}

/// App-server run/turn input accepted by `turn_start` and `turn_steer`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppRunInput {
    Text(String),
    Items(Vec<AppInput>),
}

impl From<&str> for AppRunInput {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for AppRunInput {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<AppInput> for AppRunInput {
    fn from(value: AppInput) -> Self {
        Self::Items(vec![value])
    }
}

impl From<Vec<AppInput>> for AppRunInput {
    fn from(value: Vec<AppInput>) -> Self {
        Self::Items(value)
    }
}

impl<const N: usize> From<[AppInput; N]> for AppRunInput {
    fn from(value: [AppInput; N]) -> Self {
        Self::Items(value.into())
    }
}

fn app_input_to_wire(input: impl Into<AppRunInput>) -> Vec<Value> {
    match input.into() {
        AppRunInput::Text(text) => vec![AppInput::text(text).into_wire()],
        AppRunInput::Items(items) => items.into_iter().map(AppInput::into_wire).collect(),
    }
}

/// Configuration for launching `codex app-server --listen stdio://`.
#[derive(Clone)]
pub struct AppServerConfig {
    codex_path_override: Option<PathBuf>,
    launch_args_override: Option<Vec<String>>,
    config_overrides: Vec<String>,
    cwd: Option<PathBuf>,
    env: Option<HashMap<String, String>>,
    client_name: String,
    client_title: String,
    client_version: String,
    experimental_api: bool,
    server_request_handler: ServerRequestHandler,
}

impl Default for AppServerConfig {
    fn default() -> Self {
        Self {
            codex_path_override: None,
            launch_args_override: None,
            config_overrides: Vec::new(),
            cwd: None,
            env: None,
            client_name: "codex_rust_sdk".to_string(),
            client_title: "Codex Rust SDK".to_string(),
            client_version: crate::SDK_VERSION.to_string(),
            experimental_api: true,
            server_request_handler: Arc::new(default_server_request_handler),
        }
    }
}

impl std::fmt::Debug for AppServerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppServerConfig")
            .field("codex_path_override", &self.codex_path_override)
            .field("launch_args_override", &self.launch_args_override)
            .field("config_overrides", &self.config_overrides)
            .field("cwd", &self.cwd)
            .field("env", &self.env)
            .field("client_name", &self.client_name)
            .field("client_title", &self.client_title)
            .field("client_version", &self.client_version)
            .field("experimental_api", &self.experimental_api)
            .field("server_request_handler", &"<handler>")
            .finish()
    }
}

impl AppServerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use a specific Codex executable instead of resolving `codex` from `PATH`.
    pub fn with_codex_path_override(mut self, path: impl Into<PathBuf>) -> Self {
        self.codex_path_override = Some(path.into());
        self
    }

    /// Override the full launch argv. The first element is the executable.
    pub fn with_launch_args_override<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.launch_args_override = Some(args.into_iter().map(Into::into).collect());
        self
    }

    /// Add a raw `--config key=value` override before app-server startup args.
    pub fn with_config_override(mut self, override_value: impl Into<String>) -> Self {
        self.config_overrides.push(override_value.into());
        self
    }

    /// Add raw `--config key=value` overrides before app-server startup args.
    pub fn with_config_overrides<I, S>(mut self, overrides: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config_overrides = overrides.into_iter().map(Into::into).collect();
        self
    }

    /// Set the current working directory for the app-server process.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Merge these variables into the app-server environment.
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = Some(env);
        self
    }

    /// Override client metadata sent in `initialize`.
    pub fn with_client_info(
        mut self,
        name: impl Into<String>,
        title: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        self.client_name = name.into();
        self.client_title = title.into();
        self.client_version = version.into();
        self
    }

    /// Set the app-server experimental API capability flag.
    pub fn with_experimental_api(mut self, enabled: bool) -> Self {
        self.experimental_api = enabled;
        self
    }

    /// Override handling for app-server JSON-RPC requests sent to the SDK.
    ///
    /// The default matches the Python SDK baseline for approval callbacks by
    /// accepting command and file-change approval requests and returning `{}` for
    /// unknown server request methods.
    pub fn with_server_request_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, Option<Value>) -> Result<Value> + Send + Sync + 'static,
    {
        self.server_request_handler = Arc::new(handler);
        self
    }

    fn launch_command(&self) -> Result<(String, Vec<String>)> {
        if let Some(args) = &self.launch_args_override {
            let Some((program, rest)) = args.split_first() else {
                return Err(CodexError::InvalidConfig(
                    "launch_args_override must include an executable".to_string(),
                ));
            };
            return Ok((program.clone(), rest.to_vec()));
        }

        let program = self
            .codex_path_override
            .clone()
            .unwrap_or_else(default_codex_program)
            .to_string_lossy()
            .into_owned();
        let mut args = Vec::new();
        for override_value in &self.config_overrides {
            args.push("--config".to_string());
            args.push(override_value.clone());
        }
        args.extend([
            "app-server".to_string(),
            "--listen".to_string(),
            "stdio://".to_string(),
        ]);
        Ok((program, args))
    }

    fn process_env(&self) -> HashMap<String, String> {
        let mut env_map = env::vars().collect::<HashMap<_, _>>();
        apply_runtime_path_dirs(&mut env_map);
        if let Some(env) = &self.env {
            env_map.extend(env.clone());
            apply_runtime_path_dirs(&mut env_map);
        }
        env_map
    }
}

fn default_codex_program() -> PathBuf {
    #[cfg(feature = "runtime")]
    if let Some(path) = runtime_codex_path() {
        return path;
    }

    PathBuf::from("codex")
}

#[cfg(feature = "runtime")]
fn runtime_codex_path() -> Option<PathBuf> {
    env::var_os("OPENAI_CODEX_RUNTIME_BIN")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn apply_runtime_path_dirs(env_map: &mut HashMap<String, String>) {
    #[cfg(not(feature = "runtime"))]
    let _ = env_map;

    #[cfg(feature = "runtime")]
    {
        let Some(raw_dirs) = env_map
            .get("OPENAI_CODEX_RUNTIME_PATH_DIRS")
            .filter(|value| !value.is_empty())
            .map(OsString::from)
            .or_else(|| {
                env::var_os("OPENAI_CODEX_RUNTIME_PATH_DIRS").filter(|value| !value.is_empty())
            })
        else {
            return;
        };

        let mut paths = env::split_paths(&raw_dirs).collect::<Vec<_>>();
        let existing_path = env_map.get("PATH").map(PathBuf::from);
        if let Some(existing_path) = existing_path {
            paths.extend(env::split_paths(existing_path.as_os_str()));
        }
        if let Ok(joined) = env::join_paths(paths) {
            env_map.insert("PATH".to_string(), joined.to_string_lossy().into_owned());
        }
    }
}

struct AppServerProcess {
    child: Child,
    stdin: SharedStdin,
    _reader_handle: JoinHandle<()>,
    _stderr_handle: JoinHandle<std::io::Result<()>>,
    _stderr_tail: SharedStderrTail,
}

struct AppServerInner {
    config: AppServerConfig,
    process: Mutex<Option<AppServerProcess>>,
    pending: PendingResponses,
    router: Arc<NotificationRouter>,
    request_id: AtomicU64,
    closed: Arc<AtomicBool>,
}

/// Blocking JSON-RPC client for `codex app-server --listen stdio://`.
#[derive(Clone)]
pub struct AppServerClient {
    inner: Arc<AppServerInner>,
}

impl AppServerClient {
    pub fn new(config: AppServerConfig) -> Self {
        Self {
            inner: Arc::new(AppServerInner {
                config,
                process: Mutex::new(None),
                pending: Arc::new(Mutex::new(HashMap::new())),
                router: Arc::new(NotificationRouter::new()),
                request_id: AtomicU64::new(1),
                closed: Arc::new(AtomicBool::new(false)),
            }),
        }
    }

    /// Start the app-server process. Calling this more than once is a no-op.
    pub fn start(&self) -> Result<()> {
        let mut process = self.lock_process()?;
        if self.inner.closed.load(Ordering::SeqCst) {
            if let Some(process) = process.take() {
                shutdown_process(process);
            }
            return Err(CodexError::TransportClosed);
        }
        if process.is_some() {
            return Ok(());
        }

        let (program, args) = self.inner.config.launch_command()?;
        let mut command = Command::new(program);
        command.args(args);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(self.inner.config.process_env());
        if let Some(cwd) = &self.inner.config.cwd {
            command.current_dir(cwd);
        }

        let mut child = command.spawn()?;
        let stdin = Arc::new(Mutex::new(
            child.stdin.take().ok_or(CodexError::MissingPipe("stdin"))?,
        ));
        let stdout = child
            .stdout
            .take()
            .ok_or(CodexError::MissingPipe("stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or(CodexError::MissingPipe("stderr"))?;

        let pending = Arc::clone(&self.inner.pending);
        let router = Arc::clone(&self.inner.router);
        let closed = Arc::clone(&self.inner.closed);
        let reader_stdin = Arc::clone(&stdin);
        let stderr_tail = Arc::new(Mutex::new(Vec::new()));
        let reader_stderr_tail = Arc::clone(&stderr_tail);
        let server_request_handler = Arc::clone(&self.inner.config.server_request_handler);
        let reader_handle = thread::spawn(move || {
            read_app_server_stdout(
                stdout,
                pending,
                router,
                closed,
                reader_stdin,
                reader_stderr_tail,
                server_request_handler,
            )
        });
        let stderr_handle = drain_stderr_tail(stderr, Arc::clone(&stderr_tail));

        *process = Some(AppServerProcess {
            child,
            stdin,
            _reader_handle: reader_handle,
            _stderr_handle: stderr_handle,
            _stderr_tail: stderr_tail,
        });
        Ok(())
    }

    /// Close the app-server process and fail outstanding requests.
    pub fn close(&self) {
        self.inner.closed.store(true, Ordering::SeqCst);
        if let Ok(mut process) = self.inner.process.lock()
            && let Some(process) = process.take()
        {
            shutdown_process(process);
        }
        fail_pending(&self.inner.pending);
        self.inner.router.fail_all();
    }

    /// Initialize the app-server and send the follow-up `initialized` notification.
    pub fn initialize(&self) -> Result<InitializeResponse> {
        let result = self.request(
            "initialize",
            Some(json!({
                "clientInfo": {
                    "name": self.inner.config.client_name,
                    "title": self.inner.config.client_title,
                    "version": self.inner.config.client_version,
                },
                "capabilities": {
                    "experimentalApi": self.inner.config.experimental_api,
                },
            })),
        )?;
        self.notify("initialized", None)?;
        serde_json::from_value(result).map_err(CodexError::from)
    }

    /// Send a raw JSON-RPC request and return the raw `result` payload.
    pub fn request(&self, method: impl Into<String>, params: Option<Value>) -> Result<Value> {
        self.start()?;
        let id = self
            .inner
            .request_id
            .fetch_add(1, Ordering::SeqCst)
            .to_string();
        let (tx, rx) = mpsc::sync_channel(1);
        self.lock_pending()?.insert(id.clone(), tx);
        if self.inner.closed.load(Ordering::SeqCst) {
            let _ = self.lock_pending().map(|mut pending| pending.remove(&id));
            return Err(CodexError::TransportClosed);
        }

        let method = method.into();
        let mut message = Map::new();
        message.insert("id".to_string(), Value::String(id.clone()));
        message.insert("method".to_string(), Value::String(method));
        if let Some(params) = params {
            message.insert("params".to_string(), params);
        }

        if let Err(err) = self.write_message(Value::Object(message)) {
            let _ = self.lock_pending().map(|mut pending| pending.remove(&id));
            return Err(err);
        }

        rx.recv().map_err(|_| CodexError::TransportClosed)?
    }

    /// Send a raw JSON-RPC request whose params must be a JSON object.
    ///
    /// This is the forward-compatible escape hatch for every method in
    /// `CURRENT_APP_SERVER_REQUEST_METHODS` while generated Rust protocol models
    /// remain partial.
    pub fn request_object(
        &self,
        method: impl Into<String>,
        params: Option<Value>,
    ) -> Result<Value> {
        self.request(method, Some(Value::Object(object_params(params)?)))
    }

    /// Send a raw JSON-RPC notification.
    pub fn notify(&self, method: impl Into<String>, params: Option<Value>) -> Result<()> {
        self.start()?;
        let mut message = Map::new();
        message.insert("method".to_string(), Value::String(method.into()));
        if let Some(params) = params {
            message.insert("params".to_string(), params);
        }
        self.write_message(Value::Object(message))
    }

    /// Return the next app-server notification.
    pub fn next_notification(&self) -> Result<Notification> {
        self.inner.router.next_global_notification()
    }

    /// Start routing completion notifications for one interactive login attempt.
    pub fn register_login_notifications(&self, login_id: impl Into<String>) -> Result<()> {
        self.inner.router.register_login(login_id)
    }

    /// Stop routing future notifications for one interactive login attempt.
    pub fn unregister_login_notifications(&self, login_id: impl AsRef<str>) -> Result<()> {
        self.inner.router.unregister_login(login_id.as_ref())
    }

    /// Return the next routed notification for a registered login attempt.
    pub fn next_login_notification(&self, login_id: impl AsRef<str>) -> Result<Notification> {
        self.inner.router.next_login_notification(login_id.as_ref())
    }

    /// Start routing notifications for one turn id.
    pub fn register_turn_notifications(&self, turn_id: impl Into<String>) -> Result<()> {
        self.inner.router.register_turn(turn_id)
    }

    /// Stop routing future notifications for one turn id.
    pub fn unregister_turn_notifications(&self, turn_id: impl AsRef<str>) -> Result<()> {
        self.inner.router.unregister_turn(turn_id.as_ref())
    }

    /// Return the next routed notification for a registered turn id.
    pub fn next_turn_notification(&self, turn_id: impl AsRef<str>) -> Result<Notification> {
        self.inner.router.next_turn_notification(turn_id.as_ref())
    }

    pub fn account_login_start(&self, params: Option<Value>) -> Result<Value> {
        let response = self.request(
            "account/login/start",
            Some(Value::Object(object_params(params)?)),
        )?;
        if let Some(login_id) = interactive_login_id(&response) {
            self.register_login_notifications(login_id)?;
        }
        Ok(response)
    }

    pub fn account_login_cancel(&self, login_id: impl Into<String>) -> Result<Value> {
        self.request(
            "account/login/cancel",
            Some(json!({ "loginId": login_id.into() })),
        )
    }

    pub fn account_read(&self, params: Option<Value>) -> Result<Value> {
        self.request("account/read", Some(Value::Object(object_params(params)?)))
    }

    pub fn account_logout(&self) -> Result<Value> {
        self.request("account/logout", None)
    }

    pub fn thread_start(&self, params: Option<Value>) -> Result<Value> {
        self.request("thread/start", Some(Value::Object(object_params(params)?)))
    }

    pub fn thread_resume(
        &self,
        thread_id: impl Into<String>,
        params: Option<Value>,
    ) -> Result<Value> {
        let mut payload = Map::new();
        payload.insert("threadId".to_string(), Value::String(thread_id.into()));
        payload.extend(object_params(params)?);
        self.request("thread/resume", Some(Value::Object(payload)))
    }

    pub fn thread_list(&self, params: Option<Value>) -> Result<Value> {
        self.request("thread/list", Some(Value::Object(object_params(params)?)))
    }

    pub fn thread_read(&self, thread_id: impl Into<String>, include_turns: bool) -> Result<Value> {
        self.request(
            "thread/read",
            Some(json!({ "threadId": thread_id.into(), "includeTurns": include_turns })),
        )
    }

    pub fn thread_fork(
        &self,
        thread_id: impl Into<String>,
        params: Option<Value>,
    ) -> Result<Value> {
        let mut payload = Map::new();
        payload.insert("threadId".to_string(), Value::String(thread_id.into()));
        payload.extend(object_params(params)?);
        self.request("thread/fork", Some(Value::Object(payload)))
    }

    pub fn thread_archive(&self, thread_id: impl Into<String>) -> Result<Value> {
        self.request(
            "thread/archive",
            Some(json!({ "threadId": thread_id.into() })),
        )
    }

    pub fn thread_unarchive(&self, thread_id: impl Into<String>) -> Result<Value> {
        self.request(
            "thread/unarchive",
            Some(json!({ "threadId": thread_id.into() })),
        )
    }

    pub fn thread_set_name(
        &self,
        thread_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Value> {
        self.request(
            "thread/name/set",
            Some(json!({ "threadId": thread_id.into(), "name": name.into() })),
        )
    }

    pub fn thread_compact(&self, thread_id: impl Into<String>) -> Result<Value> {
        self.request(
            "thread/compact/start",
            Some(json!({ "threadId": thread_id.into() })),
        )
    }

    pub fn turn_start<I>(
        &self,
        thread_id: impl Into<String>,
        input: I,
        params: Option<Value>,
    ) -> Result<Value>
    where
        I: Into<AppRunInput>,
    {
        let mut params = object_params(params)?;
        params.insert("threadId".to_string(), Value::String(thread_id.into()));
        params.insert("input".to_string(), Value::Array(app_input_to_wire(input)));
        let response = self.request("turn/start", Some(Value::Object(params)))?;
        if let Some(turn_id) = response_turn_id(&response) {
            self.register_turn_notifications(turn_id)?;
        }
        Ok(response)
    }

    pub fn turn_interrupt(
        &self,
        thread_id: impl Into<String>,
        turn_id: impl Into<String>,
    ) -> Result<Value> {
        self.request(
            "turn/interrupt",
            Some(json!({ "threadId": thread_id.into(), "turnId": turn_id.into() })),
        )
    }

    pub fn turn_steer<I>(
        &self,
        thread_id: impl Into<String>,
        expected_turn_id: impl Into<String>,
        input: I,
    ) -> Result<Value>
    where
        I: Into<AppRunInput>,
    {
        self.request(
            "turn/steer",
            Some(json!({
                "threadId": thread_id.into(),
                "expectedTurnId": expected_turn_id.into(),
                "input": app_input_to_wire(input),
            })),
        )
    }

    pub fn model_list(&self, include_hidden: bool) -> Result<Value> {
        self.request(
            "model/list",
            Some(json!({ "includeHidden": include_hidden })),
        )
    }

    fn write_message(&self, message: Value) -> Result<()> {
        let mut process = self.lock_process()?;
        let Some(process) = process.as_mut() else {
            return Err(CodexError::TransportClosed);
        };
        write_json_message(&process.stdin, &message)
    }

    fn lock_process(&self) -> Result<std::sync::MutexGuard<'_, Option<AppServerProcess>>> {
        self.inner
            .process
            .lock()
            .map_err(|_| CodexError::Protocol("app-server process lock poisoned".to_string()))
    }

    fn lock_pending(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, SyncSender<Result<Value>>>>> {
        self.inner
            .pending
            .lock()
            .map_err(|_| CodexError::Protocol("app-server pending lock poisoned".to_string()))
    }
}

/// High-level blocking app-server API mirroring the Python SDK's common sync workflow.
#[derive(Clone)]
pub struct AppCodex {
    client: AppServerClient,
    metadata: InitializeResponse,
}

impl AppCodex {
    /// Start and initialize `codex app-server --listen stdio://`.
    pub fn new(config: AppServerConfig) -> Result<Self> {
        let client = AppServerClient::new(config);
        match client.initialize() {
            Ok(metadata) => Ok(Self { client, metadata }),
            Err(err) => {
                client.close();
                Err(err)
            }
        }
    }

    /// Initialization metadata returned by the app-server.
    pub fn metadata(&self) -> &InitializeResponse {
        &self.metadata
    }

    /// Borrow the underlying raw JSON-RPC client.
    pub fn client(&self) -> &AppServerClient {
        &self.client
    }

    /// Close the app-server process.
    pub fn close(&self) {
        self.client.close();
    }

    /// Authenticate with an API key.
    pub fn login_api_key(&self, api_key: impl Into<String>) -> Result<()> {
        self.client
            .account_login_start(Some(json!({
                "type": "apiKey",
                "apiKey": api_key.into(),
            })))
            .map(|_| ())
    }

    /// Start browser-based ChatGPT login and return a routed login handle.
    pub fn login_chatgpt(&self) -> Result<AppLoginHandle> {
        let response = self
            .client
            .account_login_start(Some(json!({ "type": "chatgpt" })))?;
        AppLoginHandle::from_response(self.client.clone(), response)
    }

    /// Start device-code ChatGPT login and return a routed login handle.
    pub fn login_chatgpt_device_code(&self) -> Result<AppLoginHandle> {
        let response = self
            .client
            .account_login_start(Some(json!({ "type": "chatgptDeviceCode" })))?;
        AppLoginHandle::from_response(self.client.clone(), response)
    }

    /// Read the current account state.
    pub fn account(&self, refresh_token: bool) -> Result<Value> {
        self.client
            .account_read(Some(json!({ "refreshToken": refresh_token })))
    }

    /// Clear the current account session.
    pub fn logout(&self) -> Result<()> {
        self.client.account_logout().map(|_| ())
    }

    /// Create a new Codex conversation thread.
    pub fn thread_start(&self, params: Option<Value>) -> Result<AppThread> {
        let response = self.client.thread_start(params)?;
        AppThread::from_response(self.client.clone(), "thread/start", response)
    }

    /// List saved conversation threads.
    pub fn thread_list(&self, params: Option<Value>) -> Result<Value> {
        self.client.thread_list(params)
    }

    /// Resume an existing conversation thread by ID.
    pub fn thread_resume(
        &self,
        thread_id: impl Into<String>,
        params: Option<Value>,
    ) -> Result<AppThread> {
        let response = self.client.thread_resume(thread_id, params)?;
        AppThread::from_response(self.client.clone(), "thread/resume", response)
    }

    /// Create a new thread from an existing thread.
    pub fn thread_fork(
        &self,
        thread_id: impl Into<String>,
        params: Option<Value>,
    ) -> Result<AppThread> {
        let response = self.client.thread_fork(thread_id, params)?;
        AppThread::from_response(self.client.clone(), "thread/fork", response)
    }

    /// Archive a conversation thread.
    pub fn thread_archive(&self, thread_id: impl Into<String>) -> Result<Value> {
        self.client.thread_archive(thread_id)
    }

    /// Unarchive a conversation thread and return its handle.
    pub fn thread_unarchive(&self, thread_id: impl Into<String>) -> Result<AppThread> {
        let response = self.client.thread_unarchive(thread_id)?;
        AppThread::from_response(self.client.clone(), "thread/unarchive", response)
    }

    /// List available models.
    pub fn models(&self, include_hidden: bool) -> Result<Value> {
        self.client.model_list(include_hidden)
    }
}

/// Routed interactive-login handle.
#[derive(Clone)]
pub struct AppLoginHandle {
    client: AppServerClient,
    login_id: String,
    auth_url: Option<String>,
    verification_url: Option<String>,
    user_code: Option<String>,
}

impl AppLoginHandle {
    fn from_response(client: AppServerClient, response: Value) -> Result<Self> {
        let login_id = response
            .get("loginId")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CodexError::Protocol("account/login/start response missing loginId".to_string())
            })?
            .to_string();
        Ok(Self {
            client,
            login_id,
            auth_url: response
                .get("authUrl")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            verification_url: response
                .get("verificationUrl")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            user_code: response
                .get("userCode")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        })
    }

    pub fn login_id(&self) -> &str {
        &self.login_id
    }

    pub fn auth_url(&self) -> Option<&str> {
        self.auth_url.as_deref()
    }

    pub fn verification_url(&self) -> Option<&str> {
        self.verification_url.as_deref()
    }

    pub fn user_code(&self) -> Option<&str> {
        self.user_code.as_deref()
    }

    /// Wait for the completion notification for this login attempt.
    pub fn wait(&self) -> Result<Notification> {
        self.client.next_login_notification(&self.login_id)
    }

    /// Cancel this login attempt.
    pub fn cancel(&self) -> Result<Value> {
        self.client.account_login_cancel(&self.login_id)
    }
}

/// High-level thread handle backed by app-server JSON-RPC.
#[derive(Clone)]
pub struct AppThread {
    client: AppServerClient,
    id: String,
}

impl AppThread {
    fn from_response(client: AppServerClient, method: &str, response: Value) -> Result<Self> {
        let id = extract_thread_id(method, &response)?;
        Ok(Self { client, id })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// Start a turn, consume routed notifications until completion, and return a result summary.
    pub fn run<I>(&self, input: I, params: Option<Value>) -> Result<AppTurnResult>
    where
        I: Into<AppRunInput>,
    {
        self.turn(input, params)?.run()
    }

    /// Start a turn and return a low-level turn handle for streaming/control.
    pub fn turn<I>(&self, input: I, params: Option<Value>) -> Result<AppTurnHandle>
    where
        I: Into<AppRunInput>,
    {
        let response = self.client.turn_start(&self.id, input, params)?;
        let turn_id = response_turn_id(&response).ok_or_else(|| {
            CodexError::Protocol("turn/start response missing turn.id".to_string())
        })?;
        Ok(AppTurnHandle {
            client: self.client.clone(),
            thread_id: self.id.clone(),
            id: turn_id,
        })
    }

    pub fn read(&self, include_turns: bool) -> Result<Value> {
        self.client.thread_read(&self.id, include_turns)
    }

    pub fn set_name(&self, name: impl Into<String>) -> Result<Value> {
        self.client.thread_set_name(&self.id, name)
    }

    pub fn compact(&self) -> Result<Value> {
        self.client.thread_compact(&self.id)
    }
}

/// Low-level turn handle for routed streaming, steering, interruption, and collection.
#[derive(Clone)]
pub struct AppTurnHandle {
    client: AppServerClient,
    thread_id: String,
    id: String,
}

impl AppTurnHandle {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }

    pub fn steer<I>(&self, input: I) -> Result<Value>
    where
        I: Into<AppRunInput>,
    {
        self.client.turn_steer(&self.thread_id, &self.id, input)
    }

    pub fn interrupt(&self) -> Result<Value> {
        self.client.turn_interrupt(&self.thread_id, &self.id)
    }

    /// Stream routed notifications for this turn until `turn/completed` or an error.
    pub fn stream(&self) -> AppTurnStream {
        AppTurnStream {
            client: self.client.clone(),
            turn_id: self.id.clone(),
            done: false,
        }
    }

    /// Consume this turn's stream and summarize the completed turn.
    pub fn run(&self) -> Result<AppTurnResult> {
        let mut result = AppTurnResult::new(self.id.clone());
        for notification in self.stream() {
            let notification = notification?;
            result.record_notification(&notification);
            if notification.method == "turn/completed" {
                break;
            }
        }
        result.final_response = final_response_from_items(&result.items);
        if result.status.as_deref() == Some("failed") {
            let message = result
                .error
                .as_ref()
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("Codex turn failed")
                .to_string();
            return Err(CodexError::TurnFailed { message });
        }
        Ok(result)
    }
}

/// Blocking iterator over routed turn notifications.
#[derive(Clone)]
pub struct AppTurnStream {
    client: AppServerClient,
    turn_id: String,
    done: bool,
}

impl Iterator for AppTurnStream {
    type Item = Result<Notification>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.client.next_turn_notification(&self.turn_id) {
            Ok(notification) => {
                if notification.method == "turn/completed" {
                    self.done = true;
                }
                Some(Ok(notification))
            }
            Err(err) => {
                self.done = true;
                Some(Err(err))
            }
        }
    }
}

/// Summary returned by high-level app-server turn collection.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AppTurnResult {
    pub id: String,
    pub status: Option<String>,
    pub error: Option<Value>,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<i64>,
    pub final_response: Option<String>,
    pub items: Vec<Value>,
    pub usage: Option<Value>,
}

impl AppTurnResult {
    fn new(id: String) -> Self {
        Self {
            id,
            ..Self::default()
        }
    }

    fn record_notification(&mut self, notification: &Notification) {
        match notification.method.as_str() {
            "turn/started" => {
                if let Some(turn) = notification.params.get("turn") {
                    self.record_turn(turn, false);
                }
            }
            "item/completed" => {
                if let Some(item) = notification.params.get("item") {
                    self.items.push(item.clone());
                }
            }
            "thread/tokenUsage/updated" => {
                self.usage = notification.params.get("tokenUsage").cloned();
            }
            "turn/completed" => {
                if let Some(turn) = notification.params.get("turn") {
                    self.record_turn(turn, true);
                }
            }
            _ => {}
        }
    }

    fn record_turn(&mut self, turn: &Value, completed: bool) {
        if let Some(id) = turn.get("id").and_then(Value::as_str) {
            self.id = id.to_string();
        }
        if let Some(status) = turn.get("status").and_then(Value::as_str) {
            self.status = Some(status.to_string());
        }
        self.error = turn.get("error").cloned().filter(|value| !value.is_null());
        self.started_at = turn
            .get("startedAt")
            .and_then(Value::as_i64)
            .or(self.started_at);
        self.completed_at = turn
            .get("completedAt")
            .and_then(Value::as_i64)
            .or(self.completed_at);
        self.duration_ms = turn
            .get("durationMs")
            .and_then(Value::as_i64)
            .or(self.duration_ms);
        if completed
            && self.items.is_empty()
            && let Some(items) = turn.get("items").and_then(Value::as_array)
        {
            self.items = items.clone();
        }
    }
}

impl Drop for AppServerClient {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            self.close();
        }
    }
}

impl NotificationRouter {
    fn new() -> Self {
        let (global_tx, global_rx) = mpsc::channel();
        Self {
            state: Mutex::new(NotificationRouterState::default()),
            global_tx,
            global_rx: Arc::new(Mutex::new(global_rx)),
        }
    }

    fn next_global_notification(&self) -> Result<Notification> {
        recv_notification(&self.global_rx)
    }

    fn register_login(&self, login_id: impl Into<String>) -> Result<()> {
        let login_id = login_id.into();
        let (tx, replay) = {
            let mut state = self.lock_state()?;
            if let Some(queue) = state.login_queues.get(&login_id) {
                return replay_notifications(&queue.tx, VecDeque::new());
            }
            let queue = new_notification_queue();
            let tx = queue.tx.clone();
            let replay = state
                .pending_login_notifications
                .remove(&login_id)
                .unwrap_or_default();
            state.login_queues.insert(login_id, queue);
            (tx, replay)
        };
        replay_notifications(&tx, replay)
    }

    fn unregister_login(&self, login_id: &str) -> Result<()> {
        self.lock_state()?.login_queues.remove(login_id);
        Ok(())
    }

    fn next_login_notification(&self, login_id: &str) -> Result<Notification> {
        let rx = {
            let state = self.lock_state()?;
            state
                .login_queues
                .get(login_id)
                .map(|queue| Arc::clone(&queue.rx))
                .ok_or_else(|| {
                    CodexError::Protocol(format!(
                        "login {login_id:?} is not registered for notifications"
                    ))
                })?
        };
        recv_notification(&rx)
    }

    fn register_turn(&self, turn_id: impl Into<String>) -> Result<()> {
        let turn_id = turn_id.into();
        let (tx, replay) = {
            let mut state = self.lock_state()?;
            if let Some(queue) = state.turn_queues.get(&turn_id) {
                return replay_notifications(&queue.tx, VecDeque::new());
            }
            let queue = new_notification_queue();
            let tx = queue.tx.clone();
            let replay = state
                .pending_turn_notifications
                .remove(&turn_id)
                .unwrap_or_default();
            state.turn_queues.insert(turn_id, queue);
            (tx, replay)
        };
        replay_notifications(&tx, replay)
    }

    fn unregister_turn(&self, turn_id: &str) -> Result<()> {
        self.lock_state()?.turn_queues.remove(turn_id);
        Ok(())
    }

    fn next_turn_notification(&self, turn_id: &str) -> Result<Notification> {
        let rx = {
            let state = self.lock_state()?;
            state
                .turn_queues
                .get(turn_id)
                .map(|queue| Arc::clone(&queue.rx))
                .ok_or_else(|| {
                    CodexError::Protocol(format!(
                        "turn {turn_id:?} is not registered for notifications"
                    ))
                })?
        };
        recv_notification(&rx)
    }

    fn route_notification(&self, notification: Notification) -> Result<()> {
        if let Some(login_id) = notification_login_id(&notification) {
            return self.route_login_notification(login_id, notification);
        }

        if let Some(turn_id) = notification_turn_id(&notification) {
            return self.route_turn_notification(turn_id, notification);
        }

        self.global_tx
            .send(Ok(notification))
            .map_err(|_| CodexError::TransportClosed)
    }

    fn route_login_notification(&self, login_id: String, notification: Notification) -> Result<()> {
        let send_to = {
            let mut state = self.lock_state()?;
            if let Some(queue) = state.login_queues.get(&login_id) {
                queue.tx.clone()
            } else {
                state
                    .pending_login_notifications
                    .entry(login_id)
                    .or_default()
                    .push_back(notification);
                return Ok(());
            }
        };
        send_to
            .send(Ok(notification))
            .map_err(|_| CodexError::TransportClosed)
    }

    fn route_turn_notification(&self, turn_id: String, notification: Notification) -> Result<()> {
        let send_to = {
            let mut state = self.lock_state()?;
            if let Some(queue) = state.turn_queues.get(&turn_id) {
                queue.tx.clone()
            } else {
                state
                    .pending_turn_notifications
                    .entry(turn_id)
                    .or_default()
                    .push_back(notification);
                return Ok(());
            }
        };
        send_to
            .send(Ok(notification))
            .map_err(|_| CodexError::TransportClosed)
    }

    fn fail_all(&self) {
        let (login_queues, turn_queues) = match self.state.lock() {
            Ok(mut state) => {
                let login_queues = state
                    .login_queues
                    .drain()
                    .map(|(_, queue)| queue.tx)
                    .collect::<Vec<_>>();
                let turn_queues = state
                    .turn_queues
                    .drain()
                    .map(|(_, queue)| queue.tx)
                    .collect::<Vec<_>>();
                state.pending_login_notifications.clear();
                state.pending_turn_notifications.clear();
                (login_queues, turn_queues)
            }
            Err(_) => (Vec::new(), Vec::new()),
        };

        for tx in login_queues {
            let _ = tx.send(Err(CodexError::TransportClosed));
        }
        for tx in turn_queues {
            let _ = tx.send(Err(CodexError::TransportClosed));
        }
        let _ = self.global_tx.send(Err(CodexError::TransportClosed));
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, NotificationRouterState>> {
        self.state
            .lock()
            .map_err(|_| CodexError::Protocol("app-server router lock poisoned".to_string()))
    }
}

fn new_notification_queue() -> ScopedNotificationQueue {
    let (tx, rx) = mpsc::channel();
    ScopedNotificationQueue {
        tx,
        rx: Arc::new(Mutex::new(rx)),
    }
}

fn replay_notifications(
    tx: &Sender<NotificationResult>,
    notifications: VecDeque<Notification>,
) -> Result<()> {
    for notification in notifications {
        tx.send(Ok(notification))
            .map_err(|_| CodexError::TransportClosed)?;
    }
    Ok(())
}

fn recv_notification(rx: &NotificationReceiver) -> Result<Notification> {
    rx
        .lock()
        .map_err(|_| CodexError::Protocol("app-server notification lock poisoned".to_string()))?
        .recv()
        .map_err(|_| CodexError::TransportClosed)?
}

fn default_server_request_handler(method: &str, _params: Option<Value>) -> Result<Value> {
    match method {
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval" => {
            Ok(json!({ "decision": "accept" }))
        }
        _ => Ok(json!({})),
    }
}

fn write_json_message(stdin: &SharedStdin, message: &Value) -> Result<()> {
    let line = serde_json::to_string(message)?;
    let mut stdin = stdin
        .lock()
        .map_err(|_| CodexError::Protocol("app-server stdin lock poisoned".to_string()))?;
    stdin.write_all(line.as_bytes())?;
    stdin.write_all(b"\n")?;
    stdin.flush()?;
    Ok(())
}

fn shutdown_process(mut process: AppServerProcess) {
    drop(process.stdin);
    thread::sleep(Duration::from_millis(25));
    let _ = process.child.kill();
    let _ = process.child.wait();
}

fn drain_stderr_tail(
    stderr: std::process::ChildStderr,
    tail: SharedStderrTail,
) -> JoinHandle<std::io::Result<()>> {
    thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut buffer = [0_u8; 8192];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                return Ok(());
            }
            append_bounded_tail(&tail, &buffer[..read]);
        }
    })
}

fn append_bounded_tail(tail: &SharedStderrTail, bytes: &[u8]) {
    if let Ok(mut tail) = tail.lock() {
        tail.extend_from_slice(bytes);
        if tail.len() > STDERR_TAIL_BYTES {
            let excess = tail.len() - STDERR_TAIL_BYTES;
            tail.drain(0..excess);
        }
    }
}

fn stderr_tail(tail: &SharedStderrTail) -> String {
    match tail.lock() {
        Ok(tail) => String::from_utf8_lossy(&tail).into_owned(),
        Err(_) => "<stderr tail lock poisoned>".to_string(),
    }
}

fn object_params(params: Option<Value>) -> Result<Map<String, Value>> {
    match params {
        Some(Value::Object(object)) => Ok(object),
        Some(_) => Err(CodexError::InvalidConfig(
            "app-server params must be a JSON object".to_string(),
        )),
        None => Ok(Map::new()),
    }
}

fn interactive_login_id(response: &Value) -> Option<String> {
    match response.get("type").and_then(Value::as_str) {
        Some("chatgpt" | "chatgptDeviceCode") => response
            .get("loginId")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        _ => None,
    }
}

fn extract_thread_id(method: &str, response: &Value) -> Result<String> {
    response
        .get("thread")
        .and_then(Value::as_object)
        .and_then(|thread| thread.get("id"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| CodexError::Protocol(format!("{method} response missing thread.id")))
}

fn response_turn_id(response: &Value) -> Option<String> {
    response
        .get("turn")
        .and_then(Value::as_object)
        .and_then(|turn| turn.get("id"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn final_response_from_items(items: &[Value]) -> Option<String> {
    items
        .iter()
        .rev()
        .find_map(|item| {
            if is_agent_message(item)
                && item.get("phase").and_then(Value::as_str) == Some("final_answer")
            {
                item.get("text")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            } else {
                None
            }
        })
        .or_else(|| {
            items.iter().rev().find_map(|item| {
                let has_no_phase = item.get("phase").is_none_or(Value::is_null);
                if is_agent_message(item) && has_no_phase {
                    item.get("text")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                } else {
                    None
                }
            })
        })
}

fn is_agent_message(item: &Value) -> bool {
    matches!(
        item.get("type").and_then(Value::as_str),
        Some("agentMessage" | "agent_message")
    )
}

fn notification_login_id(notification: &Notification) -> Option<String> {
    if notification.method != "account/login/completed" {
        return None;
    }
    notification
        .params
        .get("loginId")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn notification_turn_id(notification: &Notification) -> Option<String> {
    notification
        .params
        .get("turnId")
        .and_then(Value::as_str)
        .or_else(|| {
            notification
                .params
                .get("turn")
                .and_then(Value::as_object)
                .and_then(|turn| turn.get("id"))
                .and_then(Value::as_str)
        })
        .map(ToOwned::to_owned)
}

fn read_app_server_stdout(
    stdout: std::process::ChildStdout,
    pending: PendingResponses,
    router: Arc<NotificationRouter>,
    closed: Arc<AtomicBool>,
    stdin: SharedStdin,
    stderr_tail_buffer: SharedStderrTail,
    server_request_handler: ServerRequestHandler,
) {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let result = match line {
            Ok(line) => {
                handle_app_server_line(&line, &pending, &router, &stdin, &server_request_handler)
            }
            Err(err) => Err(CodexError::Io(err)),
        };
        if let Err(err) = result {
            closed.store(true, Ordering::SeqCst);
            fail_pending_with(
                &pending,
                format!(
                    "app-server reader failed: {err}; stderr_tail={}",
                    stderr_tail(&stderr_tail_buffer)
                ),
            );
            router.fail_all();
            return;
        }
    }
    closed.store(true, Ordering::SeqCst);
    fail_pending(&pending);
    router.fail_all();
}

fn handle_app_server_line(
    line: &str,
    pending: &PendingResponses,
    router: &NotificationRouter,
    stdin: &SharedStdin,
    server_request_handler: &ServerRequestHandler,
) -> Result<()> {
    let value: Value = serde_json::from_str(line)?;
    let Some(object) = value.as_object() else {
        return Err(CodexError::Protocol(
            "app-server message must be a JSON object".to_string(),
        ));
    };

    if object.get("id").is_some() && object.get("method").is_some() {
        return handle_server_request(object, stdin, server_request_handler);
    }

    if let Some(id) = object.get("id") {
        let id = id
            .as_str()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| id.to_string());
        let response = if let Some(error) = object.get("error") {
            Err(rpc_error(error))
        } else if object.contains_key("result") {
            Ok(object.get("result").cloned().unwrap_or(Value::Null))
        } else {
            Err(CodexError::Protocol(format!(
                "app-server response {id} missing result or error: {value}"
            )))
        };
        if let Ok(mut pending) = pending.lock()
            && let Some(waiter) = pending.remove(&id)
        {
            let _ = waiter.send(response);
        }
        return Ok(());
    }

    if let Some(method) = object.get("method").and_then(Value::as_str) {
        let notification = Notification {
            method: method.to_string(),
            params: object.get("params").cloned().unwrap_or(Value::Null),
        };
        return router.route_notification(notification);
    }

    Err(CodexError::Protocol(
        "app-server message must contain id or method".to_string(),
    ))
}

fn handle_server_request(
    object: &Map<String, Value>,
    stdin: &SharedStdin,
    server_request_handler: &ServerRequestHandler,
) -> Result<()> {
    let id = object
        .get("id")
        .cloned()
        .ok_or_else(|| CodexError::Protocol("server request missing id".to_string()))?;
    let method = object
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CodexError::Protocol("server request method must be a string".to_string())
        })?;
    let params = object.get("params").cloned();
    let result = server_request_handler(method, params)?;
    write_json_message(stdin, &json!({ "id": id, "result": result }))
}

fn rpc_error(error: &Value) -> CodexError {
    let code = error.get("code").and_then(Value::as_i64).unwrap_or(-32000);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("app-server JSON-RPC error")
        .to_string();
    let data = error.get("data").cloned();
    CodexError::Rpc {
        code,
        message,
        data,
    }
}

fn fail_pending(pending: &PendingResponses) {
    if let Ok(mut pending) = pending.lock() {
        for (_, waiter) in pending.drain() {
            let _ = waiter.send(Err(CodexError::TransportClosed));
        }
    }
}

fn fail_pending_with(pending: &PendingResponses, message: String) {
    if let Ok(mut pending) = pending.lock() {
        for (_, waiter) in pending.drain() {
            let _ = waiter.send(Err(CodexError::Protocol(message.clone())));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_input_wire_tags_match_python_contract() {
        assert_eq!(
            app_input_to_wire([
                AppInput::text("hello"),
                AppInput::image("https://example.test/a.png"),
                AppInput::local_image("./a.png"),
                AppInput::skill("skill", "skills/skill/SKILL.md"),
                AppInput::mention("repo", "file:///repo"),
            ]),
            vec![
                json!({"type":"text", "text":"hello"}),
                json!({"type":"image", "url":"https://example.test/a.png"}),
                json!({"type":"localImage", "path":"./a.png"}),
                json!({"type":"skill", "name":"skill", "path":"skills/skill/SKILL.md"}),
                json!({"type":"mention", "name":"repo", "path":"file:///repo"}),
            ]
        );
    }

    #[test]
    fn stderr_tail_is_bounded() {
        let tail = Arc::new(Mutex::new(Vec::new()));
        let mut bytes = vec![b'x'; STDERR_TAIL_BYTES + 32];
        bytes.extend_from_slice(b"tail-marker");

        append_bounded_tail(&tail, &bytes);

        let retained = tail.lock().unwrap().clone();
        assert_eq!(retained.len(), STDERR_TAIL_BYTES);
        assert!(String::from_utf8_lossy(&retained).ends_with("tail-marker"));
    }
}
