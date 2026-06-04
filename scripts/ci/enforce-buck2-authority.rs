//! Buck2 authority policy gate.
//!
//! This is the Rust/Buck2 replacement for the retired ad hoc Python authority
//! scanner. It is deliberately local/static evidence: it scans checked-in CI,
//! policy, Prow-parity, and root-hub contracts for Buck2 authority drift without
//! posting statuses, mutating branch protection, or claiming live P0.0 authority.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MATRIX_PATH: &str = "specs/phase0-automation-matrix.json";
const COVERAGE_REGISTRY_PATH: &str = "specs/phase0-automation-coverage-registry.json";
const PROW_PARITY_PATH: &str = "specs/oya-ci-prow-capability-parity.json";
const ROOT_HUB_PATH: &str = "specs/root-hub-pointers.json";
const BUCK2_AUTHORITY_ROW_ID: &str = "AC-0.0-buck2-authority-no-cargo-regression";
const BUCK2_AUTHORITY_VERIFICATION_COMMAND: &str = "buck2 build //:buck2-authority-policy-check";

const REQUIRED_PROW_BASELINE: &[(&str, &str)] = &[
    ("repository", "https://github.com/kubernetes-sigs/prow"),
    ("docs", "https://docs.prow.k8s.io/docs/"),
    (
        "architecture_docs",
        "https://docs.prow.k8s.io/docs/overview/architecture/",
    ),
    (
        "tide_docs",
        "https://docs.prow.k8s.io/docs/components/core/tide/",
    ),
];
const REQUIRED_AUTHORITY_PRODUCER_TERMS: &[&str] = &[
    "cloud-ci/oya-ci",
    "Rust Prow reimplementation",
    "trusted",
    "source-bound",
];
const REQUIRED_PROW_CONTRACT_TERMS: &[&str] = &[
    "Rust reimplementation",
    "improvement",
    "upstream Kubernetes Prow/Tide",
    "not a greenfield CI invention",
];
const BUCK2_AUTHORITY_CLAIM_BOUNDARY_TERMS: &[&str] = &[
    "not P0.0 green",
    "not protected-branch authority",
    "trusted cloud-ci/oya-ci",
];
const REQUIRED_PROW_CAPABILITY_IDS: &[&str] = &[
    "prow-hook-webhook-ingest",
    "prow-plugin-command-routing",
    "prow-prowjob-api-and-config",
    "prow-presubmit-jobs",
    "prow-postsubmit-jobs",
    "prow-periodic-jobs",
    "prow-batch-jobs",
    "prow-controller-manager-job-controller",
    "prow-crier-status-reporting",
    "prow-deck-web-ui",
    "prow-tide-merge-automation",
    "prow-sinker-gc",
    "prow-horologium-periodic-trigger",
    "prow-branchprotector",
    "prow-pod-utilities-clonerefs",
    "prow-pod-utilities-entrypoint-sidecar",
    "prow-artifact-storage",
    "prow-service-build-cluster-split",
    "prow-trusted-untrusted-execution",
    "prow-config-validation",
    "prow-label-approval-lgtm-policy",
    "prow-retest-and-trigger-policy",
    "prow-status-reconciliation",
    "prow-metrics-observability",
];
const REQUIRED_IMPROVEMENT_IDS: &[&str] = &[
    "rust-memory-safe-single-platform",
    "forgejo-native-no-github-gcs-coupling",
    "buck2-native-gate-execution",
    "self-hosted-artifact-storage",
    "tenant-isolated-trusted-controller",
    "source-bound-required-context",
    "buck2-native-llvm-coverage",
    "dual-cargo-buck2-mutation-testing-advisory",
];
const ALLOWED_PARITY_SCOPES: &[&str] = &[
    "direct_reimplementation",
    "equivalent_reimplementation",
    "improved_reimplementation",
    "superseded_by_improvement",
];
const ALLOWED_PARITY_STATUSES: &[&str] = &[
    "bridge_existing",
    "existing_partial",
    "phase0_contract",
    "phase1_target",
    "phase2_target",
    "phase3_target",
    "phase4_target",
];
const REQUIRED_EXCLUDED_OR_SUPERSEDED_COMPONENT_IDS: &[&str] = &[
    "prow-exporter",
    "prow-gcsupload",
    "prow-hmac",
    "prow-gerrit",
    "prow-tot",
    "prow-jenkins-operator",
];
const ALLOWED_EXCLUDED_OR_SUPERSEDED_DISPOSITIONS: &[&str] = &[
    "superseded_by_improvement",
    "out_of_scope_for_forgejo_native",
    "deferred_until_needed_with_waiver",
];

