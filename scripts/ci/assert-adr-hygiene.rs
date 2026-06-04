//! Validate AC-0.3 ADR numbering, supersession, and active-doc hygiene.
//!
//! This checker is local/static fixture evidence only. It scans ADR frontmatter
//! for duplicate ids, verifies the ADR-0377 renumbering and ADR-0511 -> ADR-0513
//! supersession contract, and lints active docs for stale canonical references
//! to superseded decisions. It never mutates branch protection, posts statuses,
//! regenerates the full ADR index, or claims P0.0/Phase-0 completion.

#[allow(dead_code)]
#[path = "assert-result-bundle-output.rs"]
mod json_support;

pub use json_support::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/adr-hygiene-registry.json";
const DEFAULT_FIXTURE_DIR: &str = "specs/fixtures/phase0-adr-hygiene";
const AUTHORITY_BOUNDARY: &str = "AC-0.3 local/static ADR hygiene evidence only; no status mutation, live required-context authority, full ADR index regeneration, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven";
const FALSE_CLAIMS: [&str; 8] = [
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "full_adr_index_regenerated",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionRecord {
    pub path: String,
    pub id: String,
    pub filename_id: Option<String>,
    pub status: String,
    pub superseded_by: Vec<String>,
    pub renumbered_from: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveDocument {
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureValidation {
    pub path: String,
    pub fixture_id: String,
    pub expected_verdict: String,
    pub expected_violations: Vec<String>,
    pub observed_violations: Vec<String>,
    pub fixture_passed: bool,
    pub failures: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistrySummary {
    pub decision_record_count: usize,
    pub active_doc_scan_count: usize,
    pub superseded_reference_pattern_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub authority_boundary: String,
    pub adr_hygiene_registry_published: bool,
    pub adr_hygiene_fixture_contract_measured: bool,
    pub registry_summary: RegistrySummary,
    pub fixture_count: usize,
    pub expected_green_fixture_count: usize,
    pub expected_red_fixture_count: usize,
    pub fixture_results: Vec<FixtureValidation>,
    pub status_mutation_performed: bool,
    pub protected_branch_authority_proven: bool,
    pub live_required_context_execution_proven: bool,
    pub full_adr_index_regenerated: bool,
    pub p0_0_green: bool,
    pub phase0_complete: bool,
    pub production_ready: bool,
    pub hyperscaler_grade: bool,
    pub verdict: String,
    pub failures: Vec<String>,
}

pub fn parse_json(text: &str) -> Result<Json, String> {
    json_support::parse_json(text)
}

pub fn load_json(path: &Path) -> Result<Json, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("read {} failed: {error}", path.display()))?;
    parse_json(&text).map_err(|error| format!("parse {} failed: {error}", path.display()))
}

fn object_field<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> Option<&'a Json> {
    object.get(key)
}

fn string_field(object: &BTreeMap<String, Json>, key: &str) -> Option<String> {
    object_field(object, key)
        .and_then(Json::as_str)
        .map(str::to_string)
}

fn bool_field(object: &BTreeMap<String, Json>, key: &str) -> Option<bool> {
    object_field(object, key).and_then(Json::as_bool)
}

fn string_list(value: Option<&Json>) -> Vec<String> {
    value
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Json::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn object_list(value: Option<&Json>) -> Vec<&BTreeMap<String, Json>> {
    value
        .and_then(Json::as_array)
        .map(|items| items.iter().filter_map(Json::as_object).collect())
        .unwrap_or_default()
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn display_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn parse_frontmatter_list(raw: &str) -> Vec<String> {
    let value = raw.trim();
    if value.is_empty() {
        return Vec::new();
    }
    if let Some(inner) = value
        .strip_prefix('[')
        .and_then(|item| item.strip_suffix(']'))
    {
        let inner = inner.trim();
        if inner.is_empty() {
            return Vec::new();
        }
        return inner
            .split(',')
            .map(|item| item.trim().trim_matches(['\'', '"']).to_string())
            .filter(|item| !item.is_empty())
            .collect();
    }
    vec![value.trim_matches(['\'', '"']).to_string()]
}

fn filename_adr_id(name: &str) -> Option<String> {
    if name.len() >= 8
        && name.starts_with("ADR-")
        && name[4..8].chars().all(|ch| ch.is_ascii_digit())
    {
        Some(name[..8].to_string())
    } else {
        None
    }
}

fn parse_adr_frontmatter(path: &Path, root: &Path) -> DecisionRecord {
    let text = fs::read_to_string(path).unwrap_or_default();
    let mut fields = BTreeMap::<String, String>::new();
    let mut list_fields = BTreeMap::<String, Vec<String>>::new();
    if let Some(rest) = text.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---") {
            for line in rest[..end].lines() {
                if line.trim().is_empty() || line.starts_with(' ') || !line.contains(':') {
                    continue;
                }
                let Some((key, value)) = line.split_once(':') else {
                    continue;
                };
                let key = key.trim().to_string();
                let value = value.trim();
                if matches!(key.as_str(), "superseded_by" | "supersedes" | "related") {
                    list_fields.insert(key, parse_frontmatter_list(value));
                } else {
                    fields.insert(key, value.trim_matches('"').to_string());
                }
            }
        }
    }
    let filename_id = path
        .file_name()
        .and_then(|name| filename_adr_id(&name.to_string_lossy()));
    DecisionRecord {
        path: display_path(path, root),
        id: fields
            .get("id")
            .cloned()
            .or_else(|| filename_id.clone())
            .unwrap_or_else(|| "<missing-id>".to_string()),
        filename_id,
        status: fields.get("status").cloned().unwrap_or_default(),
        superseded_by: list_fields.remove("superseded_by").unwrap_or_default(),
        renumbered_from: fields.get("renumbered_from").cloned().unwrap_or_default(),
    }
}

fn repo_decision_records(root: &Path) -> Vec<DecisionRecord> {
    let decisions = root.join("docs/decisions");
    let mut paths = fs::read_dir(decisions)
        .unwrap_or_else(|_| panic!("read docs/decisions"))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("ADR-") && name.ends_with(".md"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .iter()
        .map(|path| parse_adr_frontmatter(path, root))
        .collect()
}

fn expand_scan_glob(root: &Path, pattern: &str) -> Vec<PathBuf> {
    if let Some((dir, suffix)) = pattern.split_once("/*") {
        let mut paths = fs::read_dir(root.join(dir))
            .into_iter()
            .flat_map(|entries| entries.filter_map(Result::ok))
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(suffix))
            })
            .collect::<Vec<_>>();
        paths.sort();
        return paths;
    }
    let path = root.join(pattern);
    if path.is_file() {
        vec![path]
    } else {
        Vec::new()
    }
}

