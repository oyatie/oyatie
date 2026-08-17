//! Governance check for audit event emission.
//!
//! Enforces ADR-0263 by requiring every state-changing endpoint with a stable
//! operation identifier to have registered audit event evidence.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

/// Stable identifier for the governance rule enforced by this crate.
pub const RULE_ID: &str = "ADR-0263";

/// Human-readable summary of the rule this crate enforces.
pub const ENFORCED_RULE: &str =
    "Every state-changing endpoint emits an ADR-0263 registered audit event class.";

/// Status returned by the enforcement entrypoint.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EnforcementStatus {
    Passed,
    Failed,
}

/// Finding categories emitted by the audit event emission check.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum FindingKind {
    MissingAuditEvent,
    MissingEndpointIdentifier,
}

/// Machine-readable failure detail for endpoints without registered audit evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuditFinding {
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
    pub mutating_endpoint_identifiers: Vec<String>,
    pub audit_evidence_files: Vec<PathBuf>,
    pub findings: Vec<AuditFinding>,
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

/// Enforces audit event emission for state-changing endpoints.
pub fn enforce_audit_event_emission(
    repo_root: impl AsRef<Path>,
) -> anyhow::Result<GovernanceCheckOutcome> {
    let repo_root = repo_root.as_ref().to_path_buf();
    let endpoint_files = candidate_endpoint_files(&repo_root)?;
    let mut endpoints = Vec::new();
    let mut findings = Vec::new();

    for path in &endpoint_files {
        let raw = fs::read_to_string(path)?;
        let parsed = extract_mutating_endpoint_identifiers(path, &raw);
        endpoints.extend(parsed.endpoints);
        findings.extend(parsed.missing_identifier_findings);
    }

    let audit_evidence_files = audit_evidence_files(&repo_root, &endpoint_files)?;
    let registered_audit_emitters = registered_audit_emitters(&audit_evidence_files)?;
    let mut mutating_endpoint_identifiers = BTreeSet::new();

    for endpoint in endpoints {
        mutating_endpoint_identifiers.insert(endpoint.identifier.clone());
        if !identifier_has_audit_evidence(&registered_audit_emitters, &endpoint.identifier) {
            findings.push(AuditFinding {
                kind: FindingKind::MissingAuditEvent,
                source_file: endpoint.source_file,
                identifier: Some(endpoint.identifier.clone()),
                hint: format!(
                    "mutating endpoint identifier `{}` has no registered audit event evidence",
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
        mutating_endpoint_identifiers: mutating_endpoint_identifiers.into_iter().collect(),
        audit_evidence_files,
        findings,
    })
}

struct ParsedEndpoints {
    endpoints: Vec<Endpoint>,
    missing_identifier_findings: Vec<AuditFinding>,
}

fn extract_mutating_endpoint_identifiers(path: &Path, raw: &str) -> ParsedEndpoints {
    if path.extension().and_then(|s| s.to_str()) == Some("proto") {
        return extract_proto_mutations(path, raw);
    }
    extract_openapi_like_mutations(path, raw)
}

fn extract_proto_mutations(path: &Path, raw: &str) -> ParsedEndpoints {
    let mut endpoints = Vec::new();
    let mut saw_service = false;
    let mut saw_mutating_shape = false;

    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("service ") {
            saw_service = true;
        }
        let Some(rest) = trimmed.strip_prefix("rpc ") else {
            continue;
        };
        let identifier = rest
            .split(|c: char| c == '(' || c.is_whitespace())
            .next()
            .unwrap_or("")
            .trim();
        if !is_mutating_identifier(identifier) {
            continue;
        }
        saw_mutating_shape = true;
        endpoints.push(Endpoint {
            source_file: path.to_path_buf(),
            identifier: identifier.to_string(),
        });
    }

    let mut missing_identifier_findings = Vec::new();
    if saw_service && raw.contains("google.api.http") && !saw_mutating_shape {
        missing_identifier_findings.push(AuditFinding {
            kind: FindingKind::MissingEndpointIdentifier,
            source_file: path.to_path_buf(),
            identifier: None,
            hint:
                "proto service contains HTTP annotations but no mutating rpc identifier was found"
                    .into(),
        });
    }

    ParsedEndpoints {
        endpoints,
        missing_identifier_findings,
    }
}

fn extract_openapi_like_mutations(path: &Path, raw: &str) -> ParsedEndpoints {
    match extract_openapi_value_mutations(path, raw) {
        Some(parsed) => parsed,
        None if is_structured_openapi_file(path) => ParsedEndpoints {
            endpoints: Vec::new(),
            missing_identifier_findings: vec![missing_identifier(
                path,
                "parseable OpenAPI document with `paths`",
            )],
        },
        None => extract_openapi_line_mutations(path, raw),
    }
}

fn extract_openapi_value_mutations(path: &Path, raw: &str) -> Option<ParsedEndpoints> {
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
            let Some(method) = method.as_str().filter(|value| is_mutating_method(value)) else {
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

fn extract_openapi_line_mutations(path: &Path, raw: &str) -> ParsedEndpoints {
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
            if is_mutating_method(method)
                && let Some(api_path) = &current_path
            {
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

fn missing_identifier(path: &Path, operation: &str) -> AuditFinding {
    AuditFinding {
        kind: FindingKind::MissingEndpointIdentifier,
        source_file: path.to_path_buf(),
        identifier: None,
        hint: format!(
            "mutating endpoint `{operation}` is missing a stable operationId/rpc identifier"
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

fn is_mutating_method(method: &str) -> bool {
    matches!(method, "post" | "put" | "patch" | "delete")
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

fn is_mutating_identifier(identifier: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "Create", "Update", "Delete", "Patch", "Put", "Set", "Grant", "Revoke", "Enable",
        "Disable", "Start", "Stop", "Cancel", "Approve", "Reject",
    ];
    PREFIXES.iter().any(|prefix| identifier.starts_with(prefix))
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

fn audit_evidence_files(
    repo_root: &Path,
    endpoint_files: &[PathBuf],
) -> anyhow::Result<Vec<PathBuf>> {
    let endpoint_files: BTreeSet<PathBuf> = endpoint_files.iter().cloned().collect();
    let mut out = Vec::new();
    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(keeps_entry)
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        if endpoint_files.contains(&path) {
            continue;
        }
        if is_audit_evidence_candidate(&path) {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
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

fn is_audit_evidence_candidate(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    let file = path
        .file_name()
        .map(|value| value.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    lower.contains("/registry/audit-events/")
        || lower.contains("/evidence/audit/")
        || file == "audit-chain.jsonl"
}

fn registered_audit_emitters(evidence_files: &[PathBuf]) -> anyhow::Result<BTreeSet<String>> {
    let mut emitters = BTreeSet::new();
    for path in evidence_files {
        let raw = fs::read_to_string(path)?;
        if path.file_name().and_then(|name| name.to_str()) == Some("audit-chain.jsonl") {
            for line in raw.lines().map(str::trim).filter(|line| !line.is_empty()) {
                if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(line) {
                    collect_audit_emitters(&value, &mut emitters);
                }
            }
        } else if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&raw) {
            collect_audit_emitters(&value, &mut emitters);
        }
    }
    Ok(emitters)
}

fn collect_audit_emitters(value: &serde_yaml::Value, emitters: &mut BTreeSet<String>) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, value) in map {
                if let Some(key) = key.as_str()
                    && is_audit_emitter_key(key)
                    && let Some(emitter) = value.as_str().map(str::trim)
                    && !emitter.is_empty()
                {
                    emitters.insert(emitter.to_string());
                }
                collect_audit_emitters(value, emitters);
            }
        }
        serde_yaml::Value::Sequence(items) => {
            for item in items {
                collect_audit_emitters(item, emitters);
            }
        }
        _ => {}
    }
}

fn is_audit_emitter_key(key: &str) -> bool {
    matches!(
        key,
        "emitted_by"
            | "operationId"
            | "operation_id"
            | "endpoint_identifier"
            | "mutating_endpoint"
            | "registered_for"
    )
}

fn identifier_has_audit_evidence(emitters: &BTreeSet<String>, identifier: &str) -> bool {
    emitters.contains(identifier)
}
