use intelligence_codex_sdk::{AppInput, AppServerClient, AppServerConfig, CodexError};
use serde_json::json;

fn main() -> intelligence_codex_sdk::Result<()> {
    let client = AppServerClient::new(AppServerConfig::default());
    let metadata = client.initialize()?;
    println!("connected to {:?}", metadata.server_info);

    let started = client.thread_start(Some(json!({"model": "gpt-test-1"})))?;
    let thread_id = started["thread"]["id"]
        .as_str()
        .ok_or_else(|| CodexError::Protocol("thread/start response missing thread.id".into()))?;

    let turn = client.turn_start(
        thread_id,
        [
            AppInput::text("Review the current repository state."),
            AppInput::mention("repo", "file:///workspace"),
        ],
        None,
    )?;
    println!("started turn: {turn}");
    client.close();
    Ok(())
}
