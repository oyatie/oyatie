//! ADR shape fitness kernel.
//!
//! Pure validator for the minimal ADR shape currently enforced by the M01-P01
//! Data Use Boundary acceptance gate. It validates title, status, and the core
//! Context -> Decision -> Consequences section order without filesystem I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrDocument {
    pub path: String, // data_class: INTERNAL_ONLY
    pub text: String, // data_class: INTERNAL_ONLY
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrShapeFitnessReport {
    pub adrs_checked: usize, // data_class: INTERNAL_ONLY
}

/// A deterministic structural observation. It is deliberately not an
/// admission decision, lifecycle transition, or authority assertion.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AdrShapeFinding {
    pub path: String,       // data_class: INTERNAL_ONLY
    pub code: &'static str, // data_class: INTERNAL_ONLY
    pub message: String,    // data_class: INTERNAL_ONLY
}

/// Diagnostic-only output for corpus migration inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrShapeAuditReport {
    pub adrs_checked: usize,            // data_class: INTERNAL_ONLY
    pub findings: Vec<AdrShapeFinding>, // data_class: INTERNAL_ONLY
}

const DIAGNOSTIC_REQUIRED_SECTIONS: [&str; 4] =
    ["Context", "Decision", "Decision Drivers", "Consequences"];

/// Return sorted, reproducible structural findings for the supplied corpus.
///
/// This function intentionally has no admission semantics: a zero-finding
/// report does not authorize a status change, planning, dispatch, or closure.
pub fn audit_adr_shape_fitness(adrs: &[AdrDocument]) -> AdrShapeAuditReport {
    let mut findings = Vec::new();
    for adr in adrs {
        findings.extend(audit_one(adr));
    }
    findings.sort();
    AdrShapeAuditReport {
        adrs_checked: adrs.len(),
        findings,
    }
}

#[derive(Debug)]
struct MarkdownHeading<'a> {
    level: usize,
    title: &'a str,
}

struct VisibleMarkdownLine<'a> {
    content: &'a str,
    is_quoted: bool,
}

fn markdown_headings(text: &str) -> Vec<MarkdownHeading<'_>> {
    let mut headings = Vec::new();
    for line in visible_markdown_lines(text)
        .into_iter()
        .filter(|line| !line.is_quoted)
    {
        let hashes = line
            .content
            .chars()
            .take_while(|character| *character == '#')
            .count();
        if (1..=6).contains(&hashes) && line.content.as_bytes().get(hashes) == Some(&b' ') {
            headings.push(MarkdownHeading {
                level: hashes,
                title: line.content[hashes + 1..].trim(),
            });
        }
    }
    headings
}

/// Lines that remain visible after Markdown code-block and indented-code
/// handling. Both heading and status extraction use this state machine.
fn visible_markdown_lines(text: &str) -> Vec<VisibleMarkdownLine<'_>> {
    let mut lines = Vec::new();
    let mut open_fence: Option<(char, usize)> = None;

    for line in text.lines() {
        let Some((content, is_quoted)) = normalized_fence_line(line) else {
            continue;
        };
        let marker = content.chars().next();
        let marker_len = marker.map_or(0, |candidate| {
            content
                .chars()
                .take_while(|character| *character == candidate)
                .count()
        });
        if let Some((open_marker, minimum_len)) = open_fence {
            if marker == Some(open_marker)
                && marker_len >= minimum_len
                && content[marker_len..].trim().is_empty()
            {
                open_fence = None;
            }
            continue;
        }
        if matches!(marker, Some('`' | '~')) && marker_len >= 3 {
            open_fence = marker.map(|candidate| (candidate, marker_len));
            continue;
        }
        lines.push(VisibleMarkdownLine { content, is_quoted });
    }
    lines
}

fn normalized_fence_line(mut line: &str) -> Option<(&str, bool)> {
    let mut is_quoted = false;
    loop {
        let indentation = line.bytes().take_while(|byte| *byte == b' ').count();
        if indentation > 3 || line.as_bytes().get(indentation) == Some(&b'\t') {
            return None;
        }
        let trimmed = &line[indentation..];
        let Some(rest) = trimmed.strip_prefix('>') else {
            return Some((trimmed, is_quoted));
        };
        is_quoted = true;
        line = rest.strip_prefix(' ').unwrap_or(rest);
    }
}

