//! `oya gate validate active-artifact-contract` — vertical enforcement loop runtime
//! for the v3.0.0 active machine-readable artifact contract per ADR-0069.
//!
//! Closes architect r17/r18/r19 + full-consensus-planner-v3 §5 hops 1-2+5-6:
//! - Reads the artifact-capabilities registry
//! - Resolves HEAD-tracked paths via `git ls-files`
//! - Delegates R01-R07 validation to `oya-check-active-artifact-contract`
//! - Optionally emits evidence bundle with `validation_duration_ms`
//! - Optionally emits ONE graph edge artifact per consensus-v3 amendment #4

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use oya_check_active_artifact_contract::{
    ArtifactRow, CapabilityDeclaration, CapabilityKind, CapabilityStatus, Severity,
    ValidationReport, validate,
};
use serde_json::{Value, json};

use crate::usage;

const PROFILE_DEFAULTS_PATH: &str = "specs/artifact-profile-defaults.json";

type CapabilityMap = BTreeMap<CapabilityKind, CapabilityDeclaration>;
type ProfileDefaults = BTreeMap<String, CapabilityMap>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveArtifactContractValidateArgs {
    registry_path: PathBuf,                 // data_class: INTERNAL_ONLY
    emit_evidence_path: Option<PathBuf>,    // data_class: INTERNAL_ONLY
    emit_graph_edges_path: Option<PathBuf>, // data_class: INTERNAL_ONLY
}

pub(crate) fn parse_active_artifact_contract_validate_args(
    args: Vec<String>,
) -> Result<ActiveArtifactContractValidateArgs, String> {
    let mut parsed = ActiveArtifactContractValidateArgs {
        registry_path: PathBuf::from("registry/artifact-capabilities-registry.json"),
        emit_evidence_path: None,
        emit_graph_edges_path: None,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--registry" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.registry_path = PathBuf::from(path);
            }
            "--emit-evidence" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.emit_evidence_path = Some(PathBuf::from(path));
            }
            "--emit-graph-edges" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.emit_graph_edges_path = Some(PathBuf::from(path));
            }
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveArtifactContractReport {
    pub rows_seen: usize,                             // data_class: INTERNAL_ONLY
    pub head_tracked_count: usize,                    // data_class: INTERNAL_ONLY
    pub untracked_paths: Vec<String>,                 // data_class: INTERNAL_ONLY
    pub duplicate_ids: Vec<String>,                   // data_class: INTERNAL_ONLY
    pub validation_duration_ms: u64,                  // data_class: INTERNAL_ONLY
    pub graph_edges: Vec<(String, String, String)>,   // data_class: INTERNAL_ONLY
    pub violations: Vec<ActiveArtifactContractIssue>, // data_class: INTERNAL_ONLY
}

