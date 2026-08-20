#[path = "support_fake_cli.rs"]
mod support;
use support::{expect_json_line, fake_cli, read_json_line, write_json_line};

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use futures::{StreamExt, stream};
use intelligence_claude_agent_sdk::{
    ClaudeAgentOptions, Message, UserMessage, startup, startup_with_timeout,
};
use serde_json::json;

#[tokio::test]
async fn startup_prewarms_before_prompt_and_streams_once_ready() {
    // The fake signals via a oneshot that it has sent the init response.
    let (init_done_tx, init_done_rx) = tokio::sync::oneshot::channel::<()>();
    let init_done_tx = Arc::new(Mutex::new(Some(init_done_tx)));

    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process({
            let init_done_tx = Arc::clone(&init_done_tx);
            fake_cli(move |mut r, mut w, _| {
                let init_done_tx = Arc::clone(&init_done_tx);
                async move {
                    let init = expect_json_line(&mut r).await;
                    assert_eq!(init["type"], "control_request");
                    assert_eq!(init["request"]["subtype"], "initialize");
                    write_json_line(&mut w, &json!({
                        "type":"control_response",
                        "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
                    })).await;
                    // Signal that initialization response was sent
                    if let Some(tx) = init_done_tx.lock().unwrap().take() {
                        let _ = tx.send(());
                    }
                    let user = expect_json_line(&mut r).await;
                    assert_eq!(user["type"], "user");
                    let content = user["message"]["content"].as_str().unwrap_or("").to_owned();
                    write_json_line(&mut w, &json!({
                        "type":"assistant","session_id":"session-warm",
                        "message":{"model":"claude-test","content":[{"type":"text","text":content}]}
                    })).await;
                    write_json_line(&mut w, &json!({
                        "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                        "is_error":false,"num_turns":1,"session_id":"session-warm","result":content
                    })).await;
                }
            })
        })
        .build();
    let mut warm = startup(options).await.unwrap();

    // startup() must have already received the init response before returning,
    // so the oneshot must be receivable without blocking.
    init_done_rx
        .await
        .expect("init response was not sent before startup() returned");

    let mut stream = warm.query("hello").unwrap();
    let first = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(first, Message::Assistant(message) if message.session_id.as_deref() == Some("session-warm"))
    );
    let second = stream.next().await.unwrap().unwrap();
    assert!(matches!(second, Message::Result(result) if result.result.as_deref() == Some("hello")));
}

#[tokio::test]
async fn warm_query_can_only_be_used_once() {
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({
                "type":"control_response",
                "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
            })).await;
            let user = expect_json_line(&mut r).await;
            let content = user["message"]["content"].as_str().unwrap_or("").to_owned();
            write_json_line(&mut w, &json!({
                "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                "is_error":false,"num_turns":1,"session_id":"session-once","result":content
            })).await;
        }))
        .build();
    let mut warm = startup(options).await.unwrap();
    let mut stream = warm.query("first").unwrap();

    let error = match warm.query("second") {
        Ok(_) => panic!("expected second warm query to fail"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("can only be called once"));

    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("first"))
    );
}

#[tokio::test]
async fn warm_query_accepts_streaming_user_messages() {
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({
                "type":"control_response",
                "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
            })).await;
            let first = expect_json_line(&mut r).await;
            let second = expect_json_line(&mut r).await;
            assert_eq!(first["message"]["content"], "warm context");
            assert_eq!(first["shouldQuery"], false);
            assert_eq!(second["message"]["content"], "warm prompt");
            let result_text = second["message"]["content"].as_str().unwrap_or("").to_owned();
            write_json_line(&mut w, &json!({
                "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                "is_error":false,"num_turns":1,"session_id":"session-warm-stream","result":result_text
            })).await;
        }))
        .build();
    let mut warm = startup(options).await.unwrap();
    let mut stream = warm
        .query_stream(stream::iter([
            UserMessage::text("warm context").should_query(false),
            UserMessage::text("warm prompt"),
        ]))
        .unwrap();

    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("warm prompt"))
    );
}

#[tokio::test]
async fn warm_query_close_closes_stdin_without_prompt() {
    let (closed_tx, closed_rx) = tokio::sync::oneshot::channel::<()>();
    let closed_tx = Arc::new(Mutex::new(Some(closed_tx)));

    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process({
            let closed_tx = Arc::clone(&closed_tx);
            fake_cli(move |mut r, mut w, _| {
                let closed_tx = Arc::clone(&closed_tx);
                async move {
                    let init = expect_json_line(&mut r).await;
                    write_json_line(&mut w, &json!({
                        "type":"control_response",
                        "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
                    })).await;
                    // Drain until EOF — SDK closes stdin on close()
                    while read_json_line(&mut r).await.is_some() {}
                    if let Some(tx) = closed_tx.lock().unwrap().take() {
                        let _ = tx.send(());
                    }
                }
            })
        })
        .build();
    let mut warm = startup(options).await.unwrap();

    warm.close().await.unwrap();

    // close() awaits the wait future which resolves after the fake task exits;
    // the fake task only exits after it drains EOF. So by the time close()
    // returns, the signal must already be sent.
    tokio::time::timeout(Duration::from_secs(1), closed_rx)
        .await
        .expect("stdin was not closed before close() returned")
        .unwrap();
}

#[tokio::test]
async fn startup_honors_initialize_timeout() {
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, w, _| async move {
            // Read the init line but never respond — simulates a hung CLI.
            let _init = expect_json_line(&mut r).await;
            // Hold the task open until the SDK drops us.
            while read_json_line(&mut r).await.is_some() {}
            drop(w);
        }))
        .build();
    let started = Instant::now();
    let error = match startup_with_timeout(options, Duration::from_millis(20)).await {
        Ok(_) => panic!("expected startup to time out"),
        Err(error) => error.to_string(),
    };

    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(error.contains("initialize"));
}
