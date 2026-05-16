//! Shared retry-budget pool — N=5 attempts per PR across BOTH CI and review sources.
//!
//! Per IP-005 §"Shared pool of N=5 attempts per PR across BOTH sources":
//! a PR doesn't get N CI retries AND N review retries; total N=5 across
//! both sources per PR. The 6th occurrence on a given PR is NOT dispatched
//! — instead the dispatcher escalates via [`crate::escalation::open_stuck_pr_issue`].
//!
//! The counter is a pure value type so the kernel logic is testable
//! without filesystem I/O; the binary entrypoint (in `main.rs`) wraps it
//! around the registry file `registry/ci-fix-loop-retry-budget.json`.

use std::collections::BTreeMap;
use std::fmt;

use crate::event::{FixLoopSource, json_string};

pub const MAX_ATTEMPTS_PER_PR: u32 = 5;

/// One entry in the retry-budget registry: PR-keyed shared counter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrBudgetEntry {
    pub pr_number: u64,
    pub attempts_used: u32,
    /// Per-source breakdown — informational only; the budget is shared, so
    /// `ci_attempts + review_attempts == attempts_used` at all times.
    pub ci_attempts: u32,
    pub review_attempts: u32,
    pub last_attempt_at_epoch: u64,
    pub escalated: bool,
}

impl PrBudgetEntry {
    pub fn new(pr_number: u64) -> Self {
        Self {
            pr_number,
            attempts_used: 0,
            ci_attempts: 0,
            review_attempts: 0,
            last_attempt_at_epoch: 0,
            escalated: false,
        }
    }

    /// JSON object representation (alphabetical-key, diff-friendly).
    pub fn to_json_object(&self) -> String {
        format!(
            "{{\"attempts_used\":{used},\"ci_attempts\":{ci},\"escalated\":{esc},\"last_attempt_at_epoch\":{last},\"pr_number\":{pr},\"review_attempts\":{rev}}}",
            used = self.attempts_used,
            ci = self.ci_attempts,
            esc = self.escalated,
            last = self.last_attempt_at_epoch,
            pr = self.pr_number,
            rev = self.review_attempts,
        )
    }
}

