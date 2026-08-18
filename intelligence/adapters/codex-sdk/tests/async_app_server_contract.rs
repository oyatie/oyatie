#![cfg(all(unix, feature = "async"))]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use intelligence_codex_sdk::{AppServerConfig, AsyncAppCodex};
use serde_json::{Value, json};
use std::sync::OnceLock;

use tempfile::TempDir;

use std::os::unix::fs::PermissionsExt;

/// The fake executable, written EXACTLY ONCE per test process.
///
/// Each test used to write its own copy and then exec it. That races: `Command::spawn` forks
/// before it execs, and the forked child inherits every open descriptor — `CLOEXEC` only clears
/// them AT exec, so a write descriptor stays open across the child's whole fork→exec window. One
/// test writing while another forked meant the writer's own exec hit ETXTBSY ("Text file busy").
/// CI hit exactly that in the sibling app-server contract suite.
///
/// Writing once behind a `OnceLock` removes the window structurally rather than by retrying: every
/// test here must obtain this path before it can spawn, so none can be forking during the single
/// write. Other test binaries are separate processes and cannot inherit these descriptors.
///
/// The `TempDir` is deliberately leaked — it must outlive every test, and a `static` is never
/// dropped, so binding it would only leave a dangling path.
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
        // The executable is shared and written once; the per-test sinks stay per-fake.
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

    fn messages(&self) -> Vec<Value> {
        fs::read_to_string(&self.messages_file)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

fn write_fake_app_server(path: &Path) {
    fs::write(
        path,
        r#"#!/usr/bin/env python3
import json
import os
import sys

messages_path = os.environ["CODEX_APP_MESSAGES_FILE"]
args_path = os.environ["CODEX_APP_ARGS_FILE"]

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
        return {"serverInfo": {"name": "codex-cli", "version": "fake"}, "userAgent": "codex-cli/fake"}
    if method in ("thread/start", "thread/resume", "thread/fork", "thread/unarchive", "thread/read"):
        return {"thread": {"id": params.get("threadId", "thread-1"), "items": [], "turns": []}}
    if method == "thread/list":
        return {"data": [{"id": "thread-1"}], "nextCursor": None, "backwardsCursor": None}
    if method in ("thread/archive", "thread/compact/start", "thread/name/set"):
        return {}
    if method == "account/read":
        return {"account": None}
    if method == "account/logout":
        return {}
    if method == "account/login/cancel":
        return {"status": "cancelled"}
    if method == "account/login/start":
        if params.get("type") == "chatgptDeviceCode":
            send({"method": "account/login/completed", "params": {"loginId": "login-device", "success": True}})
            return {"type": "chatgptDeviceCode", "loginId": "login-device", "verificationUrl": "https://example.test/device", "userCode": "ABCD-EFGH"}
        return {"type": "apiKey"}
    if method == "model/list":
        return {"data": [{"id": "gpt-test"}]}
    if method == "turn/start":
        final_item = {"id": "item-final", "type": "agentMessage", "phase": "final_answer", "text": "async done"}
        send({"method": "turn/started", "params": {"threadId": params["threadId"], "turn": {"id": "turn-1", "status": "running", "items": []}}})
        send({"method": "item/completed", "params": {"threadId": params["threadId"], "turnId": "turn-1", "completedAtMs": 2000, "item": final_item}})
        send({"method": "thread/tokenUsage/updated", "params": {"threadId": params["threadId"], "turnId": "turn-1", "tokenUsage": {"totalTokens": 3}}})
        send({"method": "turn/completed", "params": {"threadId": params["threadId"], "turn": {"id": "turn-1", "status": "completed", "durationMs": 42, "items": [final_item]}}})
        return {"turn": {"id": "turn-1", "status": "running", "items": []}}
    if method == "turn/steer":
        return {"accepted": True}
    if method == "turn/interrupt":
        return {"accepted": True}
    return {"ok": True}

for line in sys.stdin:
    message = json.loads(line)
    write_message(message)
    if "id" not in message or "method" not in message:
        continue
    params = message.get("params") or {}
    send({"id": message["id"], "result": result_for(message["method"], params)})
"#,
    )
    .unwrap();

    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

#[test]
fn async_app_codex_runs_thread_and_collects_result() {
    let fake = FakeAppServer::new();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let codex = AsyncAppCodex::new(
            AppServerConfig::new()
                .with_codex_path_override(&fake.path)
                .with_env(fake.env()),
        )
        .await
        .unwrap();

        assert_eq!(
            codex.metadata().user_agent.as_deref(),
            Some("codex-cli/fake")
        );
        codex.login_api_key("sk-test").await.unwrap();
        assert_eq!(codex.account(false).await.unwrap()["account"], Value::Null);
        assert_eq!(
            codex.models(false).await.unwrap()["data"][0]["id"],
            "gpt-test"
        );

        let login = codex.login_chatgpt_device_code().await.unwrap();
        assert_eq!(login.login_id(), "login-device");
        assert_eq!(
            login.verification_url(),
            Some("https://example.test/device")
        );
        assert_eq!(login.user_code(), Some("ABCD-EFGH"));
        assert_eq!(
            login.wait().await.unwrap().method,
            "account/login/completed"
        );

        let thread = codex
            .thread_start(Some(json!({"model": "gpt-test"})))
            .await
            .unwrap();
        assert_eq!(thread.id(), "thread-1");
        let result = thread
            .run("async hello", Some(json!({"model": "gpt-test"})))
            .await
            .unwrap();
        assert_eq!(result.status.as_deref(), Some("completed"));
        assert_eq!(result.final_response.as_deref(), Some("async done"));
        assert_eq!(result.usage.unwrap()["totalTokens"], 3);

        thread.set_name("Async Rust SDK").await.unwrap();
        thread.compact().await.unwrap();
        codex.close().await;
    });

    let messages = fake.messages();
    assert!(messages.iter().any(|message| {
        message["method"] == "turn/start"
            && message["params"]["input"] == json!([{"type": "text", "text": "async hello"}])
    }));
}

#[test]
fn async_turn_handle_streams_and_controls_turns() {
    let fake = FakeAppServer::new();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let codex = AsyncAppCodex::new(
            AppServerConfig::new()
                .with_codex_path_override(&fake.path)
                .with_env(fake.env()),
        )
        .await
        .unwrap();
        let thread = codex.thread_start(None).await.unwrap();
        let turn = thread.turn("stream me", None).await.unwrap();

        let mut stream = turn.stream();
        assert_eq!(stream.next().await.unwrap().unwrap().method, "turn/started");
        assert_eq!(
            stream.next().await.unwrap().unwrap().method,
            "item/completed"
        );
        assert_eq!(
            stream.next().await.unwrap().unwrap().method,
            "thread/tokenUsage/updated"
        );
        assert_eq!(
            stream.next().await.unwrap().unwrap().method,
            "turn/completed"
        );
        assert!(stream.next().await.is_none());

        assert_eq!(turn.steer("more").await.unwrap()["accepted"], true);
        assert_eq!(turn.interrupt().await.unwrap()["accepted"], true);
        codex.close().await;
    });
}
