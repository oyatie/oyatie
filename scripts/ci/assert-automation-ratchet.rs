//! Validate AC-0.16 automation-ratchet coverage without live authority claims.
//!
//! This checker is local/static fixture evidence only. It evaluates the checked-in
//! Phase-0 automation matrix, the coverage registry, and declared BAD/GREEN
//! fixtures so enforceable requirements cannot hide as operator procedure, map
//! back to `oya` CLI authority, or leave seed rows unmapped. It never posts
//! statuses, mutates branch protection, or claims P0.0/Phase-0 green.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_MATRIX: &str = "specs/phase0-automation-matrix.json";
const DEFAULT_COVERAGE_REGISTRY: &str = "specs/phase0-automation-coverage-registry.json";
const ALLOWED_CLASSIFICATIONS: &[&str] = &[
    "automated_blocking_now",
    "automated_advisory_until_p0_0",
    "controller_owned_in_phase_1",
    "not_automatable_human_judgment",
];
const REQUIRED_ROW_FIELDS: &[&str] = &[
    "id",
    "source_artifact",
    "requirement",
    "classification",
    "owner",
    "target_gate_or_controller",
    "blocking_fixture",
    "retirement_phase",
    "evidence_path",
    "no_new_oya_cli_surface",
];

#[derive(Clone, Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    fn as_object(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Json::Object(value) => Some(value),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(value) => Some(value),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageSummary {
    pub required_subject_count: usize,
    pub coverage_subject_count: usize,
    pub mapped_row_count: usize,
    pub missing_required_subject_ids: Vec<String>,
    pub missing_mapped_row_ids: Vec<String>,
    pub unmapped_row_ids: Vec<String>,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixSummary {
    pub row_count: usize,
    pub required_seed_row_count: usize,
    pub duplicate_row_ids: Vec<String>,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult {
    pub path: String,
    pub fixture_id: String,
    pub expected_verdict: String,
    pub expected_violations: Vec<String>,
    pub observed_violations: Vec<String>,
    pub fixture_passed: bool,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub authority_boundary: String,
    pub matrix: String,
    pub coverage_registry: String,
    pub required_row_fields: Vec<String>,
    pub allowed_classifications: Vec<String>,
    pub matrix_summary: MatrixSummary,
    pub coverage_registry_summary: CoverageSummary,
    pub fixture_results: Vec<FixtureResult>,
    pub local_fixture_contract_proven: bool,
    pub coverage_registry_local_static_proven: bool,
    pub automation_ratchet_live: bool,
    pub protected_branch_authority_proven: bool,
    pub status_mutation_performed: bool,
    pub live_required_context_execution_proven: bool,
    pub p0_0_green: bool,
    pub phase0_complete: bool,
    pub verdict: String,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub repo_root: PathBuf,
    pub matrix: PathBuf,
    pub coverage_registry: PathBuf,
    pub fixtures: Vec<PathBuf>,
}

struct Parser<'a> {
    text: &'a [u8],
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text: text.as_bytes(),
            index: 0,
        }
    }

    fn parse(mut self) -> Result<Json, String> {
        let value = self.parse_value()?;
        self.skip_ws();
        if self.index == self.text.len() {
            Ok(value)
        } else {
            Err(format!("unexpected trailing JSON at byte {}", self.index))
        }
    }

    fn parse_value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'\"') => self.parse_string().map(Json::String),
            Some(b't') => self.expect_literal(b"true", Json::Bool(true)),
            Some(b'f') => self.expect_literal(b"false", Json::Bool(false)),
            Some(b'n') => self.expect_literal(b"null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(Json::Number),
            Some(byte) => Err(format!(
                "unexpected JSON byte {} at byte {}",
                byte as char, self.index
            )),
            None => Err("unexpected end of JSON".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<Json, String> {
        self.consume(b'{')?;
        let mut object = BTreeMap::new();
        self.skip_ws();
        if self.try_consume(b'}') {
            return Ok(Json::Object(object));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.consume(b':')?;
            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_ws();
            if self.try_consume(b'}') {
                break;
            }
            self.consume(b',')?;
        }
        Ok(Json::Object(object))
    }

    fn parse_array(&mut self) -> Result<Json, String> {
        self.consume(b'[')?;
        let mut array = Vec::new();
        self.skip_ws();
        if self.try_consume(b']') {
            return Ok(Json::Array(array));
        }
        loop {
            array.push(self.parse_value()?);
            self.skip_ws();
            if self.try_consume(b']') {
                break;
            }
            self.consume(b',')?;
        }
        Ok(Json::Array(array))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume(b'\"')?;
        let mut out = String::new();
        while let Some(byte) = self.next() {
            match byte {
                b'\"' => return Ok(out),
                b'\\' => {
                    let escaped = self
                        .next()
                        .ok_or_else(|| "unterminated JSON string escape".to_string())?;
                    match escaped {
                        b'\"' => out.push('\"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => out.push(
                            char::from_u32(self.parse_unicode_escape()?).unwrap_or('\u{fffd}'),
                        ),
                        other => {
                            return Err(format!(
                                "invalid JSON string escape {} at byte {}",
                                other as char, self.index
                            ));
                        }
                    }
                }
                other => out.push(other as char),
            }
        }
        Err("unterminated JSON string".to_string())
    }

    fn parse_unicode_escape(&mut self) -> Result<u32, String> {
        let mut value = 0u32;
        for _ in 0..4 {
            let byte = self
                .next()
                .ok_or_else(|| "short JSON unicode escape".to_string())?;
            value = value * 16
                + match byte {
                    b'0'..=b'9' => (byte - b'0') as u32,
                    b'a'..=b'f' => (byte - b'a' + 10) as u32,
                    b'A'..=b'F' => (byte - b'A' + 10) as u32,
                    _ => return Err("invalid JSON unicode escape".to_string()),
                };
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<String, String> {
        let start = self.index;
        if self.peek() == Some(b'-') {
            self.index += 1;
        }
        self.consume_digits();
        if self.peek() == Some(b'.') {
            self.index += 1;
            self.consume_digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.index += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.index += 1;
            }
            self.consume_digits();
        }
        if self.index == start {
            Err(format!("invalid JSON number at byte {start}"))
        } else {
            Ok(String::from_utf8_lossy(&self.text[start..self.index]).into_owned())
        }
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.index += 1;
        }
    }

    fn expect_literal(&mut self, literal: &[u8], value: Json) -> Result<Json, String> {
        if self.text.get(self.index..self.index + literal.len()) == Some(literal) {
            self.index += literal.len();
            Ok(value)
        } else {
            Err(format!("expected literal at byte {}", self.index))
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.index += 1;
        }
    }

    fn consume(&mut self, expected: u8) -> Result<(), String> {
        if self.try_consume(expected) {
            Ok(())
        } else {
            Err(format!(
                "expected {} at byte {}",
                expected as char, self.index
            ))
        }
    }

    fn try_consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<u8> {
        self.text.get(self.index).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.index += 1;
        Some(byte)
    }
}

fn parse_json_file(path: &Path) -> Result<BTreeMap<String, Json>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    match Parser::new(&text).parse()? {
        Json::Object(object) => Ok(object),
        _ => Err(format!("{}: expected JSON object", path.display())),
    }
}

