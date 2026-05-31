//! ADR lifecycle validation — rules L1..L6.
//!
//! # Overview
//!
//! Each `AdrDecisionRecord` (parsed from frontmatter by the caller) is checked
//! against the lifecycle invariants below. Table-style ADRs that could not be
//! parsed from YAML frontmatter MUST be passed in with `status = ""` and
//! `AdrParseWarning::TableStyle` recorded by the caller — they are explicitly
//! skipped with a `Warn` rather than silently passing.
//!
//! # Rules
//!
//! | Rule | Severity | Description |
//! |------|----------|-------------|
//! | L1   | Error    | STATUS-VOCAB: status must be canonical or qualified canonical |
//! | L2   | Error    | TERMINAL-REQUIRES-LINK: Superseded/Rejected/Deprecated needs superseded_by |
//! | L3   | Error    | RECIPROCITY: supersedes/superseded_by must be bidirectional |
//! | L4   | Error    | DANGLING: every ADR-id in supersedes/superseded_by/related resolves |
//! | L5   | Warn     | HOLLOW-SUPERSEDED: Superseded ADR with < 80 body words |
//! | L6   | (stub)   | GOVERNS-ANCHOR: reserved for `governs:` frontmatter drift (not yet active) |
//!
//! # Table-style ADR handling
//!
//! Older ADRs (e.g. ADR-0146) express metadata via a markdown TABLE
//! (`| Status | Accepted |`) rather than YAML frontmatter. The
//! `AdrDecisionRecord` parser in this crate only reads YAML frontmatter;
//! table-style ADRs therefore arrive with `status = ""` and zero
//! supersedes/superseded_by/related vectors. Rather than silently passing
//! them (which would let real violations evade L1/L2), the lifecycle gate
//! accepts an explicit `AdrParseWarning::TableStyle` list and emits a `Warn`
//! violation for each so they are visible in gate output.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use crate::AdrDecisionRecord;

// ---------------------------------------------------------------------------
// Public surface
// ---------------------------------------------------------------------------

/// An ADR that could not be parsed from YAML frontmatter.
///
/// Currently only one variant is needed: table-style markdown ADRs. The
/// `adr_id` field should contain the id extracted from the filename
/// (e.g. `"ADR-0146"`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdrParseWarning {
    /// The ADR id (e.g. `"ADR-0146"`). data_class: INTERNAL_ONLY
    pub adr_id: String,
    /// Human-readable reason the ADR could not be fully parsed.
    /// data_class: INTERNAL_ONLY
    pub reason: String,
}

impl AdrParseWarning {
    /// Convenience constructor for a table-style ADR.
    #[must_use]
    pub fn table_style(adr_id: impl Into<String>) -> Self {
        Self {
            adr_id: adr_id.into(),
            reason: "ADR uses markdown-table metadata (not YAML frontmatter); \
                     L1/L2 checks cannot be applied — normalise to YAML frontmatter \
                     to bring under full lifecycle governance"
                .into(),
        }
    }
}

/// Lifecycle-rule identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LifecycleRule {
    /// L1 — status value must be canonical vocabulary.
    L1StatusVocab,
    /// L2 — terminal statuses must carry a superseded_by link.
    L2TerminalRequiresLink,
    /// L3 — supersedes/superseded_by must be bidirectional.
    L3Reciprocity,
    /// L4 — every ADR-id reference must resolve in the corpus.
    L4Dangling,
    /// L5 — Superseded ADR with very few body words (archive candidate).
    L5HollowSuperseded,
    /// L6 — governs-anchor drift (stub; not yet active).
    L6GovernsAnchor,
}

impl LifecycleRule {
    /// Human-readable short name for display.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::L1StatusVocab => "L1-status-vocab",
            Self::L2TerminalRequiresLink => "L2-terminal-requires-link",
            Self::L3Reciprocity => "L3-reciprocity",
            Self::L4Dangling => "L4-dangling",
            Self::L5HollowSuperseded => "L5-hollow-superseded",
            Self::L6GovernsAnchor => "L6-governs-anchor",
        }
    }
}

/// Violation severity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Severity {
    /// Gate-blocking; must be resolved before merge.
    Error,
    /// Advisory; reported but does not block the gate.
    Warn,
}

impl Severity {
    /// Human-readable label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
        }
    }
}

