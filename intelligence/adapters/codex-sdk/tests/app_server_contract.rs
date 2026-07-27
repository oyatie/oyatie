#![cfg(unix)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use intelligence_codex_sdk::{
    AppCodex, AppInput, AppServerClient, AppServerConfig, CodexError,
    CURRENT_APP_SERVER_REQUEST_METHODS, CURRENT_UPSTREAM_MAIN_SHA,
};
use serde_json::{json, Value};
use tempfile::TempDir;

use std::os::unix::fs::PermissionsExt;

/// The fake app-server executable, written EXACTLY ONCE per test process.
///
/// Every test in this binary used to write its own copy and then exec it. That is a race, and it
/// failed in CI with `Os { code: 26, kind: ExecutableFileBusy }`:
///
/// `Command::spawn` forks before it execs. A forked child inherits every open descriptor, and
/// although Rust opens files `CLOEXEC`, the descriptor stays open in the child for the whole
/// fork→exec window. So while test A was inside `fs::write` creating its executable, test B's fork
/// could capture A's *write* descriptor — and A's own exec then hit ETXTBSY, because the kernel
/// refuses to execute a file that any process holds open for writing.
///
/// Writing once behind a `OnceLock` closes the window structurally rather than by retrying: every
/// test in this binary must obtain this path before it can construct a client, so no test can be
/// forking while the single write happens — they are all parked on this lock. Other test binaries
/// are separate processes and cannot inherit these descriptors at all.
///
/// The `TempDir` is deliberately leaked: it must outlive every test, and a `static` is never
/// dropped, so binding it here would only create a dangling path.
static FAKE_APP_SERVER: OnceLock<PathBuf> = OnceLock::new();

fn fake_app_server_path() -> PathBuf {
    FAKE_APP_SERVER
        .get_or_init(|| {
            let dir: &'static TempDir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
            let path = dir.path().join("codex");
            write_fake_app_server(&path);
            path
        })
        .clone()
}

struct FakeAppServer {
    _dir: TempDir,
    path: PathBuf,
    messages_file: PathBuf,
    args_file: PathBuf,
}

impl FakeAppServer {
    fn new() -> Self {
        // The executable is shared and written once; the message/arg sinks stay per-test, since
        // every test asserts on its own transcript.
        let dir = tempfile::tempdir().unwrap();
        let messages_file = dir.path().join("messages.jsonl");
        let args_file = dir.path().join("args.txt");
        Self {
            _dir: dir,
            path: fake_app_server_path(),
            messages_file,
            args_file,
        }
    }

    fn env(&self) -> HashMap<String, String> {
        HashMap::from([
            (
                "CODEX_APP_MESSAGES_FILE".to_string(),
                self.messages_file.display().to_string(),
            ),
            (
                "CODEX_APP_ARGS_FILE".to_string(),
                self.args_file.display().to_string(),
            ),
        ])
    }

    fn env_with_scenario(&self, scenario: &str) -> HashMap<String, String> {
        let mut env = self.env();
        env.insert("CODEX_APP_SCENARIO".to_string(), scenario.to_string());
        env
    }

