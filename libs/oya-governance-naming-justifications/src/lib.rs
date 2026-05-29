//! Governance check for microservice manifest naming justifications.
//!
//! The lane scans `microservices/*/manifest.*` and requires each manifest to
//! carry a top-level `naming_justifications` string. The string is treated as a
//! one-line proof and must cite both BNF v4 and the 12-layer enum so reviewers
//! can see why the service, bounded-context, and layer names are admissible.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

/// Stable identifier for the governance rule enforced by this crate.
pub const RULE_ID: &str = "feedback_naming_justification";

/// Human-readable summary of the rule this crate enforces.
pub const ENFORCED_RULE: &str =
    "Every microservice manifest has naming_justifications with BNF v4 and 12-layer-enum proof.";

/// Canonical one-line example used in diagnostics and README snippets.
pub const SUGGESTED_PROOF: &str = "naming_justifications: \"BNF v4.1 service_action_resource=<service>.<bounded_context>.<action>.<resource>; 12-layer-enum=<api|rest|application|usecase|domain|kernel|adapter|worker|sdk|iac|policy|observability>\"";

/// Machine-readable pass/fail state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementStatus {
    Passed,
    Failed,
}

/// Specific failure category for a manifest naming-proof violation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NamingViolationKind {
    ParseError,
    MissingField,
    NonStringField,
    EmptyProof,
    MultiLineProof,
    UnstructuredProof,
    MissingBnfV4Citation,
    MissingTwelveLayerEnumCitation,
}

/// One line-numbered violation from a single manifest file.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NamingViolation {
    pub path: PathBuf,
    pub line: usize,
    pub kind: NamingViolationKind,
    pub message: String,
    pub suggested_fix: String,
}

/// Machine-readable outcome from the enforcement entrypoint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GovernanceCheckOutcome {
    pub rule_id: String,
    pub enforced_rule: String,
    pub repo_root: PathBuf,
    pub status: EnforcementStatus,
    pub scanned_manifests: usize,
    pub violations: Vec<NamingViolation>,
}

impl GovernanceCheckOutcome {
    pub fn is_success(&self) -> bool {
        self.status == EnforcementStatus::Passed
    }

    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FieldValue {
    Missing,
    String(String),
    NonString,
}

/// Enforces naming justification coverage for every microservice manifest.
///
/// The check is intentionally conservative: unsupported manifest extensions are
/// ignored, malformed supported manifests are reported as violations, and every
/// violation carries the best line number available from the raw source text.
pub fn enforce_naming_justifications(
    repo_root: impl AsRef<Path>,
) -> Result<GovernanceCheckOutcome> {
    let repo_root = repo_root.as_ref().to_path_buf();
    let manifests = discover_manifest_files(&repo_root)?;
    let mut violations = Vec::new();

    for manifest in &manifests {
        let content = fs::read_to_string(manifest)
            .with_context(|| format!("failed to read {}", manifest.display()))?;
        inspect_manifest(&repo_root, manifest, &content, &mut violations);
    }

    let status = if violations.is_empty() {
        EnforcementStatus::Passed
    } else {
        EnforcementStatus::Failed
    };

    Ok(GovernanceCheckOutcome {
        rule_id: RULE_ID.to_string(),
        enforced_rule: ENFORCED_RULE.to_string(),
        repo_root,
        status,
        scanned_manifests: manifests.len(),
        violations,
    })
}

/// Formats a compact text report for CLI output.
pub fn format_text_report(outcome: &GovernanceCheckOutcome) -> String {
    let mut report = String::new();
    report.push_str(&format!(
        "{}: {:?} ({} manifests, {} violations)\n",
        outcome.rule_id,
        outcome.status,
        outcome.scanned_manifests,
        outcome.violation_count()
    ));

    if outcome.violations.is_empty() {
        report.push_str("OK: every discovered microservice manifest has a valid naming proof.\n");
        return report;
    }

    for violation in &outcome.violations {
        report.push_str(&format!(
            "{}:{}: {:?}: {}\n  fix: {}\n",
            violation.path.display(),
            violation.line,
            violation.kind,
            violation.message,
            violation.suggested_fix
        ));
    }

    report
}

fn discover_manifest_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let microservices = repo_root.join("microservices");
    if !microservices.exists() {
        return Ok(Vec::new());
    }

