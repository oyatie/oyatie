//! Governance check for BYOK terminology disambiguation.
//!
//! The lane scans Markdown under `docs/` and `microservices/` for `BYOK`.
//! Each mention must be classifiable as provider-BYOK under ADR-0255 §D-4,
//! encryption-BYOK under ADR-0251 §D-10, or an explicit contrast that names
//! both terms. Bare "BYOK" prose is rejected because it collapses provider
//! credential ownership with cryptographic key ownership.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;
use walkdir::WalkDir;

/// Stable identifier for the governance rule enforced by this crate.
pub const RULE_ID: &str = "ADR-0255-D4+ADR-0251-D10";

/// Human-readable summary of the rule this crate enforces.
pub const ENFORCED_RULE: &str =
    "Docs and implementation packets disambiguate provider-BYOK from encryption-BYOK.";

const PROVIDER_FIX: &str =
    "Use provider-BYOK (ADR-0255 §D-4) for external provider credentials/API keys.";
const ENCRYPTION_FIX: &str =
    "Use encryption-BYOK (ADR-0251 §D-10) for KMS, KEK, CMK, HSM, or envelope-encryption keys.";

/// Machine-readable pass/fail state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementStatus {
    Passed,
    Failed,
}

/// Accepted classification for a BYOK mention.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ByokClassification {
    ProviderByok,
    EncryptionByok,
    ExplicitProviderAndEncryptionContrast,
}

/// Specific violation kind.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ByokViolationKind {
    AmbiguousByok,
    CollapsedProviderAndEncryptionByok,
}

/// A classified BYOK mention.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ByokReference {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub classification: ByokClassification,
    pub excerpt: String,
}

/// One ambiguous BYOK mention with a suggested replacement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ByokViolation {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub kind: ByokViolationKind,
    pub excerpt: String,
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
    pub scanned_markdown_files: usize,
    pub references: Vec<ByokReference>,
    pub violations: Vec<ByokViolation>,
}

impl GovernanceCheckOutcome {
    pub fn is_success(&self) -> bool {
        self.status == EnforcementStatus::Passed
    }

    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClassificationDecision {
    Classified(ByokClassification),
    Ambiguous,
    Collapsed,
}

/// Enforces BYOK terminology disambiguation across docs and implementation packets.
pub fn enforce_byok_disambiguation(repo_root: impl AsRef<Path>) -> Result<GovernanceCheckOutcome> {
    let repo_root = repo_root.as_ref().to_path_buf();
    let files = discover_markdown_files(&repo_root)?;
    let mut references = Vec::new();
    let mut violations = Vec::new();

    for file in &files {
        let content = fs::read_to_string(file)
            .with_context(|| format!("failed to read {}", file.display()))?;
        inspect_markdown_file(&repo_root, file, &content, &mut references, &mut violations);
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
        scanned_markdown_files: files.len(),
        references,
        violations,
    })
}

/// Formats a compact text report for CLI output.
pub fn format_text_report(outcome: &GovernanceCheckOutcome) -> String {
    let mut report = String::new();
    report.push_str(&format!(
        "{}: {:?} ({} markdown files, {} BYOK references, {} violations)\n",
        outcome.rule_id,
        outcome.status,
        outcome.scanned_markdown_files,
        outcome.references.len(),
        outcome.violation_count()
    ));

    if outcome.violations.is_empty() {
        report.push_str("OK: every BYOK reference is disambiguated.\n");
        return report;
    }

    for violation in &outcome.violations {
        report.push_str(&format!(
            "{}:{}:{}: {:?}: {}\n  excerpt: {}\n  fix: {}\n",
            violation.path.display(),
            violation.line,
            violation.column,
            violation.kind,
            violation.message,
            violation.excerpt,
            violation.suggested_fix
        ));
    }

    report
}

fn discover_markdown_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for base in ["docs", "microservices"] {
        let root = repo_root.join(base);
        if !root.exists() {
            continue;
        }

        for entry in WalkDir::new(&root).follow_links(false) {
            let entry = entry.with_context(|| format!("failed to walk {}", root.display()))?;
            if entry.file_type().is_file() && has_markdown_extension(entry.path()) {
                files.push(entry.path().to_path_buf());
            }
        }
    }

    files.sort();
    Ok(files)
}