fn audit_one(adr: &AdrDocument) -> Vec<AdrShapeFinding> {
    let headings = markdown_headings(&adr.text);
    let mut findings = Vec::new();
    let mut add = |code, message: String| {
        findings.push(AdrShapeFinding {
            path: adr.path.clone(),
            code,
            message,
        });
    };

    let titles = headings
        .iter()
        .filter(|heading| heading.level == 2)
        .map(|heading| heading.title)
        .collect::<Vec<_>>();
    for section in DIAGNOSTIC_REQUIRED_SECTIONS {
        let count = titles
            .iter()
            .filter(|title| title.eq_ignore_ascii_case(section))
            .count();
        if count == 0 {
            add("ADR_SECTION_MISSING", format!("missing ## {section}"));
        } else if count > 1 {
            add(
                "ADR_SECTION_DUPLICATE",
                format!("duplicate ## {section} headings"),
            );
        }
    }
    let positions = DIAGNOSTIC_REQUIRED_SECTIONS.map(|section| {
        titles
            .iter()
            .position(|title| title.eq_ignore_ascii_case(section))
    });
    for pair in positions.windows(2) {
        if let [Some(previous), Some(current)] = pair
            && current < previous
        {
            add(
                "ADR_SECTION_OUT_OF_ORDER",
                "required sections are not in Context -> Decision -> Decision Drivers -> Consequences order"
                    .to_owned(),
            );
            break;
        }
    }
    if headings.iter().any(|heading| {
        heading.level != 2
            && DIAGNOSTIC_REQUIRED_SECTIONS
                .iter()
                .any(|section| heading.title.eq_ignore_ascii_case(section))
    }) {
        add(
            "ADR_SECTION_MISNESTED",
            "required ADR section is not an H2 heading".to_owned(),
        );
    }

    let driver_position = headings.iter().position(|heading| {
        heading.level == 2 && heading.title.eq_ignore_ascii_case("Decision Drivers")
    });
    let decision_position = headings
        .iter()
        .position(|heading| heading.level == 2 && heading.title.eq_ignore_ascii_case("Decision"));
    let consequences_position = headings.iter().position(|heading| {
        heading.level == 2 && heading.title.eq_ignore_ascii_case("Consequences")
    });
    if !matches!(
        (decision_position, driver_position, consequences_position),
        (Some(decision), Some(drivers), Some(consequences)) if decision < drivers && drivers < consequences
    ) {
        add(
            "ADR_DECISION_DRIVERS_MISNESTED_OR_MISSING",
            "## Decision Drivers must be an H2 between ## Decision and ## Consequences".to_owned(),
        );
    }

    match diagnostic_status(&adr.text) {
        None => add(
            "ADR_STATUS_MISSING_MIGRATION_INVENTORY",
            "missing visible lifecycle status; inventory only".to_owned(),
        ),
        Some(status) if canonical_status(status).is_none() => add(
            "ADR_STATUS_MIGRATION_INVENTORY",
            format!("unrecognized or legacy status {status:?}; inventory only"),
        ),
        Some(status) if canonical_status(status) != Some(status) => add(
            "ADR_STATUS_MIGRATION_INVENTORY",
            format!("non-canonical status spelling {status:?}; inventory only"),
        ),
        _ => {}
    }
    findings
}

fn diagnostic_status(text: &str) -> Option<&str> {
    let body = if let Some((frontmatter, body)) = split_initial_yaml_frontmatter(text) {
        if let Some(status) = top_level_status_scalar(frontmatter) {
            return Some(status);
        }
        body
    } else {
        text
    };
    visible_markdown_lines(body).into_iter().find_map(|line| {
        line.content
            .trim()
            .strip_prefix("**Status:**")
            .map(str::trim)
    })
}

fn top_level_status_scalar(frontmatter: &str) -> Option<&str> {
    frontmatter.lines().find_map(|line| {
        (!line.starts_with(char::is_whitespace))
            .then(|| line.strip_prefix("status:").map(str::trim))
            .flatten()
            .filter(|value| !value.is_empty() && *value != "|" && *value != ">")
    })
}