fn simple_match(pattern: &str, value: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        value.starts_with(prefix)
    } else {
        value == pattern
    }
}

fn repo_active_documents(
    root: &Path,
    globs: &[String],
    exclude_globs: &[String],
) -> Vec<ActiveDocument> {
    let mut docs = Vec::new();
    let mut seen = BTreeSet::new();
    for pattern in globs {
        for path in expand_scan_glob(root, pattern) {
            let rel = display_path(&path, root);
            if rel.starts_with("docs/decisions/") || rel.starts_with("docs/machine-readable/") {
                continue;
            }
            if exclude_globs
                .iter()
                .any(|excluded| simple_match(excluded, &rel))
            {
                continue;
            }
            if seen.insert(rel.clone()) {
                docs.push(ActiveDocument {
                    path: rel,
                    content: fs::read_to_string(path).unwrap_or_default(),
                });
            }
        }
    }
    docs
}

fn validate_false_claims(
    mapping: &BTreeMap<String, Json>,
    failures: &mut Vec<String>,
    prefix: &str,
) {
    for claim in FALSE_CLAIMS {
        if bool_field(mapping, claim) != Some(false) {
            failures.push(format!("{prefix}forbidden_true_or_missing_claim_{claim}"));
        }
    }
}

fn record_from_object(object: &BTreeMap<String, Json>) -> DecisionRecord {
    DecisionRecord {
        path: string_field(object, "path").unwrap_or_else(|| "<missing-path>".to_string()),
        id: string_field(object, "id").unwrap_or_else(|| "<missing-id>".to_string()),
        filename_id: string_field(object, "filename_id"),
        status: string_field(object, "status").unwrap_or_default(),
        superseded_by: string_list(object_field(object, "superseded_by")),
        renumbered_from: string_field(object, "renumbered_from").unwrap_or_default(),
    }
}

fn doc_from_object(object: &BTreeMap<String, Json>) -> ActiveDocument {
    ActiveDocument {
        path: string_field(object, "path").unwrap_or_else(|| "<missing-path>".to_string()),
        content: string_field(object, "content").unwrap_or_default(),
    }
}

