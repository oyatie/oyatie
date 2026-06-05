//! Quality-lane registry authority guard.
//!
//! The quality-lane registry is a shared surface, so it must not carry active
//! local CLI, raw Cargo, npm/pnpm, or script authority. Active lanes point at
//! Buck2/Prow-owned checks; retired or not-yet-reprojected lanes stay planned
//! until a native target exists.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const REGISTRY_PATH: &str = "registry/quality/lanes.yaml";
const DOC_PATH: &str = "docs/standards/ci-lanes.md";
const POLICY_KEY: &str = "command_authority_policy:";
const POLICY_NEEDLE: &str = "Buck2/Prow";

const ALLOWED_ACTIVE_COMMAND_PREFIXES: &[&str] = &["buck2 build ", "buck2 test "];
const FORBIDDEN_ACTIVE_COMMAND_FRAGMENTS: &[&str] = &[
    "oya-dev-cli",
    "oya gate",
    "oya verify",
    "cargo ",
    "pnpm",
    "npm ",
    "node 20",
    "bacon",
    "machete",
    ".sh",
    ".py",
    "jenkins",
    "forgejo",
    "argocd",
    "argo cd",
];
const FORBIDDEN_ACTIVE_PROSE_FRAGMENTS: &[&str] = &[
    "invoked from `oya gate run-all`",
    "invoked from oya gate run-all",
    "oya verify command",
    "ts workspace",
    "pnpm",
    "node 20",
    "jenkins ci",
    "forgejo",
    "argocd",
    "argo cd",
    "rendered helm manifest",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityLaneAuthorityReport {
    pub registry_lanes: usize,
    pub doc_rows: usize,
    pub active_lanes: usize,
    pub planned_lanes: usize,
    pub active_buck2_commands: usize,
    pub failures: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Lane {
    id: String,
    stage: String,
    status: String,
    purpose: String,
    source: String,
    check_command: Option<String>,
    line: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DocRow {
    id: String,
    stage: String,
    purpose: String,
    line: usize,
}

pub fn evaluate_contents(registry: &str, doc: &str) -> QualityLaneAuthorityReport {
    let registry_lanes = parse_registry_lanes(registry);
    let doc_rows = parse_doc_rows(doc);
    let mut failures = Vec::new();

    if !registry.contains(POLICY_KEY) || !registry.contains(POLICY_NEEDLE) {
        failures.push(format!(
            "{REGISTRY_PATH}: missing {POLICY_KEY} policy containing {POLICY_NEEDLE}"
        ));
    }

    let registry_by_id = unique_registry_map(&registry_lanes, &mut failures);
    let doc_by_id = unique_doc_map(&doc_rows, &mut failures);

    for lane in &registry_lanes {
        validate_lane_shape(lane, &mut failures);
        validate_lane_command_authority(lane, &mut failures);
        validate_lane_prose_authority(lane, &mut failures);

        match doc_by_id.get(lane.id.as_str()) {
            Some(row) => {
                if row.stage != lane.stage {
                    failures.push(format!(
                        "{DOC_PATH}:{}: lane {} stage drift: expected {}, got {}",
                        row.line, lane.id, lane.stage, row.stage
                    ));
                }
                if normalize_purpose(&row.purpose) != normalize_purpose(&lane.purpose) {
                    failures.push(format!(
                        "{DOC_PATH}:{}: lane {} purpose drift: expected {:?}, got {:?}",
                        row.line,
                        lane.id,
                        normalize_purpose(&lane.purpose),
                        normalize_purpose(&row.purpose)
                    ));
                }
            }
            None => failures.push(format!(
                "{REGISTRY_PATH}:{}: lane {} missing docs/standards/ci-lanes.md mirror",
                lane.line, lane.id
            )),
        }
    }

    for row in &doc_rows {
        if !registry_by_id.contains_key(row.id.as_str()) {
            failures.push(format!(
                "{DOC_PATH}:{}: lane {} is not present in {REGISTRY_PATH}",
                row.line, row.id
            ));
        }
    }

    let active_lanes = registry_lanes
        .iter()
        .filter(|lane| lane.status == "active")
        .count();
    let planned_lanes = registry_lanes
        .iter()
        .filter(|lane| lane.status == "planned")
        .count();
    let active_buck2_commands = registry_lanes
        .iter()
        .filter(|lane| {
            lane.status == "active"
                && lane
                    .check_command
                    .as_deref()
                    .is_some_and(is_allowed_active_command)
        })
        .count();

    QualityLaneAuthorityReport {
        registry_lanes: registry_lanes.len(),
        doc_rows: doc_rows.len(),
        active_lanes,
        planned_lanes,
        active_buck2_commands,
        failures,
    }
}

fn unique_registry_map<'a>(
    lanes: &'a [Lane],
    failures: &mut Vec<String>,
) -> BTreeMap<&'a str, &'a Lane> {
    let mut seen = BTreeMap::new();
    let mut duplicate_ids = BTreeSet::new();
    for lane in lanes {
        if seen.insert(lane.id.as_str(), lane).is_some() {
            duplicate_ids.insert(lane.id.clone());
        }
    }
    for id in duplicate_ids {
        failures.push(format!("{REGISTRY_PATH}: duplicate lane id {id}"));
    }
    seen
}

fn unique_doc_map<'a>(
    rows: &'a [DocRow],
    failures: &mut Vec<String>,
) -> BTreeMap<&'a str, &'a DocRow> {
    let mut seen = BTreeMap::new();
    let mut duplicate_ids = BTreeSet::new();
    for row in rows {
        if seen.insert(row.id.as_str(), row).is_some() {
            duplicate_ids.insert(row.id.clone());
        }
    }
    for id in duplicate_ids {
        failures.push(format!("{DOC_PATH}: duplicate lane row {id}"));
    }
    seen
}

fn validate_lane_shape(lane: &Lane, failures: &mut Vec<String>) {
    if lane.id.is_empty()
        || !lane.id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        failures.push(format!(
            "{REGISTRY_PATH}:{}: invalid lane id {:?}",
            lane.line, lane.id
        ));
    }
    if !matches!(
        lane.stage.as_str(),
        "foundation" | "per-pr" | "nightly" | "per-release"
    ) {
        failures.push(format!(
            "{REGISTRY_PATH}:{}: lane {} has invalid stage {:?}",
            lane.line, lane.id, lane.stage
        ));
    }
    if !matches!(lane.status.as_str(), "active" | "planned") {
        failures.push(format!(
            "{REGISTRY_PATH}:{}: lane {} has invalid status {:?}",
            lane.line, lane.id, lane.status
        ));
    }
    for (field, value) in [
        ("purpose", lane.purpose.as_str()),
        ("source", lane.source.as_str()),
    ] {
        if value.trim().is_empty() {
            failures.push(format!(
                "{REGISTRY_PATH}:{}: lane {} missing {field}",
                lane.line, lane.id
            ));
        }
    }
}

