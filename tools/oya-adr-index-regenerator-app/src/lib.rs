//! Rust/Buck2 ADR index regeneration app.
//!
//! The ADR files under `docs/decisions/` are source-of-truth. This app parses
//! the mixed historical ADR metadata shapes, renders `docs/ADR-INDEX.md` plus
//! `docs/machine-readable/decisions.json`, and checks or writes the committed
//! generated artifacts without reviving the retired local oya CLI.
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use oya_check_adr_index::{
    AdrDecisionRecord, AdrIndexArtifacts, AdrIndexReport, generate_adr_index,
};

pub const ADR_INDEX_PATH: &str = "docs/ADR-INDEX.md";
pub const DECISIONS_JSON_PATH: &str = "docs/machine-readable/decisions.json";
pub const DECISIONS_DIR: &str = "docs/decisions";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunMode {
    Check,
    Write,
}

impl RunMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Write => "write",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegeneratorRun {
    pub mode: RunMode,               // data_class: INTERNAL_ONLY
    pub report: AdrIndexReport,      // data_class: INTERNAL_ONLY
    pub markdown_drift: bool,        // data_class: INTERNAL_ONLY
    pub json_drift: bool,            // data_class: INTERNAL_ONLY
    pub wrote_artifacts: bool,       // data_class: INTERNAL_ONLY
    pub parse_warnings: Vec<String>, // data_class: INTERNAL_ONLY
    pub source_file_count: usize,    // data_class: INTERNAL_ONLY
    pub markdown_path: String,       // data_class: INTERNAL_ONLY
    pub decisions_json_path: String, // data_class: INTERNAL_ONLY
}

impl RegeneratorRun {
    #[must_use]
    pub fn clean(&self) -> bool {
        !self.markdown_drift && !self.json_drift
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str(&format!(
            "  \"mode\": \"{}\",\n",
            json_escape(self.mode.as_str())
        ));
        out.push_str(&format!(
            "  \"source_file_count\": {},\n",
            self.source_file_count
        ));
        out.push_str(&format!("  \"records\": {},\n", self.report.records));
        out.push_str(&format!(
            "  \"next_adr\": \"{}\",\n",
            json_escape(&self.report.next_adr)
        ));
        out.push_str(&format!(
            "  \"markdown_path\": \"{}\",\n",
            json_escape(&self.markdown_path)
        ));
        out.push_str(&format!(
            "  \"decisions_json_path\": \"{}\",\n",
            json_escape(&self.decisions_json_path)
        ));
        out.push_str(&format!("  \"markdown_drift\": {},\n", self.markdown_drift));
        out.push_str(&format!("  \"json_drift\": {},\n", self.json_drift));
        out.push_str(&format!(
            "  \"wrote_artifacts\": {},\n",
            self.wrote_artifacts
        ));
        out.push_str(&format!(
            "  \"status_counts\": {},\n",
            json_object_usize(&self.report.status_counts)
        ));
        out.push_str(&format!("  \"gaps\": {},\n", json_array(&self.report.gaps)));
        out.push_str(&format!(
            "  \"parse_warnings\": {}\n",
            json_array(&self.parse_warnings)
        ));
        out.push_str("}\n");
        out
    }

    #[must_use]
    pub fn to_text(&self) -> String {
        format!(
            "ADR index {mode}: records={records} next={next} markdown_drift={markdown} json_drift={json} wrote_artifacts={wrote} warnings={warnings}\n",
            mode = self.mode.as_str(),
            records = self.report.records,
            next = self.report.next_adr,
            markdown = self.markdown_drift,
            json = self.json_drift,
            wrote = self.wrote_artifacts,
            warnings = self.parse_warnings.len(),
        )
    }
}

#[derive(Debug)]
pub enum RegeneratorError {
    Io { path: String, message: String },
    InvalidAdrPath { path: String, message: String },
    Render { message: String },
}

impl fmt::Display for RegeneratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(formatter, "I/O error at {path}: {message}"),
            Self::InvalidAdrPath { path, message } => {
                write!(formatter, "invalid ADR path {path}: {message}")
            }
            Self::Render { message } => write!(formatter, "ADR index render error: {message}"),
        }
    }
}