/// A single lifecycle violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleViolation {
    /// The ADR this violation applies to (e.g. `"ADR-0123"`).
    /// data_class: INTERNAL_ONLY
    pub adr_id: String,
    /// Which rule was violated.
    pub rule: LifecycleRule,
    /// Severity of the violation.
    pub severity: Severity,
    /// Human-readable description.
    /// data_class: INTERNAL_ONLY
    pub detail: String,
    /// Optional machine-actionable fix hint.
    /// data_class: INTERNAL_ONLY
    pub suggested_fix: Option<String>,
}

impl LifecycleViolation {
    fn error(
        adr_id: impl Into<String>,
        rule: LifecycleRule,
        detail: impl Into<String>,
        suggested_fix: Option<String>,
    ) -> Self {
        Self {
            adr_id: adr_id.into(),
            rule,
            severity: Severity::Error,
            detail: detail.into(),
            suggested_fix,
        }
    }

    fn warn(
        adr_id: impl Into<String>,
        rule: LifecycleRule,
        detail: impl Into<String>,
        suggested_fix: Option<String>,
    ) -> Self {
        Self {
            adr_id: adr_id.into(),
            rule,
            severity: Severity::Warn,
            detail: detail.into(),
            suggested_fix,
        }
    }
}

/// Aggregated counts by rule and severity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LifecycleSummary {
    /// Errors per rule (rule → count).
    pub errors_by_rule: BTreeMap<String, usize>,
    /// Warnings per rule (rule → count).
    pub warnings_by_rule: BTreeMap<String, usize>,
    /// Total error count.
    pub total_errors: usize,
    /// Total warning count.
    pub total_warnings: usize,
}

impl LifecycleSummary {
    fn from_violations(violations: &[LifecycleViolation]) -> Self {
        let mut summary = Self::default();
        for v in violations {
            let key = v.rule.as_str().to_string();
            match v.severity {
                Severity::Error => {
                    *summary.errors_by_rule.entry(key).or_insert(0) += 1;
                    summary.total_errors += 1;
                }
                Severity::Warn => {
                    *summary.warnings_by_rule.entry(key).or_insert(0) += 1;
                    summary.total_warnings += 1;
                }
            }
        }
        summary
    }
}

/// Result of running the lifecycle gate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleResult {
    /// All violations found (errors + warnings), sorted by (adr_id, rule).
    pub violations: Vec<LifecycleViolation>,
    /// Aggregated summary counts.
    pub summary: LifecycleSummary,
    /// ADRs that could not be fully parsed (e.g. table-style).
    pub parse_warnings: Vec<AdrParseWarning>,
}

impl LifecycleResult {
    /// Returns `true` when there are zero blocking (Error-severity) violations.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.summary.total_errors == 0
    }
}

// ---------------------------------------------------------------------------
// L1 — canonical status vocabulary
// ---------------------------------------------------------------------------

/// The five canonical bare status values (exact casing required).
const CANONICAL_STATUSES: &[&str] =
    &["Proposed", "Accepted", "Superseded", "Rejected", "Deprecated"];

/// Canonical base values that may appear in a qualified form
/// `<CanonicalBase> (qualifier)`.
const QUALIFIED_BASES: &[&str] =
    &["Proposed", "Accepted", "Superseded", "Rejected", "Deprecated"];

/// Returns `true` when `status` is a valid canonical or qualified status.
///
/// Valid forms:
/// - Exact canonical: `Accepted`, `Superseded`, …
/// - Qualified:       `Accepted (amendment)`, `Superseded (partial)`, …
///
/// Leading/trailing whitespace is NOT accepted (the raw frontmatter value is
/// used; L1 flags trailing whitespace explicitly).
fn is_valid_status(status: &str) -> bool {
    // Exact match.
    if CANONICAL_STATUSES.contains(&status) {
        return true;
    }
    // Qualified: `Base (qualifier)` — base must be canonical, qualifier
    // must be non-empty and not contain a `(` itself.
    if let Some(paren_start) = status.find(" (") {
        let base = &status[..paren_start];
        let rest = &status[paren_start + 2..];
        if QUALIFIED_BASES.contains(&base) && rest.ends_with(')') && rest.len() > 1 {
            let qualifier = &rest[..rest.len() - 1];
            if !qualifier.is_empty() && !qualifier.contains('(') {
                return true;
            }
        }
    }
    false
}