fn canonical_status(status: &str) -> Option<&'static str> {
    match status {
        "Proposed" => Some("Proposed"),
        "Accepted" => Some("Accepted"),
        "Amended" => Some("Amended"),
        "Superseded" => Some("Superseded"),
        "Deprecated" => Some("Deprecated"),
        "Rejected" => Some("Rejected"),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdrShapeFitnessError {
    InvalidFilename {
        path: String,
    },
    MissingTitle {
        path: String,
    },
    MissingStatus {
        path: String,
    },
    InvalidStatus {
        path: String,
        status: String,
    },
    MissingAmendedDate {
        path: String,
    },
    MissingSection {
        path: String,
        section: &'static str,
    },
    SectionsOutOfOrder {
        path: String,
        previous: &'static str,
        current: &'static str,
    },
}

impl fmt::Display for AdrShapeFitnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFilename { path } => {
                write!(f, "{path}: expected ADR-NNNN-slug.md filename")
            }
            Self::MissingTitle { path } => write!(f, "{path}: missing first-line ADR title"),
            Self::MissingStatus { path } => write!(f, "{path}: missing ADR status"),
            Self::InvalidStatus { path, status } => {
                write!(f, "{path}: invalid ADR status {status:?}")
            }
            Self::MissingAmendedDate { path } => {
                write!(
                    f,
                    "{path}: status Amended requires a canonical amended_date"
                )
            }
            Self::MissingSection { path, section } => {
                write!(f, "{path}: missing required section ## {section}")
            }
            Self::SectionsOutOfOrder {
                path,
                previous,
                current,
            } => write!(
                f,
                "{path}: required sections out of order; ## {current} appears before ## {previous}"
            ),
        }
    }
}

impl std::error::Error for AdrShapeFitnessError {}

const REQUIRED_SECTIONS: [&str; 3] = ["Context", "Decision", "Consequences"];
const VALID_STATUSES: [&str; 6] = [
    "Proposed",
    "Accepted",
    "Amended",
    "Superseded",
    "Deprecated",
    "Rejected",
];

/// Whether an ADR lifecycle status remains live for propagation consumers.
///
/// The lowercase and parenthetical spellings are exact legacy corpus forms.
/// Broader case folding or prefix matching would accidentally promote unknown
/// lifecycle variants, so callers share this one closed predicate.
pub fn is_live_decision_status(status: &str) -> bool {
    matches!(
        status.trim(),
        "Accepted" | "accepted" | "Accepted (amendment)" | "Amended"
    )
}

/// Return the initial YAML frontmatter body when the document starts with a
/// canonical opening fence.
pub fn initial_yaml_frontmatter(text: &str) -> Option<&str> {
    split_initial_yaml_frontmatter(text).map(|(frontmatter, _)| frontmatter)
}

/// Whether initial YAML frontmatter declares a valid
/// `amended_date: YYYY-MM-DD` calendar date.
pub fn has_canonical_amended_date(text: &str) -> bool {
    let mut values = initial_frontmatter_scalar_values(text, "amended_date");
    let Some(value) = values.next() else {
        return false;
    };
    values.next().is_none() && is_canonical_date_scalar(value)
}

pub fn validate_adr_shape_fitness(
    adrs: &[AdrDocument],
) -> Result<AdrShapeFitnessReport, AdrShapeFitnessError> {
    for adr in adrs {
        validate_one(adr)?;
    }
    Ok(AdrShapeFitnessReport {
        adrs_checked: adrs.len(),
    })
}

fn validate_one(adr: &AdrDocument) -> Result<(), AdrShapeFitnessError> {
    let filename = adr.path.rsplit('/').next().unwrap_or(adr.path.as_str());
    if !is_adr_filename(filename) {
        return Err(AdrShapeFitnessError::InvalidFilename {
            path: adr.path.clone(),
        });
    }
    if !adr.text.lines().any(|line| line.starts_with("# ADR-")) {
        return Err(AdrShapeFitnessError::MissingTitle {
            path: adr.path.clone(),
        });
    }
    if initial_frontmatter_scalar_values(&adr.text, "status").count() > 1 {
        return Err(AdrShapeFitnessError::InvalidStatus {
            path: adr.path.clone(),
            status: "ambiguous duplicate initial-frontmatter status fields".to_owned(),
        });
    }
    let status = extract_status(&adr.text).ok_or_else(|| AdrShapeFitnessError::MissingStatus {
        path: adr.path.clone(),
    })?;
    if !VALID_STATUSES.contains(&status.as_str()) {
        return Err(AdrShapeFitnessError::InvalidStatus {
            path: adr.path.clone(),
            status,
        });
    }
    if status == "Amended" && !has_canonical_amended_date(&adr.text) {
        return Err(AdrShapeFitnessError::MissingAmendedDate {
            path: adr.path.clone(),
        });
    }

    let headings = section_headings(&adr.text);
    let mut previous_index = None;
    let mut previous_section = None;
    for section in REQUIRED_SECTIONS {
        let index = headings
            .iter()
            .position(|heading| heading == section)
            .ok_or_else(|| AdrShapeFitnessError::MissingSection {
                path: adr.path.clone(),
                section,
            })?;
        if let (Some(previous_index), Some(previous_section)) = (previous_index, previous_section)
            && index < previous_index
        {
            return Err(AdrShapeFitnessError::SectionsOutOfOrder {
                path: adr.path.clone(),
                previous: previous_section,
                current: section,
            });
        }
        previous_index = Some(index);
        previous_section = Some(section);
    }
    Ok(())
}