impl ActiveArtifactContractReport {
    pub(crate) fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|violation| violation.severity == SeverityLabel::Error)
            .count()
    }

    pub(crate) fn warning_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|violation| violation.severity == SeverityLabel::Warn)
            .count()
    }

    pub(crate) fn warning_summary(&self) -> String {
        format_issues(
            self.violations
                .iter()
                .filter(|violation| violation.severity == SeverityLabel::Warn),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SeverityLabel {
    Error,
    Warn,
    Info,
}

impl SeverityLabel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveArtifactContractIssue {
    pub artifact_id: String,     // data_class: INTERNAL_ONLY
    pub rule_id: &'static str,   // data_class: INTERNAL_ONLY
    pub severity: SeverityLabel, // data_class: INTERNAL_ONLY
    pub message: String,         // data_class: INTERNAL_ONLY
}

pub(crate) fn validate_active_artifact_contract_gate(
    args: ActiveArtifactContractValidateArgs,
) -> Result<ActiveArtifactContractReport, String> {
    let start = Instant::now();

    let registry_text = fs::read_to_string(&args.registry_path).map_err(|error| {
        format!(
            "active-artifact-contract registry unreadable {}: {error}",
            args.registry_path.display()
        )
    })?;

    let registry_json: Value = serde_json::from_str(&registry_text).map_err(|error| {
        format!(
            "active-artifact-contract registry invalid JSON {}: {error}",
            args.registry_path.display()
        )
    })?;
    let rows_json = registry_json
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "active-artifact-contract registry missing top-level `rows` array in {}",
                args.registry_path.display()
            )
        })?;

    let mut rows = Vec::new();
    let mut paths_seen: BTreeMap<String, String> = BTreeMap::new(); // normalized git path → registry artifact_path
    let mut duplicate_ids: Vec<String> = Vec::new();
    let mut artifact_ids: BTreeMap<String, usize> = BTreeMap::new();
    let mut graph_edges: Vec<(String, String, String)> = Vec::new();
    let profile_defaults = if rows_json
        .iter()
        .any(|row| row.get("capabilities").is_none())
    {
        let defaults_path = profile_defaults_path();
        Some(load_artifact_profile_defaults(&defaults_path)?)
    } else {
        None
    };

    for (index, row_json) in rows_json.iter().enumerate() {
        let row = parse_artifact_row(
            row_json,
            index,
            &args.registry_path,
            profile_defaults.as_ref(),
        )?;
        let artifact_id = row.artifact_id.clone();
        let artifact_path = row.artifact_path.clone();
        let artifact_profile =
            string_field(row_json, "artifact_profile").unwrap_or_else(|| "unspecified".to_string());
        if artifact_ids.insert(artifact_id.clone(), index).is_some() {
            duplicate_ids.push(artifact_id.clone());
        }
        paths_seen.insert(artifact_path.clone(), artifact_path.clone());

        // Emit one graph edge per row: artifact_id --declares--> artifact_profile
        graph_edges.push((
            artifact_id.clone(),
            artifact_profile.clone(),
            "declares".to_string(),
        ));
        rows.push(row);
    }

    // Resolve HEAD-tracked paths via `git ls-files`.
    let head_tracked = git_ls_files()?;
    let mut contract_head_tracked = head_tracked.clone();

    let mut untracked_paths: Vec<String> = Vec::new();
    for (path, registry_path) in &paths_seen {
        if is_head_tracked_artifact_path(path, &head_tracked) {
            contract_head_tracked.insert(path.clone());
        } else {
            untracked_paths.push(registry_path.clone());
        }
    }

    let validation = validate(&rows, &contract_head_tracked);
    let violations = active_artifact_contract_issues(&validation);
    let validation_duration_ms = start.elapsed().as_millis() as u64;

    let report = ActiveArtifactContractReport {
        rows_seen: rows.len(),
        head_tracked_count: head_tracked.len(),
        untracked_paths: untracked_paths.clone(),
        duplicate_ids: duplicate_ids.clone(),
        validation_duration_ms,
        graph_edges: graph_edges.clone(),
        violations: violations.clone(),
    };

    // Optional evidence emission per consensus-v3 amendment #10 + step 5.
    if let Some(evidence_path) = &args.emit_evidence_path {
        write_evidence_bundle(evidence_path, &report, &args.registry_path)?;
    }

    // Optional graph-edge emission per consensus-v3 amendment #4 + step 6.
    if let Some(edges_path) = &args.emit_graph_edges_path {
        write_graph_edges(edges_path, &graph_edges)?;
    }

    if validation.has_errors() {
        return Err(format_issues(
            violations
                .iter()
                .filter(|violation| violation.severity == SeverityLabel::Error),
        ));
    }

    Ok(report)
}

fn parse_artifact_row(
    row: &Value,
    index: usize,
    registry_path: &Path,
    profile_defaults: Option<&ProfileDefaults>,
) -> Result<ArtifactRow, String> {
    let artifact_id = string_field(row, "artifact_id").ok_or_else(|| {
        format!(
            "row index {index} missing artifact_id in {}",
            registry_path.display()
        )
    })?;
    let artifact_path = string_field(row, "artifact_path").ok_or_else(|| {
        format!(
            "row artifact_id={artifact_id} missing artifact_path in {}",
            registry_path.display()
        )
    })?;
    let artifact_profile = string_field(row, "artifact_profile");
    let capabilities = parse_capabilities(row, artifact_profile.as_deref(), profile_defaults)
        .map_err(|error| format!("row artifact_id={artifact_id}: {error}"))?;

    Ok(ArtifactRow {
        artifact_id,
        artifact_path: normalize_repo_root_artifact_path(&artifact_path),
        artifact_format: string_field(row, "artifact_format").unwrap_or_else(|| "json".into()),
        contract_version: string_field(row, "contract_version").unwrap_or_else(|| "v3.0.0".into()),
        capabilities,
    })
}

