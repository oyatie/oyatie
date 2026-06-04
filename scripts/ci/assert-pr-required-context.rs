//! Fail closed when a PR status rollup does not prove github-lane-unlocker-required success.
//!
//! Input is the JSON shape returned by `gh pr view --json headRefOid,statusCheckRollup`
//! or an equivalent fixture. This check is non-mutating: it never posts commit
//! statuses and it must not be used to turn local evidence into protected-branch
//! or Phase-0 exit authority.

#[allow(dead_code)]
#[path = "assert-result-bundle-output.rs"]
mod json_support;

pub use json_support::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

const DEFAULT_REQUIRED_CONTEXT: &str = "github-lane-unlocker-required";
const DEFAULT_TRUSTED_PRODUCER: &str = "github-lane-unlocker-ci-cd";
const SUCCESS_VALUES: [&str; 4] = ["success", "successful", "passed", "pass"];
const LEGACY_CONTEXTS: [&str; 8] = [
    "cargo-fmt",
    "cargo-check",
    "cargo-clippy",
    "cargo-nextest",
    "cargo-deny",
    "oya-verify",
    "oya-gate",
    "buck2-affected-only",
];
const PRODUCER_KEYS: [&str; 12] = [
    "workflow",
    "workflowName",
    "workflow_name",
    "producer",
    "producerName",
    "producer_name",
    "provider",
    "service",
    "source",
    "app",
    "appName",
    "app_name",
];
const PRODUCER_NESTED_KEYS: [&str; 5] = [
    "app",
    "checkRun",
    "checkSuite",
    "statusContext",
    "workflowRun",
];
const PRODUCER_OBJECT_KEYS: [&str; 3] = ["name", "slug", "login"];
const AUTHORITY_BOUNDARY: &str = "status-rollup evidence only; this checker never posts statuses";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub required_context: String,
    pub head_ref_oid: Option<String>,
    pub contexts: Vec<String>,
    pub legacy_contexts_present: Vec<String>,
    pub trusted_producer: String,
    pub p0_0_green: bool,
    pub phase0_complete: bool,
    pub authority_boundary: String,
    pub verdict: String,
    pub reason: String,
    pub required_context_status: String,
    pub required_context_producer_values: Vec<String>,
    pub required_context_proven: bool,
    pub required_context_trusted_producer: Option<bool>,
}

pub fn parse_json(text: &str) -> Result<Json, String> {
    json_support::parse_json(text)
}

fn object_field<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> Option<&'a Json> {
    object.get(key)
}

fn string_field(object: &BTreeMap<String, Json>, key: &str) -> Option<String> {
    object_field(object, key)
        .and_then(Json::as_str)
        .map(str::to_string)
}

fn load_input(path: &str) -> Result<Json, String> {
    let text = if path == "-" {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .map_err(|error| format!("read stdin failed: {error}"))?;
        text
    } else {
        fs::read_to_string(Path::new(path))
            .map_err(|error| format!("read {path} failed: {error}"))?
    };
    parse_json(&text).map_err(|error| format!("parse {path} failed: {error}"))
}

fn as_rollup_items(data: &BTreeMap<String, Json>) -> Vec<&BTreeMap<String, Json>> {
    match object_field(data, "statusCheckRollup") {
        Some(Json::Array(items)) => items.iter().filter_map(Json::as_object).collect(),
        Some(Json::Object(object)) => {
            if let Some(Json::Array(nodes)) = object_field(object, "nodes") {
                nodes.iter().filter_map(Json::as_object).collect()
            } else {
                vec![object]
            }
        }
        _ => Vec::new(),
    }
}

fn context_name(item: &BTreeMap<String, Json>) -> String {
    for key in ["name", "context", "checkName"] {
        if let Some(value) = string_field(item, key).filter(|value| !value.is_empty()) {
            return value;
        }
    }
    for key in ["statusContext", "checkRun"] {
        if let Some(nested) = object_field(item, key).and_then(Json::as_object) {
            let nested_name = context_name(nested);
            if !nested_name.is_empty() {
                return nested_name;
            }
        }
    }
    String::new()
}

