//! Phase-0 RED/GREEN fixture coverage registry checker.
//!
//! AC-0.14 local/static RED/GREEN fixture registry evidence only. This verifies
//! that checked-in Phase-0 gate targets have explicit GOOD and BAD fixture/probe
//! markers, that the markers remain present, and that the contract keeps live
//! readiness claims false. It never runs live CI, posts statuses, mutates branch
//! protection, or proves Phase-0 completion.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_SPEC: &str = "specs/red-green-fixture-contract.json";
const FALSE_CLAIMS: &[&str] = &[
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];
// Policy anchor: forbidden_true_or_missing_claim_p0_0_green
const REQUIRED_ENTRY_FIELDS: &[&str] = &[
    "id",
    "buck2_target",
    "test_paths",
    "green_markers",
    "red_markers",
    "non_claim_markers",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryResult {
    pub id: String,
    pub buck2_target: Option<String>,
    pub test_path_count: usize,
    pub green_marker_count: usize,
    pub red_marker_count: usize,
    pub non_claim_marker_count: usize,
    pub automation_matrix_row_id: Option<String>,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
    pub entry_results: Vec<EntryResult>,
    pub entry_count: usize,
    pub buck2_target_count: usize,
    pub green_marker_count: usize,
    pub red_marker_count: usize,
    pub non_claim_marker_count: usize,
    pub contract_spec: String,
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

fn compact_json_text(input: &str) -> String {
    input.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&format!(
        "\"{}\":{}",
        key,
        if value { "true" } else { "false" }
    ))
}

fn has_key(text: &str, key: &str) -> bool {
    compact_json_text(text).contains(&format!("\"{}\":", key))
}

fn skip_ws(text: &str, mut index: usize) -> usize {
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

fn parse_json_string_at(text: &str, start: usize) -> Option<(String, usize)> {
    if text[start..].chars().next()? != '"' {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    let mut index = start + 1;
    while index < text.len() {
        let ch = text[index..].chars().next()?;
        index += ch.len_utf8();
        if escaped {
            let decoded = match ch {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'u' => {
                    if index + 4 <= text.len() {
                        let raw = &text[index..index + 4];
                        index += 4;
                        char::from_u32(u32::from_str_radix(raw, 16).ok()?).unwrap_or('?')
                    } else {
                        '?'
                    }
                }
                other => other,
            };
            out.push(decoded);
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some((out, index));
        } else {
            out.push(ch);
        }
    }
    None
}

fn key_value_start(text: &str, key: &str) -> Option<usize> {
    let key_token = format!("\"{key}\"");
    let key_start = text.find(&key_token)?;
    let colon = text[key_start + key_token.len()..].find(':')? + key_start + key_token.len();
    Some(skip_ws(text, colon + 1))
}

fn string_after_key(text: &str, key: &str) -> Option<String> {
    let start = key_value_start(text, key)?;
    parse_json_string_at(text, start).map(|(value, _)| value)
}

fn slice_balanced_at(text: &str, start: usize, open: char, close: char) -> Option<&str> {
    if text[start..].chars().next()? != open {
        return None;
    }
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    let mut index = start;
    while index < text.len() {
        let ch = text[index..].chars().next()?;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += ch.len_utf8();
            continue;
        }
        if ch == '"' {
            in_string = true;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                index += ch.len_utf8();
                return Some(&text[start..index]);
            }
        }
        index += ch.len_utf8();
    }
    None
}

fn array_after_key<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let start = key_value_start(text, key)?;
    slice_balanced_at(text, start, '[', ']')
}

fn object_after_key<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let start = key_value_start(text, key)?;
    slice_balanced_at(text, start, '{', '}')
}

fn array_strings_after_key(text: &str, key: &str) -> Vec<String> {
    let Some(array) = array_after_key(text, key) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    let mut index = 1usize;
    while index + 1 < array.len() {
        index = skip_ws(array, index);
        if array[index..].starts_with('"') {
            if let Some((value, end)) = parse_json_string_at(array, index) {
                values.push(value);
                index = end;
                continue;
            }
        }
        index += array[index..].chars().next().unwrap().len_utf8();
    }
    values
}

fn object_slices_in_array(array: &str) -> Vec<&str> {
    let mut objects = Vec::new();
    let mut index = 1usize;
    while index + 1 < array.len() {
        index = skip_ws(array, index);
        if array[index..].starts_with('{') {
            if let Some(object) = slice_balanced_at(array, index, '{', '}') {
                index += object.len();
                objects.push(object);
                continue;
            }
        }
        index += array[index..].chars().next().unwrap().len_utf8();
    }
    objects
}

