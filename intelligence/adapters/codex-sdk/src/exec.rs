use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::thread::{self, JoinHandle};

use serde_json::{Map, Value};

use crate::error::{CodexError, Result};
use crate::options::{ApprovalMode, ModelReasoningEffort, SandboxMode, WebSearchMode};

const INTERNAL_ORIGINATOR_ENV: &str = "CODEX_INTERNAL_ORIGINATOR_OVERRIDE";
const RUST_SDK_ORIGINATOR: &str = "codex_sdk_rs";
const STDERR_TAIL_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug)]
pub(crate) struct CodexExec {
    executable_path: PathBuf,
    env_override: Option<HashMap<String, String>>,
    config_overrides: Option<Map<String, Value>>,
}

impl CodexExec {
    pub(crate) fn new(
        executable_path: Option<PathBuf>,
        env_override: Option<HashMap<String, String>>,
        config_overrides: Option<Map<String, Value>>,
    ) -> Self {
        Self {
            executable_path: executable_path.unwrap_or_else(|| PathBuf::from("codex")),
            env_override,
            config_overrides,
        }
    }

    /// Spawns `codex exec --experimental-json` and returns a line stream.
    ///
    /// Uses `std::process::Command` with separate args (not shell splitting), per
    /// official Rust process docs:
    /// <https://doc.rust-lang.org/std/process/struct.Command.html>.
    pub(crate) fn run(&self, args: CodexExecArgs) -> Result<LineStream> {
        let command_args = self.command_args(&args)?;
        let mut command = Command::new(&self.executable_path);
        command.args(&command_args);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let env = self.process_env(args.api_key.as_deref());
        if self.env_override.is_some() {
            command.env_clear();
        }
        command.envs(env);

        let mut child = command.spawn()?;
        let stdin = child.stdin.take().ok_or(CodexError::MissingPipe("stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or(CodexError::MissingPipe("stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or(CodexError::MissingPipe("stderr"))?;
        let stderr_handle = drain_stderr(stderr);
        let stdin_handle = write_stdin(args.input, stdin);

        Ok(LineStream {
            reader: BufReader::new(stdout),
            child: Some(child),
            stderr_handle: Some(stderr_handle),
            stdin_handle: Some(stdin_handle),
            finished: false,
        })
    }

    pub(crate) fn command_args(&self, args: &CodexExecArgs) -> Result<Vec<String>> {
        // Mirrors the TypeScript SDK transport order:
        // https://github.com/openai/codex/blob/main/sdk/typescript/src/exec.ts
        let mut command_args = vec!["exec".to_string(), "--experimental-json".to_string()];

        if let Some(config) = &self.config_overrides {
            for override_value in serialize_config_overrides(config)? {
                command_args.push("--config".to_string());
                command_args.push(override_value);
            }
        }

        if let Some(base_url) = &args.base_url {
            command_args.push("--config".to_string());
            command_args.push(format!(
                "openai_base_url={}",
                to_toml_value(&Value::String(base_url.clone()), "openai_base_url")?
            ));
        }

        if let Some(model) = &args.model {
            command_args.push("--model".to_string());
            command_args.push(model.clone());
        }

        if let Some(sandbox_mode) = args.sandbox_mode {
            command_args.push("--sandbox".to_string());
            command_args.push(sandbox_mode.as_cli().to_string());
        }

        if let Some(working_directory) = &args.working_directory {
            command_args.push("--cd".to_string());
            command_args.push(path_to_string(working_directory));
        }

        for directory in &args.additional_directories {
            command_args.push("--add-dir".to_string());
            command_args.push(path_to_string(directory));
        }

        if args.skip_git_repo_check {
            command_args.push("--skip-git-repo-check".to_string());
        }

        if let Some(output_schema_file) = &args.output_schema_file {
            command_args.push("--output-schema".to_string());
            command_args.push(path_to_string(output_schema_file));
        }

        if let Some(effort) = args.model_reasoning_effort {
            command_args.push("--config".to_string());
            command_args.push(format!("model_reasoning_effort=\"{}\"", effort.as_cli()));
        }

        if let Some(enabled) = args.network_access_enabled {
            command_args.push("--config".to_string());
            command_args.push(format!("sandbox_workspace_write.network_access={enabled}"));
        }

        if let Some(mode) = args.web_search_mode {
            command_args.push("--config".to_string());
            command_args.push(format!("web_search=\"{}\"", mode.as_cli()));
        } else if let Some(enabled) = args.web_search_enabled {
            command_args.push("--config".to_string());
            command_args.push(if enabled {
                "web_search=\"live\"".to_string()
            } else {
                "web_search=\"disabled\"".to_string()
            });
        }

        if let Some(approval_policy) = args.approval_policy {
            command_args.push("--config".to_string());
            command_args.push(format!("approval_policy=\"{}\"", approval_policy.as_cli()));
        }

        if let Some(thread_id) = &args.thread_id {
            command_args.push("resume".to_string());
            command_args.push(thread_id.clone());
        }

        for image in &args.images {
            command_args.push("--image".to_string());
            command_args.push(path_to_string(image));
        }

        Ok(command_args)
    }

    fn process_env(&self, api_key: Option<&str>) -> HashMap<String, String> {
        let mut env_map: HashMap<String, String> = if let Some(env_override) = &self.env_override {
            env_override.clone()
        } else {
            env::vars().collect()
        };

        env_map
            .entry(INTERNAL_ORIGINATOR_ENV.to_string())
            .or_insert_with(|| RUST_SDK_ORIGINATOR.to_string());
        if let Some(api_key) = api_key {
            env_map.insert("CODEX_API_KEY".to_string(), api_key.to_string());
        }
        env_map
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CodexExecArgs {
    pub(crate) input: String,
    pub(crate) base_url: Option<String>,
    pub(crate) api_key: Option<String>,
    pub(crate) thread_id: Option<String>,
    pub(crate) images: Vec<PathBuf>,
    pub(crate) model: Option<String>,
    pub(crate) sandbox_mode: Option<SandboxMode>,
    pub(crate) working_directory: Option<PathBuf>,
    pub(crate) additional_directories: Vec<PathBuf>,
    pub(crate) skip_git_repo_check: bool,
    pub(crate) output_schema_file: Option<PathBuf>,
    pub(crate) model_reasoning_effort: Option<ModelReasoningEffort>,
    pub(crate) network_access_enabled: Option<bool>,
    pub(crate) web_search_mode: Option<WebSearchMode>,
    pub(crate) web_search_enabled: Option<bool>,
    pub(crate) approval_policy: Option<ApprovalMode>,
}

/// Iterator over stdout JSONL lines from a Codex CLI child process.
///
/// `BufRead::read_line` reads through newlines and returns `Ok(0)` on EOF;
/// see official Rust docs:
/// <https://doc.rust-lang.org/std/io/trait.BufRead.html#method.read_line>.
pub(crate) struct LineStream {
    reader: BufReader<ChildStdout>,
    child: Option<Child>,
    stderr_handle: Option<JoinHandle<std::io::Result<String>>>,
    stdin_handle: Option<JoinHandle<std::io::Result<()>>>,
    finished: bool,
}

impl Iterator for LineStream {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => {
                self.finished = true;
                match self.wait_for_exit() {
                    Ok(()) => None,
                    Err(err) => Some(Err(err)),
                }
            }
            Ok(_) => {
                while matches!(line.as_bytes().last(), Some(b'\n' | b'\r')) {
                    line.pop();
                }
                Some(Ok(line))
            }
            Err(err) => {
                self.finished = true;
                self.kill_child();
                Some(Err(CodexError::Io(err)))
            }
        }
    }
}

impl LineStream {
    fn wait_for_exit(&mut self) -> Result<()> {
        let Some(mut child) = self.child.take() else {
            return Ok(());
        };
        let status = child.wait()?;
        let stderr = self.take_stderr();
        let stdin_result = self.take_stdin_result();
        if status.success() {
            stdin_result
        } else {
            Err(CodexError::CliExit {
                code: status.code(),
                stderr,
            })
        }
    }

    fn take_stderr(&mut self) -> String {
        match self.stderr_handle.take() {
            Some(handle) => match handle.join() {
                Ok(Ok(stderr)) => stderr,
                Ok(Err(err)) => format!("<failed to read stderr: {err}>"),
                Err(_) => "<stderr reader thread panicked>".to_string(),
            },
            None => String::new(),
        }
    }

    fn take_stdin_result(&mut self) -> Result<()> {
        match self.stdin_handle.take() {
            Some(handle) => match handle.join() {
                Ok(Ok(())) => Ok(()),
                Ok(Err(err)) => Err(CodexError::Io(err)),
                Err(_) => Err(CodexError::Io(std::io::Error::other(
                    "stdin writer thread panicked",
                ))),
            },
            None => Ok(()),
        }
    }

    fn kill_child(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;
        let _ = self.take_stdin_result();
        let _ = self.take_stderr();
    }
}

impl Drop for LineStream {
    fn drop(&mut self) {
        if !self.finished {
            self.kill_child();
            self.finished = true;
        }
    }
}

pub(crate) fn serialize_config_overrides(
    config_overrides: &Map<String, Value>,
) -> Result<Vec<String>> {
    let mut overrides = Vec::new();
    flatten_config_overrides(config_overrides, "", &mut overrides)?;
    Ok(overrides)
}

fn flatten_config_overrides(
    value: &Map<String, Value>,
    prefix: &str,
    overrides: &mut Vec<String>,
) -> Result<()> {
    if prefix.is_empty() && value.is_empty() {
        return Ok(());
    }
    if !prefix.is_empty() && value.is_empty() {
        overrides.push(format!("{prefix}={{}}"));
        return Ok(());
    }

    for (key, child) in value {
        if key.is_empty() {
            return Err(CodexError::InvalidConfig(
                "keys must be non-empty strings".to_string(),
            ));
        }
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        if let Value::Object(child_object) = child {
            flatten_config_overrides(child_object, &path, overrides)?;
        } else {
            overrides.push(format!("{path}={}", to_toml_value(child, &path)?));
        }
    }
    Ok(())
}

fn to_toml_value(value: &Value, path: &str) -> Result<String> {
    match value {
        Value::String(value) => serde_json::to_string(value).map_err(CodexError::from),
        Value::Number(number) => Ok(number.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Array(values) => {
            let rendered = values
                .iter()
                .enumerate()
                .map(|(index, item)| to_toml_value(item, &format!("{path}[{index}]")))
                .collect::<Result<Vec<_>>>()?;
            Ok(format!("[{}]", rendered.join(", ")))
        }
        Value::Object(object) => {
            let mut parts = Vec::new();
            for (key, child) in object {
                if key.is_empty() {
                    return Err(CodexError::InvalidConfig(
                        "keys must be non-empty strings".to_string(),
                    ));
                }
                parts.push(format!(
                    "{} = {}",
                    format_toml_key(key),
                    to_toml_value(child, &format!("{path}.{key}"))?
                ));
            }
            Ok(format!("{{{}}}", parts.join(", ")))
        }
        Value::Null => Err(CodexError::InvalidConfig(format!("{path} cannot be null"))),
    }
}

fn format_toml_key(key: &str) -> String {
    if key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        key.to_string()
    } else {
        serde_json::to_string(key).expect("serializing a string key cannot fail")
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn drain_stderr(stderr: ChildStderr) -> JoinHandle<std::io::Result<String>> {
    thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut tail = Vec::new();
        let mut buffer = [0_u8; 8192];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                return Ok(String::from_utf8_lossy(&tail).into_owned());
            }
            tail.extend_from_slice(&buffer[..read]);
            if tail.len() > STDERR_TAIL_BYTES {
                let excess = tail.len() - STDERR_TAIL_BYTES;
                tail.drain(0..excess);
            }
        }
    })
}

