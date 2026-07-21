//! Foundry ADR index generation fitness kernel.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const LEGACY_AMENDMENT_EDGES: &str = include_str!("../legacy-amendment-edges.tsv");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdrDecisionRecord {
    pub number: u16,                        // data_class: INTERNAL_ONLY
    pub id: String,                         // data_class: INTERNAL_ONLY
    pub title: String,                      // data_class: INTERNAL_ONLY
    pub status: String,                     // data_class: INTERNAL_ONLY
    pub owner: String,                      // data_class: INTERNAL_ONLY
    pub date: String,                       // data_class: INTERNAL_ONLY
    pub path: String,                       // data_class: INTERNAL_ONLY
    pub supersedes: Vec<String>,            // data_class: INTERNAL_ONLY
    pub superseded_by: Vec<String>,         // data_class: INTERNAL_ONLY
    pub amends: Vec<String>,                // data_class: INTERNAL_ONLY
    pub amended_by: Vec<String>,            // data_class: INTERNAL_ONLY
    pub lifecycle_contract: Option<String>, // data_class: INTERNAL_ONLY
    pub related: Vec<String>,               // data_class: INTERNAL_ONLY
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
    SourceRead { reason: String },
    LifecycleViolation { id: String, reason: String },
}

/// Parse the authoritative ADR markdown corpus into the shared source IR used
/// by both the official producer and cross-artifact projection checks.
pub fn read_adr_decision_records(
    decisions_dir: &Path,
) -> Result<Vec<AdrDecisionRecord>, AdrIndexError> {
    let mut paths = fs::read_dir(decisions_dir)
        .map_err(|error| {
            source_read(format!(
                "ADR decisions dir unreadable {}: {error}",
                decisions_dir.display()
            ))
        })?
        .map(|entry| {
            entry.map(|entry| entry.path()).map_err(|error| {
                source_read(format!("ADR decisions dir entry unreadable: {error}"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("ADR-") && name.ends_with(".md"))
    });
    paths.sort();
    let base_decision_ids = paths
        .iter()
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
        .filter(|name| !name.contains("-amendment-"))
        .filter_map(|name| name.get(0..8).map(str::to_string))
        .collect::<BTreeSet<_>>();
    paths.retain(|path| {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return true;
        };
        !name.contains("-amendment-")
            || name
                .get(0..8)
                .is_none_or(|id| !base_decision_ids.contains(id))
    });
    let records = paths
        .iter()
        .map(|path| read_adr_decision_record(path))
        .collect::<Result<Vec<_>, _>>()?;
    if is_live_oyatie_decisions_dir(decisions_dir) {
        validate_legacy_baseline(&records)?;
    }
    validate_lifecycle(&records)?;
    Ok(records)
}

fn source_read(reason: String) -> AdrIndexError {
    AdrIndexError::SourceRead { reason }
}

fn is_live_oyatie_decisions_dir(decisions_dir: &Path) -> bool {
    decisions_dir
        .parent()
        .and_then(Path::parent)
        .is_some_and(|root| root.join("specs/root-hub-pointers.json").is_file())
}

pub fn generate_adr_index<I>(records: I) -> Result<AdrIndexArtifacts, AdrIndexError>
where
    I: IntoIterator<Item = AdrDecisionRecord>,
{
    let records = normalized_records(records)?;
    validate_lifecycle(&records)?;
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
    if normalize_text(current_markdown) != normalize_text(&artifacts.markdown) {
        return Err(AdrIndexError::MarkdownDrift);
    }
    if normalize_text(current_json) != normalize_text(&artifacts.json) {
        return Err(AdrIndexError::JsonDrift);
    }
    Ok(artifacts.report)
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

fn validate_lifecycle(records: &[AdrDecisionRecord]) -> Result<(), AdrIndexError> {
    const RECIPROCAL_ACCEPTED_V1: &str = "reciprocal-accepted-v1";
    let frozen_legacy_edges = frozen_legacy_edges()?;
    let by_id = records
        .iter()
        .map(|record| (record.id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    for record in records {
        validate_unique_lifecycle_endpoints(record, "amends", &record.amends)?;
        validate_unique_lifecycle_endpoints(record, "amended_by", &record.amended_by)?;
        validate_edge_contracts(record, &frozen_legacy_edges)?;
        let Some(contract) = record.lifecycle_contract.as_deref() else {
            continue;
        };
        if contract != RECIPROCAL_ACCEPTED_V1 {
            return Err(lifecycle_error(
                &record.id,
                format!("unsupported lifecycle_contract {contract}"),
            ));
        }
        for amended_id in &record.amends {
            let amended = lifecycle_endpoint(&by_id, &record.id, "amends", amended_id)?;
            validate_accepted_lifecycle_pair(record, amended)?;
            if amended.amended_by.iter().any(|id| id == &record.id) {
                continue;
            }
            return Err(lifecycle_error(
                &record.id,
                format!(
                    "amends {amended_id} but {amended_id}.amended_by is missing {}",
                    record.id
                ),
            ));
        }
        for amender_id in &record.amended_by {
            let amender = lifecycle_endpoint(&by_id, &record.id, "amended_by", amender_id)?;
            validate_accepted_lifecycle_pair(amender, record)?;
            if amender.amends.iter().any(|id| id == &record.id) {
                continue;
            }
            return Err(lifecycle_error(
                &record.id,
                format!(
                    "amended_by {amender_id} but {amender_id}.amends is missing {}",
                    record.id
                ),
            ));
        }
    }
    Ok(())
}

fn validate_legacy_baseline(records: &[AdrDecisionRecord]) -> Result<(), AdrIndexError> {
    let frozen_legacy_edges = frozen_legacy_edges()?;
    let actual_legacy_edges = records
        .iter()
        .filter(|record| record.lifecycle_contract.is_none())
        .flat_map(amendment_edges)
        .collect::<BTreeSet<_>>();
    if actual_legacy_edges == frozen_legacy_edges {
        return Ok(());
    }
    Err(lifecycle_error(
        "legacy-amendment-edges",
        format!(
            "frozen legacy relationship-edge baseline drift: missing={:?}; unexpected={:?}",
            frozen_legacy_edges
                .difference(&actual_legacy_edges)
                .collect::<Vec<_>>(),
            actual_legacy_edges
                .difference(&frozen_legacy_edges)
                .collect::<Vec<_>>()
        ),
    ))
}

fn validate_edge_contracts(
    record: &AdrDecisionRecord,
    frozen_legacy_edges: &BTreeSet<AmendmentEdge>,
) -> Result<(), AdrIndexError> {
    for edge in amendment_edges(record) {
        if frozen_legacy_edges.contains(&edge) {
            continue;
        }
        if record.lifecycle_contract.is_some() {
            continue;
        }
        return Err(lifecycle_error(
            &record.id,
            format!(
                "new or changed {} endpoint {} must declare lifecycle_contract reciprocal-accepted-v1",
                edge.field, edge.endpoint
            ),
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AmendmentEdge {
    source: String,
    field: &'static str,
    endpoint: String,
}

fn amendment_edges(record: &AdrDecisionRecord) -> impl Iterator<Item = AmendmentEdge> + '_ {
    record
        .amends
        .iter()
        .map(|endpoint| AmendmentEdge {
            source: record.id.clone(),
            field: "amends",
            endpoint: endpoint.clone(),
        })
        .chain(record.amended_by.iter().map(|endpoint| AmendmentEdge {
            source: record.id.clone(),
            field: "amended_by",
            endpoint: endpoint.clone(),
        }))
}

fn frozen_legacy_edges() -> Result<BTreeSet<AmendmentEdge>, AdrIndexError> {
    LEGACY_AMENDMENT_EDGES
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut cells = line.split('\t');
            let (Some(source), Some(field), Some(endpoint), None) =
                (cells.next(), cells.next(), cells.next(), cells.next())
            else {
                return Err(source_read(format!(
                    "invalid frozen legacy amendment edge {line:?}"
                )));
            };
            if !matches!(field, "amends" | "amended_by") {
                return Err(source_read(format!(
                    "invalid frozen legacy amendment field {field:?}"
                )));
            }
            Ok(AmendmentEdge {
                source: source.to_owned(),
                field,
                endpoint: endpoint.to_owned(),
            })
        })
        .collect()
}

fn validate_unique_lifecycle_endpoints(
    record: &AdrDecisionRecord,
    field: &str,
    endpoints: &[String],
) -> Result<(), AdrIndexError> {
    let mut seen = BTreeSet::new();
    for endpoint in endpoints {
        if seen.insert(endpoint) {
            continue;
        }
        return Err(lifecycle_error(
            &record.id,
            format!("{field} contains duplicate endpoint {endpoint}"),
        ));
    }
    Ok(())
}

fn lifecycle_endpoint<'a>(
    by_id: &BTreeMap<&str, &'a AdrDecisionRecord>,
    id: &str,
    field: &str,
    endpoint_id: &str,
) -> Result<&'a AdrDecisionRecord, AdrIndexError> {
    by_id.get(endpoint_id).copied().ok_or_else(|| {
        lifecycle_error(
            id,
            format!("{field} references missing endpoint {endpoint_id}"),
        )
    })
}

fn validate_accepted_lifecycle_pair(
    amender: &AdrDecisionRecord,
    amended: &AdrDecisionRecord,
) -> Result<(), AdrIndexError> {
    const RECIPROCAL_ACCEPTED_V1: &str = "reciprocal-accepted-v1";
    if amended.lifecycle_contract.as_deref() != Some(RECIPROCAL_ACCEPTED_V1) {
        return Err(lifecycle_error(
            &amender.id,
            format!(
                "amendment endpoint {} is missing lifecycle_contract {RECIPROCAL_ACCEPTED_V1}",
                amended.id
            ),
        ));
    }
    if amender.status.eq_ignore_ascii_case("Accepted")
        && amended.status.eq_ignore_ascii_case("Accepted")
    {
        return Ok(());
    }
    Err(lifecycle_error(
        &amender.id,
        format!(
            "amendment endpoint {} must be Accepted (statuses: {} -> {})",
            amended.id, amender.status, amended.status
        ),
    ))
}

fn lifecycle_error(id: &str, reason: String) -> AdrIndexError {
    AdrIndexError::LifecycleViolation {
        id: id.to_owned(),
        reason,
    }
}

fn read_adr_decision_record(path: &Path) -> Result<AdrDecisionRecord, AdrIndexError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| source_read(format!("ADR path is not utf8: {}", path.display())))?;
    let id = file_name
        .get(0..8)
        .ok_or_else(|| source_read(format!("ADR filename too short: {file_name}")))?
        .to_string();
    let number = id
        .strip_prefix("ADR-")
        .ok_or_else(|| source_read(format!("ADR filename missing ADR prefix: {file_name}")))?
        .parse::<u16>()
        .map_err(|error| source_read(format!("ADR number invalid in {file_name}: {error}")))?;
    let contents = fs::read_to_string(path)
        .map_err(|error| source_read(format!("ADR file unreadable {}: {error}", path.display())))?;
    let title = parse_adr_title(&contents, &id, path)?;
    let metadata = parse_adr_metadata(&contents);
    let status = required_adr_metadata(&metadata, "Status", path)?;
    let owner = required_adr_metadata(&metadata, "Owner", path)?;
    let date = required_adr_metadata(&metadata, "Date", path)?;

    Ok(AdrDecisionRecord {
        number,
        id,
        title,
        status,
        owner,
        date,
        path: format!("decisions/{file_name}"),
        supersedes: optional_single_adr_metadata(&metadata, "Supersedes"),
        superseded_by: optional_single_adr_metadata(&metadata, "Superseded-by"),
        amends: optional_list_adr_metadata(&metadata, "Amends"),
        amended_by: optional_list_adr_metadata(&metadata, "Amended-by"),
        lifecycle_contract: metadata.get("Lifecycle-contract").cloned(),
        related: optional_list_adr_metadata(&metadata, "Related"),
    })
}

fn parse_adr_title(
    contents: &str,
    expected_id: &str,
    path: &Path,
) -> Result<String, AdrIndexError> {
    let first_line = content_after_leading_frontmatter(contents)
        .lines()
        .find(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with("<!--")
        })
        .ok_or_else(|| source_read(format!("ADR file empty {}", path.display())))?;
    let title_line = first_line.trim().strip_prefix("# ").ok_or_else(|| {
        source_read(format!(
            "ADR first line must be an H1 in {}",
            path.display()
        ))
    })?;
    let Some((id, title)) = split_adr_h1_title(title_line) else {
        return Err(source_read(format!(
            "ADR H1 must use '<id>: <title>' in {}",
            path.display()
        )));
    };
    if id.trim() != expected_id {
        return Err(source_read(format!(
            "ADR H1 id {} does not match filename id {} in {}",
            id.trim(),
            expected_id,
            path.display()
        )));
    }
    let title = title.trim();
    if title.is_empty() {
        Err(source_read(format!(
            "ADR title empty in {}",
            path.display()
        )))
    } else {
        Ok(title.into())
    }
}

fn split_adr_h1_title(title_line: &str) -> Option<(&str, &str)> {
    match (title_line.find(':'), title_line.find(" — ")) {
        (Some(colon), Some(dash)) if dash < colon => {
            Some((&title_line[..dash], &title_line[dash + " — ".len()..]))
        }
        (Some(colon), _) => Some((&title_line[..colon], &title_line[colon + 1..])),
        (None, Some(dash)) => Some((&title_line[..dash], &title_line[dash + " — ".len()..])),
        (None, None) => None,
    }
}

fn split_leading_frontmatter(contents: &str) -> (Option<&str>, &str) {
    let mut lines = contents.split_inclusive('\n');
    let Some(first_line) = lines.next() else {
        return (None, contents);
    };
    if first_line.trim() != "---" {
        return (None, contents);
    }
    let frontmatter_start = first_line.len();
    let mut offset = first_line.len();
    for line in lines {
        let line_start = offset;
        offset += line.len();
        if line.trim() == "---" {
            return (
                Some(&contents[frontmatter_start..line_start]),
                &contents[offset..],
            );
        }
    }
    (None, contents)
}

fn content_after_leading_frontmatter(contents: &str) -> &str {
    split_leading_frontmatter(contents).1
}

fn parse_adr_metadata(contents: &str) -> BTreeMap<String, String> {
    let (frontmatter, contents) = split_leading_frontmatter(contents);
    let mut metadata = frontmatter
        .map(parse_frontmatter_metadata)
        .unwrap_or_default();
    for line in contents.lines().take(30) {
        let mut trimmed = line.trim();
        if trimmed == "---" || trimmed.starts_with("## ") {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix('>') {
            trimmed = rest.trim();
        }
        if let Some(rest) = trimmed.strip_prefix("- ") {
            trimmed = rest.trim();
        }
        if !trimmed.starts_with("**")
            && let Some((key, value)) = trimmed.split_once(':')
        {
            metadata
                .entry(canonical_adr_metadata_key(key.trim()).into())
                .or_insert_with(|| clean_adr_metadata_value(value));
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("**") else {
            continue;
        };
        let Some((key, value)) = rest.split_once(":**") else {
            continue;
        };
        metadata
            .entry(canonical_adr_metadata_key(key.trim()).into())
            .or_insert_with(|| clean_adr_metadata_value(value));
    }
    for line in contents.lines().take(40) {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            break;
        }
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }
        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() < 2
            || cells[0].eq_ignore_ascii_case("field")
            || cells[0].chars().all(|character| character == '-')
            || cells[1].chars().all(|character| character == '-')
        {
            continue;
        }
        metadata
            .entry(canonical_adr_metadata_key(cells[0]).into())
            .or_insert_with(|| clean_adr_metadata_value(cells[1]));
    }
    metadata
}

fn parse_frontmatter_metadata(frontmatter: &str) -> BTreeMap<String, String> {
    let mut raw = BTreeMap::<String, Vec<String>>::new();
    let mut current_key = None::<String>;
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            if let Some(key) = current_key.as_ref() {
                raw.entry(key.clone())
                    .or_default()
                    .extend(clean_yaml_metadata_values(item));
            }
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            current_key = None;
            continue;
        };
        let key = key.trim().to_string();
        current_key = Some(key.clone());
        raw.entry(key)
            .or_default()
            .extend(clean_yaml_metadata_values(value));
    }
    let mut metadata = BTreeMap::new();
    for (key, values) in raw {
        let Some(mapped_key) = frontmatter_metadata_key(&key) else {
            continue;
        };
        let values = values
            .into_iter()
            .map(|value| normalize_frontmatter_metadata_value(mapped_key, &value))
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if !values.is_empty() {
            metadata.insert(mapped_key.into(), values.join(", "));
        }
    }
    metadata
}