/// In-memory pure value form of the shared-pool retry-budget registry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Budget {
    entries: BTreeMap<u64, PrBudgetEntry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BudgetDecision {
    /// Bundle MAY be emitted — caller writes the bundle file and appends
    /// the dispatch event. The new attempt number is returned.
    DispatchAttempt(u32),
    /// PR has already exhausted the shared-pool budget. Caller MUST NOT
    /// emit a bundle; instead it MUST invoke
    /// [`crate::escalation::open_stuck_pr_issue`] (unless already
    /// escalated — `already_escalated == true` distinguishes the two).
    Escalate { already_escalated: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BudgetError {
    EpochZero,
    InvalidPrNumber,
}

impl fmt::Display for BudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for BudgetError {}

impl Budget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_entries(entries: impl IntoIterator<Item = PrBudgetEntry>) -> Self {
        let mut budget = Self::new();
        for entry in entries {
            budget.entries.insert(entry.pr_number, entry);
        }
        budget
    }

    pub fn entry(&self, pr_number: u64) -> Option<&PrBudgetEntry> {
        self.entries.get(&pr_number)
    }

    pub fn entries(&self) -> impl Iterator<Item = &PrBudgetEntry> {
        self.entries.values()
    }

    /// Register a new dispatch attempt for `pr_number` from `source`.
    ///
    /// Returns:
    /// - `DispatchAttempt(n)` if `attempts_used < MAX_ATTEMPTS_PER_PR` —
    ///   the entry is mutated in place (attempts_used += 1, per-source
    ///   counter += 1, last_attempt_at_epoch updated).
    /// - `Escalate { already_escalated }` if the PR has already used
    ///   `MAX_ATTEMPTS_PER_PR`. On the first escalation the `escalated`
    ///   flag is set; subsequent calls return `already_escalated = true`
    ///   so the dispatcher can no-op idempotently.
    pub fn register_attempt(
        &mut self,
        pr_number: u64,
        source: FixLoopSource,
        now_epoch: u64,
    ) -> Result<BudgetDecision, BudgetError> {
        if pr_number == 0 {
            return Err(BudgetError::InvalidPrNumber);
        }
        if now_epoch == 0 {
            return Err(BudgetError::EpochZero);
        }
        let entry = self
            .entries
            .entry(pr_number)
            .or_insert_with(|| PrBudgetEntry::new(pr_number));
        if entry.attempts_used >= MAX_ATTEMPTS_PER_PR {
            let already_escalated = entry.escalated;
            entry.escalated = true;
            return Ok(BudgetDecision::Escalate { already_escalated });
        }
        entry.attempts_used += 1;
        match source {
            FixLoopSource::CiFailure => entry.ci_attempts += 1,
            FixLoopSource::PrReviewFixRequested => entry.review_attempts += 1,
        }
        entry.last_attempt_at_epoch = now_epoch;
        Ok(BudgetDecision::DispatchAttempt(entry.attempts_used))
    }

    /// JSON serialization compatible with
    /// `registry/ci-fix-loop-retry-budget.json::entries`.
    pub fn render_entries_json_array(&self) -> String {
        let body = self
            .entries
            .values()
            .map(PrBudgetEntry::to_json_object)
            .collect::<Vec<_>>()
            .join(",");
        format!("[{body}]")
    }

    /// Wrap entries in the full registry envelope with the `_meta` block
    /// preserved verbatim from the scaffolded file.
    pub fn render_registry_json(&self, meta_json_block: &str) -> String {
        format!(
            "{{\"_meta\":{meta},\"entries\":{entries}}}",
            meta = meta_json_block,
            entries = self.render_entries_json_array(),
        )
    }
}

/// Parse a registry file's `entries` array out of a serde-free JSON blob.
///
/// We intentionally avoid pulling serde to keep the dispatcher dependency
/// footprint at zero; the parser is intentionally strict and rejects any
/// shape other than what [`Budget::render_entries_json_array`] emits.
pub fn parse_entries_block(json: &str) -> Result<Vec<PrBudgetEntry>, BudgetError> {
    // Find `"entries": [ ... ]` block; tolerate whitespace.
    let key_idx = json
        .find("\"entries\"")
        .ok_or(BudgetError::InvalidPrNumber)?;
    let after_key = &json[key_idx + "\"entries\"".len()..];
    let colon_idx = after_key.find(':').ok_or(BudgetError::InvalidPrNumber)?;
    let after_colon = &after_key[colon_idx + 1..];
    let trimmed = after_colon.trim_start();
    if !trimmed.starts_with('[') {
        return Err(BudgetError::InvalidPrNumber);
    }
    let close_idx = trimmed.find(']').ok_or(BudgetError::InvalidPrNumber)?;
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
                let slice = &body[start..=i];
                out.push(parse_one_entry(slice)?);
            }
        }
    }
    if depth != 0 {
        return Err(BudgetError::InvalidPrNumber);
    }
    Ok(out)
}

fn parse_one_entry(slice: &str) -> Result<PrBudgetEntry, BudgetError> {
    let attempts_used = scan_u32(slice, "attempts_used")?;
    let ci_attempts = scan_u32(slice, "ci_attempts")?;
    let review_attempts = scan_u32(slice, "review_attempts")?;
    let last_attempt_at_epoch = scan_u64(slice, "last_attempt_at_epoch")?;
    let pr_number = scan_u64(slice, "pr_number")?;
    let escalated = scan_bool(slice, "escalated")?;
    if pr_number == 0 {
        return Err(BudgetError::InvalidPrNumber);
    }
    Ok(PrBudgetEntry {
        pr_number,
        attempts_used,
        ci_attempts,
        review_attempts,
        last_attempt_at_epoch,
        escalated,
    })
}

fn scan_u32(slice: &str, key: &str) -> Result<u32, BudgetError> {
    let value = scan_u64(slice, key)?;
    u32::try_from(value).map_err(|_| BudgetError::InvalidPrNumber)
}

fn scan_u64(slice: &str, key: &str) -> Result<u64, BudgetError> {
    let needle = format!("{}:", json_string(key));
    let idx = slice.find(&needle).ok_or(BudgetError::InvalidPrNumber)?;
    let after = &slice[idx + needle.len()..];
    let trimmed = after.trim_start();
    let end = trimmed
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let number = &trimmed[..end];
    number.parse().map_err(|_| BudgetError::InvalidPrNumber)
}

