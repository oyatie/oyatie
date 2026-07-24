//! ADR shape fitness kernel.
//!
//! Pure validator for the minimal ADR shape currently enforced by the M01-P01
//! Data Use Boundary acceptance gate. It validates title, status, and the core
//! Context -> Decision -> Consequences section order without filesystem I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
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

/// Deterministic, non-admissible migration diagnostic. data_class: INTERNAL_ONLY
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AdrShapeFinding {
    pub path: String,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrShapeAuditReport {
    pub adrs_checked: usize,            // data_class: INTERNAL_ONLY
    pub findings: Vec<AdrShapeFinding>, // data_class: INTERNAL_ONLY
}

const FRONTMATTER_FIELDS: [&str; 9] = [
    "id",
    "title",
    "status",
    "date",
    "supersedes",
    "superseded_by",
    "owner",
    "related",
    "bominal_source",
];

/// Parse the published ADR Markdown shape and collect every migration finding.
/// This is diagnostic only: callers must not treat a clean report as admission authority.
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
struct MarkdownSection<'a> {
    level: usize,
    title: &'a str,
    line_start: usize,
    start: usize,
    end: usize,
}

fn markdown_sections(text: &str) -> Vec<MarkdownSection<'_>> {
    let mut headings = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let leading_spaces = line.bytes().take_while(|byte| *byte == b' ').count();
        if leading_spaces > 3 || line.as_bytes().get(leading_spaces) == Some(&b'\t') {
            offset += line.len();
            continue;
        }
        let trimmed = &line[leading_spaces..];
        let marker = trimmed.chars().next();
        let marker_len = marker.map_or(0, |marker| {
            trimmed
                .chars()
                .take_while(|character| *character == marker)
                .count()
        });
        if let Some((open_marker, open_len)) = fence {
            if marker == Some(open_marker)
                && marker_len >= open_len
                && trimmed[marker_len..].trim().is_empty()
            {
                fence = None;
            }
            offset += line.len();
            continue;
        }
        if matches!(marker, Some('`' | '~')) && marker_len >= 3 {
            fence = marker.map(|marker| (marker, marker_len));
            offset += line.len();
            continue;
        }
        {
            let hashes = trimmed
                .chars()
                .take_while(|character| *character == '#')
                .count();
            if (1..=6).contains(&hashes) && trimmed.as_bytes().get(hashes) == Some(&b' ') {
                headings.push(MarkdownSection {
                    level: hashes,
                    title: trimmed[hashes + 1..].trim(),
                    line_start: offset,
                    start: offset + line.len(),
                    end: text.len(),
                });
            }
        }
        offset += line.len();
    }
    for index in 0..headings.len() {
        let level = headings[index].level;
        headings[index].end = headings[index + 1..]
            .iter()
            .find(|next| next.level <= level)
            .map_or(text.len(), |next| next.line_start);
    }
    headings
}

fn table_columns(line: &str) -> Vec<String> {
    line.replace("\\|", "¦")
        .split('|')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.replace('¦', "|"))
        .collect()
}