#[derive(Debug, Clone)]
pub struct Config {
    pub policy: String,
    pub matrix: String,
    pub coverage_registry: String,
    pub prow_parity_registry: String,
    pub root_hub: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            policy: "specs/buck2-authority-policy.json".to_owned(),
            matrix: MATRIX_PATH.to_owned(),
            coverage_registry: COVERAGE_REGISTRY_PATH.to_owned(),
            prow_parity_registry: PROW_PARITY_PATH.to_owned(),
            root_hub: ROOT_HUB_PATH.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
    pub policy: String,
    pub command_scan_files: usize,
    pub command_scan_globs: usize,
    pub status_context_scan_files: usize,
    pub adr_amendment_files: usize,
    pub authority_context: String,
    pub claim_boundary: String,
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

fn compact_json_text(input: &str) -> String {
    input.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", json_escape(value))
}

fn contains_json_string(text: &str, value: &str) -> bool {
    text.contains(&json_string(value))
}

fn count_json_key_value(text: &str, key: &str, value: &str) -> usize {
    compact_json_text(text)
        .matches(&format!("\"{}\":\"{}\"", key, json_escape(value)))
        .count()
}

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&format!(
        "\"{}\":{}",
        key,
        if value { "true" } else { "false" }
    ))
}

fn read(root: &Path, rel_or_abs: &str, failures: &mut Vec<String>) -> String {
    let path = resolve(root, rel_or_abs);
    match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            failures.push(format!("{rel_or_abs}: read failed: {error}"));
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

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn require(condition: bool, failures: &mut Vec<String>, message: impl Into<String>) {
    if !condition {
        failures.push(message.into());
    }
}

fn skip_ws(text: &str, mut index: usize) -> usize {
    while let Some(ch) = text[index..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        index += ch.len_utf8();
        if index >= text.len() {
            break;
        }
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
                    // The policy files use ASCII anchors. Preserve unicode escapes in
                    // the unlikely case they appear so string matching remains stable.
                    if index + 4 <= text.len() {
                        let raw = &text[index..index + 4];
                        index += 4;
                        out.push_str("\\u");
                        out.push_str(raw);
                        continue;
                    }
                    return None;
                }
                other => other,
            };
            out.push(decoded);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some((out, index)),
            other => out.push(other),
        }
    }
    None
}

fn find_key_value_start(text: &str, key: &str) -> Option<usize> {
    let needle = json_string(key);
    let key_start = text.find(&needle)?;
    let after_key = skip_ws(text, key_start + needle.len());
    if text[after_key..].chars().next()? != ':' {
        return None;
    }
    Some(skip_ws(text, after_key + 1))
}

fn matching_delim(text: &str, open_index: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    let mut index = open_index;
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
        match ch {
            '"' => in_string = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + ch.len_utf8());
                }
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    None
}

fn array_segment(text: &str, key: &str) -> Option<String> {
    let start = find_key_value_start(text, key)?;
    if text[start..].chars().next()? != '[' {
        return None;
    }
    let end = matching_delim(text, start, '[', ']')?;
    Some(text[start..end].to_owned())
}

fn object_segment(text: &str, key: &str) -> Option<String> {
    let start = find_key_value_start(text, key)?;
    if text[start..].chars().next()? != '{' {
        return None;
    }
    let end = matching_delim(text, start, '{', '}')?;
    Some(text[start..end].to_owned())
}

fn quoted_values(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut index = 0usize;
    while index < text.len() {
        if text[index..].chars().next() == Some('"') {
            if let Some((value, next)) = parse_json_string_at(text, index) {
                values.push(value);
                index = next;
                continue;
            }
        }
        index += text[index..]
            .chars()
            .next()
            .map(|ch| ch.len_utf8())
            .unwrap_or(1);
    }
    values
}

fn string_array(text: &str, key: &str) -> Vec<String> {
    array_segment(text, key)
        .map(|segment| quoted_values(&segment))
        .unwrap_or_default()
}

fn string_field(text: &str, key: &str) -> Option<String> {
    let start = find_key_value_start(text, key)?;
    parse_json_string_at(text, start).map(|(value, _)| value)
}