impl std::error::Error for RegeneratorError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedAdrCorpus {
    pub records: Vec<AdrDecisionRecord>, // data_class: INTERNAL_ONLY
    pub warnings: Vec<String>,           // data_class: INTERNAL_ONLY
}

pub fn run(root: &Path, mode: RunMode) -> Result<RegeneratorRun, RegeneratorError> {
    let corpus = parse_adr_corpus(root)?;
    let artifacts = render_artifacts(corpus.records.clone())?;
    let markdown_path = root.join(ADR_INDEX_PATH);
    let decisions_json_path = root.join(DECISIONS_JSON_PATH);
    let current_markdown = read_optional(&markdown_path)?;
    let current_json = read_optional(&decisions_json_path)?;
    let markdown_drift = normalize_text(&current_markdown) != normalize_text(&artifacts.markdown);
    let json_drift = normalize_text(&current_json) != normalize_text(&artifacts.json);
    let wrote_artifacts = if mode == RunMode::Write {
        write_file(&markdown_path, &artifacts.markdown)?;
        write_file(&decisions_json_path, &artifacts.json)?;
        true
    } else {
        false
    };

    Ok(RegeneratorRun {
        mode,
        report: artifacts.report,
        markdown_drift,
        json_drift,
        wrote_artifacts,
        parse_warnings: corpus.warnings,
        source_file_count: corpus.records.len(),
        markdown_path: ADR_INDEX_PATH.to_owned(),
        decisions_json_path: DECISIONS_JSON_PATH.to_owned(),
    })
}

pub fn parse_adr_corpus(root: &Path) -> Result<ParsedAdrCorpus, RegeneratorError> {
    let decisions_dir = root.join(DECISIONS_DIR);
    let mut files = Vec::new();
    for entry in fs::read_dir(&decisions_dir).map_err(|error| io_error(&decisions_dir, error))? {
        let entry = entry.map_err(|error| io_error(&decisions_dir, error))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name.starts_with("ADR-") && file_name.ends_with(".md") {
            files.push(path);
        }
    }
    files.sort();

    let mut records = Vec::new();
    let mut warnings = Vec::new();
    for path in files {
        let contents = fs::read_to_string(&path).map_err(|error| io_error(&path, error))?;
        records.push(parse_adr_file(root, &path, &contents, &mut warnings)?);
    }

    Ok(ParsedAdrCorpus { records, warnings })
}

