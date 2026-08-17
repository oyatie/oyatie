use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};
use serde_json::{Map, Value};

use crate::error::{ClaudeAgentError, Result};

pub type BoxToolFuture = Pin<Box<dyn Future<Output = Result<CallToolResult>> + Send>>;
pub type ToolHandler = Arc<dyn Fn(Value, ToolCallExtra) -> BoxToolFuture + Send + Sync>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ToolCallExtra {
    #[serde(default)]
    pub raw: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallToolResult {
    pub content: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Clone)]
pub struct SdkMcpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub annotations: Option<ToolAnnotations>,
    pub meta: Map<String, Value>,
    pub handler: ToolHandler,
}

impl std::fmt::Debug for SdkMcpTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SdkMcpTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .field("annotations", &self.annotations)
            .field("meta", &self.meta)
            .finish_non_exhaustive()
    }
}

impl SdkMcpTool {
    pub fn new<F, Fut>(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: impl Into<Value>,
        handler: F,
    ) -> Self
    where
        F: Fn(Value, ToolCallExtra) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<CallToolResult>> + Send + 'static,
    {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: input_schema.into(),
            annotations: None,
            meta: Map::new(),
            handler: Arc::new(move |args, extra| Box::pin(handler(args, extra))),
        }
    }

    pub fn new_typed<T, F, Fut>(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: impl Into<Value>,
        handler: F,
    ) -> Self
    where
        T: DeserializeOwned + Send + 'static,
        F: Fn(T, ToolCallExtra) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<CallToolResult>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: input_schema.into(),
            annotations: None,
            meta: Map::new(),
            handler: Arc::new(move |args, extra| {
                let handler = Arc::clone(&handler);
                Box::pin(async move {
                    let typed_args = serde_json::from_value(args).map_err(|error| {
                        ClaudeAgentError::ToolArguments(format!(
                            "typed tool arguments do not match handler input: {error}"
                        ))
                    })?;
                    handler(typed_args, extra).await
                })
            }),
        }
    }

    pub fn annotations(mut self, annotations: ToolAnnotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    pub fn meta(mut self, key: impl Into<String>, value: Value) -> Self {
        self.meta.insert(key.into(), value);
        self
    }

    pub fn search_hint(self, hint: impl Into<String>) -> Self {
        self.meta("anthropic/searchHint", Value::String(hint.into()))
    }

    pub fn always_load(self, always_load: bool) -> Self {
        if always_load {
            self.meta("anthropic/alwaysLoad", Value::Bool(true))
        } else {
            let mut tool = self;
            tool.meta.remove("anthropic/alwaysLoad");
            tool
        }
    }
}

#[derive(Debug, Clone)]
pub struct SdkMcpServer {
    pub name: String,
    pub version: String,
    pub tools: Vec<SdkMcpTool>,
}

pub fn create_sdk_mcp_server(
    name: impl Into<String>,
    version: impl Into<String>,
    tools: Vec<SdkMcpTool>,
) -> SdkMcpServer {
    SdkMcpServer {
        name: name.into(),
        version: version.into(),
        tools,
    }
}

/// Define an in-process SDK MCP tool.
///
/// This mirrors the package-exported Python/TypeScript `tool(...)` helper while
/// keeping Rust's builder-style metadata methods (`annotations`,
/// [`SdkMcpTool::search_hint`], [`SdkMcpTool::always_load`]) available on the
/// returned tool.
pub fn tool<F, Fut>(
    name: impl Into<String>,
    description: impl Into<String>,
    input_schema: impl Into<Value>,
    handler: F,
) -> SdkMcpTool
where
    F: Fn(Value, ToolCallExtra) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<CallToolResult>> + Send + 'static,
{
    SdkMcpTool::new(name, description, input_schema, handler)
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonSchema {
    value: Value,
}

impl JsonSchema {
    pub fn from_value(value: Value) -> Self {
        Self { value }
    }

    pub fn into_value(self) -> Value {
        self.value
    }

    pub fn string() -> Self {
        Self::typed("string")
    }

    pub fn integer() -> Self {
        Self::typed("integer")
    }

    pub fn number() -> Self {
        Self::typed("number")
    }

    pub fn boolean() -> Self {
        Self::typed("boolean")
    }

    pub fn null() -> Self {
        Self::typed("null")
    }

    pub fn array(items: impl Into<JsonSchema>) -> Self {
        let mut object = Map::new();
        object.insert("type".into(), Value::String("array".into()));
        object.insert("items".into(), items.into().into_value());
        Self::from_value(Value::Object(object))
    }

    pub fn object() -> ObjectSchemaBuilder {
        ObjectSchemaBuilder::default()
    }

    pub fn any_of<I>(schemas: I) -> Self
    where
        I: IntoIterator<Item = JsonSchema>,
    {
        let mut object = Map::new();
        object.insert(
            "anyOf".into(),
            Value::Array(schemas.into_iter().map(JsonSchema::into_value).collect()),
        );
        Self::from_value(Value::Object(object))
    }

    pub fn string_enum<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut object = Map::new();
        object.insert("type".into(), Value::String("string".into()));
        object.insert(
            "enum".into(),
            Value::Array(
                values
                    .into_iter()
                    .map(|value| Value::String(value.into()))
                    .collect(),
            ),
        );
        Self::from_value(Value::Object(object))
    }

    pub fn enumeration<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        let mut object = Map::new();
        object.insert("enum".into(), Value::Array(values.into_iter().collect()));
        Self::from_value(Value::Object(object))
    }

    pub fn constant(value: Value) -> Self {
        let mut object = Map::new();
        object.insert("const".into(), value);
        Self::from_value(Value::Object(object))
    }

    pub fn reference(reference: impl Into<String>) -> Self {
        let mut object = Map::new();
        object.insert("$ref".into(), Value::String(reference.into()));
        Self::from_value(Value::Object(object))
    }

    pub fn description(self, description: impl Into<String>) -> Self {
        self.with("description", Value::String(description.into()))
    }

    pub fn title(self, title: impl Into<String>) -> Self {
        self.with("title", Value::String(title.into()))
    }

    pub fn minimum(self, minimum: i64) -> Self {
        self.with("minimum", Value::Number(minimum.into()))
    }

    pub fn maximum(self, maximum: i64) -> Self {
        self.with("maximum", Value::Number(maximum.into()))
    }

    pub fn default_value(self, value: Value) -> Self {
        self.with("default", value)
    }

    pub fn extra(self, key: impl Into<String>, value: Value) -> Self {
        self.with(key.into(), value)
    }

    fn typed(schema_type: &'static str) -> Self {
        let mut object = Map::new();
        object.insert("type".into(), Value::String(schema_type.into()));
        Self::from_value(Value::Object(object))
    }

    fn with(mut self, key: impl Into<String>, value: Value) -> Self {
        match &mut self.value {
            Value::Object(object) => {
                object.insert(key.into(), value);
            }
            other => {
                let original = std::mem::replace(other, Value::Object(Map::new()));
                if let Value::Object(object) = other {
                    object.insert("const".into(), original);
                    object.insert(key.into(), value);
                }
            }
        }
        self
    }
}

