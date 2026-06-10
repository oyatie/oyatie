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