/// Suggest the canonical fix for a non-canonical status.
fn suggest_status_fix(status: &str) -> Option<String> {
    let lower = status.trim().to_ascii_lowercase();
    for canonical in CANONICAL_STATUSES {
        if canonical.to_ascii_lowercase() == lower {
            return Some(format!("Change status to `{canonical}`"));
        }
    }
    None
}

fn check_l1(record: &AdrDecisionRecord, violations: &mut Vec<LifecycleViolation>) {
    let status = &record.status;
    // Trailing whitespace.
    if status != status.trim() {
        violations.push(LifecycleViolation::error(
            &record.id,
            LifecycleRule::L1StatusVocab,
            format!(
                "status {:?} has leading or trailing whitespace",
                status
            ),
            Some(format!(
                "Change status to `{}`",
                status.trim()
            )),
        ));
        return;
    }
    if !is_valid_status(status) {
        violations.push(LifecycleViolation::error(
            &record.id,
            LifecycleRule::L1StatusVocab,
            format!(
                "status {:?} is not a canonical value; allowed: Proposed | Accepted | Superseded | Rejected | Deprecated | <CanonicalBase> (qualifier)",
                status
            ),
            suggest_status_fix(status),
        ));
    }
}

// ---------------------------------------------------------------------------
// L2 — terminal status requires superseded_by
// ---------------------------------------------------------------------------

const TERMINAL_STATUSES: &[&str] = &["Superseded", "Rejected", "Deprecated"];

fn is_terminal_base(status: &str) -> bool {
    // Handles both bare `Superseded` and qualified `Superseded (partial)`.
    TERMINAL_STATUSES.iter().any(|t| {
        status == *t || status.starts_with(&format!("{t} ("))
    })
}

fn check_l2(record: &AdrDecisionRecord, violations: &mut Vec<LifecycleViolation>) {
    if is_terminal_base(&record.status) && record.superseded_by.is_empty() {
        violations.push(LifecycleViolation::error(
            &record.id,
            LifecycleRule::L2TerminalRequiresLink,
            format!(
                "status `{}` is terminal but superseded_by is empty",
                record.status
            ),
            Some("Add `superseded_by: [ADR-NNNN]` pointing to the superseding ADR".into()),
        ));
    }
}

// ---------------------------------------------------------------------------
// L3 — reciprocity
// ---------------------------------------------------------------------------