impl Serialize for JsonSchema {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for JsonSchema {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from_value(Value::deserialize(deserializer)?))
    }
}

impl From<JsonSchema> for Value {
    fn from(schema: JsonSchema) -> Self {
        schema.into_value()
    }
}

impl From<Value> for JsonSchema {
    fn from(value: Value) -> Self {
        Self::from_value(value)
    }
}

impl From<ObjectSchemaBuilder> for JsonSchema {
    fn from(builder: ObjectSchemaBuilder) -> Self {
        builder.build()
    }
}

impl From<ObjectSchemaBuilder> for Value {
    fn from(builder: ObjectSchemaBuilder) -> Self {
        builder.build().into_value()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObjectSchemaBuilder {
    properties: Map<String, Value>,
    required: Vec<String>,
    additional_properties: Option<Value>,
    title: Option<String>,
    description: Option<String>,
    extra: Map<String, Value>,
}

impl ObjectSchemaBuilder {
    pub fn required_property(
        mut self,
        name: impl Into<String>,
        schema: impl Into<JsonSchema>,
    ) -> Self {
        let name = name.into();
        if !self.required.iter().any(|existing| existing == &name) {
            self.required.push(name.clone());
        }
        self.properties.insert(name, schema.into().into_value());
        self
    }

    pub fn optional_property(
        mut self,
        name: impl Into<String>,
        schema: impl Into<JsonSchema>,
    ) -> Self {
        let name = name.into();
        self.required.retain(|required| required != &name);
        self.properties.insert(name, schema.into().into_value());
        self
    }

    pub fn additional_properties(mut self, allowed: bool) -> Self {
        self.additional_properties = Some(Value::Bool(allowed));
        self
    }

    pub fn additional_properties_schema(mut self, schema: impl Into<JsonSchema>) -> Self {
        self.additional_properties = Some(schema.into().into_value());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    pub fn build(self) -> JsonSchema {
        let mut object = Map::new();
        object.insert("type".into(), Value::String("object".into()));
        object.insert("properties".into(), Value::Object(self.properties));
        if !self.required.is_empty() {
            object.insert(
                "required".into(),
                Value::Array(self.required.into_iter().map(Value::String).collect()),
            );
        }
        if let Some(additional_properties) = self.additional_properties {
            object.insert("additionalProperties".into(), additional_properties);
        }
        if let Some(title) = self.title {
            object.insert("title".into(), Value::String(title));
        }
        if let Some(description) = self.description {
            object.insert("description".into(), Value::String(description));
        }
        object.extend(self.extra);
        JsonSchema::from_value(Value::Object(object))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinToolName {
    Agent,
    AskUserQuestion,
    Bash,
    Monitor,
    TaskOutput,
    Edit,
    Read,
    Write,
    Glob,
    Grep,
    TaskStop,
    NotebookEdit,
    WebFetch,
    WebSearch,
    Workflow,
    TodoWrite,
    TaskCreate,
    TaskUpdate,
    TaskGet,
    TaskList,
    EnterPlanMode,
    ExitPlanMode,
    ListMcpResources,
    Mcp,
    ReadMcpResource,
    EnterWorktree,
    ExitWorktree,
    Repl,
    CronCreate,
    CronDelete,
    CronList,
    ScheduleWakeup,
    RemoteTrigger,
    PushNotification,
}

impl BuiltinToolName {
    pub fn from_tool_name(name: &str) -> Option<Self> {
        match name {
            "Agent" | "Task" => Some(Self::Agent),
            "AskUserQuestion" => Some(Self::AskUserQuestion),
            "Bash" => Some(Self::Bash),
            "Monitor" => Some(Self::Monitor),
            "TaskOutput" => Some(Self::TaskOutput),
            "Edit" => Some(Self::Edit),
            "Read" => Some(Self::Read),
            "Write" => Some(Self::Write),
            "Glob" => Some(Self::Glob),
            "Grep" => Some(Self::Grep),
            "TaskStop" => Some(Self::TaskStop),
            "NotebookEdit" => Some(Self::NotebookEdit),
            "WebFetch" => Some(Self::WebFetch),
            "WebSearch" => Some(Self::WebSearch),
            "Workflow" => Some(Self::Workflow),
            "TodoWrite" => Some(Self::TodoWrite),
            "TaskCreate" => Some(Self::TaskCreate),
            "TaskUpdate" => Some(Self::TaskUpdate),
            "TaskGet" => Some(Self::TaskGet),
            "TaskList" => Some(Self::TaskList),
            "EnterPlanMode" => Some(Self::EnterPlanMode),
            "ExitPlanMode" => Some(Self::ExitPlanMode),
            "ListMcpResourcesTool" | "ListMcpResources" => Some(Self::ListMcpResources),
            "Mcp" => Some(Self::Mcp),
            "ReadMcpResourceTool" | "ReadMcpResource" => Some(Self::ReadMcpResource),
            "EnterWorktree" => Some(Self::EnterWorktree),
            "ExitWorktree" => Some(Self::ExitWorktree),
            "REPL" => Some(Self::Repl),
            "CronCreate" => Some(Self::CronCreate),
            "CronDelete" => Some(Self::CronDelete),
            "CronList" => Some(Self::CronList),
            "ScheduleWakeup" => Some(Self::ScheduleWakeup),
            "RemoteTrigger" => Some(Self::RemoteTrigger),
            "PushNotification" => Some(Self::PushNotification),
            _ if name.starts_with("mcp__") => Some(Self::Mcp),
            _ => None,
        }
    }

    pub fn as_tool_name(self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::AskUserQuestion => "AskUserQuestion",
            Self::Bash => "Bash",
            Self::Monitor => "Monitor",
            Self::TaskOutput => "TaskOutput",
            Self::Edit => "Edit",
            Self::Read => "Read",
            Self::Write => "Write",
            Self::Glob => "Glob",
            Self::Grep => "Grep",
            Self::TaskStop => "TaskStop",
            Self::NotebookEdit => "NotebookEdit",
            Self::WebFetch => "WebFetch",
            Self::WebSearch => "WebSearch",
            Self::Workflow => "Workflow",
            Self::TodoWrite => "TodoWrite",
            Self::TaskCreate => "TaskCreate",
            Self::TaskUpdate => "TaskUpdate",
            Self::TaskGet => "TaskGet",
            Self::TaskList => "TaskList",
            Self::EnterPlanMode => "EnterPlanMode",
            Self::ExitPlanMode => "ExitPlanMode",
            Self::ListMcpResources => "ListMcpResourcesTool",
            Self::Mcp => "mcp__",
            Self::ReadMcpResource => "ReadMcpResourceTool",
            Self::EnterWorktree => "EnterWorktree",
            Self::ExitWorktree => "ExitWorktree",
            Self::Repl => "REPL",
            Self::CronCreate => "CronCreate",
            Self::CronDelete => "CronDelete",
            Self::CronList => "CronList",
            Self::ScheduleWakeup => "ScheduleWakeup",
            Self::RemoteTrigger => "RemoteTrigger",
            Self::PushNotification => "PushNotification",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BuiltinToolInput {
    Agent(AgentInput),
    AskUserQuestion(AskUserQuestionInput),
    Bash(BashInput),
    Monitor(MonitorInput),
    TaskOutput(TaskOutputInput),
    Edit(FileEditInput),
    Read(FileReadInput),
    Write(FileWriteInput),
    Glob(GlobInput),
    Grep(GrepInput),
    TaskStop(TaskStopInput),
    NotebookEdit(NotebookEditInput),
    WebFetch(WebFetchInput),
    WebSearch(WebSearchInput),
    Workflow(WorkflowInput),
    TodoWrite(TodoWriteInput),
    TaskCreate(TaskCreateInput),
    TaskUpdate(TaskUpdateInput),
    TaskGet(TaskGetInput),
    TaskList(TaskListInput),
    EnterPlanMode(EnterPlanModeInput),
    ExitPlanMode(ExitPlanModeInput),
    ListMcpResources(ListMcpResourcesInput),
    Mcp(McpInput),
    ReadMcpResource(ReadMcpResourceInput),
    EnterWorktree(EnterWorktreeInput),
    ExitWorktree(ExitWorktreeInput),
    Repl(ReplInput),
    CronCreate(CronCreateInput),
    CronDelete(CronDeleteInput),
    CronList(CronListInput),
    ScheduleWakeup(ScheduleWakeupInput),
    RemoteTrigger(RemoteTriggerInput),
    PushNotification(PushNotificationInput),
}

impl BuiltinToolInput {
    pub fn parse(tool_name: &str, input: Value) -> crate::Result<Option<Self>> {
        let Some(name) = BuiltinToolName::from_tool_name(tool_name) else {
            return Ok(None);
        };
        Ok(Some(match name {
            BuiltinToolName::Agent => Self::Agent(from_value(input)?),
            BuiltinToolName::AskUserQuestion => Self::AskUserQuestion(from_value(input)?),
            BuiltinToolName::Bash => Self::Bash(from_value(input)?),
            BuiltinToolName::Monitor => Self::Monitor(from_value(input)?),
            BuiltinToolName::TaskOutput => Self::TaskOutput(from_value(input)?),
            BuiltinToolName::Edit => Self::Edit(from_value(input)?),
            BuiltinToolName::Read => Self::Read(from_value(input)?),
            BuiltinToolName::Write => Self::Write(from_value(input)?),
            BuiltinToolName::Glob => Self::Glob(from_value(input)?),
            BuiltinToolName::Grep => Self::Grep(from_value(input)?),
            BuiltinToolName::TaskStop => Self::TaskStop(from_value(input)?),
            BuiltinToolName::NotebookEdit => Self::NotebookEdit(from_value(input)?),
            BuiltinToolName::WebFetch => Self::WebFetch(from_value(input)?),
            BuiltinToolName::WebSearch => Self::WebSearch(from_value(input)?),
            BuiltinToolName::Workflow => Self::Workflow(from_value(input)?),
            BuiltinToolName::TodoWrite => Self::TodoWrite(from_value(input)?),
            BuiltinToolName::TaskCreate => Self::TaskCreate(from_value(input)?),
            BuiltinToolName::TaskUpdate => Self::TaskUpdate(from_value(input)?),
            BuiltinToolName::TaskGet => Self::TaskGet(from_value(input)?),
            BuiltinToolName::TaskList => Self::TaskList(from_value(input)?),
            BuiltinToolName::EnterPlanMode => Self::EnterPlanMode(from_value(input)?),
            BuiltinToolName::ExitPlanMode => Self::ExitPlanMode(from_value(input)?),
            BuiltinToolName::ListMcpResources => Self::ListMcpResources(from_value(input)?),
            BuiltinToolName::Mcp => Self::Mcp(from_value(input)?),
            BuiltinToolName::ReadMcpResource => Self::ReadMcpResource(from_value(input)?),
            BuiltinToolName::EnterWorktree => Self::EnterWorktree(from_value(input)?),
            BuiltinToolName::ExitWorktree => Self::ExitWorktree(from_value(input)?),
            BuiltinToolName::Repl => Self::Repl(from_value(input)?),
            BuiltinToolName::CronCreate => Self::CronCreate(from_value(input)?),
            BuiltinToolName::CronDelete => Self::CronDelete(from_value(input)?),
            BuiltinToolName::CronList => Self::CronList(from_value(input)?),
            BuiltinToolName::ScheduleWakeup => Self::ScheduleWakeup(from_value(input)?),
            BuiltinToolName::RemoteTrigger => Self::RemoteTrigger(from_value(input)?),
            BuiltinToolName::PushNotification => Self::PushNotification(from_value(input)?),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BuiltinToolOutput {
    Agent(AgentOutput),
    AskUserQuestion(AskUserQuestionOutput),
    Bash(BashOutput),
    Monitor(MonitorOutput),
    Edit(FileEditOutput),
    Read(FileReadOutput),
    Write(FileWriteOutput),
    Glob(GlobOutput),
    Grep(GrepOutput),
    TaskStop(TaskStopOutput),
    NotebookEdit(NotebookEditOutput),
    WebFetch(WebFetchOutput),
    WebSearch(WebSearchOutput),
    Workflow(WorkflowOutput),
    TodoWrite(TodoWriteOutput),
    TaskCreate(TaskCreateOutput),
    TaskUpdate(TaskUpdateOutput),
    TaskGet(TaskGetOutput),
    TaskList(TaskListOutput),
    EnterPlanMode(EnterPlanModeOutput),
    ExitPlanMode(ExitPlanModeOutput),
    ListMcpResources(ListMcpResourcesOutput),
    Mcp(McpOutput),
    ReadMcpResource(ReadMcpResourceOutput),
    EnterWorktree(EnterWorktreeOutput),
    ExitWorktree(ExitWorktreeOutput),
    Repl(ReplOutput),
    CronCreate(CronCreateOutput),
    CronDelete(CronDeleteOutput),
    CronList(CronListOutput),
    ScheduleWakeup(ScheduleWakeupOutput),
    RemoteTrigger(RemoteTriggerOutput),
    PushNotification(PushNotificationOutput),
}

impl BuiltinToolOutput {
    pub fn parse(tool_name: &str, output: Value) -> crate::Result<Option<Self>> {
        let Some(name) = BuiltinToolName::from_tool_name(tool_name) else {
            return Ok(None);
        };
        Ok(match name {
            BuiltinToolName::Agent => Some(Self::Agent(from_value(output)?)),
            BuiltinToolName::AskUserQuestion => Some(Self::AskUserQuestion(from_value(output)?)),
            BuiltinToolName::Bash => Some(Self::Bash(from_value(output)?)),
            BuiltinToolName::Monitor => Some(Self::Monitor(from_value(output)?)),
            BuiltinToolName::TaskOutput => None,
            BuiltinToolName::Edit => Some(Self::Edit(from_value(output)?)),
            BuiltinToolName::Read => Some(Self::Read(from_value(output)?)),
            BuiltinToolName::Write => Some(Self::Write(from_value(output)?)),
            BuiltinToolName::Glob => Some(Self::Glob(from_value(output)?)),
            BuiltinToolName::Grep => Some(Self::Grep(from_value(output)?)),
            BuiltinToolName::TaskStop => Some(Self::TaskStop(from_value(output)?)),
            BuiltinToolName::NotebookEdit => Some(Self::NotebookEdit(from_value(output)?)),
            BuiltinToolName::WebFetch => Some(Self::WebFetch(from_value(output)?)),
            BuiltinToolName::WebSearch => Some(Self::WebSearch(from_value(output)?)),
            BuiltinToolName::Workflow => Some(Self::Workflow(from_value(output)?)),
            BuiltinToolName::TodoWrite => Some(Self::TodoWrite(from_value(output)?)),
            BuiltinToolName::TaskCreate => Some(Self::TaskCreate(from_value(output)?)),
            BuiltinToolName::TaskUpdate => Some(Self::TaskUpdate(from_value(output)?)),
            BuiltinToolName::TaskGet => Some(Self::TaskGet(from_value(output)?)),
            BuiltinToolName::TaskList => Some(Self::TaskList(from_value(output)?)),
            BuiltinToolName::EnterPlanMode => Some(Self::EnterPlanMode(from_value(output)?)),
            BuiltinToolName::ExitPlanMode => Some(Self::ExitPlanMode(from_value(output)?)),
            BuiltinToolName::ListMcpResources => Some(Self::ListMcpResources(from_value(output)?)),
            BuiltinToolName::Mcp => Some(Self::Mcp(from_value(output)?)),
            BuiltinToolName::ReadMcpResource => Some(Self::ReadMcpResource(from_value(output)?)),
            BuiltinToolName::EnterWorktree => Some(Self::EnterWorktree(from_value(output)?)),
            BuiltinToolName::ExitWorktree => Some(Self::ExitWorktree(from_value(output)?)),
            BuiltinToolName::Repl => Some(Self::Repl(from_value(output)?)),
            BuiltinToolName::CronCreate => Some(Self::CronCreate(from_value(output)?)),
            BuiltinToolName::CronDelete => Some(Self::CronDelete(from_value(output)?)),
            BuiltinToolName::CronList => Some(Self::CronList(from_value(output)?)),
            BuiltinToolName::ScheduleWakeup => Some(Self::ScheduleWakeup(from_value(output)?)),
            BuiltinToolName::RemoteTrigger => Some(Self::RemoteTrigger(from_value(output)?)),
            BuiltinToolName::PushNotification => Some(Self::PushNotification(from_value(output)?)),
        })
    }
}

fn from_value<T>(value: Value) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_value(value)?)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentInput {
    pub description: String,
    pub prompt: String,
    pub subagent_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<AgentToolModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_in_background: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<AgentToolPermissionMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isolation: Option<AgentToolIsolation>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentToolModel {
    Sonnet,
    Opus,
    Haiku,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentToolPermissionMode {
    AcceptEdits,
    BypassPermissions,
    #[serde(rename = "default")]
    Default,
    DontAsk,
    Plan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentToolIsolation {
    Worktree,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AskUserQuestionInput {
    pub questions: Vec<AskUserQuestion>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AskUserQuestion {
    pub question: String,
    pub header: String,
    pub options: Vec<AskUserQuestionOption>,
    #[serde(rename = "multiSelect")]
    pub multi_select: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AskUserQuestionOption {
    pub label: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BashInput {
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_in_background: Option<bool>,
    #[serde(
        default,
        rename = "dangerouslyDisableSandbox",
        skip_serializing_if = "Option::is_none"
    )]
    pub dangerously_disable_sandbox: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorInput {
    pub command: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistent: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskOutputInput {
    pub task_id: String,
    pub block: bool,
    pub timeout: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileEditInput {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replace_all: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileReadInput {
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWriteInput {
    pub file_path: String,
    pub content: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobInput {
    pub pattern: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrepInput {
    pub pattern: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glob: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_mode: Option<GrepOutputMode>,
    #[serde(default, rename = "-i", skip_serializing_if = "Option::is_none")]
    pub ignore_case: Option<bool>,
    #[serde(default, rename = "-n", skip_serializing_if = "Option::is_none")]
    pub line_numbers: Option<bool>,
    #[serde(default, rename = "-B", skip_serializing_if = "Option::is_none")]
    pub before_context: Option<u64>,
    #[serde(default, rename = "-A", skip_serializing_if = "Option::is_none")]
    pub after_context: Option<u64>,
    #[serde(default, rename = "-C", skip_serializing_if = "Option::is_none")]
    pub context_lines: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiline: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrepOutputMode {
    Content,
    FilesWithMatches,
    Count,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskStopInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell_id: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotebookEditInput {
    pub notebook_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<String>,
    pub new_source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_type: Option<NotebookCellType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit_mode: Option<NotebookEditMode>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotebookCellType {
    Code,
    Markdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotebookEditMode {
    Replace,
    Insert,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebFetchInput {
    pub url: String,
    pub prompt: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebSearchInput {
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        default,
        rename = "scriptPath",
        skip_serializing_if = "Option::is_none"
    )]
    pub script_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,
    #[serde(
        default,
        rename = "resumeFromRunId",
        skip_serializing_if = "Option::is_none"
    )]
    pub resume_from_run_id: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoWriteInput {
    pub todos: Vec<TodoItem>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoItem {
    pub content: String,
    pub status: TodoStatus,
    #[serde(rename = "activeForm")]
    pub active_form: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCreateInput {
    pub subject: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_form: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Map<String, Value>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdateInput {
    pub task_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskUpdateToolStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_form: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add_blocks: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add_blocked_by: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Map<String, Value>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskUpdateToolStatus {
    Pending,
    InProgress,
    Completed,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskGetInput {
    pub task_id: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TaskListInput {
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EnterPlanModeInput {
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitPlanModeInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_prompts: Option<Vec<ExitPlanAllowedPrompt>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExitPlanAllowedPrompt {
    pub tool: ExitPlanAllowedPromptTool,
    pub prompt: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExitPlanAllowedPromptTool {
    Bash,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ListMcpResourcesInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct McpInput {
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadMcpResourceInput {
    pub server: String,
    pub uri: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EnterWorktreeInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExitWorktreeInput {
    pub action: WorktreeExitAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discard_changes: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorktreeExitAction {
    Keep,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplInput {
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CronCreateInput {
    pub cron: String,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurring: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub durable: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CronDeleteInput {
    pub id: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CronListInput {
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleWakeupInput {
    pub delay_seconds: u64,
    pub reason: String,
    pub prompt: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteTriggerAction {
    List,
    Get,
    Create,
    Update,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteTriggerInput {
    pub action: RemoteTriggerAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Map<String, Value>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PushNotificationStatus {
    Proactive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushNotificationInput {
    pub message: String,
    pub status: PushNotificationStatus,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AgentOutput {
    Completed {
        #[serde(rename = "agentId")]
        agent_id: String,
        content: Vec<AgentTextContent>,
        #[serde(rename = "totalToolUseCount")]
        total_tool_use_count: u64,
        #[serde(rename = "totalDurationMs")]
        total_duration_ms: u64,
        #[serde(rename = "totalTokens")]
        total_tokens: u64,
        usage: Box<AgentUsage>,
        prompt: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    AsyncLaunched {
        #[serde(rename = "agentId")]
        agent_id: String,
        description: String,
        prompt: String,
        #[serde(rename = "outputFile")]
        output_file: String,
        #[serde(
            default,
            rename = "canReadOutputFile",
            skip_serializing_if = "Option::is_none"
        )]
        can_read_output_file: Option<bool>,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    SubAgentEntered {
        description: String,
        message: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentTextContent {
    pub r#type: AgentTextContentType,
    pub text: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentTextContentType {
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_tool_use: Option<AgentServerToolUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation: Option<AgentCacheCreationUsage>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentServerToolUsage {
    pub web_search_requests: u64,
    pub web_fetch_requests: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCacheCreationUsage {
    pub ephemeral_1h_input_tokens: u64,
    pub ephemeral_5m_input_tokens: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceTier {
    Standard,
    Priority,
    Batch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AskUserQuestionOutput {
    pub questions: Vec<AskUserQuestion>,
    pub answers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BashOutput {
    pub stdout: String,
    pub stderr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_output_path: Option<String>,
    pub interrupted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_image: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backgrounded_by_user: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dangerously_disable_sandbox: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_code_interpretation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persisted_output_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persisted_output_size: Option<u64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorOutput {
    pub task_id: String,
    pub timeout_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistent: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEditOutput {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
    pub original_file: String,
    pub structured_patch: Vec<StructuredPatchHunk>,
    pub user_modified: bool,
    pub replace_all: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_diff: Option<GitDiff>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredPatchHunk {
    pub old_start: u64,
    pub old_lines: u64,
    pub new_start: u64,
    pub new_lines: u64,
    pub lines: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitDiff {
    pub filename: String,
    pub status: GitDiffStatus,
    pub additions: u64,
    pub deletions: u64,
    pub changes: u64,
    pub patch: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitDiffStatus {
    Modified,
    Added,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FileReadOutput {
    Text {
        file: FileReadTextFile,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    Image {
        file: FileReadImageFile,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    Notebook {
        file: FileReadNotebookFile,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    Pdf {
        file: FileReadPdfFile,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    Parts {
        file: FileReadPartsFile,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReadTextFile {
    pub file_path: String,
    pub content: String,
    pub num_lines: u64,
    pub start_line: u64,
    pub total_lines: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReadImageFile {
    pub base64: String,
    #[serde(rename = "type")]
    pub mime_type: FileReadImageMimeType,
    pub original_size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<FileReadImageDimensions>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileReadImageMimeType {
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/gif")]
    Gif,
    #[serde(rename = "image/webp")]
    Webp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReadImageDimensions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_width: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_height: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_width: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_height: Option<u64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReadNotebookFile {
    pub file_path: String,
    pub cells: Vec<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReadPdfFile {
    pub file_path: String,
    pub base64: String,
    pub original_size: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReadPartsFile {
    pub file_path: String,
    pub original_size: u64,
    pub count: u64,
    pub output_dir: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileWriteOutput {
    #[serde(rename = "type")]
    pub write_type: FileWriteOutputType,
    pub file_path: String,
    pub content: String,
    pub structured_patch: Vec<StructuredPatchHunk>,
    pub original_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_diff: Option<GitDiff>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileWriteOutputType {
    Create,
    Update,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobOutput {
    pub duration_ms: u64,
    pub num_files: u64,
    pub filenames: Vec<String>,
    pub truncated: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrepOutput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<GrepOutputMode>,
    pub num_files: u64,
    pub filenames: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_lines: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_matches: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_offset: Option<u64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskStopOutput {
    pub message: String,
    pub task_id: String,
    pub task_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotebookEditOutput {
    pub new_source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<String>,
    pub cell_type: NotebookCellType,
    pub language: String,
    pub edit_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub notebook_path: String,
    pub original_file: String,
    pub updated_file: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebFetchOutput {
    pub bytes: u64,
    pub code: u16,
    pub code_text: String,
    pub result: String,
    pub duration_ms: u64,
    pub url: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSearchOutput {
    pub query: String,
    pub results: Vec<WebSearchResult>,
    pub duration_seconds: f64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebSearchResult {
    Links {
        tool_use_id: String,
        content: Vec<WebSearchLink>,
    },
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebSearchLink {
    pub title: String,
    pub url: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowOutput {
    pub status: WorkflowOutputStatus,
    pub task_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowOutputStatus {
    AsyncLaunched,
    RemoteLaunched,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoWriteOutput {
    #[serde(rename = "oldTodos")]
    pub old_todos: Vec<TodoItem>,
    #[serde(rename = "newTodos")]
    pub new_todos: Vec<TodoItem>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCreateOutput {
    pub task: TaskCreateOutputTask,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCreateOutputTask {
    pub id: String,
    pub subject: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdateOutput {
    pub success: bool,
    pub task_id: String,
    pub updated_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_change: Option<TaskStatusChange>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskStatusChange {
    pub from: String,
    pub to: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskGetOutput {
    pub task: Option<TaskGetOutputTask>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskGetOutputTask {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: TaskToolStatus,
    pub blocks: Vec<String>,
    #[serde(rename = "blockedBy")]
    pub blocked_by: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskListOutput {
    pub tasks: Vec<TaskListOutputTask>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskListOutputTask {
    pub id: String,
    pub subject: String,
    pub status: TaskToolStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(rename = "blockedBy")]
    pub blocked_by: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnterPlanModeOutput {
    pub message: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskToolStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitPlanModeOutput {
    pub plan: Option<String>,
    pub is_agent: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_task_tool: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub awaiting_leader_approval: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub type ListMcpResourcesOutput = Vec<McpResourceInfo>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpOutput {
    Text(String),
    Content(Vec<McpOutputContent>),
    Object(Map<String, Value>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpOutputContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpResourceInfo {
    pub uri: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub server: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadMcpResourceOutput {
    pub contents: Vec<McpResourceContent>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpResourceContent {
    pub uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnterWorktreeOutput {
    pub worktree_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree_branch: Option<String>,
    pub message: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitWorktreeOutput {
    pub action: WorktreeExitAction,
    pub original_cwd: String,
    pub worktree_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tmux_session_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discarded_files: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discarded_commits: Option<u64>,
    pub message: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplOutput {
    pub code: String,
    pub result: Map<String, Value>,
    pub stdout: String,
    pub stderr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(
        default,
        rename = "registeredTools",
        skip_serializing_if = "Option::is_none"
    )]
    pub registered_tools: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<ReplImage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documents: Option<Vec<ReplDocument>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplImage {
    pub base64: String,
    pub media_type: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplDocument {
    pub base64: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronCreateOutput {
    pub id: String,
    pub human_schedule: String,
    pub recurring: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub durable: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CronDeleteOutput {
    pub id: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CronListOutput {
    pub jobs: Vec<CronJob>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJob {
    pub id: String,
    pub cron: String,
    pub human_schedule: String,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurring: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub durable: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleWakeupOutput {
    pub scheduled_for: u64,
    pub clamped_delay_seconds: u64,
    pub was_clamped: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteTriggerOutput {
    pub status: u16,
    pub json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushNotificationDisabledReason {
    ConfigOff,
    UserPresent,
    NoTransport,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationOutput {
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_sent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_sent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<PushNotificationDisabledReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_sec: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_focus: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_at: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContentBlock, ToolPermissionRequest};
    use serde_json::json;

    #[test]
    fn parses_documented_builtin_tool_inputs_by_name_and_alias() {
        let parsed = BuiltinToolInput::parse(
            "Task",
            json!({
                "description": "Review",
                "prompt": "review this patch",
                "subagent_type": "code-reviewer",
                "model": "sonnet",
                "mode": "acceptEdits",
                "isolation": "worktree",
                "future": true
            }),
        )
        .unwrap()
        .unwrap();
        let BuiltinToolInput::Agent(input) = parsed else {
            panic!("expected Agent input");
        };
        assert_eq!(input.model, Some(AgentToolModel::Sonnet));
        assert_eq!(input.mode, Some(AgentToolPermissionMode::AcceptEdits));
        assert_eq!(input.isolation, Some(AgentToolIsolation::Worktree));
        assert_eq!(input.extra["future"], true);

        let grep = BuiltinToolInput::parse(
            "Grep",
            json!({
                "pattern": "TODO",
                "output_mode": "content",
                "-i": true,
                "-B": 2
            }),
        )
        .unwrap()
        .unwrap();
        let BuiltinToolInput::Grep(grep) = grep else {
            panic!("expected Grep input");
        };
        assert_eq!(grep.output_mode, Some(GrepOutputMode::Content));
        assert_eq!(grep.ignore_case, Some(true));
        assert_eq!(grep.before_context, Some(2));

        let block = ContentBlock::ToolUse {
            id: "toolu_1".into(),
            name: "Read".into(),
            input: json!({"file_path": "README.md", "pages": "1-2"}),
        };
        let Some(BuiltinToolInput::Read(read)) = block.builtin_tool_input().unwrap() else {
            panic!("expected Read input");
        };
        assert_eq!(read.file_path, "README.md");
        assert_eq!(read.pages.as_deref(), Some("1-2"));

        let permission = ToolPermissionRequest {
            tool_name: "Bash".into(),
            input: json!({"command": "cargo test", "dangerouslyDisableSandbox": true}),
            tool_use_id: "toolu_2".into(),
            ..serde_json::from_value(json!({
                "tool_name": "Bash",
                "tool_use_id": "toolu_2",
                "input": {"command": "cargo test", "dangerouslyDisableSandbox": true}
            }))
            .unwrap()
        };
        let Some(BuiltinToolInput::Bash(bash)) = permission.builtin_input().unwrap() else {
            panic!("expected Bash input");
        };
        assert_eq!(bash.command, "cargo test");
        assert_eq!(bash.dangerously_disable_sandbox, Some(true));

        let parsed = BuiltinToolInput::parse(
            "mcp__docs__search",
            json!({
                "query": "agent sdk",
                "limit": 5
            }),
        )
        .unwrap()
        .unwrap();
        let BuiltinToolInput::Mcp(input) = parsed else {
            panic!("expected generic MCP input");
        };
        assert_eq!(input.extra["query"], "agent sdk");
        assert_eq!(input.extra["limit"], 5);
    }

    #[test]
    fn parses_documented_builtin_tool_outputs_by_name() {
        let parsed = BuiltinToolOutput::parse(
            "Read",
            json!({
                "type": "image",
                "file": {
                    "base64": "abc",
                    "type": "image/png",
                    "originalSize": 3,
                    "dimensions": {"displayWidth": 80}
                },
                "newField": "kept"
            }),
        )
        .unwrap()
        .unwrap();
        let BuiltinToolOutput::Read(FileReadOutput::Image { file, extra }) = parsed else {
            panic!("expected image read output");
        };
        assert_eq!(file.mime_type, FileReadImageMimeType::Png);
        assert_eq!(file.dimensions.unwrap().display_width, Some(80));
        assert_eq!(extra["newField"], "kept");

        let parsed = BuiltinToolOutput::parse(
            "TaskList",
            json!({
                "tasks": [{
                    "id": "task-1",
                    "subject": "Implement",
                    "status": "in_progress",
                    "owner": "agent",
                    "blockedBy": []
                }]
            }),
        )
        .unwrap()
        .unwrap();
        let BuiltinToolOutput::TaskList(list) = parsed else {
            panic!("expected task list output");
        };
        assert_eq!(list.tasks[0].status, TaskToolStatus::InProgress);

        let parsed = BuiltinToolOutput::parse(
            "mcp__docs__search",
            json!([{
                "type": "text",
                "text": "result"
            }]),
        )
        .unwrap()
        .unwrap();
        let BuiltinToolOutput::Mcp(McpOutput::Content(content)) = parsed else {
            panic!("expected generic MCP content output");
        };
        assert_eq!(content[0].content_type, "text");
        assert_eq!(content[0].extra["text"], "result");

        let parsed = BuiltinToolOutput::parse(
            "Mcp",
            json!({
                "structured": true
            }),
        )
        .unwrap()
        .unwrap();
        let BuiltinToolOutput::Mcp(McpOutput::Object(object)) = parsed else {
            panic!("expected generic MCP object output");
        };
        assert_eq!(object["structured"], true);
    }

    #[test]
    fn parses_current_package_exported_builtin_tool_inputs() {
        let Some(BuiltinToolInput::EnterPlanMode(input)) =
            BuiltinToolInput::parse("EnterPlanMode", json!({"future": true})).unwrap()
        else {
            panic!("expected EnterPlanMode input");
        };
        assert_eq!(input.extra["future"], true);

        let Some(BuiltinToolInput::ExitWorktree(input)) = BuiltinToolInput::parse(
            "ExitWorktree",
            json!({"action": "remove", "discard_changes": true}),
        )
        .unwrap() else {
            panic!("expected ExitWorktree input");
        };
        assert_eq!(input.action, WorktreeExitAction::Remove);
        assert_eq!(input.discard_changes, Some(true));

        let Some(BuiltinToolInput::Repl(input)) = BuiltinToolInput::parse(
            "REPL",
            json!({
                "code": "await getState()",
                "description": "Inspect persisted state",
                "timeout": 30000
            }),
        )
        .unwrap() else {
            panic!("expected REPL input");
        };
        assert_eq!(input.code, "await getState()");
        assert_eq!(input.timeout, Some(30000));

        let Some(BuiltinToolInput::CronCreate(input)) = BuiltinToolInput::parse(
            "CronCreate",
            json!({
                "cron": "*/5 * * * *",
                "prompt": "check status",
                "recurring": true,
                "durable": false
            }),
        )
        .unwrap() else {
            panic!("expected CronCreate input");
        };
        assert_eq!(input.cron, "*/5 * * * *");
        assert_eq!(input.recurring, Some(true));

        let Some(BuiltinToolInput::CronDelete(input)) =
            BuiltinToolInput::parse("CronDelete", json!({"id": "job-1"})).unwrap()
        else {
            panic!("expected CronDelete input");
        };
        assert_eq!(input.id, "job-1");

        let Some(BuiltinToolInput::CronList(input)) =
            BuiltinToolInput::parse("CronList", json!({"scope": "session"})).unwrap()
        else {
            panic!("expected CronList input");
        };
        assert_eq!(input.extra["scope"], "session");

        let Some(BuiltinToolInput::ScheduleWakeup(input)) = BuiltinToolInput::parse(
            "ScheduleWakeup",
            json!({
                "delaySeconds": 120,
                "reason": "Wait for CI",
                "prompt": "<<autonomous-loop-dynamic>>"
            }),
        )
        .unwrap() else {
            panic!("expected ScheduleWakeup input");
        };
        assert_eq!(input.delay_seconds, 120);
        assert_eq!(input.prompt, "<<autonomous-loop-dynamic>>");

        let Some(BuiltinToolInput::RemoteTrigger(input)) = BuiltinToolInput::parse(
            "RemoteTrigger",
            json!({
                "action": "run",
                "trigger_id": "trigger-1",
                "body": {"branch": "main"}
            }),
        )
        .unwrap() else {
            panic!("expected RemoteTrigger input");
        };
        assert_eq!(input.action, RemoteTriggerAction::Run);
        assert_eq!(input.body.unwrap()["branch"], "main");

        let Some(BuiltinToolInput::PushNotification(input)) = BuiltinToolInput::parse(
            "PushNotification",
            json!({"message": "Build finished", "status": "proactive"}),
        )
        .unwrap() else {
            panic!("expected PushNotification input");
        };
        assert_eq!(input.status, PushNotificationStatus::Proactive);
    }

    #[test]
    fn parses_current_package_exported_builtin_tool_outputs() {
        let Some(BuiltinToolOutput::EnterPlanMode(output)) =
            BuiltinToolOutput::parse("EnterPlanMode", json!({"message": "Plan mode entered"}))
                .unwrap()
        else {
            panic!("expected EnterPlanMode output");
        };
        assert_eq!(output.message, "Plan mode entered");

        let Some(BuiltinToolOutput::Workflow(output)) = BuiltinToolOutput::parse(
            "Workflow",
            json!({
                "status": "remote_launched",
                "taskId": "task-remote",
                "sessionUrl": "https://console.example/session",
                "warning": "branch differs"
            }),
        )
        .unwrap() else {
            panic!("expected Workflow output");
        };
        assert_eq!(output.status, WorkflowOutputStatus::RemoteLaunched);
        assert_eq!(
            output.session_url.as_deref(),
            Some("https://console.example/session")
        );
        assert_eq!(output.warning.as_deref(), Some("branch differs"));

        let Some(BuiltinToolOutput::ExitWorktree(output)) = BuiltinToolOutput::parse(
            "ExitWorktree",
            json!({
                "action": "remove",
                "originalCwd": "/repo",
                "worktreePath": "/repo/.worktrees/feature",
                "worktreeBranch": "feature",
                "tmuxSessionName": "claude-feature",
                "discardedFiles": 2,
                "discardedCommits": 1,
                "message": "Removed worktree"
            }),
        )
        .unwrap() else {
            panic!("expected ExitWorktree output");
        };
        assert_eq!(output.action, WorktreeExitAction::Remove);
        assert_eq!(output.discarded_files, Some(2));

        let Some(BuiltinToolOutput::Repl(output)) = BuiltinToolOutput::parse(
            "REPL",
            json!({
                "code": "return 1",
                "result": {"value": 1},
                "stdout": "ok\\n",
                "stderr": "",
                "registeredTools": ["inspect"],
                "images": [{"base64": "abc", "mediaType": "image/png"}],
                "documents": [{"base64": "pdf"}]
            }),
        )
        .unwrap() else {
            panic!("expected REPL output");
        };
        assert_eq!(output.result["value"], 1);
        assert_eq!(output.registered_tools.unwrap()[0], "inspect");
        assert_eq!(output.images.unwrap()[0].media_type, "image/png");
        assert_eq!(output.documents.unwrap()[0].base64, "pdf");

        let Some(BuiltinToolOutput::CronCreate(output)) = BuiltinToolOutput::parse(
            "CronCreate",
            json!({
                "id": "job-1",
                "humanSchedule": "Every 5 minutes",
                "recurring": true,
                "durable": true
            }),
        )
        .unwrap() else {
            panic!("expected CronCreate output");
        };
        assert_eq!(output.human_schedule, "Every 5 minutes");
        assert_eq!(output.durable, Some(true));

        let Some(BuiltinToolOutput::CronDelete(output)) =
            BuiltinToolOutput::parse("CronDelete", json!({"id": "job-1"})).unwrap()
        else {
            panic!("expected CronDelete output");
        };
        assert_eq!(output.id, "job-1");

        let Some(BuiltinToolOutput::CronList(output)) = BuiltinToolOutput::parse(
            "CronList",
            json!({
                "jobs": [{
                    "id": "job-1",
                    "cron": "*/5 * * * *",
                    "humanSchedule": "Every 5 minutes",
                    "prompt": "check status",
                    "recurring": true,
                    "durable": false
                }]
            }),
        )
        .unwrap() else {
            panic!("expected CronList output");
        };
        assert_eq!(output.jobs[0].prompt, "check status");

        let Some(BuiltinToolOutput::ScheduleWakeup(output)) = BuiltinToolOutput::parse(
            "ScheduleWakeup",
            json!({
                "scheduledFor": 1_733_000_000_000u64,
                "clampedDelaySeconds": 60,
                "wasClamped": true
            }),
        )
        .unwrap() else {
            panic!("expected ScheduleWakeup output");
        };
        assert_eq!(output.clamped_delay_seconds, 60);
        assert!(output.was_clamped);

        let Some(BuiltinToolOutput::RemoteTrigger(output)) = BuiltinToolOutput::parse(
            "RemoteTrigger",
            json!({"status": 200, "json": r#"{"ok":true}"#, "summary": "created"}),
        )
        .unwrap() else {
            panic!("expected RemoteTrigger output");
        };
        assert_eq!(output.status, 200);
        assert_eq!(output.summary.as_deref(), Some("created"));

        let Some(BuiltinToolOutput::PushNotification(output)) = BuiltinToolOutput::parse(
            "PushNotification",
            json!({
                "message": "Build finished",
                "pushSent": false,
                "localSent": true,
                "disabledReason": "user_present",
                "idleSec": 4,
                "hasFocus": true,
                "sentAt": "2026-06-04T03:00:00Z"
            }),
        )
        .unwrap() else {
            panic!("expected PushNotification output");
        };
        assert_eq!(
            output.disabled_reason,
            Some(PushNotificationDisabledReason::UserPresent)
        );
        assert_eq!(output.idle_sec, Some(4));
    }

    #[test]
    fn tool_helper_preserves_documented_sdk_mcp_metadata() {
        let tool = tool(
            "search",
            "Search indexed docs",
            json!({"type": "object"}),
            |_args, _extra| async {
                Ok(CallToolResult {
                    content: vec![],
                    structured_content: None,
                    is_error: None,
                })
            },
        )
        .search_hint("Use for docs lookup")
        .always_load(true)
        .meta(
            "anthropic/permissionDisplay",
            json!({"displayName": "Docs"}),
        );

        assert_eq!(tool.meta["anthropic/searchHint"], "Use for docs lookup");
        assert_eq!(tool.meta["anthropic/alwaysLoad"], true);
        assert_eq!(
            tool.meta["anthropic/permissionDisplay"]["displayName"],
            "Docs"
        );

        let tool = tool.always_load(false);
        assert!(!tool.meta.contains_key("anthropic/alwaysLoad"));
        let explicit_false = tool.meta("anthropic/alwaysLoad", json!(false));
        assert_eq!(explicit_false.meta["anthropic/alwaysLoad"], false);
    }

    #[test]
    fn json_schema_builder_generates_documented_object_schema() {
        let schema = JsonSchema::object()
            .required_property(
                "query",
                JsonSchema::string().description("Search query text"),
            )
            .optional_property("limit", JsonSchema::integer().minimum(1))
            .required_property("mode", JsonSchema::string_enum(["fast", "deep"]))
            .required_property(
                "filters",
                JsonSchema::array(
                    JsonSchema::object()
                        .required_property("field", JsonSchema::string())
                        .required_property(
                            "value",
                            JsonSchema::any_of([
                                JsonSchema::string(),
                                JsonSchema::number(),
                                JsonSchema::boolean(),
                            ]),
                        )
                        .build(),
                ),
            )
            .additional_properties(false)
            .build();
        let value: Value = schema.into();

        assert_eq!(value["type"], "object");
        assert_eq!(value["properties"]["query"]["type"], "string");
        assert_eq!(
            value["properties"]["query"]["description"],
            "Search query text"
        );
        assert_eq!(value["properties"]["limit"]["minimum"], 1);
        assert_eq!(value["properties"]["mode"]["enum"], json!(["fast", "deep"]));
        assert_eq!(value["required"], json!(["query", "mode", "filters"]));
        assert_eq!(value["additionalProperties"], false);
        assert_eq!(
            value["properties"]["filters"]["items"]["properties"]["value"]["anyOf"][1]["type"],
            "number"
        );
        assert_eq!(
            serde_json::to_value(JsonSchema::string()).unwrap(),
            json!({"type": "string"})
        );

        let raw_value_schema = JsonSchema::object()
            .optional_property("raw", json!({"type": "string"}))
            .build()
            .into_value();
        assert_eq!(raw_value_schema["properties"]["raw"]["type"], "string");

        let overridden = JsonSchema::object()
            .required_property("draft", JsonSchema::boolean())
            .optional_property("draft", JsonSchema::boolean())
            .build()
            .into_value();
        assert!(overridden.get("required").is_none());
    }
}
