//! # cloud-ci-zero-static-secrets
//!
//! Portable static-secret hygiene gate for cloud-ci. The producer owns repository I/O: it scans
//! the declared tracked-path corpus from `scm-facts.generated.json`, keeps only high-signal
//! credential-shaped candidate lines, and feeds this pure evaluator rows shaped as
//! `{path,line,text}` plus policy DATA. The evaluator emits redacted finding keys/details only;
//! raw secret material never enters findings or baseline keys.
//!
//! Bootstrap exceptions are DATA, not inline comments: a candidate line is permitted only when a
//! `policy.bootstrap_exceptions[]` row matches its path, secret kind, and non-secret line marker.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// The gate id, matching oya-ci config, workflow registration, and the baseline ratchet.
pub const GATE_ID: &str = "cloud-ci-zero-static-secrets";

/// Stable blocking violation codes emitted by this gate.
pub const VIOLATION_CODES: [&str; 6] = [
    "static_secret_detected",
    "static_secret_policy_gate_id_mismatch",
    "static_secret_exception_duplicate",
    "static_secret_exception_missing_field",
    "static_secret_observed_row_missing_field",
    "static_secret_no_scanned_paths",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// A keyed violation: stable `code`, redacted `key`, and redacted detail.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect::<BTreeSet<_>>();
        Self {
            verdict: if violations.is_empty() {
                Verdict::Green
            } else {
                Verdict::Red
            },
            violations,
        }
    }
}

#[derive(Debug, Clone)]
struct BootstrapException {
    path: String,
    secret_kind: String,
    line_contains: String,
}

impl BootstrapException {
    fn matches(&self, path: &str, secret_kind: &str, text: &str) -> bool {
        self.path == path
            && (self.secret_kind == "*" || self.secret_kind == secret_kind)
            && text.contains(self.line_contains.as_str())
    }
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).map(str::trim)
}

fn required_nonblank<'a>(
    object: &'a Value,
    field: &str,
    exception_id: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<&'a str> {
    match string_field(object, field).filter(|value| !value.is_empty()) {
        Some(value) => Some(value),
        None => {
            findings.insert(Finding::new(
                "static_secret_exception_missing_field",
                exception_id,
                format!("bootstrap exception missing non-empty `{field}`"),
            ));
            None
        }
    }
}

fn bootstrap_exceptions(
    policy: &Value,
    findings: &mut BTreeSet<Finding>,
) -> Vec<BootstrapException> {
    let gate_id = string_field(policy, "gate_id").unwrap_or_default();
    if gate_id != GATE_ID {
        findings.insert(Finding::new(
            "static_secret_policy_gate_id_mismatch",
            "<policy>",
            format!("policy gate_id does not match `{GATE_ID}`"),
        ));
    }

    let mut exceptions = Vec::new();
    let mut seen_ids: BTreeMap<String, usize> = BTreeMap::new();
    let rows = policy
        .get("bootstrap_exceptions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for (idx, exception) in rows.iter().enumerate() {
        let fallback_id = format!("<bootstrap-exception-{idx}>");
        let id = string_field(exception, "id")
            .filter(|value| !value.is_empty())
            .unwrap_or(fallback_id.as_str());
        let seen = seen_ids.entry(id.to_owned()).or_insert(0);
        *seen += 1;
        if *seen > 1 {
            findings.insert(Finding::new(
                "static_secret_exception_duplicate",
                id,
                "duplicate bootstrap exception id",
            ));
        }

        let Some(path) = required_nonblank(exception, "path", id, findings) else {
            continue;
        };
        let Some(secret_kind) = required_nonblank(exception, "secret_kind", id, findings) else {
            continue;
        };
        let Some(line_contains) = required_nonblank(exception, "line_contains", id, findings)
        else {
            continue;
        };
        if required_nonblank(exception, "owner", id, findings).is_none()
            || required_nonblank(exception, "reason", id, findings).is_none()
            || required_nonblank(exception, "replacement_contract", id, findings).is_none()
        {
            continue;
        }

        exceptions.push(BootstrapException {
            path: path.to_owned(),
            secret_kind: secret_kind.to_owned(),
            line_contains: line_contains.to_owned(),
        });
    }

    exceptions
}

fn is_secret_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
}