fn string_between_within(text: &str, left: &str, right: &str, max_gap: usize) -> bool {
    let mut start = 0;
    while let Some(offset) = text[start..].find(left) {
        let left_start = start + offset;
        let right_start = left_start + left.len();
        let tail = &text[right_start..];
        if let Some(right_offset) = tail.find(right) {
            if tail[..right_offset].chars().count() <= max_gap {
                return true;
            }
        }
        start = right_start;
    }
    false
}

fn matches_superseded_pattern(id: &str, content: &str) -> Option<bool> {
    let lower = content.to_ascii_lowercase();
    match id {
        "solidjs_as_canonical" => Some(
            string_between_within(&lower, "solidjs", "canonical", 80)
                || string_between_within(&lower, "canonical", "solidjs", 80)
                || lower.contains("shell = solidjs")
                || lower.contains("shell=solidjs"),
        ),
        "victoriametrics_as_canonical" => Some(
            lower.contains("victoriametrics for metrics")
                || string_between_within(&lower, "victoriametrics", "canonical", 80)
                || string_between_within(&lower, "canonical", "victoriametrics", 80),
        ),
        "foundry_as_live" => Some(
            lower.contains("foundry-as-live")
                || string_between_within(&lower, "foundry", "live authority", 80)
                || string_between_within(&lower, "foundry", "canonical live", 80),
        ),
        _ => None,
    }
}

fn validate_dataset(
    records: &[DecisionRecord],
    active_docs: &[ActiveDocument],
    registry: &BTreeMap<String, Json>,
    enforce_required_records: bool,
) -> Vec<String> {
    let mut failures = Vec::new();
    let mut by_id = BTreeMap::<String, Vec<String>>::new();
    let mut by_path = BTreeMap::<String, DecisionRecord>::new();
    for record in records {
        by_id
            .entry(record.id.clone())
            .or_default()
            .push(record.path.clone());
        by_path.insert(record.path.clone(), record.clone());
        if let Some(filename_id) = &record.filename_id {
            if filename_id != &record.id {
                failures.push("adr_filename_id_mismatch".to_string());
                failures.push(format!(
                    "adr_filename_id_mismatch:{}:{}!={}",
                    record.path, filename_id, record.id
                ));
            }
        }
    }
    for (id, paths) in by_id {
        if paths.len() > 1 {
            failures.push("duplicate_adr_number".to_string());
            failures.push(format!(
                "duplicate_adr_number:{id}:{}",
                sorted_unique(paths).join(",")
            ));
        }
    }

    let renumbering = registry
        .get("renumbering_contract")
        .and_then(Json::as_object);
    if let Some(renumbering) = renumbering {
        let kept = string_field(renumbering, "kept_decision_path");
        let renumbered = string_field(renumbering, "renumbered_decision_path");
        if enforce_required_records {
            if let Some(kept) = &kept {
                if !by_path.contains_key(kept) {
                    failures.push("adr_0377_kept_decision_missing".to_string());
                }
            }
            if let Some(renumbered) = &renumbered {
                match by_path.get(renumbered) {
                    Some(record) => {
                        if Some(record.id.as_str())
                            != string_field(renumbering, "renumbered_to").as_deref()
                        {
                            failures.push("renumbered_adr_id_mismatch".to_string());
                        }
                        if Some(record.renumbered_from.as_str())
                            != string_field(renumbering, "renumbered_from").as_deref()
                        {
                            failures.push("renumbered_adr_missing_renumbered_from".to_string());
                        }
                    }
                    None => failures.push("renumbered_adr_decision_missing".to_string()),
                }
            }
            for forbidden in string_list(renumbering.get("forbidden_live_paths")) {
                if by_path.contains_key(&forbidden) {
                    failures.push("forbidden_duplicate_adr_path_present".to_string());
                }
            }
        } else if let Some(renumbered) = &renumbered {
            if let Some(record) = by_path.get(renumbered) {
                if Some(record.id.as_str()) != string_field(renumbering, "renumbered_to").as_deref()
                {
                    failures.push("renumbered_adr_id_mismatch".to_string());
                }
                if Some(record.renumbered_from.as_str())
                    != string_field(renumbering, "renumbered_from").as_deref()
                {
                    failures.push("renumbered_adr_missing_renumbered_from".to_string());
                }
            }
        }
    }

    for contract in object_list(registry.get("supersession_contracts")) {
        let decision_id = string_field(contract, "decision_id").unwrap_or_default();
        let required = string_field(contract, "required_superseded_by").unwrap_or_default();
        let status_contains =
            string_field(contract, "required_status_contains").unwrap_or_default();
        let matches = records
            .iter()
            .filter(|record| record.id == decision_id)
            .collect::<Vec<_>>();
        if matches.is_empty() {
            if enforce_required_records {
                failures.push(format!("supersession_decision_missing:{decision_id}"));
            }
            continue;
        }
        for record in matches {
            if !status_contains.is_empty() && !record.status.contains(&status_contains) {
                failures.push(format!(
                    "{}_status_not_superseded",
                    decision_id.to_ascii_lowercase()
                ));
            }
            if !required.is_empty() && !record.superseded_by.iter().any(|item| item == &required) {
                failures.push(format!(
                    "{}_missing_superseded_by_{}",
                    decision_id.to_ascii_lowercase(),
                    required.to_ascii_lowercase()
                ));
                if decision_id == "ADR-0511" && required == "ADR-0513" {
                    failures.push("adr_0511_missing_superseded_by_adr_0513".to_string());
                }
            }
        }
    }

    for pattern in object_list(registry.get("superseded_reference_patterns")) {
        let id = string_field(pattern, "id").unwrap_or_else(|| "<missing-pattern-id>".to_string());
        let Some(_) = string_field(pattern, "pattern") else {
            failures.push(format!("invalid_superseded_reference_pattern:{id}"));
            continue;
        };
        let Some(is_matcher_known) = matches_superseded_pattern(&id, "") else {
            failures.push(format!("unsupported_superseded_reference_pattern:{id}"));
            continue;
        };
        debug_assert!(!is_matcher_known);
        for doc in active_docs {
            if matches_superseded_pattern(&id, &doc.content).unwrap_or(false) {
                failures.push("superseded_reference_active_doc".to_string());
                failures.push(format!("superseded_reference_active_doc:{id}:{}", doc.path));
            }
        }
    }
    failures
}