    let mut manifests = Vec::new();
    for entry in fs::read_dir(&microservices)
        .with_context(|| format!("failed to read {}", microservices.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to inspect {}", microservices.display()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        for child in fs::read_dir(&path)
            .with_context(|| format!("failed to read microservice dir {}", path.display()))?
        {
            let child = child.with_context(|| format!("failed to inspect {}", path.display()))?;
            let child_path = child.path();
            if is_manifest_file(&child_path) && supported_manifest_extension(&child_path) {
                manifests.push(child_path);
            }
        }
    }

    manifests.sort();
    Ok(manifests)
}

fn is_manifest_file(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with("manifest."))
}

fn supported_manifest_extension(path: &Path) -> bool {
    matches!(
        extension(path).as_deref(),
        Some("json" | "yaml" | "yml" | "toml")
    )
}

fn inspect_manifest(
    repo_root: &Path,
    manifest: &Path,
    content: &str,
    violations: &mut Vec<NamingViolation>,
) {
    let line = line_number_for_key(content, "naming_justifications").unwrap_or(1);
    let relative = relative_path(repo_root, manifest);

    let field = match parse_naming_field(manifest, content) {
        Ok(field) => field,
        Err(error) => {
            violations.push(NamingViolation {
                path: relative,
                line: 1,
                kind: NamingViolationKind::ParseError,
                message: format!("manifest could not be parsed: {error}"),
                suggested_fix: "Repair manifest syntax, then add the one-line naming proof."
                    .to_string(),
            });
            return;
        }
    };

    match field {
        FieldValue::Missing => {
            violations.push(NamingViolation {
                path: relative,
                line: 1,
                kind: NamingViolationKind::MissingField,
                message: "missing top-level naming_justifications field".to_string(),
                suggested_fix: SUGGESTED_PROOF.to_string(),
            });
        }
        FieldValue::NonString => {
            violations.push(NamingViolation {
                path: relative,
                line,
                kind: NamingViolationKind::NonStringField,
                message: "naming_justifications must be a single string proof, not an object/list"
                    .to_string(),
                suggested_fix: SUGGESTED_PROOF.to_string(),
            });
        }
        FieldValue::String(proof) => {
            validate_proof(&relative, line, &proof, violations);
        }
    }
}

fn parse_naming_field(path: &Path, content: &str) -> Result<FieldValue> {
    match extension(path).as_deref() {
        Some("json") => parse_json_field(content),
        Some("yaml" | "yml") => parse_yaml_field(content),
        Some("toml") => parse_toml_field(content),
        _ => Ok(FieldValue::Missing),
    }
}

fn parse_json_field(content: &str) -> Result<FieldValue> {
    let value: serde_json::Value = serde_json::from_str(content)?;
    Ok(match value.get("naming_justifications") {
        Some(serde_json::Value::String(value)) => FieldValue::String(value.clone()),
        Some(_) => FieldValue::NonString,
        None => FieldValue::Missing,
    })
}

fn parse_yaml_field(content: &str) -> Result<FieldValue> {
    let value: serde_yaml::Value = serde_yaml::from_str(content)?;
    let key = serde_yaml::Value::String("naming_justifications".to_string());
    Ok(match value.get(&key) {
        Some(serde_yaml::Value::String(value)) => FieldValue::String(value.clone()),
        Some(_) => FieldValue::NonString,
        None => FieldValue::Missing,
    })
}

