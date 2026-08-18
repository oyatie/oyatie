#![cfg(unix)]

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use intelligence_codex_sdk::{AppServerClient, AppServerConfig};
use serde_json::Value;
use tempfile::TempDir;

use std::os::unix::fs::PermissionsExt;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct FakeRuntime {
    _dir: TempDir,
    explicit_path: PathBuf,
    packaged_path: PathBuf,
    messages_file: PathBuf,
    args_file: PathBuf,
}

impl FakeRuntime {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let explicit_path = dir.path().join("explicit-codex");
        let packaged_path = dir.path().join("packaged-codex");
        let messages_file = dir.path().join("messages.jsonl");
        let args_file = dir.path().join("args.txt");
        write_fake_codex(&explicit_path, "explicit");
        write_fake_codex(&packaged_path, "packaged");
        Self {
            _dir: dir,
            explicit_path,
            packaged_path,
            messages_file,
            args_file,
        }
    }

    fn env(&self) -> HashMap<String, String> {
        HashMap::from([
            (
                "CODEX_APP_MESSAGES_FILE".to_string(),
                self.messages_file.display().to_string(),
            ),
            (
                "CODEX_APP_ARGS_FILE".to_string(),
                self.args_file.display().to_string(),
            ),
        ])
    }

    fn messages(&self) -> Vec<Value> {
        fs::read_to_string(&self.messages_file)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // SAFETY: env mutation in tests; the suite runs with --test-threads=1.
        if let Some(previous) = &self.previous {
            unsafe { std::env::set_var(self.key, previous) };
        } else {
            unsafe { std::env::remove_var(self.key) };
        }
    }
}

fn set_env_var(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> EnvVarGuard {
    let previous = std::env::var_os(key);
    // SAFETY: env mutation in tests; the suite runs with --test-threads=1.
    unsafe { std::env::set_var(key, value) };
    EnvVarGuard { key, previous }
}

#[cfg(feature = "runtime")]
fn remove_env_var(key: &'static str) -> EnvVarGuard {
    let previous = std::env::var_os(key);
    // SAFETY: env mutation in tests; the suite runs with --test-threads=1.
    unsafe { std::env::remove_var(key) };
    EnvVarGuard { key, previous }
}

fn write_fake_codex(path: &Path, marker: &str) {
    fs::write(
        path,
        format!(
            r#"#!/usr/bin/env python3
import json
import os
import sys
messages_path = os.environ["CODEX_APP_MESSAGES_FILE"]
args_path = os.environ["CODEX_APP_ARGS_FILE"]
with open(args_path, "a", encoding="utf-8") as args_file:
    args_file.write({marker:?} + "\n")
    for arg in sys.argv[1:]:
        args_file.write(arg + "\n")
for line in sys.stdin:
    message = json.loads(line)
    with open(messages_path, "a", encoding="utf-8") as messages_file:
        messages_file.write(json.dumps(message, sort_keys=True) + "\n")
    if "id" in message and message.get("method") == "initialize":
        print(json.dumps({{"id": message["id"], "result": {{"serverInfo": {{"name": {marker:?}, "version": "fake"}}}}}}), flush=True)
"#
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

#[cfg(feature = "runtime")]
fn write_fake_shell_codex(path: &Path, marker: &str) {
    fs::write(
        path,
        format!(
            r#"#!/bin/sh
messages_path="$CODEX_APP_MESSAGES_FILE"
args_path="$CODEX_APP_ARGS_FILE"
{{
  printf '%s\n' {marker:?}
  for arg in "$@"; do
    printf '%s\n' "$arg"
  done
}} >> "$args_path"
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$messages_path"
  case "$line" in
    *'"method":"initialize"'*|*'"method": "initialize"'*)
      printf '%s\n' '{{"id":"1","result":{{"serverInfo":{{"name":{marker:?},"version":"fake"}}}}}}'
      ;;
  esac
done
"#
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

#[test]
fn explicit_codex_path_override_wins_over_packaged_runtime_env() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let fake = FakeRuntime::new();
    let _runtime_bin = set_env_var("OPENAI_CODEX_RUNTIME_BIN", &fake.packaged_path);

    let client = AppServerClient::new(
        AppServerConfig::new()
            .with_codex_path_override(&fake.explicit_path)
            .with_env(fake.env()),
    );
    let init = client.initialize().unwrap();
    client.close();

    assert_eq!(init.server_info.unwrap().name.as_deref(), Some("explicit"));
    assert!(
        fake.messages()
            .iter()
            .any(|message| message["method"] == "initialize")
    );
}

#[cfg(feature = "runtime")]
#[test]
fn runtime_feature_uses_packaged_runtime_env_when_no_explicit_override() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let fake = FakeRuntime::new();
    let _runtime_bin = set_env_var("OPENAI_CODEX_RUNTIME_BIN", &fake.packaged_path);

    let client = AppServerClient::new(AppServerConfig::new().with_env(fake.env()));
    let init = client.initialize().unwrap();
    client.close();

    assert_eq!(init.server_info.unwrap().name.as_deref(), Some("packaged"));
}

#[cfg(feature = "runtime")]
#[test]
fn runtime_feature_prepends_runtime_path_dirs_for_codex_lookup() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let fake = FakeRuntime::new();
    let runtime_bin_dir = fake._dir.path().join("runtime-bin");
    let path_without_codex = fake._dir.path().join("path-without-codex");
    fs::create_dir(&runtime_bin_dir).unwrap();
    fs::create_dir(&path_without_codex).unwrap();
    write_fake_shell_codex(&runtime_bin_dir.join("codex"), "runtime-path-dir");

    let _runtime_bin = remove_env_var("OPENAI_CODEX_RUNTIME_BIN");
    let _runtime_path_dirs = set_env_var("OPENAI_CODEX_RUNTIME_PATH_DIRS", &runtime_bin_dir);
    let _path = set_env_var("PATH", &path_without_codex);

    let client = AppServerClient::new(AppServerConfig::new().with_env(fake.env()));
    let init = client.initialize().unwrap();
    client.close();

    assert_eq!(
        init.server_info.unwrap().name.as_deref(),
        Some("runtime-path-dir")
    );
}
