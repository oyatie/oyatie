//! Governance check for public API Cedar policy coverage.
//!
//! Enforces ADR-0243 by requiring every public API endpoint with a stable
//! operation identifier to have corresponding Cedar policy evidence.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

/// Stable identifier for the governance rule enforced by this crate.
pub const RULE_ID: &str = "ADR-0243";

/// Human-readable summary of the rule this crate enforces.
pub const ENFORCED_RULE: &str =
    "Every public API endpoint has a corresponding Cedar policy in policies/*.cedar.";

/// Status returned by the enforcement entrypoint.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EnforcementStatus {
    Passed,
    Failed,
}

/// Finding categories emitted by the Cedar coverage check.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum FindingKind {
    MissingCedarPolicy,
    MissingEndpointIdentifier,
}

/// Machine-readable failure detail for uncovered or unidentifiable endpoints.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CoverageFinding {
    pub kind: FindingKind,
    pub source_file: PathBuf,
    pub identifier: Option<String>,
    pub hint: String,
}

/// Machine-readable outcome from the enforcement entrypoint.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GovernanceCheckOutcome {
    pub rule_id: &'static str,
    pub enforced_rule: &'static str,
    pub repo_root: PathBuf,
    pub status: EnforcementStatus,
    pub endpoint_identifiers: Vec<String>,
    pub cedar_policy_files: Vec<PathBuf>,
    pub findings: Vec<CoverageFinding>,
}

impl GovernanceCheckOutcome {
    pub fn is_success(&self) -> bool {
        self.status == EnforcementStatus::Passed
    }

