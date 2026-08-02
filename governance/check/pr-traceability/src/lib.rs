//! Governance PR traceability fitness kernel.
//!
//! Oyatie's PR contract requires five traceability H2 sections: Issue, Summary,
//! Verification, Traceability, and Evidence. `## Code Review` is supplied by
//! the automated reviewer-agent pipeline for merge-ready bodies, so this pure
//! kernel can validate author-only bodies and merge-ready bodies by policy.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrTraceabilityDocument {
    pub document_id: String, // data_class: INTERNAL_ONLY
    pub title: String,       // data_class: INTERNAL_ONLY
    pub body: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrTraceabilityPolicy {
    pub require_code_review: bool, // data_class: INTERNAL_ONLY
    pub forbid_code_review: bool,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrTraceabilityReport {
    pub required_sections_checked: usize, // data_class: INTERNAL_ONLY
    pub code_review_present: bool,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrTraceabilityError {
    ConflictingCodeReviewPolicy,
    MissingSection {
        section: &'static str,
    },
    SectionOutOfOrder {
        section: &'static str,
        previous_section: &'static str,
    },
    EmptySection {
        section: &'static str,
    },
    MissingIssueReference,
    MissingSummaryBullet,
    MissingTraceabilityField {
        field: &'static str,
    },
    MissingEvidenceField {
        field: &'static str,
    },
    CodeReviewRequired,
    CodeReviewForbidden,
    BlockedReviewMarker {
        location: &'static str,
        marker: &'static str,
    },
    MissingCodeReviewReviewer,
    MissingCodeReviewApproval,
    MissingCodeReviewResolvedItems,
    MissingCodeReviewDeferredItems,
    CodeReviewRequestsChanges,
}

const REQUIRED_SECTIONS: [&str; 5] = [
    "Issue",
    "Summary",
    "Verification",
    "Traceability",
    "Evidence",
];

// Single source for the exact substrings the validator requires (and the scaffolder emits) so
// the two can never drift apart — this is the fix for the recurring MissingTraceabilityField /
// MissingEvidenceField / MissingIssueReference class of CI-preflight surprises.
const REQUIRED_TRACEABILITY_FIELDS: [&str; 3] = [
    "Catalog records touched",
    "Cross-axis contracts touched",
    "ADRs cited",
];
const REQUIRED_EVIDENCE_FIELDS: [&str; 3] = [
    "Audit-chain emission",
    "Foundation-bypass referenced",
    "Per-pack regulator-watch impact",
];
const ISSUE_REFERENCE_MARKERS: [&str; 3] = ["Closes #", "Refs #", "Blocks #"];

pub fn validate_pr_traceability(
    document: &PrTraceabilityDocument,
    policy: PrTraceabilityPolicy,
) -> Result<PrTraceabilityReport, PrTraceabilityError> {
    if policy.require_code_review && policy.forbid_code_review {
        return Err(PrTraceabilityError::ConflictingCodeReviewPolicy);
    }

    if let Some(marker) = blocked_review_marker_in_title(&document.title) {
        return Err(PrTraceabilityError::BlockedReviewMarker {
            location: "title",
            marker,
        });
    }
    if let Some(marker) = blocked_review_marker_in_body(&document.body) {
        return Err(PrTraceabilityError::BlockedReviewMarker {
            location: "body",
            marker,
        });
    }

    let sections = h2_sections(&document.body);
    let mut previous_index = None;
    let mut previous_section = None;
    for required in REQUIRED_SECTIONS {
        let Some(index) = section_index(&sections, required) else {
            return Err(PrTraceabilityError::MissingSection { section: required });
        };
        if let (Some(previous_index), Some(previous_section_id)) =
            (previous_index, previous_section)
            && index <= previous_index
        {
            return Err(PrTraceabilityError::SectionOutOfOrder {
                section: required,
                previous_section: previous_section_id,
            });
        }
        if section_body(&document.body, &sections, required)
            .trim()
            .is_empty()
        {
            return Err(PrTraceabilityError::EmptySection { section: required });
        }
        previous_index = Some(index);
        previous_section = Some(required);
    }

    let issue = section_body(&document.body, &sections, "Issue");
    if !contains_issue_reference(issue) {
        return Err(PrTraceabilityError::MissingIssueReference);
    }

    let summary = section_body(&document.body, &sections, "Summary");
    if !summary
        .lines()
        .any(|line| line.trim_start().starts_with("- "))
    {
        return Err(PrTraceabilityError::MissingSummaryBullet);
    }

    let traceability = section_body(&document.body, &sections, "Traceability");
    for field in REQUIRED_TRACEABILITY_FIELDS {
        if !traceability.contains(field) {
            return Err(PrTraceabilityError::MissingTraceabilityField { field });
        }
    }

    let evidence = section_body(&document.body, &sections, "Evidence");
    for field in REQUIRED_EVIDENCE_FIELDS {
        if !evidence.contains(field) {
            return Err(PrTraceabilityError::MissingEvidenceField { field });
        }
    }

    let code_review = section_body(&document.body, &sections, "Code Review");
    let code_review_present = section_index(&sections, "Code Review").is_some();
    if policy.require_code_review && !code_review_present {
        return Err(PrTraceabilityError::CodeReviewRequired);
    }
    if policy.require_code_review {
        validate_code_review_section(code_review)?;
    }
    if policy.forbid_code_review && code_review_present {
        return Err(PrTraceabilityError::CodeReviewForbidden);
    }

    Ok(PrTraceabilityReport {
        required_sections_checked: REQUIRED_SECTIONS.len(),
        code_review_present,
    })
}

/// Emits a PR-body template that passes every author-owned check in
/// [`validate_pr_traceability`] — every required section in order, every literal
/// Traceability/Evidence field label, and a valid issue-reference marker — built from the
/// SAME constants the validator checks against, so the two can never drift apart.
///
/// The `## Code Review` section is intentionally left at `Verdict: pending`: that is real
/// (no review has happened yet), so `validate_pr_traceability` with `require_code_review: true`
/// correctly still fails on `MissingCodeReviewApproval` — that is the one violation only a
/// reviewer, not the author, can close.
pub fn scaffold_pr_body() -> String {
    let mut body = String::new();
    for section in REQUIRED_SECTIONS {
        body.push_str("## ");
        body.push_str(section);
        body.push('\n');
        match section {
            "Issue" => {
                body.push_str(ISSUE_REFERENCE_MARKERS[1]);
                body.push_str("<n>  <!-- one-line issue ref; use \"Closes #<n>\" if this PR closes the issue -->\n");
            }
            "Summary" => body.push_str("- <one-line summary of what changed and why>\n"),
            "Verification" => {
                body.push_str("- <pass|fail>: `<buck2 test/build command run>` (paste excerpt)\n");
            }
            "Traceability" => {
                for field in REQUIRED_TRACEABILITY_FIELDS {
                    body.push_str("- ");
                    body.push_str(field);
                    body.push_str(": `<list>`\n");
                }
            }
            "Evidence" => {
                for field in REQUIRED_EVIDENCE_FIELDS {
                    body.push_str("- ");
                    body.push_str(field);
                    body.push_str(" (if any): `<event-id-or-none>`\n");
                }
            }
            other => unreachable!("scaffold_pr_body: unhandled required section {other}"),
        }
        body.push('\n');
    }
    body.push_str("## Code Review\n");
    body.push_str("- Reviewer agent: `<reviewer-agent>`\n");
    body.push_str("- Verdict: pending\n");
    body.push_str("- Resolved items: `<items-or-none>`\n");
    body.push_str("- Deferred items: `<items-or-none>`\n");
    body
}

/// Same rules as [`validate_pr_traceability`], but collects every violation instead of
/// stopping at the first — so `--all-violations` can show an author every defect in one pass
/// instead of one CI round-trip per fix. Field/content checks are skipped for a section that is
/// itself missing (nothing meaningful to say about content of a section that doesn't exist).
pub fn validate_pr_traceability_all(
    document: &PrTraceabilityDocument,
    policy: PrTraceabilityPolicy,
) -> Vec<PrTraceabilityError> {
    let mut errors = Vec::new();

    if policy.require_code_review && policy.forbid_code_review {
        errors.push(PrTraceabilityError::ConflictingCodeReviewPolicy);
        return errors;
    }

    if let Some(marker) = blocked_review_marker_in_title(&document.title) {
        errors.push(PrTraceabilityError::BlockedReviewMarker {
            location: "title",
            marker,
        });
    }
    if let Some(marker) = blocked_review_marker_in_body(&document.body) {
        errors.push(PrTraceabilityError::BlockedReviewMarker {
            location: "body",
            marker,
        });
    }

    let sections = h2_sections(&document.body);
    let mut previous_index = None;
    let mut previous_section = None;
    for required in REQUIRED_SECTIONS {
        let Some(index) = section_index(&sections, required) else {
            errors.push(PrTraceabilityError::MissingSection { section: required });
            continue;
        };
        if let (Some(previous_index), Some(previous_section_id)) =
            (previous_index, previous_section)
            && index <= previous_index
        {
            errors.push(PrTraceabilityError::SectionOutOfOrder {
                section: required,
                previous_section: previous_section_id,
            });
        }
        if section_body(&document.body, &sections, required)
            .trim()
            .is_empty()
        {
            errors.push(PrTraceabilityError::EmptySection { section: required });
        }
        previous_index = Some(index);
        previous_section = Some(required);
    }

    if section_index(&sections, "Issue").is_some() {
        let issue = section_body(&document.body, &sections, "Issue");
        if !contains_issue_reference(issue) {
            errors.push(PrTraceabilityError::MissingIssueReference);
        }
    }

    if section_index(&sections, "Summary").is_some() {
        let summary = section_body(&document.body, &sections, "Summary");
        if !summary
            .lines()
            .any(|line| line.trim_start().starts_with("- "))
        {
            errors.push(PrTraceabilityError::MissingSummaryBullet);
        }
    }

    if section_index(&sections, "Traceability").is_some() {
        let traceability = section_body(&document.body, &sections, "Traceability");
        for field in REQUIRED_TRACEABILITY_FIELDS {
            if !traceability.contains(field) {
                errors.push(PrTraceabilityError::MissingTraceabilityField { field });
            }
        }
    }

    if section_index(&sections, "Evidence").is_some() {
        let evidence = section_body(&document.body, &sections, "Evidence");
        for field in REQUIRED_EVIDENCE_FIELDS {
            if !evidence.contains(field) {
                errors.push(PrTraceabilityError::MissingEvidenceField { field });
            }
        }
    }

    let code_review = section_body(&document.body, &sections, "Code Review");
    let code_review_present = section_index(&sections, "Code Review").is_some();
    if policy.require_code_review && !code_review_present {
        errors.push(PrTraceabilityError::CodeReviewRequired);
    }
    if policy.require_code_review && code_review_present {
        errors.extend(validate_code_review_section_all(code_review));
    }
    if policy.forbid_code_review && code_review_present {
        errors.push(PrTraceabilityError::CodeReviewForbidden);
    }

    errors
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct H2Section<'a> {
    title: &'a str,
    body_start: usize,
    body_end: usize,
}

fn h2_sections(body: &str) -> Vec<H2Section<'_>> {
    let mut headers = Vec::new();
    let mut offset = 0usize;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if let Some(title) = trimmed.strip_prefix("## ")
            && !title.starts_with('#')
        {
            headers.push((title.trim(), offset + line.len()));
        }
        offset += line.len();
    }

    let mut sections = Vec::new();
    for (index, (title, body_start)) in headers.iter().copied().enumerate() {
        let body_end = headers
            .get(index + 1)
            .map(|(_, next_body_start)| header_start_before(body, *next_body_start))
            .unwrap_or(body.len());
        sections.push(H2Section {
            title,
            body_start,
            body_end,
        });
    }
    sections
}