fn parse_adr_file(
    root: &Path,
    path: &Path,
    contents: &str,
    warnings: &mut Vec<String>,
) -> Result<AdrDecisionRecord, RegeneratorError> {
    let relative_path = path.strip_prefix(root).unwrap_or(path);
    let relative_display = slash_path(relative_path);
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(RegeneratorError::InvalidAdrPath {
            path: relative_display,
            message: "file name is not valid UTF-8".to_owned(),
        });
    };
    let Some(number) = parse_adr_number(file_name) else {
        return Err(RegeneratorError::InvalidAdrPath {
            path: relative_display,
            message: "file name must start with ADR-NNNN".to_owned(),
        });
    };
    let id = format!("ADR-{number:04}");
    let path_for_index = format!("decisions/{file_name}");
    let metadata = parse_metadata(contents);
    if !metadata.frontmatter_present {
        warnings.push(format!(
            "{id}: parsed non-YAML ADR metadata from {path_for_index}; normalize when touching this ADR"
        ));
    }

    let title = metadata.single("title").unwrap_or_else(|| {
        h1_title(contents, &id).unwrap_or_else(|| title_from_file_name(file_name, &id))
    });
    let status = metadata
        .single("status")
        .or_else(|| inline_field(contents, "status"))
        .or_else(|| table_field(contents, "status"))
        .map(|value| normalize_status(&value))
        .unwrap_or_else(|| "unknown".to_owned());
    let owner = metadata
        .single("owner")
        .or_else(|| metadata.single("owner_team"))
        .or_else(|| metadata.single("deciders"))
        .or_else(|| inline_field(contents, "owner"))
        .or_else(|| inline_field(contents, "deciders"))
        .or_else(|| table_field(contents, "owner"))
        .or_else(|| table_field(contents, "deciders"))
        .map(|value| normalize_people(&value))
        .unwrap_or_else(|| "unknown".to_owned());
    let date = metadata
        .single("date")
        .or_else(|| inline_field(contents, "date"))
        .or_else(|| table_field(contents, "date"))
        .map(|value| normalize_date(&value))
        .unwrap_or_else(|| "unknown".to_owned());
    let supersedes = metadata
        .adr_refs("supersedes")
        .or_else(|| table_refs(contents, "supersedes"))
        .or_else(|| inline_refs(contents, "supersedes"))
        .unwrap_or_default();
    let superseded_by = metadata
        .adr_refs("superseded_by")
        .or_else(|| metadata.adr_refs("superseded by"))
        .or_else(|| table_refs(contents, "superseded_by"))
        .or_else(|| table_refs(contents, "superseded by"))
        .or_else(|| table_refs(contents, "amended_by"))
        .or_else(|| table_refs(contents, "amended by"))
        .unwrap_or_default();
    let related = metadata
        .adr_refs("related")
        .or_else(|| metadata.adr_refs("references"))
        .or_else(|| table_refs(contents, "related"))
        .or_else(|| table_refs(contents, "references"))
        .or_else(|| inline_refs(contents, "references"))
        .unwrap_or_default();

    Ok(AdrDecisionRecord {
        number,
        id,
        title,
        status,
        owner,
        date,
        path: path_for_index,
        supersedes,
        superseded_by,
        related,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Metadata {
    frontmatter_present: bool,
    fields: BTreeMap<String, Vec<String>>,
}

impl Metadata {
    fn single(&self, key: &str) -> Option<String> {
        self.fields.get(&canonical_key(key)).and_then(|values| {
            values
                .iter()
                .map(|value| clean_scalar(value))
                .find(|value| !empty_value(value))
        })
    }

    fn adr_refs(&self, key: &str) -> Option<Vec<String>> {
        self.fields
            .get(&canonical_key(key))
            .map(|values| adr_refs_from_values(values))
            .filter(|values| !values.is_empty())
    }
}

fn parse_metadata(contents: &str) -> Metadata {
    if let Some(frontmatter) = frontmatter_block(contents) {
        Metadata {
            frontmatter_present: true,
            fields: parse_frontmatter(frontmatter),
        }
    } else {
        Metadata {
            frontmatter_present: false,
            fields: BTreeMap::new(),
        }
    }
}

fn frontmatter_block(contents: &str) -> Option<&str> {
    let rest = contents.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

fn parse_frontmatter(frontmatter: &str) -> BTreeMap<String, Vec<String>> {
    let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current_key: Option<String> = None;
    for raw_line in frontmatter.lines() {
        let line = raw_line.trim_end();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            if let Some(key) = current_key.as_ref() {
                fields
                    .entry(key.clone())
                    .or_default()
                    .push(clean_scalar(item));
            }
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = canonical_key(key.trim());
            current_key = Some(key.clone());
            let value = clean_scalar(value);
            fields.entry(key).or_default();
            if !value.is_empty() {
                fields
                    .entry(current_key.clone().expect("key set"))
                    .or_default()
                    .push(value);
            }
        }
    }
    fields
}

fn inline_field(contents: &str, key: &str) -> Option<String> {
    let wanted = canonical_key(key);
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') {
            continue;
        }
        let candidate = trimmed
            .strip_prefix("- ")
            .map(str::trim_start)
            .unwrap_or(trimmed);
        let candidate = candidate
            .strip_prefix("> ")
            .map(str::trim_start)
            .unwrap_or(candidate);
        if let Some(after_prefix) = candidate.strip_prefix("**") {
            let Some((field, value)) = after_prefix.split_once(":**") else {
                continue;
            };
            if canonical_key(field) == wanted {
                return Some(clean_scalar(value));
            }
            continue;
        }
        if let Some((field, value)) = candidate.split_once(':') {
            if canonical_key(field) == wanted {
                return Some(clean_scalar(value));
            }
        }
    }
    None
}

fn inline_refs(contents: &str, key: &str) -> Option<Vec<String>> {
    inline_field(contents, key)
        .map(|value| adr_refs_from_values(&[value]))
        .filter(|values| !values.is_empty())
}