fn is_ascii_alnum_or_underscore(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn contains_prefixed_token(
    text: &str,
    prefix: &str,
    min_tail: usize,
    tail_char_ok: fn(char) -> bool,
) -> bool {
    let mut search_from = 0;
    while let Some(relative) = text[search_from..].find(prefix) {
        let start = search_from + relative;
        let has_boundary = start == 0
            || text[..start]
                .chars()
                .next_back()
                .is_none_or(|ch| !is_secret_char(ch));
        let is_explicit_redaction_marker = text[..start].ends_with("redacted:");
        let tail = &text[start + prefix.len()..];
        let tail_len = tail.chars().take_while(|ch| tail_char_ok(*ch)).count();
        if has_boundary && !is_explicit_redaction_marker && tail_len >= min_tail {
            return true;
        }
        search_from = start + prefix.len();
    }
    false
}

fn contains_authorization_bearer(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let Some(start) = lower.find("authorization: bearer ") else {
        return false;
    };
    let token_start = start + "authorization: bearer ".len();
    text[token_start..]
        .chars()
        .take_while(|ch| is_secret_char(*ch))
        .count()
        >= 20
}

fn detect_static_secret_kinds(text: &str) -> BTreeSet<&'static str> {
    let mut kinds = BTreeSet::new();
    if contains_prefixed_token(text, "sk-", 40, is_secret_char) {
        kinds.insert("openai_or_anthropic_key");
    }
    if contains_prefixed_token(text, "AKIA", 16, |ch| {
        ch.is_ascii_uppercase() || ch.is_ascii_digit()
    }) {
        kinds.insert("aws_access_key_id");
    }
    if contains_prefixed_token(text, "ghp_", 36, |ch| ch.is_ascii_alphanumeric())
        || contains_prefixed_token(text, "gho_", 36, |ch| ch.is_ascii_alphanumeric())
        || contains_prefixed_token(text, "github_pat_", 82, is_ascii_alnum_or_underscore)
    {
        kinds.insert("github_token");
    }
    if contains_authorization_bearer(text) {
        kinds.insert("authorization_bearer_token");
    }
    kinds
}

/// True when a line contains a credential-shaped literal that this gate knows how to classify.
///
/// The producer uses this as a lossless candidate filter before emitting rows to the pure
/// evaluator: rows may be a subset of all scanned lines, but only because this exact predicate says
/// the line can produce findings. Keeping the detector in this crate prevents producer/evaluator
/// pattern drift.
pub fn has_static_secret_candidate(text: &str) -> bool {
    !detect_static_secret_kinds(text).is_empty()
}

fn assert_scanned_paths(input: &Value, findings: &mut BTreeSet<Finding>) {
    let scanned_paths = input
        .get("_provenance")
        .and_then(|value| value.get("scanned_paths"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if scanned_paths == 0 {
        findings.insert(Finding::new(
            "static_secret_no_scanned_paths",
            "<zero-static-secrets-corpus>",
            "producer face did not report any scanned tracked paths",
        ));
    }
}

fn row_path_text_line<'a>(
    row: &'a Value,
    idx: usize,
    findings: &mut BTreeSet<Finding>,
) -> Option<(&'a str, &'a str, u64)> {
    let key = format!("<row-{idx}>");
    let Some(path) = string_field(row, "path").filter(|value| !value.is_empty()) else {
        findings.insert(Finding::new(
            "static_secret_observed_row_missing_field",
            key,
            "observed row missing non-empty `path`",
        ));
        return None;
    };
    let Some(text) = string_field(row, "text") else {
        findings.insert(Finding::new(
            "static_secret_observed_row_missing_field",
            key,
            "observed row missing string `text`",
        ));
        return None;
    };
    let Some(line) = row.get("line").and_then(Value::as_u64) else {
        findings.insert(Finding::new(
            "static_secret_observed_row_missing_field",
            key,
            "observed row missing numeric `line`",
        ));
        return None;
    };
    Some((path, text, line))
}

/// Pure evaluator: takes
/// `{"_provenance":{"scanned_paths":N},"policy":{...},"rows":[{"path","line","text"}]}`
/// and emits one finding per unexceptioned static-secret-shaped literal. Inline allow comments are
/// deliberately ignored; only matching `policy.bootstrap_exceptions[]` DATA can permit a bootstrap
/// secret-shaped line. Findings are redacted by construction: `key` is `path:line:secret_kind`, and
/// `detail` names the remediation without echoing the matched text.
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    assert_scanned_paths(input, &mut findings);
    let policy = input.get("policy").unwrap_or(&Value::Null);
    let exceptions = bootstrap_exceptions(policy, &mut findings);
    let rows = input
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for (idx, row) in rows.iter().enumerate() {
        let Some((path, text, line)) = row_path_text_line(row, idx, &mut findings) else {
            continue;
        };
        for secret_kind in detect_static_secret_kinds(text) {
            let is_policy_exception = exceptions
                .iter()
                .any(|exception| exception.matches(path, secret_kind, text));
            if !is_policy_exception {
                findings.insert(Finding::new(
                    "static_secret_detected",
                    format!("{path}:{line}:{secret_kind}"),
                    "credential-shaped literal detected; use SecretReference/workload identity or a policy-declared bootstrap exception",
                ));
            }
        }
    }

    findings
}

/// Bare-code projection of [`evaluate_keyed`] — the single source of truth for the verdict.
pub fn evaluate(input: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(input))
}
