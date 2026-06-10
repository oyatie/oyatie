use crate::exec::CodexExec;
use crate::options::{CodexOptions, ThreadOptions};
use crate::thread::Thread;

/// Main entry point for interacting with the Codex agent.
///
/// The SDK wraps the Codex CLI JSONL mode documented by the existing
/// TypeScript SDK: <https://github.com/openai/codex/tree/main/sdk/typescript>.
#[derive(Clone, Debug)]
pub struct Codex {
    exec: CodexExec,
    options: CodexOptions,
}

impl Codex {
    pub fn new(options: CodexOptions) -> Self {
        let exec = CodexExec::new(
            options.codex_path_override.clone(),
            options.env.clone(),
            options.config.clone(),
        );
        Self { exec, options }
    }

    /// Starts a new conversation with an agent.
    pub fn start_thread(&self, options: ThreadOptions) -> Thread {
        Thread::new(self.exec.clone(), self.options.clone(), options, None)
    }

    /// Resumes a conversation with an agent based on a thread id.
    pub fn resume_thread(&self, id: impl Into<String>, options: ThreadOptions) -> Thread {
        Thread::new(
            self.exec.clone(),
            self.options.clone(),
            options,
            Some(id.into()),
        )
    }
}

impl Default for Codex {
    fn default() -> Self {
        Self::new(CodexOptions::default())
    }
}
