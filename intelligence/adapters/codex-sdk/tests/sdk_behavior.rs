#![cfg(unix)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use intelligence_codex_sdk::{
    ApprovalMode, Codex, CodexError, CodexOptions, Input, ModelReasoningEffort, SandboxMode,
    ThreadEvent, ThreadOptions, TurnOptions, UserInput, WebSearchMode,
};
use serde_json::{json, Map, Value};
use std::sync::OnceLock;

use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// The fake executable, written EXACTLY ONCE per test process.
///
/// Each test used to write its own copy and then exec it. That races: `Command::spawn` forks
/// before it execs, and the forked child inherits every open descriptor — `CLOEXEC` only clears
/// them AT exec, so a write descriptor stays open across the child's whole fork→exec window. One
/// test writing while another forked meant the writer's own exec hit ETXTBSY ("Text file busy").
/// CI hit exactly that in the sibling app-server contract suite.
///
/// Writing once behind a `OnceLock` removes the window structurally rather than by retrying: every
/// test here must obtain this path before it can spawn, so none can be forking during the single
/// write. Other test binaries are separate processes and cannot inherit these descriptors.
///
/// The `TempDir` is deliberately leaked — it must outlive every test, and a `static` is never
/// dropped, so binding it would only leave a dangling path.
static FAKE_CODEX: OnceLock<PathBuf> = OnceLock::new();

fn fake_codex_path() -> PathBuf {
    FAKE_CODEX
        .get_or_init(|| {
            let dir: &'static TempDir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
            let path = dir.path().join("codex");
            write_fake_script(&path);
            path
        })
        .clone()
}

struct FakeCodex {
    _dir: TempDir,
    path: PathBuf,
    args_file: PathBuf,
    stdin_file: PathBuf,
    env_file: PathBuf,
    schema_status_file: PathBuf,
}

impl FakeCodex {
    fn new() -> Self {
        // The executable is shared and written once; the per-test sinks stay per-fake.
        let dir = tempfile::tempdir().unwrap();
        let args_file = dir.path().join("args.txt");
        let stdin_file = dir.path().join("stdin.txt");
        let env_file = dir.path().join("env.txt");
        let schema_status_file = dir.path().join("schema-status.txt");
        Self {
            _dir: dir,
            path: fake_codex_path(),
            args_file,
            stdin_file,
            env_file,
            schema_status_file,
        }
    }

    fn env(&self) -> HashMap<String, String> {
        HashMap::from([
            (
                "CODEX_ARGS_FILE".to_string(),
                self.args_file.display().to_string(),
            ),
            (
                "CODEX_STDIN_FILE".to_string(),
                self.stdin_file.display().to_string(),
            ),
            (
                "CODEX_ENV_FILE".to_string(),
                self.env_file.display().to_string(),
            ),
            (
                "CODEX_SCHEMA_STATUS_FILE".to_string(),
                self.schema_status_file.display().to_string(),
            ),
        ])
    }

    fn args(&self) -> Vec<String> {
        fs::read_to_string(&self.args_file)
            .unwrap()
            .lines()
            .map(ToOwned::to_owned)
            .collect()
    }

    fn stdin(&self) -> String {
        fs::read_to_string(&self.stdin_file).unwrap()
    }
}

fn write_fake_script(path: &Path) {
    fs::write(
        path,
        r#"#!/bin/sh
: > "$CODEX_ARGS_FILE"
for arg in "$@"; do
  printf '%s\n' "$arg" >> "$CODEX_ARGS_FILE"
done
/bin/cat > "$CODEX_STDIN_FILE"
if [ -n "${CODEX_ENV_FILE:-}" ]; then
  {
    printf '%s\n' "${CODEX_INTERNAL_ORIGINATOR_OVERRIDE:-}"
    printf '%s\n' "${CODEX_API_KEY:-}"
    printf '%s\n' "${CODEX_ENV_SHOULD_NOT_LEAK:-}"
  } > "$CODEX_ENV_FILE"
fi
if [ "${CODEX_CHECK_SCHEMA:-}" = "1" ]; then
  prev=''
  found=''
  for arg in "$@"; do
    if [ "$prev" = "--output-schema" ]; then
      found="$arg"
    fi
    prev="$arg"
  done
  if [ -f "$found" ]; then
    printf 'exists\n' > "$CODEX_SCHEMA_STATUS_FILE"
  else
    printf 'missing\n' > "$CODEX_SCHEMA_STATUS_FILE"
  fi
fi
if [ "${CODEX_EXIT_NONZERO:-}" = "1" ]; then
  if [ "${CODEX_LONG_STDERR:-}" = "1" ]; then
    python3 - <<'PY' >&2
import sys
sys.stderr.write("x" * 70000 + "tail-marker")
PY
  fi
  echo 'boom' >&2
  exit 7
fi
printf '%s\n' '{"type":"thread.started","thread_id":"thread-123"}'
printf '%s\n' '{"type":"turn.started"}'
if [ "${CODEX_TURN_FAILED:-}" = "1" ]; then
  printf '%s\n' '{"type":"turn.failed","error":{"message":"rate limit exceeded"}}'
  exit 0
fi
printf '%s\n' '{"type":"item.completed","item":{"id":"item-1","type":"agent_message","text":"Hi from fake Codex"}}'
printf '%s\n' '{"type":"turn.completed","usage":{"input_tokens":42,"cached_input_tokens":12,"output_tokens":5,"reasoning_output_tokens":0}}'
"#,
    )
    .unwrap();

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }
}

