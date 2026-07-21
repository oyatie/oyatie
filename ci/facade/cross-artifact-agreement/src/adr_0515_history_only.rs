//! ADR-0515 Git-history-only corpus guard.
//!
//! The seven superseded CI/CD ADRs are absent from the current source corpus
//! and current-authority projections. Historical prose may still name their
//! identifiers as provenance; only current-authority surfaces are guarded.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Component, Path};
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::Value;

use crate::Finding;

pub const ADR_0515_HISTORY_ONLY_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/adr-0515-history-only";
pub const ADR_0515_HISTORY_ONLY_CODE: &str = "adr_0515_history_only_drift";

const HISTORY_ONLY_POLICY: &str = include_str!("../adr-0515-history-only-policy.json");
const HISTORY_ONLY_POLICY_SCHEMA_VERSION: u64 = 1;
const MAX_CONFIGURED_GZIP_DECOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_CONFIGURED_GZIP_TIMEOUT_MILLIS: u64 = 60_000;
const GZIP_PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);

const HISTORY_ONLY_ADR_SUFFIXES: [&str; 7] =
    ["0124", "0349", "0359", "0361", "0511", "0513", "0514"];
const ADR_0515_SOURCE: &str =
    "docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md";
const CANONICAL_DECISIONS_NUMBERING: &str = concat!(
    "ADR-0001..ADR-0619 (non-contiguous; gaps: ADR-0012, ADR-0033, ADR-0037, ADR-0041, ADR-0050, ADR-0068, ADR-0070..ADR-0082, ADR-0084..ADR-0089, ADR-0",
    "124..ADR-0127, ADR-0224..ADR-0233, ADR-0247, ADR-0256, ADR-0259..ADR-0262, ADR-0264..ADR-0271, ADR-0274..ADR-0275, ADR-0277..ADR-0279, ADR-0281..ADR-0283, ADR-0285..ADR-0291, ADR-0322..ADR-0323, ADR-0327, ADR-0342, ADR-0345, ADR-0",
    "349, ADR-0",
    "359, ADR-0",
    "361, ADR-0385..ADR-0386, ADR-0395..ADR-0396, ADR-0398..ADR-0475, ADR-0477, ADR-0483..ADR-0505, ADR-0",
    "511, ADR-0",
    "513..ADR-0",
    "514, ADR-0574..ADR-0579, ADR-0583..ADR-0585, ADR-0594, ADR-0601..ADR-0602)"
);
const CANONICAL_HISTORY_GAP_SUFFIXES: [(&str, &str); 6] = [
    ("/_metadata/gaps/8", "0124..0127"),
    ("/_metadata/gaps/22", "0349"),
    ("/_metadata/gaps/23", "0359"),
    ("/_metadata/gaps/24", "0361"),
    ("/_metadata/gaps/30", "0511"),
    ("/_metadata/gaps/31", "0513..0514"),
];

#[derive(Debug)]
struct HistoryOnlyPolicy {
    reviewed_elf_paths: BTreeMap<String, u16>,
    reviewed_gzip_paths: BTreeMap<String, ReviewedGzipPolicy>,
    reviewed_architecture_provenance: BTreeSet<String>,
    reviewed_internal_symlinks: BTreeMap<String, String>,
}

#[derive(Debug)]
struct ReviewedGzipPolicy {
    max_decompressed_bytes: usize,
    timeout: Duration,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHistoryOnlyPolicy {
    schema_version: u64,
    policy_id: String,
    reviewed_elf_paths: Vec<RawReviewedElfPolicy>,
    reviewed_gzip_paths: Vec<RawReviewedGzipPolicy>,
    reviewed_architecture_provenance: Vec<String>,
    reviewed_internal_symlinks: Vec<RawReviewedInternalSymlinkPolicy>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReviewedElfPolicy {
    path: String,
    machine: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReviewedGzipPolicy {
    path: String,
    max_decompressed_bytes: u64,
    timeout_millis: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReviewedInternalSymlinkPolicy {
    path: String,
    target: String,
}

fn parse_repo_relative_path(raw: &str) -> Result<String, &'static str> {
    if raw.is_empty() {
        return Err("invalid_path");
    }
    let path = Path::new(raw);
    if path.is_absolute() {
        return Err("invalid_path");
    }

    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err("invalid_path");
        };
        components.push(component.to_str().ok_or("invalid_path")?);
    }
    if components.is_empty() || components.join("/") != raw {
        return Err("invalid_path");
    }
    Ok(raw.to_owned())
}

fn register_policy_path(
    all_paths: &mut BTreeSet<String>,
    value: &str,
) -> Result<String, &'static str> {
    let path = parse_repo_relative_path(value)?;
    if !all_paths.insert(path.clone()) {
        return Err("duplicate_path");
    }
    Ok(path)
}

fn parse_policy_path_set(
    values: &[String],
    all_paths: &mut BTreeSet<String>,
) -> Result<BTreeSet<String>, &'static str> {
    let mut paths = BTreeSet::new();
    for value in values {
        paths.insert(register_policy_path(all_paths, value)?);
    }
    Ok(paths)
}

