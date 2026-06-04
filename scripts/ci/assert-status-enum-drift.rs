//! Validate AC-0.2 status enum and spec/manifest drift fixtures.
//!
//! This checker is local/static fixture evidence only. It validates the
//! checked-in 3-axis status enum registry and GOOD/BAD fixtures for invalid
//! status values, retired REAL live-field tokens, spec/code/manifest mismatches,
//! and status drift. It never posts statuses, mutates branch protection, proves
//! full manifest/PRD conformance, or claims P0.0/Phase-0 completion.

#[allow(dead_code)]
#[path = "assert-result-bundle-output.rs"]
mod json_support;

pub use json_support::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/status-enum-registry.json";
const DEFAULT_FIXTURE_DIR: &str = "specs/fixtures/phase0-status-enum-drift";
const AUTHORITY_BOUNDARY: &str = "AC-0.2 local/static status enum and drift fixture evidence only; no status mutation, live required-context authority, full manifest/PRD conformance, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven";
const AXES: [&str; 3] = ["decision", "maturity", "constraint"];
pub const REQUIRED_AXIS_FIELDS: [&str; 3] =
    ["decision_status", "maturity_status", "constraint_status"];
const FALSE_CLAIMS: [&str; 9] = [
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "full_manifest_prd_conformance_proven",
    "status_drift_live_gate_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];
const FIXTURE_FALSE_CLAIMS: [&str; 7] = [
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistrySummary {
    pub axis_count: usize,
    pub allowed_value_count: usize,
    pub seed_surface_count: usize,
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
    pub status_enum_registry_published: bool,
    pub status_drift_fixture_contract_measured: bool,
    pub registry_summary: RegistrySummary,
    pub fixture_count: usize,
    pub expected_green_fixture_count: usize,
    pub expected_red_fixture_count: usize,
    pub fixture_results: Vec<FixtureValidation>,
    pub status_mutation_performed: bool,
    pub protected_branch_authority_proven: bool,
    pub live_required_context_execution_proven: bool,
    pub full_manifest_prd_conformance_proven: bool,
    pub status_drift_live_gate_proven: bool,
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
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn validate_false_claims(
    mapping: Option<&BTreeMap<String, Json>>,
    failures: &mut Vec<String>,
    claims: &[&str],
    prefix: &str,
) {
    let empty = BTreeMap::new();
    let mapping = mapping.unwrap_or(&empty);
    for claim in claims {
        if bool_field(mapping, claim) != Some(false) {
            failures.push(format!("{prefix}forbidden_true_or_missing_claim_{claim}"));
        }
    }
}

fn contains_real_token(text: &str) -> bool {
    let chars = text.chars().collect::<Vec<_>>();
    for start in 0..chars.len() {
        if chars.get(start..start + 4) == Some(&['R', 'E', 'A', 'L']) {
            let before = start
                .checked_sub(1)
                .and_then(|index| chars.get(index))
                .copied();
            let after = chars.get(start + 4).copied();
            let boundary = |ch: Option<char>| {
                !ch.is_some_and(|value| value.is_ascii_alphanumeric() || value == '_')
            };
            if boundary(before) && boundary(after) {
                return true;
            }
        }
    }
    false
}

fn validate_axis_fields(
    fields: &BTreeMap<String, Json>,
    allowed_by_field: &BTreeMap<String, BTreeSet<String>>,
    failures: &mut Vec<String>,
    prefix: &str,
) {
    for field in REQUIRED_AXIS_FIELDS {
        let value = string_field(fields, field).unwrap_or_default();
        if value.is_empty() {
            failures.push(format!("{prefix}missing_status_axis_field:{field}"));
            continue;
        }
        if !allowed_by_field
            .get(field)
            .is_some_and(|allowed| allowed.contains(&value))
        {
            failures.push(format!("{prefix}invalid_status_enum_value:{field}:{value}"));
        }
        if contains_real_token(&value) {
            failures.push(format!("{prefix}retired_real_token_live_field:{field}"));
        }
    }
}

fn validate_registry(
    root: &Path,
    registry: &Json,
) -> (
    Vec<String>,
    RegistrySummary,
    BTreeMap<String, BTreeSet<String>>,
) {
    let mut failures = Vec::new();
    let Some(registry_object) = registry.as_object() else {
        failures.push("status_enum_registry_must_be_object".to_string());
        return (
            failures,
            RegistrySummary {
                axis_count: 0,
                allowed_value_count: 0,
                seed_surface_count: 0,
            },
            REQUIRED_AXIS_FIELDS
                .iter()
                .map(|field| ((*field).to_string(), BTreeSet::new()))
                .collect(),
        );
    };
    let boundary = object_field(registry_object, "claim_boundary").and_then(Json::as_object);
    if boundary.and_then(|boundary| bool_field(boundary, "status_enum_registry_published"))
        != Some(true)
    {
        failures.push("status_enum_registry_not_published".to_string());
    }
    if boundary.and_then(|boundary| bool_field(boundary, "status_drift_fixture_contract_measured"))
        != Some(true)
    {
        failures.push("status_drift_fixture_contract_not_measured".to_string());
    }
    validate_false_claims(boundary, &mut failures, &FALSE_CLAIMS, "");

    let axes_object = object_field(registry_object, "axes")
        .and_then(Json::as_object)
        .cloned()
        .unwrap_or_default();
    let mut allowed_by_field = BTreeMap::new();
    for axis in AXES {
        let axis_spec = object_field(&axes_object, axis).and_then(Json::as_object);
        let expected_field = format!("{axis}_status");
        if axis_spec
            .and_then(|spec| string_field(spec, "field"))
            .as_deref()
            != Some(expected_field.as_str())
        {
            failures.push(format!("axis_field_mismatch:{axis}"));
        }
        let allowed = axis_spec
            .map(|spec| string_list(object_field(spec, "allowed_values")))
            .unwrap_or_default()
            .into_iter()
            .collect::<BTreeSet<_>>();
        if allowed.is_empty() {
            failures.push(format!("axis_allowed_values_missing:{axis}"));
        }
        if allowed.contains("REAL") {
            failures.push(format!("retired_real_token_allowed:{axis}"));
        }
        allowed_by_field.insert(expected_field, allowed);
    }

    let field_contract = object_field(registry_object, "field_contract").and_then(Json::as_object);
    let required_axis_fields = field_contract
        .map(|field_contract| string_list(object_field(field_contract, "required_axis_fields")))
        .unwrap_or_default();
    if required_axis_fields
        != REQUIRED_AXIS_FIELDS
            .iter()
            .map(|field| (*field).to_string())
            .collect::<Vec<_>>()
    {
        failures.push("required_axis_fields_drift".to_string());
    }
    if field_contract.and_then(|field_contract| {
        bool_field(field_contract, "full_manifest_prd_conformance_proven")
    }) != Some(false)
    {
        failures.push(
            "forbidden_true_or_missing_field_contract_full_manifest_prd_conformance_proven"
                .to_string(),
        );
    }
    if !field_contract
        .map(|field_contract| {
            string_list(object_field(field_contract, "retired_live_status_tokens"))
        })
        .unwrap_or_default()
        .contains(&"REAL".to_string())
    {
        failures.push("retired_real_token_not_registered".to_string());
    }

    let surfaces = object_list(object_field(registry_object, "seed_surface_registry"));
    for surface in &surfaces {
        let surface_id = string_field(surface, "surface_id")
            .unwrap_or_else(|| "<missing-surface-id>".to_string());
        validate_axis_fields(
            surface,
            &allowed_by_field,
            &mut failures,
            &format!("{surface_id}:"),
        );
        for path_field in ["spec_path", "code_path", "manifest_path"] {
            let path_value = string_field(surface, path_field).unwrap_or_default();
            if path_value.is_empty() {
                failures.push(format!("{surface_id}:missing_surface_path:{path_field}"));
                continue;
            }
            if !root.join(&path_value).exists() {
                failures.push(format!(
                    "{surface_id}:surface_path_missing:{path_field}:{path_value}"
                ));
            }
        }
    }

    (
        failures,
        RegistrySummary {
            axis_count: axes_object.len(),
            allowed_value_count: allowed_by_field.values().map(BTreeSet::len).sum(),
            seed_surface_count: surfaces.len(),
        },
        allowed_by_field,
    )
}

fn validate_pair(
    root: &Path,
    pair: &BTreeMap<String, Json>,
    allowed_by_field: &BTreeMap<String, BTreeSet<String>>,
    observed: &mut Vec<String>,
) {
    let surface_id =
        string_field(pair, "surface_id").unwrap_or_else(|| "<missing-surface-id>".to_string());
    for path_field in ["spec_path", "code_path", "manifest_path"] {
        let path_value = string_field(pair, path_field).unwrap_or_default();
        if path_value.is_empty() || !root.join(&path_value).exists() {
            observed.push("spec_code_manifest_mismatch".to_string());
            observed.push(format!(
                "spec_code_manifest_mismatch:{surface_id}:{path_field}"
            ));
        }
    }
    let empty = BTreeMap::new();
    let spec_fields = object_field(pair, "spec_status_fields")
        .and_then(Json::as_object)
        .unwrap_or(&empty);
    let manifest_fields = object_field(pair, "manifest_status_fields")
        .and_then(Json::as_object)
        .unwrap_or(&empty);
    validate_axis_fields(
        spec_fields,
        allowed_by_field,
        observed,
        &format!("{surface_id}:spec:"),
    );
    validate_axis_fields(
        manifest_fields,
        allowed_by_field,
        observed,
        &format!("{surface_id}:manifest:"),
    );
    for field in REQUIRED_AXIS_FIELDS {
        if object_field(spec_fields, field) != object_field(manifest_fields, field) {
            observed.push("status_drift_mismatch".to_string());
            observed.push(format!("status_drift_mismatch:{surface_id}:{field}"));
        }
    }
}

fn validate_fixture(
    root: &Path,
    path: &str,
    fixture: &Json,
    allowed_by_field: &BTreeMap<String, BTreeSet<String>>,
) -> FixtureValidation {
    let Some(fixture_object) = fixture.as_object() else {
        return FixtureValidation {
            path: path.to_string(),
            fixture_id: "<missing-fixture-id>".to_string(),
            expected_verdict: "RED".to_string(),
            expected_violations: Vec::new(),
            observed_violations: vec!["fixture_must_be_json_object".to_string()],
            fixture_passed: false,
            failures: vec![format!("{path}: fixture must be a JSON object")],
        };
    };
    let fixture_id = string_field(fixture_object, "fixture_id")
        .unwrap_or_else(|| "<missing-fixture-id>".to_string());
    let expected_verdict = match string_field(fixture_object, "expected_verdict").as_deref() {
        Some("GREEN") => "GREEN".to_string(),
        Some("RED") => "RED".to_string(),
        _ => "RED".to_string(),
    };
    let expected_violations = string_list(object_field(fixture_object, "expected_violations"))
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut observed = Vec::new();
    let boundary = object_field(fixture_object, "claim_boundary").and_then(Json::as_object);
    validate_false_claims(boundary, &mut observed, &FIXTURE_FALSE_CLAIMS, "");
    let empty = BTreeMap::new();
    let status_fields = object_field(fixture_object, "status_fields")
        .and_then(Json::as_object)
        .unwrap_or(&empty);
    validate_axis_fields(status_fields, allowed_by_field, &mut observed, "");
    for pair in object_list(object_field(fixture_object, "spec_manifest_pairs")) {
        validate_pair(root, pair, allowed_by_field, &mut observed);
    }

    let mut observed_set = observed.into_iter().collect::<BTreeSet<_>>();
    for item in observed_set.clone() {
        if item.starts_with("invalid_status_enum_value:") {
            observed_set.insert("invalid_status_enum_value".to_string());
        }
        if item.starts_with("retired_real_token_live_field:") {
            observed_set.insert("retired_real_token_live_field".to_string());
        }
        if item.starts_with("spec_code_manifest_mismatch:") {
            observed_set.insert("spec_code_manifest_mismatch".to_string());
        }
        if item.starts_with("status_drift_mismatch:") {
            observed_set.insert("status_drift_mismatch".to_string());
        }
    }

    let mut fixture_failures = Vec::new();
    if expected_verdict == "GREEN" {
        if !observed_set.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: GREEN status-enum fixture produced violations {:?}",
                observed_set.iter().collect::<Vec<_>>()
            ));
        }
        if !expected_violations.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: GREEN fixture must not list expected_violations"
            ));
        }
    } else {
        if observed_set.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: RED status-enum fixture must produce violations"
            ));
        }
        let missing_expected = expected_violations
            .difference(&observed_set)
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
        observed_violations: observed_set.into_iter().collect(),
        fixture_passed: fixture_failures.is_empty(),
        failures: fixture_failures,
    }
}