fn parse_toml_field(content: &str) -> Result<FieldValue> {
    let value: toml::Value = toml::from_str(content)?;
    Ok(match value.get("naming_justifications") {
        Some(toml::Value::String(value)) => FieldValue::String(value.clone()),
        Some(_) => FieldValue::NonString,
        None => FieldValue::Missing,
    })
}

fn validate_proof(path: &Path, line: usize, proof: &str, violations: &mut Vec<NamingViolation>) {
    let trimmed = proof.trim();
    if trimmed.is_empty() {
        violations.push(NamingViolation {
            path: path.to_path_buf(),
            line,
            kind: NamingViolationKind::EmptyProof,
            message: "naming_justifications is present but empty".to_string(),
            suggested_fix: SUGGESTED_PROOF.to_string(),
        });
        return;
    }

    if proof.lines().count() > 1 {
        violations.push(NamingViolation {
            path: path.to_path_buf(),
            line,
            kind: NamingViolationKind::MultiLineProof,
            message: "naming_justifications must be a one-line proof".to_string(),
            suggested_fix: SUGGESTED_PROOF.to_string(),
        });
    }

    if !looks_structured(trimmed) {
        violations.push(NamingViolation {
            path: path.to_path_buf(),
            line,
            kind: NamingViolationKind::UnstructuredProof,
            message: "naming proof must use structured key/value or clause separators".to_string(),
            suggested_fix: SUGGESTED_PROOF.to_string(),
        });
    }

    if !cites_bnf_v4(trimmed) {
        violations.push(NamingViolation {
            path: path.to_path_buf(),
            line,
            kind: NamingViolationKind::MissingBnfV4Citation,
            message: "naming proof must cite BNF v4 or BNF v4.1".to_string(),
            suggested_fix: SUGGESTED_PROOF.to_string(),
        });
    }

    if !cites_twelve_layer_enum(trimmed) {
        violations.push(NamingViolation {
            path: path.to_path_buf(),
            line,
            kind: NamingViolationKind::MissingTwelveLayerEnumCitation,
            message: "naming proof must cite the 12-layer-enum".to_string(),
            suggested_fix: SUGGESTED_PROOF.to_string(),
        });
    }
}

fn looks_structured(proof: &str) -> bool {
    proof.contains('=') || proof.contains(':') || proof.contains(';')
}

fn cites_bnf_v4(proof: &str) -> bool {
    let normalized = normalize_for_match(proof);
    normalized.contains("bnf v4")
        || normalized.contains("v4 bnf")
        || (normalized.contains("bnf") && normalized.contains("v4"))
}

fn cites_twelve_layer_enum(proof: &str) -> bool {
    let normalized = normalize_for_match(proof);
    normalized.contains("12 layer enum")
        || normalized.contains("12 layer enum")
        || (normalized.contains("12 layer") && normalized.contains("enum"))
}

fn normalize_for_match(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn line_number_for_key(content: &str, key: &str) -> Option<usize> {
    content.lines().enumerate().find_map(|(index, line)| {
        if line.contains(key) {
            Some(index + 1)
        } else {
            None
        }
    })
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| ext.to_ascii_lowercase())
}

fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_requires_both_citations() {
        let mut violations = Vec::new();
        validate_proof(
            Path::new("microservices/mail/manifest.yaml"),
            3,
            "BNF v4.1 action grammar only",
            &mut violations,
        );
        assert_eq!(violations.len(), 2);
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == NamingViolationKind::UnstructuredProof)
        );
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind
                    == NamingViolationKind::MissingTwelveLayerEnumCitation)
        );
    }

    #[test]
    fn structured_one_line_proof_passes() {
        let mut violations = Vec::new();
        validate_proof(
            Path::new("microservices/mail/manifest.yaml"),
            2,
            "BNF v4.1 service_action_resource=mail.notice.deliver.message; 12-layer-enum=api",
            &mut violations,
        );
        assert!(violations.is_empty());
    }
}