fn validate_lane_command_authority(lane: &Lane, failures: &mut Vec<String>) {
    match lane.status.as_str() {
        "active" => {
            let Some(command) = lane
                .check_command
                .as_deref()
                .map(str::trim)
                .filter(|command| !command.is_empty())
            else {
                failures.push(format!(
                    "{REGISTRY_PATH}:{}: active lane {} missing check_command",
                    lane.line, lane.id
                ));
                return;
            };
            if !is_allowed_active_command(command) {
                failures.push(format!(
                    "{REGISTRY_PATH}:{}: active lane {} must use Buck2/Prow check authority, got {:?}",
                    lane.line, lane.id, command
                ));
            }
            let lower = command.to_ascii_lowercase();
            for forbidden in FORBIDDEN_ACTIVE_COMMAND_FRAGMENTS {
                if lower.contains(forbidden) {
                    failures.push(format!(
                        "{REGISTRY_PATH}:{}: active lane {} command contains retired fragment {:?}",
                        lane.line, lane.id, forbidden
                    ));
                }
            }
        }
        "planned" => {
            if lane.check_command.is_some() {
                failures.push(format!(
                    "{REGISTRY_PATH}:{}: planned lane {} must not carry check_command",
                    lane.line, lane.id
                ));
            }
        }
        _ => {}
    }
}

fn validate_lane_prose_authority(lane: &Lane, failures: &mut Vec<String>) {
    if lane.status != "active" {
        return;
    }
    let prose = format!("{} {}", lane.purpose, lane.source).to_ascii_lowercase();
    for forbidden in FORBIDDEN_ACTIVE_PROSE_FRAGMENTS {
        if prose.contains(forbidden) {
            failures.push(format!(
                "{REGISTRY_PATH}:{}: active lane {} prose contains retired fragment {:?}",
                lane.line, lane.id, forbidden
            ));
        }
    }
}