fn default_fixture_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = root.join(DEFAULT_FIXTURE_DIR);
    let mut paths = fs::read_dir(&dir)
        .map_err(|error| format!("read {} failed: {error}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn fixture_paths(root: &Path, explicit: &[String]) -> Result<Vec<PathBuf>, String> {
    if explicit.is_empty() {
        return default_fixture_paths(root);
    }
    Ok(explicit
        .iter()
        .map(|item| {
            let path = PathBuf::from(item);
            if path.is_absolute() {
                path
            } else {
                root.join(path)
            }
        })
        .collect())
}

pub fn evaluate_sources(root: &Path, registry: &Json, fixtures: &[(String, Json)]) -> Report {
    let (registry_failures, registry_summary, allowed_by_field) = validate_registry(root, registry);
    let mut failures = registry_failures;
    let fixture_results = fixtures
        .iter()
        .map(|(path, fixture)| validate_fixture(root, path, fixture, &allowed_by_field))
        .collect::<Vec<_>>();
    for fixture in &fixture_results {
        failures.extend(fixture.failures.iter().cloned());
    }
    let expected_green_fixture_count = fixture_results
        .iter()
        .filter(|item| item.expected_verdict == "GREEN")
        .count();
    let expected_red_fixture_count = fixture_results
        .iter()
        .filter(|item| item.expected_verdict == "RED")
        .count();
    let status_enum_registry_published = registry
        .as_object()
        .and_then(|object| object_field(object, "claim_boundary"))
        .and_then(Json::as_object)
        .and_then(|boundary| bool_field(boundary, "status_enum_registry_published"))
        == Some(true);
    let failures = sorted_unique(failures);
    let measured = failures.is_empty();
    Report {
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
        status_enum_registry_published,
        status_drift_fixture_contract_measured: measured,
        registry_summary,
        fixture_count: fixture_results.len(),
        expected_green_fixture_count,
        expected_red_fixture_count,
        fixture_results,
        status_mutation_performed: false,
        protected_branch_authority_proven: false,
        live_required_context_execution_proven: false,
        full_manifest_prd_conformance_proven: false,
        status_drift_live_gate_proven: false,
        p0_0_green: false,
        phase0_complete: false,
        production_ready: false,
        hyperscaler_grade: false,
        verdict: if measured { "PASS" } else { "FAIL" }.to_string(),
        failures,
    }
}

pub fn evaluate_paths(
    root: &Path,
    registry_path: &Path,
    explicit_fixtures: &[String],
) -> Result<Report, String> {
    let registry = load_json(registry_path)?;
    let fixtures = fixture_paths(root, explicit_fixtures)?
        .into_iter()
        .map(|path| {
            if !path.is_file() {
                return Err(format!(
                    "fixture_path_missing:{}",
                    display_path(&path, root)
                ));
            }
            let display = display_path(&path, root);
            load_json(&path).map(|fixture| (display, fixture))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(evaluate_sources(root, &registry, &fixtures))
}

pub fn to_json(report: &Report) -> String {
    format!(
        concat!(
            "{{",
            "\"allowed_value_count\":{},",
            "\"authority_boundary\":{},",
            "\"axis_count\":{},",
            "\"expected_green_fixture_count\":{},",
            "\"expected_red_fixture_count\":{},",
            "\"failures\":{},",
            "\"fixture_count\":{},",
            "\"fixture_results\":{},",
            "\"full_manifest_prd_conformance_proven\":false,",
            "\"hyperscaler_grade\":false,",
            "\"live_required_context_execution_proven\":false,",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"production_ready\":false,",
            "\"protected_branch_authority_proven\":false,",
            "\"seed_surface_count\":{},",
            "\"status_drift_fixture_contract_measured\":{},",
            "\"status_drift_live_gate_proven\":false,",
            "\"status_enum_registry_published\":{},",
            "\"status_mutation_performed\":false,",
            "\"verdict\":{}",
            "}}"
        ),
        report.registry_summary.allowed_value_count,
        json_string(&report.authority_boundary),
        report.registry_summary.axis_count,
        report.expected_green_fixture_count,
        report.expected_red_fixture_count,
        string_array_json(&report.failures),
        report.fixture_count,
        fixture_results_json(&report.fixture_results),
        report.registry_summary.seed_surface_count,
        bool_json(report.status_drift_fixture_contract_measured),
        bool_json(report.status_enum_registry_published),
        json_string(&report.verdict),
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

fn absolute_under_root(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn main() {
    let mut repo_root = PathBuf::from(".");
    let mut registry = DEFAULT_REGISTRY.to_string();
    let mut fixtures = Vec::new();
    let mut emit_json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(args.next().expect("--repo-root requires a value"))
            }
            "--registry" => registry = args.next().expect("--registry requires a value"),
            "--fixture" => fixtures.push(args.next().expect("--fixture requires a value")),
            "--json" => emit_json = true,
            other => panic!("unknown argument {other}"),
        }
    }
    let root = repo_root
        .canonicalize()
        .unwrap_or_else(|error| panic!("canonicalize {} failed: {error}", repo_root.display()));
    let registry_path = absolute_under_root(&root, &registry);
    let report =
        evaluate_paths(&root, &registry_path, &fixtures).unwrap_or_else(|error| panic!("{error}"));
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