fn table_field(contents: &str, key: &str) -> Option<String> {
    let wanted = canonical_key(key);
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }
        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(clean_scalar)
            .collect::<Vec<_>>();
        if cells.len() >= 2 && canonical_key(&cells[0]) == wanted {
            return Some(cells[1].clone());
        }
    }
    None
}

fn table_refs(contents: &str, key: &str) -> Option<Vec<String>> {
    table_field(contents, key)
        .map(|value| adr_refs_from_values(&[value]))
        .filter(|values| !values.is_empty())
}

fn parse_adr_number(file_name: &str) -> Option<u16> {
    let digits = file_name.strip_prefix("ADR-")?.get(..4)?;
    if digits.chars().all(|character| character.is_ascii_digit()) {
        digits.parse().ok()
    } else {
        None
    }
}

fn h1_title(contents: &str, id: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        let Some(title) = trimmed.strip_prefix("# ") else {
            continue;
        };
        let title = title.trim();
        if title.is_empty() {
            continue;
        }
        if let Some(rest) = title.strip_prefix(id) {
            let stripped = rest
                .trim_start()
                .trim_start_matches([':', '—', '–', '-'])
                .trim_start();
            if !stripped.is_empty() {
                return Some(stripped.to_owned());
            }
        }
        return Some(title.to_owned());
    }
    None
}

fn title_from_file_name(file_name: &str, id: &str) -> String {
    let without_extension = file_name.strip_suffix(".md").unwrap_or(file_name);
    let prefix = format!("{id}-");
    without_extension
        .strip_prefix(&prefix)
        .unwrap_or(without_extension)
        .replace('-', " ")
}

fn clean_scalar(value: &str) -> String {
    let mut value = value.trim().trim_matches('`').trim().to_owned();
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            value = value[1..value.len() - 1].to_owned();
        }
    }
    value.trim().to_owned()
}

fn normalize_status(value: &str) -> String {
    let mut status = clean_scalar(value).replace("**", "");
    for separator in [" — ", " – "] {
        if let Some((before, _after)) = status.split_once(separator) {
            status = before.to_owned();
        }
    }
    let status = status.trim().trim_end_matches('.').trim();
    for canonical in [
        "Proposed",
        "Accepted",
        "Superseded",
        "Rejected",
        "Deprecated",
    ] {
        if status == canonical
            || status
                .strip_prefix(canonical)
                .is_some_and(|rest| rest.starts_with([' ', '(']))
        {
            return canonical.to_owned();
        }
    }
    status.to_owned()
}

fn normalize_people(value: &str) -> String {
    clean_scalar(value)
        .replace('`', "")
        .replace(" + ", ", ")
        .replace(';', ",")
        .trim()
        .trim_end_matches('.')
        .to_owned()
}

fn normalize_date(value: &str) -> String {
    let value = clean_scalar(value);
    value
        .split_whitespace()
        .next()
        .unwrap_or("unknown")
        .trim_end_matches('.')
        .to_owned()
}

fn canonical_key(key: &str) -> String {
    key.trim()
        .trim_matches('*')
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
}

fn empty_value(value: &str) -> bool {
    let value = clean_scalar(value).to_ascii_lowercase();
    matches!(
        value.as_str(),
        "" | "[]" | "—" | "-" | "n/a" | "na" | "none" | "null"
    )
}

fn adr_refs_from_values(values: &[String]) -> Vec<String> {
    let mut refs = Vec::new();
    for value in values {
        let bytes = value.as_bytes();
        let mut index = 0;
        while index + 8 <= bytes.len() {
            if value[index..].starts_with("ADR-") {
                let candidate = &value[index..index + 8];
                if candidate[4..]
                    .chars()
                    .all(|character| character.is_ascii_digit())
                    && !refs.iter().any(|known| known == candidate)
                {
                    refs.push(candidate.to_owned());
                }
                index += 8;
            } else if let Some(character) = value[index..].chars().next() {
                index += character.len_utf8();
            } else {
                break;
            }
        }
    }
    refs
}

fn render_artifacts(
    records: Vec<AdrDecisionRecord>,
) -> Result<AdrIndexArtifacts, RegeneratorError> {
    generate_adr_index(records).map_err(|error| RegeneratorError::Render {
        message: format!("{error:?}"),
    })
}

