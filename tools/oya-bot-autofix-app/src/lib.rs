//! ADR-0531 `oya-bot-autofix` skeleton.
//!
//! This crate is intentionally narrow: it can render a dry-run preview for
//! described `oya-ci-gate-contract` remediation data, and its policy surface is
//! PROPOSE-only. It does not contain merge or gate-bypass capabilities.

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use oya_ci_gate_contract::{Edit, NewFile, Remediation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    DryRun,
    ProposeReviewablePullRequest,
    MergePullRequest,
    BypassGates,
}

impl Action {
    fn denial_reason(self) -> Option<&'static str> {
        match self {
            Self::DryRun | Self::ProposeReviewablePullRequest => None,
            Self::MergePullRequest => {
                Some("oya-bot-autofix is PROPOSE-only and cannot merge pull requests")
            }
            Self::BypassGates => {
                Some("oya-bot-autofix cannot bypass oya-ci-required or any adopter gate fan-in")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BotPolicy;

impl BotPolicy {
    pub fn propose_only() -> Self {
        Self
    }

    pub fn authorize(&self, action: Action) -> Result<(), AutofixError> {
        let _ = self;
        if let Some(reason) = action.denial_reason() {
            Err(AutofixError::ForbiddenAction { action, reason })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryMode {
    DryRun,
    ProposeReviewablePullRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafetyEnvelope {
    pub mode: DeliveryMode,
    pub writes_performed: bool,
    pub can_merge: bool,
    pub can_bypass_gates: bool,
}

impl SafetyEnvelope {
    pub fn dry_run() -> Self {
        Self {
            mode: DeliveryMode::DryRun,
            writes_performed: false,
            can_merge: false,
            can_bypass_gates: false,
        }
    }

    pub fn propose_reviewable_pull_request() -> Self {
        Self {
            mode: DeliveryMode::ProposeReviewablePullRequest,
            writes_performed: true,
            can_merge: false,
            can_bypass_gates: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DryRunInput<'a> {
    pub remediation: &'a Remediation,
    pub original_text: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunReport {
    pub diff: String,
    pub safety: SafetyEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutofixError {
    ForbiddenAction {
        action: Action,
        reason: &'static str,
    },
    NoRemediation,
    ByteRangeOutOfBounds {
        path: String,
        start: usize,
        end: usize,
        len: usize,
    },
    ByteRangeNotUtf8Boundary {
        path: String,
        start: usize,
        end: usize,
    },
}

impl fmt::Display for AutofixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForbiddenAction { reason, .. } => write!(f, "{reason}"),
            Self::NoRemediation => write!(f, "no remediation was available to render"),
            Self::ByteRangeOutOfBounds {
                path,
                start,
                end,
                len,
            } => write!(
                f,
                "remediation byte range {start}..{end} is outside {path} length {len}"
            ),
            Self::ByteRangeNotUtf8Boundary { path, start, end } => write!(
                f,
                "remediation byte range {start}..{end} does not align to UTF-8 boundaries in {path}"
            ),
        }
    }
}

impl Error for AutofixError {}

pub fn render_dry_run(input: DryRunInput<'_>) -> Result<DryRunReport, AutofixError> {
    BotPolicy::propose_only().authorize(Action::DryRun)?;

    let diff = match input.remediation {
        Remediation::AutoFix(edit) => render_edit_diff(edit, input.original_text)?,
        Remediation::AutoGenerate(new_file) => render_new_file_diff(new_file),
        Remediation::None => return Err(AutofixError::NoRemediation),
    };

    Ok(DryRunReport {
        diff,
        safety: SafetyEnvelope::dry_run(),
    })
}

fn render_edit_diff(edit: &Edit, original_text: &str) -> Result<String, AutofixError> {
    let len = original_text.len();
    let start = edit.byte_range.start;
    let end = edit.byte_range.end;
    if start > end || end > len {
        return Err(AutofixError::ByteRangeOutOfBounds {
            path: edit.path.clone(),
            start,
            end,
            len,
        });
    }
    if !original_text.is_char_boundary(start) || !original_text.is_char_boundary(end) {
        return Err(AutofixError::ByteRangeNotUtf8Boundary {
            path: edit.path.clone(),
            start,
            end,
        });
    }

    let mut updated = String::with_capacity(original_text.len() + edit.replacement.len());
    updated.push_str(&original_text[..start]);
    updated.push_str(&edit.replacement);
    updated.push_str(&original_text[end..]);

    Ok(render_whole_file_unified_diff(
        &edit.path,
        original_text,
        &updated,
    ))
}

fn render_new_file_diff(new_file: &NewFile) -> String {
    let mut diff = String::new();
    diff.push_str(&format!(
        "diff --git a/{path} b/{path}\n",
        path = new_file.path
    ));
    diff.push_str("--- /dev/null\n");
    diff.push_str(&format!("+++ b/{path}\n", path = new_file.path));
    diff.push_str(&format!("@@ -0,0 +1,{} @@\n", line_count(&new_file.body)));
    for line in new_file.body.lines() {
        diff.push('+');
        diff.push_str(line);
        diff.push('\n');
    }
    diff
}

fn render_whole_file_unified_diff(path: &str, original: &str, updated: &str) -> String {
    let mut diff = String::new();
    diff.push_str(&format!("diff --git a/{path} b/{path}\n"));
    diff.push_str(&format!("--- a/{path}\n"));
    diff.push_str(&format!("+++ b/{path}\n"));
    diff.push_str(&format!(
        "@@ -1,{} +1,{} @@\n",
        line_count(original),
        line_count(updated)
    ));
    for line in original.lines() {
        diff.push('-');
        diff.push_str(line);
        diff.push('\n');
    }
    for line in updated.lines() {
        diff.push('+');
        diff.push_str(line);
        diff.push('\n');
    }
    diff
}

fn line_count(text: &str) -> usize {
    text.lines().count()
}