fn is_allowed_active_command(command: &str) -> bool {
    ALLOWED_ACTIVE_COMMAND_PREFIXES
        .iter()
        .any(|prefix| command.starts_with(prefix))
}

fn parse_registry_lanes(contents: &str) -> Vec<Lane> {
    let mut lanes = Vec::new();
    let mut current: Option<Lane> = None;

    for (idx, line) in contents.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim();
        if let Some(raw_id) = trimmed.strip_prefix("- id: ") {
            if let Some(lane) = current.take() {
                lanes.push(lane);
            }
            current = Some(Lane {
                id: clean_scalar(raw_id),
                line: line_number,
                ..Lane::default()
            });
            continue;
        }

        let Some(lane) = current.as_mut() else {
            continue;
        };
        if let Some(value) = trimmed.strip_prefix("stage: ") {
            lane.stage = clean_scalar(value);
        } else if let Some(value) = trimmed.strip_prefix("status: ") {
            lane.status = clean_scalar(value);
        } else if let Some(value) = trimmed.strip_prefix("purpose: ") {
            lane.purpose = clean_scalar(value);
        } else if let Some(value) = trimmed.strip_prefix("source: ") {
            lane.source = clean_scalar(value);
        } else if let Some(value) = trimmed.strip_prefix("check_command: ") {
            lane.check_command = Some(clean_scalar(value));
        }
    }

    if let Some(lane) = current {
        lanes.push(lane);
    }
    lanes
}

fn parse_doc_rows(contents: &str) -> Vec<DocRow> {
    let mut rows = Vec::new();
    let mut current_stage = "";

    for (idx, line) in contents.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim();
        if trimmed.starts_with("### 1.1 ") {
            current_stage = "foundation";
            continue;
        }
        if trimmed.starts_with("### 1.2 ") {
            current_stage = "per-pr";
            continue;
        }
        if trimmed.starts_with("### 1.3 ") {
            current_stage = "nightly";
            continue;
        }
        if trimmed.starts_with("### 1.4 ") {
            current_stage = "per-release";
            continue;
        }
        if !trimmed.starts_with("| `") {
            continue;
        }
        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() < 2 {
            continue;
        }
        let Some(id) = extract_backtick_value(cells[0]) else {
            continue;
        };
        rows.push(DocRow {
            id,
            stage: current_stage.into(),
            purpose: cells[1].into(),
            line: line_number,
        });
    }

    rows
}

fn clean_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn extract_backtick_value(value: &str) -> Option<String> {
    let start = value.find('`')?;
    let tail = &value[start + 1..];
    let end = tail.find('`')?;
    Some(tail[..end].to_string())
}

fn normalize_purpose(value: &str) -> String {
    clean_scalar(value)
        .replace('`', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn repo_root() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap_or_else(|error| {
        eprintln!("failed to read {rel}: {error}");
        process::exit(2);
    })
}

fn print_json(report: &QualityLaneAuthorityReport) {
    println!("{{");
    println!(
        "  \"status\": \"{}\",",
        if report.failures.is_empty() {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!("  \"registry_lanes\": {},", report.registry_lanes);
    println!("  \"doc_rows\": {},", report.doc_rows);
    println!("  \"active_lanes\": {},", report.active_lanes);
    println!("  \"planned_lanes\": {},", report.planned_lanes);
    println!(
        "  \"active_buck2_commands\": {},",
        report.active_buck2_commands
    );
    println!("  \"failures\": [");
    for (idx, failure) in report.failures.iter().enumerate() {
        let comma = if idx + 1 == report.failures.len() {
            ""
        } else {
            ","
        };
        println!("    \"{}\"{}", json_escape(failure), comma);
    }
    println!("  ]");
    println!("}}");
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn main() {
    let root = repo_root();
    let registry = read(&root, REGISTRY_PATH);
    let doc = read(&root, DOC_PATH);
    let report = evaluate_contents(&registry, &doc);
    print_json(&report);
    if !report.failures.is_empty() {
        process::exit(1);
    }
}