fn bool_field(text: &str, key: &str) -> Option<bool> {
    let start = find_key_value_start(text, key)?;
    if text[start..].starts_with("true") {
        Some(true)
    } else if text[start..].starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn object_segments(array_text: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut index = 0usize;
    while index < array_text.len() {
        if array_text[index..].chars().next() == Some('{') {
            if let Some(end) = matching_delim(array_text, index, '{', '}') {
                objects.push(array_text[index..end].to_owned());
                index = end;
                continue;
            }
        }
        index += array_text[index..]
            .chars()
            .next()
            .map(|ch| ch.len_utf8())
            .unwrap_or(1);
    }
    objects
}

fn parse_string_array_map(text: &str, key: &str) -> BTreeMap<String, Vec<String>> {
    let Some(object) = object_segment(text, key) else {
        return BTreeMap::new();
    };
    let mut map = BTreeMap::new();
    let mut index = 1usize;
    while index < object.len() {
        index = skip_ws(&object, index);
        if index >= object.len() || object[index..].chars().next() == Some('}') {
            break;
        }
        if object[index..].chars().next() != Some('"') {
            index += object[index..]
                .chars()
                .next()
                .map(|ch| ch.len_utf8())
                .unwrap_or(1);
            continue;
        }
        let Some((entry_key, after_key)) = parse_json_string_at(&object, index) else {
            break;
        };
        let colon = skip_ws(&object, after_key);
        if object[colon..].chars().next() != Some(':') {
            index = after_key;
            continue;
        }
        let value_start = skip_ws(&object, colon + 1);
        if object[value_start..].chars().next() == Some('[') {
            if let Some(value_end) = matching_delim(&object, value_start, '[', ']') {
                map.insert(entry_key, quoted_values(&object[value_start..value_end]));
                index = value_end;
                continue;
            }
        }
        index = value_start + 1;
    }
    map
}

fn simple_match(value: &str, pattern: &str) -> bool {
    if pattern == value {
        return true;
    }
    if !pattern.contains('*') {
        return false;
    }
    let mut rest = value;
    let mut first = true;
    for part in pattern.split('*') {
        if part.is_empty() {
            continue;
        }
        if first && !pattern.starts_with('*') {
            if !rest.starts_with(part) {
                return false;
            }
            rest = &rest[part.len()..];
        } else if let Some(index) = rest.find(part) {
            rest = &rest[index + part.len()..];
        } else {
            return false;
        }
        first = false;
    }
    pattern.ends_with('*') || rest.is_empty()
}

fn collect_files(dir: &Path, root: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if matches!(name.as_str(), ".git" | "buck-out" | "target") {
                continue;
            }
            collect_files(&path, root, out);
        } else if path.is_file() {
            out.push(display_path(root, &path));
        }
    }
}

fn expand_glob(root: &Path, pattern: &str) -> Vec<String> {
    let prefix = pattern
        .split(['*', '?', '['])
        .next()
        .unwrap_or("")
        .trim_end_matches('/');
    let start = if prefix.is_empty() {
        root.to_path_buf()
    } else {
        root.join(prefix)
    };
    let mut files = Vec::new();
    collect_files(&start, root, &mut files);
    files
        .into_iter()
        .filter(|path| simple_match(path, pattern))
        .collect()
}

fn expand_policy_paths(root: &Path, policy: &str, file_key: &str, glob_key: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut seen = BTreeSet::new();
    for file_name in string_array(policy, file_key) {
        if seen.insert(file_name.clone()) {
            paths.push(file_name);
        }
    }
    for pattern in string_array(policy, glob_key) {
        let mut matches = expand_glob(root, &pattern);
        matches.sort();
        if matches.is_empty() {
            paths.push(format!("<missing-glob:{pattern}>"));
            continue;
        }
        for file_name in matches {
            if seen.insert(file_name.clone()) {
                paths.push(file_name);
            }
        }
    }
    paths
}

fn is_word_boundary_before(ch: Option<char>) -> bool {
    match ch {
        None => true,
        Some(c) => c.is_whitespace() || matches!(c, ';' | '&' | '|' | '(' | '`'),
    }
}

