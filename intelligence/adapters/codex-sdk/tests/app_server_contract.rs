#![cfg(unix)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use intelligence_codex_sdk::{
    AppCodex, AppInput, AppServerClient, AppServerConfig, CURRENT_APP_SERVER_REQUEST_METHODS,
    CURRENT_UPSTREAM_MAIN_SHA, CodexError,
};
use serde_json::{Value, json};
use tempfile::TempDir;

struct FakeAppServer {
    _dir: TempDir,
    path: PathBuf,
    messages_file: PathBuf,
    args_file: PathBuf,
}

impl FakeAppServer {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("codex");
        let messages_file = dir.path().join("messages.jsonl");
        let args_file = dir.path().join("args.txt");
        write_fake_app_server(&path);
        Self {
            _dir: dir,
            path,
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

fn write_fake_app_server(path: &Path) {
    let source_path = path.with_extension("rs");
    fs::write(&source_path, FAKE_APP_SERVER_RS).unwrap();

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = Command::new(rustc)
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "failed to compile Rust fake app server\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

const FAKE_APP_SERVER_RS: &str = r####"
use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::time::Duration;

fn write_message(path: &str, message: &str) {
    let mut file = OpenOptions::new().create(true).append(true).open(path).unwrap();
    writeln!(file, "{}", message).unwrap();
}

fn send(message: &str) {
    println!("{}", message);
    io::stdout().flush().unwrap();
}

fn extract_string_field(input: &str, field: &str) -> Option<String> {
    let key = format!("\"{}\"", field);
    let key_start = input.find(&key)? + key.len();
    let colon = input[key_start..].find(':')? + key_start;
    let rest = input[colon + 1..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_id(input: &str) -> Option<String> {
    let key = "\"id\"";
    let key_start = input.find(key)? + key.len();
    let colon = input[key_start..].find(':')? + key_start;
    let rest = input[colon + 1..].trim_start();
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn thread_id(input: &str) -> String {
    extract_string_field(input, "threadId").unwrap_or_else(|| "thread-1".to_string())
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn result_for(method: &str, message: &str, scenario: &str, reader: &mut impl BufRead) -> String {
    match method {
        "initialize" => r#"{"serverInfo":{"name":"codex-cli","version":"fake"},"userAgent":"codex-cli/fake","platformFamily":"unix","platformOs":"test"}"#.to_string(),
        "thread/start" | "thread/resume" | "thread/fork" | "thread/unarchive" | "thread/read" => {
            format!(r#"{{"thread":{{"id":{},"items":[],"turns":[]}}}}"#, json_string(&thread_id(message)))
        }
        "thread/list" => r#"{"data":[{"id":"thread-1","items":[],"turns":[]}],"nextCursor":null,"backwardsCursor":null}"#.to_string(),
        "thread/archive" | "thread/compact/start" | "thread/name/set" => "{}".to_string(),
        "turn/start" => {
            let tid = thread_id(message);
            if scenario == "server-request" || scenario == "server-file-request" {
                let approval_method = if scenario == "server-file-request" { "item/fileChange/requestApproval" } else { "item/commandExecution/requestApproval" };
                send(&format!(r#"{{"id":"approval-1","method":{},"params":{{"threadId":{},"turnId":"turn-1"}}}}"#, json_string(approval_method), json_string(&tid)));
                std::thread::sleep(Duration::from_millis(25));
                let mut approval = String::new();
                reader.read_line(&mut approval).unwrap();
                if let Ok(path) = env::var("CODEX_APP_MESSAGES_FILE") {
                    write_message(&path, approval.trim_end());
                }
            }
            send(&format!(r#"{{"method":"turn/started","params":{{"threadId":{},"turn":{{"id":"turn-1","status":"running","items":[]}}}}}}"#, json_string(&tid)));
            if scenario == "complete-run" {
                let final_item = r#"{"id":"item-final","type":"agentMessage","phase":"final_answer","text":"done from app api"}"#;
                send(&format!(r#"{{"method":"item/completed","params":{{"threadId":{},"turnId":"turn-1","completedAtMs":2000,"item":{}}}}}"#, json_string(&tid), final_item));
                send(&format!(r#"{{"method":"thread/tokenUsage/updated","params":{{"threadId":{},"turnId":"turn-1","tokenUsage":{{"inputTokens":11,"outputTokens":7,"totalTokens":18}}}}}}"#, json_string(&tid)));
                send(&format!(r#"{{"method":"turn/completed","params":{{"threadId":{},"turn":{{"id":"turn-1","status":"completed","startedAt":1,"completedAt":2,"durationMs":1000,"items":[{}]}}}}}}"#, json_string(&tid), final_item));
            }
            r#"{"turn":{"id":"turn-1","status":"running","items":[]}}"#.to_string()
        }
        "turn/steer" | "turn/interrupt" => r#"{"accepted":true}"#.to_string(),
        "model/list" => r#"{"data":[{"id":"gpt-test"}]}"#.to_string(),
        "account/read" => r#"{"account":null}"#.to_string(),
        "account/logout" => "{}".to_string(),
        "account/login/cancel" => r#"{"status":"cancelled"}"#.to_string(),
        "account/login/start" => {
            if message.contains(r#""type":"chatgpt""#) || message.contains(r#""type": "chatgpt""#) {
                send(r#"{"method":"account/login/completed","params":{"loginId":"login-1","success":true}}"#);
                r#"{"type":"chatgpt","loginId":"login-1","authUrl":"https://example.test/login"}"#.to_string()
            } else {
                r#"{"type":"apiKey"}"#.to_string()
            }
        }
        _ => r#"{"ok":true}"#.to_string(),
    }
}

fn main() {
    let messages_path = env::var("CODEX_APP_MESSAGES_FILE").unwrap();
    let args_path = env::var("CODEX_APP_ARGS_FILE").unwrap();
    let scenario = env::var("CODEX_APP_SCENARIO").unwrap_or_default();

    let mut args_file = std::fs::File::create(args_path).unwrap();
    for arg in env::args().skip(1) {
        writeln!(args_file, "{}", arg).unwrap();
    }

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut raw_line = String::new();
    while reader.read_line(&mut raw_line).unwrap() != 0 {
        let line = raw_line.trim_end().to_string();
        write_message(&messages_path, &line);
        let Some(id) = extract_id(&line) else { continue };
        let Some(method) = extract_string_field(&line, "method") else { continue };
        if scenario == "invalid-json" && method == "break-reader" {
            print!("{{invalid json\n");
            io::stdout().flush().unwrap();
            continue;
        }
        if scenario == "malformed-response" && method == "malformed" {
            send(&format!(r#"{{"id":{}}}"#, id));
            continue;
        }
        let result = result_for(&method, &line, &scenario, &mut reader);
        send(&format!(r#"{{"id":{},"result":{}}}"#, id, result));
        if method == "turn/start" {
            std::thread::sleep(Duration::from_millis(25));
            let tid = thread_id(&line);
            send(&format!(
                r#"{{"method":"turn/started","params":{{"threadId":{},"turn":{{"id":"turn-1","status":"running","items":[]}}}}}}"#,
                json_string(&tid)
            ));
            if scenario == "complete-run" {
                let final_item = r#"{"id":"item-final","type":"agentMessage","phase":"final_answer","text":"done from app api"}"#;
                send(&format!(
                    r#"{{"method":"item/completed","params":{{"threadId":{},"turnId":"turn-1","completedAtMs":2000,"item":{}}}}}"#,
                    json_string(&tid),
                    final_item
                ));
                send(&format!(
                    r#"{{"method":"thread/tokenUsage/updated","params":{{"threadId":{},"turnId":"turn-1","tokenUsage":{{"inputTokens":11,"outputTokens":7,"totalTokens":18}}}}}}"#,
                    json_string(&tid)
                ));
                send(&format!(
                    r#"{{"method":"turn/completed","params":{{"threadId":{},"turn":{{"id":"turn-1","status":"completed","startedAt":1,"completedAt":2,"durationMs":1000,"items":[{}]}}}}}}"#,
                    json_string(&tid),
                    final_item
                ));
            }
        }
        raw_line.clear();
    }
}
"####;

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
    eprintln!("DEBUG result={}", result);
    eprintln!("DEBUG messages={:?}", fake.messages());
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
