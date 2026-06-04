//! Validate AC-0.17 claim-ceiling evidence without live readiness claims.
//!
//! This checker is local/static fixture evidence only. It evaluates the
//! checked-in Phase-0 claim/evidence map, the hyperscaler production-readiness
//! claim contract, and the declared BAD/GREEN fixtures so regulated readiness
//! language cannot be used without a claim row, a permitted tier, and evidence
//! appropriate to that tier. It never posts statuses, mutates branch protection,
//! or claims P0.0 green, Phase-0 completion, production readiness, or
//! hyperscaler-grade readiness.

#[allow(dead_code)]
#[path = "assert-result-bundle-output.rs"]
mod json_support;

pub use json_support::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;

const DEFAULT_CLAIM_MAP: &str = "specs/phase0-claim-evidence-map.json";
const DEFAULT_CONTRACT: &str = "specs/hyperscaler-production-readiness-claim-contract.json";
const AUTHORITY_BOUNDARY: &str = "claim-ceiling local/static fixture evidence only; this checker never posts statuses, mutates branch protection, or claims live readiness authority";

pub const REQUIRED_ROW_FIELDS: [&str; 9] = [
    "id",
    "artifact",
    "claim_text",
    "claim_tier",
    "allowed_language_now",
    "regulated_terms",
    "current_evidence",
    "missing_for_next_tier",
    "owner",
];