fn inspect_markdown_file(
    repo_root: &Path,
    file: &Path,
    content: &str,
    references: &mut Vec<ByokReference>,
    violations: &mut Vec<ByokViolation>,
) {
    let lines: Vec<&str> = content.lines().collect();
    let relative = relative_path(repo_root, file);

    for (index, line) in lines.iter().enumerate() {
        let columns = byok_columns(line);
        if columns.is_empty() {
            continue;
        }

        let context = context_window(&lines, index, 3);
        let decision = classify_byok_context(line, &context);
        for column in columns {
            match decision {
                ClassificationDecision::Classified(classification) => {
                    references.push(ByokReference {
                        path: relative.clone(),
                        line: index + 1,
                        column,
                        classification,
                        excerpt: excerpt(line),
                    });
                }
                ClassificationDecision::Ambiguous => {
                    violations.push(ByokViolation {
                        path: relative.clone(),
                        line: index + 1,
                        column,
                        kind: ByokViolationKind::AmbiguousByok,
                        excerpt: excerpt(line),
                        message: "BYOK reference does not say provider-BYOK or encryption-BYOK"
                            .to_string(),
                        suggested_fix: format!("{PROVIDER_FIX} {ENCRYPTION_FIX}"),
                    });
                }
                ClassificationDecision::Collapsed => {
                    violations.push(ByokViolation {
                        path: relative.clone(),
                        line: index + 1,
                        column,
                        kind: ByokViolationKind::CollapsedProviderAndEncryptionByok,
                        excerpt: excerpt(line),
                        message: "BYOK context mixes provider credentials and encryption keys without an explicit contrast".to_string(),
                        suggested_fix:
                            "Split the sentence into provider-BYOK (ADR-0255 §D-4) and encryption-BYOK (ADR-0251 §D-10).".to_string(),
                    });
                }
            }
        }
    }
}

fn classify_byok_context(line: &str, context: &str) -> ClassificationDecision {
    let line_norm = normalize(line);
    let context_norm = normalize(context);
    let line_has_provider_term = has_provider_byok_term(&line_norm);
    let line_has_encryption_term = has_encryption_byok_term(&line_norm);

    if line_has_provider_term && line_has_encryption_term {
        return ClassificationDecision::Classified(
            ByokClassification::ExplicitProviderAndEncryptionContrast,
        );
    }

    if line_has_provider_term {
        return ClassificationDecision::Classified(ByokClassification::ProviderByok);
    }

    if line_has_encryption_term {
        return ClassificationDecision::Classified(ByokClassification::EncryptionByok);
    }

    let provider_score = provider_context_score(&context_norm);
    let encryption_score = encryption_context_score(&context_norm);

    match (provider_score > 0, encryption_score > 0) {
        (true, false) => ClassificationDecision::Classified(ByokClassification::ProviderByok),
        (false, true) => ClassificationDecision::Classified(ByokClassification::EncryptionByok),
        (true, true) => ClassificationDecision::Collapsed,
        (false, false) => ClassificationDecision::Ambiguous,
    }
}

fn provider_context_score(context: &str) -> usize {
    let cues = [
        "adr 0255",
        "d 4",
        "provider credential",
        "provider credentials",
        "external provider",
        "api key",
        "api keys",
        "oauth",
        "credential handle",
        "credential handles",
        "provider mode",
        "provider credential mode",
        "provider sidecar",
        "tenant provider",
    ];
    cues.iter().filter(|cue| context.contains(**cue)).count()
}

fn encryption_context_score(context: &str) -> usize {
    let cues = [
        "adr 0251",
        "d 10",
        "encryption key",
        "encryption keys",
        "cryptographic key",
        "cryptographic keys",
        "kms",
        "cloud kms",
        "cmk",
        "kek",
        "hsm",
        "envelope encryption",
        "key material",
        "tenant key",
        "tenant keys",
    ];
    cues.iter().filter(|cue| context.contains(**cue)).count()
}

fn has_provider_byok_term(value: &str) -> bool {
    value.contains("provider byok")
        || value.contains("provider credential byok")
        || value.contains("provider credential mode")
}

fn has_encryption_byok_term(value: &str) -> bool {
    value.contains("encryption byok")
        || value.contains("encryption key byok")
        || value.contains("key byok")
}

fn byok_columns(line: &str) -> Vec<usize> {
    let lower = line.to_ascii_lowercase();
    lower
        .match_indices("byok")
        .map(|(index, _)| index + 1)
        .collect()
}

fn context_window(lines: &[&str], index: usize, radius: usize) -> String {
    let start = index.saturating_sub(radius);
    let end = (index + radius + 1).min(lines.len());
    lines[start..end].join("\n")
}

fn excerpt(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.len() <= 180 {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..180])
    }
}

fn has_markdown_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

fn normalize(value: &str) -> String {
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

fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_provider_term_wins() {
        assert_eq!(
            classify_byok_context("provider-BYOK keeps API keys in the sidecar", ""),
            ClassificationDecision::Classified(ByokClassification::ProviderByok)
        );
    }

    #[test]
    fn explicit_encryption_term_wins() {
        assert_eq!(
            classify_byok_context("encryption-BYOK uses tenant CMKs", ""),
            ClassificationDecision::Classified(ByokClassification::EncryptionByok)
        );
    }

    #[test]
    fn bare_byok_without_context_is_ambiguous() {
        assert_eq!(
            classify_byok_context("BYOK is supported for enterprise tenants", ""),
            ClassificationDecision::Ambiguous
        );
    }
}
