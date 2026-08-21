//! # cloud-ci-enforcement-liveness (FRIC-012 / G011)
//!
//! Enforces project hook liveness for the two tracked wiring surfaces:
//! `tools/hooks/registration/claude-settings.json` and
//! `tools/hooks/registration/codex-hooks.json`.
//!
//! ## Contract
//! Input: `{"rows":[...]}` where the producer emits two row shapes:
//! - hook rows: `{"row_type":"hook","hook_path":"tools/hooks/x.sh",
//!   "wired_in_claude":bool,"wired_in_codex":bool,"stub_marked":bool}`
//! - command-reference rows: `{"row_type":"command_reference","wiring_file":"...",
//!   "command_path":"tools/hooks/x.sh","target_exists":bool}`
//!
//! `evaluate_keyed` emits one `Finding{code,key,remediation}` per violation. All codes are
//! born-blocking and frozen-empty in the firewall baseline because today's tracked tree is clean.
//! Malformed producer output is red instead of silently becoming an empty green face.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

pub const GATE_ID: &str = "cloud-ci-enforcement-liveness";

pub const VIOLATION_CODES: [&str; 5] = [
    "malformed_enforcement_liveness_face",
    "malformed_enforcement_liveness_row",
    "hook_unwired_without_stub_marker",
    "hook_wiring_mirror_drift",
    "wired_hook_missing_file",
];

pub const CLAUDE_WIRING_FILE: &str = "tools/hooks/registration/claude-settings.json";
pub const CODEX_WIRING_FILE: &str = "tools/hooks/registration/codex-hooks.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub remediation: String,
}

impl Finding {
    fn new(code: &str, key: &str, remediation: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            remediation: remediation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_codes(violations: BTreeSet<String>) -> Self {
        let verdict = if violations.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            violations,
        }
    }
}

fn bool_field(row: &Value, key: &str) -> Option<bool> {
    row.get(key).and_then(Value::as_bool)
}

fn str_field<'a>(row: &'a Value, key: &str) -> Option<&'a str> {
    row.get(key).and_then(Value::as_str)
}

fn hook_remediation(hook_path: &str) -> String {
    format!(
        "wire {hook_path} in both {CLAUDE_WIRING_FILE} and {CODEX_WIRING_FILE}, or mark it as a deliberate compatibility stub; land the correction through a governance PR"
    )
}

fn mirror_remediation(hook_path: &str) -> String {
    format!(
        "make {CLAUDE_WIRING_FILE} and {CODEX_WIRING_FILE} reference the same live hook {hook_path}; keep the two wiring files in sync through a governance PR"
    )
}

fn missing_file_remediation(wiring_file: &str, command_path: &str) -> String {
    format!(
        "edit {wiring_file} to reference an existing tracked tools/hooks/*.sh command, or restore {command_path}; land the correction through a governance PR"
    )
}

fn malformed_face_remediation(key: &str) -> String {
    format!(
        "repair enforcement-liveness producer output so `{key}` is present with the documented shape; land the correction through a governance PR"
    )
}

fn malformed_row_remediation(key: &str) -> String {
    format!(
        "repair enforcement-liveness row field `{key}` so malformed producer output cannot pass as green; land the correction through a governance PR"
    )
}

fn row_key(index: usize, field: &str) -> String {
    format!("rows[{index}].{field}")
}

fn malformed_row(index: usize, field: &str) -> Finding {
    let key = row_key(index, field);
    Finding::new(
        "malformed_enforcement_liveness_row",
        &key,
        malformed_row_remediation(&key),
    )
}

pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let Some(rows) = input.get("rows").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "malformed_enforcement_liveness_face",
            "rows",
            malformed_face_remediation("rows"),
        ));
        return findings;
    };

    for (index, row) in rows.iter().enumerate() {
        match str_field(row, "row_type") {
            Some("hook") => {
                let Some(hook_path) = str_field(row, "hook_path") else {
                    findings.insert(malformed_row(index, "hook_path"));
                    continue;
                };
                let stub_marked = bool_field(row, "stub_marked");
                let wired_in_claude = bool_field(row, "wired_in_claude");
                let wired_in_codex = bool_field(row, "wired_in_codex");
                if stub_marked.is_none() {
                    findings.insert(malformed_row(index, "stub_marked"));
                }
                if wired_in_claude.is_none() {
                    findings.insert(malformed_row(index, "wired_in_claude"));
                }
                if wired_in_codex.is_none() {
                    findings.insert(malformed_row(index, "wired_in_codex"));
                }
                let (Some(stub_marked), Some(wired_in_claude), Some(wired_in_codex)) =
                    (stub_marked, wired_in_claude, wired_in_codex)
                else {
                    continue;
                };
                if !(stub_marked || wired_in_claude && wired_in_codex) {
                    findings.insert(Finding::new(
                        "hook_unwired_without_stub_marker",
                        hook_path,
                        hook_remediation(hook_path),
                    ));
                }
                if !stub_marked && wired_in_claude != wired_in_codex {
                    findings.insert(Finding::new(
                        "hook_wiring_mirror_drift",
                        hook_path,
                        mirror_remediation(hook_path),
                    ));
                }
            }
            Some("command_reference") => {
                let Some(wiring_file) = str_field(row, "wiring_file") else {
                    findings.insert(malformed_row(index, "wiring_file"));
                    continue;
                };
                let Some(command_path) = str_field(row, "command_path") else {
                    findings.insert(malformed_row(index, "command_path"));
                    continue;
                };
                let Some(target_exists) = bool_field(row, "target_exists") else {
                    findings.insert(malformed_row(index, "target_exists"));
                    continue;
                };
                if !target_exists {
                    let key = format!("{wiring_file}:{command_path}");
                    findings.insert(Finding::new(
                        "wired_hook_missing_file",
                        &key,
                        missing_file_remediation(wiring_file, command_path),
                    ));
                }
            }
            _ => {
                findings.insert(malformed_row(index, "row_type"));
            }
        }
    }

    findings
}

pub fn evaluate(input: &Value) -> Report {
    let codes: BTreeSet<String> = evaluate_keyed(input)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    Report::from_codes(codes)
}