    pub fn is_scaffolded(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
struct Endpoint {
    source_file: PathBuf,
    identifier: String,
}

/// Enforces Cedar policy coverage for public API endpoints.
pub fn enforce_cedar_coverage(
    repo_root: impl AsRef<Path>,
) -> anyhow::Result<GovernanceCheckOutcome> {
    let repo_root = repo_root.as_ref().to_path_buf();
    let mut endpoints = Vec::new();
    let mut findings = Vec::new();

    for path in candidate_endpoint_files(&repo_root)? {
        let raw = fs::read_to_string(&path)?;
        let parsed = extract_endpoint_identifiers(&path, &raw);
        endpoints.extend(parsed.endpoints);
        findings.extend(parsed.missing_identifier_findings);
    }

    let cedar_policy_files = cedar_policy_files(&repo_root)?;
    let cedar_actions = cedar_policy_actions(&cedar_policy_files)?;
    let mut endpoint_identifiers = BTreeSet::new();

    for endpoint in endpoints {
        endpoint_identifiers.insert(endpoint.identifier.clone());
        if !identifier_has_policy_evidence(&cedar_actions, &endpoint.identifier) {
            findings.push(CoverageFinding {
                kind: FindingKind::MissingCedarPolicy,
                source_file: endpoint.source_file,
                identifier: Some(endpoint.identifier.clone()),
                hint: format!(
                    "endpoint identifier `{}` has no matching Cedar policy evidence",
                    endpoint.identifier
                ),
            });
        }
    }

    findings.sort_by(|a, b| {
        a.source_file
            .cmp(&b.source_file)
            .then_with(|| a.identifier.cmp(&b.identifier))
            .then_with(|| a.hint.cmp(&b.hint))
    });

    let status = if findings.is_empty() {
        EnforcementStatus::Passed
    } else {
        EnforcementStatus::Failed
    };

    Ok(GovernanceCheckOutcome {
        rule_id: RULE_ID,
        enforced_rule: ENFORCED_RULE,
        repo_root,
        status,
        endpoint_identifiers: endpoint_identifiers.into_iter().collect(),
        cedar_policy_files,
        findings,
    })
}

struct ParsedEndpoints {
    endpoints: Vec<Endpoint>,
    missing_identifier_findings: Vec<CoverageFinding>,
}

fn extract_endpoint_identifiers(path: &Path, raw: &str) -> ParsedEndpoints {
    if path.extension().and_then(|s| s.to_str()) == Some("proto") {
        return extract_proto_endpoints(path, raw);
    }
    extract_openapi_like_endpoints(path, raw)
}

fn extract_proto_endpoints(path: &Path, raw: &str) -> ParsedEndpoints {
    let mut endpoints = Vec::new();
    let mut saw_service = false;
    let mut saw_endpoint_shape = false;

    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("service ") {
            saw_service = true;
        }
        if let Some(rest) = trimmed.strip_prefix("rpc ") {
            saw_endpoint_shape = true;
            let identifier = rest
                .split(|c: char| c == '(' || c.is_whitespace())
                .next()
                .unwrap_or("")
                .trim();
            if identifier.is_empty() {
                continue;
            }
            endpoints.push(Endpoint {
                source_file: path.to_path_buf(),
                identifier: identifier.to_string(),
            });
        }
    }

    let mut missing_identifier_findings = Vec::new();
    if saw_service && !saw_endpoint_shape {
        missing_identifier_findings.push(CoverageFinding {
            kind: FindingKind::MissingEndpointIdentifier,
            source_file: path.to_path_buf(),
            identifier: None,
            hint: "public proto service declares no rpc endpoint identifiers".into(),
        });
    }

    ParsedEndpoints {
        endpoints,
        missing_identifier_findings,
    }
}

fn extract_openapi_like_endpoints(path: &Path, raw: &str) -> ParsedEndpoints {
    match extract_openapi_value_endpoints(path, raw) {
        Some(parsed) => parsed,
        None if is_structured_openapi_file(path) => ParsedEndpoints {
            endpoints: Vec::new(),
            missing_identifier_findings: vec![missing_identifier(
                path,
                "parseable OpenAPI document with `paths`",
            )],
        },
        None => extract_openapi_line_endpoints(path, raw),
    }
}

fn extract_openapi_value_endpoints(path: &Path, raw: &str) -> Option<ParsedEndpoints> {
    let value = serde_yaml::from_str::<serde_yaml::Value>(raw).ok()?;
    let paths = mapping_field(&value, "paths")?;
    let serde_yaml::Value::Mapping(paths) = paths else {
        return Some(ParsedEndpoints {
            endpoints: Vec::new(),
            missing_identifier_findings: Vec::new(),
        });
    };

    let mut endpoints = Vec::new();
    let mut missing_identifier_findings = Vec::new();

    for (api_path, methods) in paths {
        let Some(api_path) = api_path.as_str().filter(|value| value.starts_with('/')) else {
            continue;
        };
        let serde_yaml::Value::Mapping(methods) = methods else {
            continue;
        };
        for (method, operation) in methods {
            let Some(method) = method.as_str().filter(|value| is_http_method(value)) else {
                continue;
            };
            if let Some(identifier) = mapping_string_field(operation, "operationId") {
                endpoints.push(Endpoint {
                    source_file: path.to_path_buf(),
                    identifier,
                });
            } else {
                missing_identifier_findings.push(missing_identifier(
                    path,
                    &format!("{} {}", method.to_uppercase(), api_path),
                ));
            }
        }
    }

    Some(ParsedEndpoints {
        endpoints,
        missing_identifier_findings,
    })
}

fn mapping_field<'a>(value: &'a serde_yaml::Value, field: &str) -> Option<&'a serde_yaml::Value> {
    let serde_yaml::Value::Mapping(map) = value else {
        return None;
    };
    map.get(serde_yaml::Value::String(field.to_string()))
}