/// Extract an `ADR-NNNN` id from a raw list entry, or `None` for non-ADR targets.
/// Path-shaped entries (containing `/`) are treated as non-ADR targets.
fn adr_id_of(entry: &str) -> Option<String> {
    let entry = entry.trim();
    if entry.contains('/') {
        return None;
    }
    let start = entry.find("ADR-")?;
    let digits_start = start + 4;
    let digits: String = entry[digits_start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.len() != 4 {
        return None;
    }
    Some(format!("ADR-{digits}"))
}

fn resolve_adr_ids(entries: &[String]) -> BTreeSet<String> {
    entries.iter().filter_map(|e| adr_id_of(e)).collect()
}

fn check_l3(
    records: &BTreeMap<String, &AdrDecisionRecord>,
    violations: &mut Vec<LifecycleViolation>,
) {
    // FORWARD: A.supersedes contains B => B.superseded_by contains A.
    for (id, record) in records {
        for raw in &record.supersedes {
            let Some(target_id) = adr_id_of(raw) else {
                continue;
            };
            let Some(target) = records.get(&target_id) else {
                continue; // Dangling — handled by L4.
            };
            let target_sb: BTreeSet<String> = resolve_adr_ids(&target.superseded_by);
            if !target_sb.contains(id) {
                violations.push(LifecycleViolation::error(
                    id,
                    LifecycleRule::L3Reciprocity,
                    format!(
                        "{id} supersedes {target_id}, but {target_id}.superseded_by does not contain {id}"
                    ),
                    Some(format!(
                        "Add `{id}` to {target_id}'s `superseded_by:` list"
                    )),
                ));
            }
        }
    }
    // REVERSE: B.superseded_by contains A => A.supersedes contains B.
    for (id, record) in records {
        for raw in &record.superseded_by {
            let Some(source_id) = adr_id_of(raw) else {
                continue;
            };
            let Some(source) = records.get(&source_id) else {
                continue; // Dangling — handled by L4.
            };
            let source_sup: BTreeSet<String> = resolve_adr_ids(&source.supersedes);
            if !source_sup.contains(id) {
                violations.push(LifecycleViolation::error(
                    id,
                    LifecycleRule::L3Reciprocity,
                    format!(
                        "{id} is superseded_by {source_id}, but {source_id}.supersedes does not contain {id}"
                    ),
                    Some(format!(
                        "Add `{id}` to {source_id}'s `supersedes:` list"
                    )),
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// L4 — dangling ADR-id references
// ---------------------------------------------------------------------------

/// Returns `true` when the raw entry looks like an ADR id reference (not a
/// path or free-form text), i.e. it contains `ADR-NNNN` with exactly four
/// digits.
fn is_adr_id_ref(entry: &str) -> bool {
    adr_id_of(entry).is_some()
}

fn check_l4(
    record: &AdrDecisionRecord,
    corpus_ids: &BTreeSet<String>,
    violations: &mut Vec<LifecycleViolation>,
) {
    let all_refs = record
        .supersedes
        .iter()
        .chain(record.superseded_by.iter())
        .chain(record.related.iter());

    for raw in all_refs {
        if !is_adr_id_ref(raw) {
            // Non-ADR path reference — exempt per design.
            continue;
        }
        let Some(ref_id) = adr_id_of(raw) else {
            continue;
        };
        if !corpus_ids.contains(&ref_id) {
            violations.push(LifecycleViolation::error(
                &record.id,
                LifecycleRule::L4Dangling,
                format!(
                    "{} references {ref_id} (from {:?}) but {ref_id} is not in the ADR corpus",
                    record.id, raw
                ),
                Some(format!(
                    "Either add ADR-file for {ref_id} or remove the reference from {}'s frontmatter",
                    record.id
                )),
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// L5 — hollow superseded (ArchiveCandidate)
// ---------------------------------------------------------------------------

/// Minimum body word count for a Superseded ADR to pass L5.
///
/// Calibration: the smallest substantive live superseded ADR in the corpus
/// (ADR-0107) has 144 words after stripping frontmatter and headings; the
/// smallest hollow stub that should be flagged would have fewer than ~80.
/// Setting the threshold at 80 ensures every current real ADR passes while
/// genuinely empty stubs are flagged.
pub const L5_WORD_COUNT_THRESHOLD: usize = 80;

/// Count content words in `body`: skip blank lines, heading lines (`# …`),
/// and lines that are part of a markdown table (`| … |`).
fn body_word_count(body: &str) -> usize {
    body.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with('|')
                && !trimmed.starts_with("---")
        })
        .map(|line| line.split_whitespace().count())
        .sum()
}

fn check_l5(
    record: &AdrDecisionRecord,
    body: &str,
    violations: &mut Vec<LifecycleViolation>,
) {
    if !is_terminal_base(&record.status) || !record.status.starts_with("Superseded") {
        return;
    }
    let word_count = body_word_count(body);
    if word_count < L5_WORD_COUNT_THRESHOLD {
        violations.push(LifecycleViolation::warn(
            &record.id,
            LifecycleRule::L5HollowSuperseded,
            format!(
                "status=Superseded but body has only {word_count} words (threshold: {L5_WORD_COUNT_THRESHOLD}); this is an ArchiveCandidate"
            ),
            Some(format!(
                "Either expand the body to document the decision context (>= {L5_WORD_COUNT_THRESHOLD} words) or archive to docs/decisions/archive/"
            )),
        ));
    }
}

// ---------------------------------------------------------------------------
// L6 — governs-anchor (stub — not yet active)
// ---------------------------------------------------------------------------
//
// L6: governs-anchor drift — implemented when the `governs:` frontmatter
// field is populated; see docs/ideas/adr-ssot-masterplan-drift.md
//
// No ADR currently carries a `governs:` frontmatter field, so there is
// nothing to validate. The extension point is documented here so future
// implementors can add the check without touching the public API:
//
//   fn check_l6(record: &AdrDecisionRecord, violations: &mut Vec<LifecycleViolation>) {
//       for governed in &record.governs {
//           // validate that the governed path / id exists in the corpus
//       }
//   }
//
// `governs: Vec<String>` can be added to `AdrDecisionRecord` (zero-cost
// for existing call sites that use struct-literal construction) once ADRs
// start populating the field.

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Validate the lifecycle invariants for a corpus of ADRs.
///
/// # Parameters
///
/// * `records` — the parsed ADR records (YAML-frontmatter ADRs only).
/// * `bodies` — map from ADR id to the full file body text (including
///   frontmatter). Used for L5 word-count. Pass an empty map to skip L5.
/// * `parse_warnings` — ADRs that could not be parsed from YAML frontmatter
///   (e.g. table-style). These generate `Warn` violations rather than being
///   silently ignored.
///
/// # Returns
///
/// A `LifecycleResult` containing all violations and the aggregated summary.
/// The result is deterministic: violations are sorted by `(adr_id, rule)`.
pub fn validate_lifecycle<'a, R>(
    records: R,
    bodies: &BTreeMap<String, &str>,
    parse_warnings: &[AdrParseWarning],
) -> LifecycleResult
where
    R: IntoIterator<Item = &'a AdrDecisionRecord>,
{
    let records_vec: Vec<&AdrDecisionRecord> = records.into_iter().collect();

    // Build lookup map and corpus id set.
    let record_map: BTreeMap<String, &AdrDecisionRecord> = records_vec
        .iter()
        .map(|r| (r.id.clone(), *r))
        .collect();
    let corpus_ids: BTreeSet<String> = record_map.keys().cloned().collect();

    let mut violations: Vec<LifecycleViolation> = Vec::new();

    // Emit Warn for table-style (un-parseable) ADRs.
    for pw in parse_warnings {
        violations.push(LifecycleViolation::warn(
            &pw.adr_id,
            LifecycleRule::L1StatusVocab,
            format!("ADR {} could not be lifecycle-checked: {}", pw.adr_id, pw.reason),
            Some(
                "Convert the metadata table to YAML frontmatter (--- ... ---) to enable full lifecycle governance".into(),
            ),
        ));
    }

    // Per-record checks (L1, L2, L4, L5).
    for record in &records_vec {
        check_l1(record, &mut violations);
        check_l2(record, &mut violations);
        check_l4(record, &corpus_ids, &mut violations);
        if let Some(body) = bodies.get(&record.id) {
            check_l5(record, body, &mut violations);
        }
    }

    // Cross-record checks (L3).
    check_l3(&record_map, &mut violations);

    // Sort for determinism: (adr_id, rule, detail).
    violations.sort_by(|a, b| {
        a.adr_id
            .cmp(&b.adr_id)
            .then_with(|| a.rule.cmp(&b.rule))
            .then_with(|| a.detail.cmp(&b.detail))
    });
    violations.dedup_by(|a, b| a.adr_id == b.adr_id && a.rule == b.rule && a.detail == b.detail);

    let summary = LifecycleSummary::from_violations(&violations);

    LifecycleResult {
        violations,
        summary,
        parse_warnings: parse_warnings.to_vec(),
    }
}

/// Convenience: render a one-line human summary of the result.
#[must_use]
pub fn summary(result: &LifecycleResult) -> String {
    format!(
        "errors={} warnings={} parse_warnings={}",
        result.summary.total_errors,
        result.summary.total_warnings,
        result.parse_warnings.len(),
    )
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AdrDecisionRecord;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn record(id: &str, status: &str) -> AdrDecisionRecord {
        let number: u16 = id[4..].parse().unwrap_or(1);
        AdrDecisionRecord {
            number,
            id: id.into(),
            title: format!("Decision {id}"),
            status: status.into(),
            owner: "council-architecture".into(),
            date: "2026-01-01".into(),
            path: format!("decisions/{id}-decision.md"),
            supersedes: vec![],
            superseded_by: vec![],
            related: vec![],
        }
    }

    fn with_supersedes(mut r: AdrDecisionRecord, targets: &[&str]) -> AdrDecisionRecord {
        r.supersedes = targets.iter().map(|s| s.to_string()).collect();
        r
    }

    fn with_superseded_by(mut r: AdrDecisionRecord, sources: &[&str]) -> AdrDecisionRecord {
        r.superseded_by = sources.iter().map(|s| s.to_string()).collect();
        r
    }

    fn with_related(mut r: AdrDecisionRecord, refs: &[&str]) -> AdrDecisionRecord {
        r.related = refs.iter().map(|s| s.to_string()).collect();
        r
    }

    fn run(records: &[AdrDecisionRecord]) -> LifecycleResult {
        validate_lifecycle(records.iter(), &BTreeMap::new(), &[])
    }

    fn run_with_bodies<'a>(
        records: &'a [AdrDecisionRecord],
        bodies: &BTreeMap<String, &'a str>,
    ) -> LifecycleResult {
        validate_lifecycle(records.iter(), bodies, &[])
    }

    fn errors(result: &LifecycleResult) -> Vec<&LifecycleViolation> {
        result
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .collect()
    }

    fn warnings(result: &LifecycleResult) -> Vec<&LifecycleViolation> {
        result
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Warn)
            .collect()
    }

    // -----------------------------------------------------------------------
    // L1 — STATUS-VOCAB
    // -----------------------------------------------------------------------

    #[test]
    fn l1_accepts_all_canonical_bare_statuses() {
        for status in &["Proposed", "Accepted", "Superseded", "Rejected", "Deprecated"] {
            let mut r = record("ADR-0001", status);
            if ["Superseded", "Rejected", "Deprecated"].contains(status) {
                r.superseded_by = vec!["ADR-0002".into()];
            }
            let r2 = record("ADR-0002", "Accepted");
            let result = run(&[r, r2]);
            let l1_errs: Vec<_> = errors(&result)
                .into_iter()
                .filter(|v| v.rule == LifecycleRule::L1StatusVocab)
                .collect();
            assert!(
                l1_errs.is_empty(),
                "canonical status `{status}` should pass L1, got {l1_errs:?}"
            );
        }
    }

    #[test]
    fn l1_accepts_qualified_canonical_status() {
        let r = record("ADR-0001", "Accepted (amendment)");
        let result = run(&[r]);
        assert!(
            errors(&result)
                .iter()
                .all(|v| v.rule != LifecycleRule::L1StatusVocab),
            "qualified canonical status should pass L1"
        );
    }

    #[test]
    fn l1_rejects_lowercase_status() {
        let r = record("ADR-0001", "accepted");
        let result = run(&[r]);
        let l1_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L1StatusVocab)
            .collect();
        assert_eq!(l1_errs.len(), 1);
        assert!(l1_errs[0].suggested_fix.as_deref().unwrap_or("").contains("Accepted"));
    }

    #[test]
    fn l1_rejects_junk_statuses() {
        for status in &["OK", "completed", "Substantially", "planned", "pending", "Draft", "Amended"] {
            let r = record("ADR-0001", status);
            let result = run(&[r]);
            let l1_errs: Vec<_> = errors(&result)
                .into_iter()
                .filter(|v| v.rule == LifecycleRule::L1StatusVocab)
                .collect();
            assert!(
                !l1_errs.is_empty(),
                "junk status `{status}` should fail L1"
            );
        }
    }

    #[test]
    fn l1_rejects_trailing_whitespace_status() {
        let r = record("ADR-0001", "Accepted ");
        let result = run(&[r]);
        let l1_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L1StatusVocab)
            .collect();
        assert_eq!(l1_errs.len(), 1, "trailing whitespace should fail L1");
    }

    #[test]
    fn l1_rejects_qualified_with_wrong_base() {
        let r = record("ADR-0001", "Draft (partial)");
        let result = run(&[r]);
        let l1_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L1StatusVocab)
            .collect();
        assert!(!l1_errs.is_empty(), "non-canonical qualified base should fail L1");
    }

    // -----------------------------------------------------------------------
    // L2 — TERMINAL-REQUIRES-LINK
    // -----------------------------------------------------------------------

    #[test]
    fn l2_accepts_terminal_with_superseded_by() {
        let r = with_superseded_by(record("ADR-0001", "Superseded"), &["ADR-0002"]);
        let r2 = with_supersedes(record("ADR-0002", "Accepted"), &["ADR-0001"]);
        let result = run(&[r, r2]);
        let l2_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L2TerminalRequiresLink)
            .collect();
        assert!(l2_errs.is_empty(), "terminal with link should pass L2");
    }

    #[test]
    fn l2_rejects_superseded_without_link() {
        let r = record("ADR-0001", "Superseded");
        let result = run(&[r]);
        let l2_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L2TerminalRequiresLink)
            .collect();
        assert_eq!(l2_errs.len(), 1);
    }

    #[test]
    fn l2_rejects_rejected_without_link() {
        let r = record("ADR-0001", "Rejected");
        let result = run(&[r]);
        let l2_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L2TerminalRequiresLink)
            .collect();
        assert_eq!(l2_errs.len(), 1);
    }

    #[test]
    fn l2_rejects_deprecated_without_link() {
        let r = record("ADR-0001", "Deprecated");
        let result = run(&[r]);
        let l2_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L2TerminalRequiresLink)
            .collect();
        assert_eq!(l2_errs.len(), 1);
    }

    // -----------------------------------------------------------------------
    // L3 — RECIPROCITY
    // -----------------------------------------------------------------------

    #[test]
    fn l3_accepts_bidirectional_pair() {
        let a = with_supersedes(record("ADR-0001", "Accepted"), &["ADR-0002"]);
        let b = with_superseded_by(record("ADR-0002", "Superseded"), &["ADR-0001"]);
        // Also give b a superseded_by link so L2 passes.
        let result = run(&[a, b]);
        let l3_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L3Reciprocity)
            .collect();
        assert!(l3_errs.is_empty(), "bidirectional pair must pass L3");
    }

    #[test]
    fn l3_rejects_forward_without_backlink() {
        // A supersedes B, but B.superseded_by is empty.
        let a = with_supersedes(record("ADR-0001", "Accepted"), &["ADR-0002"]);
        let b = record("ADR-0002", "Accepted");
        let result = run(&[a, b]);
        let l3_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L3Reciprocity)
            .collect();
        assert!(!l3_errs.is_empty(), "forward without backlink must fail L3");
    }

    #[test]
    fn l3_rejects_backlink_without_forward() {
        // B says superseded_by A, but A.supersedes is empty.
        let a = record("ADR-0001", "Accepted");
        let b = with_superseded_by(record("ADR-0002", "Superseded"), &["ADR-0001"]);
        let result = run(&[a, b]);
        let l3_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L3Reciprocity)
            .collect();
        assert!(!l3_errs.is_empty(), "backlink without forward must fail L3");
    }

    // -----------------------------------------------------------------------
    // L4 — DANGLING
    // -----------------------------------------------------------------------

    #[test]
    fn l4_accepts_resolvable_ref() {
        let a = with_supersedes(record("ADR-0001", "Accepted"), &["ADR-0002"]);
        let b = with_superseded_by(record("ADR-0002", "Superseded"), &["ADR-0001"]);
        let result = run(&[a, b]);
        let l4_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L4Dangling)
            .collect();
        assert!(l4_errs.is_empty());
    }

    #[test]
    fn l4_rejects_dangling_adr_id() {
        // ADR-0001 supersedes ADR-9999 which does not exist in corpus.
        let a = with_supersedes(record("ADR-0001", "Accepted"), &["ADR-9999"]);
        let result = run(&[a]);
        let l4_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L4Dangling)
            .collect();
        assert!(!l4_errs.is_empty(), "dangling ADR-id should fail L4");
    }

    #[test]
    fn l4_exempts_non_adr_path_references() {
        // Path-shaped references like `microservices/cell/PRD.md` are exempt.
        let mut r = record("ADR-0001", "Accepted");
        r.supersedes = vec!["microservices/cell/PRD.md".into()];
        let result = run(&[r]);
        let l4_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L4Dangling)
            .collect();
        assert!(l4_errs.is_empty(), "non-ADR path reference must be exempt from L4");
    }

    #[test]
    fn l4_checks_related_field_for_dangling_adr_ids() {
        let a = with_related(record("ADR-0001", "Accepted"), &["ADR-9998"]);
        let result = run(&[a]);
        let l4_errs: Vec<_> = errors(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L4Dangling)
            .collect();
        assert!(!l4_errs.is_empty(), "dangling ADR-id in related should fail L4");
    }

    // -----------------------------------------------------------------------
    // L5 — HOLLOW-SUPERSEDED
    // -----------------------------------------------------------------------

    #[test]
    fn l5_passes_superseded_with_sufficient_body() {
        let r = with_superseded_by(record("ADR-0001", "Superseded"), &["ADR-0002"]);
        let r2 = with_supersedes(record("ADR-0002", "Accepted"), &["ADR-0001"]);
        let long_body = "word ".repeat(100);
        let mut bodies = BTreeMap::new();
        bodies.insert("ADR-0001".to_string(), long_body.as_str());
        let result = run_with_bodies(&[r, r2], &bodies);
        let l5_warns: Vec<_> = warnings(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L5HollowSuperseded)
            .collect();
        assert!(l5_warns.is_empty(), "sufficient body should pass L5");
    }

    #[test]
    fn l5_warns_superseded_with_hollow_body() {
        let r = with_superseded_by(record("ADR-0001", "Superseded"), &["ADR-0002"]);
        let r2 = with_supersedes(record("ADR-0002", "Accepted"), &["ADR-0001"]);
        let short_body = "word ".repeat(10); // well under 80
        let mut bodies = BTreeMap::new();
        bodies.insert("ADR-0001".to_string(), short_body.as_str());
        let result = run_with_bodies(&[r, r2], &bodies);
        let l5_warns: Vec<_> = warnings(&result)
            .into_iter()
            .filter(|v| v.rule == LifecycleRule::L5HollowSuperseded)
            .collect();
        assert!(!l5_warns.is_empty(), "hollow Superseded ADR should emit L5 warn");
        assert_eq!(l5_warns[0].severity, Severity::Warn);
    }

    #[test]
    fn l5_does_not_warn_for_accepted_status() {
        let r = record("ADR-0001", "Accepted");
        let short_body = "word ".repeat(5);
        let mut bodies = BTreeMap::new();
        bodies.insert("ADR-0001".to_string(), short_body.as_str());
        let result = run_with_bodies(&[r], &bodies);
        let l5: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.rule == LifecycleRule::L5HollowSuperseded)
            .collect();
        assert!(l5.is_empty(), "Accepted ADRs should not trigger L5");
    }

    #[test]
    fn l5_threshold_constant_is_80() {
        assert_eq!(L5_WORD_COUNT_THRESHOLD, 80);
    }

    // -----------------------------------------------------------------------
    // Table-style ADR parse-warning
    // -----------------------------------------------------------------------

    #[test]
    fn table_style_adr_emits_warn_not_silently_skipped() {
        let pw = AdrParseWarning::table_style("ADR-0146");
        let result = validate_lifecycle(
            std::iter::empty::<&AdrDecisionRecord>(),
            &BTreeMap::new(),
            &[pw],
        );
        let table_warns: Vec<_> = warnings(&result)
            .into_iter()
            .filter(|v| v.adr_id == "ADR-0146")
            .collect();
        assert!(
            !table_warns.is_empty(),
            "table-style ADR must generate a warning, not be silently skipped"
        );
        assert_eq!(table_warns[0].severity, Severity::Warn);
    }

    #[test]
    fn table_style_adr_warn_is_not_an_error() {
        let pw = AdrParseWarning::table_style("ADR-0146");
        let result = validate_lifecycle(
            std::iter::empty::<&AdrDecisionRecord>(),
            &BTreeMap::new(),
            &[pw],
        );
        assert!(
            result.is_clean(),
            "table-style warn must not block the gate (is_clean should be true)"
        );
        assert_eq!(result.summary.total_errors, 0);
        assert_eq!(result.summary.total_warnings, 1);
    }

    // -----------------------------------------------------------------------
    // Summary
    // -----------------------------------------------------------------------

    #[test]
    fn summary_counts_correctly() {
        // Two errors (one L1, one L2) + one L5 warn.
        let r1 = record("ADR-0001", "bad-status"); // L1 error
        let r2 = record("ADR-0002", "Superseded"); // L2 error (no superseded_by)
        let r3 = with_superseded_by(record("ADR-0003", "Superseded"), &["ADR-9999"]); // L4 error (dangling)
        let result = run(&[r1, r2, r3]);
        // At least 3 errors.
        assert!(result.summary.total_errors >= 3);
        assert!(!result.is_clean());
    }

    // -----------------------------------------------------------------------
    // is_valid_status edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn is_valid_status_rejects_empty_qualifier() {
        assert!(!is_valid_status("Accepted ()"));
        assert!(!is_valid_status("Accepted ( )"));
    }

    #[test]
    fn is_valid_status_rejects_nested_parens() {
        assert!(!is_valid_status("Accepted (foo (bar))"));
    }

    #[test]
    fn is_valid_status_accepts_multi_word_qualifier() {
        assert!(is_valid_status("Accepted (partial amendment)"));
    }
}