fn registry_summary(
    root: &Path,
    registry: &BTreeMap<String, Json>,
) -> (Vec<String>, RegistrySummary) {
    let mut failures = Vec::new();
    let boundary = registry
        .get("claim_boundary")
        .and_then(Json::as_object)
        .cloned()
        .unwrap_or_default();
    for required_true in [
        "adr_hygiene_registry_published",
        "adr_hygiene_fixture_contract_measured",
        "duplicate_adr_0377_resolved",
        "adr_0511_superseded_by_adr_0513",
        "superseded_reference_lint_measured",
    ] {
        if bool_field(&boundary, required_true) != Some(true) {
            failures.push(format!("claim_boundary_missing_true_{required_true}"));
        }
    }
    validate_false_claims(&boundary, &mut failures, "");

    let records = repo_decision_records(root);
    let active_docs = repo_active_documents(
        root,
        &string_list(registry.get("active_doc_scan_globs")),
        &string_list(registry.get("active_doc_scan_exclude_globs")),
    );
    failures.extend(validate_dataset(&records, &active_docs, registry, true));
    let summary = RegistrySummary {
        decision_record_count: records.len(),
        active_doc_scan_count: active_docs.len(),
        superseded_reference_pattern_count: object_list(
            registry.get("superseded_reference_patterns"),
        )
        .len(),
    };
    (failures, summary)
}

