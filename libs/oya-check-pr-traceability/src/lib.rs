//! Foundry PR traceability fitness kernel.
//!
//! Oyatie's PR contract requires five traceability H2 sections: Issue, Summary,
//! Verification, Traceability, and Evidence. `## Code Review` is supplied by
//! the automated reviewer-agent pipeline for merge-ready bodies, so this pure
//! kernel can validate author-only bodies and merge-ready bodies by policy.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrTraceabilityDocument {
    pub document_id: String, // data_class: INTERNAL_ONLY
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
    MissingCodeReviewField {
        field: &'static str,
    },
    CodeReviewRequired,
    CodeReviewForbidden,
}

const REQUIRED_SECTIONS: [&str; 5] = [
    "Issue",
    "Summary",
    "Verification",
    "Traceability",
    "Evidence",
];

pub fn validate_pr_traceability(
    document: &PrTraceabilityDocument,
    policy: PrTraceabilityPolicy,
) -> Result<PrTraceabilityReport, PrTraceabilityError> {
    if policy.require_code_review && policy.forbid_code_review {
        return Err(PrTraceabilityError::ConflictingCodeReviewPolicy);
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
    for field in [
        "Catalog records touched",
        "Cross-axis contracts touched",
        "ADRs cited",
    ] {
        if !traceability.contains(field) {
            return Err(PrTraceabilityError::MissingTraceabilityField { field });
        }
    }

    let evidence = section_body(&document.body, &sections, "Evidence");
    for field in [
        "Audit-chain emission",
        "Foundation-bypass referenced",
        "Per-pack regulator-watch impact",
    ] {
        if !evidence.contains(field) {
            return Err(PrTraceabilityError::MissingEvidenceField { field });
        }
    }

    let code_review_present = section_index(&sections, "Code Review").is_some();
    if policy.require_code_review && !code_review_present {
        return Err(PrTraceabilityError::CodeReviewRequired);
    }
    if policy.require_code_review {
        validate_code_review_evidence(section_body(&document.body, &sections, "Code Review"))?;
    }
    if policy.forbid_code_review && code_review_present {
        return Err(PrTraceabilityError::CodeReviewForbidden);
    }

    Ok(PrTraceabilityReport {
        required_sections_checked: REQUIRED_SECTIONS.len(),
        code_review_present,
    })
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
    issue.contains("Closes #") || issue.contains("Refs #") || issue.contains("Blocks #")
}

fn validate_code_review_evidence(code_review: &str) -> Result<(), PrTraceabilityError> {
    require_code_review_field(
        code_review,
        &["Reviewer agent", "reviewer-agent"],
        "Reviewer agent",
    )?;

    let verdict = require_code_review_field(code_review, &["Verdict"], "Verdict: APPROVE")?;
    if !verdict.eq_ignore_ascii_case("APPROVE") {
        return Err(PrTraceabilityError::MissingCodeReviewField {
            field: "Verdict: APPROVE",
        });
    }

    require_code_review_field(code_review, &["Resolved items"], "Resolved items")?;
    require_code_review_field(code_review, &["Deferred items"], "Deferred items")?;
    Ok(())
}

fn require_code_review_field<'a>(
    code_review: &'a str,
    labels: &[&str],
    field: &'static str,
) -> Result<&'a str, PrTraceabilityError> {
    let Some(value) = code_review_field_value(code_review, labels) else {
        return Err(PrTraceabilityError::MissingCodeReviewField { field });
    };
    if value.is_empty() {
        return Err(PrTraceabilityError::MissingCodeReviewField { field });
    }
    Ok(value)
}

fn code_review_field_value<'a>(code_review: &'a str, labels: &[&str]) -> Option<&'a str> {
    for line in code_review.lines() {
        let mut candidate = line.trim();
        for prefix in ["- ", "* "] {
            if let Some(rest) = candidate.strip_prefix(prefix) {
                candidate = rest.trim_start();
            }
        }

        let Some((raw_label, raw_value)) = candidate.split_once(':') else {
            continue;
        };
        if labels.iter().any(|expected| {
            normalize_code_review_label(raw_label) == normalize_code_review_label(expected)
        }) {
            return Some(raw_value.trim().trim_start_matches('*').trim());
        }
    }
    None
}