fn mapping_string_field(value: &serde_yaml::Value, field: &str) -> Option<String> {
    mapping_field(value, field)
        .and_then(serde_yaml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn is_http_method(value: &str) -> bool {
    matches!(
        value,
        "get" | "put" | "post" | "delete" | "patch" | "head" | "options" | "trace"
    )
}

fn is_structured_openapi_file(path: &Path) -> bool {
    let file = path
        .file_name()
        .map(|value| value.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    file.ends_with(".openapi.yaml")
        || file.ends_with(".openapi.yml")
        || file.ends_with(".openapi.json")
}

fn extract_openapi_line_endpoints(path: &Path, raw: &str) -> ParsedEndpoints {
    let mut endpoints = Vec::new();
    let mut missing_identifier_findings = Vec::new();
    let mut current_path: Option<String> = None;
    let mut pending_operation: Option<String> = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(api_path) = yaml_path_key(trimmed) {
            if let Some(pending) = pending_operation.take() {
                missing_identifier_findings.push(missing_identifier(path, &pending));
            }
            current_path = Some(api_path.to_string());
            continue;
        }

        if let Some(method) = http_method_key(trimmed) {
            if let Some(pending) = pending_operation.take() {
                missing_identifier_findings.push(missing_identifier(path, &pending));
            }
            if let Some(api_path) = &current_path {
                pending_operation = Some(format!("{} {}", method.to_uppercase(), api_path));
            }
            continue;
        }

        if let Some(identifier) = scalar_value(trimmed, "operationId")
            && let Some(_pending) = pending_operation.take()
        {
            endpoints.push(Endpoint {
                source_file: path.to_path_buf(),
                identifier,
            });
        }
    }

    if let Some(pending) = pending_operation.take() {
        missing_identifier_findings.push(missing_identifier(path, &pending));
    }

    ParsedEndpoints {
        endpoints,
        missing_identifier_findings,
    }
}

fn missing_identifier(path: &Path, operation: &str) -> CoverageFinding {
    CoverageFinding {
        kind: FindingKind::MissingEndpointIdentifier,
        source_file: path.to_path_buf(),
        identifier: None,
        hint: format!(
            "public endpoint `{operation}` is missing a stable operationId/rpc identifier"
        ),
    }
}

fn yaml_path_key(trimmed: &str) -> Option<&str> {
    let without_quote = trimmed.trim_matches('"').trim_matches('\'');
    if !without_quote.starts_with('/') {
        return None;
    }
    without_quote
        .split_once(':')
        .map(|(path, _)| path.trim().trim_matches('"').trim_matches('\''))
        .filter(|path| path.starts_with('/'))
}

fn http_method_key(trimmed: &str) -> Option<&'static str> {
    const METHODS: &[&str] = &[
        "get", "put", "post", "delete", "patch", "head", "options", "trace",
    ];
    METHODS.iter().copied().find(|method| {
        trimmed
            .strip_prefix(*method)
            .is_some_and(|rest| rest.trim_start().starts_with(':'))
    })
}

fn scalar_value(trimmed: &str, key: &str) -> Option<String> {
    let rest = trimmed.strip_prefix(key)?.trim_start();
    let rest = rest.strip_prefix(':')?.trim();
    let value = rest.trim_matches('"').trim_matches('\'').trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn candidate_endpoint_files(repo_root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(keeps_entry)
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if is_public_api_file(path) {
            out.push(path.to_path_buf());
        }
    }
    out.sort();
    Ok(out)
}

fn cedar_policy_files(repo_root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(keeps_entry)
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("cedar") && is_cedar_policy_file(path)
        {
            out.push(path.to_path_buf());
        }
    }
    out.sort();
    Ok(out)
}

fn is_cedar_policy_file(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case("policies")
    })
}

fn keeps_entry(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    !matches!(
        name.as_ref(),
        ".git" | "target" | "third-party" | "node_modules"
    )
}

fn is_public_api_file(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    let file = path
        .file_name()
        .map(|s| s.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    lower.contains("/contracts/openapi/")
        || lower.contains("/contracts/proto/")
        || lower.contains("/registry/openapi/")
        || file.ends_with(".openapi.yaml")
        || file.ends_with(".openapi.yml")
        || file.ends_with(".openapi.json")
        || file.ends_with(".proto")
}

fn cedar_policy_actions(policy_files: &[PathBuf]) -> anyhow::Result<BTreeSet<String>> {
    let mut actions = BTreeSet::new();
    for path in policy_files {
        actions.extend(extract_cedar_action_literals(&fs::read_to_string(path)?));
    }
    Ok(actions)
}

fn extract_cedar_action_literals(raw: &str) -> BTreeSet<String> {
    let mut actions = BTreeSet::new();
    let code = strip_cedar_comments(raw);
    let mut rest = code.as_str();
    while let Some(start) = rest.find("Action::\"") {
        rest = &rest[start + "Action::\"".len()..];
        let Some(end) = rest.find('"') else {
            break;
        };
        let action = &rest[..end];
        if !action.is_empty() {
            actions.insert(action.to_string());
        }
        rest = &rest[end + 1..];
    }
    actions
}

fn strip_cedar_comments(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars().peekable();
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            for next in chars.by_ref() {
                if next == '\n' {
                    out.push('\n');
                    break;
                }
            }
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            in_block_comment = true;
            continue;
        }

        out.push(ch);
    }

    out
}

fn identifier_has_policy_evidence(actions: &BTreeSet<String>, identifier: &str) -> bool {
    actions.contains(identifier)
}
