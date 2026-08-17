use futures::StreamExt;
use intelligence_claude_agent_sdk::{ClaudeAgentOptions, Message, query};

#[tokio::test]
#[ignore = "requires an authenticated Claude CLI and consumes live Claude usage"]
async fn live_claude_cli_query_smoke() {
    let mut stream = query(
        "Respond with exactly the token RUST_SDK_LIVE_SMOKE and no punctuation.",
        ClaudeAgentOptions::default(),
    )
    .expect("failed to spawn Claude CLI");

    let mut saw_expected_text = false;
    let mut saw_result = false;
    while let Some(message) = stream.next().await {
        match message.expect("live Claude CLI stream returned an error") {
            Message::Assistant(assistant) => {
                saw_expected_text |= assistant.content.iter().any(|block| {
                    matches!(block, intelligence_claude_agent_sdk::ContentBlock::Text { text } if text.trim() == "RUST_SDK_LIVE_SMOKE")
                });
            }
            Message::Result(result) => {
                saw_result = true;
                if let Some(text) = result.result.as_deref() {
                    saw_expected_text |= text.trim() == "RUST_SDK_LIVE_SMOKE";
                }
            }
            _ => {}
        }
    }

    assert!(saw_result, "live CLI stream ended without a result message");
    assert!(
        saw_expected_text,
        "live CLI response did not contain the expected smoke token"
    );
}