fn header_start_before(body: &str, next_body_start: usize) -> usize {
    body[..next_body_start]
        .rfind("\n## ")
        .map(|index| index + 1)
        .or_else(|| body[..next_body_start].find("## "))
        .unwrap_or(next_body_start)
}

fn section_index(sections: &[H2Section<'_>], title: &str) -> Option<usize> {
    sections.iter().position(|section| section.title == title)
}

fn section_body<'a>(body: &'a str, sections: &[H2Section<'_>], title: &str) -> &'a str {
    let Some(section) = sections.iter().find(|section| section.title == title) else {
        return "";
    };
    &body[section.body_start..section.body_end]
}

fn contains_issue_reference(issue: &str) -> bool {
    ISSUE_REFERENCE_MARKERS
        .iter()
        .any(|marker| issue.contains(marker))
}

fn validate_code_review_section(code_review: &str) -> Result<(), PrTraceabilityError> {
    if code_review_requests_changes(code_review) {
        return Err(PrTraceabilityError::CodeReviewRequestsChanges);
    }
    if !code_review_has_reviewer(code_review) {
        return Err(PrTraceabilityError::MissingCodeReviewReviewer);
    }
    if !code_review_has_approval(code_review) {
        return Err(PrTraceabilityError::MissingCodeReviewApproval);
    }
    if !code_review_has_resolved_items(code_review) {
        return Err(PrTraceabilityError::MissingCodeReviewResolvedItems);
    }
    if !code_review_has_deferred_items(code_review) {
        return Err(PrTraceabilityError::MissingCodeReviewDeferredItems);
    }

    Ok(())
}

/// Collect-all sibling of [`validate_code_review_section`] for `validate_pr_traceability_all`.
fn validate_code_review_section_all(code_review: &str) -> Vec<PrTraceabilityError> {
    let mut errors = Vec::new();
    if code_review_requests_changes(code_review) {
        errors.push(PrTraceabilityError::CodeReviewRequestsChanges);
    }
    if !code_review_has_reviewer(code_review) {
        errors.push(PrTraceabilityError::MissingCodeReviewReviewer);
    }
    if !code_review_has_approval(code_review) {
        errors.push(PrTraceabilityError::MissingCodeReviewApproval);
    }
    if !code_review_has_resolved_items(code_review) {
        errors.push(PrTraceabilityError::MissingCodeReviewResolvedItems);
    }
    if !code_review_has_deferred_items(code_review) {
        errors.push(PrTraceabilityError::MissingCodeReviewDeferredItems);
    }
    errors
}

fn code_review_requests_changes(code_review: &str) -> bool {
    code_review_has_field_value(
        code_review,
        &["verdict", "reviewer-agent", "reviewer agent"],
        |value| {
            value.contains("request changes")
                || value.contains("request_changes")
                || value.contains("changes requested")
                || value.contains("changes_requested")
        },
    )
}

fn code_review_has_reviewer(code_review: &str) -> bool {
    code_review_has_field_value(
        code_review,
        &["reviewer-agent", "reviewer agent", "reviewer"],
        |value| !value.is_empty(),
    )
}

fn code_review_has_approval(code_review: &str) -> bool {
    code_review_has_field_value(code_review, &["verdict"], verdict_value_is_approval)
}

fn code_review_has_resolved_items(code_review: &str) -> bool {
    code_review_has_field_value(code_review, &["resolved items", "resolved"], |value| {
        !value.is_empty()
    })
}

fn code_review_has_deferred_items(code_review: &str) -> bool {
    code_review_has_field_value(code_review, &["deferred items", "deferred"], |value| {
        !value.is_empty()
    })
}

fn code_review_has_field_value(
    code_review: &str,
    field_names: &[&str],
    predicate: impl Fn(&str) -> bool,
) -> bool {
    code_review.lines().any(|line| {
        let normalized = review_marker_line(line);
        let Some((field, value)) = normalized.split_once(':') else {
            return false;
        };
        let field = review_field_name(field);
        field_names.iter().any(|name| *name == field) && predicate(value.trim())
    })
}

fn verdict_value_is_approval(value: &str) -> bool {
    matches!(
        verdict_value_normalized(value).as_deref(),
        Some("approve" | "approved")
    )
}

fn verdict_value_normalized(value: &str) -> Option<String> {
    let normalized = value
        .trim()
        .trim_matches(|character| matches!(character, '*' | '_' | '`'))
        .trim()
        .to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn blocked_review_marker_in_title(title: &str) -> Option<&'static str> {
    let normalized = title.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    if normalized.contains("pending review") {
        return Some("pending review");
    }
    if normalized.contains("review pending") {
        return Some("review pending");
    }
    if normalized.contains("awaiting review") {
        return Some("awaiting review");
    }
    if normalized.contains("needs review") {
        return Some("needs review");
    }
    if normalized.starts_with("blocked")
        || normalized.contains("[blocked")
        || normalized.contains("(blocked")
        || normalized.contains(": blocked")
        || normalized.contains(" blocked:")
    {
        return Some("blocked");
    }
    None
}

fn blocked_review_marker_in_body(body: &str) -> Option<&'static str> {
    body.lines().find_map(blocked_review_marker_in_body_line)
}

fn blocked_review_marker_in_body_line(line: &str) -> Option<&'static str> {
    let normalized = review_marker_line(line);
    if normalized.is_empty() {
        return None;
    }
    if let Some((field, value)) = normalized.split_once(':') {
        let field = review_field_name(field);
        if matches!(
            field.as_str(),
            "verdict" | "review" | "review status" | "status"
        ) && let Some(marker) = blocked_review_marker_in_text(value.trim())
        {
            return Some(marker);
        }
    }
    if normalized.starts_with("blocked")
        || normalized.starts_with("status: blocked")
        || normalized.starts_with("review: blocked")
        || normalized.starts_with("review status: blocked")
    {
        return Some("blocked");
    }
    if normalized.starts_with("pending review")
        || normalized.starts_with("status: pending review")
        || normalized.starts_with("review: pending review")
        || normalized.starts_with("review status: pending review")
    {
        return Some("pending review");
    }
    if normalized.starts_with("review pending") || normalized.starts_with("status: review pending")
    {
        return Some("review pending");
    }
    if normalized.starts_with("awaiting review")
        || normalized.starts_with("status: awaiting review")
    {
        return Some("awaiting review");
    }
    if normalized.starts_with("needs review") || normalized.starts_with("status: needs review") {
        return Some("needs review");
    }
    None
}