fn buck_targets(buck_text: &str) -> BTreeSet<String> {
    let mut targets = BTreeSet::new();
    for line in buck_text.lines() {
        let trimmed = line.trim_start();
        let Some(after_name) = trimmed.strip_prefix("name") else {
            continue;
        };
        let after_ws = after_name.trim_start();
        let Some(after_equals) = after_ws.strip_prefix('=') else {
            continue;
        };
        let start = skip_ws(after_equals, 0);
        if let Some((value, _)) = parse_json_string_at(after_equals, start) {
            targets.insert(value);
        }
    }
    targets
}

fn read(root: &Path, rel_or_abs: &str, failures: &mut Vec<String>) -> String {
    let path = resolve(root, rel_or_abs);
    match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            failures.push(format!("{rel_or_abs}:read_failed:{error}"));
            String::new()
        }
    }
}

fn resolve(root: &Path, rel_or_abs: &str) -> PathBuf {
    let path = Path::new(rel_or_abs);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn display_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn decode_json_string_raw(raw: &str) -> String {
    let mut out = String::new();
    let mut escaped = false;
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if escaped {
            let decoded = match ch {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'u' => {
                    let raw_code = chars.by_ref().take(4).collect::<String>();
                    char::from_u32(u32::from_str_radix(&raw_code, 16).unwrap_or(b'?' as u32))
                        .unwrap_or('?')
                }
                other => other,
            };
            out.push(decoded);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    out
}

fn raw_string_literal_at(text: &str, start: usize) -> Option<(String, usize)> {
    if text[start..].chars().next()? != '"' {
        return None;
    }
    let mut escaped = false;
    let mut index = start + 1;
    let raw_start = index;
    while index < text.len() {
        let ch = text[index..].chars().next()?;
        if escaped {
            escaped = false;
            index += ch.len_utf8();
            continue;
        }
        if ch == '\\' {
            escaped = true;
            index += ch.len_utf8();
            continue;
        }
        if ch == '"' {
            return Some((decode_json_string_raw(&text[raw_start..index]), index + 1));
        }
        index += ch.len_utf8();
    }
    None
}

fn json_string_literals(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut index = 0usize;
    while index < text.len() {
        index = skip_ws(text, index);
        if index >= text.len() {
            break;
        }
        if text[index..].starts_with('"') {
            if let Some((value, end)) = raw_string_literal_at(text, index) {
                values.push(value);
                index = end;
                continue;
            }
        }
        index += text[index..].chars().next().unwrap().len_utf8();
    }
    values
}

fn marker_tokens(marker: &str) -> Vec<String> {
    let Some(start) = key_value_start(marker, "contains") else {
        return Vec::new();
    };
    if marker[start..].starts_with('"') {
        return raw_string_literal_at(marker, start)
            .map(|(value, _)| vec![value])
            .unwrap_or_default();
    }
    if marker[start..].starts_with('[') {
        return slice_balanced_at(marker, start, '[', ']')
            .map(json_string_literals)
            .unwrap_or_default();
    }
    Vec::new()
}

fn validate_marker(root: &Path, marker: &str, prefix: &str, failures: &mut Vec<String>) {
    let Some(marker_path) = string_after_key(marker, "path").filter(|path| !path.is_empty()) else {
        failures.push(format!("{prefix}:marker_missing_path"));
        return;
    };
    let path = root.join(&marker_path);
    if !path.is_file() {
        failures.push(format!("{prefix}:marker_path_missing:{marker_path}"));
        return;
    }
    let tokens = marker_tokens(marker);
    if tokens.is_empty() {
        failures.push(format!("{prefix}:marker_missing_contains:{marker_path}"));
        return;
    }
    let text = fs::read_to_string(&path).unwrap_or_default();
    for token in tokens {
        if !text.contains(&token) {
            failures.push(format!(
                "{prefix}:marker_text_missing:{marker_path}:{token}"
            ));
        }
    }
}

fn validate_spec(spec: &str) -> Vec<String> {
    let mut failures = Vec::new();
    let boundary = object_after_key(spec, "claim_boundary").unwrap_or("");
    if !has_bool(boundary, "red_green_fixture_contract_measured", true) {
        failures.push("red_green_fixture_contract_not_measured".to_owned());
    }
    for claim in FALSE_CLAIMS {
        if !has_bool(boundary, claim, false) {
            failures.push(format!("forbidden_true_or_missing_claim_{claim}"));
        }
    }
    let automated_chain = array_strings_after_key(spec, "automated_chain").join("\n");
    if !automated_chain.contains("//:phase0-red-green-fixture-contract-check") {
        failures.push("missing_buck2_target_in_automated_chain".to_owned());
    }
    if !automated_chain.contains("scripts/ci/assert-red-green-fixture-contract.rs") {
        failures.push("missing_checker_in_automated_chain".to_owned());
    }
    failures
}

fn validate_entry(
    root: &Path,
    entry: &str,
    targets: &BTreeSet<String>,
    matrix_text: &str,
    buck_text: &str,
) -> EntryResult {
    let mut failures = Vec::new();
    let entry_id = string_after_key(entry, "id").unwrap_or_else(|| "<missing-id>".to_owned());
    for field in REQUIRED_ENTRY_FIELDS {
        if !has_key(entry, field) {
            failures.push(format!("{entry_id}:missing_required_field:{field}"));
        }
    }
    let target = string_after_key(entry, "buck2_target");
    let target_name = if let Some(target) = target.as_ref() {
        if let Some(name) = target.strip_prefix("//:") {
            if !targets.contains(name) {
                failures.push(format!("{entry_id}:buck2_target_missing:{target}"));
            }
            name
        } else {
            failures.push(format!("{entry_id}:invalid_buck2_target"));
            ""
        }
    } else {
        failures.push(format!("{entry_id}:invalid_buck2_target"));
        ""
    };

    let test_paths = array_strings_after_key(entry, "test_paths");
    if test_paths.is_empty() {
        failures.push(format!("{entry_id}:missing_test_paths"));
    }
    for path_value in &test_paths {
        if !root.join(path_value).is_file() {
            failures.push(format!("{entry_id}:test_path_missing:{path_value}"));
        }
        if !buck_text.contains(path_value) {
            failures.push(format!(
                "{entry_id}:test_path_not_wired_in_buck:{path_value}"
            ));
        }
    }

    let row_id = string_after_key(entry, "automation_matrix_row_id");
    if let Some(row_id) = row_id.as_ref() {
        if !matrix_text.contains(&format!("\"id\": \"{row_id}\"")) {
            failures.push(format!("{entry_id}:automation_matrix_row_missing:{row_id}"));
        } else if let Some(target) = target.as_ref() {
            if !matrix_text.contains(target) {
                failures.push(format!(
                    "{entry_id}:automation_matrix_row_missing_target:{target}"
                ));
            }
        }
    }

    let mut counts = [0usize; 3];
    for (index, marker_kind) in ["green_markers", "red_markers", "non_claim_markers"]
        .iter()
        .enumerate()
    {
        let markers = array_after_key(entry, marker_kind)
            .map(object_slices_in_array)
            .unwrap_or_default();
        counts[index] = markers.len();
        if markers.is_empty() {
            failures.push(format!("{entry_id}:missing_{marker_kind}"));
            continue;
        }
        for marker in markers {
            validate_marker(
                root,
                marker,
                &format!("{entry_id}:{marker_kind}"),
                &mut failures,
            );
        }
    }

    if target_name.is_empty() && target.is_none() {
        failures.push(format!("{entry_id}:buck2_target_name_empty"));
    }

    EntryResult {
        id: entry_id,
        buck2_target: target,
        test_path_count: test_paths.len(),
        green_marker_count: counts[0],
        red_marker_count: counts[1],
        non_claim_marker_count: counts[2],
        automation_matrix_row_id: row_id,
        failures,
    }
}

pub fn evaluate(root: &Path, spec_path: &Path) -> Evaluation {
    let mut failures = Vec::new();
    let spec = if spec_path.is_file() {
        fs::read_to_string(spec_path).unwrap_or_default()
    } else {
        failures.push("missing_contract_spec".to_owned());
        String::new()
    };
    failures.extend(validate_spec(&spec));
    let buck_text = read(root, "BUCK", &mut failures);
    let matrix_text = read(root, "specs/phase0-automation-matrix.json", &mut failures);
    let targets = buck_targets(&buck_text);
    let entries_array = array_after_key(&spec, "fixture_contract_entries").unwrap_or("[]");
    let entries = object_slices_in_array(entries_array);
    if entries.is_empty() {
        failures.push("missing_fixture_contract_entries".to_owned());
    }
    let mut entry_results = Vec::new();
    for entry in entries {
        let result = validate_entry(root, entry, &targets, &matrix_text, &buck_text);
        failures.extend(result.failures.clone());
        entry_results.push(result);
    }
    let required_minimum = string_after_key(&spec, "minimum_entry_count")
        .and_then(|value| value.parse::<usize>().ok())
        .or_else(|| {
            let compact = compact_json_text(&spec);
            compact
                .split("\"minimum_entry_count\":")
                .nth(1)
                .and_then(|tail| tail.split([',', '}']).next())
                .and_then(|value| value.parse::<usize>().ok())
        });
    if let Some(minimum) = required_minimum {
        if entry_results.len() < minimum {
            failures.push("entry_count_below_minimum".to_owned());
        }
    }
    failures.sort();
    failures.dedup();
    let buck2_targets = entry_results
        .iter()
        .filter_map(|entry| entry.buck2_target.clone())
        .collect::<BTreeSet<_>>();
    let green_marker_count = entry_results
        .iter()
        .map(|entry| entry.green_marker_count)
        .sum();
    let red_marker_count = entry_results
        .iter()
        .map(|entry| entry.red_marker_count)
        .sum();
    let non_claim_marker_count = entry_results
        .iter()
        .map(|entry| entry.non_claim_marker_count)
        .sum();
    Evaluation {
        verdict: if failures.is_empty() {
            "PASS".to_owned()
        } else {
            "FAIL".to_owned()
        },
        failures,
        entry_count: entry_results.len(),
        buck2_target_count: buck2_targets.len(),
        green_marker_count,
        red_marker_count,
        non_claim_marker_count,
        entry_results,
        contract_spec: display_path(spec_path, root),
    }
}

fn entry_json(entry: &EntryResult) -> String {
    format!(
        concat!(
            "{{",
            "\"id\":{},",
            "\"buck2_target\":{},",
            "\"test_path_count\":{},",
            "\"green_marker_count\":{},",
            "\"red_marker_count\":{},",
            "\"non_claim_marker_count\":{},",
            "\"automation_matrix_row_id\":{},",
            "\"failures\":{}",
            "}}"
        ),
        json_string(&entry.id),
        entry
            .buck2_target
            .as_ref()
            .map(|target| json_string(target))
            .unwrap_or_else(|| "null".to_owned()),
        entry.test_path_count,
        entry.green_marker_count,
        entry.red_marker_count,
        entry.non_claim_marker_count,
        entry
            .automation_matrix_row_id
            .as_ref()
            .map(|row_id| json_string(row_id))
            .unwrap_or_else(|| "null".to_owned()),
        json_string_array(&entry.failures),
    )
}

pub fn to_json(evaluation: &Evaluation) -> String {
    let entries = evaluation
        .entry_results
        .iter()
        .map(entry_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"authority_boundary\":{},",
            "\"red_green_fixture_contract_measured\":{},",
            "\"status_mutation_performed\":false,",
            "\"protected_branch_authority_proven\":false,",
            "\"live_required_context_execution_proven\":false,",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"production_ready\":false,",
            "\"hyperscaler_grade\":false,",
            "\"contract_spec\":{},",
            "\"entry_count\":{},",
            "\"buck2_target_count\":{},",
            "\"green_marker_count\":{},",
            "\"red_marker_count\":{},",
            "\"non_claim_marker_count\":{},",
            "\"entries\":[{}],",
            "\"verdict\":{},",
            "\"failures\":{}",
            "}}"
        ),
        json_string(
            "AC-0.14 local/static RED/GREEN fixture registry evidence only; no status mutation, live required-context authority, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven"
        ),
        json_bool(evaluation.failures.is_empty()),
        json_string(&evaluation.contract_spec),
        evaluation.entry_count,
        evaluation.buck2_target_count,
        evaluation.green_marker_count,
        evaluation.red_marker_count,
        evaluation.non_claim_marker_count,
        entries,
        json_string(&evaluation.verdict),
        json_string_array(&evaluation.failures),
    )
}

fn print_usage(program: &str) {
    eprintln!("usage: {program} [--repo-root PATH] [--spec PATH] [--json]");
}

fn run() -> i32 {
    let mut repo_root = ".".to_owned();
    let mut spec = DEFAULT_SPEC.to_owned();
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = args.next() else {
                    print_usage("assert-red-green-fixture-contract");
                    return 2;
                };
                repo_root = value;
            }
            "--spec" => {
                let Some(value) = args.next() else {
                    print_usage("assert-red-green-fixture-contract");
                    return 2;
                };
                spec = value;
            }
            "--json" => json = true,
            "--help" | "-h" => {
                print_usage("assert-red-green-fixture-contract");
                return 0;
            }
            other => {
                eprintln!("unknown argument: {other}");
                print_usage("assert-red-green-fixture-contract");
                return 2;
            }
        }
    }
    let root = Path::new(&repo_root)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(repo_root));
    let spec_path = resolve(&root, &spec);
    let evaluation = evaluate(&root, &spec_path);
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
    std::process::exit(run());
}