fn frontmatter_metadata_key(key: &str) -> Option<&'static str> {
    match key {
        "status" => Some("Status"),
        "date" => Some("Date"),
        "deciders" => Some("Deciders"),
        "owner" | "owner_team" | "decision_owner" => Some("Owner"),
        "owners" => Some("Owners"),
        "supersedes" => Some("Supersedes"),
        "superseded_by" => Some("Superseded-by"),
        "amends" => Some("Amends"),
        "amended_by" => Some("Amended-by"),
        "lifecycle_contract" => Some("Lifecycle-contract"),
        "related" | "related_adrs" => Some("Related"),
        _ => None,
    }
}

fn canonical_adr_metadata_key(key: &str) -> &str {
    match key {
        "Superseded by" => "Superseded-by",
        "Amended by" => "Amended-by",
        "Related ADRs" => "Related",
        value => value,
    }
}

fn clean_yaml_metadata_values(value: &str) -> Vec<String> {
    let value = value.trim();
    if is_empty_metadata_value(value) || value == "~" || value == "[]" {
        return Vec::new();
    }
    if let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    {
        return inner
            .split(',')
            .map(clean_yaml_scalar)
            .filter(|value| !value.is_empty())
            .collect();
    }
    let value = clean_yaml_scalar(value);
    if value.is_empty() {
        Vec::new()
    } else {
        vec![value]
    }
}

