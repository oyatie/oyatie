#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

//! Advisory OpenAPI/AsyncAPI version-pin checker.
//!
//! Input: `TOOL_INPUT` or stdin using Codex/Gemini hook JSON shapes for
//! YAML/JSON contract-like files. Deployment is Buck2/CI/Prow first; this is
//! not installed as a local runtime hook until the Buck2 host Rust target is
//! portable on agent workstations. The checker is advisory and exits 0 even
//! when it emits a suggestion.

use std::{
    env, fs,
    io::{self, IsTerminal, Read},
    path::Path,
};

const CANONICAL_OPENAPI: &str = "3.2.0";
const CANONICAL_ASYNCAPI: &str = "3.1.0";

fn main() {
    if let Some(file_path) = hook_file_path() {
        emit_suggestions(&file_path);
    }
}

fn hook_file_path() -> Option<String> {
    if let Ok(tool_input) = env::var("TOOL_INPUT") {
        if let Some(path) = extract_tool_path(&tool_input) {
            return Some(path);
        }
    }

    if io::stdin().is_terminal() {
        return None;
    }

    let mut stdin = String::new();
    if io::stdin().read_to_string(&mut stdin).is_ok() {
        let trimmed = stdin.trim();
        if trimmed.is_empty() {
            None
        } else {
            extract_tool_path(trimmed).or_else(|| Some(trimmed.to_owned()))
        }
    } else {
        None
    }
}

fn emit_suggestions(file_path: &str) {
    if !is_supported_contract_file(file_path) {
        return;
    }

    let path = Path::new(file_path);
    if !path.is_file() {
        return;
    }

    let Ok(content) = fs::read_to_string(path) else {
        return;
    };

    if let Some(version) = version_for_key(&content, "openapi") {
        if version != CANONICAL_OPENAPI {
            eprintln!("ℹ [spec-version-pin-suggester] Detected openapi: {version} in {file_path}");
            eprintln!(
                "ℹ  As of 2026-06-05, canonical version is {CANONICAL_OPENAPI} (spec.openapis.org/oas/v3.2.0)."
            );
            eprintln!("ℹ  Consider updating unless you have a verified source for {version}.");
        }
    }

    if let Some(version) = version_for_key(&content, "asyncapi") {
        if version != CANONICAL_ASYNCAPI {
            eprintln!("ℹ [spec-version-pin-suggester] Detected asyncapi: {version} in {file_path}");
            eprintln!(
                "ℹ  As of 2026-06-05, canonical version is {CANONICAL_ASYNCAPI} (asyncapi.com/docs/reference/specification/v3.1.0)."
            );
            eprintln!("ℹ  Consider updating unless you have a verified source for {version}.");
        }
    }
}

fn is_supported_contract_file(file_path: &str) -> bool {
    let lower = file_path.to_ascii_lowercase();
    lower.ends_with(".yaml") || lower.ends_with(".yml") || lower.ends_with(".json")
}

fn extract_tool_path(input: &str) -> Option<String> {
    extract_json_string(input, "file_path")
        .or_else(|| extract_json_string(input, "path"))
        .filter(|path| !path.is_empty())
}

fn extract_json_string(input: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = input.find(&needle)? + needle.len();
    let after_key = &input[start..];
    let colon = after_key.find(':')?;
    let mut chars = after_key[colon + 1..].trim_start().chars();
    if chars.next()? != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            value.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(value);
        } else {
            value.push(ch);
        }
    }
    None
}

fn version_for_key(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = yaml_value_after_key(trimmed, key) {
            if let Some(version) = first_semver(rest) {
                return Some(version);
            }
        }
        if let Some(rest) = json_value_after_key(trimmed, key) {
            if let Some(version) = first_semver(rest) {
                return Some(version);
            }
        }
    }
    None
}

fn yaml_value_after_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.strip_prefix(key)
        .and_then(|rest| rest.strip_prefix(':'))
        .map(str::trim_start)
}

fn json_value_after_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let line = line.strip_prefix('"')?;
    let rest = line.strip_prefix(key)?;
    let rest = rest.strip_prefix('"')?.trim_start();
    rest.strip_prefix(':').map(str::trim_start)
}

fn first_semver(input: &str) -> Option<String> {
    let mut start = None;
    for (index, ch) in input.char_indices() {
        if ch.is_ascii_digit() {
            start = Some(index);
            break;
        }
    }
    let start = start?;
    let version: String = input[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect();
    if version.matches('.').count() == 2 {
        Some(version)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_nested_codex_tool_path() {
        let input = r#"{"tool_input":{"file_path":"contracts/example.yaml"}}"#;
        assert_eq!(
            extract_tool_path(input).as_deref(),
            Some("contracts/example.yaml")
        );
    }

    #[test]
    fn extracts_flat_harness_path() {
        let input = r#"{"path":"contracts/example.yaml"}"#;
        assert_eq!(
            extract_tool_path(input).as_deref(),
            Some("contracts/example.yaml")
        );
    }

    #[test]
    fn detects_yaml_and_json_versions() {
        assert_eq!(
            version_for_key("openapi: 3.1.0\n", "openapi").as_deref(),
            Some("3.1.0")
        );
        assert_eq!(
            version_for_key("  \"asyncapi\": \"3.0.0\",", "asyncapi").as_deref(),
            Some("3.0.0")
        );
    }

    #[test]
    fn ignores_non_matching_keys() {
        assert_eq!(version_for_key("openapi_version: 3.1.0", "openapi"), None);
    }
}