fn get<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> Option<&'a Json> {
    object.get(key)
}

fn string_field(object: &BTreeMap<String, Json>, key: &str) -> Option<String> {
    get(object, key)
        .and_then(Json::as_str)
        .map(ToOwned::to_owned)
}

fn bool_field(object: &BTreeMap<String, Json>, key: &str) -> Option<bool> {
    match get(object, key) {
        Some(Json::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn object_field<'a>(
    object: &'a BTreeMap<String, Json>,
    key: &str,
) -> Option<&'a BTreeMap<String, Json>> {
    get(object, key).and_then(Json::as_object)
}

fn object_array_field<'a>(
    object: &'a BTreeMap<String, Json>,
    key: &str,
) -> Vec<&'a BTreeMap<String, Json>> {
    get(object, key)
        .and_then(Json::as_array)
        .map(|items| items.iter().filter_map(Json::as_object).collect())
        .unwrap_or_default()
}

fn string_array_field(object: &BTreeMap<String, Json>, key: &str) -> Vec<String> {
    get(object, key)
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Json::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn json_string(input: &str) -> String {
    format!("\"{}\"", json_escape(input))
}

fn json_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn json_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn display_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|value| value.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn path_from(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn non_empty(value: Option<&Json>) -> bool {
    match value {
        Some(Json::String(text)) => !text.trim().is_empty(),
        Some(Json::Array(items)) => !items.is_empty(),
        Some(Json::Object(object)) => !object.is_empty(),
        Some(Json::Null) | None => false,
        Some(_) => true,
    }
}

fn iter_strings(value: &Json, out: &mut Vec<String>) {
    match value {
        Json::String(text) => out.push(text.clone()),
        Json::Array(items) => {
            for item in items {
                iter_strings(item, out);
            }
        }
        Json::Object(object) => {
            for item in object.values() {
                iter_strings(item, out);
            }
        }
        Json::Null | Json::Bool(_) | Json::Number(_) => {}
    }
}

fn all_strings(object: &BTreeMap<String, Json>) -> Vec<String> {
    let mut out = Vec::new();
    for value in object.values() {
        iter_strings(value, &mut out);
    }
    out
}

fn set_from_slice(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn sorted_set(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn contains_oya_cli_pattern(lower: &str) -> Vec<(usize, usize)> {
    let patterns = [
        "local oya output",
        "local oya gate",
        "local oya verify",
        "bin/oya gate",
        "bin/oya verify",
        "oya gate",
        "oya verify",
    ];
    let mut matches = Vec::new();
    for pattern in patterns {
        let mut start = 0;
        while let Some(offset) = lower[start..].find(pattern) {
            let absolute = start + offset;
            matches.push((absolute, absolute + pattern.len()));
            start = absolute + pattern.len();
        }
    }
    matches.sort_unstable();
    matches.dedup();
    matches
}

fn negates_oya_authority_reference(text: &str, start: usize, end: usize) -> bool {
    let lower = text.to_lowercase();
    let window_start = start.saturating_sub(90);
    let window_end = lower.len().min(end + 120);
    let window = &lower[window_start..window_end];
    let has_not_authority = window.contains("is not protected-branch authority")
        || window.contains("is not ci authority")
        || window.contains("is not authority")
        || window.contains("not protected-branch authority")
        || window.contains("not ci authority")
        || window.contains("not authority");
    let has_prevent_authority = window.contains("prevent")
        && window.contains("from becoming")
        && window.contains("authority");
    let has_must_not_authority = window.contains("must not")
        && (window.contains("authority")
            || window.contains("satisfy")
            || window.contains("satisfied"));
    let has_cannot_authority = window.contains("cannot")
        && (window.contains("authority")
            || window.contains("satisfy")
            || window.contains("satisfied"));
    has_not_authority || has_prevent_authority || has_must_not_authority || has_cannot_authority
}

fn has_oya_cli_authority_reference(text: &str, allow_negated: bool) -> bool {
    let lower = text.to_lowercase();
    contains_oya_cli_pattern(&lower)
        .into_iter()
        .any(|(start, end)| !allow_negated || !negates_oya_authority_reference(text, start, end))
}

fn hard_authority_text(row: &BTreeMap<String, Json>) -> String {
    [
        "target_gate_or_controller",
        "evidence_path",
        "blocking_fixture",
    ]
    .iter()
    .filter_map(|key| string_field(row, key))
    .collect::<Vec<_>>()
    .join("\n")
}

fn row_maps_to_oya_cli_authority(row: &BTreeMap<String, Json>) -> bool {
    if bool_field(row, "no_new_oya_cli_surface") != Some(true) {
        return true;
    }
    if has_oya_cli_authority_reference(&hard_authority_text(row), false) {
        return true;
    }
    for key in [
        "source_artifact",
        "requirement",
        "claim_boundary",
        "human_judgment_reason",
        "owner",
        "retirement_phase",
    ] {
        if let Some(value) = string_field(row, key) {
            if has_oya_cli_authority_reference(&value, true) {
                return true;
            }
        }
    }
    false
}

fn unique_ids(rows: &[&BTreeMap<String, Json>]) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for row in rows {
        let Some(row_id) = string_field(row, "id").filter(|value| !value.is_empty()) else {
            continue;
        };
        if !seen.insert(row_id.clone()) {
            duplicates.insert(row_id);
        }
    }
    (seen, duplicates)
}

fn validate_rows(
    rows: &[&BTreeMap<String, Json>],
    required_fields: &[String],
    required_row_ids: &[String],
    allowed_classifications: &BTreeSet<String>,
) -> Vec<String> {
    let mut violations = Vec::new();
    let (row_ids, duplicates) = unique_ids(rows);
    if !duplicates.is_empty() {
        violations.push("duplicate_row_id".to_string());
    }

    if !required_row_ids.is_empty() {
        let missing = required_row_ids
            .iter()
            .filter(|row_id| !row_ids.contains(*row_id))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            violations.push("missing_required_row_id".to_string());
        }
    }

    for row in rows {
        if required_fields
            .iter()
            .any(|field| !non_empty(row.get(field)))
        {
            violations.push("missing_or_empty_required_field".to_string());
        }
        let classification = string_field(row, "classification").unwrap_or_default();
        if !allowed_classifications.contains(&classification) {
            violations.push("unknown_classification".to_string());
        }
        if classification == "not_automatable_human_judgment"
            && bool_field(row, "enforceable_or_automatable") == Some(true)
        {
            violations.push("enforceable_or_automatable_marked_human_judgment".to_string());
        }
        if row_maps_to_oya_cli_authority(row) {
            violations.push("blocking_invariant_mapped_to_oya_cli".to_string());
        }
        if classification == "not_automatable_human_judgment" {
            let has_reason = string_field(row, "human_judgment_reason")
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false);
            if !has_reason {
                violations.push("human_judgment_missing_irreducible_reason".to_string());
            }
        }
    }

    sorted_set(violations)
}

fn validate_coverage_registry(
    registry: &BTreeMap<String, Json>,
    matrix_row_ids: &BTreeSet<String>,
) -> CoverageSummary {
    let mut violations = Vec::new();
    let required_subject_ids = string_array_field(registry, "required_subject_ids")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let coverage_subjects = object_array_field(registry, "coverage_subjects");
    let subject_ids = coverage_subjects
        .iter()
        .filter_map(|subject| string_field(subject, "id"))
        .collect::<BTreeSet<_>>();

    let declared_row_ids = string_array_field(registry, "row_ids")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let row_ids = if declared_row_ids.is_empty() {
        matrix_row_ids.clone()
    } else {
        declared_row_ids
    };

    let missing_subjects = required_subject_ids
        .difference(&subject_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_subjects.is_empty() {
        violations.push("missing_required_coverage_subject_id".to_string());
    }

    let mut subject_seen = BTreeSet::new();
    let mut duplicate_subjects = BTreeSet::new();
    for subject in &coverage_subjects {
        if let Some(subject_id) = string_field(subject, "id") {
            if !subject_seen.insert(subject_id.clone()) {
                duplicate_subjects.insert(subject_id);
            }
        }
    }
    if !duplicate_subjects.is_empty() {
        violations.push("duplicate_coverage_subject_id".to_string());
    }

    let mut mapped_row_ids = BTreeSet::new();
    for subject in &coverage_subjects {
        let mapped = string_array_field(subject, "mapped_row_ids");
        if mapped.is_empty() {
            violations.push("coverage_subject_without_rows".to_string());
        }
        mapped_row_ids.extend(mapped);
    }

    let missing_mapped_row_ids = mapped_row_ids
        .difference(&row_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_mapped_row_ids.is_empty() {
        violations.push("coverage_mapped_row_missing".to_string());
    }

    let unmapped_row_ids = row_ids
        .difference(&mapped_row_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !unmapped_row_ids.is_empty() {
        violations.push("coverage_row_unmapped".to_string());
    }

    if all_strings(registry)
        .iter()
        .any(|text| has_oya_cli_authority_reference(text, false))
    {
        violations.push("blocking_invariant_mapped_to_oya_cli".to_string());
    }

    let boundary = object_field(registry, "claim_boundary");
    let green_boundary = boundary
        .map(|object| {
            bool_field(object, "p0_0_green") == Some(false)
                && bool_field(object, "phase0_complete") == Some(false)
        })
        .unwrap_or(false);
    if !green_boundary {
        violations.push("green_claim_boundary_without_live_authority".to_string());
    }

    CoverageSummary {
        required_subject_count: required_subject_ids.len(),
        coverage_subject_count: subject_ids.len(),
        mapped_row_count: mapped_row_ids.len(),
        missing_required_subject_ids: missing_subjects,
        missing_mapped_row_ids,
        unmapped_row_ids,
        violations: sorted_set(violations),
    }
}

fn expected_verdict(fixture: &BTreeMap<String, Json>) -> String {
    match string_field(fixture, "expected_verdict").as_deref() {
        Some("GREEN") => "GREEN".to_string(),
        _ => "RED".to_string(),
    }
}

fn fixture_id(fixture: &BTreeMap<String, Json>, path: &Path) -> String {
    string_field(fixture, "fixture_id")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string())
        })
}

fn validate_fixture(
    path: &Path,
    matrix: &BTreeMap<String, Json>,
    row_ids: &BTreeSet<String>,
    root: &Path,
) -> FixtureResult {
    let fixture = match parse_json_file(path) {
        Ok(value) => value,
        Err(error) => {
            return FixtureResult {
                path: display_path(path, root),
                fixture_id: display_path(path, root),
                expected_verdict: "RED".to_string(),
                expected_violations: Vec::new(),
                observed_violations: Vec::new(),
                fixture_passed: false,
                failures: vec![error],
            };
        }
    };
    let expected_verdict = expected_verdict(&fixture);
    let expected_violations = string_array_field(&fixture, "expected_violations");
    let fixture_id = fixture_id(&fixture, path);
    let required_fields = string_array_field(matrix, "required_row_fields");
    let required_fields = if required_fields.is_empty() {
        REQUIRED_ROW_FIELDS
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    } else {
        required_fields
    };
    let allowed_classifications = string_array_field(matrix, "classifications")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let allowed_classifications = if allowed_classifications.is_empty() {
        set_from_slice(ALLOWED_CLASSIFICATIONS)
    } else {
        allowed_classifications
    };

    let mut observed_violations = Vec::new();
    if fixture.contains_key("rows") {
        let rows = object_array_field(&fixture, "rows");
        observed_violations.extend(validate_rows(
            &rows,
            &required_fields,
            &string_array_field(&fixture, "required_row_ids"),
            &allowed_classifications,
        ));
    }
    if fixture.contains_key("coverage_subjects") {
        observed_violations.extend(validate_coverage_registry(&fixture, row_ids).violations);
    }
    if !fixture.contains_key("rows") && !fixture.contains_key("coverage_subjects") {
        observed_violations.push("fixture_missing_rows_or_coverage_subjects".to_string());
    }
    observed_violations = sorted_set(observed_violations);

    let observed = observed_violations.iter().cloned().collect::<BTreeSet<_>>();
    let expected = expected_violations.iter().cloned().collect::<BTreeSet<_>>();
    let mut failures = Vec::new();
    if expected_verdict == "GREEN" {
        if !observed.is_empty() {
            failures.push(format!(
                "{fixture_id}: GREEN automation-ratchet fixture produced violations {:?}",
                observed_violations
            ));
        }
        if !expected.is_empty() {
            failures.push(format!(
                "{fixture_id}: GREEN fixture must not list expected_violations"
            ));
        }
    } else {
        if observed.is_empty() {
            failures.push(format!(
                "{fixture_id}: RED automation-ratchet fixture must produce violations"
            ));
        }
        let missing_expected = expected.difference(&observed).cloned().collect::<Vec<_>>();
        if !missing_expected.is_empty() {
            failures.push(format!(
                "{fixture_id}: expected violations were not observed {:?}",
                missing_expected
            ));
        }
    }

    FixtureResult {
        path: display_path(path, root),
        fixture_id,
        expected_verdict,
        expected_violations,
        observed_violations,
        fixture_passed: failures.is_empty(),
        failures,
    }
}

fn default_fixture_paths(root: &Path, matrix: &BTreeMap<String, Json>) -> Vec<PathBuf> {
    object_field(matrix, "fixture_set")
        .map(|fixture_set| string_array_field(fixture_set, "all_fixture_paths"))
        .unwrap_or_default()
        .into_iter()
        .map(|value| path_from(root, &value))
        .collect()
}

pub fn default_config(repo_root: PathBuf) -> Config {
    Config {
        matrix: path_from(&repo_root, DEFAULT_MATRIX),
        coverage_registry: path_from(&repo_root, DEFAULT_COVERAGE_REGISTRY),
        fixtures: Vec::new(),
        repo_root,
    }
}

pub fn evaluate(config: &Config) -> Evaluation {
    let root = &config.repo_root;
    let matrix_path = config.matrix.clone();
    let coverage_path = config.coverage_registry.clone();
    let matrix = parse_json_file(&matrix_path).unwrap_or_else(|error| {
        let mut result = BTreeMap::new();
        result.insert("_parse_error".to_string(), Json::String(error));
        result
    });
    let coverage_registry = parse_json_file(&coverage_path).unwrap_or_else(|error| {
        let mut result = BTreeMap::new();
        result.insert("_parse_error".to_string(), Json::String(error));
        result
    });

    let required_row_fields = string_array_field(&matrix, "required_row_fields");
    let required_row_fields = if required_row_fields.is_empty() {
        REQUIRED_ROW_FIELDS
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    } else {
        required_row_fields
    };
    let allowed_classifications = string_array_field(&matrix, "classifications");
    let allowed_classifications = if allowed_classifications.is_empty() {
        ALLOWED_CLASSIFICATIONS
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    } else {
        allowed_classifications
    };
    let allowed_classification_set = allowed_classifications
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let rows = object_array_field(&matrix, "seed_rows");
    let (row_ids, duplicate_row_ids) = unique_ids(&rows);

    let mut failures = Vec::new();
    if let Some(error) = string_field(&matrix, "_parse_error") {
        failures.push(format!("matrix:parse_error:{error}"));
    }
    if let Some(error) = string_field(&coverage_registry, "_parse_error") {
        failures.push(format!("coverage_registry:parse_error:{error}"));
    }

    let mut matrix_violations = validate_rows(
        &rows,
        &required_row_fields,
        &string_array_field(&matrix, "required_seed_row_ids"),
        &allowed_classification_set,
    );
    if allowed_classification_set != set_from_slice(ALLOWED_CLASSIFICATIONS) {
        matrix_violations.push("classification_set_drift".to_string());
    }
    let required_field_set = required_row_fields.iter().cloned().collect::<BTreeSet<_>>();
    if required_field_set != set_from_slice(REQUIRED_ROW_FIELDS) {
        matrix_violations.push("required_row_fields_drift".to_string());
    }
    matrix_violations = sorted_set(matrix_violations);
    failures.extend(
        matrix_violations
            .iter()
            .map(|violation| format!("matrix:{violation}")),
    );

    let coverage_summary = validate_coverage_registry(&coverage_registry, &row_ids);
    failures.extend(
        coverage_summary
            .violations
            .iter()
            .map(|violation| format!("coverage_registry:{violation}")),
    );

    let fixture_paths = if config.fixtures.is_empty() {
        default_fixture_paths(root, &matrix)
    } else {
        config.fixtures.clone()
    };
    let fixture_results = fixture_paths
        .iter()
        .map(|path| validate_fixture(path, &matrix, &row_ids, root))
        .collect::<Vec<_>>();
    for fixture in &fixture_results {
        failures.extend(fixture.failures.iter().cloned());
    }

    let pass = failures.is_empty();
    Evaluation {
        authority_boundary: "automation-ratchet local/static fixture evidence only; this checker never posts statuses, mutates branch protection, or claims live required-context authority".to_string(),
        matrix: display_path(&matrix_path, root),
        coverage_registry: display_path(&coverage_path, root),
        required_row_fields,
        allowed_classifications,
        matrix_summary: MatrixSummary {
            row_count: rows.len(),
            required_seed_row_count: string_array_field(&matrix, "required_seed_row_ids").len(),
            duplicate_row_ids: duplicate_row_ids.into_iter().collect(),
            violations: matrix_violations,
        },
        coverage_registry_summary: coverage_summary,
        fixture_results,
        local_fixture_contract_proven: pass,
        coverage_registry_local_static_proven: pass,
        automation_ratchet_live: false,
        protected_branch_authority_proven: false,
        status_mutation_performed: false,
        live_required_context_execution_proven: false,
        p0_0_green: false,
        phase0_complete: false,
        verdict: if pass { "PASS" } else { "FAIL" }.to_string(),
        failures,
    }
}

pub fn to_json(evaluation: &Evaluation) -> String {
    let fixtures = evaluation
        .fixture_results
        .iter()
        .map(|fixture| {
            format!(
                "{{\"path\":{},\"fixture_id\":{},\"expected_verdict\":{},\"expected_violations\":{},\"observed_violations\":{},\"fixture_passed\":{},\"failures\":{}}}",
                json_string(&fixture.path),
                json_string(&fixture.fixture_id),
                json_string(&fixture.expected_verdict),
                json_string_array(&fixture.expected_violations),
                json_string_array(&fixture.observed_violations),
                json_bool(fixture.fixture_passed),
                json_string_array(&fixture.failures),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"authority_boundary\":{},\"matrix\":{},\"coverage_registry\":{},\"required_row_fields\":{},\"allowed_classifications\":{},\"matrix_summary\":{{\"row_count\":{},\"required_seed_row_count\":{},\"duplicate_row_ids\":{},\"violations\":{}}},\"coverage_registry_summary\":{{\"required_subject_count\":{},\"coverage_subject_count\":{},\"mapped_row_count\":{},\"missing_required_subject_ids\":{},\"missing_mapped_row_ids\":{},\"unmapped_row_ids\":{},\"violations\":{}}},\"fixture_results\":[{}],\"local_fixture_contract_proven\":{},\"coverage_registry_local_static_proven\":{},\"automation_ratchet_live\":{},\"protected_branch_authority_proven\":{},\"status_mutation_performed\":{},\"live_required_context_execution_proven\":{},\"p0_0_green\":{},\"phase0_complete\":{},\"verdict\":{},\"failures\":{}}}",
        json_string(&evaluation.authority_boundary),
        json_string(&evaluation.matrix),
        json_string(&evaluation.coverage_registry),
        json_string_array(&evaluation.required_row_fields),
        json_string_array(&evaluation.allowed_classifications),
        evaluation.matrix_summary.row_count,
        evaluation.matrix_summary.required_seed_row_count,
        json_string_array(&evaluation.matrix_summary.duplicate_row_ids),
        json_string_array(&evaluation.matrix_summary.violations),
        evaluation.coverage_registry_summary.required_subject_count,
        evaluation.coverage_registry_summary.coverage_subject_count,
        evaluation.coverage_registry_summary.mapped_row_count,
        json_string_array(
            &evaluation
                .coverage_registry_summary
                .missing_required_subject_ids
        ),
        json_string_array(&evaluation.coverage_registry_summary.missing_mapped_row_ids),
        json_string_array(&evaluation.coverage_registry_summary.unmapped_row_ids),
        json_string_array(&evaluation.coverage_registry_summary.violations),
        fixtures,
        json_bool(evaluation.local_fixture_contract_proven),
        json_bool(evaluation.coverage_registry_local_static_proven),
        json_bool(evaluation.automation_ratchet_live),
        json_bool(evaluation.protected_branch_authority_proven),
        json_bool(evaluation.status_mutation_performed),
        json_bool(evaluation.live_required_context_execution_proven),
        json_bool(evaluation.p0_0_green),
        json_bool(evaluation.phase0_complete),
        json_string(&evaluation.verdict),
        json_string_array(&evaluation.failures),
    )
}

fn parse_args() -> Result<(Config, bool), String> {
    let mut repo_root = env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("current directory"));
    let mut matrix: Option<PathBuf> = None;
    let mut coverage_registry: Option<PathBuf> = None;
    let mut fixtures = Vec::new();
    let mut json = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--repo-root requires a value".to_string())?,
                );
            }
            "--matrix" => {
                matrix = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--matrix requires a value".to_string())?,
                ));
            }
            "--coverage-registry" => {
                coverage_registry =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "--coverage-registry requires a value".to_string()
                    })?));
            }
            "--fixture" => {
                fixtures.push(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--fixture requires a value".to_string())?,
                ));
            }
            "--json" => json = true,
            "--help" | "-h" => {
                return Err("usage: assert-automation-ratchet [--repo-root PATH] [--matrix PATH] [--coverage-registry PATH] [--fixture PATH ...] [--json]".to_string());
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }

    let config = Config {
        matrix: matrix
            .map(|path| {
                if path.is_absolute() {
                    path
                } else {
                    repo_root.join(path)
                }
            })
            .unwrap_or_else(|| path_from(&repo_root, DEFAULT_MATRIX)),
        coverage_registry: coverage_registry
            .map(|path| {
                if path.is_absolute() {
                    path
                } else {
                    repo_root.join(path)
                }
            })
            .unwrap_or_else(|| path_from(&repo_root, DEFAULT_COVERAGE_REGISTRY)),
        fixtures: fixtures
            .into_iter()
            .map(|path| {
                if path.is_absolute() {
                    path
                } else {
                    repo_root.join(path)
                }
            })
            .collect(),
        repo_root,
    };
    Ok((config, json))
}

fn main() {
    let (config, json) = match parse_args() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let evaluation = evaluate(&config);
    let rendered = to_json(&evaluation);
    if json || evaluation.failures.is_empty() {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if !evaluation.failures.is_empty() {
        std::process::exit(1);
    }
}
