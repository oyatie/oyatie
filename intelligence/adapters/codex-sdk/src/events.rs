use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Value;

use crate::items::ThreadItem;

/// Emitted when a new thread is started as the first event.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ThreadStartedEvent {
    pub thread_id: String,
}

/// Emitted when a turn is started.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnStartedEvent {}

/// Token usage reported at turn completion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
}

/// Emitted when a turn is completed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnCompletedEvent {
    pub usage: Usage,
}

/// Fatal turn error payload.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ThreadError {
    pub message: String,
}

/// Emitted when a turn fails.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnFailedEvent {
    pub error: ThreadError,
}

/// Emitted when a new item is added to the thread.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ItemStartedEvent {
    pub item: ThreadItem,
}

/// Emitted when an item is updated.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ItemUpdatedEvent {
    pub item: ThreadItem,
}

/// Emitted when an item reaches a terminal state.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ItemCompletedEvent {
    pub item: ThreadItem,
}

/// Top-level JSONL events emitted by `codex exec --experimental-json`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ThreadEvent {
    ThreadStarted(ThreadStartedEvent),
    TurnStarted(TurnStartedEvent),
    TurnCompleted(TurnCompletedEvent),
    TurnFailed(TurnFailedEvent),
    ItemStarted(ItemStartedEvent),
    ItemUpdated(ItemUpdatedEvent),
    ItemCompleted(ItemCompletedEvent),
    Error {
        message: String,
    },
    /// Forward-compatible fallback for upstream event types this SDK does not model yet.
    Unknown {
        raw: Value,
    },
}

impl<'de> Deserialize<'de> for ThreadEvent {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let event_type = value
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| de::Error::missing_field("type"))?;

        match event_type {
            "thread.started" => serde_json::from_value(value)
                .map(Self::ThreadStarted)
                .map_err(de::Error::custom),
            "turn.started" => serde_json::from_value(value)
                .map(Self::TurnStarted)
                .map_err(de::Error::custom),
            "turn.completed" => serde_json::from_value(value)
                .map(Self::TurnCompleted)
                .map_err(de::Error::custom),
            "turn.failed" => serde_json::from_value(value)
                .map(Self::TurnFailed)
                .map_err(de::Error::custom),
            "item.started" => serde_json::from_value(value)
                .map(Self::ItemStarted)
                .map_err(de::Error::custom),
            "item.updated" => serde_json::from_value(value)
                .map(Self::ItemUpdated)
                .map_err(de::Error::custom),
            "item.completed" => serde_json::from_value(value)
                .map(Self::ItemCompleted)
                .map_err(de::Error::custom),
            "error" => {
                #[derive(Deserialize)]
                struct ErrorEvent {
                    message: String,
                }

                let error =
                    serde_json::from_value::<ErrorEvent>(value).map_err(de::Error::custom)?;
                Ok(Self::Error {
                    message: error.message,
                })
            }
            _ => Ok(Self::Unknown { raw: value }),
        }
    }
}

impl Serialize for ThreadEvent {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::ThreadStarted(event) => serialize_with_type(serializer, "thread.started", event),
            Self::TurnStarted(event) => serialize_with_type(serializer, "turn.started", event),
            Self::TurnCompleted(event) => serialize_with_type(serializer, "turn.completed", event),
            Self::TurnFailed(event) => serialize_with_type(serializer, "turn.failed", event),
            Self::ItemStarted(event) => serialize_with_type(serializer, "item.started", event),
            Self::ItemUpdated(event) => serialize_with_type(serializer, "item.updated", event),
            Self::ItemCompleted(event) => serialize_with_type(serializer, "item.completed", event),
            Self::Error { message } => {
                let mut value = serde_json::Map::new();
                value.insert("type".to_string(), Value::String("error".to_string()));
                value.insert("message".to_string(), Value::String(message.clone()));
                value.serialize(serializer)
            }
            Self::Unknown { raw } => raw.serialize(serializer),
        }
    }
}

fn serialize_with_type<T, S>(
    serializer: S,
    event_type: &str,
    payload: &T,
) -> std::result::Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    let mut value = serde_json::to_value(payload).map_err(serde::ser::Error::custom)?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| serde::ser::Error::custom("event payload must serialize to an object"))?;
    object.insert("type".to_string(), Value::String(event_type.to_string()));
    value.serialize(serializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preserves_unknown_events_as_raw_json() {
        let raw = json!({"type":"future.event","payload":{"answer":42}});
        let event: ThreadEvent = serde_json::from_value(raw.clone()).unwrap();

        assert_eq!(event, ThreadEvent::Unknown { raw: raw.clone() });
        assert_eq!(serde_json::to_value(event).unwrap(), raw);
    }
}
