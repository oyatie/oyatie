//! Foundry ADR index generation fitness kernel.
//!
//! The [`AdrDecisionIr`] renderer in this crate is a dormant foundation, not
//! the admitted ADR-index production path. The current decision corpus is not
//! yet fully parseable by the strict IR, and existing consumers still use the
//! transitional [`AdrDecisionRecord`] adapter. A successfully parsed subset
//! must never be rendered or described as the authoritative index. Planning
//! remains on HOLD until corpus population, consumer cutover, projection
//! parity, and independent admission gates are satisfied.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use corpus_doc_parser::AdrDecisionIr;

/// Transitional projection retained for callers that have not yet adopted the
/// canonical parser IR. New producers should pass [`AdrDecisionIr`] through
/// [`generate_adr_index_from_ir`] instead of parsing or assembling this record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdrDecisionRecord {
    pub number: u16,                // data_class: INTERNAL_ONLY
    pub id: String,                 // data_class: INTERNAL_ONLY
    pub title: String,              // data_class: INTERNAL_ONLY
    pub status: String,             // data_class: INTERNAL_ONLY
    pub owner: String,              // data_class: INTERNAL_ONLY
    pub date: String,               // data_class: INTERNAL_ONLY
    pub path: String,               // data_class: INTERNAL_ONLY
    pub supersedes: Vec<String>,    // data_class: INTERNAL_ONLY
    pub superseded_by: Vec<String>, // data_class: INTERNAL_ONLY
    pub related: Vec<String>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdrIndexArtifacts {
    pub markdown: String,       // data_class: INTERNAL_ONLY
    pub json: String,           // data_class: INTERNAL_ONLY
    pub report: AdrIndexReport, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdrIndexReport {
    pub records: usize,                         // data_class: INTERNAL_ONLY
    pub next_adr: String,                       // data_class: INTERNAL_ONLY
    pub gaps: Vec<String>,                      // data_class: INTERNAL_ONLY
    pub status_counts: BTreeMap<String, usize>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdrIndexError {
    NoRecords,
    InvalidRecord { id: String, reason: String },
    DuplicateAdr { id: String },
    NonContiguousNumber { expected: u16, actual: u16 },
    MarkdownDrift,
    JsonDrift,
}

/// Render the stable ADR index faces from canonical parsed decisions.
///
/// This function performs no filesystem access and no metadata parsing. It
/// preserves lifecycle spelling and typed relation order from the parser IR.
/// It is not the live producer while the IR remains dormant: callers must not
/// pass a parseable subset of the current corpus as though it were complete.
pub fn generate_adr_index_from_ir<I>(decisions: I) -> Result<AdrIndexArtifacts, AdrIndexError>
where
    I: IntoIterator<Item = AdrDecisionIr>,
{
    let records = decisions
        .into_iter()
        .map(project_ir_record)
        .collect::<Result<Vec<_>, _>>()?;
    generate_adr_index(records)
}

/// Compare current index faces with deterministic rendering of canonical IR.
pub fn validate_adr_index_from_ir<I>(
    decisions: I,
    current_markdown: &str,
    current_json: &str,
) -> Result<AdrIndexReport, AdrIndexError>
where
    I: IntoIterator<Item = AdrDecisionIr>,
{
    let artifacts = generate_adr_index_from_ir(decisions)?;
    validate_rendered_artifacts(artifacts, current_markdown, current_json)
}

/// Compatibility renderer for existing record-producing consumers.
///
/// Canonical ADR parsing belongs to `corpus-doc-parser`; this adapter remains
/// compile-compatible while those out-of-crate consumers migrate to the IR.
pub fn generate_adr_index<I>(records: I) -> Result<AdrIndexArtifacts, AdrIndexError>
where
    I: IntoIterator<Item = AdrDecisionRecord>,
{
    let records = normalized_records(records)?;
    let Some(last_record) = records.last() else {
        return Err(AdrIndexError::NoRecords);
    };
    let status_counts = status_counts(&records);
    let gaps = adr_number_gaps(&records);
    let next_adr = format!("ADR-{:04}", last_record.number + 1);
    let report = AdrIndexReport {
        records: records.len(),
        next_adr: next_adr.clone(),
        gaps,
        status_counts,
    };
    Ok(AdrIndexArtifacts {
        markdown: render_markdown(&records, &report),
        json: render_json(&records, &report),
        report,
    })
}

pub fn validate_adr_index<I>(
    records: I,
    current_markdown: &str,
    current_json: &str,
) -> Result<AdrIndexReport, AdrIndexError>
where
    I: IntoIterator<Item = AdrDecisionRecord>,
{
    let artifacts = generate_adr_index(records)?;
    validate_rendered_artifacts(artifacts, current_markdown, current_json)
}

fn validate_rendered_artifacts(
    artifacts: AdrIndexArtifacts,
    current_markdown: &str,
    current_json: &str,
) -> Result<AdrIndexReport, AdrIndexError> {
    if normalize_text(current_markdown) != normalize_text(&artifacts.markdown) {
        return Err(AdrIndexError::MarkdownDrift);
    }
    if normalize_text(current_json) != normalize_text(&artifacts.json) {
        return Err(AdrIndexError::JsonDrift);
    }
    Ok(artifacts.report)
}

fn project_ir_record(decision: AdrDecisionIr) -> Result<AdrDecisionRecord, AdrIndexError> {
    let Some(index_path) = decision.source_path().strip_prefix("docs/") else {
        return Err(AdrIndexError::InvalidRecord {
            id: decision.id().as_str().to_owned(),
            reason: "canonical IR source path must start with docs/".to_owned(),
        });
    };
    Ok(AdrDecisionRecord {
        number: decision.id().number(),
        id: decision.id().as_str().to_owned(),
        title: decision.title().to_owned(),
        status: decision.status().to_owned(),
        owner: decision.owner().to_owned(),
        date: decision.date().to_owned(),
        path: index_path.to_owned(),
        supersedes: relation_ids(decision.supersedes()),
        superseded_by: relation_ids(decision.superseded_by()),
        related: relation_ids(decision.related()),
    })
}

fn relation_ids(references: &[corpus_doc_parser::AdrReference]) -> Vec<String> {
    references
        .iter()
        .map(|reference| reference.id().as_str().to_owned())
        .collect()
}

fn normalized_records<I>(records: I) -> Result<Vec<AdrDecisionRecord>, AdrIndexError>
where
    I: IntoIterator<Item = AdrDecisionRecord>,
{
    let mut sorted = records.into_iter().collect::<Vec<_>>();
    sorted.sort_by_key(|record| record.number);
    let mut seen = BTreeSet::new();
    for record in &sorted {
        validate_record(record)?;
        if !seen.insert(record.id.clone()) {
            return Err(AdrIndexError::DuplicateAdr {
                id: record.id.clone(),
            });
        }
    }
    Ok(sorted)
}

fn validate_record(record: &AdrDecisionRecord) -> Result<(), AdrIndexError> {
    let expected_id = format!("ADR-{:04}", record.number);
    if record.id != expected_id {
        return Err(AdrIndexError::InvalidRecord {
            id: record.id.clone(),
            reason: format!("id must match number {expected_id}"),
        });
    }
    for (field, value) in [
        ("title", &record.title),
        ("status", &record.status),
        ("owner", &record.owner),
        ("date", &record.date),
        ("path", &record.path),
    ] {
        if value.trim().is_empty() {
            return Err(AdrIndexError::InvalidRecord {
                id: record.id.clone(),
                reason: format!("{field} must be non-empty"),
            });
        }
    }
    if !record.path.starts_with("decisions/ADR-") || !record.path.ends_with(".md") {
        return Err(AdrIndexError::InvalidRecord {
            id: record.id.clone(),
            reason: "path must point at decisions/ADR-*.md".into(),
        });
    }
    if !record.path.contains(&record.id) {
        return Err(AdrIndexError::InvalidRecord {
            id: record.id.clone(),
            reason: "path must contain ADR id".into(),
        });
    }
    Ok(())
}

fn status_counts(records: &[AdrDecisionRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for record in records {
        *counts.entry(record.status.clone()).or_insert(0) += 1;
    }
    counts
}

fn render_markdown(records: &[AdrDecisionRecord], report: &AdrIndexReport) -> String {
    let (Some(first), Some(last)) = (records.first(), records.last()) else {
        return String::new();
    };
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(
        "purpose: Generated ADR index and machine-readable mirror pointer for ADR freshness, numbering, owner, status, and supersession review.\n",
    );
    out.push_str("doc_status: published\n");
    out.push_str("---\n\n");
    out.push_str("# Oyatie — ADR Index\n\n");
    out.push_str("> **Generated:** from [`decisions/`](decisions/) by `oya doc adr-index`. Do not hand-edit generated rows.\n");
    out.push_str("> **Authoritative:** `crew-adr-promotion` owns freshness per [DOC-CATALOG.md `doc.adr_index`](DOC-CATALOG.md).\n");
    out.push_str("> **Machine-readable mirror:** [`machine-readable/decisions.json`](machine-readable/decisions.json).\n\n");
    out.push_str("## At-a-glance\n\n");
    out.push_str(&format!("- **Total ADRs:** {}\n", report.records));
    out.push_str(&format!(
        "- **Numbering:** {}\n",
        numbering_summary(
            &first.id,
            &last.id,
            &report.gaps,
            GapRenderMode::CitationSafe
        )
    ));
    // ADR citation policy treats `ADR-NNNN` tokens in active docs as claims
    // that the decision file exists. The generated index therefore exposes the
    // next available number without rendering a future ADR citation.
    out.push_str(&format!(
        "- **Next ADR number:** {}\n",
        report.next_adr.trim_start_matches("ADR-")
    ));
    out.push_str(&format!(
        "- **Status counts:** {}\n",
        render_status_counts(&report.status_counts)
    ));
    out.push_str("- **Legacy retirement:** see [`ADR-LEGACY-REGRESSION-MAPPING.md`](ADR-LEGACY-REGRESSION-MAPPING.md).\n\n");
    out.push_str("## Full table (one row per ADR, sorted by ADR number)\n\n");
    out.push_str("| ADR | Status | Title | Owner | File |\n");
    out.push_str("|---|---|---|---|---|\n");
    for record in records {
        out.push_str(&format!(
            "| {} | {} | {} | {} | [`{}`]({}) |\n",
            record.id,
            markdown_cell(&record.status),
            markdown_title_cell(&record.title),
            markdown_cell(&record.owner),
            file_name(&record.path),
            record.path
        ));
    }
    out.push_str("\n## Update protocol\n\n");
    out.push_str(
        "- Per-event + monthly per `doc.adr_index` row in [`DOC-CATALOG.md`](DOC-CATALOG.md).\n",
    );
    out.push_str(&format!(
        "- New ADRs land via [`templates/adr-template.md`](templates/adr-template.md) and use the next available number ({}), unless an explicit reserved-number ADR is being filled.\n",
        report.next_adr.trim_start_matches("ADR-")
    ));
    out.push_str("- Per-ADR amendments preserve the original ADR number; the amended ADR cites its original date and links to the amending PR.\n");
    out.push_str(
        "- Supersession is recorded in the per-ADR header and mirrored here on regeneration.\n\n",
    );
    if !report.gaps.is_empty() {
        out.push_str("## Deleted / unassigned ADR numbers\n\n");
        out.push_str("The directory is intentionally non-contiguous. Every existing `docs/decisions/ADR-*.md` file is included in the table and machine-readable mirror; the following gaps are not counted as ADR files.\n\n");
        out.push_str("| Number range | Reason |\n");
        out.push_str("|---|---|\n");
        for gap in &report.gaps {
            let gap = citation_safe_gap(gap);
            out.push_str(&format!(
                "| {gap} | Not represented by a `docs/decisions/ADR-*.md` file; reserved, deleted, or retired. |\n"
            ));
        }
        out.push('\n');
    }
    out.push_str("## Sources scanned\n\n");
    out.push_str(&format!(
        "- `decisions/` directory listing — {} ADR files (sorted ascending)\n",
        report.records
    ));
    out.push_str("- [`machine-readable/decisions.json`](machine-readable/decisions.json) — generated machine mirror\n");
    out.push_str("- [`DOC-CATALOG.md`](DOC-CATALOG.md) — owner / cadence / dependent docs / validation checks\n");
    out
}

fn render_json(records: &[AdrDecisionRecord], report: &AdrIndexReport) -> String {
    let (Some(first), Some(last)) = (records.first(), records.last()) else {
        return String::new();
    };
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"_schema\": {\n");
    out.push_str("    \"version\": \"0.2\",\n");
    out.push_str("    \"description\": \"Machine-readable mirror of ADR-INDEX.md. Generated by oya doc adr-index.\",\n");
    out.push_str("    \"owner_team\": \"crew-adr-promotion\"\n");
    out.push_str("  },\n");
    out.push_str("  \"_metadata\": {\n");
    out.push_str("    \"source\": \"docs/decisions\",\n");
    out.push_str(&format!("    \"total_adrs\": {},\n", report.records));
    out.push_str(&format!(
        "    \"numbering\": \"{}\",\n",
        json_escape(&numbering_summary(
            &first.id,
            &last.id,
            &report.gaps,
            GapRenderMode::Canonical
        ))
    ));
    out.push_str(&format!("    \"gaps\": {},\n", json_array(&report.gaps)));
    out.push_str(&format!(
        "    \"next_adr\": \"{}\",\n",
        json_escape(&report.next_adr)
    ));
    out.push_str("    \"status_counts\": {");
    for (index, (status, count)) in report.status_counts.iter().enumerate() {
        if index == 0 {
            out.push('\n');
        } else {
            out.push_str(",\n");
        }
        out.push_str(&format!("      \"{}\": {}", json_escape(status), count));
    }
    if report.status_counts.is_empty() {
        out.push_str("}\n");
    } else {
        out.push_str("\n    }\n");
    }
    out.push_str("  },\n");
    out.push_str("  \"decisions\": [\n");
    for (index, record) in records.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!(
            "      \"adr\": \"{}\",\n",
            json_escape(&record.id)
        ));
        out.push_str(&format!("      \"number\": {},\n", record.number));
        out.push_str(&format!(
            "      \"title\": \"{}\",\n",
            json_escape(&record.title)
        ));
        out.push_str(&format!(
            "      \"status\": \"{}\",\n",
            json_escape(&record.status)
        ));
        out.push_str(&format!(
            "      \"owner\": \"{}\",\n",
            json_escape(&record.owner)
        ));
        out.push_str(&format!(
            "      \"date\": \"{}\",\n",
            json_escape(&record.date)
        ));
        out.push_str(&format!(
            "      \"path\": \"{}\",\n",
            json_escape(&record.path)
        ));
        out.push_str(&format!(
            "      \"supersedes\": {},\n",
            json_array(&record.supersedes)
        ));
        out.push_str(&format!(
            "      \"superseded_by\": {},\n",
            json_array(&record.superseded_by)
        ));
        out.push_str(&format!(
            "      \"related\": {}\n",
            json_array(&record.related)
        ));
        if index + 1 == records.len() {
            out.push_str("    }\n");
        } else {
            out.push_str("    },\n");
        }
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn render_status_counts(status_counts: &BTreeMap<String, usize>) -> String {
    status_counts
        .iter()
        .map(|(status, count)| format!("{status} {count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn adr_number_gaps(records: &[AdrDecisionRecord]) -> Vec<String> {
    let mut gaps = Vec::new();
    let Some(first) = records.first() else {
        return gaps;
    };
    let mut expected = first.number;
    for record in records {
        if record.number > expected {
            gaps.push(render_adr_range(expected, record.number - 1));
        }
        expected = record.number.saturating_add(1);
    }
    gaps
}

fn render_adr_range(start: u16, end: u16) -> String {
    if start == end {
        format!("ADR-{start:04}")
    } else {
        format!("ADR-{start:04}..ADR-{end:04}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GapRenderMode {
    Canonical,
    CitationSafe,
}

fn numbering_summary(
    first_id: &str,
    last_id: &str,
    gaps: &[String],
    gap_render_mode: GapRenderMode,
) -> String {
    if gaps.is_empty() {
        format!("contiguous {first_id}..{last_id} (gap-free)")
    } else {
        let gaps = match gap_render_mode {
            GapRenderMode::Canonical => gaps.join(", "),
            GapRenderMode::CitationSafe => gaps
                .iter()
                .map(|gap| citation_safe_gap(gap))
                .collect::<Vec<_>>()
                .join(", "),
        };
        format!("{first_id}..{last_id} (non-contiguous; gaps: {})", gaps)
    }
}

fn citation_safe_gap(gap: &str) -> String {
    gap.replace("ADR-", "")
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn markdown_title_cell(value: &str) -> String {
    let cell = markdown_cell(value);
    code_span_forbidden_glossary_tokens(&cell)
}

fn code_span_forbidden_glossary_tokens(value: &str) -> String {
    const TOKENS: &[&str] = &[
        "Closed-User-Group",
        "milestone-zero",
        "milestone-one",
        "MVP",
        "M0",
        "M1",
        "M2",
        "M3",
        "CUG",
    ];

    let mut output = String::new();
    let mut index = 0;
    while index < value.len() {
        let matched = TOKENS.iter().find_map(|token| {
            let end = index + token.len();
            let candidate = value.get(index..end)?;
            if candidate.eq_ignore_ascii_case(token)
                && has_glossary_token_boundaries(value, index, end)
            {
                Some(end)
            } else {
                None
            }
        });
        if let Some(end) = matched {
            output.push('`');
            output.push_str(&value[index..end]);
            output.push('`');
            index = end;
        } else if let Some(character) = value[index..].chars().next() {
            output.push(character);
            index += character.len_utf8();
        } else {
            break;
        }
    }
    output
}

fn has_glossary_token_boundaries(value: &str, start: usize, end: usize) -> bool {
    let previous = value[..start].chars().next_back();
    let next = value[end..].chars().next();
    !is_word_or_dash(previous) && !is_word_or_dash(next)
}

fn is_word_or_dash(character: Option<char>) -> bool {
    character.is_some_and(|character| character.is_ascii_alphanumeric() || character == '-')
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn json_array(values: &[String]) -> String {
    if values.is_empty() {
        return "[]".into();
    }
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

fn normalize_text(value: &str) -> String {
    value.replace("\r\n", "\n").trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_markdown_and_json_from_decision_records() {
        let artifacts = generate_adr_index([record(1, "Proposed"), record(2, "Accepted")])
            .expect("index generated");

        assert!(artifacts.markdown.contains("**Total ADRs:** 2"));
        assert!(artifacts.markdown.contains("Proposed 1"));
        assert!(artifacts.markdown.contains("Accepted 1"));
        assert!(artifacts.markdown.contains("**Next ADR number:** 0003"));
        assert!(!artifacts.markdown.contains("**Next ADR:** ADR-0003"));
        assert!(
            artifacts
                .markdown
                .contains("Decision 1 with `M0`/`M1`/`M2`/`M3`/`MVP` retired terms")
        );
        assert!(artifacts.json.contains("\"total_adrs\": 2"));
        assert!(artifacts.json.contains("\"next_adr\": \"ADR-0003\""));
        assert_eq!(artifacts.report.records, 2);
    }

    #[test]
    fn validates_matching_artifacts() {
        let records = [record(1, "Proposed"), record(2, "Accepted")];
        let artifacts = generate_adr_index(records.clone()).expect("index generated");

        let report = validate_adr_index(records, &artifacts.markdown, &artifacts.json)
            .expect("artifacts validate");

        assert_eq!(report.next_adr, "ADR-0003");
    }

    #[test]
    fn rejects_markdown_or_json_drift() {
        let records = [record(1, "Proposed")];
        let artifacts = generate_adr_index(records.clone()).expect("index generated");

        assert_eq!(
            validate_adr_index(records.clone(), "stale", &artifacts.json),
            Err(AdrIndexError::MarkdownDrift)
        );
        assert_eq!(
            validate_adr_index(records, &artifacts.markdown, "{}"),
            Err(AdrIndexError::JsonDrift)
        );
    }

    #[test]
    fn rejects_duplicate_and_non_contiguous_numbers() {
        assert_eq!(
            generate_adr_index([record(1, "Proposed"), record(1, "Accepted")]),
            Err(AdrIndexError::DuplicateAdr {
                id: "ADR-0001".into(),
            })
        );
    }

    #[test]
    fn accepts_non_contiguous_deleted_or_reserved_numbers() {
        let artifacts = generate_adr_index([record(1, "Proposed"), record(3, "Accepted")])
            .expect("non-contiguous index generated");

        assert_eq!(artifacts.report.gaps, vec!["ADR-0002"]);
        assert_eq!(artifacts.report.next_adr, "ADR-0004");
        assert!(artifacts.markdown.contains("non-contiguous"));
        assert!(
            artifacts
                .markdown
                .contains("| 0002 | Not represented by a `docs/decisions/ADR-*.md` file")
        );
        assert!(!artifacts.markdown.contains("gaps: ADR-0002"));
        assert!(artifacts.json.contains("\"gaps\": [\"ADR-0002\"]"));
        assert_eq!(
            validate_adr_index(
                [record(1, "Proposed"), record(3, "Accepted")],
                &artifacts.markdown,
                &artifacts.json,
            )
            .expect("non-contiguous artifacts validate")
            .gaps,
            vec!["ADR-0002"]
        );
    }

    #[test]
    fn rejects_invalid_record_shape() {
        let mut invalid = record(1, "Proposed");
        invalid.owner.clear();

        assert!(matches!(
            generate_adr_index([invalid]),
            Err(AdrIndexError::InvalidRecord { .. })
        ));
    }

    fn record(number: u16, status: &str) -> AdrDecisionRecord {
        AdrDecisionRecord {
            number,
            id: format!("ADR-{number:04}"),
            title: if number == 1 {
                "Decision 1 with M0/M1/M2/M3/MVP retired terms".into()
            } else {
                format!("Decision {number}")
            },
            status: status.into(),
            owner: "council-architecture".into(),
            date: "2026-05-09".into(),
            path: format!("decisions/ADR-{number:04}-decision-{number}.md"),
            supersedes: vec![],
            superseded_by: vec![],
            related: vec![],
        }
    }
}