fn is_adr_filename(filename: &str) -> bool {
    let Some(rest) = filename.strip_prefix("ADR-") else {
        return false;
    };
    let Some((digits, slug)) = rest.split_once('-') else {
        return false;
    };
    digits.len() == 4
        && digits.chars().all(|c| c.is_ascii_digit())
        && slug.ends_with(".md")
        && slug.len() > 3
}

fn extract_status(text: &str) -> Option<String> {
    let lifecycle_surface = if text.starts_with("---\n") || text.starts_with("---\r\n") {
        let (frontmatter, body) = split_initial_yaml_frontmatter(text)?;
        if let Some(status) = frontmatter_scalar_values(frontmatter, "status").next() {
            return Some(clean_status(status));
        }
        body
    } else {
        text
    };
    let lines = lifecycle_surface.lines().take(32).collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim().trim_start_matches("- ").trim();
        if let Some(status) = trimmed.strip_prefix("> **Status:**") {
            return Some(clean_status(status));
        }
        if let Some(status) = trimmed.strip_prefix("**Status:**") {
            return Some(clean_status(status));
        }
        if let Some(status) = trimmed.strip_prefix("status:") {
            return Some(clean_status(status));
        }
        if trimmed.eq_ignore_ascii_case("## Status") {
            for candidate in lines.iter().skip(index + 1) {
                let candidate = candidate.trim();
                if candidate.is_empty() {
                    continue;
                }
                return Some(clean_status(candidate));
            }
        }
    }
    None
}

fn initial_frontmatter_scalar_values<'a>(
    text: &'a str,
    field: &'a str,
) -> impl Iterator<Item = &'a str> {
    initial_yaml_frontmatter(text)
        .into_iter()
        .flat_map(move |frontmatter| frontmatter_scalar_values(frontmatter, field))
}

fn frontmatter_scalar_values<'a>(
    frontmatter: &'a str,
    field: &'a str,
) -> impl Iterator<Item = &'a str> {
    frontmatter.lines().filter_map(move |line| {
        let (key, value) = line.split_once(':')?;
        (key == field).then_some(value.trim())
    })
}

fn split_initial_yaml_frontmatter(text: &str) -> Option<(&str, &str)> {
    let rest = text
        .strip_prefix("---\r\n")
        .or_else(|| text.strip_prefix("---\n"))?;
    for delimiter in ["\r\n---\r\n", "\n---\n"] {
        if let Some(end) = rest.find(delimiter) {
            return Some((&rest[..end], &rest[end + delimiter.len()..]));
        }
    }
    rest.strip_suffix("\r\n---")
        .or_else(|| rest.strip_suffix("\n---"))
        .map(|frontmatter| (frontmatter, ""))
}

fn is_canonical_date_scalar(value: &str) -> bool {
    let value = value.trim();
    let bytes = value.as_bytes();
    let value = if matches!(bytes.first(), Some(b'"' | b'\'')) {
        if bytes.len() < 2 || bytes.last() != bytes.first() {
            return false;
        }
        &value[1..value.len() - 1]
    } else {
        if matches!(bytes.last(), Some(b'"' | b'\'')) {
            return false;
        }
        value
    };
    is_canonical_date(value)
}

fn is_canonical_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
    {
        return false;
    }
    let year = value[0..4].parse::<u16>().unwrap_or_default();
    let month = value[5..7].parse::<u8>().unwrap_or_default();
    let day = value[8..10].parse::<u8>().unwrap_or_default();
    if year == 0 {
        return false;
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) => {
            29
        }
        2 => 28,
        _ => return false,
    };
    day > 0 && day <= max_day
}

