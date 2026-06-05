//! Read-only agent hook runtime manifest checker.
//!
//! Runtime hooks are advisory stdout/stderr surfaces. This checker is the
//! Buck2/Prow-owned evidence path that validates the manifest, checked-in
//! Codex/Gemini configs, and hook files stay aligned without adding runtime
//! mutation authority.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_PATH: &str = "specs/agent-hook-runtime-manifest.json";
const CODEX_CONFIG_PATH: &str = ".codex/hooks.json";
const GEMINI_CONFIG_PATH: &str = ".gemini/settings.json";
const CLAUDE_CONFIG_PATH: &str = ".claude/settings.json";
const CHECKER_TARGET: &str = "//:agent-hook-runtime-manifest-check";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub manifest_path: String,
    pub runtime_hook_count: usize,
    pub config_reference_count: usize,
    pub retired_surface_count: usize,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHook {
    pub path: String,
    pub command: String,
    pub object: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigCommand {
    pub config_path: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedInputs {
    pub manifest: String,
    pub codex_config: String,
    pub gemini_config: String,
    pub claude_config: String,
    pub hooks: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub repo_root: PathBuf,
    pub json: bool,
}

pub fn evaluate(repo_root: &Path) -> Result<Evaluation, String> {
    let manifest = read_repo_file(repo_root, MANIFEST_PATH)?;
    let runtime_hooks = parse_runtime_hooks(&manifest)?;
    let mut hook_texts = BTreeMap::new();
    for hook in &runtime_hooks {
        let text = read_repo_file(repo_root, &hook.path)?;
        hook_texts.insert(hook.path.clone(), text);
    }
    let inputs = LoadedInputs {
        manifest,
        codex_config: read_repo_file(repo_root, CODEX_CONFIG_PATH)?,
        gemini_config: read_repo_file(repo_root, GEMINI_CONFIG_PATH)?,
        claude_config: read_repo_file(repo_root, CLAUDE_CONFIG_PATH)?,
        hooks: hook_texts,
    };
    evaluate_loaded(repo_root, &inputs)
}

pub fn evaluate_loaded(repo_root: &Path, inputs: &LoadedInputs) -> Result<Evaluation, String> {
    let runtime_hooks = parse_runtime_hooks(&inputs.manifest)?;
    let mut failures = Vec::new();

    failures.extend(manifest_contract_failures(&inputs.manifest));
    failures.extend(config_contract_failures(
        &inputs.manifest,
        &inputs.codex_config,
        &inputs.gemini_config,
        &inputs.claude_config,
        &runtime_hooks,
    ));

    for hook in &runtime_hooks {
        let Some(text) = inputs.hooks.get(&hook.path) else {
            failures.push(format!(
                "runtime_hook_missing_from_loaded_inputs:{}",
                hook.path
            ));
            continue;
        };
        failures.extend(runtime_hook_file_failures(repo_root, hook, text));
    }

    let config_reference_count = config_commands(CODEX_CONFIG_PATH, &inputs.codex_config).len()
        + config_commands(GEMINI_CONFIG_PATH, &inputs.gemini_config).len();
    let retired_surface_count = array_section_after_key(&inputs.manifest, "retired_surfaces")
        .map(|section| objects_in_array(section).len())
        .unwrap_or(0);

    Ok(Evaluation {
        verdict: if failures.is_empty() {
            "PASS".to_owned()
        } else {
            "FAIL".to_owned()
        },
        manifest_path: MANIFEST_PATH.to_owned(),
        runtime_hook_count: runtime_hooks.len(),
        config_reference_count,
        retired_surface_count,
        failures,
    })
}

pub fn manifest_contract_failures(manifest: &str) -> Vec<String> {
    let mut failures = Vec::new();
    let first_class_config_paths = first_class_config_paths(manifest);
    for token in [
        "First-class non-OMC/non-OMX project hook runtime",
        "stdout/stderr advisory guidance only; no project-state mutation",
        "specs/canonical-primitives.json",
        "references to retired wrapper command guidance",
        CHECKER_TARGET,
        "rust_buck2_manifest_config_drift_checker",
    ] {
        if !manifest.contains(token) {
            failures.push(format!("manifest_missing_anchor:{}", token));
        }
    }
    for config in [CODEX_CONFIG_PATH, GEMINI_CONFIG_PATH] {
        if !first_class_config_paths.contains(config) {
            failures.push(format!("manifest_missing_first_class_config:{}", config));
        }
    }
    if manifest.contains("tools/hooks/_canonical-primitives.md\"")
        && !manifest.contains("\"status\": \"deleted\"")
    {
        failures.push("retired_canonical_primitives_not_tombstoned".to_owned());
    }
    failures
}

pub fn first_class_config_paths(manifest: &str) -> BTreeSet<String> {
    let Some(section) = array_section_after_key(manifest, "config_files") else {
        return BTreeSet::new();
    };
    objects_in_array(section)
        .into_iter()
        .filter(|object| compact_json_text(object).contains("\"first_class\":true"))
        .flat_map(|object| json_string_values_after_key(&object, "path"))
        .collect()
}

pub fn config_contract_failures(
    manifest: &str,
    codex_config: &str,
    gemini_config: &str,
    claude_config: &str,
    runtime_hooks: &[RuntimeHook],
) -> Vec<String> {
    let mut failures = Vec::new();
    let allowed_commands = runtime_hooks
        .iter()
        .map(|hook| hook.command.clone())
        .collect::<BTreeSet<_>>();
    let mut referenced = BTreeSet::new();
    let mut commands = Vec::new();
    commands.extend(config_commands(CODEX_CONFIG_PATH, codex_config));
    commands.extend(config_commands(GEMINI_CONFIG_PATH, gemini_config));

    if commands.is_empty() {
        failures.push("active_configs_reference_no_runtime_hooks".to_owned());
    }

    for command in commands {
        referenced.insert(command.command.clone());
        if !allowed_commands.contains(&command.command) {
            failures.push(format!(
                "config_references_unmanifested_hook:{}:{}",
                command.config_path, command.command
            ));
        }
    }

    for allowed in allowed_commands {
        if !referenced.contains(&allowed) {
            failures.push(format!("manifest_runtime_hook_unreferenced:{}", allowed));
        }
    }

    if codex_config.contains("UserPromptSubmit") {
        failures.push("codex_user_prompt_submit_runtime_hook_reintroduced".to_owned());
    }
    if gemini_config.contains("BeforeAgent") {
        failures.push("gemini_before_agent_runtime_hook_reintroduced".to_owned());
    }
    if contains_json_key(claude_config, "hooks") {
        failures.push("claude_project_runtime_hooks_reintroduced".to_owned());
    }

    for token in [
        "tools/hooks/_canonical-primitives.md",
        "oya git",
        "oya vcs",
        "oya gate",
        "oya verify",
        "./bin/oya",
        "bin/oya",
        "Oya CLI",
        "oya CLI",
    ] {
        if active_config_contains_forbidden_token(codex_config, token)
            || active_config_contains_forbidden_token(gemini_config, token)
        {
            failures.push(format!("active_config_contains_retired_guidance:{}", token));
        }
    }

    if !manifest.contains(".claude/settings.json") || !manifest.contains("OMC / oh-my-claudecode") {
        failures.push("manifest_missing_claude_omc_boundary".to_owned());
    }

    failures
}

pub fn runtime_hook_file_failures(repo_root: &Path, hook: &RuntimeHook, text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    if !hook.path.starts_with("tools/hooks/") {
        failures.push(format!("runtime_hook_outside_tools_hooks:{}", hook.path));
    }
    if hook.path.ends_with(".sh") && !text.starts_with("#!/usr/bin/env bash") {
        failures.push(format!(
            "runtime_hook_shell_missing_bash_shebang:{}",
            hook.path
        ));
    }
    if hook.path.ends_with(".sh") && !is_executable(repo_root.join(&hook.path).as_path()) {
        failures.push(format!("runtime_hook_shell_not_executable:{}", hook.path));
    }
    if !compact_json_text(&hook.object).contains("\"network\":false") {
        failures.push(format!(
            "runtime_hook_manifest_network_not_false:{}",
            hook.path
        ));
    }
    if !compact_json_text(&hook.object).contains("\"writes\":[]") {
        failures.push(format!(
            "runtime_hook_manifest_writes_not_empty:{}",
            hook.path
        ));
    }
    for failure in forbidden_runtime_behavior_failures(&hook.path, text) {
        failures.push(failure);
    }
    failures
}

pub fn forbidden_runtime_behavior_failures(path: &str, text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for word in ["curl", "wget", "gh", "ssh", "scp", "nc"] {
        if contains_shell_word(text, word) {
            failures.push(format!(
                "{}:forbidden_network_or_remote_command:{}",
                path, word
            ));
        }
    }
    for word in [
        "python", "python3", "node", "npm", "pnpm", "npx", "bun", "deno", "ruby", "perl", "php",
        "lua",
    ] {
        if contains_shell_word(text, word) || text.contains(&format!("/{}", word)) {
            failures.push(format!("{}:forbidden_runtime_interpreter:{}", path, word));
        }
    }
    for token in [
        "codex exec",
        "claude -p",
        "claude --print",
        "claude code",
        "claude mcp",
        "claude exec",
        "gemini -p",
        "gemini --prompt",
        "gemini exec",
    ] {
        if text.contains(token) {
            failures.push(format!("{}:forbidden_agent_recursion:{}", path, token));
        }
    }
    for token in [
        "git push",
        "rm -rf",
        ".omc",
        ".omx",
        "_canonical-primitives.md",
        "oya git",
        "oya vcs",
        "oya gate",
        "oya verify",
        "./bin/oya",
        "bin/oya",
        "oya --help",
        "Oya CLI",
        "oya CLI",
    ] {
        if text.contains(token) {
            failures.push(format!("{}:forbidden_runtime_token:{}", path, token));
        }
    }
    failures
}

pub fn parse_runtime_hooks(manifest: &str) -> Result<Vec<RuntimeHook>, String> {
    let section = array_section_after_key(manifest, "runtime_hooks")
        .ok_or_else(|| "manifest_missing_runtime_hooks_array".to_owned())?;
    let objects = objects_in_array(section);
    if objects.is_empty() {
        return Err("manifest_runtime_hooks_array_empty".to_owned());
    }
    let mut hooks = Vec::new();
    let mut seen_paths = BTreeSet::new();
    for object in objects {
        let path = json_string_values_after_key(&object, "path")
            .into_iter()
            .next()
            .ok_or_else(|| "runtime_hook_object_missing_path".to_owned())?;
        if !seen_paths.insert(path.clone()) {
            return Err(format!("duplicate_runtime_hook_path:{}", path));
        }
        let command = json_string_values_after_key(&object, "command")
            .into_iter()
            .next()
            .unwrap_or_else(|| path.clone());
        hooks.push(RuntimeHook {
            path,
            command,
            object,
        });
    }
    Ok(hooks)
}

pub fn config_commands(config_path: &str, text: &str) -> Vec<ConfigCommand> {
    json_string_values_after_key(text, "command")
        .into_iter()
        .map(|command| ConfigCommand {
            config_path: config_path.to_owned(),
            command,
        })
        .collect()
}

pub fn array_section_after_key<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let key_token = format!("\"{}\"", key);
    let key_start = text.find(&key_token)?;
    let after_key = &text[key_start + key_token.len()..];
    let bracket_offset = after_key.find('[')?;
    let section_start = key_start + key_token.len() + bracket_offset;
    let section_end = matching_delimiter(text, section_start, '[', ']')?;
    Some(&text[section_start..=section_end])
}

pub fn objects_in_array(array_text: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut depth = 0usize;
    let mut start = None;
    for (index, character) in array_text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(object_start) = start.take() {
                            objects.push(array_text[object_start..=index].to_owned());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    objects
}

pub fn json_string_values_after_key(text: &str, key: &str) -> Vec<String> {
    let mut values = Vec::new();
    let key_token = format!("\"{}\"", key);
    let mut search_from = 0usize;
    while let Some(relative_key_start) = text[search_from..].find(&key_token) {
        let key_start = search_from + relative_key_start;
        let after_key = &text[key_start + key_token.len()..];
        let Some(colon_offset) = after_key.find(':') else {
            break;
        };
        let after_colon = after_key[colon_offset + 1..].trim_start();
        if let Some(after_quote) = after_colon.strip_prefix('"') {
            if let Some((value, consumed)) = parse_json_string_body(after_quote) {
                values.push(value);
                search_from = key_start + key_token.len() + colon_offset + 1 + consumed + 1;
                continue;
            }
        }
        search_from = key_start + key_token.len();
    }
    values
}

fn parse_json_string_body(after_quote: &str) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;
    for (index, character) in after_quote.char_indices() {
        if escaped {
            value.push(match character {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '"' => return Some((value, index)),
            other => value.push(other),
        }
    }
    None
}

fn matching_delimiter(text: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut in_string = false;
    let mut escaped = false;
    let mut depth = 0usize;
    for (relative_index, character) in text[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            character if character == open => depth += 1,
            character if character == close => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(start + relative_index);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn contains_json_key(text: &str, key: &str) -> bool {
    let key_token = format!("\"{}\"", key);
    let mut search_from = 0usize;
    while let Some(relative_key_start) = text[search_from..].find(&key_token) {
        let key_start = search_from + relative_key_start;
        let after_key = &text[key_start + key_token.len()..];
        if after_key.trim_start().starts_with(':') {
            return true;
        }
        search_from = key_start + key_token.len();
    }
    false
}

pub fn contains_shell_word(text: &str, word: &str) -> bool {
    let bytes = text.as_bytes();
    let needle = word.as_bytes();
    if needle.is_empty() || bytes.len() < needle.len() {
        return false;
    }
    for start in 0..=bytes.len() - needle.len() {
        if &bytes[start..start + needle.len()] != needle {
            continue;
        }
        let before_ok = start == 0 || !is_word_byte(bytes[start - 1]);
        let after = start + needle.len();
        let after_ok = after == bytes.len() || !is_word_byte(bytes[after]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'/' | b'.')
}

fn active_config_contains_forbidden_token(text: &str, token: &str) -> bool {
    text.contains(token)
}

fn compact_json_text(input: &str) -> String {
    input
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        metadata.is_file()
    }
}

fn read_repo_file(repo_root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(repo_root.join(relative))
        .map_err(|error| format!("read {}: {}", relative, error))
}

fn repo_root_from_env_or_cwd() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn parse_args() -> Config {
    let mut json = false;
    let mut repo_root = repo_root_from_env_or_cwd();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--repo-root" => {
                if let Some(value) = args.next() {
                    repo_root = PathBuf::from(value);
                } else {
                    eprintln!("--repo-root requires a path");
                    std::process::exit(64);
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            unknown => {
                eprintln!("unknown argument: {}", unknown);
                print_help();
                std::process::exit(64);
            }
        }
    }
    Config { repo_root, json }
}

fn print_help() {
    println!(
        "Validate read-only agent hook runtime manifest/config drift.\n\nUsage: assert-agent-hook-runtime-manifest [--json] [--repo-root PATH]"
    );
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|character| match character {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            other => vec![other],
        })
        .collect()
}

fn evaluation_to_json(evaluation: &Evaluation) -> String {
    let failures = evaluation
        .failures
        .iter()
        .map(|failure| format!("\"{}\"", json_escape(failure)))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        concat!(
            "{{\n",
            "  \"verdict\": \"{}\",\n",
            "  \"manifest_path\": \"{}\",\n",
            "  \"checker_target\": \"{}\",\n",
            "  \"runtime_hook_count\": {},\n",
            "  \"config_reference_count\": {},\n",
            "  \"retired_surface_count\": {},\n",
            "  \"failures\": [{}]\n",
            "}}\n"
        ),
        json_escape(&evaluation.verdict),
        json_escape(&evaluation.manifest_path),
        json_escape(CHECKER_TARGET),
        evaluation.runtime_hook_count,
        evaluation.config_reference_count,
        evaluation.retired_surface_count,
        failures
    )
}

fn main() {
    let config = parse_args();
    match evaluate(&config.repo_root) {
        Ok(evaluation) => {
            if config.json {
                print!("{}", evaluation_to_json(&evaluation));
            } else {
                println!(
                    "agent hook runtime manifest {}: runtime_hooks={} config_refs={} retired_surfaces={} failures={}",
                    evaluation.verdict,
                    evaluation.runtime_hook_count,
                    evaluation.config_reference_count,
                    evaluation.retired_surface_count,
                    evaluation.failures.len()
                );
                for failure in &evaluation.failures {
                    println!("- {}", failure);
                }
            }
            if evaluation.verdict != "PASS" {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}