fn line_has_forbidden_cargo(line: &str, subcommands: &[String]) -> bool {
    let lower = line.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut index = 0usize;
    while let Some(pos) = lower[index..].find("cargo") {
        let start = index + pos;
        let before = lower[..start].chars().next_back();
        let after = start + "cargo".len();
        if !is_word_boundary_before(before)
            || bytes.get(after).copied() != Some(b' ') && bytes.get(after).copied() != Some(b'\t')
        {
            index = after;
            continue;
        }
        let mut cursor = skip_ws(&lower, after);
        if lower[cursor..].starts_with('+') {
            while cursor < lower.len() {
                let Some(ch) = lower[cursor..].chars().next() else {
                    break;
                };
                if ch.is_whitespace() {
                    break;
                }
                cursor += ch.len_utf8();
            }
            cursor = skip_ws(&lower, cursor);
        }
        let command_start = cursor;
        while cursor < lower.len() {
            let Some(ch) = lower[cursor..].chars().next() else {
                break;
            };
            if !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') {
                break;
            }
            cursor += ch.len_utf8();
        }
        let command = &lower[command_start..cursor];
        let after_command = lower[cursor..].chars().next();
        let command_terminated = after_command.map(|ch| ch.is_whitespace()).unwrap_or(true);
        if command_terminated && subcommands.iter().any(|sub| sub == command) {
            return true;
        }
        index = after;
    }
    false
}

fn validate_root_hub_pointer(root_hub: &str, failures: &mut Vec<String>) {
    let Some(entry_points) = object_segment(root_hub, "entry_points") else {
        failures.push(format!("{ROOT_HUB_PATH}.entry_points: expected object"));
        return;
    };
    let Some(entry) = object_segment(&entry_points, "oya_ci_prow_capability_parity") else {
        failures.push(format!(
            "{ROOT_HUB_PATH}.entry_points must include oya_ci_prow_capability_parity"
        ));
        return;
    };
    if string_field(&entry, "kind").as_deref() != Some("spec") {
        failures.push("root hub oya_ci_prow_capability_parity.kind must be spec".to_owned());
    }
    if string_field(&entry, "current_path").as_deref() != Some(&format!("/{PROW_PARITY_PATH}")) {
        failures.push(format!(
            "root hub oya_ci_prow_capability_parity.current_path must be /{PROW_PARITY_PATH}"
        ));
    }
    let purpose = string_field(&entry, "purpose").unwrap_or_default();
    if !purpose.contains("Prow") || !purpose.contains("Rust") {
        failures.push(
            "root hub oya_ci_prow_capability_parity.purpose must describe Rust Prow parity"
                .to_owned(),
        );
    }
    let pointers = object_segment(root_hub, "pointers").unwrap_or_default();
    if string_field(&pointers, "oya_ci_prow_capability_parity").as_deref() != Some(PROW_PARITY_PATH)
    {
        failures.push(format!(
            "{ROOT_HUB_PATH}.pointers.oya_ci_prow_capability_parity must point to {PROW_PARITY_PATH}"
        ));
    }
}

fn sorted_missing(required: &[&str], present: &BTreeSet<String>) -> Vec<String> {
    required
        .iter()
        .filter(|item| !present.contains(**item))
        .map(|item| (*item).to_owned())
        .collect()
}