const STRONG_CLAIM_TIERS: [&str; 2] = ["production_ready", "hyperscaler_grade"];
const BUDGET_SIGNALS: [&str; 8] = [
    "p50",
    "p95",
    "p99",
    "throughput",
    "concurrency target",
    "performance_budget",
    "performance budget",
    "PERF-CAPACITY",
];
const MEASURED_RESULT_SIGNALS: [&str; 6] = [
    "measured_result",
    "measured result",
    "load result",
    "soak result",
    "load/soak result",
    "capacity breakpoint",
];
const FORBIDDEN_MECHANICAL_EVIDENCE_SIGNALS: [&str; 9] = [
    "local-only command",
    "legacy oya cli invocation",
    "oya verify",
    "oya gate",
    "local oya",
    "advisory check",
    "unrequired status",
    "stale sha",
    "legacy oya CLI invocation",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimMapSummary {
    pub row_count: usize,
    pub regulated_vocabulary_count: usize,
    pub allowed_tiers: Vec<String>,
    pub violations: Vec<String>,
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
pub struct Report {
    pub authority_boundary: String,
    pub claim_map: String,
    pub contract: String,
    pub claim_map_summary: ClaimMapSummary,
    pub fixture_results: Vec<FixtureValidation>,
    pub local_fixture_contract_proven: bool,
    pub claim_evidence_map_local_static_proven: bool,
    pub claim_ceiling_live: bool,
    pub protected_branch_authority_proven: bool,
    pub status_mutation_performed: bool,
    pub live_required_context_execution_proven: bool,
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

pub fn load_json(path: &str) -> Result<Json, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {path} failed: {error}"))?;
    parse_json(&text).map_err(|error| format!("parse {path} failed: {error}"))
}

fn object_field<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> Option<&'a Json> {
    object.get(key)
}

fn string_field(object: &BTreeMap<String, Json>, key: &str) -> Option<String> {
    object_field(object, key)
        .and_then(Json::as_str)
        .map(str::to_string)
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

fn non_empty(value: Option<&Json>) -> bool {
    match value {
        Some(Json::String(value)) => !value.trim().is_empty(),
        Some(Json::Array(value)) => !value.is_empty(),
        Some(Json::Object(value)) => !value.is_empty(),
        Some(Json::Null) | None => false,
        Some(Json::Bool(_)) | Some(Json::Number(_)) => true,
    }
}

fn row_text(row: &BTreeMap<String, Json>) -> String {
    let mut parts = Vec::new();
    for value in row.values() {
        match value {
            Json::String(value) => parts.push(value.clone()),
            Json::Array(items) => parts.extend(
                items
                    .iter()
                    .filter_map(Json::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
            ),
            _ => {}
        }
    }
    parts.join("\n")
}

fn allowed_tiers(contract: &Json) -> BTreeSet<String> {
    let Some(object) = contract.as_object() else {
        return BTreeSet::new();
    };
    object_list(object_field(object, "claim_tiers"))
        .into_iter()
        .filter_map(|tier| string_field(tier, "tier"))
        .collect()
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn has_signal(text: &str, signals: &[&str]) -> bool {
    let text_lower = text.to_lowercase();
    signals
        .iter()
        .any(|signal| text_lower.contains(&signal.to_lowercase()))
}

fn contains_forbidden_mechanical_evidence(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    FORBIDDEN_MECHANICAL_EVIDENCE_SIGNALS
        .iter()
        .any(|signal| text_lower.contains(&signal.to_lowercase()))
}

fn is_boundary(ch: Option<char>) -> bool {
    !ch.is_some_and(|value| value.is_ascii_alphanumeric() || value == '_')
}

fn term_matches(text: &str, term: &str) -> bool {
    let text = text.to_lowercase();
    let chars = text.chars().collect::<Vec<_>>();
    let pieces = term
        .trim()
        .to_lowercase()
        .split(|ch: char| ch == '-' || ch.is_ascii_whitespace())
        .filter(|piece| !piece.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if pieces.is_empty() {
        return false;
    }
    for start in 0..chars.len() {
        if !is_boundary(start.checked_sub(1).map(|index| chars[index])) {
            continue;
        }
        let mut cursor = start;
        let mut matched = true;
        for (piece_index, piece) in pieces.iter().enumerate() {
            for expected in piece.chars() {
                if chars.get(cursor) != Some(&expected) {
                    matched = false;
                    break;
                }
                cursor += 1;
            }
            if !matched {
                break;
            }
            if piece_index + 1 < pieces.len() {
                let separator_start = cursor;
                while chars
                    .get(cursor)
                    .is_some_and(|ch| *ch == '-' || ch.is_ascii_whitespace())
                {
                    cursor += 1;
                }
                if cursor == separator_start {
                    matched = false;
                    break;
                }
            }
        }
        if matched && is_boundary(chars.get(cursor).copied()) {
            return true;
        }
    }
    false
}

fn detected_terms(text: &str, vocabulary: &BTreeSet<String>) -> BTreeSet<String> {
    vocabulary
        .iter()
        .filter(|term| term_matches(text, term))
        .cloned()
        .collect()
}

pub fn validate_rows(
    rows: &[&BTreeMap<String, Json>],
    text: &str,
    vocabulary: &BTreeSet<String>,
    tiers: &BTreeSet<String>,
) -> Vec<String> {
    let mut violations = Vec::new();
    let mut row_ids = BTreeSet::new();
    let mut duplicate_ids = BTreeSet::new();
    let mut covered_terms = BTreeSet::new();

    for row in rows {
        if let Some(row_id) = string_field(row, "id").filter(|value| !value.is_empty()) {
            if row_ids.contains(&row_id) {
                duplicate_ids.insert(row_id.clone());
            }
            row_ids.insert(row_id);
        }
        if REQUIRED_ROW_FIELDS
            .iter()
            .any(|field| !non_empty(object_field(row, field)))
        {
            violations.push("missing_or_empty_required_field".to_string());
        }

        let tier = string_field(row, "claim_tier").unwrap_or_default();
        if !tiers.contains(&tier) {
            violations.push("unknown_claim_tier".to_string());
        }

        let row_terms = string_list(object_field(row, "regulated_terms"))
            .into_iter()
            .collect::<BTreeSet<_>>();
        covered_terms.extend(row_terms.iter().cloned());
        if !row_terms
            .difference(vocabulary)
            .collect::<Vec<_>>()
            .is_empty()
        {
            violations.push("unknown_regulated_term".to_string());
        }

        let combined = row_text(row);
        if tier == "mechanically_enforced"
            || combined.to_lowercase().contains("mechanically enforced")
        {
            if contains_forbidden_mechanical_evidence(&combined) {
                violations.push("forbidden_local_or_oya_evidence_for_mechanical_claim".to_string());
            }
        }

        if STRONG_CLAIM_TIERS.iter().any(|strong| *strong == tier) {
            // Strong readiness tiers need separately attributable evidence buckets.
            // A single prose entry such as "p95 budget and load result" must not
            // satisfy both requirements.
            let evidence_entries = string_list(object_field(row, "current_evidence"));
            let budget_only_entries = evidence_entries
                .iter()
                .filter(|entry| {
                    has_signal(entry, &BUDGET_SIGNALS)
                        && !has_signal(entry, &MEASURED_RESULT_SIGNALS)
                })
                .collect::<Vec<_>>();
            let measured_result_only_entries = evidence_entries
                .iter()
                .filter(|entry| {
                    has_signal(entry, &MEASURED_RESULT_SIGNALS)
                        && !has_signal(entry, &BUDGET_SIGNALS)
                })
                .collect::<Vec<_>>();
            if budget_only_entries.is_empty() || measured_result_only_entries.is_empty() {
                violations.push("performance_claim_without_budget_or_measured_result".to_string());
            }
        }

        if matches!(tier.as_str(), "target_non_claim" | "spec_ready")
            && string_list(object_field(row, "missing_for_next_tier")).is_empty()
        {
            violations.push("missing_next_tier_gap_for_non_live_claim".to_string());
        }
    }

    if !duplicate_ids.is_empty() {
        violations.push("duplicate_claim_row_id".to_string());
    }

    let terms_in_text = detected_terms(text, vocabulary);
    if !terms_in_text
        .difference(&covered_terms)
        .collect::<Vec<_>>()
        .is_empty()
    {
        violations.push("regulated_vocabulary_without_claim_row".to_string());
    }

    sorted_unique(violations)
}

fn fixture_paths(claim_map: &Json, explicit: &[String]) -> Vec<String> {
    if !explicit.is_empty() {
        return explicit.to_vec();
    }
    let Some(object) = claim_map.as_object() else {
        return Vec::new();
    };
    let fixture_set = object_field(object, "fixture_set").and_then(Json::as_object);
    fixture_set
        .map(|fixture_set| string_list(object_field(fixture_set, "all_fixture_paths")))
        .unwrap_or_default()
}

fn expected_from_fixture(fixture: &BTreeMap<String, Json>) -> (String, BTreeSet<String>, String) {
    let expected_verdict = match string_field(fixture, "expected_verdict").as_deref() {
        Some("GREEN") => "GREEN".to_string(),
        Some("RED") => "RED".to_string(),
        _ => "RED".to_string(),
    };
    let expected_violations = string_list(object_field(fixture, "expected_violations"))
        .into_iter()
        .collect::<BTreeSet<_>>();
    let fixture_id =
        string_field(fixture, "fixture_id").unwrap_or_else(|| "unknown-fixture".to_string());
    (expected_verdict, expected_violations, fixture_id)
}

fn validate_fixture_source(
    path: &str,
    fixture: &Json,
    vocabulary: &BTreeSet<String>,
    tiers: &BTreeSet<String>,
) -> FixtureValidation {
    let Some(fixture_object) = fixture.as_object() else {
        return FixtureValidation {
            path: path.to_string(),
            fixture_id: "unknown-fixture".to_string(),
            expected_verdict: "RED".to_string(),
            expected_violations: Vec::new(),
            observed_violations: vec!["fixture_must_be_json_object".to_string()],
            fixture_passed: false,
            failures: vec![format!("{path}: fixture must be a JSON object")],
        };
    };
    let (expected_verdict, expected_violations, fixture_id) = expected_from_fixture(fixture_object);
    let text = string_field(fixture_object, "text").unwrap_or_default();
    let rows = object_list(object_field(fixture_object, "claim_rows"));
    let observed = validate_rows(&rows, &text, vocabulary, tiers)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut fixture_failures = Vec::new();
    if expected_verdict == "GREEN" {
        if !observed.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: GREEN claim-ceiling fixture produced violations {:?}",
                observed.iter().collect::<Vec<_>>()
            ));
        }
        if !expected_violations.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: GREEN fixture must not list expected_violations"
            ));
        }
    } else {
        if observed.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: RED claim-ceiling fixture must produce violations"
            ));
        }
        let missing_expected = expected_violations
            .difference(&observed)
            .cloned()
            .collect::<Vec<_>>();
        if !missing_expected.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: expected violations were not observed {missing_expected:?}"
            ));
        }
    }
    FixtureValidation {
        path: path.to_string(),
        fixture_id,
        expected_verdict,
        expected_violations: expected_violations.into_iter().collect(),
        observed_violations: observed.into_iter().collect(),
        fixture_passed: fixture_failures.is_empty(),
        failures: fixture_failures,
    }
}