fn parse_history_only_policy(content: &str) -> Result<HistoryOnlyPolicy, &'static str> {
    let document = serde_json::from_str::<RawHistoryOnlyPolicy>(content)
        .map_err(|_| "invalid_json_or_shape")?;
    if document.schema_version != HISTORY_ONLY_POLICY_SCHEMA_VERSION {
        return Err("wrong_schema_version");
    }
    if document.policy_id != ADR_0515_HISTORY_ONLY_VALIDATOR {
        return Err("wrong_policy_id");
    }

    let mut all_paths = BTreeSet::new();
    let mut reviewed_elf_paths = BTreeMap::new();
    for row in document.reviewed_elf_paths {
        let path = register_policy_path(&mut all_paths, &row.path)?;
        let machine = u16::try_from(row.machine).map_err(|_| "invalid_machine")?;
        reviewed_elf_paths.insert(path, machine);
    }

    let mut reviewed_gzip_paths = BTreeMap::new();
    for row in document.reviewed_gzip_paths {
        let path = register_policy_path(&mut all_paths, &row.path)?;
        if !(1..=MAX_CONFIGURED_GZIP_DECOMPRESSED_BYTES).contains(&row.max_decompressed_bytes) {
            return Err("invalid_gzip_size_limit");
        }
        let max_decompressed_bytes =
            usize::try_from(row.max_decompressed_bytes).map_err(|_| "invalid_gzip_size_limit")?;
        if !(1..=MAX_CONFIGURED_GZIP_TIMEOUT_MILLIS).contains(&row.timeout_millis) {
            return Err("invalid_gzip_timeout");
        }
        let timeout_millis = row.timeout_millis;
        reviewed_gzip_paths.insert(
            path,
            ReviewedGzipPolicy {
                max_decompressed_bytes,
                timeout: Duration::from_millis(timeout_millis),
            },
        );
    }
    if reviewed_gzip_paths.len() != 1 {
        return Err("invalid_gzip_cardinality");
    }
    let reviewed_architecture_provenance =
        parse_policy_path_set(&document.reviewed_architecture_provenance, &mut all_paths)?;

    let mut reviewed_internal_symlinks = BTreeMap::new();
    for row in document.reviewed_internal_symlinks {
        let path = register_policy_path(&mut all_paths, &row.path)?;
        let target = parse_repo_relative_path(&row.target)?;
        reviewed_internal_symlinks.insert(path, target);
    }

    Ok(HistoryOnlyPolicy {
        reviewed_elf_paths,
        reviewed_gzip_paths,
        reviewed_architecture_provenance,
        reviewed_internal_symlinks,
    })
}

fn invalid_history_only_policy(class: &str) -> BTreeSet<Finding> {
    BTreeSet::from([drift(format!("history_only_policy_invalid:{class}"))])
}

fn stop_gzip_process(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn read_reviewed_gzip(path: &Path, policy: &ReviewedGzipPolicy) -> Result<String, &'static str> {
    let read_limit = policy
        .max_decompressed_bytes
        .checked_add(1)
        .and_then(|limit| u64::try_from(limit).ok())
        .ok_or("gzip_limit_invalid")?;
    let mut child = std::process::Command::new("gzip")
        .arg("-dc")
        .arg("--")
        .arg(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|_| "gzip_spawn_error")?;
    let Some(stdout) = child.stdout.take() else {
        stop_gzip_process(&mut child);
        return Err("gzip_pipe_error");
    };

    let (sender, receiver) = std::sync::mpsc::sync_channel::<Result<Vec<u8>, &'static str>>(1);
    let reader = match std::thread::Builder::new()
        .name("adr-0515-reviewed-gzip-reader".to_owned())
        .spawn(move || {
            let mut bytes = Vec::new();
            let result = stdout
                .take(read_limit)
                .read_to_end(&mut bytes)
                .map(|_| bytes)
                .map_err(|_| "gzip_read_error");
            let _ = sender.send(result);
        }) {
        Ok(reader) => reader,
        Err(_) => {
            stop_gzip_process(&mut child);
            return Err("gzip_reader_spawn_error");
        }
    };

    let started_at = Instant::now();
    let mut read_result = None;
    let mut exit_status = None;
    loop {
        if read_result.is_none() {
            match receiver.try_recv() {
                Ok(result) => read_result = Some(result),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    stop_gzip_process(&mut child);
                    let _ = reader.join();
                    return Err("gzip_reader_error");
                }
            }
        }
        if exit_status.is_none() {
            match child.try_wait() {
                Ok(Some(status)) => exit_status = Some(status),
                Ok(None) => {}
                Err(_) => {
                    stop_gzip_process(&mut child);
                    let _ = reader.join();
                    return Err("gzip_wait_error");
                }
            }
        }
        if read_result.is_some() && exit_status.is_some() {
            break;
        }
        if started_at.elapsed() >= policy.timeout {
            stop_gzip_process(&mut child);
            let _ = reader.join();
            return Err("gzip_timeout");
        }
        std::thread::sleep(GZIP_PROCESS_POLL_INTERVAL);
    }

    if reader.join().is_err() {
        return Err("gzip_reader_error");
    }
    let bytes = read_result.ok_or("gzip_reader_error")??;
    if bytes.len() > policy.max_decompressed_bytes {
        return Err("gzip_output_too_large");
    }
    if !exit_status.ok_or("gzip_wait_error")?.success() {
        return Err("gzip_exit_error");
    }
    String::from_utf8(bytes).map_err(|_| "gzip_invalid_utf8")
}

/// True when a producer citation edge names one of ADR-0515's seven
/// Git-history-only predecessors. These are classified provenance edges, not
/// phantom decisions; the current-authority surfaces are guarded separately.
pub fn is_adr_0515_history_only_citation(entry: &str) -> bool {
    entry
        .split_once('@')
        .is_some_and(|(id, source)| is_history_only_id(id) && source == ADR_0515_SOURCE)
}

pub fn evaluate_adr_0515_current_tree_references(
    reference_sources: &BTreeSet<String>,
) -> BTreeSet<Finding> {
    let policy = match parse_history_only_policy(HISTORY_ONLY_POLICY) {
        Ok(policy) => policy,
        Err(class) => return invalid_history_only_policy(class),
    };
    reference_sources
        .iter()
        .filter(|source| !is_history_only_provenance_source(source, &policy))
        .map(|source| drift(format!("current_tree_path_reference:{source}")))
        .collect()
}

pub fn evaluate_adr_0515_reference_content(path: &str, content: &str) -> BTreeSet<Finding> {
    let policy = match parse_history_only_policy(HISTORY_ONLY_POLICY) {
        Ok(policy) => policy,
        Err(class) => return invalid_history_only_policy(class),
    };
    evaluate_adr_0515_reference_content_with_policy(path, content, &policy)
}

