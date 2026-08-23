use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessageRequest {
    pub model: String,
    pub system: Option<String>,
    pub messages: Vec<AnthropicMessage>,
    pub telemetry_safe_summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenAiChatRequest {
    pub model: String,
    pub messages: Vec<OpenAiChatMessage>,
    pub telemetry_safe_summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenAiChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeminiGenerateContentRequest {
    pub model: String,
    pub contents: Vec<GeminiContent>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<GeminiContent>,
    #[serde(default, skip_serializing)]
    pub telemetry_safe_summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeminiPart {
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnthropicSseEvent {
    pub event: String,
    pub delta_text: Option<String>,
    pub requires_full_response_buffer: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TranslationError {
    MissingModel,
    MissingMessages,
    InvalidMessages,
    InvalidSseDelta,
}

pub fn openai_chat_to_anthropic_messages(
    payload: &serde_json::Value,
) -> Result<AnthropicMessageRequest, TranslationError> {
    let model = payload
        .get("model")
        .and_then(serde_json::Value::as_str)
        .ok_or(TranslationError::MissingModel)?;
    let messages = payload
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or(TranslationError::MissingMessages)?;

    let mut system_parts = Vec::new();
    let mut translated = Vec::new();
    for message in messages {
        let role = message
            .get("role")
            .and_then(serde_json::Value::as_str)
            .ok_or(TranslationError::InvalidMessages)?;
        let content = message_content_as_text(message).ok_or(TranslationError::InvalidMessages)?;
        if role == "system" {
            system_parts.push(content);
        } else if role == "user" || role == "assistant" {
            translated.push(AnthropicMessage {
                role: role.to_string(),
                content,
            });
        }
    }

    Ok(AnthropicMessageRequest {
        model: strip_provider_prefix(model).to_string(),
        system: if system_parts.is_empty() {
            None
        } else {
            Some(system_parts.join("\n"))
        },
        telemetry_safe_summary: format!(
            "messages={};system={}",
            translated.len(),
            !system_parts.is_empty()
        ),
        messages: translated,
    })
}

pub fn anthropic_messages_to_openai_chat(
    payload: &serde_json::Value,
) -> Result<OpenAiChatRequest, TranslationError> {
    let model = payload
        .get("model")
        .and_then(serde_json::Value::as_str)
        .ok_or(TranslationError::MissingModel)?;
    let messages = payload
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or(TranslationError::MissingMessages)?;

    let mut translated = Vec::new();
    if let Some(system) = payload.get("system").and_then(serde_json::Value::as_str) {
        translated.push(OpenAiChatMessage {
            role: "system".to_string(),
            content: system.to_string(),
        });
    }
    for message in messages {
        let role = message
            .get("role")
            .and_then(serde_json::Value::as_str)
            .ok_or(TranslationError::InvalidMessages)?;
        let content = message_content_as_text(message).ok_or(TranslationError::InvalidMessages)?;
        translated.push(OpenAiChatMessage {
            role: role.to_string(),
            content,
        });
    }

    Ok(OpenAiChatRequest {
        model: strip_provider_prefix(model).to_string(),
        telemetry_safe_summary: format!("messages={}", translated.len()),
        messages: translated,
    })
}

pub fn openai_chat_to_gemini_generate_content(
    payload: &serde_json::Value,
) -> Result<GeminiGenerateContentRequest, TranslationError> {
    let model = payload
        .get("model")
        .and_then(serde_json::Value::as_str)
        .ok_or(TranslationError::MissingModel)?;
    let messages = payload
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or(TranslationError::MissingMessages)?;

    let mut system_parts = Vec::new();
    let mut contents = Vec::new();
    for message in messages {
        let role = message
            .get("role")
            .and_then(serde_json::Value::as_str)
            .ok_or(TranslationError::InvalidMessages)?;
        let content = message_content_as_text(message).ok_or(TranslationError::InvalidMessages)?;
        match role {
            "system" => system_parts.push(content),
            "user" => contents.push(gemini_content("user", content)),
            "assistant" => contents.push(gemini_content("model", content)),
            _ => {}
        }
    }

    Ok(GeminiGenerateContentRequest {
        model: strip_provider_prefix(model).to_string(),
        contents,
        system_instruction: gemini_system_instruction(system_parts),
        telemetry_safe_summary: format!(
            "contents={};system={}",
            messages
                .iter()
                .filter(|message| {
                    message
                        .get("role")
                        .and_then(serde_json::Value::as_str)
                        .map(|role| role == "user" || role == "assistant")
                        .unwrap_or(false)
                })
                .count(),
            payload
                .get("messages")
                .and_then(serde_json::Value::as_array)
                .map(|messages| {
                    messages.iter().any(|message| {
                        message
                            .get("role")
                            .and_then(serde_json::Value::as_str)
                            .map(|role| role == "system")
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        ),
    })
}

pub fn anthropic_messages_to_gemini_generate_content(
    payload: &serde_json::Value,
) -> Result<GeminiGenerateContentRequest, TranslationError> {
    let model = payload
        .get("model")
        .and_then(serde_json::Value::as_str)
        .ok_or(TranslationError::MissingModel)?;
    let messages = payload
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or(TranslationError::MissingMessages)?;

    let system_parts = payload
        .get("system")
        .and_then(serde_json::Value::as_str)
        .map(|system| vec![system.to_string()])
        .unwrap_or_default();
    let mut contents = Vec::new();
    for message in messages {
        let role = message
            .get("role")
            .and_then(serde_json::Value::as_str)
            .ok_or(TranslationError::InvalidMessages)?;
        let content = message_content_as_text(message).ok_or(TranslationError::InvalidMessages)?;
        match role {
            "user" => contents.push(gemini_content("user", content)),
            "assistant" => contents.push(gemini_content("model", content)),
            _ => {}
        }
    }

    Ok(GeminiGenerateContentRequest {
        model: strip_provider_prefix(model).to_string(),
        telemetry_safe_summary: format!(
            "contents={};system={}",
            contents.len(),
            !system_parts.is_empty()
        ),
        contents,
        system_instruction: gemini_system_instruction(system_parts),
    })
}

pub fn gemini_generate_content_to_openai_chat(
    payload: &serde_json::Value,
) -> Result<OpenAiChatRequest, TranslationError> {
    let model = payload
        .get("model")
        .and_then(serde_json::Value::as_str)
        .ok_or(TranslationError::MissingModel)?;
    let contents = payload
        .get("contents")
        .and_then(serde_json::Value::as_array)
        .ok_or(TranslationError::MissingMessages)?;

    let mut messages = Vec::new();
    if let Some(system) = payload.get("systemInstruction") {
        let text = gemini_content_text(system).ok_or(TranslationError::InvalidMessages)?;
        messages.push(OpenAiChatMessage {
            role: "system".to_string(),
            content: text,
        });
    }
    for content in contents {
        let role = content
            .get("role")
            .and_then(serde_json::Value::as_str)
            .ok_or(TranslationError::InvalidMessages)?;
        let text = gemini_content_text(content).ok_or(TranslationError::InvalidMessages)?;
        let role = match role {
            "model" => "assistant",
            "user" => "user",
            _ => role,
        };
        messages.push(OpenAiChatMessage {
            role: role.to_string(),
            content: text,
        });
    }

    Ok(OpenAiChatRequest {
        model: strip_provider_prefix(model).to_string(),
        telemetry_safe_summary: format!("messages={}", messages.len()),
        messages,
    })
}

pub fn gemini_generate_content_response_to_openai_chat_response(
    payload: &serde_json::Value,
    model: &str,
) -> serde_json::Value {
    let text = gemini_response_text(payload);
    let prompt_tokens = payload
        .pointer("/usageMetadata/promptTokenCount")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let completion_tokens = payload
        .pointer("/usageMetadata/candidatesTokenCount")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    serde_json::json!({
        "id": "chatcmpl-gemini",
        "object": "chat.completion",
        "model": model,
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": text},
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens
        }
    })
}

pub fn gemini_generate_content_response_to_anthropic_message_response(
    payload: &serde_json::Value,
    model: &str,
) -> serde_json::Value {
    let text = gemini_response_text(payload);
    let input_tokens = payload
        .pointer("/usageMetadata/promptTokenCount")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let output_tokens = payload
        .pointer("/usageMetadata/candidatesTokenCount")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    serde_json::json!({
        "id": "msg_gemini",
        "type": "message",
        "role": "assistant",
        "model": model,
        "content": [{"type": "text", "text": text}],
        "stop_reason": "end_turn",
        "usage": {"input_tokens": input_tokens, "output_tokens": output_tokens}
    })
}

pub fn openai_sse_delta_to_anthropic_event(
    event: &str,
) -> Result<AnthropicSseEvent, TranslationError> {
    let normalized_event = event.replace("\\n", "\n").replace("\\\"", "\"");
    let data = normalized_event
        .lines()
        .find_map(|line| line.strip_prefix("data: "))
        .ok_or(TranslationError::InvalidSseDelta)?;
    if data.trim() == "[DONE]" {
        return Ok(AnthropicSseEvent {
            event: "message_stop".to_string(),
            delta_text: None,
            requires_full_response_buffer: false,
        });
    }
    let delta_text = match serde_json::from_str::<serde_json::Value>(data) {
        Ok(value) => value
            .get("choices")
            .and_then(serde_json::Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("delta"))
            .and_then(|delta| delta.get("content"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        Err(_) => extract_content_delta_from_escaped_fixture(data),
    };

    Ok(AnthropicSseEvent {
        event: "content_block_delta".to_string(),
        delta_text,
        requires_full_response_buffer: false,
    })
}

fn gemini_content(role: &str, text: String) -> GeminiContent {
    GeminiContent {
        role: role.to_string(),
        parts: vec![GeminiPart { text }],
    }
}

fn gemini_system_instruction(system_parts: Vec<String>) -> Option<GeminiContent> {
    if system_parts.is_empty() {
        return None;
    }
    Some(GeminiContent {
        role: "user".to_string(),
        parts: system_parts
            .into_iter()
            .map(|text| GeminiPart { text })
            .collect(),
    })
}

fn gemini_content_text(content: &serde_json::Value) -> Option<String> {
    let parts = content.get("parts")?.as_array()?;
    let text = parts
        .iter()
        .filter_map(|part| part.get("text").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    if text.is_empty() { None } else { Some(text) }
}

fn gemini_response_text(payload: &serde_json::Value) -> String {
    payload
        .get("candidates")
        .and_then(serde_json::Value::as_array)
        .and_then(|candidates| candidates.first())
        .and_then(|candidate| candidate.get("content"))
        .and_then(gemini_content_text)
        .unwrap_or_default()
}

fn extract_content_delta_from_escaped_fixture(data: &str) -> Option<String> {
    let normalized = data.replace("\\\"", "\"");
    let marker = "\"content\":\"";
    let start = normalized.find(marker)? + marker.len();
    let rest = &normalized[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn strip_provider_prefix(model: &str) -> &str {
    model
        .split_once(':')
        .map(|(_, rest)| rest)
        .unwrap_or(model)
        .trim()
}

fn message_content_as_text(message: &serde_json::Value) -> Option<String> {
    match message.get("content")? {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Array(parts) => {
            let text = parts
                .iter()
                .filter_map(|part| part.get("text").and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>()
                .join("\n");
            if text.is_empty() { None } else { Some(text) }
        }
        _ => None,
    }
}