fn review_marker_line(line: &str) -> String {
    let mut normalized = line.trim().to_ascii_lowercase();
    loop {
        let stripped = normalized
            .strip_prefix("- ")
            .or_else(|| normalized.strip_prefix("* "))
            .or_else(|| normalized.strip_prefix("> "))
            .or_else(|| normalized.strip_prefix("[ ] "))
            .or_else(|| normalized.strip_prefix("[x] "))
            .map(str::trim_start);
        let Some(stripped) = stripped else {
            return normalized;
        };
        normalized = stripped.to_string();
    }
}

fn review_field_name(field: &str) -> String {
    field
        .trim()
        .trim_matches(|character| matches!(character, '*' | '_' | '`'))
        .trim()
        .to_owned()
}

fn blocked_review_marker_in_text(text: &str) -> Option<&'static str> {
    if text.contains("pending review") {
        return Some("pending review");
    }
    if text.contains("review pending") {
        return Some("review pending");
    }
    if text.contains("awaiting review") {
        return Some("awaiting review");
    }
    if text.contains("needs review") {
        return Some("needs review");
    }
    if text.contains("blocked") {
        return Some("blocked");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_author_pr_shape_without_code_review() {
        assert_eq!(
            validate_pr_traceability(&document(valid_body()), author_policy()),
            Ok(PrTraceabilityReport {
                required_sections_checked: 5,
                code_review_present: false,
            })
        );
    }

    #[test]
    fn accepts_merge_ready_body_when_code_review_required() {
        assert_eq!(
            validate_pr_traceability(&document(merge_ready_body()), merge_policy()),
            Ok(PrTraceabilityReport {
                required_sections_checked: 5,
                code_review_present: true,
            })
        );

        let markdown_field_body = format!(
            "{}\n## Code Review\n**Reviewer agent:** rust-reviewer\n**Verdict:** APPROVE\n**Resolved items:** none\n**Deferred items:** none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(&document(&markdown_field_body), merge_policy()),
            Ok(PrTraceabilityReport {
                required_sections_checked: 5,
                code_review_present: true,
            })
        );
    }

    #[test]
    fn rejects_missing_or_out_of_order_sections() {
        assert_eq!(
            validate_pr_traceability(&document("## Issue\nCloses #1\n"), author_policy()),
            Err(PrTraceabilityError::MissingSection { section: "Summary" })
        );

        let out_of_order = "## Summary\n- thing\n## Issue\nCloses #1\n## Verification\npass\n## Traceability\nCatalog records touched: n/a\nCross-axis contracts touched: n/a\nADRs cited: n/a\n## Evidence\nAudit-chain emission: n/a\nFoundation-bypass referenced: n/a\nPer-pack regulator-watch impact: n/a\n";
        assert_eq!(
            validate_pr_traceability(&document(out_of_order), author_policy()),
            Err(PrTraceabilityError::SectionOutOfOrder {
                section: "Summary",
                previous_section: "Issue",
            })
        );
    }

    #[test]
    fn rejects_weak_required_content() {
        let missing_issue = valid_body().replace("Closes #123", "No ticket");
        assert_eq!(
            validate_pr_traceability(&document(&missing_issue), author_policy()),
            Err(PrTraceabilityError::MissingIssueReference)
        );

        let missing_summary_bullet =
            valid_body().replace("- Implemented the thing.\n", "Implemented the thing.\n");
        assert_eq!(
            validate_pr_traceability(&document(&missing_summary_bullet), author_policy()),
            Err(PrTraceabilityError::MissingSummaryBullet)
        );
    }

    #[test]
    fn rejects_blocked_or_pending_review_markers() {
        assert_eq!(
            validate_pr_traceability(
                &document_with_title("BLOCKED: pending review", merge_ready_body()),
                merge_policy(),
            ),
            Err(PrTraceabilityError::BlockedReviewMarker {
                location: "title",
                marker: "pending review",
            })
        );

        let blocked_body = format!("{}\nStatus: awaiting review\n", merge_ready_body());
        assert_eq!(
            validate_pr_traceability(&document(&blocked_body), merge_policy()),
            Err(PrTraceabilityError::BlockedReviewMarker {
                location: "body",
                marker: "awaiting review",
            })
        );

        let checkbox_blocked_body = format!("{}\n- [ ] Pending review\n", merge_ready_body());
        assert_eq!(
            validate_pr_traceability(&document(&checkbox_blocked_body), merge_policy()),
            Err(PrTraceabilityError::BlockedReviewMarker {
                location: "body",
                marker: "pending review",
            })
        );

        let review_status_blocked_body =
            format!("{}\nReview status: blocked\n", merge_ready_body());
        assert_eq!(
            validate_pr_traceability(&document(&review_status_blocked_body), merge_policy()),
            Err(PrTraceabilityError::BlockedReviewMarker {
                location: "body",
                marker: "blocked",
            })
        );

        let pending_verdict_body = format!(
            "{}\n## Code Review\nreviewer-agent: rust-reviewer\nverdict: APPROVE (pending review)\nResolved items: none\nDeferred items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(&document(&pending_verdict_body), merge_policy()),
            Err(PrTraceabilityError::BlockedReviewMarker {
                location: "body",
                marker: "pending review",
            })
        );

        let blocked_verdict_body = format!(
            "{}\n## Code Review\nreviewer-agent: rust-reviewer\nverdict: APPROVE / BLOCKED\nResolved items: none\nDeferred items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(&document(&blocked_verdict_body), merge_policy()),
            Err(PrTraceabilityError::BlockedReviewMarker {
                location: "body",
                marker: "blocked",
            })
        );

        let pending_review_status_body = format!(
            "{}\n## Code Review\nreviewer-agent: rust-reviewer\nreview status: APPROVE pending review\nverdict: APPROVE\nResolved items: none\nDeferred items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(&document(&pending_review_status_body), merge_policy()),
            Err(PrTraceabilityError::BlockedReviewMarker {
                location: "body",
                marker: "pending review",
            })
        );
    }

    #[test]
    fn enforces_code_review_policy() {
        assert_eq!(
            validate_pr_traceability(&document(valid_body()), merge_policy()),
            Err(PrTraceabilityError::CodeReviewRequired)
        );

        let body = format!("{}\n## Code Review\nAPPROVE\n", valid_body());
        assert_eq!(
            validate_pr_traceability(&document(&body), author_policy()),
            Err(PrTraceabilityError::CodeReviewForbidden)
        );
    }

    #[test]
    fn rejects_incomplete_or_negative_code_review_evidence() {
        let missing_reviewer = format!(
            "{}\n## Code Review\nverdict: APPROVE\nResolved items: none\nDeferred items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(&document(&missing_reviewer), merge_policy()),
            Err(PrTraceabilityError::MissingCodeReviewReviewer)
        );

        let missing_deferred = format!(
            "{}\n## Code Review\nreviewer-agent: rust-reviewer\nverdict: APPROVE\nResolved items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(&document(&missing_deferred), merge_policy()),
            Err(PrTraceabilityError::MissingCodeReviewDeferredItems)
        );

        let rejected = format!(
            "{}\n## Code Review\nreviewer-agent: rust-reviewer\nverdict: REQUEST CHANGES\nResolved items: none\nDeferred items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(&document(&rejected), merge_policy()),
            Err(PrTraceabilityError::CodeReviewRequestsChanges)
        );

        let missing_verdict = format!(
            "{}\n## Code Review\nreviewer-agent: rust-reviewer\nResolved items: approved stale thread text\nDeferred items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(&document(&missing_verdict), merge_policy()),
            Err(PrTraceabilityError::MissingCodeReviewApproval)
        );

        for negative_verdict in ["not approved", "unapproved", "disapproved"] {
            let body = format!(
                "{}\n## Code Review\nreviewer-agent: rust-reviewer\nverdict: {negative_verdict}\nResolved items: none\nDeferred items: none\n",
                valid_body()
            );
            assert_eq!(
                validate_pr_traceability(&document(&body), merge_policy()),
                Err(PrTraceabilityError::MissingCodeReviewApproval)
            );
        }

        for qualified_approval in ["approve with nits", "approved after follow-up"] {
            let body = format!(
                "{}\n## Code Review\nreviewer-agent: rust-reviewer\nverdict: {qualified_approval}\nResolved items: none\nDeferred items: none\n",
                valid_body()
            );
            assert_eq!(
                validate_pr_traceability(&document(&body), merge_policy()),
                Err(PrTraceabilityError::MissingCodeReviewApproval)
            );
        }
    }

    #[test]
    fn rejects_missing_traceability_or_evidence_fields() {
        let missing_traceability = valid_body().replace("ADRs cited: ADR-0001\n", "");
        assert_eq!(
            validate_pr_traceability(&document(&missing_traceability), author_policy()),
            Err(PrTraceabilityError::MissingTraceabilityField {
                field: "ADRs cited",
            })
        );

        let missing_evidence = valid_body().replace("Audit-chain emission: EVT-1\n", "");
        assert_eq!(
            validate_pr_traceability(&document(&missing_evidence), author_policy()),
            Err(PrTraceabilityError::MissingEvidenceField {
                field: "Audit-chain emission",
            })
        );
    }

    #[test]
    fn scaffold_contains_every_required_section_and_field_label() {
        let scaffold = scaffold_pr_body();
        for section in REQUIRED_SECTIONS {
            assert!(
                scaffold.contains(&format!("## {section}")),
                "scaffold missing section header {section:?}:\n{scaffold}"
            );
        }
        for field in REQUIRED_TRACEABILITY_FIELDS {
            assert!(
                scaffold.contains(field),
                "scaffold missing Traceability field {field:?}:\n{scaffold}"
            );
        }
        for field in REQUIRED_EVIDENCE_FIELDS {
            assert!(
                scaffold.contains(field),
                "scaffold missing Evidence field {field:?}:\n{scaffold}"
            );
        }
        assert!(scaffold.contains(ISSUE_REFERENCE_MARKERS[1]));
        assert!(scaffold.contains("## Code Review"));
        assert!(scaffold.contains("Verdict: pending"));
    }

    #[test]
    fn scaffold_passes_validator_except_for_the_pending_review_verdict() {
        // Keystone test: the scaffold satisfies every author-owned rule; the ONLY violation
        // under merge policy is the (correct, expected) unapproved verdict.
        let scaffold = scaffold_pr_body();
        assert_eq!(
            validate_pr_traceability(&document(&scaffold), merge_policy()),
            Err(PrTraceabilityError::MissingCodeReviewApproval)
        );
        assert_eq!(
            validate_pr_traceability_all(&document(&scaffold), merge_policy()),
            vec![PrTraceabilityError::MissingCodeReviewApproval]
        );
    }

    #[test]
    fn all_violations_mode_reports_every_defect_not_just_the_first() {
        let broken_body = valid_body()
            .replace("Closes #123", "No ticket")
            .replace("Audit-chain emission: EVT-1\n", "");
        assert_eq!(
            validate_pr_traceability_all(&document(&broken_body), author_policy()),
            vec![
                PrTraceabilityError::MissingIssueReference,
                PrTraceabilityError::MissingEvidenceField {
                    field: "Audit-chain emission",
                },
            ]
        );
    }

    #[test]
    fn rejects_the_body_defect_shapes_observed_on_real_prs() {
        // Regression pins for the four body defects actually observed across ten PRs (~a dozen
        // wasted FULL-tier runs). Each is reachable only through the constants and matchers
        // above, so pinning the REAL shapes — not synthetic mutations — is what makes the
        // `--check` author workflow trustworthy enough to run instead of CI.
        let singular_adr_field = merge_ready_body().replace("- ADRs cited:", "- ADR cited:");
        assert_eq!(
            validate_pr_traceability(&document(&singular_adr_field), merge_policy()),
            Err(PrTraceabilityError::MissingTraceabilityField {
                field: "ADRs cited",
            })
        );

        // Body composed through a shell that never interpreted `\n`, so `## Evidence` is real
        // text sitting on the tail of another line and never starts one.
        let escaped_newlines =
            merge_ready_body().replace("\n\n## Evidence\n", "\\n\\n## Evidence\\n");
        assert_eq!(
            validate_pr_traceability(&document(&escaped_newlines), merge_policy()),
            Err(PrTraceabilityError::MissingSection {
                section: "Evidence",
            })
        );

        let summary_prose_without_bullet = merge_ready_body().replace(
            "- Implemented the thing.",
            "Implemented the thing, because the old path double-counted refunds.",
        );
        assert_eq!(
            validate_pr_traceability(&document(&summary_prose_without_bullet), merge_policy()),
            Err(PrTraceabilityError::MissingSummaryBullet)
        );

        // Decorated verdict: `verdict_value_is_approval` needs the normalized value to be
        // EXACTLY `approve`/`approved`, and the trailing severity tally is part of the value.
        let decorated_verdict = merge_ready_body().replace(
            "verdict: APPROVE\n",
            "- Verdict: **APPROVED** — 0 CRITICAL, 0 HIGH\n",
        );
        assert_eq!(
            validate_pr_traceability(&document(&decorated_verdict), merge_policy()),
            Err(PrTraceabilityError::MissingCodeReviewApproval)
        );

        // Control: the undefected base body is admissible, so every fixture above fails for its
        // own defect rather than for a broken fixture base.
        assert_eq!(
            validate_pr_traceability(&document(merge_ready_body()), merge_policy()),
            Ok(PrTraceabilityReport {
                required_sections_checked: 5,
                code_review_present: true,
            })
        );
    }

    fn valid_body() -> &'static str {
        "## Issue\nCloses #123\n\n## Summary\n- Implemented the thing.\n\n## Verification\n- pass: oya dev check\n\n## Traceability\n- Catalog records touched: oya-intelligence-capability-kernel\n- Cross-axis contracts touched: none\n- ADRs cited: ADR-0001\n\n## Evidence\n- Audit-chain emission: EVT-1\n- Foundation-bypass referenced (if any): none\n- Per-pack regulator-watch impact (if any): none\n"
    }

    fn merge_ready_body() -> &'static str {
        "## Issue\nCloses #123\n\n## Summary\n- Implemented the thing.\n\n## Verification\n- pass: oya dev check\n\n## Traceability\n- Catalog records touched: oya-intelligence-capability-kernel\n- Cross-axis contracts touched: none\n- ADRs cited: ADR-0001\n\n## Evidence\n- Audit-chain emission: EVT-1\n- Foundation-bypass referenced (if any): none\n- Per-pack regulator-watch impact (if any): none\n\n## Code Review\nreviewer-agent: rust-reviewer\nverdict: APPROVE\nResolved items: none\nDeferred items: none\n"
    }

    fn document(body: &str) -> PrTraceabilityDocument {
        document_with_title("Ready for review", body)
    }

    fn document_with_title(title: &str, body: &str) -> PrTraceabilityDocument {
        PrTraceabilityDocument {
            document_id: "pr-body".into(),
            title: title.into(),
            body: body.into(),
        }
    }

    fn author_policy() -> PrTraceabilityPolicy {
        PrTraceabilityPolicy {
            require_code_review: false,
            forbid_code_review: true,
        }
    }

    fn merge_policy() -> PrTraceabilityPolicy {
        PrTraceabilityPolicy {
            require_code_review: true,
            forbid_code_review: false,
        }
    }
}