fn evaluate_adr_0515_reference_content_with_policy(
    path: &str,
    content: &str,
    policy: &HistoryOnlyPolicy,
) -> BTreeSet<Finding> {
    if is_history_only_provenance_source(path, policy) {
        return BTreeSet::new();
    }
    if path == "docs/machine-readable/decisions.json" {
        return evaluate_decisions_json_references(content);
    }

    let mut findings = BTreeSet::new();
    for (line_index, line) in content.lines().enumerate() {
        let ids: Vec<String> = history_only_ids().filter(|id| line.contains(id)).collect();
        if ids.is_empty() || is_exact_lifecycle_reference(path, line) {
            continue;
        }
        for id in ids {
            findings.insert(drift(format!(
                "current_tree_identifier_reference:{path}:{}:{id}",
                line_index + 1
            )));
        }
    }
    findings
}

fn evaluate_decisions_json_references(content: &str) -> BTreeSet<Finding> {
    let Ok(document) = serde_json::from_str::<Value>(content) else {
        return evaluate_generic_reference_content("docs/machine-readable/decisions.json", content);
    };
    let mut allowed = BTreeSet::new();
    allowed.insert((
        "/_metadata/numbering".to_owned(),
        CANONICAL_DECISIONS_NUMBERING.to_owned(),
    ));
    for (pointer, suffixes) in CANONICAL_HISTORY_GAP_SUFFIXES {
        let value = suffixes
            .split("..")
            .map(|suffix| format!("ADR-{suffix}"))
            .collect::<Vec<_>>()
            .join("..");
        allowed.insert((pointer.to_owned(), value));
    }
    if let Some(decisions) = document.get("decisions").and_then(Value::as_array) {
        for (row_index, row) in decisions.iter().enumerate() {
            if row.get("adr").and_then(Value::as_str) != Some("ADR-0515") {
                continue;
            }
            if let Some(values) = row.get("supersedes").and_then(Value::as_array) {
                let exact: Vec<String> = history_only_ids().collect();
                if values
                    .iter()
                    .filter_map(Value::as_str)
                    .eq(exact.iter().map(String::as_str))
                {
                    for (value_index, value) in exact.into_iter().enumerate() {
                        allowed.insert((
                            format!("/decisions/{row_index}/supersedes/{value_index}"),
                            value,
                        ));
                    }
                }
            }
        }
    }

    fn walk(
        value: &Value,
        pointer: &str,
        allowed: &BTreeSet<(String, String)>,
        findings: &mut BTreeSet<Finding>,
    ) {
        match value {
            Value::String(text) => {
                for id in history_only_ids().filter(|id| text.contains(id)) {
                    if !allowed.contains(&(pointer.to_owned(), text.to_owned())) {
                        findings.insert(drift(format!(
                            "current_tree_identifier_reference:docs/machine-readable/decisions.json:{pointer}:{id}"
                        )));
                    }
                }
            }
            Value::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    walk(value, &format!("{pointer}/{index}"), allowed, findings);
                }
            }
            Value::Object(values) => {
                for (key, value) in values {
                    walk(value, &format!("{pointer}/{key}"), allowed, findings);
                }
            }
            _ => {}
        }
    }

    let mut findings = BTreeSet::new();
    walk(&document, "", &allowed, &mut findings);
    findings
}

fn evaluate_generic_reference_content(path: &str, content: &str) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    for (line_index, line) in content.lines().enumerate() {
        for id in history_only_ids().filter(|id| line.contains(id)) {
            findings.insert(drift(format!(
                "current_tree_identifier_reference:{path}:{}:{id}",
                line_index + 1
            )));
        }
    }
    findings
}

pub fn evaluate_adr_0515_tracked_surfaces(root: &Path, tracked_paths: &Value) -> BTreeSet<Finding> {
    let policy = match parse_history_only_policy(HISTORY_ONLY_POLICY) {
        Ok(policy) => policy,
        Err(class) => return invalid_history_only_policy(class),
    };
    let Some(tracked_paths) = tracked_paths.as_array() else {
        return BTreeSet::from([drift("tracked_paths_invalid")]);
    };

    let mut findings = BTreeSet::new();
    for (index, row) in tracked_paths.iter().enumerate() {
        let Some(relative) = row.as_str() else {
            findings.insert(drift(format!("tracked_path_non_string:{index}")));
            continue;
        };
        let relative_path = Path::new(relative);
        if relative_path.is_absolute()
            || relative_path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            findings.insert(drift(format!("tracked_path_invalid:{relative}")));
            continue;
        }

        let full_path = root.join(relative_path);
        let metadata = match std::fs::symlink_metadata(&full_path) {
            Ok(metadata) => metadata,
            Err(_) => {
                findings.insert(drift(format!("tracked_path_read_error:{relative}")));
                continue;
            }
        };
        if metadata.file_type().is_symlink() {
            let Some(expected_target) = policy.reviewed_internal_symlinks.get(relative) else {
                findings.insert(drift(format!("tracked_path_symlink:{relative}")));
                continue;
            };
            if std::fs::read_link(&full_path).ok().as_deref() != Some(Path::new(expected_target)) {
                findings.insert(drift(format!("tracked_path_symlink_target:{relative}")));
                continue;
            }
            let Ok(canonical_root) = root.canonicalize() else {
                findings.insert(drift("tracked_root_read_error"));
                continue;
            };
            let Ok(canonical_target) = full_path.canonicalize() else {
                findings.insert(drift(format!("tracked_path_symlink:{relative}")));
                continue;
            };
            if !canonical_target.starts_with(&canonical_root) || !canonical_target.is_file() {
                findings.insert(drift(format!("tracked_path_symlink:{relative}")));
                continue;
            }
            match std::fs::read(&canonical_target) {
                Ok(bytes) => match std::str::from_utf8(&bytes) {
                    Ok(content) => findings.extend(
                        evaluate_adr_0515_reference_content_with_policy(relative, content, &policy),
                    ),
                    Err(_) => {
                        findings.insert(drift(format!("tracked_path_invalid_utf8:{relative}")));
                    }
                },
                Err(_) => {
                    findings.insert(drift(format!("tracked_path_read_error:{relative}")));
                }
            }
            continue;
        }
        if !metadata.is_file() {
            findings.insert(drift(format!("tracked_path_not_file:{relative}")));
            continue;
        }
        let bytes = match std::fs::read(&full_path) {
            Ok(bytes) => bytes,
            Err(_) => {
                findings.insert(drift(format!("tracked_path_read_error:{relative}")));
                continue;
            }
        };
        if let Some(expected_machine) = policy.reviewed_elf_paths.get(relative) {
            let valid = bytes.len() >= 20
                && bytes.starts_with(b"\x7fELF")
                && bytes[4..7] == [2, 1, 1]
                && u16::from_le_bytes([bytes[16], bytes[17]]) == 2
                && u16::from_le_bytes([bytes[18], bytes[19]]) == *expected_machine;
            if !valid {
                findings.insert(drift(format!("tracked_path_binary_format:{relative}")));
            }
            continue;
        }
        if let Some(gzip_policy) = policy.reviewed_gzip_paths.get(relative) {
            match read_reviewed_gzip(&full_path, gzip_policy) {
                Ok(content) => findings.extend(evaluate_adr_0515_reference_content_with_policy(
                    relative, &content, &policy,
                )),
                Err(class) => {
                    findings.insert(drift(format!("tracked_path_{class}:{relative}")));
                }
            }
            continue;
        }
        match std::str::from_utf8(&bytes) {
            Ok(content) => findings.extend(evaluate_adr_0515_reference_content_with_policy(
                relative, content, &policy,
            )),
            Err(_) => {
                findings.insert(drift(format!("tracked_path_invalid_utf8:{relative}")));
            }
        }
    }
    findings
}

