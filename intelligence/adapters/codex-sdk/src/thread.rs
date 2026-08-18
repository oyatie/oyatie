use std::path::PathBuf;

use crate::error::{CodexError, Result};
use crate::events::{ThreadEvent, Usage};
use crate::exec::{CodexExec, CodexExecArgs, LineStream};
use crate::input::{Input, normalize_input};
use crate::items::ThreadItem;
use crate::options::{CodexOptions, ThreadOptions, TurnOptions};
use crate::schema::{OutputSchemaFile, create_output_schema_file};

/// Completed turn.
#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub items: Vec<ThreadItem>,
    pub final_response: String,
    pub usage: Option<Usage>,
}

/// Alias for the result of `Thread::run`.
pub type RunResult = Turn;

/// The result of `Thread::run_streamed`.
pub struct StreamedTurn<'a> {
    pub events: EventStream<'a>,
}

/// Alias for the result of `Thread::run_streamed`.
pub type RunStreamedResult<'a> = StreamedTurn<'a>;

/// A thread of conversation with the agent.
#[derive(Clone, Debug)]
pub struct Thread {
    exec: CodexExec,
    options: CodexOptions,
    id: Option<String>,
    thread_options: ThreadOptions,
}

impl Thread {
    pub(crate) fn new(
        exec: CodexExec,
        options: CodexOptions,
        thread_options: ThreadOptions,
        id: Option<String>,
    ) -> Self {
        Self {
            exec,
            options,
            id,
            thread_options,
        }
    }

    /// Returns the thread id after the first turn starts, or immediately for a resumed thread.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Provides input to the agent and streams events as they are produced.
    pub fn run_streamed<I>(
        &mut self,
        input: I,
        turn_options: TurnOptions,
    ) -> Result<StreamedTurn<'_>>
    where
        I: Into<Input>,
    {
        let schema = create_output_schema_file(turn_options.output_schema.as_ref())?;
        let (prompt, images) = normalize_input(input.into());
        let exec_args = self.exec_args(prompt, images, schema.path().map(PathBuf::from));
        let lines = self.exec.run(exec_args)?;
        Ok(StreamedTurn {
            events: EventStream {
                lines,
                thread_id: &mut self.id,
                _schema: schema,
            },
        })
    }

    /// Provides input to the agent and returns the completed turn.
    pub fn run<I>(&mut self, input: I, turn_options: TurnOptions) -> Result<Turn>
    where
        I: Into<Input>,
    {
        let mut streamed = self.run_streamed(input, turn_options)?;
        let mut items = Vec::new();
        let mut final_response = String::new();
        let mut usage = None;

        for event in &mut streamed.events {
            match event? {
                ThreadEvent::ItemCompleted(event) => {
                    if let ThreadItem::AgentMessage(message) = &event.item {
                        final_response = message.text.clone();
                    }
                    items.push(event.item);
                }
                ThreadEvent::TurnCompleted(event) => usage = Some(event.usage),
                ThreadEvent::TurnFailed(event) => {
                    return Err(CodexError::TurnFailed {
                        message: event.error.message,
                    });
                }
                _ => {}
            }
        }

        Ok(Turn {
            items,
            final_response,
            usage,
        })
    }

    fn exec_args(
        &self,
        input: String,
        images: Vec<PathBuf>,
        output_schema_file: Option<PathBuf>,
    ) -> CodexExecArgs {
        CodexExecArgs {
            input,
            base_url: self.options.base_url.clone(),
            api_key: self.options.api_key.clone(),
            thread_id: self.id.clone(),
            images,
            model: self.thread_options.model.clone(),
            sandbox_mode: self.thread_options.sandbox_mode,
            working_directory: self.thread_options.working_directory.clone(),
            additional_directories: self.thread_options.additional_directories.clone(),
            skip_git_repo_check: self.thread_options.skip_git_repo_check,
            output_schema_file,
            model_reasoning_effort: self.thread_options.model_reasoning_effort,
            network_access_enabled: self.thread_options.network_access_enabled,
            web_search_mode: self.thread_options.web_search_mode,
            web_search_enabled: self.thread_options.web_search_enabled,
            approval_policy: self.thread_options.approval_policy,
        }
    }
}

/// Iterator over parsed Codex JSONL events.
pub struct EventStream<'a> {
    pub(crate) lines: LineStream,
    pub(crate) thread_id: &'a mut Option<String>,
    pub(crate) _schema: OutputSchemaFile,
}

impl Iterator for EventStream<'_> {
    type Item = Result<ThreadEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?;
        match line {
            Ok(line) => {
                let parsed: Result<ThreadEvent> =
                    serde_json::from_str(&line).map_err(CodexError::from);
                if let Ok(ThreadEvent::ThreadStarted(event)) = &parsed {
                    *self.thread_id = Some(event.thread_id.clone());
                }
                Some(parsed)
            }
            Err(err) => Some(Err(err)),
        }
    }
}