fn validate_fixture(
    path: String,
    fixture: &BTreeMap<String, Json>,
    registry: &BTreeMap<String, Json>,
) -> FixtureValidation {
    let fixture_id =
        string_field(fixture, "fixture_id").unwrap_or_else(|| "<missing-fixture-id>".to_string());
    let expected_verdict = match string_field(fixture, "expected_verdict").as_deref() {
        Some("GREEN") => "GREEN".to_string(),
        _ => "RED".to_string(),
    };
    let expected_violations = sorted_unique(string_list(fixture.get("expected_violations")));
    let mut observed = Vec::new();
    let boundary = fixture
        .get("claim_boundary")
        .and_then(Json::as_object)
        .cloned()
        .unwrap_or_default();
    validate_false_claims(&boundary, &mut observed, "");
    let records = object_list(fixture.get("decision_records"))
        .into_iter()
        .map(record_from_object)
        .collect::<Vec<_>>();
    let active_docs = object_list(fixture.get("active_documents"))
        .into_iter()
        .map(doc_from_object)
        .collect::<Vec<_>>();
    observed.extend(validate_dataset(&records, &active_docs, registry, false));

    let mut observed_set = observed.iter().cloned().collect::<BTreeSet<_>>();
    for item in &observed {
        if item.starts_with("duplicate_adr_number:") {
            observed_set.insert("duplicate_adr_number".to_string());
        }
        if item.starts_with("superseded_reference_active_doc:") {
            observed_set.insert("superseded_reference_active_doc".to_string());
        }
        if item.starts_with("adr_filename_id_mismatch:") {
            observed_set.insert("adr_filename_id_mismatch".to_string());
        }
    }
    let observed_violations = observed_set.into_iter().collect::<Vec<_>>();
    let observed_lookup = observed_violations.iter().cloned().collect::<BTreeSet<_>>();
    let expected_lookup = expected_violations.iter().cloned().collect::<BTreeSet<_>>();
    let mut failures = Vec::new();
    if expected_verdict == "GREEN" {
        if !observed_violations.is_empty() {
            failures.push(format!(
                "{fixture_id}: GREEN ADR hygiene fixture produced violations {:?}",
                observed_violations
            ));
        }
        if !expected_violations.is_empty() {
            failures.push(format!(
                "{fixture_id}: GREEN fixture must not list expected_violations"
            ));
        }
    } else {
        if observed_violations.is_empty() {
            failures.push(format!(
                "{fixture_id}: RED ADR hygiene fixture must produce violations"
            ));
        }
        let missing = expected_lookup
            .difference(&observed_lookup)
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            failures.push(format!(
                "{fixture_id}: expected violations were not observed {:?}",
                missing
            ));
        }
    }
    FixtureValidation {
        path,
        fixture_id,
        expected_verdict,
        expected_violations,
        observed_violations,
        fixture_passed: failures.is_empty(),
        failures,
    }
}

fn default_fixture_inputs(root: &Path) -> Vec<(String, Json)> {
    let mut paths = fs::read_dir(root.join(DEFAULT_FIXTURE_DIR))
        .unwrap_or_else(|_| panic!("read {DEFAULT_FIXTURE_DIR}"))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let display = display_path(&path, root);
            let json = load_json(&path).unwrap_or_else(|error| panic!("{error}"));
            (display, json)
        })
        .collect()
}

pub fn evaluate_sources(root: &Path, registry: &Json, fixtures: &[(String, Json)]) -> Report {
    let mut failures = Vec::new();
    let Some(registry_object) = registry.as_object() else {
        failures.push("missing_adr_hygiene_registry".to_string());
        return Report {
            authority_boundary: AUTHORITY_BOUNDARY.to_string(),
            adr_hygiene_registry_published: false,
            adr_hygiene_fixture_contract_measured: false,
            registry_summary: RegistrySummary {
                decision_record_count: 0,
                active_doc_scan_count: 0,
                superseded_reference_pattern_count: 0,
            },
            fixture_count: 0,
            expected_green_fixture_count: 0,
            expected_red_fixture_count: 0,
            fixture_results: Vec::new(),
            status_mutation_performed: false,
            protected_branch_authority_proven: false,
            live_required_context_execution_proven: false,
            full_adr_index_regenerated: false,
            p0_0_green: false,
            phase0_complete: false,
            production_ready: false,
            hyperscaler_grade: false,
            verdict: "FAIL".to_string(),
            failures,
        };
    };
    let (registry_failures, summary) = registry_summary(root, registry_object);
    failures.extend(registry_failures);
    let mut fixture_results = Vec::new();
    for (path, fixture) in fixtures {
        let Some(fixture_object) = fixture.as_object() else {
            failures.push(format!("fixture_not_object:{path}"));
            continue;
        };
        let result = validate_fixture(path.clone(), fixture_object, registry_object);
        failures.extend(result.failures.clone());
        fixture_results.push(result);
    }
    let expected_green = fixture_results
        .iter()
        .filter(|item| item.expected_verdict == "GREEN")
        .count();
    let expected_red = fixture_results
        .iter()
        .filter(|item| item.expected_verdict == "RED")
        .count();
    failures = sorted_unique(failures);
    let verdict = if failures.is_empty() { "PASS" } else { "FAIL" }.to_string();
    Report {
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
        adr_hygiene_registry_published: registry_object
            .get("claim_boundary")
            .and_then(Json::as_object)
            .and_then(|boundary| bool_field(boundary, "adr_hygiene_registry_published"))
            == Some(true),
        adr_hygiene_fixture_contract_measured: failures.is_empty(),
        registry_summary: summary,
        fixture_count: fixture_results.len(),
        expected_green_fixture_count: expected_green,
        expected_red_fixture_count: expected_red,
        fixture_results,
        status_mutation_performed: false,
        protected_branch_authority_proven: false,
        live_required_context_execution_proven: false,
        full_adr_index_regenerated: false,
        p0_0_green: false,
        phase0_complete: false,
        production_ready: false,
        hyperscaler_grade: false,
        verdict,
        failures,
    }
}