fn normalize_code_review_label(label: &str) -> String {
    label
        .chars()
        .filter_map(|character| match character {
            '*' | '`' => None,
            '_' | '-' => Some('-'),
            character if character.is_whitespace() => Some('-'),
            character => Some(character.to_ascii_lowercase()),
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
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
        let body = format!(
            "{}\n## Code Review\n- Reviewer agent: architect 136-IdentityBridgeReReview\n- Verdict: APPROVE\n- Resolved items: none\n- Deferred items: none\n",
            valid_body()
        );

        assert_eq!(
            validate_pr_traceability(
                &document(&body),
                PrTraceabilityPolicy {
                    require_code_review: true,
                    forbid_code_review: false,
                },
            ),
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
    fn enforces_code_review_policy() {
        assert_eq!(
            validate_pr_traceability(
                &document(valid_body()),
                PrTraceabilityPolicy {
                    require_code_review: true,
                    forbid_code_review: false,
                },
            ),
            Err(PrTraceabilityError::CodeReviewRequired)
        );

        let body = format!("{}\n## Code Review\nAPPROVE\n", valid_body());
        assert_eq!(
            validate_pr_traceability(&document(&body), author_policy()),
            Err(PrTraceabilityError::CodeReviewForbidden)
        );
    }

    #[test]
    fn rejects_code_review_without_reviewer_agent_approve_fields() {
        let missing_reviewer_agent = format!(
            "{}\n## Code Review\n- Verdict: APPROVE\n- Resolved items: none\n- Deferred items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(
                &document(&missing_reviewer_agent),
                PrTraceabilityPolicy {
                    require_code_review: true,
                    forbid_code_review: false,
                },
            ),
            Err(PrTraceabilityError::MissingCodeReviewField {
                field: "Reviewer agent",
            })
        );

        let missing_approve_verdict = format!(
            "{}\n## Code Review\n- Reviewer agent: architect 136-IdentityBridgeReReview\n- Verdict: REQUEST CHANGES\n- Resolved items: none\n- Deferred items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(
                &document(&missing_approve_verdict),
                PrTraceabilityPolicy {
                    require_code_review: true,
                    forbid_code_review: false,
                },
            ),
            Err(PrTraceabilityError::MissingCodeReviewField {
                field: "Verdict: APPROVE",
            })
        );

        let ambiguous_approve_verdict = format!(
            "{}\n## Code Review\n- Reviewer agent: architect 136-IdentityBridgeReReview\n- Verdict: DO NOT APPROVE\n- Resolved items: none\n- Deferred items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(
                &document(&ambiguous_approve_verdict),
                PrTraceabilityPolicy {
                    require_code_review: true,
                    forbid_code_review: false,
                },
            ),
            Err(PrTraceabilityError::MissingCodeReviewField {
                field: "Verdict: APPROVE",
            })
        );

        let missing_resolved_items = format!(
            "{}\n## Code Review\n- Reviewer agent: architect 136-IdentityBridgeReReview\n- Verdict: APPROVE\n- Deferred items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(
                &document(&missing_resolved_items),
                PrTraceabilityPolicy {
                    require_code_review: true,
                    forbid_code_review: false,
                },
            ),
            Err(PrTraceabilityError::MissingCodeReviewField {
                field: "Resolved items",
            })
        );

        let missing_deferred_items = format!(
            "{}\n## Code Review\n- Reviewer agent: architect 136-IdentityBridgeReReview\n- Verdict: APPROVE\n- Resolved items: none\n",
            valid_body()
        );
        assert_eq!(
            validate_pr_traceability(
                &document(&missing_deferred_items),
                PrTraceabilityPolicy {
                    require_code_review: true,
                    forbid_code_review: false,
                },
            ),
            Err(PrTraceabilityError::MissingCodeReviewField {
                field: "Deferred items",
            })
        );
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

    fn valid_body() -> &'static str {
        "## Issue\nCloses #123\n\n## Summary\n- Implemented the thing.\n\n## Verification\n- pass: oya dev check\n\n## Traceability\n- Catalog records touched: oya-intelligence-capability-kernel\n- Cross-axis contracts touched: none\n- ADRs cited: ADR-0001\n\n## Evidence\n- Audit-chain emission: EVT-1\n- Foundation-bypass referenced (if any): none\n- Per-pack regulator-watch impact (if any): none\n"
    }

    fn document(body: &str) -> PrTraceabilityDocument {
        PrTraceabilityDocument {
            document_id: "pr-body".into(),
            body: body.into(),
        }
    }

    fn author_policy() -> PrTraceabilityPolicy {
        PrTraceabilityPolicy {
            require_code_review: false,
            forbid_code_review: true,
        }
    }
}
