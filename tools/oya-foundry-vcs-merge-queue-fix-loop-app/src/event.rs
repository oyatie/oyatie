//! Admission-event parser. Consumes
//! `registries/cross-cutting/merge-queue-admission-log.json::entries`,
//! the schema co-defined with IP-004's dispatcher.
//!
//! The parser is intentionally serde-free (zero deps); it scans for the
//! canonical key set written by the IP-004 dispatcher.

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionEventKind {
    /// `pr-review-approved` — feed into Scheduler::admit.
    PrReviewApproved,
    /// `pr-review-fix-requested` — feed into Scheduler::park(... ReviewChangesRequested, ...).
    PrReviewFixRequested,
}

impl AdmissionEventKind {
    pub fn from_wire(value: &str) -> Result<Self, EventParseError> {
        match value {
            "pr-review-approved" => Ok(AdmissionEventKind::PrReviewApproved),
            "pr-review-fix-requested" => Ok(AdmissionEventKind::PrReviewFixRequested),
            other => Err(EventParseError::UnknownKind(other.to_string())),
        }
    }

    pub fn as_wire(&self) -> &'static str {
        match self {
            AdmissionEventKind::PrReviewApproved => "pr-review-approved",
            AdmissionEventKind::PrReviewFixRequested => "pr-review-fix-requested",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionEvent {
    pub kind: AdmissionEventKind,
    pub pr_number: u64,
    pub changeset_id: String,
    pub head_sha: String,
    pub base_sha: String,
    pub emitted_at_epoch: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventParseError {
    UnknownKind(String),
    MissingKey(&'static str),
    InvalidNumber(&'static str),
    InvalidEnvelope,
}

impl fmt::Display for EventParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for EventParseError {}

/// Parse the `entries: [...]` array from the IP-004 admission-log
/// registry envelope.
pub fn parse_admission_log(json: &str) -> Result<Vec<AdmissionEvent>, EventParseError> {
    let key_idx = json
        .find("\"entries\"")
        .ok_or(EventParseError::InvalidEnvelope)?;
    let after_key = &json[key_idx + "\"entries\"".len()..];
    let colon_idx = after_key
        .find(':')
        .ok_or(EventParseError::InvalidEnvelope)?;
    let after_colon = &after_key[colon_idx + 1..];
    let trimmed = after_colon.trim_start();
    if !trimmed.starts_with('[') {
        return Err(EventParseError::InvalidEnvelope);
    }
    let close_idx = trimmed.find(']').ok_or(EventParseError::InvalidEnvelope)?;
    let body = &trimmed[1..close_idx];
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let bytes = body.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b'{' {
            if depth == 0 {
                start = i;
            }
            depth += 1;
        } else if byte == b'}' {
            depth -= 1;
            if depth == 0 {
                out.push(parse_one(&body[start..=i])?);
            }
        }
    }
    if depth != 0 {
        return Err(EventParseError::InvalidEnvelope);
    }
    Ok(out)
}

fn parse_one(slice: &str) -> Result<AdmissionEvent, EventParseError> {
    let kind_wire = scan_string(slice, "kind")?;
    let kind = AdmissionEventKind::from_wire(&kind_wire)?;
    let pr_number = scan_u64(slice, "pr_number")?;
    let changeset_id = scan_string(slice, "changeset_id")?;
    let head_sha = scan_string(slice, "head_sha")?;
    let base_sha = scan_string(slice, "base_sha")?;
    let emitted_at_epoch = scan_u64(slice, "emitted_at_epoch")?;
    Ok(AdmissionEvent {
        kind,
        pr_number,
        changeset_id,
        head_sha,
        base_sha,
        emitted_at_epoch,
    })
}

fn scan_u64(slice: &str, key: &str) -> Result<u64, EventParseError> {
    let needle = format!("\"{key}\":");
    let idx = slice
        .find(&needle)
        .ok_or(EventParseError::MissingKey(static_key(key)))?;
    let after = &slice[idx + needle.len()..].trim_start();
    let end = after
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(after.len());
    after[..end]
        .parse()
        .map_err(|_| EventParseError::InvalidNumber(static_key(key)))
}

fn scan_string(slice: &str, key: &str) -> Result<String, EventParseError> {
    let needle = format!("\"{key}\":");
    let idx = slice
        .find(&needle)
        .ok_or(EventParseError::MissingKey(static_key(key)))?;
    let after = &slice[idx + needle.len()..].trim_start();
    if !after.starts_with('"') {
        return Err(EventParseError::InvalidEnvelope);
    }
    let mut chars = after[1..].chars();
    let mut out = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                let esc = chars.next().ok_or(EventParseError::InvalidEnvelope)?;
                match esc {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    _ => return Err(EventParseError::InvalidEnvelope),
                }
            }
            '"' => return Ok(out),
            ch => out.push(ch),
        }
    }
    Err(EventParseError::InvalidEnvelope)
}

fn static_key(key: &str) -> &'static str {
    match key {
        "kind" => "kind",
        "pr_number" => "pr_number",
        "changeset_id" => "changeset_id",
        "head_sha" => "head_sha",
        "base_sha" => "base_sha",
        "emitted_at_epoch" => "emitted_at_epoch",
        _ => "<unknown>",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_envelope() {
        let json = r#"{"_meta":{},"entries":[
            {"base_sha":"2222222222222222222222222222222222222222","changeset_id":"cs_a","emitted_at_epoch":1,"head_sha":"1111111111111111111111111111111111111111","kind":"pr-review-approved","pr_number":42}
        ]}"#;
        let events = parse_admission_log(json).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, AdmissionEventKind::PrReviewApproved);
        assert_eq!(events[0].pr_number, 42);
        assert_eq!(events[0].changeset_id, "cs_a");
    }

    #[test]
    fn parses_multiple_entries_and_both_kinds() {
        let json = r#"{"_meta":{},"entries":[
            {"base_sha":"2222222222222222222222222222222222222222","changeset_id":"cs_a","emitted_at_epoch":1,"head_sha":"1111111111111111111111111111111111111111","kind":"pr-review-approved","pr_number":42},
            {"base_sha":"3333333333333333333333333333333333333333","changeset_id":"cs_b","emitted_at_epoch":2,"head_sha":"4444444444444444444444444444444444444444","kind":"pr-review-fix-requested","pr_number":43}
        ]}"#;
        let events = parse_admission_log(json).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].kind, AdmissionEventKind::PrReviewFixRequested);
    }

    #[test]
    fn rejects_unknown_kind() {
        let json = r#"{"_meta":{},"entries":[
            {"base_sha":"2222222222222222222222222222222222222222","changeset_id":"cs_a","emitted_at_epoch":1,"head_sha":"1111111111111111111111111111111111111111","kind":"unknown","pr_number":1}
        ]}"#;
        let err = parse_admission_log(json).unwrap_err();
        match err {
            EventParseError::UnknownKind(s) => assert_eq!(s, "unknown"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_envelope() {
        let err = parse_admission_log("not json").unwrap_err();
        assert_eq!(err, EventParseError::InvalidEnvelope);
    }
}