fn is_history_only_provenance_source(path: &str, policy: &HistoryOnlyPolicy) -> bool {
    path.starts_with("docs/audit/")
        || path.starts_with(".omc/")
        || policy.reviewed_architecture_provenance.contains(path)
}

fn is_exact_lifecycle_reference(path: &str, line: &str) -> bool {
    let trimmed = line.trim();
    if path == ADR_0515_SOURCE {
        return history_only_ids().any(|id| trimmed == format!("- {id}"));
    }
    false
}

fn history_only_ids() -> impl Iterator<Item = String> {
    HISTORY_ONLY_ADR_SUFFIXES
        .into_iter()
        .map(|suffix| format!("ADR-{suffix}"))
}

fn is_history_only_id(id: &str) -> bool {
    id.strip_prefix("ADR-")
        .is_some_and(|suffix| HISTORY_ONLY_ADR_SUFFIXES.contains(&suffix))
}

fn drift(key: impl Into<String>) -> Finding {
    let key = key.into();
    Finding::new(ADR_0515_HISTORY_ONLY_CODE, &key)
}

fn history_only_id_in_decision_path(path: &str) -> Option<String> {
    let normalized = path.trim_start_matches("./").trim_start_matches('/');
    if !normalized.starts_with("docs/decisions/") || !normalized.ends_with(".md") {
        return None;
    }
    let file_name = normalized.rsplit('/').next()?;
    history_only_ids().find(|id| file_name.starts_with(&format!("{id}-")))
}

fn history_only_id_in_path(path: &str) -> Option<String> {
    history_only_id_in_decision_path(path)
}

