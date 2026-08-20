#![allow(clippy::expect_used, clippy::panic)]

use intelligence_translation_kernel::{
    AnthropicMessageRequest, GeminiGenerateContentRequest, OpenAiChatRequest,
    anthropic_messages_to_gemini_generate_content, anthropic_messages_to_openai_chat,
    gemini_generate_content_response_to_anthropic_message_response,
    gemini_generate_content_response_to_openai_chat_response,
    gemini_generate_content_to_openai_chat, openai_chat_to_anthropic_messages,
    openai_chat_to_gemini_generate_content, openai_sse_delta_to_anthropic_event,
};

#[test]
fn openai_chat_to_anthropic_preserves_messages_and_system_without_raw_logging_fields() {
    let input = serde_json::json!({
        "model": "claude:opus",
        "messages": [
            {"role": "system", "content": "Be concise"},
            {"role": "user", "content": "hello"},
            {"role": "assistant", "content": "hi"}
        ],
        "stream": false
    });

    let translated: AnthropicMessageRequest = openai_chat_to_anthropic_messages(&input)
        .expect("OpenAI chat should translate to Anthropic messages");

    assert_eq!(translated.model, "opus");
    assert_eq!(translated.system.as_deref(), Some("Be concise"));
    assert_eq!(translated.messages.len(), 2);
    assert_eq!(translated.messages[0].role, "user");
    assert_eq!(translated.messages[0].content, "hello");
    assert!(!translated.telemetry_safe_summary.contains("hello"));
}

#[test]
fn anthropic_messages_to_openai_chat_preserves_model_messages_and_system_role() {
    let input = serde_json::json!({
        "model": "openai:gpt-4o",
        "system": "Use JSON",
        "messages": [
            {"role": "user", "content": "summarize"},
            {"role": "assistant", "content": "{}"}
        ]
    });

    let translated: OpenAiChatRequest = anthropic_messages_to_openai_chat(&input)
        .expect("Anthropic message should translate to OpenAI chat");

    assert_eq!(translated.model, "gpt-4o");
    assert_eq!(translated.messages.len(), 3);
    assert_eq!(translated.messages[0].role, "system");
    assert_eq!(translated.messages[0].content, "Use JSON");
    assert_eq!(translated.messages[1].role, "user");
}

#[test]
fn openai_sse_delta_fixture_translates_without_buffering_full_response() {
    let event = openai_sse_delta_to_anthropic_event(
        r#"data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\n"#,
    )
    .expect("SSE delta should translate");

    assert_eq!(event.event, "content_block_delta");
    assert_eq!(event.delta_text.as_deref(), Some("hi"));
    assert!(!event.requires_full_response_buffer);
}

#[test]
fn openai_chat_to_gemini_generate_content_maps_roles_and_system_instruction_without_raw_logging() {
    let input = serde_json::json!({
        "model": "gemini:gemini-2.5-flash",
        "messages": [
            {"role": "system", "content": "Use terse JSON"},
            {"role": "user", "content": "hello"},
            {"role": "assistant", "content": "hi"}
        ]
    });

    let translated: GeminiGenerateContentRequest = openai_chat_to_gemini_generate_content(&input)
        .expect("OpenAI chat should translate to Gemini generateContent");

    assert_eq!(translated.model, "gemini-2.5-flash");
    assert_eq!(
        translated.system_instruction.as_ref().map(|instruction| {
            instruction
                .parts
                .iter()
                .map(|part| part.text.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        }),
        Some("Use terse JSON".to_string())
    );
    assert_eq!(translated.contents.len(), 2);
    assert_eq!(translated.contents[0].role, "user");
    assert_eq!(translated.contents[0].parts[0].text, "hello");
    assert_eq!(translated.contents[1].role, "model");
    assert_eq!(translated.contents[1].parts[0].text, "hi");
    assert!(!translated.telemetry_safe_summary.contains("hello"));
}

#[test]
fn anthropic_messages_to_gemini_generate_content_maps_assistant_to_model_role() {
    let input = serde_json::json!({
        "model": "google:gemini-2.5-pro",
        "system": "Be accurate",
        "messages": [
            {"role": "user", "content": "summarize"},
            {"role": "assistant", "content": "summary"}
        ]
    });

    let translated: GeminiGenerateContentRequest =
        anthropic_messages_to_gemini_generate_content(&input)
            .expect("Anthropic messages should translate to Gemini generateContent");

    assert_eq!(translated.model, "gemini-2.5-pro");
    assert_eq!(translated.contents[0].role, "user");
    assert_eq!(translated.contents[1].role, "model");
    assert_eq!(translated.telemetry_safe_summary, "contents=2;system=true");
}

#[test]
fn gemini_generate_content_to_openai_chat_maps_model_role_back_to_assistant() {
    let input = serde_json::json!({
        "model": "gemini-2.5-flash",
        "systemInstruction": {"parts":[{"text":"Use JSON"}]},
        "contents": [
            {"role": "user", "parts": [{"text": "hello"}]},
            {"role": "model", "parts": [{"text": "hi"}]}
        ]
    });

    let translated: OpenAiChatRequest = gemini_generate_content_to_openai_chat(&input)
        .expect("Gemini generateContent should translate to OpenAI chat");

    assert_eq!(translated.model, "gemini-2.5-flash");
    assert_eq!(translated.messages[0].role, "system");
    assert_eq!(translated.messages[1].role, "user");
    assert_eq!(translated.messages[2].role, "assistant");
}

#[test]
fn gemini_generate_content_response_maps_to_openai_and_anthropic_contracts() {
    let input = serde_json::json!({
        "candidates": [{"content": {"role": "model", "parts": [{"text": "ok"}]}}],
        "usageMetadata": {"promptTokenCount": 2, "candidatesTokenCount": 3}
    });

    let openai =
        gemini_generate_content_response_to_openai_chat_response(&input, "gemini-2.5-flash");
    assert_eq!(openai["choices"][0]["message"]["role"], "assistant");
    assert_eq!(openai["choices"][0]["message"]["content"], "ok");
    assert_eq!(openai["usage"]["total_tokens"], 5);

    let anthropic =
        gemini_generate_content_response_to_anthropic_message_response(&input, "gemini-2.5-flash");
    assert_eq!(anthropic["role"], "assistant");
    assert_eq!(anthropic["content"][0]["text"], "ok");
    assert_eq!(anthropic["usage"]["input_tokens"], 2);
}