fn scan_bool(slice: &str, key: &str) -> Result<bool, BudgetError> {
    let needle = format!("{}:", json_string(key));
    let idx = slice.find(&needle).ok_or(BudgetError::InvalidPrNumber)?;
    let after = &slice[idx + needle.len()..].trim_start();
    if after.starts_with("true") {
        Ok(true)
    } else if after.starts_with("false") {
        Ok(false)
    } else {
        Err(BudgetError::InvalidPrNumber)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_budget_increments_attempts_used_across_sources() {
        let mut budget = Budget::new();
        let d1 = budget
            .register_attempt(42, FixLoopSource::CiFailure, 100)
            .unwrap();
        let d2 = budget
            .register_attempt(42, FixLoopSource::PrReviewFixRequested, 200)
            .unwrap();
        let d3 = budget
            .register_attempt(42, FixLoopSource::CiFailure, 300)
            .unwrap();
        assert_eq!(d1, BudgetDecision::DispatchAttempt(1));
        assert_eq!(d2, BudgetDecision::DispatchAttempt(2));
        assert_eq!(d3, BudgetDecision::DispatchAttempt(3));
        let entry = budget.entry(42).unwrap();
        assert_eq!(entry.attempts_used, 3);
        assert_eq!(entry.ci_attempts, 2);
        assert_eq!(entry.review_attempts, 1);
        assert_eq!(entry.last_attempt_at_epoch, 300);
        assert!(!entry.escalated);
    }

    #[test]
    fn sixth_attempt_returns_escalate_first_then_already_escalated() {
        let mut budget = Budget::new();
        for i in 0..MAX_ATTEMPTS_PER_PR {
            let decision = budget
                .register_attempt(42, FixLoopSource::CiFailure, u64::from(i + 1))
                .unwrap();
            assert_eq!(decision, BudgetDecision::DispatchAttempt(i + 1));
        }
        let sixth = budget
            .register_attempt(42, FixLoopSource::CiFailure, 1000)
            .unwrap();
        assert_eq!(
            sixth,
            BudgetDecision::Escalate {
                already_escalated: false,
            }
        );
        let seventh = budget
            .register_attempt(42, FixLoopSource::CiFailure, 1001)
            .unwrap();
        assert_eq!(
            seventh,
            BudgetDecision::Escalate {
                already_escalated: true,
            }
        );
        assert!(budget.entry(42).unwrap().escalated);
    }

    #[test]
    fn register_rejects_pr_zero_and_epoch_zero() {
        let mut budget = Budget::new();
        assert_eq!(
            budget.register_attempt(0, FixLoopSource::CiFailure, 1),
            Err(BudgetError::InvalidPrNumber)
        );
        assert_eq!(
            budget.register_attempt(1, FixLoopSource::CiFailure, 0),
            Err(BudgetError::EpochZero)
        );
    }

    #[test]
    fn render_entries_json_array_round_trips_through_parser() {
        let entry = PrBudgetEntry {
            pr_number: 7,
            attempts_used: 3,
            ci_attempts: 2,
            review_attempts: 1,
            last_attempt_at_epoch: 1_715_000_000,
            escalated: false,
        };
        let budget = Budget::from_entries([entry.clone()]);
        let array_json = budget.render_entries_json_array();
        let envelope = format!("{{\"entries\":{array_json}}}");
        let parsed = parse_entries_block(&envelope).unwrap();
        assert_eq!(parsed, vec![entry]);
    }

    #[test]
    fn parser_handles_multiple_entries_with_whitespace() {
        let json = "{\n  \"entries\":[{\"attempts_used\":1,\"ci_attempts\":1,\"escalated\":false,\"last_attempt_at_epoch\":5,\"pr_number\":2,\"review_attempts\":0},{\"attempts_used\":5,\"ci_attempts\":3,\"escalated\":true,\"last_attempt_at_epoch\":9,\"pr_number\":3,\"review_attempts\":2}]}";
        let parsed = parse_entries_block(json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].pr_number, 2);
        assert_eq!(parsed[1].pr_number, 3);
        assert!(parsed[1].escalated);
    }
}
