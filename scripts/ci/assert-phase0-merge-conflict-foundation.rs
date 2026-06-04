//! Validate AC-0.15 generated-artifact and merge-conflict foundation.
//!
//! This is local/static fixture evidence only. It proves the checked-in seed
//! registry, conflict taxonomy, merge-tree readiness fixtures, and one-lane-one-path
//! fail-closed cases are wired through Buck2. It never posts statuses, mutates
//! branch protection, presses a provider merge button, proves full generated-output
//! coverage, claims Phase-1 Tide batching, or claims P0.0/Phase-0 completion.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/generated-artifact-registry.json";
const REQUIRED_TAXONOMY_IDS: &[&str] = &[
    "clean_merge",
    "merge_tree_conflict",
    "path_overlap_without_review",
    "generated_artifact_missing_registry",
    "generated_artifact_stale_output",
    "one_lane_one_path_violation",
    "phase1_tide_batched_projection_overclaim",
];
const FALSE_CLAIMS: &[&str] = &[
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "phase1_tide_batching_claimed",
    "full_repo_generated_artifact_coverage_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];
// Policy anchors: forbidden_true_or_missing_claim_p0_0_green, phase1_tide_batching_claimed
const REQUIRED_ARTIFACT_FIELDS: &[&str] = &[
    "id",
    "output_path",
    "generator",
    "source_paths",
    "regeneration_command",
    "owner_team",
    "commit_policy",
    "drift_gate",
    "stale_output_policy",
    "path_claims",
];
const REQUIRED_AUTOMATED_CHAIN_TOKENS: &[&str] = &[
    "//:phase0-merge-conflict-foundation-check",
    "scripts/ci/assert-phase0-merge-conflict-foundation.rs",
    "scripts/tests/phase0_merge_conflict_foundation_check.rs",
    "git merge-tree --write-tree",
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

    fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(value) => Some(*value),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Json::Number(value) => value.parse().ok(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult {
    pub fixture_id: String,
    pub path: String,
    pub expected_verdict: String,
    pub expected_violations: Vec<String>,
    pub observed_violations: Vec<String>,
    pub expectation_failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
    pub authority_boundary: String,
    pub generated_artifact_registry_published: bool,
    pub merge_tree_fixture_contract_measured: bool,
    pub status_mutation_performed: bool,
    pub protected_branch_authority_proven: bool,
    pub live_required_context_execution_proven: bool,
    pub phase1_tide_batching_claimed: bool,
    pub full_repo_generated_artifact_coverage_proven: bool,
    pub p0_0_green: bool,
    pub phase0_complete: bool,
    pub production_ready: bool,
    pub hyperscaler_grade: bool,
    pub registry: String,
    pub registered_artifact_count: usize,
    pub taxonomy_count: usize,
    pub fixture_count: usize,
    pub expected_green_fixture_count: usize,
    pub expected_red_fixture_count: usize,
    pub fixtures: Vec<FixtureResult>,
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
            Some(b'"') => self.parse_string().map(Json::String),
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
        self.consume(b'"')?;
        let mut out = String::new();
        while let Some(byte) = self.next() {
            match byte {
                b'"' => return Ok(out),
                b'\\' => {
                    let escaped = self
                        .next()
                        .ok_or_else(|| "unterminated JSON string escape".to_string())?;
                    match escaped {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let value = self.parse_unicode_escape()?;
                            out.push(char::from_u32(value).unwrap_or('\u{fffd}'));
                        }
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

fn parse_json_file(path: &Path) -> Result<Json, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Parser::new(&text)
        .parse()
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn get<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> Option<&'a Json> {
    object.get(key)
}

fn object_field<'a>(
    object: &'a BTreeMap<String, Json>,
    key: &str,
) -> Option<&'a BTreeMap<String, Json>> {
    get(object, key).and_then(Json::as_object)
}

fn array_field<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> Vec<&'a Json> {
    get(object, key)
        .and_then(Json::as_array)
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}

fn object_array_field<'a>(
    object: &'a BTreeMap<String, Json>,
    key: &str,
) -> Vec<&'a BTreeMap<String, Json>> {
    array_field(object, key)
        .into_iter()
        .filter_map(Json::as_object)
        .collect()
}

fn string_array_field(object: &BTreeMap<String, Json>, key: &str) -> Vec<String> {
    array_field(object, key)
        .into_iter()
        .filter_map(Json::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn string_field(object: &BTreeMap<String, Json>, key: &str) -> Option<String> {
    get(object, key)
        .and_then(Json::as_str)
        .map(ToOwned::to_owned)
}

fn bool_field(object: &BTreeMap<String, Json>, key: &str) -> Option<bool> {
    get(object, key).and_then(Json::as_bool)
}

fn bool_is(object: &BTreeMap<String, Json>, key: &str, expected: bool) -> bool {
    bool_field(object, key) == Some(expected)
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

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn validate_false_claims(
    mapping: &BTreeMap<String, Json>,
    failures: &mut Vec<String>,
    prefix: &str,
) {
    for claim in FALSE_CLAIMS {
        if !bool_is(mapping, claim, false) {
            failures.push(format!("{prefix}forbidden_true_or_missing_claim_{claim}"));
        }
    }
}

fn validate_registry(
    root: &Path,
    registry: &BTreeMap<String, Json>,
) -> (Vec<String>, BTreeMap<String, BTreeMap<String, Json>>) {
    let mut failures = Vec::new();
    let empty = BTreeMap::new();
    let boundary = object_field(registry, "claim_boundary").unwrap_or(&empty);
    if !bool_is(boundary, "generated_artifact_registry_published", true) {
        failures.push("generated_artifact_registry_not_published".to_string());
    }
    if !bool_is(boundary, "merge_tree_fixture_contract_measured", true) {
        failures.push("merge_tree_fixture_contract_not_measured".to_string());
    }
    validate_false_claims(boundary, &mut failures, "");

    let scope = object_field(registry, "coverage_scope").unwrap_or(&empty);
    if !bool_is(scope, "full_repo_generated_artifact_coverage_proven", false) {
        failures.push(
            "forbidden_true_or_missing_coverage_scope_full_repo_generated_artifact_coverage_proven"
                .to_string(),
        );
    }

    let taxonomy_ids: BTreeSet<String> = object_array_field(registry, "conflict_taxonomy")
        .into_iter()
        .filter_map(|item| string_field(item, "id"))
        .collect();
    for required in REQUIRED_TAXONOMY_IDS {
        if !taxonomy_ids.contains(*required) {
            failures.push(format!("missing_conflict_taxonomy:{required}"));
        }
    }

    let readiness = object_field(registry, "merge_tree_readiness").unwrap_or(&empty);
    if string_field(readiness, "engine").as_deref() != Some("git merge-tree --write-tree") {
        failures.push("missing_git_merge_tree_write_tree_engine".to_string());
    }
    for key in [
        "mutates_working_tree",
        "mutates_index",
        "provider_side_merge_button_used",
        "phase1_tide_batching_claimed",
    ] {
        if !bool_is(readiness, key, false) {
            failures.push(format!(
                "merge_tree_readiness_forbidden_true_or_missing:{key}"
            ));
        }
    }

    let lane_policy = object_field(registry, "lane_ownership_policy").unwrap_or(&empty);
    for key in [
        "one_lane_one_path",
        "overlap_review_required",
        "generated_artifact_source_pairing_required",
        "owner_ack_required_for_overlap",
    ] {
        if !bool_is(lane_policy, key, true) {
            failures.push(format!("lane_ownership_policy_missing_true:{key}"));
        }
    }

    let automated_chain = string_array_field(registry, "automated_chain").join("\n");
    for token in REQUIRED_AUTOMATED_CHAIN_TOKENS {
        if !automated_chain.contains(token) {
            failures.push(format!("missing_automated_chain_token:{token}"));
        }
    }

    let artifacts = object_array_field(registry, "registered_artifacts");
    let minimum = get(registry, "minimum_registered_artifact_count").and_then(Json::as_i64);
    if minimum.is_some_and(|minimum| artifacts.len() < minimum as usize) {
        failures.push("registered_artifact_count_below_minimum".to_string());
    }
    if artifacts.is_empty() {
        failures.push("missing_registered_artifacts".to_string());
    }

    let mut id_counts: BTreeMap<String, usize> = BTreeMap::new();
    for artifact in &artifacts {
        if let Some(id) = string_field(artifact, "id") {
            *id_counts.entry(id).or_default() += 1;
        }
    }
    for (artifact_id, count) in id_counts {
        if count > 1 {
            failures.push(format!("duplicate_artifact_id:{artifact_id}"));
        }
    }

    let mut artifact_by_id = BTreeMap::new();
    for artifact in artifacts {
        let artifact_id =
            string_field(artifact, "id").unwrap_or_else(|| "<missing-id>".to_string());
        artifact_by_id.insert(artifact_id.clone(), artifact.clone());
        for field in REQUIRED_ARTIFACT_FIELDS {
            if !artifact.contains_key(*field) {
                failures.push(format!(
                    "{artifact_id}:missing_required_artifact_field:{field}"
                ));
            }
        }

        match string_field(artifact, "output_path") {
            Some(output_path) if !output_path.is_empty() => {
                let path = root.join(&output_path);
                if !path.is_file() {
                    failures.push(format!(
                        "{artifact_id}:artifact_output_path_missing:{output_path}"
                    ));
                } else if let Some(marker) = string_field(artifact, "generated_marker") {
                    if !marker.is_empty() && !read(&path).contains(&marker) {
                        failures.push(format!(
                            "{artifact_id}:generated_marker_missing:{output_path}"
                        ));
                    }
                }
            }
            _ => failures.push(format!(
                "{artifact_id}:artifact_output_path_missing_or_invalid"
            )),
        }

        let source_paths = string_array_field(artifact, "source_paths");
        if source_paths.is_empty() {
            failures.push(format!("{artifact_id}:artifact_source_paths_missing"));
        }
        for source in source_paths {
            if !root.join(&source).exists() {
                failures.push(format!(
                    "{artifact_id}:artifact_source_path_missing_or_invalid:{source}"
                ));
            }
        }

        match string_field(artifact, "regeneration_command") {
            Some(regen) if !regen.is_empty() && root.join(&regen).is_file() => {}
            _ => failures.push(format!(
                "{artifact_id}:regeneration_command_missing_or_invalid"
            )),
        }

        if string_field(artifact, "drift_gate").as_deref()
            != Some("//:phase0-merge-conflict-foundation-check")
        {
            failures.push(format!("{artifact_id}:missing_phase0_drift_gate"));
        }

        let path_claims = object_field(artifact, "path_claims").unwrap_or(&empty);
        if !bool_is(path_claims, "overlap_review_required", true) {
            failures.push(format!(
                "{artifact_id}:path_claim_missing_overlap_review_required"
            ));
        }
        if !bool_is(path_claims, "phase1_tide_batching_claimed", false) {
            failures.push(format!(
                "{artifact_id}:path_claim_phase1_tide_batching_overclaim"
            ));
        }
    }

    let fixture_set = object_field(registry, "fixture_set").unwrap_or(&empty);
    let fixtures = string_array_field(fixture_set, "required_fixture_paths");
    if fixtures.len() < 5 {
        failures.push("fixture_set_missing_required_fixture_paths".to_string());
    }
    for fixture in fixtures {
        if !root.join(&fixture).is_file() {
            failures.push(format!("fixture_path_missing:{fixture}"));
        }
    }

    (failures, artifact_by_id)
}

fn approved_overlap_paths(fixture: &BTreeMap<String, Json>) -> BTreeSet<String> {
    object_array_field(fixture, "approved_overlap_reviews")
        .into_iter()
        .filter_map(|item| {
            let path = string_field(item, "path")?;
            let review_id = string_field(item, "review_id")?;
            if !review_id.is_empty() && bool_is(item, "owner_ack", true) {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

fn validate_fixture(
    fixture: &BTreeMap<String, Json>,
    artifact_by_id: &BTreeMap<String, BTreeMap<String, Json>>,
) -> FixtureResult {
    let fixture_id =
        string_field(fixture, "fixture_id").unwrap_or_else(|| "<missing-fixture-id>".to_string());
    let mut expected_verdict =
        string_field(fixture, "expected_verdict").unwrap_or_else(|| "RED".to_string());
    if expected_verdict != "GREEN" && expected_verdict != "RED" {
        expected_verdict = "RED".to_string();
    }
    let expected_violations: BTreeSet<String> = string_array_field(fixture, "expected_violations")
        .into_iter()
        .collect();
    let mut observed = Vec::<String>::new();
    let empty = BTreeMap::new();

    let merge_tree = object_field(fixture, "merge_tree_simulation").unwrap_or(&empty);
    if string_field(merge_tree, "engine").as_deref() != Some("git merge-tree --write-tree") {
        observed.push("missing_git_merge_tree_write_tree_engine".to_string());
    }
    if string_field(merge_tree, "result").as_deref() != Some("clean") {
        observed.push("merge_tree_conflict".to_string());
    }
    for key in [
        "mutates_working_tree",
        "mutates_index",
        "provider_side_merge_button_used",
    ] {
        if !bool_is(merge_tree, key, false) {
            observed.push(format!(
                "merge_tree_simulation_forbidden_true_or_missing:{key}"
            ));
        }
    }

    let mut path_to_lanes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for lane in object_array_field(fixture, "lanes") {
        let lane_id =
            string_field(lane, "lane_id").unwrap_or_else(|| "<missing-lane-id>".to_string());
        if string_field(lane, "owner_team").is_none_or(|owner| owner.is_empty()) {
            observed.push("one_lane_one_path_violation".to_string());
        }
        for path in string_array_field(lane, "owned_paths") {
            path_to_lanes.entry(path).or_default().push(lane_id.clone());
        }
    }
    let approved = approved_overlap_paths(fixture);
    for (path, owners) in path_to_lanes {
        if owners.len() > 1 && !approved.contains(&path) {
            observed.push("path_overlap_without_review".to_string());
        }
    }

    for change in object_array_field(fixture, "generated_artifact_changes") {
        let artifact_id = string_field(change, "artifact_id");
        let registry_entry_present = bool_field(change, "registry_entry_present");
        let Some(artifact_id) = artifact_id else {
            observed.push("generated_artifact_missing_registry".to_string());
            continue;
        };
        let Some(artifact) = artifact_by_id.get(&artifact_id) else {
            observed.push("generated_artifact_missing_registry".to_string());
            continue;
        };
        if registry_entry_present == Some(false) {
            observed.push("generated_artifact_missing_registry".to_string());
            continue;
        }
        if string_field(change, "path") != string_field(artifact, "output_path") {
            observed.push("generated_artifact_missing_registry".to_string());
        }
        let sources: BTreeSet<String> = string_array_field(change, "source_paths")
            .into_iter()
            .collect();
        let required_sources: BTreeSet<String> = string_array_field(artifact, "source_paths")
            .into_iter()
            .collect();
        if sources.is_empty() || !required_sources.is_subset(&sources) {
            observed.push("generated_artifact_stale_output".to_string());
        }
        if string_field(change, "regeneration_command")
            != string_field(artifact, "regeneration_command")
        {
            observed.push("generated_artifact_stale_output".to_string());
        }
    }

    let boundary = object_field(fixture, "claim_boundary").unwrap_or(&empty);
    for claim in [
        "status_mutation_performed",
        "protected_branch_authority_proven",
        "live_required_context_execution_proven",
        "p0_0_green",
        "phase0_complete",
        "production_ready",
        "hyperscaler_grade",
    ] {
        if !bool_is(boundary, claim, false) {
            observed.push(format!("forbidden_true_or_missing_claim_{claim}"));
        }
    }
    if !bool_is(boundary, "phase1_tide_batching_claimed", false) {
        observed.push("phase1_tide_batched_projection_overclaim".to_string());
    }

    let observed_set: BTreeSet<String> = observed.into_iter().collect();
    let mut expectation_failures = Vec::new();
    if expected_verdict == "GREEN" && !observed_set.is_empty() {
        expectation_failures.push("GREEN merge-conflict fixture produced violations".to_string());
    }
    if expected_verdict == "RED" && observed_set.is_empty() {
        expectation_failures.push("RED merge-conflict fixture must produce violations".to_string());
    }
    for missing in expected_violations.difference(&observed_set) {
        expectation_failures.push(format!("expected_violation_missing:{missing}"));
    }

    FixtureResult {
        fixture_id,
        path: String::new(),
        expected_verdict,
        expected_violations: expected_violations.into_iter().collect(),
        observed_violations: observed_set.into_iter().collect(),
        expectation_failures,
    }
}

pub fn evaluate(root: &Path, registry_path: &Path, fixture_overrides: &[PathBuf]) -> Evaluation {
    let mut failures = Vec::new();
    let mut artifact_by_id = BTreeMap::new();
    let registry_json = if registry_path.is_file() {
        match parse_json_file(registry_path) {
            Ok(Json::Object(object)) => Some(object),
            Ok(_) => {
                failures.push("generated_artifact_registry_not_object".to_string());
                None
            }
            Err(error) => {
                failures.push(format!("generated_artifact_registry_parse_error:{error}"));
                None
            }
        }
    } else {
        failures.push("missing_generated_artifact_registry".to_string());
        None
    };

    if let Some(registry) = &registry_json {
        let (registry_failures, artifacts) = validate_registry(root, registry);
        failures.extend(registry_failures);
        artifact_by_id = artifacts;
    }

    let fixture_paths = if fixture_overrides.is_empty() {
        registry_json
            .as_ref()
            .and_then(|registry| object_field(registry, "fixture_set"))
            .map(|fixture_set| string_array_field(fixture_set, "required_fixture_paths"))
            .unwrap_or_default()
            .into_iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>()
    } else {
        fixture_overrides.to_vec()
    };

    let mut fixture_results = Vec::new();
    for fixture_value in fixture_paths {
        let fixture_path = if fixture_value.is_absolute() {
            fixture_value
        } else {
            root.join(fixture_value)
        };
        if !fixture_path.is_file() {
            failures.push(format!(
                "fixture_path_missing:{}",
                display_path(&fixture_path, root)
            ));
            continue;
        }
        match parse_json_file(&fixture_path) {
            Ok(Json::Object(fixture)) => {
                let mut result = validate_fixture(&fixture, &artifact_by_id);
                result.path = display_path(&fixture_path, root);
                failures.extend(result.expectation_failures.clone());
                fixture_results.push(result);
            }
            Ok(_) => failures.push(format!(
                "fixture_not_object:{}",
                display_path(&fixture_path, root)
            )),
            Err(error) => failures.push(format!("fixture_parse_error:{error}")),
        }
    }

    let registry_artifacts = registry_json
        .as_ref()
        .map(|registry| object_array_field(registry, "registered_artifacts").len())
        .unwrap_or(0);
    let taxonomy_count = registry_json
        .as_ref()
        .map(|registry| {
            object_array_field(registry, "conflict_taxonomy")
                .into_iter()
                .filter_map(|item| string_field(item, "id"))
                .collect::<BTreeSet<_>>()
                .len()
        })
        .unwrap_or(0);
    let expected_green = fixture_results
        .iter()
        .filter(|item| item.expected_verdict == "GREEN")
        .count();
    let expected_red = fixture_results
        .iter()
        .filter(|item| item.expected_verdict == "RED")
        .count();

    let unique_failures: Vec<String> = failures
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let pass = unique_failures.is_empty();
    Evaluation {
        verdict: if pass { "PASS" } else { "FAIL" }.to_string(),
        failures: unique_failures,
        authority_boundary: "AC-0.15 local/static generated-artifact registry and merge-tree readiness evidence only; no status mutation, live required-context authority, Phase-1 Tide batching, full-repo generated-artifact coverage, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven".to_string(),
        generated_artifact_registry_published: registry_json.is_some() && pass,
        merge_tree_fixture_contract_measured: registry_json.is_some() && pass,
        status_mutation_performed: false,
        protected_branch_authority_proven: false,
        live_required_context_execution_proven: false,
        phase1_tide_batching_claimed: false,
        full_repo_generated_artifact_coverage_proven: false,
        p0_0_green: false,
        phase0_complete: false,
        production_ready: false,
        hyperscaler_grade: false,
        registry: if registry_path.exists() {
            display_path(registry_path, root)
        } else {
            registry_path.to_string_lossy().to_string()
        },
        registered_artifact_count: registry_artifacts,
        taxonomy_count,
        fixture_count: fixture_results.len(),
        expected_green_fixture_count: expected_green,
        expected_red_fixture_count: expected_red,
        fixtures: fixture_results,
    }
}

pub fn to_json(evaluation: &Evaluation) -> String {
    let fixtures = evaluation
        .fixtures
        .iter()
        .map(|fixture| {
            format!(
                "{{\"expectation_failures\":{},\"expected_verdict\":{},\"expected_violations\":{},\"fixture_id\":{},\"observed_violations\":{},\"path\":{}}}",
                json_string_array(&fixture.expectation_failures),
                json_string(&fixture.expected_verdict),
                json_string_array(&fixture.expected_violations),
                json_string(&fixture.fixture_id),
                json_string_array(&fixture.observed_violations),
                json_string(&fixture.path),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"authority_boundary\":{},",
            "\"expected_green_fixture_count\":{},",
            "\"expected_red_fixture_count\":{},",
            "\"failures\":{},",
            "\"fixture_count\":{},",
            "\"fixtures\":[{}],",
            "\"full_repo_generated_artifact_coverage_proven\":{},",
            "\"generated_artifact_registry_published\":{},",
            "\"hyperscaler_grade\":{},",
            "\"live_required_context_execution_proven\":{},",
            "\"merge_tree_fixture_contract_measured\":{},",
            "\"p0_0_green\":{},",
            "\"phase0_complete\":{},",
            "\"phase1_tide_batching_claimed\":{},",
            "\"production_ready\":{},",
            "\"protected_branch_authority_proven\":{},",
            "\"registered_artifact_count\":{},",
            "\"registry\":{},",
            "\"status_mutation_performed\":{},",
            "\"taxonomy_count\":{},",
            "\"verdict\":{}",
            "}}"
        ),
        json_string(&evaluation.authority_boundary),
        evaluation.expected_green_fixture_count,
        evaluation.expected_red_fixture_count,
        json_string_array(&evaluation.failures),
        evaluation.fixture_count,
        fixtures,
        json_bool(evaluation.full_repo_generated_artifact_coverage_proven),
        json_bool(evaluation.generated_artifact_registry_published),
        json_bool(evaluation.hyperscaler_grade),
        json_bool(evaluation.live_required_context_execution_proven),
        json_bool(evaluation.merge_tree_fixture_contract_measured),
        json_bool(evaluation.p0_0_green),
        json_bool(evaluation.phase0_complete),
        json_bool(evaluation.phase1_tide_batching_claimed),
        json_bool(evaluation.production_ready),
        json_bool(evaluation.protected_branch_authority_proven),
        evaluation.registered_artifact_count,
        json_string(&evaluation.registry),
        json_bool(evaluation.status_mutation_performed),
        evaluation.taxonomy_count,
        json_string(&evaluation.verdict),
    )
}

fn usage() -> &'static str {
    "usage: assert-phase0-merge-conflict-foundation [--repo-root PATH] [--registry PATH] [--fixture PATH ...] [--json]"
}

pub fn run_cli() -> i32 {
    let mut repo_root = PathBuf::from(".");
    let mut registry = PathBuf::from(DEFAULT_REGISTRY);
    let mut fixtures = Vec::new();
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = args.next() else {
                    eprintln!("{}", usage());
                    return 2;
                };
                repo_root = PathBuf::from(value);
            }
            "--registry" => {
                let Some(value) = args.next() else {
                    eprintln!("{}", usage());
                    return 2;
                };
                registry = PathBuf::from(value);
            }
            "--fixture" => {
                let Some(value) = args.next() else {
                    eprintln!("{}", usage());
                    return 2;
                };
                fixtures.push(PathBuf::from(value));
            }
            "--json" => json = true,
            "-h" | "--help" => {
                println!("{}", usage());
                return 0;
            }
            other => {
                eprintln!("unexpected argument: {other}\n{}", usage());
                return 2;
            }
        }
    }

    let root = repo_root.canonicalize().unwrap_or(repo_root);
    let registry_path = if registry.is_absolute() {
        registry
    } else {
        root.join(registry)
    };
    let evaluation = evaluate(&root, &registry_path, &fixtures);
    let rendered = to_json(&evaluation);
    if json || evaluation.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if evaluation.verdict == "PASS" { 0 } else { 1 }
}

#[cfg(not(test))]
fn main() {
    std::process::exit(run_cli());
}
