use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Value;

macro_rules! wire_string_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $wire:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant,)+
            Unknown(String),
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Ok(match value.as_str() {
                    $($wire => Self::$variant,)+
                    _ => Self::Unknown(value),
                })
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                match self {
                    $(Self::$variant => serializer.serialize_str($wire),)+
                    Self::Unknown(value) => serializer.serialize_str(value),
                }
            }
        }
    };
}

wire_string_enum! {
    /// The status of a command execution.
    CommandExecutionStatus {
        InProgress => "in_progress",
        Completed => "completed",
        Failed => "failed",
    }
}

/// A command executed by the agent.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CommandExecutionItem {
    pub id: String,
    pub command: String,
    pub aggregated_output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub status: CommandExecutionStatus,
}

wire_string_enum! {
    /// Indicates the type of a file change.
    PatchChangeKind {
        Add => "add",
        Delete => "delete",
        Update => "update",
    }
}

/// A single file update within a patch.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FileUpdateChange {
    pub path: String,
    pub kind: PatchChangeKind,
}

wire_string_enum! {
    /// The status of an applied patch.
    PatchApplyStatus {
        Completed => "completed",
        Failed => "failed",
    }
}

/// A set of file changes by the agent.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FileChangeItem {
    pub id: String,
    pub changes: Vec<FileUpdateChange>,
    pub status: PatchApplyStatus,
}

wire_string_enum! {
    /// The status of an MCP tool call.
    McpToolCallStatus {
        InProgress => "in_progress",
        Completed => "completed",
        Failed => "failed",
    }
}

/// Result payload returned by an MCP tool call.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct McpToolCallResult {
    #[serde(default)]
    pub content: Vec<Value>,
    #[serde(default, rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,
}

/// Error payload returned by a failed MCP tool call.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McpToolCallError {
    pub message: String,
}

/// Represents a call to an MCP tool.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct McpToolCallItem {
    pub id: String,
    pub server: String,
    pub tool: String,
    pub arguments: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<McpToolCallResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<McpToolCallError>,
    pub status: McpToolCallStatus,
}

/// Response from the agent.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentMessageItem {
    pub id: String,
    pub text: String,
}

/// Agent's reasoning summary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReasoningItem {
    pub id: String,
    pub text: String,
}

/// Captures a web search request.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebSearchItem {
    pub id: String,
    pub query: String,
}

/// Describes a non-fatal error surfaced as an item.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ErrorItem {
    pub id: String,
    pub message: String,
}

/// An item in the agent's to-do list.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TodoItem {
    pub text: String,
    pub completed: bool,
}

/// Tracks the agent's running to-do list.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TodoListItem {
    pub id: String,
    pub items: Vec<TodoItem>,
}

/// Canonical union of Codex thread items emitted by `codex exec --experimental-json`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ThreadItem {
    AgentMessage(AgentMessageItem),
    Reasoning(ReasoningItem),
    CommandExecution(CommandExecutionItem),
    FileChange(FileChangeItem),
    McpToolCall(McpToolCallItem),
    WebSearch(WebSearchItem),
    TodoList(TodoListItem),
    Error(ErrorItem),
    /// Forward-compatible fallback for upstream item types this SDK does not model yet.
    Unknown {
        raw: Value,
    },
}

impl<'de> Deserialize<'de> for ThreadItem {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let item_type = value
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| de::Error::missing_field("type"))?;

        match item_type {
            "agent_message" => serde_json::from_value(value)
                .map(Self::AgentMessage)
                .map_err(de::Error::custom),
            "reasoning" => serde_json::from_value(value)
                .map(Self::Reasoning)
                .map_err(de::Error::custom),
            "command_execution" => serde_json::from_value(value)
                .map(Self::CommandExecution)
                .map_err(de::Error::custom),
            "file_change" => serde_json::from_value(value)
                .map(Self::FileChange)
                .map_err(de::Error::custom),
            "mcp_tool_call" => serde_json::from_value(value)
                .map(Self::McpToolCall)
                .map_err(de::Error::custom),
            "web_search" => serde_json::from_value(value)
                .map(Self::WebSearch)
                .map_err(de::Error::custom),
            "todo_list" => serde_json::from_value(value)
                .map(Self::TodoList)
                .map_err(de::Error::custom),
            "error" => serde_json::from_value(value)
                .map(Self::Error)
                .map_err(de::Error::custom),
            _ => Ok(Self::Unknown { raw: value }),
        }
    }
}

impl Serialize for ThreadItem {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::AgentMessage(item) => serialize_with_type(serializer, "agent_message", item),
            Self::Reasoning(item) => serialize_with_type(serializer, "reasoning", item),
            Self::CommandExecution(item) => {
                serialize_with_type(serializer, "command_execution", item)
            }
            Self::FileChange(item) => serialize_with_type(serializer, "file_change", item),
            Self::McpToolCall(item) => serialize_with_type(serializer, "mcp_tool_call", item),
            Self::WebSearch(item) => serialize_with_type(serializer, "web_search", item),
            Self::TodoList(item) => serialize_with_type(serializer, "todo_list", item),
            Self::Error(item) => serialize_with_type(serializer, "error", item),
            Self::Unknown { raw } => raw.serialize(serializer),
        }
    }
}

fn serialize_with_type<T, S>(
    serializer: S,
    item_type: &str,
    payload: &T,
) -> std::result::Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    let mut value = serde_json::to_value(payload).map_err(serde::ser::Error::custom)?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| serde::ser::Error::custom("item payload must serialize to an object"))?;
    object.insert("type".to_string(), Value::String(item_type.to_string()));
    value.serialize(serializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preserves_unknown_items_as_raw_json() {
        let raw = json!({"type":"future_item","id":"item-1","payload":{"answer":42}});
        let item: ThreadItem = serde_json::from_value(raw.clone()).unwrap();

        assert_eq!(item, ThreadItem::Unknown { raw: raw.clone() });
        assert_eq!(serde_json::to_value(item).unwrap(), raw);
    }

    #[test]
    fn preserves_unknown_nested_status_values() {
        let command = json!({
            "type":"command_execution",
            "id":"cmd-1",
            "command":"codex --version",
            "aggregated_output":"",
            "status":"queued_by_future_runtime"
        });
        let item: ThreadItem = serde_json::from_value(command.clone()).unwrap();
        assert_eq!(serde_json::to_value(item).unwrap(), command);

        let file_change = json!({
            "type":"file_change",
            "id":"patch-1",
            "changes":[{"path":"src/lib.rs", "kind":"renamed_by_future_runtime"}],
            "status":"partially_applied"
        });
        let item: ThreadItem = serde_json::from_value(file_change.clone()).unwrap();
        assert_eq!(serde_json::to_value(item).unwrap(), file_change);

        let mcp = json!({
            "type":"mcp_tool_call",
            "id":"mcp-1",
            "server":"local",
            "tool":"future",
            "arguments":{},
            "status":"waiting_on_user"
        });
        let item: ThreadItem = serde_json::from_value(mcp.clone()).unwrap();
        assert_eq!(serde_json::to_value(item).unwrap(), mcp);
    }
}