fn validate_prow_parity_registry(registry: &str, failures: &mut Vec<String>) {
    let Some(claim_boundary) = object_segment(registry, "claim_boundary") else {
        failures.push(format!(
            "{PROW_PARITY_PATH}.claim_boundary: expected object"
        ));
        return;
    };
    for key in [
        "p0_0_green",
        "phase0_complete",
        "live_full_parity_claimed",
        "protected_branch_authority_proven",
        "production_readiness",
        "hyperscaler_grade_readiness",
    ] {
        if bool_field(&claim_boundary, key) != Some(false) {
            failures.push(format!(
                "{PROW_PARITY_PATH}.claim_boundary.{key} must be false"
            ));
        }
    }

    let Some(upstream_sources) = object_segment(registry, "upstream_sources") else {
        failures.push(format!(
            "{PROW_PARITY_PATH}.upstream_sources: expected object"
        ));
        return;
    };
    for (policy_key, registry_key, expected) in [
        (
            "repository",
            "repository",
            "https://github.com/kubernetes-sigs/prow",
        ),
        ("docs", "documentation", "https://docs.prow.k8s.io/docs/"),
        (
            "architecture_docs",
            "architecture",
            "https://docs.prow.k8s.io/docs/overview/architecture/",
        ),
        (
            "tide_docs",
            "tide",
            "https://docs.prow.k8s.io/docs/components/core/tide/",
        ),
    ] {
        if string_field(&upstream_sources, registry_key).as_deref() != Some(expected) {
            failures.push(format!(
                "{PROW_PARITY_PATH}.upstream_sources.{registry_key} must be {expected}"
            ));
        }
        let _ = policy_key;
    }
    if string_field(&upstream_sources, "controller_manager").as_deref()
        != Some("https://docs.prow.k8s.io/docs/components/")
    {
        failures.push(format!(
            "{PROW_PARITY_PATH}.upstream_sources.controller_manager must cite the upstream components page"
        ));
    }
    if string_field(&upstream_sources, "plank_deprecated").as_deref()
        != Some("https://docs.prow.k8s.io/docs/components/deprecated/plank/")
    {
        failures.push(format!(
            "{PROW_PARITY_PATH}.upstream_sources.plank_deprecated must cite deprecated Plank as legacy context only"
        ));
    }

    let required_capability_ids: BTreeSet<String> =
        string_array(registry, "required_capability_ids")
            .into_iter()
            .collect();
    let missing_required_ids =
        sorted_missing(REQUIRED_PROW_CAPABILITY_IDS, &required_capability_ids);
    if !missing_required_ids.is_empty() {
        failures.push(format!(
            "{PROW_PARITY_PATH}.required_capability_ids missing {}",
            missing_required_ids.join(", ")
        ));
    }

    let capability_objects = array_segment(registry, "capabilities")
        .map(|array| object_segments(&array))
        .unwrap_or_default();
    let mut capability_ids = BTreeSet::new();
    let mut duplicate_capability_id = false;
    for capability in &capability_objects {
        if let Some(id) = string_field(capability, "id") {
            if !capability_ids.insert(id) {
                duplicate_capability_id = true;
            }
        }
    }
    let missing_capabilities = sorted_missing(REQUIRED_PROW_CAPABILITY_IDS, &capability_ids);
    if !missing_capabilities.is_empty() {
        failures.push(format!(
            "{PROW_PARITY_PATH}.capabilities missing {}",
            missing_capabilities.join(", ")
        ));
    }
    if duplicate_capability_id {
        failures.push(format!(
            "{PROW_PARITY_PATH}.capabilities must have unique string ids"
        ));
    }

    for capability in &capability_objects {
        let Some(capability_id) = string_field(capability, "id") else {
            continue;
        };
        for field in [
            "upstream_feature",
            "upstream_source",
            "oya_ci_equivalent",
            "parity_requirement",
            "verification",
        ] {
            if string_field(capability, field)
                .map(|value| value.trim().is_empty())
                .unwrap_or(true)
            {
                failures.push(format!(
                    "{PROW_PARITY_PATH}.{capability_id}.{field}: expected non-empty string"
                ));
            }
        }
        if !string_field(capability, "parity_scope")
            .map(|value| ALLOWED_PARITY_SCOPES.contains(&value.as_str()))
            .unwrap_or(false)
        {
            failures.push(format!(
                "{PROW_PARITY_PATH}.{capability_id}.parity_scope is not allowed"
            ));
        }
        if !string_field(capability, "current_status")
            .map(|value| ALLOWED_PARITY_STATUSES.contains(&value.as_str()))
            .unwrap_or(false)
        {
            failures.push(format!(
                "{PROW_PARITY_PATH}.{capability_id}.current_status is not allowed"
            ));
        }
        if string_array(capability, "repo_artifacts").is_empty() {
            failures.push(format!(
                "{PROW_PARITY_PATH}.{capability_id}.repo_artifacts must list local artifacts"
            ));
        }
        if string_array(capability, "improvements_over_upstream").is_empty() {
            failures.push(format!(
                "{PROW_PARITY_PATH}.{capability_id}.improvements_over_upstream must be non-empty"
            ));
        }
        if bool_field(capability, "live_authority_claimed") != Some(false) {
            failures.push(format!(
                "{PROW_PARITY_PATH}.{capability_id}.live_authority_claimed must be false"
            ));
        }
    }

    let excluded_objects = array_segment(registry, "excluded_or_superseded_upstream_components")
        .map(|array| object_segments(&array))
        .unwrap_or_default();
    let mut excluded_ids = BTreeSet::new();
    for item in &excluded_objects {
        if let Some(id) = string_field(item, "id") {
            excluded_ids.insert(id);
        }
    }
    let missing_excluded =
        sorted_missing(REQUIRED_EXCLUDED_OR_SUPERSEDED_COMPONENT_IDS, &excluded_ids);
    if !missing_excluded.is_empty() {
        failures.push(format!(
            "{PROW_PARITY_PATH}.excluded_or_superseded_upstream_components missing {}",
            missing_excluded.join(", ")
        ));
    }
    for item in &excluded_objects {
        let Some(item_id) = string_field(item, "id") else {
            failures.push(format!(
                "{PROW_PARITY_PATH}.excluded_or_superseded_upstream_components item missing string id"
            ));
            continue;
        };
        if !string_field(item, "disposition")
            .map(|value| ALLOWED_EXCLUDED_OR_SUPERSEDED_DISPOSITIONS.contains(&value.as_str()))
            .unwrap_or(false)
        {
            failures.push(format!(
                "{PROW_PARITY_PATH}.{item_id}.disposition is not allowed"
            ));
        }
        for field in [
            "upstream_component",
            "upstream_source",
            "rationale",
            "replacement_or_reason",
        ] {
            if string_field(item, field)
                .map(|value| value.trim().is_empty())
                .unwrap_or(true)
            {
                failures.push(format!(
                    "{PROW_PARITY_PATH}.{item_id}.{field}: expected non-empty string"
                ));
            }
        }
    }

    let required_improvement_ids: BTreeSet<String> =
        string_array(registry, "required_improvement_ids")
            .into_iter()
            .collect();
    let missing_improvement_ids =
        sorted_missing(REQUIRED_IMPROVEMENT_IDS, &required_improvement_ids);
    if !missing_improvement_ids.is_empty() {
        failures.push(format!(
            "{PROW_PARITY_PATH}.required_improvement_ids missing {}",
            missing_improvement_ids.join(", ")
        ));
    }
    let improvement_objects = array_segment(registry, "improvements")
        .map(|array| object_segments(&array))
        .unwrap_or_default();
    let mut improvement_ids = BTreeSet::new();
    for item in &improvement_objects {
        if let Some(id) = string_field(item, "id") {
            improvement_ids.insert(id);
        }
    }
    let missing_improvements = sorted_missing(REQUIRED_IMPROVEMENT_IDS, &improvement_ids);
    if !missing_improvements.is_empty() {
        failures.push(format!(
            "{PROW_PARITY_PATH}.improvements missing {}",
            missing_improvements.join(", ")
        ));
    }
    for improvement in &improvement_objects {
        let Some(improvement_id) = string_field(improvement, "id") else {
            continue;
        };
        if string_field(improvement, "description")
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
        {
            failures.push(format!(
                "{PROW_PARITY_PATH}.{improvement_id}.description: expected non-empty string"
            ));
        }
        let mapped: BTreeSet<String> = string_array(improvement, "capability_ids")
            .into_iter()
            .collect();
        if mapped.is_empty() {
            failures.push(format!(
                "{PROW_PARITY_PATH}.{improvement_id}.capability_ids must be non-empty"
            ));
        }
        let unknown: Vec<String> = mapped.difference(&capability_ids).cloned().collect();
        if !unknown.is_empty() {
            failures.push(format!(
                "{PROW_PARITY_PATH}.{improvement_id}.capability_ids unknown: {}",
                unknown.join(", ")
            ));
        }
    }
}