fn audit_one(adr: &AdrDocument) -> Vec<AdrShapeFinding> {
    let sections = markdown_sections(&adr.text);
    let mut findings = Vec::new();
    let add = |findings: &mut Vec<AdrShapeFinding>, code, message: String| {
        findings.push(AdrShapeFinding {
            path: adr.path.clone(),
            code,
            message,
        })
    };
    let table = sections
        .iter()
        .find(|section| section.level == 2 && section.title.eq_ignore_ascii_case("Frontmatter"));
    let mut fields = BTreeSet::<String>::new();
    if let Some(section) = table {
        for line in adr.text[section.start..section.end].lines() {
            let columns = table_columns(line);
            if let [field, value] = columns.as_slice() {
                let field = field.trim_matches('*');
                if FRONTMATTER_FIELDS.contains(&field) {
                    let sentinel =
                        matches!(field, "supersedes" | "superseded_by" | "related") && value == "-";
                    if !sentinel && value.trim().is_empty() {
                        add(
                            &mut findings,
                            "ADR_FRONTMATTER_EMPTY",
                            format!("frontmatter field {field} is empty"),
                        );
                    }
                    if !fields.insert(field.to_owned()) {
                        add(
                            &mut findings,
                            "ADR_FRONTMATTER_DUPLICATE",
                            format!("frontmatter field {field} is duplicated"),
                        );
                    }
                }
            }
        }
    } else {
        add(
            &mut findings,
            "ADR_FRONTMATTER_MISSING",
            "missing required ## Frontmatter table".to_owned(),
        );
    }
    for field in FRONTMATTER_FIELDS {
        if !fields.contains(field) {
            add(
                &mut findings,
                "ADR_FRONTMATTER_FIELD_MISSING",
                format!("missing frontmatter field {field}"),
            );
        }
    }
    if let Some(section) = table {
        let status = adr.text[section.start..section.end]
            .lines()
            .find_map(|line| {
                let columns = table_columns(line);
                match columns.as_slice() {
                    [field, value] if field.trim_matches('*') == "status" => Some(value.clone()),
                    _ => None,
                }
            });
        if let Some(status) = status
            && !matches!(
                status.as_str(),
                "Accepted"
                    | "accepted"
                    | "Proposed"
                    | "proposed"
                    | "Amended"
                    | "Superseded"
                    | "Deprecated"
                    | "Rejected"
                    | "Accepted (amendment)"
            )
        {
            add(
                &mut findings,
                "ADR_STATUS_INVALID",
                format!("unsupported lifecycle status {status:?}"),
            );
        }
    }
    for title in [
        "Context",
        "Decision",
        "Consequences",
        "Clean Architecture Impact",
        "Alternatives Considered",
        "References",
    ] {
        if !sections
            .iter()
            .any(|section| section.level == 2 && section.title.eq_ignore_ascii_case(title))
        {
            add(
                &mut findings,
                "ADR_SECTION_MISSING",
                format!("missing ## {title}"),
            );
        }
    }
    let mut previous = None;
    for title in [
        "Context",
        "Decision",
        "Consequences",
        "Clean Architecture Impact",
        "Alternatives Considered",
        "References",
    ] {
        let position = sections
            .iter()
            .position(|section| section.level == 2 && section.title.eq_ignore_ascii_case(title));
        if let (Some(previous), Some(position)) = (previous, position)
            && position < previous
        {
            add(
                &mut findings,
                "ADR_SECTION_OUT_OF_ORDER",
                format!("## {title} appears out of canonical order"),
            );
        }
        if let Some(position) = position {
            previous = Some(position);
        }
    }
    if let Some(consequences) = sections
        .iter()
        .find(|section| section.level == 2 && section.title.eq_ignore_ascii_case("Consequences"))
    {
        for title in [
            "Concrete file and crate changes",
            "Integration via Workflow + Ontology",
            "Positive",
            "Negative",
            "Operational",
        ] {
            let nested = sections.iter().any(|section| {
                section.level == 3
                    && section.start >= consequences.start
                    && section.start < consequences.end
                    && section.title.eq_ignore_ascii_case(title)
            });
            if !nested {
                add(
                    &mut findings,
                    "ADR_CONSEQUENCE_MISNESTED_OR_MISSING",
                    format!("missing ### {title} under ## Consequences"),
                );
            }
        }
    }
    if let Some(impact) = sections.iter().find(|section| {
        section.level == 2
            && section
                .title
                .eq_ignore_ascii_case("Clean Architecture Impact")
    }) {
        let body = &adr.text[impact.start..impact.end];
        for lane in CLEAN_ARCHITECTURE_LANES {
            if !body.lines().any(|line| {
                let columns = table_columns(line);
                columns.len() == 3
                    && columns[0]
                        .trim_start_matches('`')
                        .split('`')
                        .next()
                        .is_some_and(|label| label == lane)
            }) {
                add(
                    &mut findings,
                    "ADR_CLEAN_ARCHITECTURE_LANE_MISSING",
                    format!("missing Clean Architecture row {lane}"),
                );
            }
        }
    }
    if let Some(alternatives) = sections.iter().find(|section| {
        section.level == 2
            && section
                .title
                .eq_ignore_ascii_case("Alternatives Considered")
    }) {
        let items: Vec<_> = sections
            .iter()
            .filter(|section| {
                section.level == 3
                    && section.start >= alternatives.start
                    && section.start < alternatives.end
            })
            .collect();
        if items.is_empty() {
            let body = &adr.text[alternatives.start..alternatives.end];
            let mut current = None;
            let mut fields = BTreeSet::new();
            for line in body.lines().chain(std::iter::once("**END**")) {
                if line.trim_start().starts_with("**Alternative") {
                    if let Some(name) = current.take() {
                        for field in ["Description", "Pros", "Cons", "Reason rejected"] {
                            if !fields.contains(field) {
                                add(
                                    &mut findings,
                                    "ADR_ALTERNATIVE_FIELD_MISSING",
                                    format!("alternative {name:?} is missing {field}"),
                                );
                            }
                        }
                    }
                    current = Some(line.trim().to_owned());
                    fields.clear();
                } else if current.is_some() {
                    for field in ["Description", "Pros", "Cons", "Reason rejected"] {
                        if line.trim_start_matches(['-', '*', ' ']).starts_with(field) {
                            fields.insert(field);
                        }
                    }
                }
            }
            if let Some(name) = current {
                for field in ["Description", "Pros", "Cons", "Reason rejected"] {
                    if !fields.contains(field) {
                        add(
                            &mut findings,
                            "ADR_ALTERNATIVE_FIELD_MISSING",
                            format!("alternative {name:?} is missing {field}"),
                        );
                    }
                }
            } else {
                add(
                    &mut findings,
                    "ADR_ALTERNATIVE_MISSING",
                    "Alternatives Considered has no ### or bold Alternative item".to_owned(),
                );
            }
        } else {
            for item in items {
                let body = &adr.text[item.start..item.end];
                for field in ["Description", "Pros", "Cons", "Reason rejected"] {
                    if !body
                        .lines()
                        .any(|line| line.trim_start_matches(['-', '*', ' ']).starts_with(field))
                    {
                        add(
                            &mut findings,
                            "ADR_ALTERNATIVE_FIELD_MISSING",
                            format!("alternative {:?} is missing {field}", item.title),
                        );
                    }
                }
            }
        }
    }
    findings
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
    MalformedFrontmatter {
        path: String,
    },
    DuplicateFrontmatterField {
        path: String,
        field: String,
    },
    MissingBominalDeclaration {
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
    MissingCleanArchitectureLane {
        path: String,
        lane: &'static str,
    },
    MalformedAlternative {
        path: String,
        missing: &'static str,
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
            Self::MalformedFrontmatter { path } => {
                write!(f, "{path}: malformed initial YAML frontmatter")
            }
            Self::DuplicateFrontmatterField { path, field } => {
                write!(f, "{path}: duplicate initial-frontmatter field {field:?}")
            }
            Self::MissingBominalDeclaration { path } => {
                write!(
                    f,
                    "{path}: missing Bominal inheritance-or-override declaration"
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
            Self::MissingCleanArchitectureLane { path, lane } => {
                write!(f, "{path}: missing Clean Architecture Impact lane {lane:?}")
            }
            Self::MalformedAlternative { path, missing } => {
                write!(f, "{path}: alternative is missing required {missing}")
            }
        }
    }
}

impl std::error::Error for AdrShapeFitnessError {}

const REQUIRED_SECTIONS: [&str; 11] = [
    "Context",
    "Decision",
    "Consequences",
    "Concrete file and crate changes",
    "Integration via Workflow + Ontology",
    "Positive",
    "Negative",
    "Operational",
    "Clean Architecture Impact",
    "Alternatives Considered",
    "References",
];
const ORDERED_SECTIONS: [&str; 5] = [
    "Context",
    "Decision",
    "Consequences",
    "Alternatives Considered",
    "References",
];
const CLEAN_ARCHITECTURE_LANES: [&str; 6] = [
    "dependency-direction",
    "cross-product-refusal",
    "port-location",
    "layer-correctness",
    "composition-root-only",
    "sdk-kernel-only",
];
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
    let Some(_title) = adr.text.lines().find(|line| line.starts_with("# ADR-")) else {
        return Err(AdrShapeFitnessError::MissingTitle {
            path: adr.path.clone(),
        });
    };
    let frontmatter = validated_frontmatter(adr)?;
    let status = extract_status(&adr.text, frontmatter).ok_or_else(|| {
        AdrShapeFitnessError::MissingStatus {
            path: adr.path.clone(),
        }
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

    if !has_bominal_declaration(frontmatter, &adr.text) {
        return Err(AdrShapeFitnessError::MissingBominalDeclaration {
            path: adr.path.clone(),
        });
    }

    let headings = section_headings(&adr.text);
    for section in REQUIRED_SECTIONS {
        if !headings
            .iter()
            .any(|heading| heading.eq_ignore_ascii_case(section))
        {
            return Err(AdrShapeFitnessError::MissingSection {
                path: adr.path.clone(),
                section,
            });
        }
    }
    let mut previous_index = None;
    let mut previous_section = None;
    for section in ORDERED_SECTIONS {
        let index = headings
            .iter()
            .position(|heading| heading.eq_ignore_ascii_case(section))
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
    validate_clean_architecture_lanes(adr, &headings)?;
    validate_alternatives(adr, &headings)?;
    Ok(())
}

fn validated_frontmatter<'a>(
    adr: &'a AdrDocument,
) -> Result<Option<&'a str>, AdrShapeFitnessError> {
    let Some(frontmatter) = initial_yaml_frontmatter(&adr.text) else {
        if adr.text.starts_with("---\n") || adr.text.starts_with("---\r\n") {
            return Err(AdrShapeFitnessError::MalformedFrontmatter {
                path: adr.path.clone(),
            });
        }
        return Ok(None);
    };
    let mut fields = BTreeSet::new();
    for line in frontmatter
        .lines()
        .filter(|line| !line.starts_with(char::is_whitespace))
    {
        let Some((field, _)) = line.split_once(':') else {
            continue;
        };
        if !fields.insert(field) {
            return Err(AdrShapeFitnessError::DuplicateFrontmatterField {
                path: adr.path.clone(),
                field: field.to_owned(),
            });
        }
    }
    Ok(Some(frontmatter))
}

fn has_bominal_declaration(frontmatter: Option<&str>, text: &str) -> bool {
    frontmatter
        .into_iter()
        .flat_map(|frontmatter| frontmatter_scalar_values(frontmatter, "bominal_source"))
        .any(|value| !value.is_empty())
        || text
            .lines()
            .any(|line| line.trim_start().starts_with("| **bominal_source** |"))
}

fn validate_clean_architecture_lanes(
    adr: &AdrDocument,
    _headings: &[String],
) -> Result<(), AdrShapeFitnessError> {
    let section_text = section_body(&adr.text, "Clean Architecture Impact");
    for lane in CLEAN_ARCHITECTURE_LANES {
        if !section_text.contains(lane) {
            return Err(AdrShapeFitnessError::MissingCleanArchitectureLane {
                path: adr.path.clone(),
                lane,
            });
        }
    }
    Ok(())
}

fn validate_alternatives(
    adr: &AdrDocument,
    _headings: &[String],
) -> Result<(), AdrShapeFitnessError> {
    let alternatives = section_body(&adr.text, "Alternatives Considered");
    for required in ["Description", "Pros", "Cons", "Reason rejected"] {
        if !alternatives.lines().any(|line| {
            line.trim_start_matches(['-', '*', ' '])
                .starts_with(required)
        }) {
            return Err(AdrShapeFitnessError::MalformedAlternative {
                path: adr.path.clone(),
                missing: required,
            });
        }
    }
    Ok(())
}

fn section_body<'a>(text: &'a str, section: &str) -> &'a str {
    markdown_sections(text)
        .into_iter()
        .find(|heading| heading.level == 2 && heading.title.eq_ignore_ascii_case(section))
        .map_or("", |heading| &text[heading.start..heading.end])
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

fn extract_status(text: &str, frontmatter: Option<&str>) -> Option<String> {
    let lifecycle_surface = if let Some(frontmatter) = frontmatter {
        if let Some(status) = frontmatter_scalar_values(frontmatter, "status").next() {
            return Some(clean_status(status));
        }
        return None;
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
    markdown_sections(text)
        .into_iter()
        .filter(|heading| matches!(heading.level, 2 | 3))
        .map(|heading| heading.title.trim().to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_doc() -> AdrDocument {
        AdrDocument {
            path: "docs/decisions/ADR-0008-data-use-boundary.md".to_string(),
            text: "# ADR-0008: Enforce Data Use Boundary\n\n## Frontmatter\n| **bominal_source** | no Bominal equivalent |\n\n> **Status:** Accepted\n\n## Context\nA\n\n## Decision\nB\n\n## Consequences\n\n### Concrete file and crate changes\n| Path / Crate | Change type | BNF v4.1 name | Layer |\n| --- | --- | --- | --- |\n| `libs/example/` | update | `oya-example-kernel` | kernel |\n\n### Integration via Workflow + Ontology\nNot applicable; the integration point is documented in the affected service PRD.\n\n### Positive\n- A\n\n### Negative\n- B\n\n### Operational\n- C\n\n## Clean Architecture Impact\n| Lane | Impact | Action required |\n| --- | --- | --- |\n| `dependency-direction` | Not affected | none |\n| `cross-product-refusal` | Not affected | none |\n| `port-location` | Not affected | none |\n| `layer-correctness` | Not affected | none |\n| `composition-root-only` | Not affected | none |\n| `sdk-kernel-only` | Not affected | none |\n\n## Alternatives Considered\n### Alternative 1 — Existing parser\n- Description: A\n- Pros: B\n- Cons: C\n- Reason rejected: D\n\n## References\n- ADR-0056\n".to_string(),
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
            Err(AdrShapeFitnessError::DuplicateFrontmatterField { .. })
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
            Err(AdrShapeFitnessError::DuplicateFrontmatterField { .. })
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
        doc.text = doc.text.replace("## Consequences\n\n", "");
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
        let context = "## Context\nA\n\n";
        let decision = "## Decision\nB\n\n";
        doc.text = doc.text.replace(context, "").replace(decision, "");
        doc.text = doc.text.replace(
            "## Consequences\n",
            &format!("{decision}{context}## Consequences\n"),
        );
        assert!(matches!(
            validate_adr_shape_fitness(&[doc]),
            Err(AdrShapeFitnessError::SectionsOutOfOrder { .. })
        ));
    }
}

#[cfg(test)]
mod template_contract_tests;