fn read_optional(path: &Path) -> Result<String, RegeneratorError> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(io_error(path, error)),
    }
}

fn write_file(path: &Path, contents: &str) -> Result<(), RegeneratorError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
    }
    fs::write(path, contents).map_err(|error| io_error(path, error))
}

fn io_error(path: &Path, error: std::io::Error) -> RegeneratorError {
    RegeneratorError::Io {
        path: slash_path(path),
        message: error.to_string(),
    }
}

fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn normalize_text(value: &str) -> String {
    value.replace("\r\n", "\n").trim_end().to_owned()
}

fn json_object_usize(values: &BTreeMap<String, usize>) -> String {
    if values.is_empty() {
        return "{}".to_owned();
    }
    let body = values
        .iter()
        .map(|(key, value)| format!("\"{}\": {}", json_escape(key), value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{body}}}")
}

fn json_array(values: &[String]) -> String {
    if values.is_empty() {
        return "[]".to_owned();
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

#[must_use]
pub fn repo_root_from_env_or_cwd() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn parses_frontmatter_and_table_style_adrs() {
        let root = test_root("parse-frontmatter-and-table");
        write_test_file(
            &root,
            "docs/decisions/ADR-0001-first.md",
            r#"---
id: ADR-0001
title: "First decision"
status: Accepted
date: 2026-06-05
owner: council-architecture
supersedes: []
superseded_by: []
related: [ADR-0003]
---
# ADR-0001: Fallback title should not win
"#,
        );
        write_test_file(
            &root,
            "docs/decisions/ADR-0003-table-style.md",
            r#"# ADR-0003 — Table style decision

| Field | Value |
| --- | --- |
| Status | Proposed |
| Date | 2026-06-06 |
| Deciders | council-cloud, council-security |
| Supersedes | ADR-0001 |
| Superseded by | — |
| Related | ADR-0001 |
"#,
        );

        let corpus = parse_adr_corpus(&root).expect("corpus parses");

        assert_eq!(corpus.records.len(), 2);
        assert_eq!(corpus.records[0].title, "First decision");
        assert_eq!(corpus.records[0].related, vec!["ADR-0003"]);
        assert_eq!(corpus.records[1].owner, "council-cloud, council-security");
        assert_eq!(corpus.records[1].supersedes, vec!["ADR-0001"]);
        assert_eq!(corpus.warnings.len(), 1);
    }

    #[test]
    fn write_then_check_roundtrip_removes_generated_artifact_drift() {
        let root = test_root("write-then-check-roundtrip");
        write_test_file(
            &root,
            "docs/decisions/ADR-0001-first.md",
            r#"# ADR-0001: First decision

> **Status:** Accepted
> **Date:** 2026-06-05
> **Owner:** council-architecture
> **Supersedes:** N/A
> **References:** ADR-0003
"#,
        );
        write_test_file(
            &root,
            "docs/decisions/ADR-0003-third.md",
            r#"# ADR-0003 — Third decision

**Status:** Proposed
**Date:** 2026-06-06
**Owner:** council-cloud
**Supersedes:** ADR-0001
"#,
        );

        let write_run = run(&root, RunMode::Write).expect("write run succeeds");
        assert!(write_run.markdown_drift);
        assert!(write_run.json_drift);
        assert!(write_run.wrote_artifacts);
        assert_eq!(write_run.report.next_adr, "ADR-0004");
        assert_eq!(write_run.report.gaps, vec!["ADR-0002"]);

        let check_run = run(&root, RunMode::Check).expect("check run succeeds");
        assert!(check_run.clean());
        assert!(!check_run.wrote_artifacts);
        assert!(
            fs::read_to_string(root.join(ADR_INDEX_PATH))
                .expect("markdown exists")
                .contains("by the Rust/Buck2 ADR index generator")
        );
    }

    fn test_root(name: &str) -> PathBuf {
        let sequence = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "oya-adr-index-regenerator-{name}-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/decisions")).expect("decisions dir created");
        root
    }

    fn write_test_file(root: &Path, relative: &str, contents: &str) {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent created");
        }
        fs::write(path, contents).expect("test file written");
    }
}
