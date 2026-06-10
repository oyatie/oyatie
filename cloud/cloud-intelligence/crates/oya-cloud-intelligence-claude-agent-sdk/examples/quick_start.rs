use oya_cloud_intelligence_claude_agent_sdk::{ClaudeAgentOptions, Message, query};
use futures::StreamExt;

#[tokio::main]
async fn main() -> oya_cloud_intelligence_claude_agent_sdk::Result<()> {
    let mut stream = query(
        "What files are in this directory?",
        ClaudeAgentOptions::builder().allowed_tool("Bash").build(),
    )?;

    while let Some(message) = stream.next().await.transpose()? {
        match message {
            Message::Assistant(assistant) => println!("assistant: {assistant:?}"),
            Message::Result(result) => println!("result: {}", result.result.unwrap_or_default()),
            other => println!("{other:?}"),
        }
    }
    Ok(())
}