fn clean_yaml_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn normalize_frontmatter_metadata_value(key: &str, value: &str) -> String {
    if matches!(
        key,
        "Supersedes" | "Superseded-by" | "Amends" | "Amended-by" | "Related"
    ) && let Some(adr) = extract_adr_id(value)
    {
        return adr;
    }
    value.to_string()
}

fn extract_adr_id(value: &str) -> Option<String> {
    let start = value.find("ADR-")?;
    let candidate = value.get(start..start + 8)?;
    let digits = candidate.strip_prefix("ADR-")?;
    if digits.chars().all(|character| character.is_ascii_digit()) {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn required_adr_metadata(
    metadata: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<String, AdrIndexError> {
    let value = metadata.get(key).or_else(|| {
        if key == "Owner" {
            metadata
                .get("Owners")
                .or_else(|| metadata.get("Deciders"))
                .or_else(|| metadata.get("Authors"))
        } else {
            None
        }
    });
    value
        .filter(|value| !is_empty_metadata_value(value.trim()))
        .cloned()
        .ok_or_else(|| source_read(format!("ADR metadata {key} missing in {}", path.display())))
}

fn optional_list_adr_metadata(metadata: &BTreeMap<String, String>, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !is_empty_metadata_value(item))
                .map(|item| extract_adr_id(item).unwrap_or_else(|| item.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn optional_single_adr_metadata(metadata: &BTreeMap<String, String>, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !is_empty_metadata_value(value))
        .map(|value| vec![value.to_owned()])
        .unwrap_or_default()
}

fn clean_adr_metadata_value(value: &str) -> String {
    value.trim().replace('`', "")
}

fn is_empty_metadata_value(value: &str) -> bool {
    let value = value.trim();
    value.is_empty()
        || value == "-"
        || value == "—"
        || value.eq_ignore_ascii_case("none")
        || value.eq_ignore_ascii_case("n/a")
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
            "      \"amends\": {},\n",
            json_array(&record.amends)
        ));
        out.push_str(&format!(
            "      \"amended_by\": {},\n",
            json_array(&record.amended_by)
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    static FIXTURE_COUNTER: AtomicUsize = AtomicUsize::new(0);

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

    #[test]
    fn lifecycle_rejects_missing_amendment_endpoint() {
        let mut amender = reciprocal_record(1, "Accepted");
        amender.amends = vec!["ADR-0002".into()];

        assert!(matches!(
            generate_adr_index([amender]),
            Err(AdrIndexError::LifecycleViolation { reason, .. })
                if reason.contains("missing endpoint ADR-0002")
        ));
    }

    #[test]
    fn lifecycle_rejects_mismatched_reciprocal_edge() {
        let mut amender = reciprocal_record(1, "Accepted");
        amender.amends = vec!["ADR-0002".into()];
        let amended = reciprocal_record(2, "Accepted");

        assert!(matches!(
            generate_adr_index([amender, amended]),
            Err(AdrIndexError::LifecycleViolation { reason, .. })
                if reason.contains("amended_by is missing ADR-0001")
        ));
    }

    #[test]
    fn lifecycle_rejects_duplicate_amendment_endpoint() {
        let mut amender = reciprocal_record(1, "Accepted");
        amender.amends = vec!["ADR-0002".into(), "ADR-0002".into()];
        let mut amended = reciprocal_record(2, "Accepted");
        amended.amended_by = vec!["ADR-0001".into()];

        assert!(matches!(
            generate_adr_index([amender, amended]),
            Err(AdrIndexError::LifecycleViolation { reason, .. })
                if reason.contains("duplicate endpoint ADR-0002")
        ));
    }

    #[test]
    fn lifecycle_rejects_non_accepted_amendment_endpoint() {
        let mut amender = reciprocal_record(1, "Accepted");
        amender.amends = vec!["ADR-0002".into()];
        let mut amended = reciprocal_record(2, "Proposed");
        amended.amended_by = vec!["ADR-0001".into()];

        assert!(matches!(
            generate_adr_index([amender, amended]),
            Err(AdrIndexError::LifecycleViolation { reason, .. })
                if reason.contains("must be Accepted")
        ));
    }

    #[test]
    fn lifecycle_rejects_bilateral_edge_without_contract_marker() {
        let mut amender = record(1, "Accepted");
        amender.amends = vec!["ADR-0002".into()];
        let mut amended = record(2, "Accepted");
        amended.amended_by = vec!["ADR-0001".into()];

        assert!(matches!(
            generate_adr_index([amender, amended]),
            Err(AdrIndexError::LifecycleViolation { reason, .. })
                if reason.contains("must declare lifecycle_contract reciprocal-accepted-v1")
        ));
    }

    #[test]
    fn lifecycle_rejects_duplicate_uncontracted_relationship_occurrence() {
        let mut amender = record(1, "Accepted");
        amender.amends = vec!["ADR-0002".into(), "ADR-0002".into()];

        assert!(matches!(
            generate_adr_index([amender]),
            Err(AdrIndexError::LifecycleViolation { reason, .. })
                if reason.contains("duplicate endpoint ADR-0002")
        ));
    }

    #[test]
    fn lifecycle_accepts_reciprocal_accepted_edges_and_projects_them() {
        let mut amender = reciprocal_record(1, "Accepted");
        amender.amends = vec!["ADR-0002".into()];
        let mut amended = reciprocal_record(2, "Accepted");
        amended.amended_by = vec!["ADR-0001".into()];

        let artifacts = generate_adr_index([amender, amended]).expect("valid lifecycle");
        assert!(artifacts.json.contains("\"amends\": [\"ADR-0002\"]"));
        assert!(artifacts.json.contains("\"amended_by\": [\"ADR-0001\"]"));
    }

    #[test]
    fn source_fixture_rejects_missing_amendment_endpoint() {
        let fixture = SourceFixture::new(&[(
            "ADR-0001-amender.md",
            adr_source("ADR-0001", "Accepted", "[ADR-0002]", "[]"),
        )]);

        assert_lifecycle_reason(
            read_adr_decision_records(fixture.path()),
            "missing endpoint ADR-0002",
        );
    }

    #[test]
    fn source_fixture_rejects_mismatched_reciprocal_edge() {
        let fixture = SourceFixture::new(&[
            (
                "ADR-0001-amender.md",
                adr_source("ADR-0001", "Accepted", "[ADR-0002]", "[]"),
            ),
            (
                "ADR-0002-amended.md",
                adr_source("ADR-0002", "Accepted", "[]", "[]"),
            ),
        ]);

        assert_lifecycle_reason(
            read_adr_decision_records(fixture.path()),
            "amended_by is missing ADR-0001",
        );
    }

    #[test]
    fn source_fixture_rejects_duplicate_amendment_endpoint() {
        let fixture = SourceFixture::new(&[
            (
                "ADR-0001-amender.md",
                adr_source("ADR-0001", "Accepted", "[ADR-0002, ADR-0002]", "[]"),
            ),
            (
                "ADR-0002-amended.md",
                adr_source("ADR-0002", "Accepted", "[]", "[ADR-0001]"),
            ),
        ]);

        assert_lifecycle_reason(
            read_adr_decision_records(fixture.path()),
            "duplicate endpoint ADR-0002",
        );
    }

    #[test]
    fn source_fixture_rejects_non_accepted_amendment_endpoint() {
        let fixture = SourceFixture::new(&[
            (
                "ADR-0001-amender.md",
                adr_source("ADR-0001", "Accepted", "[ADR-0002]", "[]"),
            ),
            (
                "ADR-0002-amended.md",
                adr_source("ADR-0002", "Proposed", "[]", "[ADR-0001]"),
            ),
        ]);

        assert_lifecycle_reason(
            read_adr_decision_records(fixture.path()),
            "must be Accepted",
        );
    }

    #[test]
    fn source_markdown_drift_fails_projection_validation() {
        let fixture = SourceFixture::new(&[(
            "ADR-0001-decision.md",
            adr_source("ADR-0001", "Accepted", "[]", "[]"),
        )]);
        let artifacts = generate_adr_index(
            read_adr_decision_records(fixture.path()).expect("read initial source"),
        )
        .expect("render initial projection");
        fixture.write(
            "ADR-0001-decision.md",
            &adr_source("ADR-0001", "Proposed", "[]", "[]"),
        );

        assert_eq!(
            validate_adr_index(
                read_adr_decision_records(fixture.path()).expect("read changed source"),
                &artifacts.markdown,
                &artifacts.json,
            ),
            Err(AdrIndexError::MarkdownDrift)
        );
    }

    fn assert_lifecycle_reason(
        result: Result<Vec<AdrDecisionRecord>, AdrIndexError>,
        expected_reason: &str,
    ) {
        assert!(matches!(
            result,
            Err(AdrIndexError::LifecycleViolation { reason, .. }) if reason.contains(expected_reason)
        ));
    }

    fn adr_source(id: &str, status: &str, amends: &str, amended_by: &str) -> String {
        format!(
            "---\nstatus: {status}\nowner: council-architecture\ndate: 2026-07-21\namends: {amends}\namended_by: {amended_by}\nlifecycle_contract: reciprocal-accepted-v1\n---\n# {id}: Fixture decision\n"
        )
    }

    struct SourceFixture {
        dir: std::path::PathBuf,
    }

    impl SourceFixture {
        fn new(files: &[(&str, String)]) -> Self {
            let index = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "oya-check-adr-index-{}-{index}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).expect("create fixture dir");
            let fixture = Self { dir };
            for (name, contents) in files {
                fixture.write(name, contents);
            }
            fixture
        }

        fn path(&self) -> &Path {
            &self.dir
        }

        fn write(&self, name: &str, contents: &str) {
            std::fs::write(self.dir.join(name), contents).expect("write fixture ADR");
        }
    }

    impl Drop for SourceFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn reciprocal_record(number: u16, status: &str) -> AdrDecisionRecord {
        let mut record = record(number, status);
        record.lifecycle_contract = Some("reciprocal-accepted-v1".into());
        record
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
            amends: vec![],
            amended_by: vec![],
            lifecycle_contract: None,
            related: vec![],
        }
    }
}