fn codex_with_fake(fake: &FakeCodex) -> Codex {
    Codex::new(
        CodexOptions::new()
            .with_codex_path_override(&fake.path)
            .with_env(fake.env()),
    )
}

#[test]
fn run_collects_agent_message_usage_and_thread_id() {
    let fake = FakeCodex::new();
    let codex = codex_with_fake(&fake);
    let mut thread = codex.start_thread(ThreadOptions::default());

    let turn = thread.run("Hello, world!", TurnOptions::default()).unwrap();

    assert_eq!(thread.id(), Some("thread-123"));
    assert_eq!(turn.final_response, "Hi from fake Codex");
    assert_eq!(turn.items.len(), 1);
    assert_eq!(turn.usage.unwrap().input_tokens, 42);
    assert_eq!(fake.stdin(), "Hello, world!");
    assert_eq!(fake.args()[..2], ["exec", "--experimental-json"]);
}

#[test]
fn run_streamed_yields_events_and_forwards_resume_before_images() {
    let fake = FakeCodex::new();
    let codex = codex_with_fake(&fake);
    let mut thread = codex.resume_thread("existing-thread", ThreadOptions::default());

    let streamed = thread
        .run_streamed(
            Input::from(vec![
                UserInput::text("Describe these screenshots"),
                UserInput::local_image("./ui.png"),
                UserInput::local_image("./diagram.jpg"),
            ]),
            TurnOptions::default(),
        )
        .unwrap();
    let events = streamed.events.collect::<Result<Vec<_>, _>>().unwrap();

    assert!(matches!(
        events.first(),
        Some(ThreadEvent::ThreadStarted(_))
    ));
    assert_eq!(thread.id(), Some("thread-123"));
    assert_eq!(fake.stdin(), "Describe these screenshots");

    let args = fake.args();
    let resume_index = args.iter().position(|arg| arg == "resume").unwrap();
    let image_index = args.iter().position(|arg| arg == "--image").unwrap();
    assert!(
        resume_index < image_index,
        "resume args must precede image args: {args:?}"
    );
    assert!(args
        .windows(2)
        .any(|pair| pair == ["resume", "existing-thread"]));
    assert_eq!(
        args.iter()
            .enumerate()
            .filter(|(_, arg)| *arg == "--image")
            .map(|(index, _)| args[index + 1].clone())
            .collect::<Vec<_>>(),
        vec!["./ui.png", "./diagram.jpg"]
    );
}

#[test]
fn forwards_global_and_thread_options_as_cli_args_and_env() {
    // SAFETY: env mutation in tests; the suite runs with --test-threads=1.
    unsafe { std::env::set_var("CODEX_ENV_SHOULD_NOT_LEAK", "leak") };
    let fake = FakeCodex::new();
    let config = json!({
        "approval_policy": "never",
        "sandbox_workspace_write": { "network_access": true },
        "retry_budget": 3,
        "tool_rules": { "allow": ["git status", "git diff"] }
    });
    let config = config
        .as_object()
        .cloned()
        .unwrap_or_else(Map::<String, Value>::new);
    let mut env = fake.env();
    env.insert("CUSTOM_ENV".to_string(), "custom".to_string());
    let codex = Codex::new(
        CodexOptions::new()
            .with_codex_path_override(&fake.path)
            .with_base_url("https://example.test")
            .with_api_key("test-key")
            .with_config(config)
            .with_env(env),
    );
    let working_directory = tempfile::tempdir().unwrap();
    let mut thread = codex.start_thread(
        ThreadOptions::new()
            .with_model("gpt-test-1")
            .with_sandbox_mode(SandboxMode::WorkspaceWrite)
            .with_working_directory(working_directory.path())
            .with_additional_directories(["../backend", "/tmp/shared"])
            .with_skip_git_repo_check(true)
            .with_model_reasoning_effort(ModelReasoningEffort::High)
            .with_network_access_enabled(true)
            .with_web_search_mode(WebSearchMode::Cached)
            .with_approval_policy(ApprovalMode::OnRequest),
    );

    thread.run("apply options", TurnOptions::default()).unwrap();
    // SAFETY: env mutation in tests; the suite runs with --test-threads=1.
    unsafe { std::env::remove_var("CODEX_ENV_SHOULD_NOT_LEAK") };

    let args = fake.args();
    assert_pair(&args, "--config", "approval_policy=\"never\"");
    assert_pair(
        &args,
        "--config",
        "sandbox_workspace_write.network_access=true",
    );
    assert_pair(&args, "--config", "retry_budget=3");
    assert_pair(
        &args,
        "--config",
        "tool_rules.allow=[\"git status\", \"git diff\"]",
    );
    assert_pair(
        &args,
        "--config",
        "openai_base_url=\"https://example.test\"",
    );
    assert_pair(&args, "--model", "gpt-test-1");
    assert_pair(&args, "--sandbox", "workspace-write");
    assert_pair(
        &args,
        "--cd",
        &working_directory.path().display().to_string(),
    );
    assert_pair(&args, "--add-dir", "../backend");
    assert_pair(&args, "--add-dir", "/tmp/shared");
    assert!(args.iter().any(|arg| arg == "--skip-git-repo-check"));
    assert_pair(&args, "--config", "model_reasoning_effort=\"high\"");
    assert_pair(&args, "--config", "web_search=\"cached\"");
    assert_pair(&args, "--config", "approval_policy=\"on-request\"");

    let env_lines = fs::read_to_string(&fake.env_file).unwrap();
    let env_lines = env_lines.lines().collect::<Vec<_>>();
    assert_eq!(env_lines, vec!["codex_sdk_rs", "test-key", ""]);
}

