//! Reading the workspace's committed facts, and the parsers that turn them
//! into the map's inputs.

use crate::MapBuildError;
use std::fs;
use std::path::Path;

pub(crate) fn read(path: &Path) -> Result<String, MapBuildError> {
    fs::read_to_string(path).map_err(|error| MapBuildError::Io {
        path: path.to_path_buf(),
        source: error.to_string(),
    })
}

pub(crate) fn relativize(root: &Path, full: &Path) -> String {
    full.strip_prefix(root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| full.display().to_string())
}

/// Parse the `members = [...]` array out of root Cargo.toml.
/// Std-only TOML scan: find `[workspace]` table, then the `members` key, then
/// collect each quoted string until the closing `]`.
pub(crate) fn parse_cargo_members(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_workspace = false;
    let mut in_members = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace]" {
            in_workspace = true;
            continue;
        }
        if in_workspace && trimmed.starts_with('[') && trimmed != "[workspace]" {
            in_workspace = false;
            in_members = false;
            continue;
        }
        if !in_workspace {
            continue;
        }
        if trimmed.starts_with("members") && trimmed.contains('[') {
            in_members = true;
            continue;
        }
        if !in_members {
            continue;
        }
        if trimmed.starts_with(']') {
            in_members = false;
            continue;
        }
        // Match `"path",` or `'path',`.
        let inner = trimmed.trim_end_matches(',').trim_end_matches('"');
        if let Some(stripped) = inner.strip_prefix('"') {
            out.push(stripped.to_string());
        } else if let Some(stripped) = inner.strip_prefix('\'') {
            out.push(stripped.trim_end_matches('\'').to_string());
        }
    }
    out
}

/// Parse a JSON document and collect every value of `field_name` that appears
/// as a string field anywhere in the document. Used for "microservice_id"
/// extraction from microservices.json.
pub(crate) fn parse_json_string_array_values(text: &str, field_name: &str) -> Vec<String> {
    let needle = format!("\"{field_name}\"");
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(idx) = rest.find(&needle) {
        rest = &rest[idx + needle.len()..];
        let Some(colon) = rest.find(':') else { break };
        let after = rest[colon + 1..].trim_start();
        if let Some(value) = read_json_string(after) {
            out.push(value);
        }
    }
    out
}

/// Parse `bc_id` + `microservice_id` pairs from bounded-contexts.json.
pub(crate) fn parse_bc_pairs(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current_bc: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(v) = json_field(trimmed, "bc_id") {
            current_bc = Some(v);
            continue;
        }
        if let Some(v) = json_field(trimmed, "microservice_id")
            && let Some(bc) = current_bc.take()
        {
            out.push((bc, v));
        }
    }
    out
}

/// Parse `fragment_id` + `consumed_by_openapi[]` pairs from cedar-fragments.json.
pub(crate) fn parse_fragment_consumed_pairs(text: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_consumed: Vec<String> = Vec::new();
    let mut in_consumed_array = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(v) = json_field(trimmed, "fragment_id") {
            if let Some(prev) = current_id.take() {
                out.push((prev, std::mem::take(&mut current_consumed)));
            }
            current_id = Some(v);
            continue;
        }
        if trimmed.starts_with("\"consumed_by_openapi\"") && trimmed.ends_with('[') {
            in_consumed_array = true;
            continue;
        }
        if in_consumed_array {
            if trimmed.starts_with(']') {
                in_consumed_array = false;
                continue;
            }
            let inner = trimmed.trim_end_matches(',');
            if let Some(stripped) = inner.strip_prefix('"')
                && let Some(value) = stripped.strip_suffix('"')
            {
                current_consumed.push(value.to_string());
            }
        }
    }
    if let Some(id) = current_id {
        out.push((id, current_consumed));
    }
    out
}

/// Return the string value of `"key": "value"` on this trimmed line, if it
/// matches.
pub(crate) fn json_field(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let after_key = line.strip_prefix(&needle)?.trim_start().strip_prefix(':')?;
    read_json_string(after_key.trim_start())
}

/// Read a JSON string starting at the leading `"`, up to the closing `"`.
pub(crate) fn read_json_string(input: &str) -> Option<String> {
    let after_quote = input.strip_prefix('"')?;
    let close = after_quote.find('"')?;
    Some(after_quote[..close].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_field_extracts_string_value() {
        assert_eq!(
            json_field(r#""foo": "bar","#, "foo"),
            Some("bar".to_string())
        );
        assert_eq!(
            json_field(r#""foo": "bar""#, "foo"),
            Some("bar".to_string())
        );
        assert!(json_field(r#""other": "bar""#, "foo").is_none());
    }

    #[test]
    fn parse_cargo_members_basic() {
        let toml = r#"[workspace]
resolver = "2"
members = [
  "crates/a",
  "crates/b",
  "crates/c-d-e"
]

[workspace.dependencies]
serde = "1"
"#;
        let members = parse_cargo_members(toml);
        assert_eq!(members, vec!["crates/a", "crates/b", "crates/c-d-e"]);
    }

    #[test]
    fn parse_cargo_members_empty() {
        assert!(parse_cargo_members("").is_empty());
        assert!(parse_cargo_members("[package]\nname = \"foo\"\n").is_empty());
    }

    #[test]
    fn parse_json_string_array_values_extracts_field() {
        let json = r#"{"a": [{"microservice_id": "ops"}, {"microservice_id": "foundry"}]}"#;
        let out = parse_json_string_array_values(json, "microservice_id");
        assert_eq!(out, vec!["ops", "foundry"]);
    }

    #[test]
    fn parse_bc_pairs_extracts_pairs() {
        let json = r#"{
  "bounded_contexts": [
    {
      "bc_id": "ops/docs-portal",
      "microservice_id": "ops"
    },
    {
      "bc_id": "ops/workspace",
      "microservice_id": "ops"
    }
  ]
}"#;
        let pairs = parse_bc_pairs(json);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("ops/docs-portal".to_string(), "ops".to_string()));
        assert_eq!(pairs[1], ("ops/workspace".to_string(), "ops".to_string()));
    }

    #[test]
    fn parse_fragment_consumed_pairs_extracts_pairs() {
        let json = r#"{
  "fragments": [
    {
      "fragment_id": "ops-internal-public",
      "consumed_by_openapi": [
        "contracts/ops-workspace-shell-v1.openapi.yaml",
        "contracts/ops-docs-v1.openapi.yaml"
      ]
    },
    {
      "fragment_id": "ops-tenant-private",
      "consumed_by_openapi": [
        "contracts/ops-workspace-shell-v1.openapi.yaml"
      ]
    }
  ]
}"#;
        let pairs = parse_fragment_consumed_pairs(json);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0, "ops-internal-public");
        assert_eq!(pairs[0].1.len(), 2);
        assert_eq!(pairs[1].0, "ops-tenant-private");
        assert_eq!(pairs[1].1.len(), 1);
    }
}