pub fn evaluate(root: &Path, registry_path: &Path, fixture_paths: &[PathBuf]) -> Report {
    let registry = match load_json(registry_path) {
        Ok(registry) => registry,
        Err(_) => Json::Object(BTreeMap::new()),
    };
    let fixtures = if fixture_paths.is_empty() {
        default_fixture_inputs(root)
    } else {
        fixture_paths
            .iter()
            .map(|path| {
                let absolute = if path.is_absolute() {
                    path.clone()
                } else {
                    root.join(path)
                };
                let display = display_path(&absolute, root);
                let json = load_json(&absolute).unwrap_or_else(|error| panic!("{error}"));
                (display, json)
            })
            .collect()
    };
    evaluate_sources(root, &registry, &fixtures)
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn string_array_json(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn fixture_json(result: &FixtureValidation) -> String {
    format!(
        "{{\"expected_verdict\":{},\"expected_violations\":{},\"failures\":{},\"fixture_id\":{},\"fixture_passed\":{},\"observed_violations\":{},\"path\":{}}}",
        json_string(&result.expected_verdict),
        string_array_json(&result.expected_violations),
        string_array_json(&result.failures),
        json_string(&result.fixture_id),
        result.fixture_passed,
        string_array_json(&result.observed_violations),
        json_string(&result.path),
    )
}

pub fn to_json(report: &Report) -> String {
    format!(
        concat!(
            "{{",
            "\"active_doc_scan_count\":{},",
            "\"adr_hygiene_fixture_contract_measured\":{},",
            "\"adr_hygiene_registry_published\":{},",
            "\"authority_boundary\":{},",
            "\"decision_record_count\":{},",
            "\"expected_green_fixture_count\":{},",
            "\"expected_red_fixture_count\":{},",
            "\"failures\":{},",
            "\"fixture_count\":{},",
            "\"fixture_results\":[{}],",
            "\"full_adr_index_regenerated\":false,",
            "\"hyperscaler_grade\":false,",
            "\"live_required_context_execution_proven\":false,",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"production_ready\":false,",
            "\"protected_branch_authority_proven\":false,",
            "\"status_mutation_performed\":false,",
            "\"superseded_reference_pattern_count\":{},",
            "\"verdict\":{}",
            "}}"
        ),
        report.registry_summary.active_doc_scan_count,
        report.adr_hygiene_fixture_contract_measured,
        report.adr_hygiene_registry_published,
        json_string(&report.authority_boundary),
        report.registry_summary.decision_record_count,
        report.expected_green_fixture_count,
        report.expected_red_fixture_count,
        string_array_json(&report.failures),
        report.fixture_count,
        report
            .fixture_results
            .iter()
            .map(fixture_json)
            .collect::<Vec<_>>()
            .join(","),
        report.registry_summary.superseded_reference_pattern_count,
        json_string(&report.verdict),
    )
}

fn print_usage(program: &str) {
    eprintln!(
        "usage: {program} [--repo-root <path>] [--registry <path>] [--fixture <path>]... [--json]"
    );
}

pub fn run_cli(args: &[String]) -> i32 {
    let mut root = PathBuf::from(".");
    let mut registry = PathBuf::from(DEFAULT_REGISTRY);
    let mut fixtures = Vec::<PathBuf>::new();
    let mut json = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--repo-root" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    print_usage(&args[0]);
                    return 2;
                };
                root = PathBuf::from(value);
            }
            "--registry" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    print_usage(&args[0]);
                    return 2;
                };
                registry = PathBuf::from(value);
            }
            "--fixture" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    print_usage(&args[0]);
                    return 2;
                };
                fixtures.push(PathBuf::from(value));
            }
            "--json" => json = true,
            "--help" | "-h" => {
                print_usage(&args[0]);
                return 0;
            }
            other => {
                eprintln!("unknown argument: {other}");
                print_usage(&args[0]);
                return 2;
            }
        }
        index += 1;
    }
    if !registry.is_absolute() {
        registry = root.join(registry);
    }
    let report = evaluate(&root, &registry, &fixtures);
    let rendered = to_json(&report);
    if json || report.failures.is_empty() {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if report.failures.is_empty() { 0 } else { 1 }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    std::process::exit(run_cli(&args));
}