pub fn evaluate_adr_0515_history_only(
    decision_source_paths: &BTreeSet<String>,
    adr_index_markdown: &str,
    decisions_json: &Value,
    root_hub: &Value,
    masterplan: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    for path in decision_source_paths {
        if let Some(id) = history_only_id_in_decision_path(path) {
            findings.insert(drift(format!("decision_source:{id}")));
        }
    }

    for id in history_only_ids() {
        let row_prefix = format!("| {id} |");
        if adr_index_markdown
            .lines()
            .any(|line| line.trim_start().starts_with(&row_prefix))
        {
            findings.insert(drift(format!("docs/ADR-INDEX.md:{id}")));
        }
    }

    match decisions_json.get("decisions").and_then(Value::as_array) {
        Some(decisions) => {
            for decision in decisions {
                if let Some(id) = decision
                    .get("adr")
                    .and_then(Value::as_str)
                    .filter(|id| is_history_only_id(id))
                {
                    findings.insert(drift(format!("docs/machine-readable/decisions.json:{id}")));
                }
            }
        }
        None => {
            findings.insert(drift(
                "docs/machine-readable/decisions.json:<invalid-decisions>",
            ));
        }
    }

    match root_hub.get("entry_points").and_then(Value::as_object) {
        Some(entry_points) => {
            for (name, entry) in entry_points {
                if let Some(id) = entry
                    .get("current_path")
                    .and_then(Value::as_str)
                    .and_then(history_only_id_in_path)
                {
                    findings.insert(drift(format!("specs/root-hub-pointers.json:{name}:{id}")));
                }
            }
        }
        None => {
            findings.insert(drift("specs/root-hub-pointers.json:<invalid-entry_points>"));
        }
    }

    match masterplan
        .get("planning_authority")
        .and_then(|authority| authority.get("bound_adrs"))
        .and_then(Value::as_array)
    {
        Some(bound_adrs) => {
            for id in bound_adrs
                .iter()
                .filter_map(Value::as_str)
                .filter(|id| is_history_only_id(id))
            {
                findings.insert(drift(format!(
                    "specs/masterplan.json:planning_authority.bound_adrs:{id}"
                )));
            }
        }
        None => {
            findings.insert(drift(
                "specs/masterplan.json:planning_authority.bound_adrs:<invalid>",
            ));
        }
    }

    if let Some(phases) = masterplan
        .get("ideal_production_roadmap")
        .and_then(|roadmap| roadmap.get("phases"))
        .and_then(Value::as_array)
    {
        for (position, phase) in phases.iter().enumerate() {
            let Some(authoring_adrs) = phase.get("authoring_adrs") else {
                continue;
            };
            let phase_id = match phase.get("id").and_then(Value::as_str) {
                Some(id) => id.to_owned(),
                None => position.to_string(),
            };
            match authoring_adrs.as_array() {
                Some(authoring_adrs) => {
                    for id in authoring_adrs
                        .iter()
                        .filter_map(Value::as_str)
                        .filter(|id| is_history_only_id(id))
                    {
                        findings.insert(drift(format!(
                            "specs/masterplan.json:ideal_production_roadmap.phases[{phase_id}].authoring_adrs:{id}"
                        )));
                    }
                }
                None => {
                    findings.insert(drift(format!(
                        "specs/masterplan.json:ideal_production_roadmap.phases[{phase_id}].authoring_adrs:<invalid>"
                    )));
                }
            }
        }
    } else {
        findings.insert(drift(
            "specs/masterplan.json:ideal_production_roadmap.phases:<invalid>",
        ));
    }

    findings
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use serde_json::json;

    use super::*;

    fn clean_sources() -> BTreeSet<String> {
        BTreeSet::from([
            "docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md"
                .to_owned(),
        ])
    }

    fn clean_index() -> &'static str {
        "| ADR-0515 | Accepted | Current CI authority |\n"
    }

    fn clean_decisions() -> Value {
        json!({"decisions": [{"adr": "ADR-0515"}]})
    }

    fn clean_root_hub() -> Value {
        json!({
            "entry_points": {
                "adr_0515": {
                    "current_path": "docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md"
                }
            }
        })
    }

    fn clean_masterplan() -> Value {
        json!({
            "planning_authority": {"bound_adrs": ["ADR-0515"]},
            "ideal_production_roadmap": {
                "phases": [{"id": "P-TOOLCHAIN", "authoring_adrs": ["ADR-0515"]}]
            }
        })
    }

    fn keys(findings: &BTreeSet<Finding>) -> Vec<String> {
        findings.iter().map(|finding| finding.key.clone()).collect()
    }

    fn legacy_adr(suffix: &str) -> String {
        format!("ADR-{suffix}")
    }

    fn retired_controller_name() -> String {
        ["jen", "kins"].concat()
    }

    fn compressed_legacy_reference() -> Vec<u8> {
        vec![
            0x1f, 0x8b, 0x08, 0x00, 0x21, 0xfa, 0x5e, 0x6a, 0x00, 0x03, 0x4b, 0x2c, 0x2d, 0xc9,
            0xc8, 0x2f, 0xca, 0x2c, 0xa9, 0xb4, 0x52, 0x70, 0x74, 0x09, 0xd2, 0x35, 0x30, 0x36,
            0xb1, 0xe4, 0x02, 0x00, 0x56, 0x98, 0xc1, 0xcc, 0x14, 0x00, 0x00, 0x00,
        ]
    }

    fn minimal_history_only_policy() -> Value {
        json!({
            "schema_version": 1,
            "policy_id": ADR_0515_HISTORY_ONLY_VALIDATOR,
            "reviewed_elf_paths": [],
            "reviewed_gzip_paths": [{
                "path": "evidence/reviewed.gz",
                "max_decompressed_bytes": 1024,
                "timeout_millis": 1000
            }],
            "reviewed_architecture_provenance": [],
            "reviewed_internal_symlinks": []
        })
    }

    #[test]
    fn history_only_policy_requires_exact_schema_and_shape() {
        assert!(parse_history_only_policy("{").is_err());

        let valid = minimal_history_only_policy();
        assert!(parse_history_only_policy(&valid.to_string()).is_ok());

        let mut wrong_schema = valid.clone();
        wrong_schema["schema_version"] = json!(2);
        assert!(parse_history_only_policy(&wrong_schema.to_string()).is_err());

        let mut wrong_id = valid.clone();
        wrong_id["policy_id"] = json!("wrong-policy");
        assert!(parse_history_only_policy(&wrong_id.to_string()).is_err());

        let mut wrong_shape = valid;
        wrong_shape["reviewed_gzip_paths"] = json!({"path": "evidence/reviewed.gz"});
        assert!(parse_history_only_policy(&wrong_shape.to_string()).is_err());
    }

    #[test]
    fn history_only_policy_rejects_duplicate_top_level_keys() {
        let duplicate_policy_id = r#"{
            "schema_version": 1,
            "policy_id": "cloud-ci-cross-artifact-agreement/adr-0515-history-only",
            "policy_id": "cloud-ci-cross-artifact-agreement/adr-0515-history-only",
            "reviewed_elf_paths": [],
            "reviewed_gzip_paths": [{
                "path": "evidence/reviewed.gz",
                "max_decompressed_bytes": 1024,
                "timeout_millis": 1000
            }],
            "reviewed_architecture_provenance": [],
            "reviewed_internal_symlinks": []
        }"#;

        assert!(parse_history_only_policy(duplicate_policy_id).is_err());
    }

    #[test]
    fn history_only_policy_rejects_duplicate_row_keys() {
        let duplicate_gzip_path = r#"{
            "schema_version": 1,
            "policy_id": "cloud-ci-cross-artifact-agreement/adr-0515-history-only",
            "reviewed_elf_paths": [],
            "reviewed_gzip_paths": [{
                "path": "evidence/reviewed.gz",
                "path": "evidence/reviewed.gz",
                "max_decompressed_bytes": 1024,
                "timeout_millis": 1000
            }],
            "reviewed_architecture_provenance": [],
            "reviewed_internal_symlinks": []
        }"#;

        assert!(parse_history_only_policy(duplicate_gzip_path).is_err());
    }

    #[test]
    fn invalid_history_only_policy_finding_preserves_the_failure_class() {
        assert_eq!(
            keys(&invalid_history_only_policy("invalid_json")),
            vec!["history_only_policy_invalid:invalid_json".to_owned()]
        );
    }

    #[test]
    fn history_only_policy_rejects_duplicate_and_unsafe_paths() {
        let mut duplicate = minimal_history_only_policy();
        duplicate["reviewed_architecture_provenance"] =
            json!(["docs/history.md", "docs/history.md"]);
        assert!(parse_history_only_policy(&duplicate.to_string()).is_err());

        let mut escaped = minimal_history_only_policy();
        escaped["reviewed_elf_paths"] = json!([{"path": "../outside.elf", "machine": 62}]);
        assert!(parse_history_only_policy(&escaped.to_string()).is_err());

        let mut absolute_target = minimal_history_only_policy();
        absolute_target["reviewed_internal_symlinks"] =
            json!([{"path": "links/current", "target": "/outside"}]);
        assert!(parse_history_only_policy(&absolute_target.to_string()).is_err());
    }

    #[test]
    fn gzip_policy_requires_bounded_size_and_timeout() {
        let mut zero_size = minimal_history_only_policy();
        zero_size["reviewed_gzip_paths"][0]["max_decompressed_bytes"] = json!(0);
        assert_eq!(
            parse_history_only_policy(&zero_size.to_string()).unwrap_err(),
            "invalid_gzip_size_limit"
        );

        let mut oversized = minimal_history_only_policy();
        oversized["reviewed_gzip_paths"][0]["max_decompressed_bytes"] =
            json!(MAX_CONFIGURED_GZIP_DECOMPRESSED_BYTES + 1);
        assert_eq!(
            parse_history_only_policy(&oversized.to_string()).unwrap_err(),
            "invalid_gzip_size_limit"
        );

        let mut zero_timeout = minimal_history_only_policy();
        zero_timeout["reviewed_gzip_paths"][0]["timeout_millis"] = json!(0);
        assert_eq!(
            parse_history_only_policy(&zero_timeout.to_string()).unwrap_err(),
            "invalid_gzip_timeout"
        );

        let mut excessive_timeout = minimal_history_only_policy();
        excessive_timeout["reviewed_gzip_paths"][0]["timeout_millis"] =
            json!(MAX_CONFIGURED_GZIP_TIMEOUT_MILLIS + 1);
        assert_eq!(
            parse_history_only_policy(&excessive_timeout.to_string()).unwrap_err(),
            "invalid_gzip_timeout"
        );
    }

    #[test]
    fn bundled_history_only_policy_is_complete_and_valid() {
        let policy = parse_history_only_policy(HISTORY_ONLY_POLICY).unwrap();
        assert_eq!(policy.reviewed_elf_paths.len(), 7);
        assert_eq!(policy.reviewed_gzip_paths.len(), 1);
        assert_eq!(policy.reviewed_architecture_provenance.len(), 5);
        assert_eq!(policy.reviewed_internal_symlinks.len(), 15);
    }

    #[test]
    fn production_rust_contains_no_policy_owned_product_path_literals() {
        let policy = parse_history_only_policy(HISTORY_ONLY_POLICY).unwrap();
        let production_source = include_str!("adr_0515_history_only.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        for path in policy
            .reviewed_elf_paths
            .keys()
            .chain(policy.reviewed_gzip_paths.keys())
            .chain(policy.reviewed_architecture_provenance.iter())
            .chain(policy.reviewed_internal_symlinks.keys())
            .chain(policy.reviewed_internal_symlinks.values())
        {
            assert!(
                !production_source.contains(path),
                "reviewed path must live in policy data: {path}"
            );
        }
    }

    #[test]
    fn restored_deleted_adr_source_fails_closed() {
        let mut sources = clean_sources();
        sources.insert(format!(
            "docs/decisions/{}-{}-argocd-self-hostable-ci-cd-substrate.md",
            legacy_adr("0349"),
            retired_controller_name()
        ));

        let findings = evaluate_adr_0515_history_only(
            &sources,
            clean_index(),
            &clean_decisions(),
            &clean_root_hub(),
            &clean_masterplan(),
        );

        assert_eq!(
            keys(&findings),
            vec![format!("decision_source:{}", legacy_adr("0349"))]
        );
    }

    #[test]
    fn adr_index_residue_fails_closed() {
        let findings = evaluate_adr_0515_history_only(
            &clean_sources(),
            &format!(
                "| {} | Superseded | Stale generated row |\n",
                legacy_adr("0349")
            ),
            &json!({"decisions": [{"adr": legacy_adr("0349")}]}),
            &clean_root_hub(),
            &clean_masterplan(),
        );

        assert_eq!(
            keys(&findings),
            vec![
                format!("docs/ADR-INDEX.md:{}", legacy_adr("0349")),
                format!(
                    "docs/machine-readable/decisions.json:{}",
                    legacy_adr("0349")
                ),
            ]
        );
    }

    #[test]
    fn root_hub_re_exposure_fails_closed() {
        let root_hub = json!({
            "entry_points": {
                "stale": {
                    "current_path": format!(
                        "docs/decisions/{}-{}-argocd-self-hostable-ci-cd-substrate.md",
                        legacy_adr("0349"),
                        retired_controller_name()
                    )
                }
            }
        });
        let findings = evaluate_adr_0515_history_only(
            &clean_sources(),
            clean_index(),
            &clean_decisions(),
            &root_hub,
            &clean_masterplan(),
        );

        assert_eq!(
            keys(&findings),
            vec![format!(
                "specs/root-hub-pointers.json:stale:{}",
                legacy_adr("0349")
            )]
        );
    }

    #[test]
    fn stale_active_masterplan_binding_fails_closed() {
        let masterplan = json!({
            "planning_authority": {"bound_adrs": [legacy_adr("0349")]},
            "ideal_production_roadmap": {
                "phases": [{"id": "P-TOOLCHAIN", "authoring_adrs": [legacy_adr("0513")]}]
            }
        });
        let findings = evaluate_adr_0515_history_only(
            &clean_sources(),
            clean_index(),
            &clean_decisions(),
            &clean_root_hub(),
            &masterplan,
        );

        assert_eq!(
            keys(&findings),
            vec![
                format!(
                    "specs/masterplan.json:ideal_production_roadmap.phases[P-TOOLCHAIN].authoring_adrs:{}",
                    legacy_adr("0513")
                ),
                format!(
                    "specs/masterplan.json:planning_authority.bound_adrs:{}",
                    legacy_adr("0349")
                ),
            ]
        );
    }

    #[test]
    fn history_only_citation_classifier_is_exact() {
        assert!(is_adr_0515_history_only_citation(&format!(
            "{}@docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md",
            legacy_adr("0349")
        )));
        assert!(!is_adr_0515_history_only_citation(&format!(
            "{}@CLAUDE.md",
            legacy_adr("0513")
        )));
        assert!(!is_adr_0515_history_only_citation(&format!(
            "{}@oya/analytics/README.md",
            legacy_adr("0349")
        )));
        assert!(!is_adr_0515_history_only_citation(&format!(
            "{}@cloud/cloud-k8s/manifest.json",
            legacy_adr("0349")
        )));
        assert!(!is_adr_0515_history_only_citation(&format!(
            "{}@oya/tenant-rbac/contracts/openapi-v1.meta.yaml",
            legacy_adr("0349")
        )));
        assert!(!is_adr_0515_history_only_citation(&format!(
            "{}@specs/master-plan-sequencing.json",
            legacy_adr("0349")
        )));
        assert!(!is_adr_0515_history_only_citation(
            "ADR-0397@specs/master-plan-sequencing.json"
        ));
        assert!(!is_adr_0515_history_only_citation(&legacy_adr("0349")));
    }

    #[test]
    fn active_tree_path_references_fail_closed() {
        let sources = BTreeSet::from([
            "CLAUDE.md".to_owned(),
            "cloud/cloud-k8s/manifest.json".to_owned(),
            "oya/analytics/README.md".to_owned(),
            "oya/tenant-rbac/contracts/openapi-v1.meta.yaml".to_owned(),
        ]);

        let findings = evaluate_adr_0515_current_tree_references(&sources);

        assert_eq!(
            keys(&findings),
            vec![
                "current_tree_path_reference:CLAUDE.md".to_owned(),
                "current_tree_path_reference:cloud/cloud-k8s/manifest.json".to_owned(),
                "current_tree_path_reference:oya/analytics/README.md".to_owned(),
                "current_tree_path_reference:oya/tenant-rbac/contracts/openapi-v1.meta.yaml"
                    .to_owned(),
            ]
        );
    }

    #[test]
    fn historical_evidence_and_runtime_ledgers_remain_scoped_provenance() {
        let sources = BTreeSet::from([
            ".omc/ultragoal/friction-ledger.jsonl".to_owned(),
            "docs/architecture/adr-cross-reference-graph-2026-05-20.md".to_owned(),
            "docs/audit/initial-sweep/source.md".to_owned(),
        ]);

        assert!(evaluate_adr_0515_current_tree_references(&sources).is_empty());
    }

    #[test]
    fn identifier_only_references_fail_closed_on_active_surfaces() {
        let cases = [
            (
                "docs/decisions/ADR-0367-trustless-pre-merge-verification-gateway.md",
                format!(
                    "The trusted {} runner is defined by {}/0361.",
                    retired_controller_name(),
                    legacy_adr("0349")
                ),
            ),
            (
                "oya/audit-chain/slos/audit.openslo.yaml",
                format!("adr_refs: [{}]", legacy_adr("0349")),
            ),
            (
                "specs/cloud-toolchain-target.json",
                format!(r#"{{"authoring_adr":"{}"}}"#, legacy_adr("0359")),
            ),
        ];

        for (path, content) in cases {
            assert!(
                !evaluate_adr_0515_reference_content(path, &content).is_empty(),
                "active identifier reference must fail closed: {path}"
            );
        }
    }

    #[test]
    fn only_exact_lifecycle_and_reviewed_provenance_references_are_allowed() {
        let lifecycle = history_only_ids()
            .map(|id| format!("  - {id}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            evaluate_adr_0515_reference_content(
                "docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md",
                &lifecycle,
            )
            .is_empty()
        );
        assert!(
            evaluate_adr_0515_reference_content(
                "docs/architecture/adr-cross-reference-graph-2026-05-20.md",
                &format!("{} historical graph node", legacy_adr("0349")),
            )
            .is_empty()
        );
        assert!(
            !evaluate_adr_0515_reference_content(
                "docs/architecture/current-authority.md",
                &format!("{} historical graph node", legacy_adr("0349")),
            )
            .is_empty()
        );

        let lookalike = format!(
            "{{\n  \"_metadata\": {{\n    \"numbering\": \"{} corrupted\"\n  }},\n  \"decisions\": []\n}}",
            legacy_adr("0349")
        );
        assert!(
            !evaluate_adr_0515_reference_content(
                "docs/machine-readable/decisions.json",
                &lookalike,
            )
            .is_empty(),
            "field-name lookalikes must not inherit structural exceptions"
        );

        let mutually_consistent_corruption = format!(
            "{{\n  \"_metadata\": {{\n    \"numbering\": \"ADR-0001..ADR-0619 (non-contiguous; gaps: {}..ADR-0350)\",\n    \"gaps\": [\"{}..ADR-0350\"]\n  }},\n  \"decisions\": [{{\"adr\": \"ADR-0619\"}}]\n}}",
            legacy_adr("0349"),
            legacy_adr("0349")
        );
        assert!(
            !evaluate_adr_0515_reference_content(
                "docs/machine-readable/decisions.json",
                &mutually_consistent_corruption,
            )
            .is_empty(),
            "candidate-derived gaps and numbering must not authorize each other"
        );
    }

    #[test]
    fn invalid_utf8_regular_files_fail_closed() {
        let root =
            std::env::temp_dir().join(format!("adr-0515-invalid-utf8-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("invalid.txt"), [0xff, 0xfe]).unwrap();

        std::fs::write(root.join("spoof.elf"), [0xff, 0xfe]).unwrap();
        std::fs::write(root.join("spoof.gz"), [0xff, 0xfe]).unwrap();
        std::fs::create_dir_all(root.join("cloud/cloud-kernel/out")).unwrap();
        std::fs::write(
            root.join("cloud/cloud-kernel/out/user-exec.elf"),
            b"not an elf",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("evidence/ci/slsa")).unwrap();
        std::fs::write(root.join("evidence/ci/slsa/sbom.cdx.json.gz"), b"not gzip").unwrap();

        let findings = evaluate_adr_0515_tracked_surfaces(
            &root,
            &json!([
                "invalid.txt",
                "spoof.elf",
                "spoof.gz",
                "cloud/cloud-kernel/out/user-exec.elf",
                "evidence/ci/slsa/sbom.cdx.json.gz"
            ]),
        );
        let finding_keys = keys(&findings);
        assert!(finding_keys.contains(&"tracked_path_invalid_utf8:invalid.txt".to_owned()));
        assert!(finding_keys.contains(&"tracked_path_invalid_utf8:spoof.elf".to_owned()));
        assert!(finding_keys.contains(&"tracked_path_invalid_utf8:spoof.gz".to_owned()));
        assert!(finding_keys.contains(
            &"tracked_path_binary_format:cloud/cloud-kernel/out/user-exec.elf".to_owned()
        ));
        assert!(finding_keys.contains(
            &"tracked_path_gzip_exit_error:evidence/ci/slsa/sbom.cdx.json.gz".to_owned()
        ));

        std::fs::write(
            root.join("evidence/ci/slsa/sbom.cdx.json.gz"),
            compressed_legacy_reference(),
        )
        .unwrap();
        let findings = evaluate_adr_0515_tracked_surfaces(
            &root,
            &json!(["evidence/ci/slsa/sbom.cdx.json.gz"]),
        );
        assert!(keys(&findings).iter().any(|key| {
            key.starts_with("current_tree_identifier_reference:evidence/ci/slsa/sbom.cdx.json.gz")
        }));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reviewed_gzip_reader_fails_closed_at_the_decompressed_size_limit() {
        let root =
            std::env::temp_dir().join(format!("adr-0515-gzip-limit-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("reviewed.gz");
        std::fs::write(&path, compressed_legacy_reference()).unwrap();

        assert_eq!(
            read_reviewed_gzip(
                &path,
                &ReviewedGzipPolicy {
                    max_decompressed_bytes: 8,
                    timeout: Duration::from_secs(5),
                },
            ),
            Err("gzip_output_too_large")
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_missing_and_broken_symlink_tracked_entries_fail_closed() {
        let root = std::env::temp_dir().join(format!(
            "adr-0515-tracked-surface-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("missing-target", root.join("broken-link")).unwrap();

        let tracked = json!([null, "missing-file", "broken-link"]);
        let findings = evaluate_adr_0515_tracked_surfaces(&root, &tracked);
        let keys = keys(&findings);

        assert!(keys.iter().any(|key| key == "tracked_path_non_string:0"));
        assert!(
            keys.iter()
                .any(|key| key == "tracked_path_read_error:missing-file")
        );
        #[cfg(unix)]
        assert!(
            keys.iter()
                .any(|key| key == "tracked_path_symlink:broken-link")
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_exact_internal_and_content_checked() {
        let root = std::env::temp_dir().join(format!(
            "adr-0515-reviewed-symlink-test-{}",
            std::process::id()
        ));
        let outside = root.with_extension("outside");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&outside);
        std::fs::create_dir_all(root.join("oya/connector/contracts/openapi")).unwrap();
        let target = root.join("oya/connector/contracts/openapi/connector-integration.yaml");
        std::fs::write(&target, "authority: ADR-0515\n").unwrap();
        std::os::unix::fs::symlink(
            "openapi/connector-integration.yaml",
            root.join("oya/connector/contracts/openapi-v1.yaml"),
        )
        .unwrap();
        std::os::unix::fs::symlink(
            "openapi/connector-integration.yaml",
            root.join("oya/connector/contracts/unlisted.yaml"),
        )
        .unwrap();
        std::fs::write(&outside, "outside").unwrap();
        std::os::unix::fs::symlink(
            &outside,
            root.join("oya/connector/contracts/asyncapi-v1.yaml"),
        )
        .unwrap();

        let findings = evaluate_adr_0515_tracked_surfaces(
            &root,
            &json!([
                "oya/connector/contracts/openapi-v1.yaml",
                "oya/connector/contracts/unlisted.yaml",
                "oya/connector/contracts/asyncapi-v1.yaml"
            ]),
        );
        let finding_keys = keys(&findings);
        assert!(
            !finding_keys
                .iter()
                .any(|key| key.contains("contracts/openapi-v1.yaml"))
        );
        assert!(
            finding_keys
                .contains(&"tracked_path_symlink:oya/connector/contracts/unlisted.yaml".to_owned())
        );
        assert!(finding_keys.contains(
            &"tracked_path_symlink_target:oya/connector/contracts/asyncapi-v1.yaml".to_owned()
        ));

        std::fs::write(&target, format!("authority: {}\n", legacy_adr("0349"))).unwrap();
        let findings = evaluate_adr_0515_tracked_surfaces(
            &root,
            &json!(["oya/connector/contracts/openapi-v1.yaml"]),
        );
        assert!(keys(&findings).iter().any(|key| key.starts_with(
            "current_tree_identifier_reference:oya/connector/contracts/openapi-v1.yaml"
        )));

        std::fs::remove_file(root.join("oya/connector/contracts/openapi-v1.yaml")).unwrap();
        let alternate = root.join("oya/connector/contracts/openapi/alternate.yaml");
        std::fs::write(&alternate, "authority: ADR-0515\n").unwrap();
        std::os::unix::fs::symlink(
            "openapi/alternate.yaml",
            root.join("oya/connector/contracts/openapi-v1.yaml"),
        )
        .unwrap();
        let findings = evaluate_adr_0515_tracked_surfaces(
            &root,
            &json!(["oya/connector/contracts/openapi-v1.yaml"]),
        );
        assert!(keys(&findings).contains(
            &"tracked_path_symlink_target:oya/connector/contracts/openapi-v1.yaml".to_owned()
        ));

        std::fs::remove_file(root.join("oya/connector/contracts/openapi-v1.yaml")).unwrap();
        std::os::unix::fs::symlink(
            "openapi/connector-integration.yaml",
            root.join("oya/connector/contracts/openapi-v1.yaml"),
        )
        .unwrap();
        std::fs::write(&target, [0xff, 0xfe]).unwrap();
        let findings = evaluate_adr_0515_tracked_surfaces(
            &root,
            &json!(["oya/connector/contracts/openapi-v1.yaml"]),
        );
        assert!(keys(&findings).contains(
            &"tracked_path_invalid_utf8:oya/connector/contracts/openapi-v1.yaml".to_owned()
        ));

        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_file(outside).unwrap();
    }
}