    fn messages(&self) -> Vec<Value> {
        fs::read_to_string(&self.messages_file)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn args(&self) -> Vec<String> {
        fs::read_to_string(&self.args_file)
            .unwrap()
            .lines()
            .map(ToOwned::to_owned)
            .collect()
    }
}

/// How many times the fake executable has actually been written. The whole point of the
/// `OnceLock` above is that this stays at 1 no matter how many fakes are constructed, so the
/// counter is what [`fake_app_server_is_written_exactly_once`] asserts against.
static WRITES: AtomicUsize = AtomicUsize::new(0);

fn write_fake_app_server(path: &Path) {
    WRITES.fetch_add(1, Ordering::SeqCst);
    fs::write(
        path,
        r#"#!/usr/bin/env python3
import json
import os
import sys

messages_path = os.environ["CODEX_APP_MESSAGES_FILE"]
args_path = os.environ["CODEX_APP_ARGS_FILE"]
scenario = os.environ.get("CODEX_APP_SCENARIO", "")

with open(args_path, "w", encoding="utf-8") as args_file:
    for arg in sys.argv[1:]:
        args_file.write(arg + "\n")


def write_message(message):
    with open(messages_path, "a", encoding="utf-8") as messages_file:
        messages_file.write(json.dumps(message, sort_keys=True) + "\n")


def send(message):
    sys.stdout.write(json.dumps(message, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def result_for(method, params):
    if method == "initialize":
        return {
            "serverInfo": {"name": "codex-cli", "version": "fake"},
            "userAgent": "codex-cli/fake",
            "platformFamily": "unix",
            "platformOs": "test",
        }
    if method in ("thread/start", "thread/resume", "thread/fork", "thread/unarchive", "thread/read"):
        return {"thread": {"id": params.get("threadId", "thread-1"), "items": [], "turns": []}}
    if method == "thread/list":
        return {"data": [{"id": "thread-1", "items": [], "turns": []}], "nextCursor": None, "backwardsCursor": None}
    if method in ("thread/archive", "thread/compact/start", "thread/name/set"):
        return {}
    if method == "turn/start":
        if scenario in ("server-request", "server-file-request"):
            approval_method = "item/fileChange/requestApproval" if scenario == "server-file-request" else "item/commandExecution/requestApproval"
            send({"id": "approval-1", "method": approval_method, "params": {"threadId": params["threadId"], "turnId": "turn-1"}})
            approval_response = json.loads(sys.stdin.readline())
            write_message(approval_response)
        send({"method": "turn/started", "params": {"threadId": params["threadId"], "turn": {"id": "turn-1", "status": "running", "items": []}}})
        if scenario == "complete-run":
            final_item = {"id": "item-final", "type": "agentMessage", "phase": "final_answer", "text": "done from app api"}
            send({"method": "item/completed", "params": {"threadId": params["threadId"], "turnId": "turn-1", "completedAtMs": 2000, "item": final_item}})
            send({"method": "thread/tokenUsage/updated", "params": {"threadId": params["threadId"], "turnId": "turn-1", "tokenUsage": {"inputTokens": 11, "outputTokens": 7, "totalTokens": 18}}})
            send({"method": "turn/completed", "params": {"threadId": params["threadId"], "turn": {"id": "turn-1", "status": "completed", "startedAt": 1, "completedAt": 2, "durationMs": 1000, "items": [final_item]}}})
        return {"turn": {"id": "turn-1", "status": "running", "items": []}}
    if method == "turn/steer":
        return {"accepted": True}
    if method == "turn/interrupt":
        return {"accepted": True}
    if method == "model/list":
        return {"data": [{"id": "gpt-test"}]}
    if method == "account/read":
        return {"account": None}
    if method == "account/logout":
        return {}
    if method == "account/login/cancel":
        return {"status": "cancelled"}
    if method == "account/login/start":
        if params.get("type") == "chatgpt":
            send({"method": "account/login/completed", "params": {"loginId": "login-1", "success": True}})
            return {"type": "chatgpt", "loginId": "login-1", "authUrl": "https://example.test/login"}
        return {"type": "apiKey"}
    return {"ok": True}


for line in sys.stdin:
    message = json.loads(line)
    write_message(message)
    if scenario == "invalid-json" and message.get("method") == "break-reader":
        sys.stdout.write("{invalid json\n")
        sys.stdout.flush()
        continue
    if scenario == "malformed-response" and message.get("method") == "malformed":
        send({"id": message["id"]})
        continue
    if "id" not in message:
        continue
    if "method" not in message:
        continue
    try:
        params = message.get("params") or {}
        send({"id": message["id"], "result": result_for(message["method"], params)})
    except Exception as exc:
        send({"id": message["id"], "error": {"code": -32000, "message": str(exc)}})
"#,
    )
    .unwrap();

    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

fn client_with_fake(fake: &FakeAppServer) -> AppServerClient {
    AppServerClient::new(
        AppServerConfig::new()
            .with_codex_path_override(&fake.path)
            .with_env(fake.env()),
    )
}

fn client_with_fake_scenario(fake: &FakeAppServer, scenario: &str) -> AppServerClient {
    AppServerClient::new(
        AppServerConfig::new()
            .with_codex_path_override(&fake.path)
            .with_env(fake.env_with_scenario(scenario)),
    )
}

fn app_codex_with_fake_scenario(fake: &FakeAppServer, scenario: &str) -> AppCodex {
    AppCodex::new(
        AppServerConfig::new()
            .with_codex_path_override(&fake.path)
            .with_env(fake.env_with_scenario(scenario)),
    )
    .unwrap()
}

fn manifest_params(method: &str) -> Value {
    match method {
        "account/login/cancel" => json!({"loginId": "login-1"}),
        "thread/resume"
        | "thread/archive"
        | "thread/unsubscribe"
        | "thread/name/set"
        | "thread/goal/get"
        | "thread/goal/clear"
        | "thread/metadata/update"
        | "thread/unarchive"
        | "thread/compact/start"
        | "thread/shellCommand"
        | "thread/approveGuardianDeniedAction"
        | "thread/rollback"
        | "thread/read"
        | "thread/fork"
        | "thread/goal/set" => json!({"threadId": "thread-1", "name": "Rust SDK"}),
        "thread/inject_items" => json!({"threadId": "thread-1", "items": []}),
        "turn/start" => {
            json!({"threadId": "thread-1", "input": [{"type": "text", "text": "hello"}]})
        }
        "turn/steer" => {
            json!({"threadId": "thread-1", "expectedTurnId": "turn-1", "input": [{"type": "text", "text": "hello"}]})
        }
        "turn/interrupt" => json!({"threadId": "thread-1", "turnId": "turn-1"}),
        "command/exec/write" | "command/exec/terminate" | "command/exec/resize" => {
            json!({"callId": "cmd-1"})
        }
        "fs/watch" | "fs/unwatch" => json!({"watchId": "watch-1", "path": "/repo"}),
        "plugin/read"
        | "plugin/skill/read"
        | "plugin/share/checkout"
        | "plugin/share/delete"
        | "plugin/share/updateTargets"
        | "plugin/share/save"
        | "plugin/uninstall" => json!({"id": "plugin-1"}),
        "plugin/install" => json!({"source": {"type": "local", "path": "/repo/plugin"}}),
        "mcpServer/resource/read" | "mcpServer/tool/call" | "mcpServer/oauth/login" => {
            json!({"serverId": "mcp-1"})
        }
        "windowsSandbox/setupStart" => json!({"mode": "unelevated"}),
        _ => json!({"probe": method}),
    }
}

#[test]
fn initializes_app_server_and_sends_initialized_notification() {
    let fake = FakeAppServer::new();
    let client = client_with_fake(&fake);

    let init = client.initialize().unwrap();
    client.close();

    assert_eq!(init.user_agent.as_deref(), Some("codex-cli/fake"));
    assert_eq!(init.server_info.unwrap().name.as_deref(), Some("codex-cli"));
    assert_eq!(fake.args(), vec!["app-server", "--listen", "stdio://"]);

    let messages = fake.messages();
    assert_eq!(messages[0]["method"], "initialize");
    assert_eq!(
        messages[0]["params"]["clientInfo"]["name"],
        "codex_rust_sdk"
    );
    assert_eq!(
        messages[0]["params"]["capabilities"]["experimentalApi"],
        true
    );
    assert_eq!(messages[1], json!({"method":"initialized"}));
}

#[test]
fn current_upstream_request_method_manifest_is_exercised_against_app_server() {
    let fake = FakeAppServer::new();
    let client = client_with_fake(&fake);

    assert_eq!(
        CURRENT_UPSTREAM_MAIN_SHA,
        "ad2012d645b7146d31bb03f98e2bd9371635d11a"
    );
    assert_eq!(CURRENT_APP_SERVER_REQUEST_METHODS.len(), 82);

    client.initialize().unwrap();
    for method in CURRENT_APP_SERVER_REQUEST_METHODS
        .iter()
        .copied()
        .filter(|method| *method != "initialize")
    {
        client
            .request_object(method, Some(manifest_params(method)))
            .unwrap_or_else(|err| panic!("{method} failed: {err}"));
    }
    client.close();

    let sent_methods = fake
        .messages()
        .into_iter()
        .filter_map(|message| {
            message
                .get("id")
                .map(|_| message["method"].as_str().unwrap().to_owned())
        })
        .collect::<Vec<_>>();

    assert_eq!(sent_methods, CURRENT_APP_SERVER_REQUEST_METHODS);
}

#[test]
fn app_codex_high_level_api_collects_turn_result_and_login_handles() {
    let fake = FakeAppServer::new();
    let codex = app_codex_with_fake_scenario(&fake, "complete-run");

    assert_eq!(
        codex.metadata().user_agent.as_deref(),
        Some("codex-cli/fake")
    );

    codex.login_api_key("sk-test").unwrap();
    let login = codex.login_chatgpt().unwrap();
    assert_eq!(login.login_id(), "login-1");
    assert_eq!(login.auth_url(), Some("https://example.test/login"));
    assert_eq!(login.wait().unwrap().method, "account/login/completed");

    assert_eq!(codex.account(true).unwrap()["account"], Value::Null);
    let thread = codex
        .thread_start(Some(json!({"model": "gpt-test"})))
        .unwrap();
    assert_eq!(thread.id(), "thread-1");

    let result = thread
        .run("hello app api", Some(json!({"model": "gpt-turn"})))
        .unwrap();

    assert_eq!(result.id, "turn-1");
    assert_eq!(result.status.as_deref(), Some("completed"));
    assert_eq!(result.final_response.as_deref(), Some("done from app api"));
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.usage.unwrap()["totalTokens"], 18);
    assert_eq!(result.duration_ms, Some(1000));

    thread.set_name("Rust SDK").unwrap();
    thread.compact().unwrap();
    codex.close();

    let messages = fake.messages();
    assert!(messages.iter().any(|message| {
        message["method"] == "account/login/start"
            && message["params"] == json!({"type": "apiKey", "apiKey": "sk-test"})
    }));
    assert!(messages.iter().any(|message| {
        message["method"] == "turn/start"
            && message["params"]["input"] == json!([{"type": "text", "text": "hello app api"}])
    }));
}

#[test]
fn exposes_python_app_server_method_payload_contracts() {
    let fake = FakeAppServer::new();
    let client = client_with_fake(&fake);

    client.initialize().unwrap();
    client
        .thread_start(Some(json!({"model":"gpt-test", "cwd":"/repo"})))
        .unwrap();
    client
        .thread_resume("thread-1", Some(json!({"model":"gpt-resume"})))
        .unwrap();
    client
        .thread_list(Some(json!({"limit": 10, "searchTerm": "sdk"})))
        .unwrap();
    client.thread_read("thread-1", true).unwrap();
    client
        .thread_fork("thread-1", Some(json!({"ephemeral": true})))
        .unwrap();
    client.thread_archive("thread-1").unwrap();
    client.thread_unarchive("thread-1").unwrap();
    client.thread_set_name("thread-1", "Rust SDK").unwrap();
    client.thread_compact("thread-1").unwrap();
    client.model_list(true).unwrap();
    client
        .account_read(Some(json!({"refreshToken": true})))
        .unwrap();
    client.account_logout().unwrap();
    client.close();

    let messages = fake.messages();
    let methods = messages
        .iter()
        .filter_map(|message| {
            message
                .get("id")
                .map(|_| message["method"].as_str().unwrap())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        methods,
        vec![
            "initialize",
            "thread/start",
            "thread/resume",
            "thread/list",
            "thread/read",
            "thread/fork",
            "thread/archive",
            "thread/unarchive",
            "thread/name/set",
            "thread/compact/start",
            "model/list",
            "account/read",
            "account/logout",
        ]
    );

    assert!(messages.iter().any(|message| {
        message["method"] == "thread/resume"
            && message["params"] == json!({"threadId":"thread-1", "model":"gpt-resume"})
    }));
    assert!(messages.iter().any(|message| {
        message["method"] == "thread/read"
            && message["params"] == json!({"threadId":"thread-1", "includeTurns": true})
    }));
    assert!(messages.iter().any(|message| {
        message["method"] == "model/list" && message["params"] == json!({"includeHidden": true})
    }));
}

#[test]
fn normalizes_app_server_inputs_and_receives_notifications() {
    let fake = FakeAppServer::new();
    let client = client_with_fake(&fake);

    client.initialize().unwrap();
    let result = client
        .turn_start(
            "thread-1",
            [
                AppInput::text("describe this"),
                AppInput::image("https://example.test/screenshot.png"),
                AppInput::local_image("./ui.png"),
                AppInput::skill("review", "skills/review/SKILL.md"),
                AppInput::mention("repo", "file:///repo"),
            ],
            Some(json!({"model":"gpt-turn", "outputSchema": {"type":"object"}})),
        )
        .unwrap();
    let notification = client.next_turn_notification("turn-1").unwrap();
    client
        .turn_steer("thread-1", "turn-1", "additional steering")
        .unwrap();
    client.turn_interrupt("thread-1", "turn-1").unwrap();
    client.close();

    assert_eq!(result["turn"]["id"], "turn-1");
    assert_eq!(notification.method, "turn/started");
    assert_eq!(notification.params["threadId"], "thread-1");

    let turn_start = fake
        .messages()
        .into_iter()
        .find(|message| message["method"] == "turn/start")
        .unwrap();
    assert_eq!(
        turn_start["params"],
        json!({
            "threadId": "thread-1",
            "model": "gpt-turn",
            "outputSchema": {"type": "object"},
            "input": [
                {"type":"text", "text":"describe this"},
                {"type":"image", "url":"https://example.test/screenshot.png"},
                {"type":"localImage", "path":"./ui.png"},
                {"type":"skill", "name":"review", "path":"skills/review/SKILL.md"},
                {"type":"mention", "name":"repo", "path":"file:///repo"}
            ]
        })
    );
}

#[test]
fn routes_scoped_turn_and_login_notifications_like_python_client() {
    let fake = FakeAppServer::new();
    let client = client_with_fake(&fake);

    client.initialize().unwrap();

    let turn = client.turn_start("thread-1", "hello", None).unwrap();
    let turn_notification = client.next_turn_notification("turn-1").unwrap();

    let login = client
        .account_login_start(Some(json!({"type": "chatgpt"})))
        .unwrap();
    let login_notification = client.next_login_notification("login-1").unwrap();

    client.close();

    assert_eq!(turn["turn"]["id"], "turn-1");
    assert_eq!(turn_notification.method, "turn/started");
    assert_eq!(turn_notification.params["turn"]["id"], "turn-1");
    assert_eq!(login["loginId"], "login-1");
    assert_eq!(login_notification.method, "account/login/completed");
    assert_eq!(login_notification.params["loginId"], "login-1");
}

#[test]
fn closes_transport_after_reader_failure_without_hanging_future_requests() {
    let fake = FakeAppServer::new();
    let client = client_with_fake_scenario(&fake, "invalid-json");

    let first = client.request("break-reader", None).unwrap_err();
    assert!(
        matches!(first, CodexError::Protocol(ref message) if message.contains("app-server reader failed")),
        "unexpected first error: {first:?}"
    );

    let started = std::time::Instant::now();
    let second = client.request("after-reader-failed", None).unwrap_err();
    assert!(matches!(second, CodexError::TransportClosed));
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "closed transport should fail immediately"
    );

    client.close();
}

#[test]
fn rejects_malformed_json_rpc_response_without_null_success() {
    let fake = FakeAppServer::new();
    let client = client_with_fake_scenario(&fake, "malformed-response");

    let err = client.request("malformed", None).unwrap_err();
    assert!(
        matches!(err, CodexError::Protocol(ref message) if message.contains("missing result or error")),
        "unexpected malformed response error: {err:?}"
    );

    let ok = client.request("thread/list", Some(json!({}))).unwrap();
    client.close();

    assert_eq!(ok["data"][0]["id"], "thread-1");
}

#[test]
fn responds_to_app_server_approval_requests_like_python_default_handler() {
    let fake = FakeAppServer::new();
    let client = client_with_fake_scenario(&fake, "server-request");

    client.initialize().unwrap();
    let turn = client
        .turn_start("thread-1", "approve command", None)
        .unwrap();
    client.close();

    assert_eq!(turn["turn"]["id"], "turn-1");
    assert!(fake.messages().iter().any(|message| {
        message == &json!({"id": "approval-1", "result": {"decision": "accept"}})
    }));
}

#[test]
fn accepts_file_change_approval_requests_like_python_default_handler() {
    let fake = FakeAppServer::new();
    let client = client_with_fake_scenario(&fake, "server-file-request");

    client.initialize().unwrap();
    client
        .turn_start("thread-1", "approve file change", None)
        .unwrap();
    client.close();

    assert!(fake.messages().iter().any(|message| {
        message == &json!({"id": "approval-1", "result": {"decision": "accept"}})
    }));
}

#[test]
fn supports_custom_app_server_request_handler() {
    let fake = FakeAppServer::new();
    let client = AppServerClient::new(
        AppServerConfig::new()
            .with_codex_path_override(&fake.path)
            .with_env(fake.env_with_scenario("server-request"))
            .with_server_request_handler(|method, params| {
                assert_eq!(method, "item/commandExecution/requestApproval");
                assert_eq!(params.unwrap()["turnId"], "turn-1");
                Ok(json!({"decision": "reject", "reason": "test policy"}))
            }),
    );

    client.initialize().unwrap();
    client
        .turn_start("thread-1", "reject command", None)
        .unwrap();
    client.close();

    assert!(fake.messages().iter().any(|message| {
        message
            == &json!({
                "id": "approval-1",
                "result": {"decision": "reject", "reason": "test policy"}
            })
    }));
}

#[test]
fn fake_app_server_is_written_exactly_once() {
    // Pins the mechanism, not just the outcome. Every test used to write its own executable and
    // then exec it, which races: a concurrent `Command::spawn` forks, inherits the in-flight write
    // descriptor, and the exec fails ETXTBSY ("Text file busy"). CI hit it.
    //
    // Sharing one write is what removes the window, so regressing to a per-fake write must fail
    // here rather than turn back into an intermittent CI red that reads as unrelated.
    let first = FakeAppServer::new();
    let second = FakeAppServer::new();

    assert_eq!(
        first.path, second.path,
        "every fake must exec the SAME executable; a per-fake copy reopens the ETXTBSY race"
    );
    assert_eq!(
        WRITES.load(Ordering::SeqCst),
        1,
        "the fake executable must be written exactly once per process, not once per fake"
    );

    // The per-test sinks stay distinct — sharing those would cross transcripts between tests.
    assert_ne!(
        first.messages_file, second.messages_file,
        "message sinks must remain per-fake"
    );
    assert_ne!(
        first.args_file, second.args_file,
        "arg sinks must remain per-fake"
    );
}