fn write_stdin(input: String, mut stdin: ChildStdin) -> JoinHandle<std::io::Result<()>> {
    thread::spawn(move || stdin.write_all(input.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_config_overrides_like_typescript_sdk() {
        let config = json!({
            "approval_policy": "never",
            "sandbox_workspace_write": { "network_access": true },
            "retry_budget": 3,
            "tool_rules": { "allow": ["git status", "git diff"] },
            "quoted.key": "value"
        });
        let object = config.as_object().unwrap();
        let mut overrides = serialize_config_overrides(object).unwrap();
        // Compare order-agnostic: `serde_json::Map` iteration order depends on the
        // resolved `preserve_order` feature (sorted under cargo's default, insertion
        // order under the buck2 third-party feature union); the --config override
        // contract is a SET of key=value pairs, not an ordering.
        overrides.sort();
        assert_eq!(
            overrides,
            vec![
                "approval_policy=\"never\"",
                "quoted.key=\"value\"",
                "retry_budget=3",
                "sandbox_workspace_write.network_access=true",
                "tool_rules.allow=[\"git status\", \"git diff\"]",
            ]
        );
    }

    #[test]
    fn rejects_null_config_values() {
        let config = json!({ "a": null });
        let err = serialize_config_overrides(config.as_object().unwrap()).unwrap_err();
        assert!(matches!(err, CodexError::InvalidConfig(_)));
    }
}
