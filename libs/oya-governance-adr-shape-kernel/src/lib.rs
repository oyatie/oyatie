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
}