fn validate_matrix(matrix: &str, failures: &mut Vec<String>) {
    require(
        count_json_key_value(matrix, "id", BUCK2_AUTHORITY_ROW_ID) == 1,
        failures,
        format!(
            "{MATRIX_PATH}.seed_rows: expected exactly one {BUCK2_AUTHORITY_ROW_ID} entry, found {}",
            count_json_key_value(matrix, "id", BUCK2_AUTHORITY_ROW_ID)
        ),
    );
    require(
        contains_json_string(matrix, BUCK2_AUTHORITY_VERIFICATION_COMMAND),
        failures,
        format!("{BUCK2_AUTHORITY_ROW_ID} row must record Buck2 authority verification command"),
    );
    for term in BUCK2_AUTHORITY_CLAIM_BOUNDARY_TERMS {
        require(
            matrix.contains(term),
            failures,
            format!("{BUCK2_AUTHORITY_ROW_ID}.claim_boundary missing '{term}'"),
        );
    }
    require(
        has_bool(matrix, "no_new_oya_cli_surface", true),
        failures,
        format!("{BUCK2_AUTHORITY_ROW_ID}.no_new_oya_cli_surface must be true"),
    );
}

fn validate_coverage(coverage: &str, failures: &mut Vec<String>) {
    require(
        count_json_key_value(coverage, "id", "AC-0.0") == 1,
        failures,
        format!("{COVERAGE_REGISTRY_PATH}.coverage_subjects: expected exactly one AC-0.0 entry"),
    );
    require(
        contains_json_string(coverage, BUCK2_AUTHORITY_ROW_ID),
        failures,
        format!("AC-0.0 coverage subject must map {BUCK2_AUTHORITY_ROW_ID}"),
    );
    require(
        contains_json_string(coverage, BUCK2_AUTHORITY_VERIFICATION_COMMAND),
        failures,
        format!("AC-0.0 coverage subject must record Buck2 command for {BUCK2_AUTHORITY_ROW_ID}"),
    );
}