fn parse_capabilities(
    row: &Value,
    artifact_profile: Option<&str>,
    profile_defaults: Option<&ProfileDefaults>,
) -> Result<CapabilityMap, String> {
    let mut capabilities = if let Some(capabilities) = row.get("capabilities") {
        parse_declared_capabilities(capabilities)?
    } else {
        let profile = artifact_profile
            .filter(|profile| !profile.trim().is_empty())
            .ok_or_else(|| "row missing capabilities and artifact_profile".to_string())?;
        profile_defaults
            .and_then(|defaults| defaults.get(profile))
            .cloned()
            .ok_or_else(|| {
                format!(
                    "unknown artifact_profile `{profile}`; add it to {PROFILE_DEFAULTS_PATH} or declare full capabilities"
                )
            })?
    };

    if let Some(overrides) = row.get("capability_overrides").and_then(Value::as_object) {
        for (name, override_value) in overrides {
            let kind = parse_capability_kind(name)
                .ok_or_else(|| format!("unknown capability override `{name}`"))?;
            let entry = capabilities.get_mut(&kind).ok_or_else(|| {
                format!("capability override `{name}` has no base capability declaration")
            })?;
            apply_capability_override(entry, override_value)?;
        }
    }

    Ok(capabilities)
}

fn load_artifact_profile_defaults(path: &Path) -> Result<ProfileDefaults, String> {
    let defaults_text = fs::read_to_string(path).map_err(|error| {
        format!(
            "active-artifact-contract profile defaults unreadable {}: {error}",
            path.display()
        )
    })?;
    let defaults_json: Value = serde_json::from_str(&defaults_text).map_err(|error| {
        format!(
            "active-artifact-contract profile defaults invalid JSON {}: {error}",
            path.display()
        )
    })?;
    let profiles = defaults_json
        .get("profiles")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "active-artifact-contract profile defaults missing `profiles` object in {}",
                path.display()
            )
        })?;

    let mut defaults = ProfileDefaults::new();
    for (profile_name, profile) in profiles {
        let capabilities = profile
            .get("default_capabilities")
            .ok_or_else(|| {
                format!(
                    "profile `{profile_name}` missing default_capabilities in {}",
                    path.display()
                )
            })
            .and_then(parse_declared_capabilities)
            .map_err(|error| format!("profile `{profile_name}` invalid: {error}"))?;
        validate_profile_default_capabilities(profile_name, &capabilities)?;
        defaults.insert(profile_name.clone(), capabilities);
    }
    Ok(defaults)
}

fn validate_profile_default_capabilities(
    profile_name: &str,
    capabilities: &CapabilityMap,
) -> Result<(), String> {
    for kind in CapabilityKind::ALL {
        let declaration = capabilities.get(&kind).ok_or_else(|| {
            format!(
                "profile `{profile_name}` missing default capability `{}`",
                kind.name()
            )
        })?;
        match declaration.status {
            CapabilityStatus::Operational => {
                if declaration
                    .evidence_ref
                    .as_ref()
                    .map(|evidence_ref| evidence_ref.trim().is_empty())
                    .unwrap_or(true)
                {
                    return Err(format!(
                        "profile `{profile_name}` capability `{}` is operational without evidence_ref",
                        kind.name()
                    ));
                }
            }
            CapabilityStatus::Planned | CapabilityStatus::BlockedByFoundation => {
                if declaration.prerequisite_for_operational.is_empty() {
                    return Err(format!(
                        "profile `{profile_name}` capability `{}` is {:?} without prerequisite_for_operational",
                        kind.name(),
                        declaration.status
                    ));
                }
            }
            CapabilityStatus::NotApplicable => {
                if declaration
                    .not_applicable_rationale
                    .as_ref()
                    .map(|rationale| rationale.trim().is_empty())
                    .unwrap_or(true)
                {
                    return Err(format!(
                        "profile `{profile_name}` capability `{}` is not-applicable without not_applicable_rationale",
                        kind.name()
                    ));
                }
            }
        }
    }
    Ok(())
}

fn profile_defaults_path() -> PathBuf {
    let relative = PathBuf::from(PROFILE_DEFAULTS_PATH);
    if relative.exists() {
        return relative;
    }
    PathBuf::from(option_env!("CARGO_MANIFEST_DIR").unwrap_or("."))
        .ancestors()
        .nth(4)
        .unwrap_or_else(|| Path::new("."))
        .join(PROFILE_DEFAULTS_PATH)
}