fn state_value(item: &BTreeMap<String, Json>) -> String {
    // Prefer a terminal conclusion over a generic status/state. GitHub check
    // runs commonly expose status=COMPLETED plus conclusion=SUCCESS/FAILURE;
    // "completed" alone is not a pass signal.
    for key in ["conclusion", "state", "bucket", "status"] {
        if let Some(value) = string_field(item, key).filter(|value| !value.is_empty()) {
            return value.to_lowercase().replace('-', "_");
        }
    }
    for key in ["statusContext", "checkRun"] {
        if let Some(nested) = object_field(item, key).and_then(Json::as_object) {
            let nested_state = state_value(nested);
            if !nested_state.is_empty() {
                return nested_state;
            }
        }
    }
    String::new()
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.is_empty() && !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn producer_values(item: &BTreeMap<String, Json>) -> Vec<String> {
    let mut values = Vec::new();
    for key in PRODUCER_KEYS {
        match object_field(item, key) {
            Some(Json::String(value)) => push_unique(&mut values, value.clone()),
            Some(Json::Object(object)) => {
                for nested_key in PRODUCER_OBJECT_KEYS {
                    if let Some(value) =
                        string_field(object, nested_key).filter(|value| !value.is_empty())
                    {
                        push_unique(&mut values, value);
                    }
                }
            }
            _ => {}
        }
    }
    for key in PRODUCER_NESTED_KEYS {
        if let Some(nested) = object_field(item, key).and_then(Json::as_object) {
            for value in producer_values(nested) {
                push_unique(&mut values, value);
            }
        }
    }
    values
}

fn is_success(item: &BTreeMap<String, Json>) -> bool {
    let value = state_value(item);
    SUCCESS_VALUES.contains(&value.as_str())
}

fn is_trusted_producer(item: &BTreeMap<String, Json>, trusted_producer: &str) -> bool {
    let values = producer_values(item)
        .into_iter()
        .map(|value| value.to_lowercase())
        .collect::<BTreeSet<_>>();
    let expected = trusted_producer.to_lowercase();
    if values.contains(&expected) {
        return true;
    }
    if let Some((left, right)) = expected.split_once('/') {
        return values.contains(left) && values.contains(right);
    }
    false
}

pub fn summarize(data: &Json, required_context: &str, trusted_producer: &str) -> Report {
    let data = data.as_object().unwrap_or_else(|| {
        panic!("input must be a JSON object");
    });
    let items = as_rollup_items(data);
    let contexts = items
        .iter()
        .map(|item| context_name(item))
        .collect::<Vec<_>>();
    let legacy_contexts_present = contexts
        .iter()
        .filter(|name| LEGACY_CONTEXTS.contains(&name.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let matches = items
        .iter()
        .copied()
        .filter(|item| context_name(item) == required_context)
        .collect::<Vec<_>>();
    let head_ref_oid = string_field(data, "headRefOid");

    let base = |verdict: &str, reason: &str, required_context_status: &str| Report {
        required_context: required_context.to_string(),
        head_ref_oid: head_ref_oid.clone(),
        contexts: contexts.clone(),
        legacy_contexts_present: legacy_contexts_present.clone(),
        trusted_producer: trusted_producer.to_string(),
        p0_0_green: false,
        phase0_complete: false,
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
        verdict: verdict.to_string(),
        reason: reason.to_string(),
        required_context_status: required_context_status.to_string(),
        required_context_producer_values: Vec::new(),
        required_context_proven: false,
        required_context_trusted_producer: None,
    };

    if items.is_empty() {
        return base("FAIL", "no_status_checks_reported", "missing");
    }
    if matches.is_empty() {
        return base("FAIL", "missing_required_context", "missing");
    }

    let winning = matches[0];
    let state = match state_value(winning) {
        value if value.is_empty() => "unknown".to_string(),
        value => value,
    };
    let producer = producer_values(winning);
    if !is_success(winning) {
        let mut report = base("FAIL", "required_context_not_success", &state);
        report.required_context_producer_values = producer;
        return report;
    }
    if producer.is_empty() {
        let mut report = base("FAIL", "missing_required_context_producer", &state);
        report.required_context_trusted_producer = Some(false);
        return report;
    }
    if !is_trusted_producer(winning, trusted_producer) {
        let mut report = base("FAIL", "untrusted_required_context_producer", &state);
        report.required_context_producer_values = producer;
        report.required_context_trusted_producer = Some(false);
        return report;
    }

    Report {
        required_context: required_context.to_string(),
        head_ref_oid,
        contexts,
        legacy_contexts_present,
        trusted_producer: trusted_producer.to_string(),
        p0_0_green: false,
        phase0_complete: false,
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
        verdict: "PASS".to_string(),
        reason: "required_context_success".to_string(),
        required_context_status: state,
        required_context_producer_values: producer,
        required_context_proven: true,
        required_context_trusted_producer: Some(true),
    }
}

pub fn summarize_path(
    input: &str,
    required_context: &str,
    trusted_producer: &str,
) -> Result<Report, String> {
    let data = load_input(input)?;
    Ok(summarize(&data, required_context, trusted_producer))
}

pub fn to_json(report: &Report) -> String {
    let trusted = report
        .required_context_trusted_producer
        .map(bool_json)
        .unwrap_or("null");
    format!(
        concat!(
            "{{",
            "\"authority_boundary\":{},",
            "\"contexts\":{},",
            "\"headRefOid\":{},",
            "\"legacy_contexts_present\":{},",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"reason\":{},",
            "\"required_context\":{},",
            "\"required_context_producer_values\":{},",
            "\"required_context_proven\":{},",
            "\"required_context_status\":{},",
            "\"required_context_trusted_producer\":{},",
            "\"trusted_producer\":{},",
            "\"verdict\":{}",
            "}}"
        ),
        json_string(&report.authority_boundary),
        string_array_json(&report.contexts),
        optional_string_json(report.head_ref_oid.as_deref()),
        string_array_json(&report.legacy_contexts_present),
        json_string(&report.reason),
        json_string(&report.required_context),
        string_array_json(&report.required_context_producer_values),
        bool_json(report.required_context_proven),
        json_string(&report.required_context_status),
        trusted,
        json_string(&report.trusted_producer),
        json_string(&report.verdict),
    )
}

fn optional_string_json(value: Option<&str>) -> String {
    value.map(json_string).unwrap_or_else(|| "null".to_string())
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
    let mut input: Option<String> = None;
    let mut required_context = DEFAULT_REQUIRED_CONTEXT.to_string();
    let mut trusted_producer = DEFAULT_TRUSTED_PRODUCER.to_string();
    let mut emit_json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(args.next().expect("--input requires a value")),
            "--required-context" => {
                required_context = args.next().expect("--required-context requires a value")
            }
            "--trusted-producer" => {
                trusted_producer = args.next().expect("--trusted-producer requires a value")
            }
            "--json" => emit_json = true,
            other => panic!("unknown argument {other}"),
        }
    }
    let input = input.expect("--input is required");
    let report = summarize_path(&input, &required_context, &trusted_producer)
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