fn validate_policy_baseline(policy: &str, failures: &mut Vec<String>) -> (String, String) {
    for id in [
        "production-release-image-binary-optimization",
        "buck2-graph-metadata-only",
    ] {
        require(
            count_json_key_value(policy, "id", id) > 0,
            failures,
            if id == "production-release-image-binary-optimization" {
                "policy lacks production release image/binary Cargo exception".to_owned()
            } else {
                "policy lacks metadata-only Buck2 graph exception".to_owned()
            },
        );
    }

    let upstream = object_segment(policy, "upstream_prow_baseline");
    let upstream_text = upstream.as_deref().unwrap_or("");
    if upstream.is_none() {
        failures.push(
            "policy must record upstream Kubernetes Prow baseline for the Rust reimplementation"
                .to_owned(),
        );
    } else {
        for (key, expected) in REQUIRED_PROW_BASELINE {
            if string_field(upstream_text, key).as_deref() != Some(*expected) {
                failures.push(format!("upstream_prow_baseline.{key} must be {expected}"));
            }
        }
        let contract = string_field(upstream_text, "contract").unwrap_or_default();
        for term in REQUIRED_PROW_CONTRACT_TERMS {
            if !contract.contains(term) {
                failures.push(format!("upstream_prow_baseline.contract missing '{term}'"));
            }
        }
    }

    let target_authority = object_segment(policy, "target_authority");
    let target_text = target_authority.as_deref().unwrap_or("");
    let authority_context = string_field(target_text, "required_context").unwrap_or_default();
    if target_authority.is_none() {
        failures.push("target_authority must be an object".to_owned());
    } else if authority_context != "oya-ci-required" {
        failures.push("target_authority.required_context must be oya-ci-required".to_owned());
    }
    let producer = string_field(target_text, "producer").unwrap_or_default();
    if producer.is_empty() {
        failures.push("target_authority.producer must be a string".to_owned());
    }
    for term in REQUIRED_AUTHORITY_PRODUCER_TERMS {
        if !producer.contains(term) {
            failures.push(format!("target_authority.producer must contain '{term}'"));
        }
    }
    let claim_boundary = string_field(policy, "claim_boundary").unwrap_or_default();
    (authority_context, claim_boundary)
}