fn parse_declared_capabilities(value: &Value) -> Result<CapabilityMap, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "`capabilities` must be an object".to_string())?;
    let mut capabilities = BTreeMap::new();
    for (name, declaration) in object {
        let kind = parse_capability_kind(name)
            .ok_or_else(|| format!("unknown capability declaration `{name}`"))?;
        capabilities.insert(kind, parse_capability_declaration(name, declaration)?);
    }
    Ok(capabilities)
}

fn parse_capability_declaration(
    name: &str,
    value: &Value,
) -> Result<CapabilityDeclaration, String> {
    let status = string_field(value, "status")
        .ok_or_else(|| format!("capability `{name}` missing status"))
        .and_then(|status| parse_capability_status(&status))?;
    Ok(CapabilityDeclaration {
        status,
        evidence_ref: string_field(value, "evidence_ref"),
        prerequisite_for_operational: string_array_field(value, "prerequisite_for_operational")
            .unwrap_or_default(),
        not_applicable_rationale: string_field(value, "not_applicable_rationale"),
    })
}

fn apply_capability_override(
    declaration: &mut CapabilityDeclaration,
    value: &Value,
) -> Result<(), String> {
    let replace_mode = bool_field(value, "replace_mode").unwrap_or(false);
    if let Some(status) = string_field(value, "status") {
        declaration.status = parse_capability_status(&status)?;
    }
    if let Some(evidence_ref) = string_field(value, "evidence_ref") {
        declaration.evidence_ref = Some(evidence_ref);
    }
    if let Some(prerequisites) = string_array_field(value, "prerequisite_for_operational") {
        if replace_mode {
            declaration.prerequisite_for_operational = prerequisites;
        } else {
            declaration
                .prerequisite_for_operational
                .extend(prerequisites);
        }
    }
    if let Some(rationale) = string_field(value, "not_applicable_rationale") {
        declaration.not_applicable_rationale = Some(rationale);
    }
    Ok(())
}

fn parse_capability_kind(name: &str) -> Option<CapabilityKind> {
    CapabilityKind::ALL
        .into_iter()
        .find(|kind| kind.name() == name)
}

