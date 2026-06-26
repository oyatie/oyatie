#![cfg(all(unix, feature = "async"))]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use intelligence_codex_sdk::{AppServerConfig, AsyncAppCodex};
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

    fn messages(&self) -> Vec<Value> {
        fs::read_to_string(&self.messages_file)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
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
use std::io::{self, BufRead, Write};

fn write_message(path: &str, message: &str) {
    let mut file = OpenOptions::new().create(true).append(true).open(path).unwrap();
    writeln!(file, "{}", message).unwrap();
}

fn send(message: &str) {
    println!("{}", message);
    io::stdout().flush().unwrap();
}

fn extract_string_field(input: &str, field: &str) -> Option<String> {
    let marker = format!("\"{}\":", field);
    let start = input.find(&marker)? + marker.len();
    let rest = input[start..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_id(input: &str) -> Option<String> {
    let marker = "\"id\":";
    let start = input.find(marker)? + marker.len();
    let rest = input[start..].trim_start();
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn thread_id(input: &str) -> String {
    extract_string_field(input, "threadId").unwrap_or_else(|| "thread-1".to_string())
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn result_for(method: &str, message: &str) -> String {
    match method {
        "initialize" => r#"{"serverInfo":{"name":"codex-cli","version":"fake"},"userAgent":"codex-cli/fake"}"#.to_string(),
        "thread/start" | "thread/resume" | "thread/fork" | "thread/unarchive" | "thread/read" => {
            format!(r#"{{"thread":{{"id":{},"items":[],"turns":[]}}}}"#, json_string(&thread_id(message)))
        }
        "thread/list" => r#"{"data":[{"id":"thread-1"}],"nextCursor":null,"backwardsCursor":null}"#.to_string(),
        "thread/archive" | "thread/compact/start" | "thread/name/set" => "{}".to_string(),
        "account/read" => r#"{"account":null}"#.to_string(),
        "account/logout" => "{}".to_string(),
        "account/login/cancel" => r#"{"status":"cancelled"}"#.to_string(),
        "account/login/start" => {
            if message.contains(r#""type":"chatgptDeviceCode""#) || message.contains(r#""type": "chatgptDeviceCode""#) {
                send(r#"{"method":"account/login/completed","params":{"loginId":"login-device","success":true}}"#);
                r#"{"type":"chatgptDeviceCode","loginId":"login-device","verificationUrl":"https://example.test/device","userCode":"ABCD-EFGH"}"#.to_string()
            } else {
                r#"{"type":"apiKey"}"#.to_string()
            }
        }
        "model/list" => r#"{"data":[{"id":"gpt-test"}]}"#.to_string(),
        "turn/start" => {
            let tid = thread_id(message);
            let final_item = r#"{"id":"item-final","type":"agentMessage","phase":"final_answer","text":"async done"}"#;
            send(&format!(r#"{{"method":"turn/started","params":{{"threadId":{},"turn":{{"id":"turn-1","status":"running","items":[]}}}}}}"#, json_string(&tid)));
            send(&format!(r#"{{"method":"item/completed","params":{{"threadId":{},"turnId":"turn-1","completedAtMs":2000,"item":{}}}}}"#, json_string(&tid), final_item));
            send(&format!(r#"{{"method":"thread/tokenUsage/updated","params":{{"threadId":{},"turnId":"turn-1","tokenUsage":{{"totalTokens":3}}}}}}"#, json_string(&tid)));
            send(&format!(r#"{{"method":"turn/completed","params":{{"threadId":{},"turn":{{"id":"turn-1","status":"completed","durationMs":42,"items":[{}]}}}}}}"#, json_string(&tid), final_item));
            r#"{"turn":{"id":"turn-1","status":"running","items":[]}}"#.to_string()
        }
        "turn/steer" | "turn/interrupt" => r#"{"accepted":true}"#.to_string(),
        _ => r#"{"ok":true}"#.to_string(),
    }
}

fn main() {
    let messages_path = env::var("CODEX_APP_MESSAGES_FILE").unwrap();
    let args_path = env::var("CODEX_APP_ARGS_FILE").unwrap();

    let mut args_file = std::fs::File::create(args_path).unwrap();
    for arg in env::args().skip(1) {
        writeln!(args_file, "{}", arg).unwrap();
    }

    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        write_message(&messages_path, &line);
        let Some(id) = extract_id(&line) else { continue };
        let Some(method) = extract_string_field(&line, "method") else { continue };
        let result = result_for(&method, &line);
        send(&format!(r#"{{"id":{},"result":{}}}"#, id, result));
    }
}
"####;

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