fn clean_status(raw: &str) -> String {
    raw.trim()
        .trim_matches(|c: char| c == '*' || c == '`' || c == '.')
        .split(['—', '-', '('])
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

fn section_headings(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(str::trim)
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_doc() -> AdrDocument {
        AdrDocument {
            path: "docs/decisions/ADR-0008-data-use-boundary.md".to_string(),
            text: "# ADR-0008: Data Use Boundary\n\n> **Status:** Accepted\n\n## Context\nA\n\n## Decision\nB\n\n## Consequences\nC\n".to_string(),
        }
    }

    fn document(path: &str, text: &str) -> AdrDocument {
        AdrDocument {
            path: path.to_owned(),
            text: text.to_owned(),
        }
    }

    #[test]
    fn accepts_core_shape() {
        let report = validate_adr_shape_fitness(&[valid_doc()]).unwrap();
        assert_eq!(report.adrs_checked, 1);
    }

    #[test]
    fn accepts_frontmatter_before_title() {
        let mut doc = valid_doc();
        doc.text = format!("---\nstatus: Accepted\n---\n\n{}", doc.text);
        assert!(validate_adr_shape_fitness(&[doc]).is_ok());
    }

    #[test]
    fn accepts_amended_frontmatter_with_date() {
        let mut doc = valid_doc();
        doc.text = format!(
            "---\nstatus: Amended\namended_date: 2026-07-22\n---\n\n{}",
            doc.text
        );
        assert!(validate_adr_shape_fitness(&[doc]).is_ok());
    }

    #[test]
    fn rejects_amended_date_with_mismatched_quotes() {
        let mut doc = valid_doc();
        doc.text = format!(
            "---\nstatus: Amended\namended_date: \"2026-07-22'\n---\n\n{}",
            doc.text
        );
        assert!(matches!(
            validate_adr_shape_fitness(&[doc]),
            Err(AdrShapeFitnessError::MissingAmendedDate { .. })
        ));
    }

    #[test]
    fn accepts_amended_frontmatter_with_crlf_delimiters() {
        let mut doc = valid_doc();
        doc.text = format!(
            "---\r\nstatus: Amended\r\namended_date: 2026-07-22\r\n---\r\n\r\n{}",
            doc.text.replace('\n', "\r\n")
        );
        assert!(validate_adr_shape_fitness(&[doc]).is_ok());
    }

    #[test]
    fn recognizes_exact_live_statuses_including_legacy_forms() {
        for status in ["Accepted", "accepted", "Accepted (amendment)", "Amended"] {
            assert!(
                is_live_decision_status(status),
                "{status} should remain live"
            );
        }
        for status in [
            "Accepted amendment",
            "accepted (amendment)",
            "Amended (2026-07-22)",
        ] {
            assert!(
                !is_live_decision_status(status),
                "{status} must not be accepted by a broad match"
            );
        }
    }

    #[test]
    fn rejects_amended_frontmatter_without_date() {
        let mut doc = valid_doc();
        doc.text = format!("---\nstatus: Amended\n---\n\n{}", doc.text);
        assert!(matches!(
            validate_adr_shape_fitness(&[doc]),
            Err(AdrShapeFitnessError::MissingAmendedDate { .. })
        ));
    }

    #[test]
    fn rejects_amended_date_outside_initial_frontmatter() {
        let mut doc = valid_doc();
        doc.text = format!(
            "---\nstatus: Amended\n---\n\n```yaml\namended_date: 2026-07-22\n```\n\n{}",
            doc.text
        );
        assert!(matches!(
            validate_adr_shape_fitness(&[doc]),
            Err(AdrShapeFitnessError::MissingAmendedDate { .. })
        ));
    }

    #[test]
    fn rejects_noncanonical_amended_dates() {
        for amended_date in ["2026-7-22", "2026-02-30", "0000-01-01"] {
            let mut doc = valid_doc();
            doc.text = format!(
                "---\nstatus: Amended\namended_date: {amended_date}\n---\n\n{}",
                doc.text
            );
            assert!(matches!(
                validate_adr_shape_fitness(&[doc]),
                Err(AdrShapeFitnessError::MissingAmendedDate { .. })
            ));
        }
    }

    #[test]
    fn rejects_duplicate_amended_dates_even_when_one_is_valid() {
        let mut doc = valid_doc();
        doc.text = format!(
            "---\nstatus: Amended\namended_date: 2026-07-22\namended_date: 2026-02-30\n---\n\n{}",
            doc.text
        );
        assert!(matches!(
            validate_adr_shape_fitness(&[doc]),
            Err(AdrShapeFitnessError::MissingAmendedDate { .. })
        ));
    }

    #[test]
    fn rejects_conflicting_duplicate_frontmatter_status_keys() {
        let mut doc = valid_doc();
        doc.text = format!(
            "---\nstatus: Amended\nstatus: Accepted\namended_date: 2026-07-22\n---\n\n{}",
            doc.text
        );
        assert!(matches!(
            validate_adr_shape_fitness(&[doc]),
            Err(AdrShapeFitnessError::InvalidStatus { .. })
        ));
    }

    #[test]
    fn nested_frontmatter_fields_do_not_establish_lifecycle_authority() {
        let mut doc = valid_doc();
        doc.text = doc.text.replace("> **Status:** Accepted\n", "");
        doc.text = format!(
            "---\nmetadata:\n  status: Amended\n  amended_date: 2026-07-22\n---\n\n{}",
            doc.text
        );
        assert!(matches!(
            validate_adr_shape_fitness(&[doc]),
            Err(AdrShapeFitnessError::MissingStatus { .. })
        ));
    }

    #[test]
    fn accepts_status_section() {
        let mut doc = valid_doc();
        doc.text = doc.text.replace(
            "> **Status:** Accepted",
            "## Status\n\nAccepted (2026-05-14).",
        );
        assert!(validate_adr_shape_fitness(&[doc]).is_ok());
    }

    #[test]
    fn rejects_invalid_status() {
        let mut doc = valid_doc();
        doc.text = doc.text.replace("Accepted", "Maybe");
        assert!(matches!(
            validate_adr_shape_fitness(&[doc]),
            Err(AdrShapeFitnessError::InvalidStatus { .. })
        ));
    }

    #[test]
    fn lifecycle_statuses_match_adr_0388() {
        let mut rejected = valid_doc();
        rejected.text = rejected.text.replace("Accepted", "Rejected");
        assert!(validate_adr_shape_fitness(&[rejected]).is_ok());

        let mut retracted = valid_doc();
        retracted.text = retracted.text.replace("Accepted", "Retracted");
        assert!(matches!(
            validate_adr_shape_fitness(&[retracted]),
            Err(AdrShapeFitnessError::InvalidStatus { .. })
        ));
    }

    #[test]
    fn rejects_missing_consequences() {
        let mut doc = valid_doc();
        doc.text = doc.text.replace("\n## Consequences\nC\n", "");
        assert!(matches!(
            validate_adr_shape_fitness(&[doc]),
            Err(AdrShapeFitnessError::MissingSection {
                section: "Consequences",
                ..
            })
        ));
    }

    #[test]
    fn rejects_out_of_order_sections() {
        let mut doc = valid_doc();
        doc.text = "# ADR-0008: Data Use Boundary\n\n> **Status:** Accepted\n\n## Decision\nB\n\n## Context\nA\n\n## Consequences\nC\n".to_string();
        assert!(matches!(
            validate_adr_shape_fitness(&[doc]),
            Err(AdrShapeFitnessError::SectionsOutOfOrder { .. })
        ));
    }

    #[test]
    fn diagnostic_is_deterministic_for_reversed_input_order() {
        let valid = document(
            "docs/decisions/ADR-9001-valid.md",
            "# ADR-9001: Record the diagnostic boundary\n\n> **Status:** Proposed\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- Determinism.\n\n## Consequences\nC\n",
        );
        let malformed = document(
            "docs/decisions/ADR-9002-malformed.md",
            "    # ADR-9002: Pseudo ADR\n\n    ## Context\n",
        );

        let forward = audit_adr_shape_fitness(&[valid.clone(), malformed.clone()]);
        let reverse = audit_adr_shape_fitness(&[malformed, valid]);

        assert_eq!(forward, reverse);
        assert!(
            forward
                .findings
                .iter()
                .any(|finding| finding.code == "ADR_SECTION_MISSING")
        );
    }

    #[test]
    fn diagnostic_rejects_malformed_and_misnested_structure_without_false_sections() {
        let report = audit_adr_shape_fitness(&[document(
            "docs/decisions/ADR-9003-structure.md",
            "# ADR-9003: Diagnose fenced headings\n\n> **Status:** Proposed\n\n```md\n## Context\n~~~\n## Decision\n```still-open\n## Consequences\n\n### Decision Drivers\n- misplaced\n",
        )]);

        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "ADR_SECTION_MISSING"
                    && finding.message.contains("Context"))
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| { finding.code == "ADR_DECISION_DRIVERS_MISNESTED_OR_MISSING" })
        );
    }

    #[test]
    fn diagnostic_reports_legacy_status_as_migration_inventory_not_live_acceptance() {
        let report = audit_adr_shape_fitness(&[document(
            "docs/decisions/ADR-9004-legacy-status.md",
            "# ADR-9004: Preserve status evidence\n\n> **Status:** accepted (historical)\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
        )]);

        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "ADR_STATUS_MIGRATION_INVENTORY")
        );
    }

    #[test]
    fn diagnostic_rejects_duplicate_and_misordered_headings() {
        let report = audit_adr_shape_fitness(&[document(
            "docs/decisions/ADR-9005-order.md",
            "# ADR-9005: Preserve structure\n\n> **Status:** Proposed\n\n## Decision\nB\n\n## Context\nA\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n\n## Context\nDuplicate\n",
        )]);

        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "ADR_SECTION_DUPLICATE")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "ADR_SECTION_OUT_OF_ORDER")
        );
    }

    #[test]
    fn diagnostic_does_not_treat_escaped_table_pipes_as_headings() {
        let report = audit_adr_shape_fitness(&[document(
            "docs/decisions/ADR-9006-escaped-pipe.md",
            "# ADR-9006: Preserve table boundaries\n\n> **Status:** Proposed\n\n| Field | Value |\n| --- | --- |\n| note | \\| ## Context |\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
        )]);

        assert!(report.findings.iter().any(|finding| {
            finding.code == "ADR_SECTION_MISSING" && finding.message.contains("Context")
        }));
    }

    #[test]
    fn diagnostic_inventories_missing_status_and_ignores_fenced_pseudo_status() {
        let missing = audit_adr_shape_fitness(&[document(
            "docs/decisions/ADR-9007-missing-status.md",
            "# ADR-9007: Missing status\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
        )]);
        assert!(
            missing
                .findings
                .iter()
                .any(|finding| finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY")
        );

        let fenced = audit_adr_shape_fitness(&[document(
            "docs/decisions/ADR-9008-fenced-status.md",
            "# ADR-9008: Fenced status\n\n```md\n> **Status:** Accepted\n```\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
        )]);
        assert!(
            fenced
                .findings
                .iter()
                .any(|finding| finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY")
        );

        let block_literal = audit_adr_shape_fitness(&[document(
            "docs/decisions/ADR-9009-block-status.md",
            "---\nnotes: |\n  > **Status:** Accepted\n---\n\n# ADR-9009: Block status\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
        )]);
        assert!(
            block_literal
                .findings
                .iter()
                .any(|finding| finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY")
        );
    }

    #[test]
    fn diagnostic_uses_visible_body_status_when_frontmatter_omits_status() {
        let report = audit_adr_shape_fitness(&[document(
            "docs/decisions/ADR-9010-frontmatter-body-status.md",
            "---\nid: ADR-9010\n---\n\n# ADR-9010: Body status fallback\n\n> **Status:** Accepted\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
        )]);

        assert!(
            !report
                .findings
                .iter()
                .any(|finding| { finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY" })
        );
    }

    #[test]
    fn diagnostic_ignores_status_inside_block_quoted_fences() {
        let report = audit_adr_shape_fitness(&[document(
            "docs/decisions/ADR-9011-quoted-fence.md",
            "# ADR-9011: Quoted fence\n\n> ```md\n> **Status:** Accepted\n> ```\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
        )]);

        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY")
        );
    }

    #[test]
    fn diagnostic_does_not_promote_fully_quoted_pseudo_adr_sections() {
        let report = audit_adr_shape_fitness(&[document(
            "docs/decisions/ADR-9012-quoted-pseudo-adr.md",
            "> # ADR-9012: Quoted pseudo ADR\n>\n> **Status:** Proposed\n>\n> ## Context\n> A\n>\n> ## Decision\n> B\n>\n> ## Decision Drivers\n> - C\n>\n> ## Consequences\n> D\n",
        )]);

        for section in ["Context", "Decision", "Decision Drivers", "Consequences"] {
            assert!(report.findings.iter().any(|finding| {
                finding.code == "ADR_SECTION_MISSING"
                    && finding.message == format!("missing ## {section}")
            }));
        }
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY")
        );
    }
}