pub fn evaluate_sources(
    claim_map: &Json,
    contract: &Json,
    fixtures: &[(String, Json)],
    extra_text: &str,
    claim_map_path: &str,
    contract_path: &str,
) -> Report {
    let claim_map_object = claim_map.as_object();
    let vocabulary = claim_map_object
        .map(|object| string_list(object_field(object, "regulated_vocabulary")))
        .unwrap_or_default()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let tiers = allowed_tiers(contract);
    let rows = claim_map_object
        .map(|object| object_list(object_field(object, "seed_claim_rows")))
        .unwrap_or_default();

    let mut failures = Vec::new();
    let claim_map_violations = validate_rows(&rows, extra_text, &vocabulary, &tiers);
    failures.extend(
        claim_map_violations
            .iter()
            .map(|violation| format!("claim_map:{violation}")),
    );

    let fixture_results = fixtures
        .iter()
        .map(|(path, fixture)| validate_fixture_source(path, fixture, &vocabulary, &tiers))
        .collect::<Vec<_>>();
    for fixture in &fixture_results {
        failures.extend(fixture.failures.iter().cloned());
    }

    let proven = failures.is_empty();
    Report {
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
        claim_map: claim_map_path.to_string(),
        contract: contract_path.to_string(),
        claim_map_summary: ClaimMapSummary {
            row_count: rows.len(),
            regulated_vocabulary_count: vocabulary.len(),
            allowed_tiers: tiers.into_iter().collect(),
            violations: claim_map_violations,
        },
        fixture_results,
        local_fixture_contract_proven: proven,
        claim_evidence_map_local_static_proven: proven,
        claim_ceiling_live: false,
        protected_branch_authority_proven: false,
        status_mutation_performed: false,
        live_required_context_execution_proven: false,
        p0_0_green: false,
        phase0_complete: false,
        production_ready: false,
        hyperscaler_grade: false,
        verdict: if proven { "PASS" } else { "FAIL" }.to_string(),
        failures,
    }
}