#[test]
fn output_schema_is_available_during_turn_and_removed_afterward() {
    let fake = FakeCodex::new();
    let mut env = fake.env();
    env.insert("CODEX_CHECK_SCHEMA".to_string(), "1".to_string());
    let codex = Codex::new(
        CodexOptions::new()
            .with_codex_path_override(&fake.path)
            .with_env(env),
    );
    let mut thread = codex.start_thread(ThreadOptions::default());
    let schema = json!({
        "type": "object",
        "properties": { "answer": { "type": "string" } },
        "required": ["answer"],
        "additionalProperties": false
    });

    thread
        .run("structured", TurnOptions::new().with_output_schema(schema))
        .unwrap();

    assert_eq!(
        fs::read_to_string(&fake.schema_status_file).unwrap(),
        "exists\n"
    );
    let args = fake.args();
    let schema_path = args[args
        .iter()
        .position(|arg| arg == "--output-schema")
        .unwrap()
        + 1]
    .clone();
    assert!(
        !Path::new(&schema_path).exists(),
        "schema temp dir should be cleaned after run"
    );
}

#[test]
fn turn_failed_event_returns_error() {
    let fake = FakeCodex::new();
    let mut env = fake.env();
    env.insert("CODEX_TURN_FAILED".to_string(), "1".to_string());
    let codex = Codex::new(
        CodexOptions::new()
            .with_codex_path_override(&fake.path)
            .with_env(env),
    );
    let mut thread = codex.start_thread(ThreadOptions::default());

    let err = thread.run("fail", TurnOptions::default()).unwrap_err();

    assert!(matches!(err, CodexError::TurnFailed { message } if message == "rate limit exceeded"));
}

#[test]
fn nonzero_cli_exit_returns_stderr() {
    let fake = FakeCodex::new();
    let mut env = fake.env();
    env.insert("CODEX_EXIT_NONZERO".to_string(), "1".to_string());
    let codex = Codex::new(
        CodexOptions::new()
            .with_codex_path_override(&fake.path)
            .with_env(env),
    );
    let mut thread = codex.start_thread(ThreadOptions::default());

    let err = thread.run("boom", TurnOptions::default()).unwrap_err();

    assert!(
        matches!(err, CodexError::CliExit { code: Some(7), stderr } if stderr.contains("boom"))
    );
}

#[test]
fn nonzero_cli_exit_stderr_is_bounded_to_tail() {
    let fake = FakeCodex::new();
    let mut env = fake.env();
    env.insert("CODEX_EXIT_NONZERO".to_string(), "1".to_string());
    env.insert("CODEX_LONG_STDERR".to_string(), "1".to_string());
    let codex = Codex::new(
        CodexOptions::new()
            .with_codex_path_override(&fake.path)
            .with_env(env),
    );
    let mut thread = codex.start_thread(ThreadOptions::default());

    let err = thread.run("boom", TurnOptions::default()).unwrap_err();

    match err {
        CodexError::CliExit {
            code: Some(7),
            stderr,
        } => {
            assert!(stderr.len() <= 64 * 1024);
            assert!(stderr.contains("tail-marker"));
            assert!(stderr.contains("boom"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

fn assert_pair(args: &[String], flag: &str, value: &str) {
    assert!(
        args.windows(2).any(|pair| pair == [flag, value]),
        "expected pair [{flag:?}, {value:?}] in {args:?}"
    );
}