pub fn evaluate(root: &Path, config: &Config) -> Evaluation {
    let mut failures = Vec::new();
    let policy_path = resolve(root, &config.policy);
    let policy = read(root, &config.policy, &mut failures);
    let cargo_subcommands = string_array(&policy, "forbidden_cargo_subcommands")
        .into_iter()
        .map(|item| item.to_ascii_lowercase())
        .collect::<Vec<_>>();

    let command_scan_files =
        expand_policy_paths(root, &policy, "command_scan_files", "command_scan_globs");
    for file_name in &command_scan_files {
        if file_name.starts_with("<missing-glob:") {
            failures.push(format!(
                "command-scan glob matched no files: {}",
                file_name
                    .trim_start_matches("<missing-glob:")
                    .trim_end_matches('>')
            ));
            continue;
        }
        let file_path = root.join(file_name);
        if !file_path.is_file() {
            failures.push(format!("missing command-scan file: {file_name}"));
            continue;
        }
        let text = fs::read_to_string(&file_path).unwrap_or_default();
        for (line_no, line) in text.lines().enumerate() {
            if line_has_forbidden_cargo(line, &cargo_subcommands) {
                failures.push(format!(
                    "{file_name}:{}: forbidden Cargo executable lane: {}",
                    line_no + 1,
                    line.trim()
                ));
            }
        }
    }

    let status_context_scan_files = string_array(&policy, "status_context_scan_files");
    let forbidden_contexts = string_array(&policy, "forbidden_status_contexts");
    for file_name in &status_context_scan_files {
        let file_path = root.join(file_name);
        if !file_path.is_file() {
            failures.push(format!("missing status-context-scan file: {file_name}"));
            continue;
        }
        let text = fs::read_to_string(&file_path).unwrap_or_default();
        for context in &forbidden_contexts {
            if text.contains(context) {
                failures.push(format!(
                    "{file_name}: forbidden legacy status context '{context}'; use oya-ci-required"
                ));
            }
        }
    }

    for (file_name, anchors) in parse_string_array_map(&policy, "required_anchors") {
        let file_path = root.join(&file_name);
        if !file_path.is_file() {
            failures.push(format!("missing required-anchor file: {file_name}"));
            continue;
        }
        let text = fs::read_to_string(&file_path).unwrap_or_default();
        for anchor in anchors {
            if !text.contains(&anchor) {
                failures.push(format!(
                    "{file_name}: missing required Buck2 authority anchor '{anchor}'"
                ));
            }
        }
    }

    for group in array_segment(&policy, "required_glob_anchors")
        .map(|array| object_segments(&array))
        .unwrap_or_default()
    {
        let pattern = string_field(&group, "glob").unwrap_or_default();
        let anchors = string_array(&group, "anchors");
        let mut matches = expand_glob(root, &pattern);
        matches.sort();
        if matches.is_empty() {
            failures.push(format!("required-anchor glob matched no files: {pattern}"));
            continue;
        }
        for file_name in matches {
            let text = fs::read_to_string(root.join(&file_name)).unwrap_or_default();
            for anchor in &anchors {
                if !text.contains(anchor) {
                    failures.push(format!(
                        "{file_name}: missing required Buck2 authority anchor '{anchor}'"
                    ));
                }
            }
        }
    }

    let amendment = string_field(&policy, "required_adr_amendment_text").unwrap_or_default();
    let adr_amendment_files = string_array(&policy, "adr_amendment_files");
    for file_name in &adr_amendment_files {
        let file_path = root.join(file_name);
        if !file_path.is_file() {
            failures.push(format!("missing ADR amendment file: {file_name}"));
            continue;
        }
        let text = fs::read_to_string(&file_path).unwrap_or_default();
        if !text.contains(&amendment) || !text.contains("specs/buck2-authority-policy.json") {
            failures.push(format!(
                "{file_name}: missing '{amendment}' and policy cross-reference"
            ));
        }
    }

    let (authority_context, claim_boundary) = validate_policy_baseline(&policy, &mut failures);
    let matrix = read(root, &config.matrix, &mut failures);
    validate_matrix(&matrix, &mut failures);
    let prow_parity = read(root, &config.prow_parity_registry, &mut failures);
    validate_prow_parity_registry(&prow_parity, &mut failures);
    let root_hub = read(root, &config.root_hub, &mut failures);
    validate_root_hub_pointer(&root_hub, &mut failures);
    let coverage = read(root, &config.coverage_registry, &mut failures);
    validate_coverage(&coverage, &mut failures);

    Evaluation {
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned(),
        failures,
        policy: display_path(root, &policy_path),
        command_scan_files: command_scan_files.len(),
        command_scan_globs: string_array(&policy, "command_scan_globs").len(),
        status_context_scan_files: status_context_scan_files.len(),
        adr_amendment_files: adr_amendment_files.len(),
        authority_context,
        claim_boundary,
    }
}

fn render_json(evaluation: &Evaluation) -> String {
    format!(
        "{{\"adr_amendment_files\":{},\"authority_context\":{},\"claim_boundary\":{},\"command_scan_files\":{},\"command_scan_globs\":{},\"policy\":{},\"status_context_scan_files\":{},\"verdict\":{}}}",
        evaluation.adr_amendment_files,
        json_string(&evaluation.authority_context),
        json_string(&evaluation.claim_boundary),
        evaluation.command_scan_files,
        evaluation.command_scan_globs,
        json_string(&evaluation.policy),
        evaluation.status_context_scan_files,
        json_string(&evaluation.verdict),
    )
}

fn parse_args() -> Config {
    let mut config = Config::default();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--policy" => config.policy = args.next().unwrap_or_else(|| config.policy.clone()),
            "--matrix" => config.matrix = args.next().unwrap_or_else(|| config.matrix.clone()),
            "--coverage-registry" => {
                config.coverage_registry = args
                    .next()
                    .unwrap_or_else(|| config.coverage_registry.clone())
            }
            "--prow-parity-registry" => {
                config.prow_parity_registry = args
                    .next()
                    .unwrap_or_else(|| config.prow_parity_registry.clone())
            }
            "--root-hub" => {
                config.root_hub = args.next().unwrap_or_else(|| config.root_hub.clone())
            }
            _ => {}
        }
    }
    config
}

fn repo_root() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf))
                .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        })
        .canonicalize()
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn main() {
    let config = parse_args();
    let root = repo_root();
    let evaluation = evaluate(&root, &config);
    if !evaluation.failures.is_empty() {
        eprintln!("buck2-authority-policy: RED");
        for failure in &evaluation.failures {
            eprintln!("- {failure}");
        }
        std::process::exit(1);
    }
    println!("{}", render_json(&evaluation));
}