pub fn evaluate_paths(
    claim_map_path: &str,
    contract_path: &str,
    explicit_fixture_paths: &[String],
    extra_text: &str,
) -> Result<Report, String> {
    let claim_map = load_json(claim_map_path)?;
    let contract = load_json(contract_path)?;
    let paths = fixture_paths(&claim_map, explicit_fixture_paths);
    let fixtures = paths
        .iter()
        .map(|path| load_json(path).map(|fixture| (path.clone(), fixture)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(evaluate_sources(
        &claim_map,
        &contract,
        &fixtures,
        extra_text,
        claim_map_path,
        contract_path,
    ))
}

pub fn to_json(report: &Report) -> String {
    format!(
        concat!(
            "{{",
            "\"authority_boundary\":{},",
            "\"claim_ceiling_live\":false,",
            "\"claim_evidence_map_local_static_proven\":{},",
            "\"claim_map\":{},",
            "\"claim_map_summary\":{},",
            "\"contract\":{},",
            "\"failures\":{},",
            "\"fixture_results\":{},",
            "\"hyperscaler_grade\":false,",
            "\"live_required_context_execution_proven\":false,",
            "\"local_fixture_contract_proven\":{},",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"production_ready\":false,",
            "\"protected_branch_authority_proven\":false,",
            "\"status_mutation_performed\":false,",
            "\"verdict\":{}",
            "}}"
        ),
        json_string(&report.authority_boundary),
        bool_json(report.claim_evidence_map_local_static_proven),
        json_string(&report.claim_map),
        claim_map_summary_json(&report.claim_map_summary),
        json_string(&report.contract),
        string_array_json(&report.failures),
        fixture_results_json(&report.fixture_results),
        bool_json(report.local_fixture_contract_proven),
        json_string(&report.verdict),
    )
}

fn claim_map_summary_json(summary: &ClaimMapSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"allowed_tiers\":{},",
            "\"regulated_vocabulary_count\":{},",
            "\"row_count\":{},",
            "\"violations\":{}",
            "}}"
        ),
        string_array_json(&summary.allowed_tiers),
        summary.regulated_vocabulary_count,
        summary.row_count,
        string_array_json(&summary.violations),
    )
}

fn fixture_results_json(results: &[FixtureValidation]) -> String {
    format!(
        "[{}]",
        results
            .iter()
            .map(fixture_validation_json)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn fixture_validation_json(result: &FixtureValidation) -> String {
    format!(
        concat!(
            "{{",
            "\"expected_violations\":{},",
            "\"expected_verdict\":{},",
            "\"failures\":{},",
            "\"fixture_id\":{},",
            "\"fixture_passed\":{},",
            "\"observed_violations\":{},",
            "\"path\":{}",
            "}}"
        ),
        string_array_json(&result.expected_violations),
        json_string(&result.expected_verdict),
        string_array_json(&result.failures),
        json_string(&result.fixture_id),
        bool_json(result.fixture_passed),
        string_array_json(&result.observed_violations),
        json_string(&result.path),
    )
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

fn bool_json(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('\"');
    out
}

fn main() {
    let mut claim_map = DEFAULT_CLAIM_MAP.to_string();
    let mut contract = DEFAULT_CONTRACT.to_string();
    let mut fixtures = Vec::new();
    let mut extra_text = String::new();
    let mut emit_json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--claim-map" => claim_map = args.next().expect("--claim-map requires a value"),
            "--contract" => contract = args.next().expect("--contract requires a value"),
            "--fixture" => fixtures.push(args.next().expect("--fixture requires a value")),
            "--text" => extra_text = args.next().expect("--text requires a value"),
            "--json" => emit_json = true,
            other => panic!("unknown argument {other}"),
        }
    }
    let report = evaluate_paths(&claim_map, &contract, &fixtures, &extra_text)
        .unwrap_or_else(|error| panic!("{error}"));
    let rendered = to_json(&report);
    if emit_json || report.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if report.verdict != "PASS" {
        std::process::exit(1);
    }
}