fn parse_capability_status(value: &str) -> Result<CapabilityStatus, String> {
    match value {
        "operational" | "Operational" => Ok(CapabilityStatus::Operational),
        "planned" | "Planned" => Ok(CapabilityStatus::Planned),
        "blocked-by-foundation" | "BlockedByFoundation" => {
            Ok(CapabilityStatus::BlockedByFoundation)
        }
        "not-applicable" | "NotApplicable" => Ok(CapabilityStatus::NotApplicable),
        other => Err(format!("unknown capability status `{other}`")),
    }
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn bool_field(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn string_array_field(value: &Value, key: &str) -> Option<Vec<String>> {
    Some(
        value
            .get(key)?
            .as_array()?
            .iter()
            .map(Value::as_str)
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .map(ToOwned::to_owned)
            .collect(),
    )
}

fn active_artifact_contract_issues(report: &ValidationReport) -> Vec<ActiveArtifactContractIssue> {
    report
        .violations
        .iter()
        .map(|violation| ActiveArtifactContractIssue {
            artifact_id: violation.artifact_id.clone(),
            rule_id: violation.rule_id,
            severity: match violation.severity {
                Severity::Error => SeverityLabel::Error,
                Severity::Warn => SeverityLabel::Warn,
                Severity::Info => SeverityLabel::Info,
            },
            message: violation.message.clone(),
        })
        .collect()
}

fn format_issues<'a>(issues: impl Iterator<Item = &'a ActiveArtifactContractIssue>) -> String {
    issues
        .map(|issue| {
            format!(
                "{}: {} {}: {}",
                issue.severity.as_str(),
                issue.artifact_id,
                issue.rule_id,
                issue.message
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn git_ls_files() -> Result<BTreeSet<String>, String> {
    let output = Command::new("git")
        .args(["ls-files"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("git ls-files failed to spawn: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(format!("git ls-files exit {}: {stderr}", output.status));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("git ls-files output not UTF-8: {error}"))?;
    Ok(stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect())
}

fn normalize_repo_root_artifact_path(path: &str) -> String {
    path.strip_prefix('/').unwrap_or(path).to_string()
}

fn is_head_tracked_artifact_path(path: &str, head_tracked: &BTreeSet<String>) -> bool {
    head_tracked.contains(path)
        || head_tracked
            .iter()
            .any(|tracked| is_child_path(path, tracked))
}

fn is_child_path(parent: &str, child: &str) -> bool {
    let Some(rest) = child.strip_prefix(parent) else {
        return false;
    };
    rest.starts_with('/')
}

fn write_evidence_bundle(
    path: &Path,
    report: &ActiveArtifactContractReport,
    registry_path: &Path,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "evidence bundle dir unwriteable {}: {error}",
                parent.display()
            )
        })?;
    }

    let outcome = if report.error_count() == 0 {
        "success"
    } else {
        "failure"
    };
    let violations = report
        .violations
        .iter()
        .map(|violation| {
            json!({
                "artifact_id": &violation.artifact_id,
                "rule_id": violation.rule_id,
                "severity": violation.severity.as_str(),
                "message": &violation.message,
            })
        })
        .collect::<Vec<_>>();
    let body = json!({
        "$schema_ref": "/templates/evidence-bundle-template.json",
        "_artifact_id": "active-artifact-contract-lane-run",
        "_meta": {
            "emitter": "oya-dev-cli gate validate active-artifact-contract",
            "registry_path": registry_path.display().to_string()
        },
        "outcome": outcome,
        "rows_seen": report.rows_seen,
        "head_tracked_count": report.head_tracked_count,
        "untracked_paths": &report.untracked_paths,
        "duplicate_ids": &report.duplicate_ids,
        "error_count": report.error_count(),
        "warning_count": report.warning_count(),
        "violations": violations,
        "validation_duration_ms": report.validation_duration_ms,
        "graph_edge_count": report.graph_edges.len()
    });
    let body = format!(
        "{}\n",
        serde_json::to_string_pretty(&body)
            .map_err(|error| format!("evidence bundle JSON serialize failed: {error}"))?
    );
    fs::write(path, body)
        .map_err(|error| format!("evidence bundle write failed {}: {error}", path.display()))?;
    Ok(())
}

fn write_graph_edges(path: &Path, edges: &[(String, String, String)]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!("graph edges dir unwriteable {}: {error}", parent.display())
        })?;
    }
    let edges = edges
        .iter()
        .map(|(source, target, edge_type)| {
            json!({ "source": source, "target": target, "edge_type": edge_type })
        })
        .collect::<Vec<_>>();
    let body = format!(
        "{}\n",
        serde_json::to_string_pretty(&json!({
            "$schema_ref": "specs/knowledge-graph-schema.json",
            "_artifact_id": "active-artifact-contract-edges",
            "_meta": {
                "emitter": "oya-dev-cli gate validate active-artifact-contract",
                "layer": "semantic",
                "purpose": "Generated graph edges that connect active machine-readable artifacts to their declared schemas, registries, templates, and ledgers."
            },
            "edges": edges,
        }))
        .map_err(|error| format!("graph edges JSON serialize failed: {error}"))?
    );
    fs::write(path, body)
        .map_err(|error| format!("graph edges write failed {}: {error}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile_defaults() -> ProfileDefaults {
        load_artifact_profile_defaults(&profile_defaults_path()).unwrap()
    }

    #[test]
    fn parse_args_defaults() {
        let args = parse_active_artifact_contract_validate_args(vec![]).unwrap();
        assert_eq!(
            args.registry_path,
            PathBuf::from("registry/artifact-capabilities-registry.json")
        );
        assert!(args.emit_evidence_path.is_none());
        assert!(args.emit_graph_edges_path.is_none());
    }

    #[test]
    fn parse_args_full() {
        let args = parse_active_artifact_contract_validate_args(vec![
            "--registry".into(),
            "tests/fixtures/missing-row-registry.json".into(),
            "--emit-evidence".into(),
            "/tmp/evidence.json".into(),
            "--emit-graph-edges".into(),
            "/tmp/edges.json".into(),
        ])
        .unwrap();
        assert_eq!(
            args.registry_path.display().to_string(),
            "tests/fixtures/missing-row-registry.json"
        );
        assert_eq!(
            args.emit_evidence_path.unwrap().display().to_string(),
            "/tmp/evidence.json"
        );
        assert_eq!(
            args.emit_graph_edges_path.unwrap().display().to_string(),
            "/tmp/edges.json"
        );
    }

    #[test]
    fn parse_args_unknown_flag_errors() {
        let result = parse_active_artifact_contract_validate_args(vec!["--bogus".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_artifact_row_uses_profile_defaults() {
        let defaults = profile_defaults();
        let row = serde_json::json!({
            "artifact_id": "active-artifact-contract-spec",
            "artifact_path": "/specs/active-machine-readable-artifact-contract.json",
            "artifact_profile": "schema"
        });
        let parsed = parse_artifact_row(
            &row,
            0,
            Path::new("registry/artifact-capabilities-registry.json"),
            Some(&defaults),
        )
        .unwrap();
        assert_eq!(
            parsed.artifact_path,
            "specs/active-machine-readable-artifact-contract.json"
        );
        assert_eq!(parsed.capabilities.len(), 9);
    }

    #[test]
    fn graph_edge_emission_serializes_vertical_tab_as_valid_json() {
        let temp = std::env::temp_dir().join(format!(
            "active-artifact-contract-graph-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&temp);
        write_graph_edges(
            &temp,
            &[(
                "source\u{000b}".to_owned(),
                "target".to_owned(),
                "declares".to_owned(),
            )],
        )
        .expect("graph edges write");
        let emitted = fs::read_to_string(&temp).expect("graph edges read");
        let parsed: Value = serde_json::from_str(&emitted).expect("canonical graph JSON");
        assert_eq!(parsed["edges"][0]["source"], "source\u{000b}");
        let expected = format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
                "$schema_ref": "specs/knowledge-graph-schema.json",
                "_artifact_id": "active-artifact-contract-edges",
                "_meta": {
                    "emitter": "oya-dev-cli gate validate active-artifact-contract",
                    "layer": "semantic",
                    "purpose": "Generated graph edges that connect active machine-readable artifacts to their declared schemas, registries, templates, and ledgers."
                },
                "edges": [{
                    "source": "source\u{000b}",
                    "target": "target",
                    "edge_type": "declares"
                }]
            }))
            .expect("canonical graph projection serializes")
        );
        assert_eq!(emitted, expected);
        assert!(emitted.contains("\\u000b"));
        fs::remove_file(temp).ok();
    }

    #[test]
    fn parse_artifact_row_uses_extension_profile_defaults() {
        let defaults = profile_defaults();
        let row = serde_json::json!({
            "artifact_id": "kernel-profile",
            "artifact_path": "specs/active-machine-readable-artifact-contract.json",
            "artifact_profile": "kernel-crate"
        });
        let parsed = parse_artifact_row(
            &row,
            0,
            Path::new("registry/artifact-capabilities-registry.json"),
            Some(&defaults),
        )
        .unwrap();
        assert_eq!(parsed.capabilities.len(), 9);
    }

    #[test]
    fn parse_artifact_row_rejects_unknown_profile_without_capabilities() {
        let defaults = profile_defaults();
        let row = serde_json::json!({
            "artifact_id": "unknown-profile",
            "artifact_path": "specs/active-machine-readable-artifact-contract.json",
            "artifact_profile": "definitely-not-a-canonical-profile"
        });
        let result = parse_artifact_row(
            &row,
            0,
            Path::new("registry/artifact-capabilities-registry.json"),
            Some(&defaults),
        );
        assert!(result.is_err());
    }

    #[test]
    fn repo_root_artifact_paths_match_git_ls_files_shape() {
        assert_eq!(
            normalize_repo_root_artifact_path("/specs/masterplan.json"),
            "specs/masterplan.json"
        );
        assert_eq!(
            normalize_repo_root_artifact_path("registry/test-set-registry.json"),
            "registry/test-set-registry.json"
        );
    }

    #[test]
    fn directory_artifact_paths_are_covered_by_tracked_children() {
        let tracked = BTreeSet::from([
            "crates/oya-intelligence-settings-template-kernel/Cargo.toml".to_string(),
            "crates/oya-intelligence-settings-template-kernel/src/lib.rs".to_string(),
        ]);

        assert!(is_head_tracked_artifact_path(
            "crates/oya-intelligence-settings-template-kernel",
            &tracked
        ));
        assert!(!is_head_tracked_artifact_path(
            "crates/oya-intelligence-settings-template",
            &tracked
        ));
    }
}
